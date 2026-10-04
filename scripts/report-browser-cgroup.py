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


def main():
    report = collect_report()
    print(json.dumps(report, sort_keys=True))
    verify_report(report)


if __name__ == "__main__":
    main()
