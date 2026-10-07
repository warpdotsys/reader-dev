"""Record the full-image smoke container's cgroup v2 resource budget."""

import json
from pathlib import Path
import platform


ROOT = Path("/sys/fs/cgroup")
MAX_MEMORY_STAT_BYTES = 64 * 1024
# Current, overlapping kernel categories: not a reconstruction of peak RSS.
MEMORY_STAT_FIELDS = (
    "anon", "file", "shmem", "kernel", "kernel_stack", "pagetables", "sock",
    "file_mapped", "file_dirty", "file_writeback", "inactive_file", "active_file",
    "slab_reclaimable", "slab_unreclaimable",
)


def number(name, optional=False):
    path = ROOT / name
    if optional and not path.is_file():
        return None
    value = path.read_text().strip()
    return value if value == "max" else int(value)


def counters(name):
    return {key: int(value) for key, value in
            (line.split() for line in (ROOT / name).read_text().splitlines())}


def memory_stat_bytes():
    path = ROOT / "memory.stat"
    if not path.is_file():
        return None
    with path.open("rb") as stream:
        raw = stream.read(MAX_MEMORY_STAT_BYTES + 1)
    if len(raw) > MAX_MEMORY_STAT_BYTES:
        raise ValueError("Memory category observation exceeds its byte limit")
    try:
        lines = raw.decode("ascii").splitlines()
    except UnicodeDecodeError:
        raise ValueError("Invalid memory category observation") from None
    values = {}
    for line in lines:
        parts = line.split()
        if not parts or parts[0] not in MEMORY_STAT_FIELDS:
            continue
        if (len(parts) != 2 or parts[0] in values or not 1 <= len(parts[1]) <= 20 or
                not parts[1].isascii() or not parts[1].isdecimal()):
            raise ValueError("Invalid memory category observation")
        values[parts[0]] = int(parts[1])
    # Missing kernel-version-dependent categories stay unknown, never zero.
    return {name: values.get(name) for name in MEMORY_STAT_FIELDS}


def collect_report(scope="one full Reader image; sequential probes plus four-account burst"):
    quota, period = (ROOT / "cpu.max").read_text().split()
    report = {
        "scope": scope,
        "kernel": platform.release(),
        "architecture": platform.machine(),
        "memoryPeakBytes": number("memory.peak"),
        "memoryCurrentBytes": number("memory.current", optional=True),
        "memoryMaxBytes": number("memory.max"),
        "memoryEvents": counters("memory.events"),
        "memoryStatBytes": memory_stat_bytes(),
        "swapCurrentBytes": number("memory.swap.current"),
        "swapMaxBytes": number("memory.swap.max"),
        "pidsPeak": number("pids.peak", optional=True),
        "pidsCurrent": number("pids.current"),
        "pidsMax": number("pids.max"),
        "pidsEvents": counters("pids.events"),
        "cpuQuota": quota,
        "cpuPeriod": int(period),
    }
    return report


def verify_report(report, require_no_swap=False):
    quota, period = report["cpuQuota"], report["cpuPeriod"]
    if (report["memoryMaxBytes"] != 2 * 1024 ** 3 or
            report["memoryPeakBytes"] > report["memoryMaxBytes"] or
            report["pidsMax"] != 256 or quota == "max" or int(quota) != 2 * int(period) or
            "max" not in report["memoryEvents"] or
            any(report["memoryEvents"].get(key, 0) for key in ("max", "oom", "oom_kill", "oom_group_kill")) or
            "max" not in report["pidsEvents"] or report["pidsEvents"]["max"] or
            (require_no_swap and (report["swapMaxBytes"] != 0 or report["swapCurrentBytes"] != 0))):
        raise SystemExit("Bundled-browser smoke exceeded or did not use its configured resource budget")
    categories = report.get("memoryStatBytes")
    if categories is not None and (not isinstance(categories, dict) or
            set(categories) != set(MEMORY_STAT_FIELDS) or
            any(value is not None and (type(value) is not int or value < 0)
                for value in categories.values())):
        raise SystemExit("Invalid bounded memory category report")


def main():
    report = collect_report()
    print(json.dumps(report, sort_keys=True))
    verify_report(report)


if __name__ == "__main__":
    main()
