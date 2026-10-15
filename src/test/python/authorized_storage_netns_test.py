"""Safety and redaction tests; no private data or JAR is executed in CI."""

import importlib.util
import hashlib
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "compare-authorized-storage-in-netns.py"
spec = importlib.util.spec_from_file_location("authorized_storage_netns", SCRIPT)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class AuthorizedStorageNetnsTest(unittest.TestCase):
    def test_digest_of_empty_binary_fixture(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "empty.bin"
            source.write_bytes(b"")
            self.assertEqual(hashlib.sha256(b"").hexdigest(), probe.digest(source))

    def test_digest_matches_known_binary_vector(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "vector.bin"
            source.write_bytes(b"abc")
            self.assertEqual("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", probe.digest(source))

    def test_digest_without_python311_file_digest(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "generated.bin"
            source.write_bytes(b"GENERATED_HASH_COMPATIBILITY_ONLY\x00\xff\r\n")
            expected = hashlib.sha256(source.read_bytes()).hexdigest()
            with patch.object(probe.hashlib, "file_digest", create=True,
                    side_effect=AttributeError("file_digest is absent on Python 3.10")) as unavailable:
                self.assertEqual(expected, probe.digest(source))
                unavailable.assert_not_called()

    def test_digest_preserves_binary_bytes_across_chunks(self):
        generated = bytes(range(256)) * 10000 + b"\x00\r\n\xffTAIL"
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "multiple-chunks.bin"
            source.write_bytes(generated)
            self.assertEqual(hashlib.sha256(generated).hexdigest(), probe.digest(source))

    def test_digest_reads_bounded_chunks_not_whole_file(self):
        class TrackingBytes(io.BytesIO):
            def read(self, size=-1):
                self_sizes.append(size)
                return super().read(size)
        self_sizes = []
        generated = b"GENERATED_ONLY" * 100000
        with patch.object(Path, "open", return_value=TrackingBytes(generated)):
            self.assertEqual(hashlib.sha256(generated).hexdigest(), probe.digest(Path("never-read-private-data")))
        self.assertGreater(len(self_sizes), 1)
        self.assertTrue(all(size == 1024 * 1024 for size in self_sizes))

    def test_host_invocation_fails_before_data_read_or_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "must-not-exist.json"
            result = subprocess.run([sys.executable, str(SCRIPT), "--java", str(SCRIPT),
                "--original", str(SCRIPT), "--restored", str(SCRIPT),
                "--business-directory", temporary, "--report", str(report)],
                capture_output=True, text=True, timeout=10)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Refusing to start JARs", result.stderr)
            self.assertFalse(report.exists())

    def test_acknowledgment_cannot_replace_kernel_isolation(self):
        for current, initial, interfaces in ((1, 1, ["lo"]), (2, 1, ["lo", "eth0"]),
                                             (2, 1, []), (2, 1, ["eth0"])):
            with self.subTest(interfaces=interfaces):
                with self.assertRaises(SystemExit):
                    probe.check_namespace(current, initial, interfaces)
        probe.check_namespace(2, 1, ["lo"])

    def test_comparison_does_not_publish_private_values(self):
        left = {"status": 200, "replacementCharacter": False,
                "value": {"isSuccess": True, "errorMsg": "", "data": [{"name": "private-a"}]}}
        right = {"status": 200, "replacementCharacter": False,
                 "value": {"isSuccess": True, "errorMsg": "", "data": [{"name": "private-b"}]}}
        result = probe.aggregate(left, right)
        self.assertFalse(result["dataEqual"])
        self.assertEqual(1, result["originalCount"])
        self.assertNotIn("private-", str(result))
        self.assertEqual(5, len(probe.BUSINESS_FILES))
        self.assertNotIn("users.json", probe.BUSINESS_FILES)

    def test_json_comparison_preserves_types_and_defaults(self):
        self.assertFalse(probe.same_json({"enabled": False}, {"enabled": 0}))
        self.assertFalse(probe.same_json({"enabled": True}, {"enabled": 1}))
        self.assertFalse(probe.same_json({"value": None}, {"value": ""}))
        self.assertFalse(probe.same_json({"count": 0}, {"count": 0.0}))
        self.assertTrue(probe.same_json({"a": 0, "b": False}, {"b": False, "a": 0}))


if __name__ == "__main__":
    unittest.main()
