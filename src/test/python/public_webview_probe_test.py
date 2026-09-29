"""No-network contract checks for the paired public WebView report."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "report-public-webview-probes.py"
URL = "https://m.jjjxsw.com/txt/"


class PublicWebviewProbeTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.archived = [
            {"public": {"status": 200, "bodySha256": "a" * 64}},
            {"publicDomBookListCount": {
                "url": URL, "status": 200, "count": 17,
                "observedAt": "2026-09-29T01:00:00+00:00"}},
        ]
        self.camoufox = [{
            "source": URL, "status": 200, "isSuccess": True,
            "bookCount": 17, "projectionSha256": "b" * 64,
            "observedAt": "2026-09-29T01:00:09+00:00",
        }]

    def run_compare(self):
        archived_path = self.root / "archived.log"
        camoufox_path = self.root / "camoufox.log"
        report_path = self.root / "report.json"
        archived_path.write_text("".join(json.dumps(row) + "\n" for row in self.archived),
                                 encoding="utf-8")
        camoufox_path.write_text("".join(json.dumps(row) + "\n" for row in self.camoufox),
                                 encoding="utf-8")
        result = subprocess.run([
            sys.executable, str(SCRIPT), "--archived-log", str(archived_path),
            "--camoufox-log", str(camoufox_path), "--report", str(report_path),
        ], capture_output=True, text=True, check=False)
        report = json.loads(report_path.read_text(encoding="utf-8")) if report_path.exists() else None
        return result, report

    def test_matching_count_is_bounded_observation_only(self):
        process, report = self.run_compare()
        self.assertEqual(0, process.returncode, process.stderr)
        self.assertTrue(report["countEqual"])
        self.assertFalse(report["originalJarCompared"])
        self.assertFalse(report["referenceProvenProduction"])
        self.assertFalse(report["fullResponseParityProven"])
        self.assertEqual(9, report["observationDeltaSeconds"])

    def test_divergence_is_reported_without_false_failure(self):
        self.camoufox[0]["bookCount"] = 16
        process, report = self.run_compare()
        self.assertEqual(0, process.returncode, process.stderr)
        self.assertFalse(report["countEqual"])
        self.assertEqual("divergent-count-needs-investigation", report["interpretation"])
        self.assertIn("::warning::", process.stdout)

    def test_different_url_is_not_compared(self):
        self.camoufox[0]["source"] = "https://example.org/txt/"
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)

    def test_missing_public_count_is_not_a_pass(self):
        self.archived.pop()
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)

    def test_bool_count_is_not_accepted_as_integer(self):
        self.archived[1]["publicDomBookListCount"]["count"] = True
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)


if __name__ == "__main__":
    unittest.main()
