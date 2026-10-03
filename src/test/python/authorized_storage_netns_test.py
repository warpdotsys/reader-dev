"""Safety and redaction tests; no private data or JAR is executed in CI."""

import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "compare-authorized-storage-in-netns.py"
spec = importlib.util.spec_from_file_location("authorized_storage_netns", SCRIPT)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class AuthorizedStorageNetnsTest(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
