"""Generated thread-name fixtures only, not a JVM/Reader pressure test."""
import importlib.util
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("reader_thread_groups", ROOT / "scripts/probe-reader-thread-groups.py")
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


class ReaderThreadGroupsTest(unittest.TestCase):
    def test_linux_truncated_known_groups(self):
        for name, expected in (("vert.x-eventloo", "vertxEventLoop"), ("vert.x-interna", "vertxInternalWorker"),
                ("vert.x-worker-1", "vertxWorker"), ("DefaultDispatch", "coroutineDispatcher"),
                ("reader-browser-", "readerBrowser"), ("reader-camoufox", "readerBrowser"),
                ("OkHttp Dispatch", "okHttp"), ("ForkJoinPool.co", "forkJoin"),
                ("pool-1-thread-1", "scheduler"), ("scheduling-1", "scheduler"),
                ("GC Thread#0", "jvmRuntime"), ("java", "jvmRuntime"), ("G1 Main Marker", "jvmRuntime")):
            with self.subTest(name=name): self.assertEqual(PROBE.thread_group(name), expected)

    def test_unknown_or_generated_sensitive_name_is_not_exported(self):
        self.assertEqual(PROBE.thread_group("fake-secret"), "other")

    def test_name_budget_and_type_are_mandatory(self):
        for name in ("", None, 1, "a" * 65, "汉" * 22):
            with self.subTest(name=name), self.assertRaises(ValueError): PROBE.thread_group(name)

    def test_summary_counts_partition_observed_threads(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for number, name in enumerate(("java", "DefaultDispatch", "DefaultDispatch", "fake-secret"), 1):
                task = root / str(number); task.mkdir(); (task / "comm").write_bytes((name + "\n").encode())
            result = PROBE.summarize_threads(root)
            self.assertEqual(result["observedThreads"], 4)
            self.assertEqual(sum(result["groups"].values()), 4)
            self.assertEqual(result["groups"]["coroutineDispatcher"], 2)
            self.assertFalse(result["rawThreadNamesExported"])
            self.assertNotIn("fake-secret", str(result))

    def test_disappearing_task_is_not_falsely_counted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / "1").mkdir()
            result = PROBE.summarize_threads(root)
            self.assertEqual(result["observedThreads"], 0)
            self.assertEqual(result["disappearedDuringRead"], 1)

    def test_empty_inventory_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(RuntimeError):
            PROBE.summarize_threads(Path(directory))

    def test_non_numeric_task_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            (Path(directory) / "other").mkdir()
            with self.assertRaises(RuntimeError): PROBE.summarize_threads(Path(directory))

    def test_comm_byte_budget_is_not_truncated_into_success(self):
        with tempfile.TemporaryDirectory() as directory:
            task = Path(directory) / "1"; task.mkdir(); (task / "comm").write_bytes(b"a" * 65)
            with self.assertRaises(RuntimeError): PROBE.summarize_threads(Path(directory))

    def test_generated_unit_and_candidate_hash_accepted(self):
        PROBE.validate_inputs("reader-thread-profile-20261012-a.service", "e" * 64)

    def test_production_or_escaped_unit_rejected(self):
        for unit in ("reader.service", "reader-thread-profile-20261012-a.service/child", "../reader.service", None):
            with self.subTest(unit=unit), self.assertRaises(ValueError): PROBE.validate_inputs(unit, "e" * 64)

    def test_original_or_unverified_hash_rejected(self):
        for sha in (PROBE.ORIGINAL_SHA, "e" * 63, "E" * 64, "e" * 64 + "\n", None):
            with self.subTest(sha=sha), self.assertRaises(ValueError):
                PROBE.validate_inputs("reader-thread-profile-20261012-a.service", sha)

    def test_optimized_cli_is_rejected_before_arguments_or_proc(self):
        run = subprocess.run([sys.executable, "-O", str(ROOT / "scripts/probe-reader-thread-groups.py")],
            capture_output=True, text=True, timeout=10)
        self.assertNotEqual(run.returncode, 0)
        self.assertEqual(run.stdout, "")
        self.assertIn("Refuses disabled safety assertions", run.stderr)

    def test_generated_loopback_redirect_is_rejected(self):
        with self.assertRaises(RuntimeError): PROBE.NoRedirect().redirect_request(None)

    def test_startup_marker_export_does_not_include_raw_log(self):
        result = PROBE.startup_markers(b"fake-secret Read-only file system Application run failed")
        self.assertTrue(result["Read-only file system"])
        self.assertTrue(result["Application run failed"])
        self.assertFalse(result["OutOfMemoryError"])
        self.assertNotIn("fake-secret", str(result))

    def test_startup_log_type_and_byte_limit_are_mandatory(self):
        for raw in ("fake-secret", b"x" * (4 * 1024**2 + 1)):
            with self.assertRaises(ValueError): PROBE.startup_markers(raw)

    def test_reader_command_uses_packaged_camoufox_and_no_invalid_empty_cli_option(self):
        cmd = PROBE.reader_command(PurePosixPath("/verification/reader.jar"), PurePosixPath("/tmp/generated/store"))
        self.assertIn("--reader.app.webviewRenderer=camoufox", cmd)
        self.assertIn("--reader.app.camoufoxPythonExecutable=/usr/bin/python3", cmd)
        self.assertNotIn("--reader.app.inviteCode=", cmd)
        self.assertTrue(all(not arg.endswith("=") for arg in cmd))
        self.assertIn("--reader.app.workDir=/tmp/generated/store", cmd)
