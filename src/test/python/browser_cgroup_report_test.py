"""The smoke budget must fail closed; its report describes native architecture."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "browser_cgroup_report", Path(__file__).resolve().parents[3] / "scripts/report-browser-cgroup.py")
REPORT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REPORT)


class BrowserCgroupReportTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.values = {
            "cpu.max": "200000 100000",
            "memory.peak": "832761856",
            "memory.max": "2147483648",
            "memory.events": "max 0\noom 0\noom_kill 0\noom_group_kill 0",
            "memory.swap.current": "0",
            "memory.swap.max": "1073741824",
            "pids.peak": "199",
            "pids.current": "38",
            "pids.max": "256",
            "pids.events": "max 0",
        }
        self.addCleanup(self.temp.cleanup)

    def run_report(self, changes=None):
        for name, value in dict(self.values, **(changes or {})).items():
            if value is not None:
                (self.root / name).write_text(value, encoding="ascii")
        output = io.StringIO()
        with patch.object(REPORT, "ROOT", self.root), patch.object(
                REPORT.platform, "machine", return_value="aarch64"), contextlib.redirect_stdout(output):
            REPORT.main()
        return json.loads(output.getvalue())

    def test_bounded_arm64_report_has_raw_counters_and_architecture(self):
        report = self.run_report()
        self.assertEqual("aarch64", report["architecture"])
        self.assertEqual(832761856, report["memoryPeakBytes"])
        self.assertEqual(199, report["pidsPeak"])
        self.assertEqual(0, report["memoryEvents"]["oom_kill"])

    def test_missing_optional_pids_peak_is_not_invented(self):
        self.assertIsNone(self.run_report({"pids.peak": None})["pidsPeak"])

    def test_oom_is_failure_even_when_final_memory_is_within_budget(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.events": "oom 1\noom_kill 1\noom_group_kill 0"})

    def test_pids_limit_event_is_failure(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"pids.events": "max 1"})

    def test_memory_limit_pressure_is_failure_without_an_oom(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.events": "max 7\noom 0\noom_kill 0\noom_group_kill 0"})

    def test_missing_memory_pressure_counter_fails_closed(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.events": "oom 0\noom_kill 0"})

    def test_soak_must_enforce_zero_swap_not_only_observe_it(self):
        report = self.run_report()
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            REPORT.verify_report(report, require_no_swap=True)
        report["swapMaxBytes"] = 0
        REPORT.verify_report(report, require_no_swap=True)
        report["swapCurrentBytes"] = 1
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            REPORT.verify_report(report, require_no_swap=True)

    def test_unbounded_memory_is_failure(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.max": "max"})

    def test_wrong_cpu_quota_is_failure(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"cpu.max": "300000 100000"})


if __name__ == "__main__":
    unittest.main()
