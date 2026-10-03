"""Record the full-image smoke container's cgroup v2 resource budget."""

import json
from pathlib import Path
import platform


ROOT = Path("/sys/fs/cgroup")


def number(name, optional=False):
    path = ROOT / name
    if optional and not path.is_file():
        return None
    value = path.read_text().strip()
    return value if value == "max" else int(value)


def counters(name):
    return {key: int(value) for key, value in
            (line.split() for line in (ROOT / name).read_text().splitlines())}


def main():
    quota, period = (ROOT / "cpu.max").read_text().split()
    report = {
        "scope": "one full Reader image; sequential probes plus four-account burst",
        "kernel": platform.release(),
        "memoryPeakBytes": number("memory.peak"),
        "memoryMaxBytes": number("memory.max"),
        "memoryEvents": counters("memory.events"),
        "swapCurrentBytes": number("memory.swap.current"),
        "swapMaxBytes": number("memory.swap.max"),
        "pidsPeak": number("pids.peak", optional=True),
        "pidsCurrent": number("pids.current"),
        "pidsMax": number("pids.max"),
        "pidsEvents": counters("pids.events"),
        "cpuQuota": quota,
        "cpuPeriod": int(period),
    }
    print(json.dumps(report, sort_keys=True))
    if (report["memoryMaxBytes"] != 2 * 1024 ** 3 or
            report["memoryPeakBytes"] > report["memoryMaxBytes"] or
            report["pidsMax"] != 256 or quota == "max" or int(quota) != 2 * int(period) or
            any(report["memoryEvents"].get(key, 0) for key in ("oom", "oom_kill", "oom_group_kill")) or
            report["pidsEvents"].get("max", 0)):
        raise SystemExit("Bundled-browser smoke exceeded or did not use its configured resource budget")


if __name__ == "__main__":
    main()
