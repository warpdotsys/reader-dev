"""Reclaim only an owned, stopped historical test leaf, never global caches.

This is harness handoff cleanup, not a Reader memory-policy change or proof of
the cause of an earlier timeout. memory.reclaim may partially reclaim/EAGAIN:
https://docs.kernel.org/admin-guide/cgroup-v2.html#memory
"""
import errno
import os
from pathlib import Path
import re
import sys
import time

SYSTEM_SLICE = Path("/sys/fs/cgroup/system.slice")
LINUX = sys.platform.startswith("linux")
LIMITS = {
    "cpu.max": "200000 100000", "memory.max": "2147483648",
    "memory.high": "1610612736", "memory.swap.max": "0", "pids.max": "256",
}
MEMORY_KEYS = {"anon", "file", "shmem", "file_dirty", "file_writeback"}
EVENT_KEYS = {"high", "max", "oom", "oom_kill", "oom_group_kill"}


def read_value(path):
    if path.is_symlink() or not path.is_file():
        raise RuntimeError("Unexpected owned cgroup control file")
    value = path.read_text(encoding="ascii")
    if len(value) > 32768:
        raise RuntimeError("Oversized owned cgroup control")
    return value.strip()


def counters(path, expected):
    rows = {}
    for line in read_value(path).splitlines():
        key, value = line.split()
        if key in expected:
            if key in rows or not value.isdecimal():
                raise RuntimeError("Invalid owned cgroup counters")
            rows[key] = int(value)
    if set(rows) != expected:
        raise RuntimeError("Missing owned cgroup counters")
    return rows


def snapshot(root, child):
    limits = {key: read_value(root / key) for key in LIMITS}
    if limits != LIMITS or read_value(root / "cpuset.cpus.effective") != "0-1":
        raise RuntimeError("Exact aggregate test budget missing")
    if read_value(child / "cgroup.procs") or read_value(child / "cgroup.threads") or \
            read_value(child / "pids.current") != "0" or \
            counters(child / "cgroup.events", {"populated"})["populated"] != 0:
        raise RuntimeError("Historical cgroup is not independently empty")
    # No descendant may be hidden beneath the named stopped leaf.
    if any(path.is_dir() for path in child.iterdir()):
        raise RuntimeError("Historical cgroup is not a leaf")
    current = int(read_value(child / "memory.current"))
    if not 0 <= current <= int(LIMITS["memory.max"]):
        raise RuntimeError("Historical memory charge out of bounds")
    return {"parentMemoryCurrent": int(read_value(root / "memory.current")),
        "childMemoryCurrent": current, "childPidsCurrent": 0,
        "childMemoryStat": counters(child / "memory.stat", MEMORY_KEYS),
        "parentMemoryEvents": counters(root / "memory.events", EVENT_KEYS),
        "aggregateLimits": limits, "aggregateCpus": "0-1"}


def reclaim_stopped_reference(unit, runtime_state):
    if not LINUX or not isinstance(unit, str) or \
            re.fullmatch(r"reader-three-way-[a-z0-9-]{1,80}[.]service", unit) is None:
        raise RuntimeError("Not an owned Linux three-way test unit")
    expected_id = unit.removesuffix(".service") + "-server"
    if not isinstance(runtime_state, dict) or runtime_state.get("id") != expected_id or \
            runtime_state.get("status") != "stopped" or \
            type(runtime_state.get("pid")) is not int or runtime_state["pid"] != 0:
        raise RuntimeError("Owned historical runtime must be stopped with PID zero")
    root = SYSTEM_SLICE / unit
    child = root / "server"
    if root.is_symlink() or child.is_symlink() or not child.is_dir() or \
            root.resolve(strict=True).parent != SYSTEM_SLICE.resolve(strict=True) or \
            child.resolve(strict=True).parent != root.resolve(strict=True):
        raise RuntimeError("Owned cgroup path escaped its exact test unit")
    before = snapshot(root, child)
    request = before["childMemoryCurrent"]
    result = {"scope": "owned-stopped-historical-server-leaf-only",
        "unit": unit, "runtimeId": expected_id, "stoppedPidZeroObserved": True,
        "before": before, "requestedBytes": request,
        "globalCacheDropUsed": False, "diskFilesDeleted": False,
        "readerBudgetChanged": False, "earlierTimeoutCauseProven": False,
        "writeAttempted": False, "kernelPartialReclaimObserved": False}
    started = time.monotonic()
    if request:
        control = child / "memory.reclaim"
        if control.is_symlink() or not control.is_file():
            raise RuntimeError("Owned reclaim control absent")
        fd = os.open(control, os.O_WRONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            # Revalidate after opening; refuse live, moved, or relaxed targets.
            if snapshot(root, child)["childPidsCurrent"] != 0:
                raise RuntimeError("Stopped leaf identity changed")
            payload = (str(request) + " swappiness=0").encode("ascii")
            result["writeAttempted"] = True
            try:
                if os.write(fd, payload) != len(payload):
                    raise RuntimeError("Partial owned reclaim command write")
            except OSError as error:
                if error.errno != errno.EAGAIN:
                    raise
                result["kernelPartialReclaimObserved"] = True
        finally:
            os.close(fd)
    result["after"] = snapshot(root, child)
    result["seconds"] = round(time.monotonic() - started, 6)
    result["observedChildChargeReduction"] = before["childMemoryCurrent"] - result["after"]["childMemoryCurrent"]
    if result["after"]["aggregateLimits"] != before["aggregateLimits"]:
        raise RuntimeError("Aggregate test budget changed during reclaim")
    return result

