"""CLI safety checks; never starts the archived JAR or a network listener."""

import subprocess
import sys
import tempfile
import unittest
import os
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "compare-webview-cookie.py"


def invoke(*arguments):
    return subprocess.run(
        [sys.executable, "-B", str(SCRIPT), *map(str, arguments)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )


class WebviewCookieCliSafetyTest(unittest.TestCase):
    def test_refuses_to_start_without_an_explicit_network_mode(self):
        with tempfile.TemporaryDirectory(prefix="reader-webview-cli-") as directory:
            report = Path(directory) / "new-report.json"
            result = invoke("--report", report)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("--restored-only", result.stderr)
            self.assertIn("--original-network-isolated", result.stderr)
            self.assertFalse(report.exists())

    def test_refuses_to_overwrite_an_existing_report_before_looking_for_jars(self):
        with tempfile.TemporaryDirectory(prefix="reader-webview-cli-") as directory:
            report = Path(directory) / "user-report.json"
            original = b"keep this existing report unchanged\n"
            report.write_bytes(original)
            result = invoke("--restored-only", "--report", report)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Report already exists", result.stderr)
            self.assertEqual(original, report.read_bytes())

    def test_original_mode_also_requires_isolation_acknowledgment(self):
        with tempfile.TemporaryDirectory(prefix="reader-webview-cli-") as directory:
            report = Path(directory) / "new-report.json"
            environment = os.environ.copy()
            environment.pop("READER_ORIGINAL_JAR_NETWORK_ISOLATED", None)
            result = subprocess.run(
                [sys.executable, "-B", str(SCRIPT),
                 "--original-network-isolated", "--report", str(report)],
                cwd=ROOT, env=environment, capture_output=True, text=True,
                timeout=10, check=False,
            )
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Refusing to start the original JAR", result.stderr)
            self.assertFalse(report.exists())


if __name__ == "__main__":
    unittest.main()
