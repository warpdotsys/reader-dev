"""No-systemd tests for atomic local runtime source refresh."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "scripts" / "local-reader"
sys.path.insert(0, str(SOURCE))
SPEC = importlib.util.spec_from_file_location("local_reader_stage", SOURCE / "stage-runtime.py")
stage = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(stage)


class LocalReaderStageTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.work = self.root / "work"; self.work.mkdir()
        self.target = self.work / "runtime-software"; self.target.mkdir()
        self.source = self.root / "source"; self.source.mkdir()
        self.names = stage.SOURCE_NAMES
        self.old = {}
        for name in self.names:
            value = ("old-" + name).encode()
            target = self.target / "source" / name; target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(value); self.old[name] = value
            (self.source / name).write_bytes(("new-" + name).encode())
        fixed = {
            "runc/bin/runc": b"runc", "runc/lib/libseccomp.so.2": b"seccomp",
            "license/server.jar": b"license",
        }
        records = []
        for relative, value in fixed.items():
            target = self.target / relative; target.parent.mkdir(parents=True, exist_ok=True); target.write_bytes(value)
            records.append({"path": relative, "bytes": len(value), "sha256": stage.prepare.sha256_file(target)})
        for name in self.names:
            target = self.target / "source" / name
            records.append({"path": "source/" + name, "bytes": len(self.old[name]), "sha256": stage.prepare.sha256_file(target)})
        self.manifest = self.target / "runtime-software.lock.json"
        self.manifest.write_text(json.dumps({"schemaVersion": 1, "files": records}), encoding="utf-8")

    def tearDown(self):
        self.temp.cleanup()

    def test_refresh_preserves_old_sources_then_publishes_manifest_last(self):
        old_manifest = self.manifest.read_bytes()
        with patch.object(stage, "_unit_must_be_inactive") as inactive, contextlib.redirect_stdout(io.StringIO()):
            stage.refresh_source(self.target, work=self.work, source_root=self.source, unit="test.service")
        inactive.assert_called_once_with("test.service")
        result = json.loads(self.manifest.read_text())
        by_path = {item["path"]: item for item in result["files"]}
        history = list((self.target / "source-history").iterdir())
        self.assertEqual(1, len(history))
        self.assertEqual(old_manifest, (history[0] / "runtime-software.lock.json").read_bytes())
        for name in self.names:
            durable = self.target / "source" / name
            self.assertEqual(("new-" + name).encode(), durable.read_bytes())
            self.assertEqual(self.old[name], (history[0] / name).read_bytes())
            self.assertEqual(stage.prepare.sha256_file(durable), by_path["source/" + name]["sha256"])

    def test_invalid_source_is_rejected_before_history_or_replacement(self):
        before = self.manifest.read_bytes()
        broken = self.source / "service.py"
        broken.unlink(); broken.mkdir()
        with patch.object(stage, "_unit_must_be_inactive"), self.assertRaisesRegex(ValueError, "regular"):
            stage.refresh_source(self.target, work=self.work, source_root=self.source)
        self.assertEqual(before, self.manifest.read_bytes())
        self.assertEqual(self.old["prepare.py"], (self.target / "source/prepare.py").read_bytes())
        self.assertFalse((self.target / "source-history").exists())

    def test_busy_controller_lock_refuses_without_calling_systemd(self):
        lock = self.work / "service-controller.lock"
        descriptor = os.open(lock, os.O_RDWR | os.O_CREAT, 0o600)
        try:
            import fcntl
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            with patch.object(stage, "_unit_must_be_inactive") as inactive, self.assertRaisesRegex(ValueError, "busy"):
                stage.refresh_source(self.target, work=self.work, source_root=self.source)
            inactive.assert_not_called()
        finally:
            os.close(descriptor)

    def test_active_unit_is_rejected_without_source_change(self):
        before = self.manifest.read_bytes()
        with patch.object(stage, "_unit_must_be_inactive", side_effect=ValueError("active")), self.assertRaisesRegex(ValueError, "active"):
            stage.refresh_source(self.target, work=self.work, source_root=self.source)
        self.assertEqual(before, self.manifest.read_bytes())

    def test_output_handle_is_closed_before_replace_for_drvfs(self):
        original_open, original_close, original_replace = os.open, os.close, os.replace
        opened = {}
        def tracked_open(path, *args, **kwargs):
            descriptor = original_open(path, *args, **kwargs)
            opened[descriptor] = Path(path)
            return descriptor
        def tracked_close(descriptor):
            original_close(descriptor); opened.pop(descriptor, None)
        def checked_replace(source, target):
            self.assertNotIn(Path(source), opened.values())
            original_replace(source, target)
        with patch.object(stage.os, 'open', side_effect=tracked_open), \
                patch.object(stage.os, 'close', side_effect=tracked_close), \
                patch.object(stage.os, 'replace', side_effect=checked_replace):
            source = self.source / 'service.py'
            stage._copy_atomic(source, self.target / 'copied.py', stage.prepare.sha256_file(source))
            stage._write_manifest_atomic(self.target / 'new-manifest.json', {'generated': True})
        self.assertEqual({}, opened)


if __name__ == "__main__":
    unittest.main()
