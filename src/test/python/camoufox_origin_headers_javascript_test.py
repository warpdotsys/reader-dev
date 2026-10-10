"""Exact policy JS semantics; not actual network, browser or HTTPS acceptance."""
import os
from pathlib import Path
import re
import shutil
import subprocess
import unittest


class CamoufoxOriginHeadersJavaScriptTest(unittest.TestCase):
    def test_exact_origin_listener_and_readiness_handshake(self):
        node = os.environ.get("READER_TEST_NODE") or shutil.which("node")
        self.assertTrue(node, "Node is required; listener semantics must not be skipped")
        fixture = Path(__file__).parents[1] / "javascript" / "camoufox-origin-headers.test.mjs"
        completed = subprocess.run([node, "--test", "--test-reporter=tap", str(fixture)],
                                   capture_output=True, text=True, encoding="utf-8", timeout=15)
        self.assertEqual(0, completed.returncode, "Generated-only policy JS failed; run the Node fixture")
        self.assertRegex(completed.stdout, re.compile(r"^# tests 19$", re.MULTILINE))
        for counter in ("fail", "cancelled", "skipped", "todo"):
            self.assertRegex(completed.stdout, re.compile(r"^# " + counter + r" 0$", re.MULTILINE))


if __name__ == "__main__":
    unittest.main()
