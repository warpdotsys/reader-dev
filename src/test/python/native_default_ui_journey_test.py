"""Generated XML/header doubles test rejection, not actual browser execution."""
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

ROOT = Path(__file__).parents[3]
SPEC = importlib.util.spec_from_file_location("native_ui_guard", ROOT / "scripts/verify-native-default-ui-journey.py")
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)
REVISION = "a" * 40


class NativeUiGuardTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="reader-generated-ui-guard-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.xml = self.root / ("TEST-" + GUARD.CLASS + ".xml")
        self.raw = ('<testsuite name="' + GUARD.CLASS + '" tests="1" failures="0" errors="0" skipped="0">'
                    '<testcase classname="' + GUARD.CLASS + '" name="' + GUARD.METHOD + '" time="12.5"/>'
                    '</testsuite>')
        self.xml.write_text(self.raw, encoding="utf-8")
        for stage in GUARD.STAGES:
            (self.root / ("native-default-ui-generated-" + stage + ".png")).write_bytes(
                b"\x89PNG\r\n\x1a\n" + struct.pack(">I", 13) + b"IHDR" + struct.pack(">II", 1280, 900) + b"\x00" * 9)

    def verify(self, revision=REVISION):
        return GUARD.verify(self.root, self.root, revision)

    def test_finite_doubles_only_show_guard_acceptance_with_explicit_limits(self):
        value = self.verify()
        self.assertEqual(1, value["tests"])
        self.assertEqual(4, len(value["screenshots"]))
        self.assertTrue(value["clientRunsOutsideServerResourceBudget"])
        self.assertFalse(value["realAccountOrPrivateBooksTested"])
        self.assertFalse(value["productionOrLongRunProven"])

    def test_exact_revision_is_required(self):
        for revision in (None, "a" * 39, "A" * 40, "", True):
            with self.subTest(revision=revision), self.assertRaises(GUARD.JourneyRejected):
                self.verify(revision)

    def test_missing_suite_rejected(self):
        self.xml.unlink()
        with self.assertRaises(GUARD.JourneyRejected):
            self.verify()

    def test_extra_or_mixed_junit_suite_rejected(self):
        (self.root / "TEST-another.xml").write_text(self.raw, encoding="utf-8")
        with self.assertRaises(GUARD.JourneyRejected):
            self.verify()

    def test_counts_failures_and_skips_are_exact(self):
        for field, before in (("tests", "1"), ("failures", "0"), ("errors", "0"), ("skipped", "0")):
            for after in ("2", "true", "", "01"):
                self.xml.write_text(self.raw.replace(field + '="' + before + '"', field + '="' + after + '"'), encoding="utf-8")
                with self.subTest(field=field, after=after), self.assertRaises(GUARD.JourneyRejected):
                    self.verify()

    def test_wrong_case_or_hidden_failed_child_rejected(self):
        variants = (self.raw.replace(GUARD.METHOD, "oldTest"),
                    self.raw.replace('<testcase classname="' + GUARD.CLASS, '<testcase classname="oldClass'),
                    self.raw.replace('/></testsuite>', '><failure/></testcase></testsuite>'),
                    self.raw.replace('/></testsuite>', '><skipped/></testcase></testsuite>'))
        for value in variants:
            self.xml.write_text(value, encoding="utf-8")
            with self.assertRaises(GUARD.JourneyRejected):
                self.verify()

    def test_duration_must_prove_finite_nonzero_execution(self):
        for value in ("0", "-1", "nan", "inf", "181", "bad"):
            self.xml.write_text(self.raw.replace('time="12.5"', 'time="' + value + '"'), encoding="utf-8")
            with self.assertRaises(GUARD.JourneyRejected):
                self.verify()

    def test_malformed_or_entity_xml_rejected(self):
        for value in ("<testsuite>", '<!DOCTYPE x [<!ENTITY e "text">]>' + self.raw):
            self.xml.write_text(value, encoding="utf-8")
            with self.assertRaises(GUARD.JourneyRejected):
                self.verify()

    def test_empty_or_oversized_xml_rejected(self):
        for value in (b"", b" " * (GUARD.MAX_XML + 1)):
            self.xml.write_bytes(value)
            with self.assertRaises(GUARD.JourneyRejected):
                self.verify()

    def test_each_missing_screenshot_rejected(self):
        image = self.root / "native-default-ui-generated-reading.png"
        image.unlink()
        with self.assertRaises(GUARD.JourneyRejected):
            self.verify()

    def test_bad_header_or_wrong_viewport_rejected(self):
        image = self.root / "native-default-ui-generated-login.png"
        original = image.read_bytes()
        for value in (b"not png", original[:16] + struct.pack(">II", 1024, 900) + original[24:]):
            image.write_bytes(value)
            with self.assertRaises(GUARD.JourneyRejected):
                self.verify()


if __name__ == "__main__":
    unittest.main()
