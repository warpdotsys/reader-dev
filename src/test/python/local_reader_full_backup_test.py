#!/usr/bin/env python3
"""Generated-data contract tests for scripts/local-reader/full-backup.py."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "local-reader" / "full-backup.py"
SPEC = importlib.util.spec_from_file_location("reader_full_backup", SCRIPT)
backup = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
sys.modules[SPEC.name] = backup
SPEC.loader.exec_module(backup)


def stopped(_unit):
    return {"ActiveState": "inactive", "SubState": "dead", "MainPID": "0"}


def active(_unit):
    return {"ActiveState": "active", "SubState": "running", "MainPID": "77"}


class FullBackupTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        work = self.base / "work"
        mount = self.base / "mount"
        work.mkdir()
        self.data = mount / "data"
        self.backups = work / "full-backups"
        self.backups.mkdir()
        roots = {name: self.data / name for name in backup.ROOT_NAMES}
        for root in roots.values():
            root.mkdir(parents=True)
        # Whole roots are backed up, not a hand-picked namespace: this fixture
        # contains global user state, two users, SQLite-like bytes and secrets.
        (roots["storage"] / "globalusers").mkdir()
        (roots["storage"] / "globalusers" / "users.sqlite").write_bytes(b"SQLite format 3\x00generated-users")
        for user in ("alice", "bob"):
            point = roots["storage"] / "data" / user / "library"
            point.mkdir(parents=True)
            (point / "book.txt").write_text("generated " + user, encoding="utf-8")
        (roots["license-data"] / "licenses.sqlite").write_bytes(b"SQLite format 3\x00generated-license")
        (roots["secrets"] / "test-license.key").write_bytes(b"generated-test-key-only")
        os.chmod(roots["secrets"] / "test-license.key", 0o600)
        candidate = backup.candidate_profile.CandidateProfile(
            "reader-g3-test", "6.1.1", "abcdef0", "sha256:" + "1" * 64,
            "2" * 64, "3" * 64, "4" * 64, "5" * 64,
            str(work), str(mount), str(work / "image.ext4"), str(work / "data.ext4"),
            "reader-candidate-reader-g3-test.service", "reader-candidate-reader-g3-test",
            19063, 19064, 2 * 1024 ** 3, 4 * 1024 ** 3, 200, 2 * 1024 ** 3,
            1024 ** 3, 0, 128, 1024 * 1024, 2, 1024 * 1024, 2)
        self.profile = backup.from_candidate_profile(candidate)

    def tearDown(self):
        self.temp.cleanup()

    def make_snapshot(self):
        result = backup.backup(self.profile, True, stopped, lambda _: None)
        return Path(result["snapshot"])

    def test_cli_profile_is_canonical_and_rejects_old_or_spoofed_unit(self):
        candidate_id = "reader-g3-proof"
        work = "/mnt/d/Reader-Pro-Candidate-" + candidate_id
        raw = {
            "schemaVersion": 1, "candidateId": candidate_id,
            "release": {"version": "6.1.1", "revision": "abcdef0"},
            "artifacts": {"imageId": "sha256:" + "1" * 64, "archiveSha256": "2" * 64,
                          "readerJarSha256": "3" * 64, "licenseJarSha256": "4" * 64,
                          "softwareManifestSha256": "5" * 64},
            "paths": {"workDir": work, "mountRoot": "/mnt/reader-candidate-" + candidate_id,
                      "imageDisk": work + "/image.ext4", "dataDisk": work + "/data.ext4"},
            "runtime": {"unit": "reader-candidate-" + candidate_id + ".service",
                        "name": "reader-candidate-" + candidate_id, "readerPort": 19063,
                        "licensePort": 19064, "cpuQuotaPercent": 200,
                        "memoryMaxBytes": 2 * 1024 ** 3, "memoryHighBytes": 1024 ** 3,
                        "memorySwapMaxBytes": 0, "tasksMax": 128},
            "disks": {"imageBytes": 2 * 1024 ** 3, "dataBytes": 4 * 1024 ** 3},
            "logging": {"currentMaxBytes": 1024 * 1024, "previousFiles": 2,
                        "outerMaxBytes": 1024 * 1024, "outerRetainedFiles": 2},
        }
        profile_path = self.base / "canonical-profile.json"
        profile_path.write_text(json.dumps(raw), encoding="utf-8")
        loaded = backup.load_profile(profile_path)
        self.assertEqual("/mnt/reader-candidate-reader-g3-proof/data", str(loaded.data_root))
        self.assertEqual("/mnt/d/Reader-Pro-Candidate-reader-g3-proof/full-backups", str(loaded.backups_root))
        raw["runtime"]["unit"] = "reader-local-full-610.service"
        profile_path.write_text(json.dumps(raw), encoding="utf-8")
        with self.assertRaises(backup.BackupError):
            backup.load_profile(profile_path)

    def test_plan_only_does_not_create_snapshot(self):
        result = backup.backup(self.profile, False, stopped, lambda _: None)
        self.assertFalse(result["executed"])
        self.assertEqual([], [x for x in self.backups.iterdir() if not x.name.endswith(".lock")])
        self.assertGreaterEqual(result["files_and_directories"], 8)

    def test_execute_is_fail_closed_until_candidate_unit_protocol_is_integrated(self):
        with self.assertRaises(backup.BackupError):
            backup.backup(self.profile, True, stopped)

    def test_round_trip_preserves_all_roots_and_modes(self):
        original = (self.data / "storage" / "data" / "alice" / "library" / "book.txt").read_bytes()
        snapshot = self.make_snapshot()
        # Replace all live roots with a different generated state, then restore.
        (self.data / "storage" / "data" / "alice" / "library" / "book.txt").write_text("mutated", encoding="utf-8")
        (self.data / "license-data" / "licenses.sqlite").write_bytes(b"mutated")
        restored = backup.restore(self.profile, snapshot, True, stopped, lambda _: None)
        self.assertTrue(restored["executed"])
        self.assertEqual(original, (self.data / "storage" / "data" / "alice" / "library" / "book.txt").read_bytes())
        self.assertEqual(b"SQLite format 3\x00generated-license", (self.data / "license-data" / "licenses.sqlite").read_bytes())
        self.assertEqual(0o600, stat.S_IMODE((self.data / "secrets" / "test-license.key").stat().st_mode))
        quarantine = Path(restored["quarantine"])
        self.assertTrue((quarantine / "storage" / "data" / "alice" / "library" / "book.txt").exists())

    def test_tampered_or_truncated_snapshot_does_not_touch_live_roots(self):
        snapshot = self.make_snapshot()
        live = self.data / "storage" / "data" / "alice" / "library" / "book.txt"
        live.write_text("live-must-remain", encoding="utf-8")
        target = snapshot / "storage" / "data" / "alice" / "library" / "book.txt"
        target.write_bytes(b"truncated")
        with self.assertRaises(backup.BackupError):
            backup.restore(self.profile, snapshot, True, stopped, lambda _: None)
        self.assertEqual("live-must-remain", live.read_text(encoding="utf-8"))

    def test_manifest_path_escape_and_extra_file_are_rejected(self):
        snapshot = self.make_snapshot()
        manifest_path = snapshot / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["entries"][0]["path"] = "storage/../escaped"
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        with self.assertRaises(backup.BackupError):
            backup.restore(self.profile, snapshot, False, stopped, lambda _: None)
        # A separately fresh snapshot verifies strict complete-set checking.
        snapshot = self.make_snapshot()
        (snapshot / "storage" / "unexpected").write_bytes(b"no")
        with self.assertRaises(backup.BackupError):
            backup.restore(self.profile, snapshot, False, stopped, lambda _: None)

    def test_active_or_wrong_runtime_scope_refuses_mutation(self):
        with self.assertRaises(backup.BackupError):
            backup.backup(self.profile, True, active, lambda _: None)
        snapshot = self.make_snapshot()
        with self.assertRaises(backup.BackupError):
            backup.restore(self.profile, snapshot, True, lambda _: {"ActiveState": "inactive", "SubState": "dead", "MainPID": "1"}, lambda _: None)

    @unittest.skipUnless(os.name == "posix", "hardlink/symlink filesystem assertions require POSIX")
    def test_links_are_rejected_before_snapshot(self):
        point = self.data / "storage" / "data" / "alice" / "library"
        os.link(point / "book.txt", point / "hard.txt")
        with self.assertRaises(backup.BackupError):
            backup.backup(self.profile, False, stopped, lambda _: None)
        (point / "hard.txt").unlink()
        os.symlink(point / "book.txt", point / "symlink.txt")
        with self.assertRaises(backup.BackupError):
            backup.backup(self.profile, False, stopped, lambda _: None)

    @unittest.skipUnless(hasattr(os, "fork"), "SIGKILL recovery requires POSIX fork")
    def test_sigkill_mid_rename_recovers_old_complete_candidate(self):
        snapshot = self.make_snapshot()
        live = self.data / "storage" / "data" / "alice" / "library" / "book.txt"
        live.write_text("live-before-kill", encoding="utf-8")
        child = os.fork()
        if child == 0:
            def kill_after_second_old_move(phase):
                if phase == "old-moved:license-data":
                    os.kill(os.getpid(), 9)
            backup.restore(self.profile, snapshot, True, stopped, lambda _: None, kill_after_second_old_move)
            os._exit(99)
        _pid, status = os.waitpid(child, 0)
        self.assertTrue(os.WIFSIGNALED(status) and os.WTERMSIG(status) == 9)
        with self.assertRaises(backup.BackupError):
            backup.startup_guard(self.profile)
        with backup.maintenance_lock(self.profile):
            recovered = backup.recover_transaction(self.profile, True)
        self.assertTrue(recovered["executed"])
        self.assertEqual("live-before-kill", live.read_text(encoding="utf-8"))
        backup.startup_guard(self.profile)

    @unittest.skipUnless(hasattr(os, "fork"), "SIGKILL recovery requires POSIX fork")
    def test_sigkill_after_committed_journal_still_recovers_complete_old_candidate(self):
        snapshot = self.make_snapshot()
        live = self.data / "storage" / "data" / "alice" / "library" / "book.txt"
        live.write_text("old-before-commit-window-kill", encoding="utf-8")
        child = os.fork()
        if child == 0:
            backup.restore(self.profile, snapshot, True, stopped, lambda _: None,
                           lambda phase: os.kill(os.getpid(), 9) if phase == "committed" else None)
            os._exit(99)
        _pid, status = os.waitpid(child, 0)
        self.assertTrue(os.WIFSIGNALED(status) and os.WTERMSIG(status) == 9)
        with backup.maintenance_lock(self.profile):
            recovered = backup.recover_transaction(self.profile, True)
        self.assertTrue(recovered["executed"])
        self.assertEqual("old-before-commit-window-kill", live.read_text(encoding="utf-8"))
        backup.startup_guard(self.profile)


if __name__ == "__main__":
    unittest.main(verbosity=2)
