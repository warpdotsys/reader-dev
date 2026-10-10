"""Reap adopted zombies in a guarded UID-10001 fixture PID namespace only.

This is diagnostic init support, not a patch to the historical renderer. The
caller must independently establish the private namespace and aggregate budget.
It never signals a live process and must not consume Popen's direct child.
Compatible with the historical fixture's Python 3.8 runtime.
"""
import os
import sys
from pathlib import Path

PROC = Path("/proc")


def reap_adopted_zombies(excluded_direct_pid):
    if not sys.platform.startswith("linux") or os.getpid() != 1:
        raise RuntimeError("Guarded reaping requires Linux PID 1")
    if os.getuid() != 10001 or os.getgid() != 10001 or os.getgroups() != []:
        raise RuntimeError("Guarded reaping requires the non-root fixture identity")
    if type(excluded_direct_pid) is not int or excluded_direct_pid <= 1:
        raise ValueError("A direct Popen child PID must be explicitly excluded")
    reaped = []
    for entry in PROC.iterdir():
        if not entry.name.isascii() or not entry.name.isdecimal():
            continue
        pid = int(entry.name)
        if pid <= 1 or pid == excluded_direct_pid:
            continue
        if entry.is_symlink():
            raise RuntimeError("Unexpected numeric proc symlink")
        try:
            raw = (entry / "status").read_bytes()
            if len(raw) > 65536:
                raise RuntimeError("Unexpected proc status size")
            fields = dict(line.split(b":", 1) for line in raw.splitlines() if b":" in line)
            parent = int(fields[b"PPid"].strip())
            state = fields[b"State"].strip().split()[0]
            if parent != 1 or state != b"Z":
                continue
            actual_pid, status = os.waitpid(pid, os.WNOHANG)
            if actual_pid:
                if actual_pid != pid:
                    raise RuntimeError("Unexpected reaped child PID")
                reaped.append((actual_pid, status))
        except (FileNotFoundError, ProcessLookupError, ChildProcessError):
            # A child can disappear or already have been reaped during enumeration.
            continue
    return reaped
