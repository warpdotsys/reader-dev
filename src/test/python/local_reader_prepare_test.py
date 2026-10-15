"""Unit tests for the guarded local exact-image preparation planner.

No test invokes --execute, mounts a loop device, formats a disk, or reads the
large archive.  The tests exercise the safety gates that keep default runs
read-only and prevent accidental use of ordinary files as disks.
"""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = ROOT / "scripts/local-reader/prepare.py"
SPEC = importlib.util.spec_from_file_location("local_reader_prepare_test_module", MODULE_PATH)
prepare = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(prepare)


class LocalReaderPrepareTest(unittest.TestCase):
    def write_disk_and_marker(self, directory: Path, name: str, purpose: str, size: int) -> Path:
        disk = directory / name
        with disk.open("xb") as stream:
            stream.truncate(size)
        prepare.marker_path(disk).write_text(json.dumps({
            "schemaVersion": 1, "task": prepare.TASK, "purpose": purpose,
            "bytes": size, "state": "blank-for-first-format",
        }), encoding="utf-8")
        return disk

    def test_marker_accepts_exact_fresh_owned_disk(self):
        marker = {
            "schemaVersion": 1, "task": prepare.TASK, "purpose": "image",
            "bytes": prepare.IMAGE_BYTES, "state": "blank-for-first-format",
        }
        self.assertEqual(marker["task"], prepare.TASK)
        self.assertEqual(marker["bytes"], prepare.IMAGE_BYTES)

    def test_marker_rejects_wrong_task_or_size_before_execution(self):
        with tempfile.TemporaryDirectory() as raw:
            disk = Path(raw) / "disk.img"
            with disk.open("xb") as stream:
                stream.truncate(32)
            prepare.marker_path(disk).write_text(json.dumps({
                "schemaVersion": 1, "task": "not-this-task", "purpose": "image",
                "bytes": 32, "state": "blank-for-first-format",
            }), encoding="utf-8")
            with self.assertRaises(AssertionError):
                prepare.load_owned_marker(disk, "image", 32)

    def test_real_zero_byte_inode_gate_accepts_only_empty_regular_file(self):
        with tempfile.TemporaryDirectory() as raw:
            disk = Path(raw) / "fresh.ext4"
            disk.touch()
            descriptor = prepare.open_empty_owned_disk(disk)
            try:
                self.assertEqual(__import__("os").fstat(descriptor).st_size, 0)
            finally:
                __import__("os").close(descriptor)
            disk.write_bytes(b"not blank")
            with self.assertRaises(AssertionError):
                prepare.open_empty_owned_disk(disk)

    def test_only_fixed_work_and_disk_paths_are_accepted(self):
        args = prepare.parse_args(["--image-disk", prepare.WORK_DIR + "/image.ext4",
                                   "--data-disk", prepare.WORK_DIR + "/data.ext4"])
        work, image, data = prepare.fixed_paths(args)
        self.assertEqual(str(work).replace("\\", "/"), prepare.WORK_DIR)
        self.assertEqual(str(image).replace("\\", "/"), prepare.WORK_DIR + "/image.ext4")
        self.assertEqual(str(data).replace("\\", "/"), prepare.WORK_DIR + "/data.ext4")
        args.work_dir = "/mnt/d/other"
        with self.assertRaises(AssertionError):
            prepare.fixed_paths(args)

    def test_default_cli_is_plan_only(self):
        args = prepare.parse_args(["--image-disk", "/mnt/d/image.img", "--data-disk", "/mnt/d/data.img"])
        self.assertFalse(args.execute)
        self.assertEqual(args.mount_root, "/mnt/reader-local-full-610")

    def test_execute_is_explicit(self):
        args = prepare.parse_args(["--image-disk", "/mnt/d/image.img", "--data-disk", "/mnt/d/data.img", "--execute"])
        self.assertTrue(args.execute)

    def test_environment_pins_every_temp_cache_root_to_owned_mount(self):
        fake = {name: Path("/tool/usr/bin") / name for name in ("skopeo", "umoci", "runc")}
        env = prepare.isolated_environment(Path("/mnt/reader-local-full-610/image/tmp"), Path("/mnt/reader-local-full-610/image/cache"), fake)
        self.assertEqual(env["TMPDIR"], "/mnt/reader-local-full-610/image/tmp")
        self.assertEqual(env["TMP"], env["TMPDIR"])
        self.assertEqual(env["TEMP"], env["TMPDIR"])
        self.assertEqual(env["XDG_CACHE_HOME"], "/mnt/reader-local-full-610/image/cache")


if __name__ == "__main__":
    unittest.main()
