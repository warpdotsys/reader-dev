"""Synthetic XML validates the release guard, not actual browser compatibility."""

import importlib.util
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET


SCRIPT = Path(__file__).parents[3] / "scripts" / "verify-camoufox-contracts.py"
SPEC = importlib.util.spec_from_file_location("camoufox_contract_guard", SCRIPT)
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)


class CamoufoxContractReportTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="reader-generated-contract-report-")
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "generated.xml"
        self.suite = ET.Element("testsuite", name=GUARD.SUITE,
                                tests=str(len(GUARD.CONTRACTS)), failures="0", errors="0", skipped="0")
        for name in sorted(GUARD.CONTRACTS):
            ET.SubElement(self.suite, "testcase", name=name, classname=GUARD.SUITE)

    def verify(self):
        ET.ElementTree(self.suite).write(self.path, encoding="utf-8", xml_declaration=True)
        return GUARD.verify_report(self.path)

    def test_complete_synthetic_report_exercises_only_the_guard(self):
        result = self.verify()
        self.assertEqual(20, result["tests"])
        self.assertTrue(result["navigationContractPresent"])
        self.assertTrue(result["earlyNavigationContractPresent"])
        self.assertTrue(result["endlessNavigationContractPresent"])
        self.assertTrue(result["promiseContractPresent"])
        self.assertTrue(result["stateDeletionContractPresent"])
        self.assertTrue(result["sourceNavigationContractPresent"])
        self.assertTrue(result["numericalThreadContractPresent"])
        self.assertTrue(result["utf8PostContractPresent"])
        self.assertEqual(64, len(result["xmlSha256"]))

    def test_all_environment_gated_cases_are_rejected(self):
        self.suite.set("skipped", str(len(GUARD.CONTRACTS)))
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_actual_navigation_failure_counter_is_rejected(self):
        self.suite.set("failures", "1")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_old_twelve_contracts_cannot_replace_navigation_acceptance(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.SOURCE_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.STATE_DELETION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.EARLY_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.ENDLESS_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.PROMISE_CONTRACT))
        self.suite.set("tests", "12")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_thirteen_contracts_do_not_prove_endless_navigation(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.SOURCE_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.STATE_DELETION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.EARLY_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.ENDLESS_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.PROMISE_CONTRACT))
        self.suite.set("tests", "13")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_fourteen_contracts_do_not_prove_promise_budget_or_recovery(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.SOURCE_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.STATE_DELETION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.EARLY_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.PROMISE_CONTRACT))
        self.suite.set("tests", "14")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_fifteen_contracts_do_not_prove_navigation_before_dom_ready(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.SOURCE_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.STATE_DELETION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.EARLY_NAVIGATION_CONTRACT))
        self.suite.set("tests", "15")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_sixteen_contracts_do_not_prove_state_loss_non_replay_and_recovery(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.SOURCE_NAVIGATION_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.STATE_DELETION_CONTRACT))
        self.suite.set("tests", "16")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_seventeen_contracts_do_not_prove_source_rule_load_document_timing(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.SOURCE_NAVIGATION_CONTRACT))
        self.suite.set("tests", "17")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_eighteen_contracts_do_not_prove_native_numerical_thread_budget(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT))
        self.suite.set("tests", "18")
        self.assertEqual(18, len(self.suite.findall("testcase")))
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_previous_nineteen_contracts_do_not_prove_raw_utf8_post_or_script_result(self):
        self.suite.remove(next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT))
        self.suite.set("tests", "19")
        self.assertEqual(19, len(self.suite.findall("testcase")))
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_skipped_utf8_post_is_rejected_even_with_forged_zero_counters(self):
        case = next(case for case in self.suite if case.get("name") == GUARD.UTF8_POST_CONTRACT)
        ET.SubElement(case, "skipped")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_skipped_numerical_import_is_rejected_even_with_forged_zero_counters(self):
        case = next(case for case in self.suite if case.get("name") == GUARD.NUMERICAL_THREAD_CONTRACT)
        ET.SubElement(case, "skipped")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_duplicate_case_cannot_preserve_the_expected_total(self):
        self.suite[0].set("name", self.suite[1].get("name"))
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_forged_zero_counters_do_not_hide_case_results(self):
        for tag in ("failure", "error", "skipped"):
            with self.subTest(tag=tag):
                marker = ET.SubElement(self.suite[0], tag)
                with self.assertRaises(GUARD.ContractReportError):
                    self.verify()
                self.suite[0].remove(marker)

    def test_missing_or_negative_counters_are_rejected(self):
        for value in ("", "-1", "0.0"):
            with self.subTest(value=value):
                self.suite.set("errors", value)
                with self.assertRaises(GUARD.ContractReportError):
                    self.verify()

    def test_wrong_suite_or_case_class_is_rejected(self):
        self.suite.set("name", "generated-wrong-suite")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()
        self.suite.set("name", GUARD.SUITE)
        self.suite[0].set("classname", "generated-wrong-case-class")
        with self.assertRaises(GUARD.ContractReportError):
            self.verify()

    def test_missing_malformed_and_oversized_files_are_rejected(self):
        with self.assertRaises(OSError):
            GUARD.verify_report(self.path)
        self.path.write_bytes(b"<generated-incomplete")
        with self.assertRaises(GUARD.ContractReportError):
            GUARD.verify_report(self.path)
        self.path.write_bytes(b"x" * (GUARD.MAX_REPORT_BYTES + 1))
        with self.assertRaises(GUARD.ContractReportError):
            GUARD.verify_report(self.path)


if __name__ == "__main__":
    unittest.main()
