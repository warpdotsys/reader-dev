"""Synthetic report rejection; no actual browser acceptance is claimed here."""

import copy
import importlib.util
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location("source_document_report",
    Path(__file__).parents[3] / "scripts/verify-source-document-observation.py")
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


class SourceDocumentObservationReportTest(unittest.TestCase):
    def report(self):
        return {"schemaVersion": 1, "scope": "generated helper-level pinned browser; not Reader parity",
            "networkDenials": 0, "passed": True, "cases": [
                {"case": name, "documentObservation": observation,
                 "mainFrameNavigationObserved": navigation, "sourceRuns": 1, "passed": True}
                for name, (observation, navigation) in GATE.CASES.items()]}

    def test_accepts_exact_three_named_cases_with_correct_observations(self):
        GATE.verify_report(self.report())

    def test_missing_duplicate_or_unknown_case_is_rejected(self):
        for change in (lambda r: r["cases"].pop(),
                       lambda r: r["cases"].__setitem__(1, copy.deepcopy(r["cases"][0])),
                       lambda r: r["cases"][0].__setitem__("case", "PRIVATE_URL")):
            report = self.report()
            change(report)
            with self.assertRaises(ValueError):
                GATE.verify_report(report)

    def test_failure_replay_wrong_document_or_integer_navigation_cannot_pass(self):
        for change in ({"passed": False}, {"sourceRuns": 2}, {"sourceRuns": True},
                       {"mainFrameNavigationObserved": 0}, {"documentObservation": "unavailable"}):
            report = self.report()
            report["cases"][0].update(change)
            with self.assertRaises(ValueError):
                GATE.verify_report(report)

    def test_raw_data_unknown_fields_or_network_denial_are_rejected(self):
        for change in (lambda r: r.__setitem__("rawHtml", "PRIVATE_BODY"),
                       lambda r: r["cases"][0].__setitem__("cookie", "PRIVATE_COOKIE"),
                       lambda r: r.__setitem__("networkDenials", 1),
                       lambda r: r.__setitem__("networkDenials", False),
                       lambda r: r.__setitem__("schemaVersion", True)):
            report = self.report()
            change(report)
            with self.assertRaises(ValueError):
                GATE.verify_report(report)

    def test_actual_gate_is_explicit_and_both_workflows_preserve_small_report(self):
        root = Path(__file__).parents[3]
        for workflow in ("release-native.yml", "browser-image.yml"):
            text = (root / ".github/workflows" / workflow).read_text(encoding="utf-8")
            self.assertIn('timeout 60s "$READER_CAMOUFOX_PYTHON" scripts/verify-source-document-observation.py', text)
            self.assertIn('build/source-document-observation.json', text)
            self.assertIn("if-no-files-found: error", text)
        self.assertNotIn("skip", (root / "scripts/verify-source-document-observation.py").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
