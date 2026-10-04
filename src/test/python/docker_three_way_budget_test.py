"""Generated cgroup-file validation; these tests never run Docker or Java."""
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).resolve().parents[3] / "scripts/run-three-way-webview-in-docker.py"
SPEC = importlib.util.spec_from_file_location("three_way_budget", SCRIPT)
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


class DockerThreeWayBudgetTest(unittest.TestCase):
    def files(self, directory):
        values = {"cpu.max": "200000 100000", "memory.max": "2147483648",
                  "memory.peak": "1000000000", "memory.events": "max 0\noom 0\noom_kill 0\n",
                  "memory.swap.max": "0", "pids.max": "512", "pids.events": "max 0\n",
                  "memory.current": "900000000", "pids.current": "42",
                  "memory.stat": "anon 300000000\nfile 550000000\nshmem 12000000\n"}
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

    def test_sample_records_actual_memory_categories_and_phase(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.files(directory)
            with mock.patch.object(PROBE.time, "monotonic", return_value=2):
                sample = PROBE.resource_sample(directory, "generated-phase", 1)
            self.assertEqual("generated-phase", sample["phase"])
            self.assertEqual(1000, sample["elapsedMs"])
            self.assertEqual(900000000, sample["memoryCurrentBytes"])
            self.assertEqual(300000000, sample["memoryAnonBytes"])
            self.assertEqual(550000000, sample["memoryFileBytes"])
            self.assertEqual(12000000, sample["memoryShmemBytes"])
            self.assertEqual(42, sample["pidsCurrent"])

    def test_wrong_container_ownership_is_rejected_before_mutation(self):
        state = [{"Config": {"Labels": {"com.medwarp.reader.generated-test": "another-run"}}}]
        with mock.patch.object(PROBE, "command", return_value=
                               subprocess.CompletedProcess([], 0, stdout=json.dumps(state))) as command:
            with self.assertRaisesRegex(RuntimeError, "ownership label"):
                PROBE.owned_container("generated-cid", "this-run")
            command.assert_called_once_with("docker", "inspect", "generated-cid")

    def test_host_ack_is_written_only_after_handoff_returns(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            phases = directory / "phases"
            phases.mkdir()
            (phases / "remote-complete").touch()
            process = mock.Mock(returncode=0)
            process.poll.side_effect = [None, 0, 0]
            handoff = mock.Mock(side_effect=lambda: self.assertFalse(
                (phases / "camoufox-permitted").exists()))
            sample = mock.Mock()
            with mock.patch.object(PROBE.subprocess, "Popen", return_value=process), \
                    mock.patch.object(PROBE.time, "sleep"):
                self.assertEqual(0, PROBE.run_probe_with_handoff(
                    ["generated-only"], directory / "probe.log", phases, handoff, sample))
            handoff.assert_called_once_with()
            sample.assert_called_once_with("camoufox")
            self.assertEqual(b"", (phases / "camoufox-permitted").read_bytes())
            process.terminate.assert_not_called()

    def test_failed_handoff_does_not_acknowledge_and_terminates_client(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            phases = directory / "phases"
            phases.mkdir()
            (phases / "remote-complete").touch()
            process = mock.Mock(returncode=None)
            process.poll.return_value = None
            with mock.patch.object(PROBE.subprocess, "Popen", return_value=process):
                with self.assertRaisesRegex(RuntimeError, "generated handoff failed"):
                    PROBE.run_probe_with_handoff(["generated-only"], directory / "probe.log", phases,
                        mock.Mock(side_effect=RuntimeError("generated handoff failed")), mock.Mock())
            self.assertFalse((phases / "camoufox-permitted").exists())
            process.terminate.assert_called_once_with()
            process.wait.assert_called_once_with(timeout=5)

    def test_zero_exit_without_handoff_is_not_accepted(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            process = mock.Mock(returncode=0)
            process.poll.return_value = 0
            with mock.patch.object(PROBE.subprocess, "Popen", return_value=process):
                with self.assertRaisesRegex(RuntimeError, "without the renderer handoff"):
                    PROBE.run_probe_with_handoff(["generated-only"], directory / "probe.log",
                                                directory / "phases", mock.Mock(), mock.Mock())

    def test_nonempty_completion_marker_never_stops_renderer(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            phases = directory / "phases"
            phases.mkdir()
            (phases / "remote-complete").write_text("not-complete")
            process = mock.Mock(returncode=None)
            process.poll.return_value = None
            handoff = mock.Mock()
            with mock.patch.object(PROBE.subprocess, "Popen", return_value=process):
                with self.assertRaisesRegex(RuntimeError, "Invalid completed historical"):
                    PROBE.run_probe_with_handoff(["generated-only"], directory / "probe.log",
                                                phases, handoff, mock.Mock())
            handoff.assert_not_called()
            self.assertFalse((phases / "camoufox-permitted").exists())

    def test_timeout_terminates_client_without_acknowledging(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            process = mock.Mock(returncode=None)
            process.poll.return_value = None
            handoff = mock.Mock()
            with mock.patch.object(PROBE.subprocess, "Popen", return_value=process):
                with self.assertRaisesRegex(TimeoutError, "bounded runtime"):
                    PROBE.run_probe_with_handoff(["generated-only"], directory / "probe.log",
                                                directory / "phases", handoff, mock.Mock(), timeout=0)
            handoff.assert_not_called()
            process.terminate.assert_called_once_with()

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
