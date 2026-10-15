import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "verify-browser-jvm-contracts.py"
SPEC = importlib.util.spec_from_file_location("browser_jvm_contracts", SCRIPT)
VERIFY = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(VERIFY)


class BrowserJvmContractsTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.directory = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def write_standard_reports(self):
        for class_name, names in VERIFY.EXPECTED.items():
            cases = "".join(
                f'<testcase classname="{class_name}" name="{name}" time="0.01"/>' for name in names
            )
            report = (
                f'<testsuite name="{class_name}" tests="{len(names)}" skipped="0" failures="0" errors="0">'
                f"{cases}</testsuite>"
            )
            (self.directory / f"TEST-{class_name}.xml").write_text(report, encoding="utf-8")

    def test_standard_junit_reports_pass_and_return_machine_summary(self):
        self.write_standard_reports()
        summary = VERIFY.verify_results(self.directory)
        self.assertEqual(35, summary["requiredTests"])
        self.assertEqual(35, summary["executedTests"])
        self.assertEqual(0, summary["skipped"])

    def test_empty_suite_cannot_fake_a_green_summary(self):
        self.write_standard_reports()
        path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
        path.write_text(
            '<testsuite name="com.htmake.reader.utils.LocalWebviewRendererTest" tests="0" skipped="0" failures="0" errors="0"/>',
            encoding="utf-8",
        )
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)

    def test_duplicate_required_case_is_rejected(self):
        self.write_standard_reports()
        path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
        text = path.read_text(encoding="utf-8")
        needle = '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser" time="0.01"/>'
        path.write_text(text.replace(needle, needle + needle).replace('tests="11"', 'tests="12"'), encoding="utf-8")
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)

    def test_missing_required_case_is_rejected(self):
        self.write_standard_reports()
        path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
        text = path.read_text(encoding="utf-8")
        needle = '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser" time="0.01"/>'
        path.write_text(text.replace(needle, "").replace('tests="11"', 'tests="10"'), encoding="utf-8")
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)

    def test_all_three_skip_encodings_are_rejected(self):
        variants = (
            '<skipped/>',
            '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser" status="notrun"/>',
            '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser" result="skipped"/>',
        )
        for variant in variants:
            with self.subTest(variant=variant):
                self.write_standard_reports()
                path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
                text = path.read_text(encoding="utf-8")
                old = '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser" time="0.01"/>'
                replacement = (
                    '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" '
                    'name="getAndPostExecuteInBrowser">' + variant + "</testcase>"
                    if variant == '<skipped/>' else variant
                )
                path.write_text(text.replace(old, replacement), encoding="utf-8")
                with self.assertRaises(VERIFY.VerificationError):
                    VERIFY.verify_results(self.directory)

    def test_failure_error_and_dtd_are_rejected(self):
        self.write_standard_reports()
        path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
        text = path.read_text(encoding="utf-8")
        old = '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser" time="0.01"/>'
        path.write_text(text.replace(old, '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="getAndPostExecuteInBrowser"><failure/></testcase>'), encoding="utf-8")
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)
        path.write_text('<!DOCTYPE x [<!ENTITY x SYSTEM "file:///not-read">]><testsuite/>', encoding="utf-8")
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)

    def test_missing_summary_mismatch_foreign_class_and_unknown_status_fail(self):
        for transform in (
            lambda text: text.replace('tests="11"', 'tests="12"'),
            lambda text: text.replace('skipped="0"', 'skipped="1"'),
            lambda text: text.replace('errors="0"', 'errors="1"'),
            lambda text: text.replace('errors="0"', ''),
            lambda text: text.replace(' time="0.01"', ' status="unknown"', 1),
            lambda text: text.replace(' time="0.01"', ' result="failure"', 1),
            lambda text: text.replace('classname="com.htmake.reader.utils.LocalWebviewRendererTest"', 'classname="wrong"', 1),
            lambda text: text.replace(' time="0.01"/>', ' time="0.01"><error/></testcase>', 1),
        ):
            self.write_standard_reports()
            path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
            path.write_text(transform(path.read_text(encoding='utf-8')), encoding='utf-8')
            with self.assertRaises(VERIFY.VerificationError):
                VERIFY.verify_results(self.directory)

    def test_utf16_dtd_and_oversized_xml_fail_before_parsing(self):
        self.write_standard_reports()
        path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
        for encoding in ('utf-16', 'utf-16-le', 'utf-16-be'):
            path.write_bytes('<!DOCTYPE x [<!ENTITY x "generated">]><testsuite/>'.encode(encoding))
            with self.assertRaises(VERIFY.VerificationError):
                VERIFY.verify_results(self.directory)
        path.write_bytes(b' ' * (VERIFY.MAX_XML_BYTES + 1))
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)

    def test_missing_suite_and_additional_successful_test(self):
        self.write_standard_reports()
        path = self.directory / "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml"
        text = path.read_text(encoding='utf-8')
        path.write_text(text.replace('tests="11"', 'tests="12"').replace('</testsuite>',
            '<testcase classname="com.htmake.reader.utils.LocalWebviewRendererTest" name="newGeneratedCase"/></testsuite>'), encoding='utf-8')
        self.assertEqual(36, VERIFY.verify_results(self.directory)['executedTests'])
        path.unlink()
        with self.assertRaises(VERIFY.VerificationError):
            VERIFY.verify_results(self.directory)

    def test_hosted_workflow_runs_and_preserves_the_complete_matrix(self):
        workflow = (ROOT / ".github" / "workflows" / "browser-image.yml").read_text(
            encoding="utf-8"
        )

        self.assertEqual(sum(map(len, VERIFY.EXPECTED.values())), 35)
        self.assertGreaterEqual(
            workflow.count("- 'scripts/verify-browser-jvm-contracts.py'"), 2
        )
        self.assertGreaterEqual(
            workflow.count("- 'src/test/python/browser_jvm_contracts_test.py'"), 2
        )
        self.assertIn("Verify redirect, resources, and streaming in Chromium", workflow)
        self.assertIn("Verify Camoufox GET/POST, scripts, cookies, and sourceRegex", workflow)
        self.assertIn("scripts/verify-camoufox-contracts.py", workflow)

        matrix_step = workflow.split(
            "      - name: Verify all 35 browser JVM contracts actually executed", 1
        )[1].split("      - name: Preserve Camoufox contract report", 1)[0]
        self.assertIn("if: always()", matrix_step)
        self.assertIn(
            'local_report="$RUNNER_TEMP/local-chromium-contract-report/'
            "TEST-com.htmake.reader.utils.LocalWebviewRendererTest.xml\"",
            matrix_step,
        )
        self.assertIn(
            'camoufox_report="build/test-results/test/'
            "TEST-com.htmake.reader.utils.CamoufoxWebviewRendererTest.xml\"",
            matrix_step,
        )
        self.assertIn("python3 scripts/verify-browser-jvm-contracts.py", matrix_step)
        self.assertIn('--results-dir "$report_dir"', matrix_step)
        self.assertIn('--json-output "$report_dir/SUMMARY.json"', matrix_step)
        self.assertIn("name: browser-jvm-contract-report", matrix_step)
        self.assertIn("path: ${{ runner.temp }}/browser-jvm-contract-report/", matrix_step)


if __name__ == "__main__":
    unittest.main()
