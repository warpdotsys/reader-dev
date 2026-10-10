"""Generated log filtering checks; no server logs or real browser required."""

import importlib.util
import io
import json
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location("browser_failure_collector",
    Path(__file__).parents[3] / "scripts/collect-browser-failure.py")
COLLECTOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COLLECTOR)


class BrowserFailureCollectorTest(unittest.TestCase):
    def state(self):
        return {"operation": "sourceScriptRead", "kind": "sourceStateMissing",
                "errorClass": "SourceScriptStateLost", "mainFrameNavigationObserved": False}

    def line(self, value):
        return COLLECTOR.PREFIX + json.dumps(value).encode() + b"\n"

    def test_accepts_fixed_state_and_existing_browser_labels_only(self):
        ordinary = {"operation": "initialNavigation", "kind": "navigationInterrupted", "errorClass": "Error"}
        report = COLLECTOR.collect(io.BytesIO(self.line(self.state()) + self.line(ordinary)))
        self.assertEqual([self.state(), ordinary], report["records"])
        self.assertFalse(report["rawLogsPersisted"])
        self.assertFalse(report["truncated"])

    def test_raw_logs_urls_credentials_and_unknown_json_are_not_persisted(self):
        raw = b"PRIVATE_BODY https://example.org/?cookie=PRIVATE_COOKIE\n"
        for change in ({"kind": "PRIVATE_BODY"}, {"operation": "PRIVATE_COOKIE"},
                       {"errorClass": "PRIVATE_CLASS"}, {"rawUrl": "PRIVATE_URL"},
                       {"mainFrameNavigationObserved": 1}, {"mainFrameNavigationObserved": "true"},
                       {"operation": "initialNavigation"}, {"kind": []}):
            raw += self.line(dict(self.state(), **change))
        raw += COLLECTOR.PREFIX + b'[{"PRIVATE":"BODY"}]\n'
        report = COLLECTOR.collect(io.BytesIO(raw))
        self.assertEqual([], report["records"])
        self.assertNotIn("PRIVATE", json.dumps(report))

    def test_accepts_only_three_optional_document_observation_labels(self):
        for label in ("sameDocument", "differentDocument", "unavailable"):
            value = dict(self.state(), sourceDocumentObservation=label)
            self.assertEqual([value], COLLECTOR.collect(io.BytesIO(self.line(value)))["records"])
        for label in ("PRIVATE_COOKIE", "https://private.invalid", None, 1, True, [], {}):
            value = dict(self.state(), sourceDocumentObservation=label)
            self.assertEqual([], COLLECTOR.collect(io.BytesIO(self.line(value)))["records"])

    def test_optional_document_label_rejects_unknown_and_duplicate_fields(self):
        value = dict(self.state(), sourceDocumentObservation="sameDocument")
        self.assertEqual([], COLLECTOR.collect(io.BytesIO(self.line(dict(value, rawDocument="PRIVATE_BODY"))))["records"])
        duplicate = self.line(value).rstrip()[:-1] + b',"sourceDocumentObservation":"unavailable"}\n'
        self.assertEqual([], COLLECTOR.collect(io.BytesIO(duplicate))["records"])

    def test_duplicate_fields_invalid_utf8_and_trailing_raw_text_are_rejected(self):
        line = self.line(self.state()).rstrip()
        for raw in (line[:-1] + b',"kind":"sourceStateMissing"}\n',
                    line + b" PRIVATE_COOKIE\n", COLLECTOR.PREFIX + b"\xff\n",
                    b"prefix " + line + b"\n"):
            self.assertEqual([], COLLECTOR.collect(io.BytesIO(raw))["records"])

    def test_long_log_line_cannot_inject_a_label_from_its_continuation(self):
        raw = b"x" * (COLLECTOR.MAX_LINE_BYTES + 1) + self.line(self.state())
        report = COLLECTOR.collect(io.BytesIO(raw + self.line(self.state())))
        self.assertEqual([self.state()], report["records"])

    def test_total_input_budget_and_record_count_are_bounded(self):
        report = COLLECTOR.collect(io.BytesIO(self.line(self.state()) * 9))
        self.assertEqual(8, len(report["records"]))
        self.assertTrue(report["truncated"])
        report = COLLECTOR.collect(io.BytesIO(b"x" * (COLLECTOR.MAX_INPUT_BYTES + 1)))
        self.assertEqual([], report["records"])
        self.assertTrue(report["truncated"])

    def test_wrapper_filters_only_owned_container_logs_without_echoing_them(self):
        wrapper = (Path(__file__).parents[3] / "scripts/probe-public-native-image.sh").read_text(encoding="utf-8")
        self.assertIn('docker logs --tail 200 "$container_id" 2>&1 |', wrapper)
        self.assertIn('python3 scripts/collect-browser-failure.py "$output/WORKER_DIAGNOSTICS.json"', wrapper)
        self.assertNotIn('docker logs "$container_id"', wrapper)


if __name__ == "__main__":
    unittest.main()
