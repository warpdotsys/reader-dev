"""No-network contract checks for the paired public WebView report."""

from email.message import Message
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "report-public-webview-probes.py"
ARCHIVED_SCRIPT = SCRIPT.with_name("probe-archived-remote-webview.py")
URL = "https://m.jjjxsw.com/txt/"


class ArchivedProjectionTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location("archived_probe", ARCHIVED_SCRIPT)
        cls.probe = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.probe)

    def render_raw(self, raw):
        response = io.BytesIO(raw)
        response.status = 200
        response.headers = Message()
        with patch.object(self.probe.urllib.request, "urlopen", return_value=response):
            return self.probe.render("http://127.0.0.1:18050", URL,
                                     js_source="[]", expect_projection=True)

    def test_projection_is_hashed_without_emitting_book_titles(self):
        projection = [["公开测试书", URL + "123.html"]]
        raw = json.dumps(projection, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        result = self.render_raw(raw)
        self.assertEqual(1, result["projectionCount"])
        self.assertEqual(hashlib.sha256(raw).hexdigest(), result["projectionSha256"])
        self.assertNotIn("公开测试书", json.dumps(result, ensure_ascii=False))

    def test_projection_rejects_unrelated_urls(self):
        with self.assertRaisesRegex(RuntimeError, "invalid book projection"):
            self.render_raw(b'[["x","https://example.org/txt/1.html"]]')


class PublicWebviewProbeTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.archived = [
            {"public": {"status": 200, "bodySha256": "a" * 64}},
            {"publicDomBookProjection": {
                "url": URL, "status": 200, "count": 17,
                "projectionSha256": "b" * 64,
                "observedAt": "2026-09-29T01:00:04+00:00"}},
        ]
        self.camoufox = [{
            "source": URL, "status": 200, "isSuccess": True, "errorMsg": "",
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
        self.assertTrue(report["projectionEqual"])
        self.assertFalse(report["originalJarCompared"])
        self.assertFalse(report["referenceProvenProduction"])
        self.assertFalse(report["fullResponseParityProven"])
        self.assertEqual(5, report["observationDeltaSeconds"])

    def test_divergence_is_reported_without_false_failure(self):
        self.camoufox[0]["bookCount"] = 16
        self.camoufox[0]["projectionSha256"] = "c" * 64
        process, report = self.run_compare()
        self.assertEqual(0, process.returncode, process.stderr)
        self.assertFalse(report["countEqual"])
        self.assertFalse(report["projectionEqual"])
        self.assertEqual("divergent-projection-needs-investigation", report["interpretation"])
        self.assertIn("::warning::", process.stdout)

    def test_different_url_is_not_compared(self):
        self.camoufox[0]["source"] = "https://example.org/txt/"
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)

    def test_missing_direct_page_is_not_a_pass(self):
        self.archived.pop(0)
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)

    def test_missing_projection_is_not_a_pass(self):
        self.archived.pop()
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)

    def test_bool_count_is_not_accepted_as_integer(self):
        self.archived[1]["publicDomBookProjection"]["count"] = True
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)

    def test_success_with_nonempty_error_is_not_a_pass(self):
        self.camoufox[0]["errorMsg"] = "unexpected warning"
        process, report = self.run_compare()
        self.assertNotEqual(0, process.returncode)
        self.assertIsNone(report)


if __name__ == "__main__":
    unittest.main()
