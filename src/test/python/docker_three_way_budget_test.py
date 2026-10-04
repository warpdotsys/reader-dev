"""Generated cgroup-file validation; these tests never run Docker or Java."""
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[3] / "scripts/run-three-way-webview-in-docker.py"
SPEC = importlib.util.spec_from_file_location("three_way_budget", SCRIPT)
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


class DockerThreeWayBudgetTest(unittest.TestCase):
    def files(self, directory):
        values = {"cpu.max": "200000 100000", "memory.max": "2147483648",
                  "memory.peak": "1000000000", "memory.events": "max 0\noom 0\noom_kill 0\n",
                  "memory.swap.max": "0", "pids.max": "512", "pids.events": "max 0\n"}
        for name, value in values.items():
            (directory / name).write_text(value)

    def test_aggregate_budget_uses_actual_files_not_expected_values(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.files(directory)
            report = PROBE.budget_snapshot(directory)
            self.assertEqual(1000000000, report["memoryPeakBytes"])
            self.assertEqual(2147483648, report["memoryMaxBytes"])
            self.assertEqual(200000, report["cpuQuota"])
            self.assertEqual(0, report["swapMaxBytes"])
            self.assertEqual(512, report["pidsMax"])

    def test_relaxed_budget_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for name, value in (("cpu.max", "400000 100000"), ("memory.max", "4294967296"),
                                ("memory.swap.max", "2147483648"), ("pids.max", "1024")):
                with self.subTest(name=name):
                    self.files(directory)
                    (directory / name).write_text(value)
                    with self.assertRaisesRegex(RuntimeError, "aggregate resource budget"):
                        PROBE.budget_snapshot(directory)

    def test_reclaim_pressure_is_not_misreported_as_an_unpressured_pass(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.files(directory)
            clean = PROBE.budget_snapshot(directory)
            PROBE.require_unpressured_budget(clean)
            for value in ("max 4007\noom 0\noom_kill 0\n", "max 0\noom 1\noom_kill 1\n"):
                (directory / "memory.events").write_text(value)
                with self.assertRaisesRegex(RuntimeError, "aggregate resource limit"):
                    PROBE.require_unpressured_budget(PROBE.budget_snapshot(directory))

    def test_provenance_cannot_overwrite_an_existing_report(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "provenance.json"
            PROBE.write_new(report, {"generated": True})
            with self.assertRaises(FileExistsError):
                PROBE.write_new(report, {"generated": False})
            self.assertIs(True, json.loads(report.read_text())["generated"])

    def test_regular_host_refuses_before_reading_inputs_or_making_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "must-not-exist"
            result = subprocess.run([sys.executable, "-B", str(SCRIPT), "--original", "absent",
                                     "--restored", "absent", "--runtime-image", "sha256:" + "a" * 64,
                                     "--output", str(output)], capture_output=True, text=True,
                                    timeout=10, check=False)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("root-owned Linux Docker/systemd host", result.stderr)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
