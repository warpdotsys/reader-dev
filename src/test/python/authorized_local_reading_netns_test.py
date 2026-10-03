"""Synthetic safety tests. CI never loads the owner's EPUB or bookshelf."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[3] / "scripts/compare-authorized-local-reading-in-netns.py"
spec = importlib.util.spec_from_file_location("authorized_local_reading", SCRIPT)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class AuthorizedLocalReadingNetnsTest(unittest.TestCase):
    def test_host_fails_before_loading_any_private_input(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "must-not-exist.json"
            arguments = [sys.executable, "-B", str(SCRIPT)]
            for name in ("java", "original", "restored", "shelf", "asset"):
                arguments.extend(["--" + name, str(Path(temporary) / "absent")])
            result = subprocess.run(arguments + ["--report", str(report)],
                                    capture_output=True, text=True, timeout=10)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Refusing to start JARs", result.stderr)
            self.assertFalse(report.exists())

    def test_only_exact_owner_authorized_local_title_is_selected(self):
        allowed = {"name": probe.ALLOWED_TITLE, "origin": "loc_book"}
        self.assertIs(allowed, probe.select_book([{"name": "private-other", "origin": "loc_book"}, allowed]))
        for shelf in ([], [dict(allowed, origin="https://example.invalid")], [allowed, allowed],
                      [{"name": probe.ALLOWED_TITLE + " extra", "origin": "loc_book"}]):
            with self.subTest(shelf=shelf), self.assertRaises(ValueError):
                probe.select_book(shelf)

    def test_report_facts_redact_html_and_ignore_injected_script(self):
        first = probe.html_facts("<html><head><title>secret</title></head><body><p>合成正文</p></body></html>")
        second = probe.html_facts("<body><p>合成正文</p><script>private-token</script></body>")
        self.assertEqual(first["bodyTextSha256"], second["bodyTextSha256"])
        self.assertNotEqual(first["htmlSha256"], second["htmlSha256"])
        self.assertEqual(4, first["bodyCharacters"])
        self.assertNotIn("合成正文", str(first))
        self.assertNotIn("private-token", str(second))
        self.assertEqual(1, probe.html_facts("<body>\ufffd</body>")["replacementCharacters"])


if __name__ == "__main__":
    unittest.main()
