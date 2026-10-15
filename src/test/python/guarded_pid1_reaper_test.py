"""Generated proc fixtures and mocked waitpid; never signal or reap host children."""
import contextlib
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("guarded_pid1_reaper", ROOT / "scripts/guarded_pid1_reaper.py")
REAPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REAPER)


class GuardedPid1ReaperTest(unittest.TestCase):
    @contextlib.contextmanager
    def guarded(self, root):
        with mock.patch.object(REAPER, "PROC", root), \
                mock.patch.object(REAPER.sys, "platform", "linux"), \
                mock.patch.object(REAPER.os, "getpid", return_value=1), \
                mock.patch.object(REAPER.os, "getuid", return_value=10001, create=True), \
                mock.patch.object(REAPER.os, "getgid", return_value=10001, create=True), \
                mock.patch.object(REAPER.os, "getgroups", return_value=[], create=True), \
                mock.patch.object(REAPER.os, "WNOHANG", 1, create=True):
            yield

    def proc(self, root, pid, state="Z", parent=1):
        directory = root / str(pid)
        directory.mkdir()
        (directory / "status").write_bytes(("Name:\tgenerated\nState:\t%s\nPPid:\t%d\n" % (state, parent)).encode("ascii"))
        return directory

    def test_only_adopted_zombies_are_reaped_and_direct_child_is_excluded(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for pid, state, parent in [(1, "Z", 1), (6, "Z", 1), (20, "Z", 1),
                    (21, "S", 1), (22, "R", 1), (23, "Z", 6)]:
                self.proc(root, pid, state, parent)
            (root / "self").mkdir()
            with self.guarded(root), mock.patch.object(REAPER.os, "waitpid", return_value=(20, 0), create=True) as wait:
                self.assertEqual([(20, 0)], REAPER.reap_adopted_zombies(6))
                wait.assert_called_once_with(20, os.WNOHANG)

    def test_non_pid1_refused_before_proc_read(self):
        with self.guarded(Path("generated-never-read")), \
                mock.patch.object(REAPER.os, "getpid", return_value=99), \
                mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
            with self.assertRaisesRegex(RuntimeError, "PID 1"):
                REAPER.reap_adopted_zombies(6)
            wait.assert_not_called()

    def test_non_linux_refused_before_proc_read(self):
        with self.guarded(Path("generated-never-read")), \
                mock.patch.object(REAPER.sys, "platform", "win32"), \
                mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
            with self.assertRaisesRegex(RuntimeError, "Linux"):
                REAPER.reap_adopted_zombies(6)
            wait.assert_not_called()

    def test_root_wrong_gid_and_extra_groups_refused(self):
        for field, value in (("getuid", 0), ("getgid", 1000), ("getgroups", [10001])):
            with self.subTest(field=field), self.guarded(Path("generated-never-read")), \
                    mock.patch.object(REAPER.os, field, return_value=value, create=True), \
                    mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
                with self.assertRaisesRegex(RuntimeError, "non-root"):
                    REAPER.reap_adopted_zombies(6)
                wait.assert_not_called()

    def test_missing_or_invalid_direct_child_exclusion_refused(self):
        for pid in (None, True, "6", -1, 0, 1):
            with self.subTest(pid=pid), self.guarded(Path("generated-never-read")), \
                    mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
                with self.assertRaises(ValueError):
                    REAPER.reap_adopted_zombies(pid)
                wait.assert_not_called()

    def test_disappeared_or_already_reaped_children_are_not_failures(self):
        for failure in (FileNotFoundError(), ProcessLookupError(), ChildProcessError()):
            with self.subTest(failure=type(failure).__name__), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.proc(root, 20)
                with self.guarded(root), mock.patch.object(REAPER.os, "waitpid", side_effect=failure, create=True):
                    self.assertEqual([], REAPER.reap_adopted_zombies(6))

    def test_not_ready_waitpid_does_not_claim_reaping(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.proc(root, 20)
            with self.guarded(root), mock.patch.object(REAPER.os, "waitpid", return_value=(0, 0), create=True):
                self.assertEqual([], REAPER.reap_adopted_zombies(6))

    def test_wrong_waitpid_result_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.proc(root, 20)
            with self.guarded(root), mock.patch.object(REAPER.os, "waitpid", return_value=(21, 0), create=True):
                with self.assertRaisesRegex(RuntimeError, "child PID"):
                    REAPER.reap_adopted_zombies(6)

    def test_malformed_status_and_permission_failures_are_not_silently_ignored(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            proc = self.proc(root, 20)
            (proc / "status").write_bytes(b"State:\tZ\n")
            with self.guarded(root), mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
                with self.assertRaises(KeyError):
                    REAPER.reap_adopted_zombies(6)
                wait.assert_not_called()
            with self.guarded(root), mock.patch.object(Path, "read_bytes", side_effect=PermissionError()), \
                    mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
                with self.assertRaises(PermissionError):
                    REAPER.reap_adopted_zombies(6)
                wait.assert_not_called()

    def test_oversized_status_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            proc = self.proc(root, 20)
            (proc / "status").write_bytes(b"x" * 65537)
            with self.guarded(root), mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
                with self.assertRaisesRegex(RuntimeError, "status size"):
                    REAPER.reap_adopted_zombies(6)
                wait.assert_not_called()

    def test_numeric_proc_symlink_refused_without_waitpid(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.proc(root, 20)
            with self.guarded(root), mock.patch.object(Path, "is_symlink", return_value=True), \
                    mock.patch.object(REAPER.os, "waitpid", create=True) as wait:
                with self.assertRaisesRegex(RuntimeError, "proc symlink"):
                    REAPER.reap_adopted_zombies(6)
                wait.assert_not_called()


if __name__ == "__main__":
    unittest.main()
