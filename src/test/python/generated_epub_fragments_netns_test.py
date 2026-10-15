"""Pure projection/fixture tests and a real negative guard; never starts Java."""
import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/compare-generated-epub-fragments-in-netns.py"
SPEC = importlib.util.spec_from_file_location("generated_fragments", SCRIPT)
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


def generated_fixture(path, identity="synthetic-fragments", extra=False):
    with ZipFile(path, "w") as archive:
        archive.writestr("mimetype", "application/epub+zip")
        archive.writestr("META-INF/container.xml", "<container/>")
        archive.writestr("OEBPS/book.opf", "<package xmlns:dc='http://purl.org/dc/elements/1.1/'>"
                         + "<dc:identifier>" + identity + "</dc:identifier></package>")
        archive.writestr("OEBPS/toc.ncx", "<ncx/>")
        archive.writestr("OEBPS/Text/shared.xhtml", "<html><body>generated</body></html>")
        if identity == "synthetic-nav-fragments":
            archive.writestr("OEBPS/Nav/toc.xhtml", "<html/>")
            archive.writestr("OEBPS/Text/later.xhtml", "<html/>")
        if extra:
            archive.writestr("OEBPS/unapproved.xhtml", "<html/>")


class GeneratedFragmentsNetnsTest(unittest.TestCase):
    def test_both_generated_identities_and_entry_sets(self):
        with tempfile.TemporaryDirectory() as temporary:
            for identity in ("synthetic-fragments", "synthetic-nav-fragments"):
                path = Path(temporary) / (identity + ".epub")
                generated_fixture(path, identity)
                self.assertEqual(path.read_bytes(), PROBE.fixture_bytes(path, identity))

    def test_extra_entry_and_wrong_identity_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "generated.epub"
            generated_fixture(path, extra=True)
            with self.assertRaisesRegex(ValueError, "exported generated"):
                PROBE.fixture_bytes(path, "synthetic-fragments")
            generated_fixture(path, "other-identity")
            with self.assertRaisesRegex(ValueError, "identifier"):
                PROBE.fixture_bytes(path, "synthetic-fragments")

    def test_normalization_preserves_defaults_types_and_non_time_values(self):
        value = {"durChapterTime": 123, "lastCheckTime": 0, "latestChapterTime": False,
                 "index": 0, "flag": False, "empty": "", "null": None,
                 "content": "http://127.0.0.1:1/book/owner/example"}
        normalized = PROBE.normalize(value, "http://127.0.0.1:1", "owner", Path("/tmp/work"))
        self.assertEqual("__OBSERVED_TIME__", normalized["durChapterTime"])
        self.assertEqual(0, normalized["lastCheckTime"])
        self.assertIs(False, normalized["latestChapterTime"])
        self.assertIs(False, normalized["flag"])
        self.assertIsNone(normalized["null"])
        self.assertEqual("", normalized["empty"])
        self.assertEqual("__API_ROOT__/book/__OWNER__/example", normalized["content"])
        self.assertFalse(PROBE.BASE.same_json({"value": 0}, {"value": False}))

    def test_projection_hashes_exact_normalized_html_without_mutating_response(self):
        reply = {"status": 200, "replacementCharacter": False, "value": {
            "isSuccess": True, "errorMsg": "", "data": {"content": "<p>中文</p>", "count": 0}}}
        before = json.dumps(reply)
        result = PROBE.projected(reply, "http://127.0.0.1:1", "owner", Path("/tmp/work"))
        self.assertEqual(hashlib.sha256("<p>中文</p>".encode()).hexdigest(),
                         result["body"]["data"]["content"]["sha256"])
        self.assertEqual(0, result["body"]["data"]["count"])
        self.assertEqual(before, json.dumps(reply))

    def test_ordinary_host_is_rejected_before_inputs_or_report(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "must-not-exist.json"
            command = [sys.executable, "-B", str(SCRIPT), "--java", "absent-java",
                       "--original", "absent-original", "--restored", "absent-restored",
                       "--fixtures", "absent-fixtures", "--report", str(report)]
            reply = subprocess.run(command, capture_output=True, text=True, timeout=15, check=False)
            self.assertNotEqual(0, reply.returncode)
            self.assertIn("Refusing to start JARs", reply.stderr)
            self.assertFalse(report.exists())


if __name__ == "__main__":
    unittest.main()
