"""Execute the worker's exact JS helpers in Node; not real browser acceptance."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import unittest


class CamoufoxSourceScriptJavaScriptTest(unittest.TestCase):
    def test_actual_javascript_helpers_preserve_resolved_results_and_sanitized_failures(self):
        node = os.environ.get("READER_TEST_NODE") or shutil.which("node")
        self.assertTrue(node, "Node is required for JS semantics; this check must not be skipped")
        fixture = Path(__file__).parents[1] / "javascript" / "camoufox-source-script.test.mjs"
        completed = subprocess.run([node, "--test", "--test-reporter=tap", str(fixture)], capture_output=True,
                                   text=True, encoding="utf-8", timeout=15)
        self.assertEqual(0, completed.returncode,
                         "Generated-only worker JS tests failed; run the Node fixture for diagnostics")
        self.assertRegex(completed.stdout, re.compile(r"^# tests 29$", re.MULTILINE))
        for counter in ("fail", "cancelled", "skipped", "todo"):
            self.assertRegex(completed.stdout, re.compile(r"^# " + counter + r" 0$", re.MULTILINE))


if __name__ == "__main__":
    unittest.main()
