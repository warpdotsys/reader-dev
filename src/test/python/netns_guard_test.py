"""A normal host invocation must never launch the original JAR."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "run-webview-cookie-in-linux-netns.py"


class NetnsGuardTest(unittest.TestCase):
    def test_regular_host_cannot_acknowledge_isolation(self):
        for extra in ([], ["--archived-renderer"],
                      ["--archived-renderer", "--camoufox-python", sys.executable]):
            with self.subTest(extra=extra):
                self.check_host_refused(extra)

    def check_host_refused(self, extra):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "must-not-exist.json"
            result = subprocess.run([
                sys.executable, str(SCRIPT),
                "--java", str(SCRIPT),
                "--original", str(SCRIPT),
                "--restored", str(SCRIPT),
                "--report", str(report),
                "--exercise-post", *extra,
            ], capture_output=True, text=True, timeout=10, check=False)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Refusing to start JARs", result.stderr)
            self.assertFalse(report.exists())


if __name__ == "__main__":
    unittest.main()
