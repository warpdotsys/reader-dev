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
            "memory.events": "high 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0",
            "memory.stat": "anon 300000000\nfile 450000000\nshmem 12000000\n"
                           "kernel_stack 1048576\npagetables 2097152\nfuture_counter 987654\n",
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

    def test_memory_categories_are_current_raw_bytes_with_a_finite_allowlist(self):
        report = self.run_report()
        self.assertEqual(set(REPORT.MEMORY_STAT_FIELDS), set(report["memoryStatBytes"]))
        self.assertEqual(300000000, report["memoryStatBytes"]["anon"])
        self.assertEqual(450000000, report["memoryStatBytes"]["file"])
        self.assertEqual(12000000, report["memoryStatBytes"]["shmem"])
        self.assertEqual(1048576, report["memoryStatBytes"]["kernel_stack"])
        self.assertEqual(2097152, report["memoryStatBytes"]["pagetables"])
        self.assertNotIn("future_counter", report["memoryStatBytes"])
        self.assertIsNone(report["memoryStatBytes"]["kernel"])

    def test_missing_category_file_is_unknown_not_zero_or_an_invented_breakdown(self):
        self.assertIsNone(self.run_report({"memory.stat": None})["memoryStatBytes"])

    def test_optional_categories_do_not_retroactively_reject_historical_reports(self):
        report = self.run_report()
        del report["memoryStatBytes"]
        REPORT.verify_report(report)

    def test_cache_categories_never_subtract_from_the_total_resource_guard(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.peak": "2147483649",
                "memory.stat": "anon 1\nfile 2147483648\nshmem 0\n"})
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.events": "max 1\noom 0\noom_kill 0\n",
                "memory.stat": "anon 1\nfile 2147483647\nshmem 0\n"})

    def test_invalid_or_duplicate_known_categories_are_rejected_without_raw_values(self):
        for raw in ("anon -1", "anon 1.0", "anon invalid", "anon 1\nanon 2",
                    "anon 1 extra", "anon " + "9" * 21):
            with self.subTest(raw=raw), self.assertRaisesRegex(ValueError, "Invalid memory category observation"):
                self.run_report({"memory.stat": raw})

    def test_oversized_category_file_is_rejected_before_json_output(self):
        with self.assertRaisesRegex(ValueError, "byte limit"):
            self.run_report({"memory.stat": "x" * (REPORT.MAX_MEMORY_STAT_BYTES + 1)})

    def test_invalid_observed_category_types_cannot_be_forged_as_raw_bytes(self):
        report = self.run_report()
        for value in (True, -1, "0", 1.0):
            with self.subTest(value=value):
                report["memoryStatBytes"]["anon"] = value
                with self.assertRaisesRegex(SystemExit, "memory category report"):
                    REPORT.verify_report(report)
        report["memoryStatBytes"] = {"unexpected": 0}
        with self.assertRaisesRegex(SystemExit, "memory category report"):
            REPORT.verify_report(report)

    def test_oom_is_failure_even_when_final_memory_is_within_budget(self):
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            self.run_report({"memory.events": "max 0\noom 1\noom_kill 1\noom_group_kill 0"})

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

    def test_memory_high_is_observed_not_invented_for_older_reports(self):
        self.assertIsNone(self.run_report()["memoryHighBytes"])
        report = self.run_report({"memory.high": "1610612736"})
        self.assertEqual(1610612736, report["memoryHighBytes"])
        REPORT.verify_report(report, expected_memory_high=1610612736)

    def test_explicit_high_policy_requires_actual_matching_limit(self):
        for value in (None, "max", 0, True, "1610612736", 2147483648):
            with self.subTest(value=value):
                report = self.run_report()
                report["memoryHighBytes"] = value
                with self.assertRaisesRegex(SystemExit, "memory high"):
                    REPORT.verify_report(report, expected_memory_high=1610612736)

    def test_high_pressure_is_visible_and_cannot_mask_hard_pressure_or_swap(self):
        report = self.run_report({"memory.high": "1610612736", "memory.swap.max": "0",
            "memory.events": "high 2893\nmax 0\noom 0\noom_kill 0\noom_group_kill 0"})
        REPORT.verify_report(report, require_no_swap=True, expected_memory_high=1610612736)
        self.assertEqual(2893, report["memoryEvents"]["high"])
        for key in ("max", "oom", "oom_kill", "oom_group_kill"):
            report["memoryEvents"][key] = 1
            with self.subTest(key=key), self.assertRaisesRegex(SystemExit, "resource budget"):
                REPORT.verify_report(report, require_no_swap=True, expected_memory_high=1610612736)
            report["memoryEvents"][key] = 0
        report["swapCurrentBytes"] = 1
        with self.assertRaisesRegex(SystemExit, "resource budget"):
            REPORT.verify_report(report, require_no_swap=True, expected_memory_high=1610612736)

    def test_invalid_high_values_and_invalid_expectations_are_not_normalized(self):
        report = self.run_report()
        for value in (False, -1, 1.5, "0", {}):
            report["memoryHighBytes"] = value
            with self.subTest(value=value), self.assertRaisesRegex(SystemExit, "memory high"):
                REPORT.verify_report(report)
        report["memoryHighBytes"] = 1610612736
        for expected in (True, -1, "1610612736", 2147483648):
            with self.subTest(expected=expected), self.assertRaisesRegex(SystemExit, "memory high"):
                REPORT.verify_report(report, expected_memory_high=expected)


if __name__ == "__main__":
    unittest.main()
