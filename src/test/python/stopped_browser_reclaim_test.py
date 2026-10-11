"""Fake-file guard tests; these do not claim execution of kernel reclaim."""
import errno
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("stopped_browser_reclaim",
    REPO / "scripts/reclaim-stopped-browser-cgroup.py")
GUARD = importlib.util.module_from_spec(spec)
spec.loader.exec_module(GUARD)


class StoppedBrowserReclaimTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.parent = Path(self.temporary.name)
        self.unit = "reader-three-way-metadata-test.service"
        self.root = self.parent / self.unit
        self.child = self.root / "server"
        self.child.mkdir(parents=True)
        for key, value in GUARD.LIMITS.items():
            (self.root / key).write_text(value, encoding="ascii")
        (self.root / "cpuset.cpus.effective").write_text("0-1", encoding="ascii")
        (self.root / "memory.current").write_text("900000000", encoding="ascii")
        (self.root / "memory.events").write_text("high 3\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n", encoding="ascii")
        for key, value in {"cgroup.procs": "", "cgroup.threads": "", "pids.current": "0",
                "cgroup.events": "populated 0\nfrozen 0", "memory.current": "500000000",
                "memory.stat": "anon 0\nfile 499000000\nshmem 0\nfile_dirty 0\nfile_writeback 0\n",
                "memory.reclaim": ""}.items():
            (self.child / key).write_text(value, encoding="ascii")
        self.state = {"id": self.unit.removesuffix(".service") + "-server", "status": "stopped", "pid": 0}
        self.patch_parent = patch.object(GUARD, "SYSTEM_SLICE", self.parent)
        self.patch_linux = patch.object(GUARD, "LINUX", True)
        self.patch_parent.start()
        self.patch_linux.start()

    def tearDown(self):
        self.patch_linux.stop()
        self.patch_parent.stop()
        self.temporary.cleanup()

    def run_guard(self):
        return GUARD.reclaim_stopped_reference(self.unit, self.state)

    def rejected_without_write(self):
        with self.assertRaises(RuntimeError):
            self.run_guard()
        self.assertEqual((self.child / "memory.reclaim").read_text(), "")

    def test_only_exact_stopped_leaf_accepts_one_bounded_command(self):
        value = self.run_guard()
        self.assertEqual((self.child / "memory.reclaim").read_text(), "500000000 swappiness=0")
        self.assertTrue(value["writeAttempted"])
        self.assertEqual(value["requestedBytes"], 500000000)
        self.assertFalse(value["earlierTimeoutCauseProven"])
        self.assertFalse(value["readerBudgetChanged"])
        self.assertFalse(value["diskFilesDeleted"])
        self.assertFalse(value["globalCacheDropUsed"])

    def test_running_runtime_is_rejected_before_write(self):
        self.state.update(status="running", pid=123)
        self.rejected_without_write()

    def test_nonzero_or_boolean_pid_is_rejected(self):
        for pid in (1, False):
            with self.subTest(pid=pid):
                self.state["pid"] = pid
                self.rejected_without_write()

    def test_other_owned_id_is_rejected(self):
        self.state["id"] = "reader-three-way-other-server"
        self.rejected_without_write()

    def test_broad_or_path_traversal_units_are_rejected(self):
        for unit in ("system.slice", "../system.slice", "reader.service", "reader-three-way-../other.service"):
            with self.subTest(unit=unit):
                with self.assertRaises(RuntimeError):
                    GUARD.reclaim_stopped_reference(unit, self.state)

    def test_live_task_or_populated_subtree_is_rejected(self):
        for key, bad in (("cgroup.procs", "123"), ("cgroup.threads", "123"),
                         ("pids.current", "1"), ("cgroup.events", "populated 1")):
            with self.subTest(key=key):
                original = (self.child / key).read_text()
                (self.child / key).write_text(bad)
                self.rejected_without_write()
                (self.child / key).write_text(original)

    def test_nested_child_is_rejected(self):
        (self.child / "unexpected").mkdir()
        self.rejected_without_write()

    def test_every_relaxed_budget_is_rejected(self):
        for key in GUARD.LIMITS:
            with self.subTest(key=key):
                original = (self.root / key).read_text()
                (self.root / key).write_text("max")
                self.rejected_without_write()
                (self.root / key).write_text(original)

    def test_other_affinity_is_rejected(self):
        (self.root / "cpuset.cpus.effective").write_text("0-3")
        self.rejected_without_write()

    def test_missing_or_duplicate_counters_are_rejected(self):
        for value in ("anon 0\nfile 1", "anon 0\nanon 0\nfile 1\nshmem 0\nfile_dirty 0\nfile_writeback 0"):
            with self.subTest(value=value):
                (self.child / "memory.stat").write_text(value)
                self.rejected_without_write()

    def test_no_memory_charge_does_not_write(self):
        (self.child / "memory.current").write_text("0")
        value = self.run_guard()
        self.assertFalse(value["writeAttempted"])
        self.assertEqual((self.child / "memory.reclaim").read_text(), "")

    def test_eagain_is_recorded_as_partial_not_full_reclaim(self):
        with patch.object(GUARD.os, "write", side_effect=OSError(errno.EAGAIN, "partial")):
            value = self.run_guard()
        self.assertTrue(value["kernelPartialReclaimObserved"])
        self.assertEqual(value["observedChildChargeReduction"], 0)

    def test_other_write_error_is_not_silently_accepted(self):
        with patch.object(GUARD.os, "write", side_effect=OSError(errno.EPERM, "denied")):
            with self.assertRaises(OSError):
                self.run_guard()

    def test_non_linux_is_rejected(self):
        with patch.object(GUARD, "LINUX", False):
            self.rejected_without_write()

    def test_symlinked_unit_or_leaf_is_rejected(self):
        for target in (self.root, self.child):
            with self.subTest(target=target), patch.object(GUARD.Path, "is_symlink", autospec=True,
                    side_effect=lambda path: path == target):
                self.rejected_without_write()

    def test_symlinked_reclaim_control_is_rejected(self):
        target = self.child / "memory.reclaim"
        with patch.object(GUARD.Path, "is_symlink", autospec=True, side_effect=lambda path: path == target):
            self.rejected_without_write()

    def test_task_appearing_after_open_is_rejected_before_command(self):
        original_open = GUARD.os.open
        def appeared(path, flags):
            fd = original_open(path, flags)
            (self.child / "pids.current").write_text("1")
            return fd
        with patch.object(GUARD.os, "open", side_effect=appeared):
            self.rejected_without_write()

    def test_budget_change_after_open_is_rejected_before_command(self):
        original_open = GUARD.os.open
        def relaxed(path, flags):
            fd = original_open(path, flags)
            (self.root / "memory.max").write_text("max")
            return fd
        with patch.object(GUARD.os, "open", side_effect=relaxed):
            self.rejected_without_write()


if __name__ == "__main__":
    unittest.main()
