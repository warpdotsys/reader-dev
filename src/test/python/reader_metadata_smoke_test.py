"""Finite guard and wire/helper doubles, not actual browser or real-site evidence."""

import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import threading
import unittest
import urllib.request
from unittest.mock import patch


ROOT = Path(__file__).parents[3]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SMOKE = load("metadata_smoke", "scripts/smoke-local-webview.py")
GUARD = load("metadata_smoke_guard", "scripts/verify-reader-metadata-smoke.py")


def report():
    return {"metadataReader": {"schemaVersion": 1, "scope": "generated-delayed-dom",
        "bookInfoApiCalls": 1, "status": 200, "isSuccess": True, "errorMsg": "",
        "dataIsObject": True, "nameMatches": True, "authorMatches": True, "coverMatches": True,
        "pageDiagnostics": dict(GUARD.DIAGNOSTICS),
        "sourceDefinitionRoundtrip": {key: True for key in GUARD.ROUNDTRIP_KEYS},
        "sourceScriptSha256": GUARD.SOURCE_SCRIPT_SHA256,
        "diagnosticRuleSha256": GUARD.DIAGNOSTIC_RULE_SHA256,
        "targetRequestCount": 1,
        "targetRequest": {"httpMethod": "GET", "body": None, "testHeader": None, "cookie": ""}}}


class MetadataGuardTest(unittest.TestCase):
    def reject(self, value):
        with self.assertRaises(GUARD.MetadataSmokeRejected):
            GUARD.verify(value)

    def test_generated_json_only_tests_guard_not_site_or_browser(self):
        value = GUARD.verify(report())
        self.assertEqual(1, value["generatedBrowserBackedMetadataCases"])
        self.assertFalse(value["realSiteMetadataOrAuthenticationProven"])

    def test_old_smoke_success_cannot_prove_detail_flow(self):
        for value in ({}, {"asyncReaderCases": [{"isSuccess": True}]}, None, []):
            self.reject(value)

    def test_status_and_single_call_counts_are_typed_and_exact(self):
        for key in ("schemaVersion", "bookInfoApiCalls", "status", "targetRequestCount"):
            for wrong in (True, 0, "1", 2, None):
                value = report()
                value["metadataReader"][key] = wrong
                self.reject(value)

    def test_empty_shell_or_false_or_coerced_metadata_is_rejected(self):
        for key in ("isSuccess", "dataIsObject", "nameMatches", "authorMatches", "coverMatches"):
            for wrong in (False, 1, "true", None):
                value = report()
                value["metadataReader"][key] = wrong
                self.reject(value)

    def test_error_scope_or_unreviewed_scripts_are_rejected(self):
        for key, wrong in (("errorMsg", None), ("errorMsg", "PRIVATE"),
                           ("scope", "real-site"), ("sourceScriptSha256", "0" * 64),
                           ("diagnosticRuleSha256", "0" * 64)):
            value = report()
            value["metadataReader"][key] = wrong
            self.reject(value)

    def test_missing_extra_or_raw_metadata_fields_are_rejected(self):
        for key in GUARD.KEYS:
            value = report()
            del value["metadataReader"][key]
            self.reject(value)
        value = report()
        value["metadataReader"]["rawHtml"] = "PRIVATE"
        self.reject(value)

    def test_snapshot_flags_are_all_exact_booleans_not_visibility_claims(self):
        for key, expected in GUARD.DIAGNOSTICS.items():
            for wrong in (not expected, int(expected), "true", None):
                value = report()
                value["metadataReader"]["pageDiagnostics"][key] = wrong
                self.reject(value)
        for wrong in ({}, {**GUARD.DIAGNOSTICS, "rawText": "PRIVATE"}, None):
            value = report()
            value["metadataReader"]["pageDiagnostics"] = wrong
            self.reject(value)

    def test_saved_source_roundtrip_is_required_not_an_asserted_passed_bit(self):
        for key in GUARD.ROUNDTRIP_KEYS:
            for wrong in (False, 1, "true", None):
                value = report()
                value["metadataReader"]["sourceDefinitionRoundtrip"][key] = wrong
                self.reject(value)
        value = report()
        value["metadataReader"]["sourceDefinitionRoundtrip"] = {"passed": True}
        self.reject(value)

    def test_method_body_header_cookie_and_no_extra_target_fields(self):
        for key, wrong in (("httpMethod", "POST"), ("body", "PRIVATE"),
                           ("testHeader", "unexpected"), ("cookie", "PRIVATE"), ("url", "PRIVATE")):
            value = report()
            value["metadataReader"]["targetRequest"][key] = wrong
            self.reject(value)

    def test_report_bytes_json_duplicates_and_exact_success(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.json"
            for raw in (b"{", b"\xff", b"x" * (GUARD.MAX_BYTES + 1),
                        b'{"metadataReader":null,"metadataReader":null}'):
                path.write_bytes(raw)
                with self.assertRaises(GUARD.MetadataSmokeRejected):
                    GUARD.verify_file(path)
            raw = json.dumps(report()).encode()
            path.write_bytes(raw)
            self.assertEqual(hashlib.sha256(raw).hexdigest(), GUARD.verify_file(path)["jsonSha256"])

    def test_cli_failure_does_not_echo_raw_fields(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.json"
            path.write_text('{"PRIVATE":"SECRET"}', encoding="utf-8")
            with patch("sys.argv", ["guard", str(path)]), patch("sys.stderr", new_callable=io.StringIO) as error:
                with self.assertRaises(SystemExit) as caught:
                    GUARD.main()
            self.assertEqual(1, caught.exception.code)
            self.assertNotIn("SECRET", error.getvalue())


class MetadataFixtureTest(unittest.TestCase):
    def test_definitions_are_exact_reviewed_public_probe_not_substituted(self):
        self.assertEqual(GUARD.SOURCE_SCRIPT_SHA256,
            hashlib.sha256(SMOKE.METADATA.METADATA_DOM_SCRIPT.encode()).hexdigest())
        self.assertEqual(GUARD.DIAGNOSTIC_RULE_SHA256,
            hashlib.sha256(SMOKE.METADATA.PAGE_DIAGNOSTIC_RULE.encode()).hexdigest())
        self.assertNotIn("fetch(", SMOKE.METADATA.METADATA_DOM_SCRIPT)
        self.assertNotIn("document.cookie", SMOKE.METADATA.METADATA_DOM_SCRIPT)

    def test_loopback_wire_has_delayed_markup_and_no_search_cookie_or_body(self):
        fixture = SMOKE.Fixture(("127.0.0.1", 0))
        thread = threading.Thread(target=fixture.serve_forever, daemon=True)
        thread.start()
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        base = f"http://127.0.0.1:{fixture.server_port}"
        try:
            with opener.open(base + "/book-info") as response:
                raw = response.read().decode("utf-8")
                self.assertIsNone(response.headers.get("Set-Cookie"))
            self.assertIn("setTimeout", raw)
            self.assertIn("300", raw)
            self.assertEqual('<html><body><main id="generated-metadata"></main>', raw.split('<script>')[0])
            self.assertNotIn("<h1", raw.split('<script>')[0])
            self.assertEqual([], fixture.cookies)
            self.assertEqual([], fixture.requests)
            self.assertEqual([report()["metadataReader"]["targetRequest"]], fixture.metadata_requests)
            with opener.open(base + "/generated-cover.svg") as response:
                self.assertEqual("image/svg+xml", response.headers.get("Content-Type"))
                self.assertLess(len(response.read()), 100)
            self.assertEqual(1, len(fixture.metadata_requests))
        finally:
            fixture.shutdown()
            fixture.server_close()
            thread.join(timeout=3)

    def run_double(self, fault=None):
        fixture = SimpleNamespace(server_port=1, metadata_requests=[])
        saved = None
        calls = []
        def call(_opener, _base, path, body):
            nonlocal saved
            calls.append(path)
            if path == "/reader3/saveBookSource":
                saved = copy.deepcopy(body)
                return {"isSuccess": True, "errorMsg": "", "data": {}}
            if path == "/reader3/getBookSource":
                if fault == "lost-rule":
                    saved["ruleBookInfo"].pop("intro")
                return {"isSuccess": True, "errorMsg": "", "data": saved}
            self.assertEqual("/reader3/getBookInfo", path)
            self.assertEqual("http://127.0.0.1:1/metadata-source", body["bookSourceUrl"])
            target, options = body["url"].split(", ", 1)
            self.assertEqual("http://127.0.0.1:1/book-info", target)
            self.assertEqual({"webView": True, "webJs": SMOKE.METADATA.METADATA_DOM_SCRIPT}, json.loads(options))
            self.assertNotIn("infoHtml", body)
            fixture.metadata_requests.append(copy.deepcopy(report()["metadataReader"]["targetRequest"]))
            if fault == "replay":
                fixture.metadata_requests.append(fixture.metadata_requests[0])
            if fault == "cookie":
                fixture.metadata_requests[0]["cookie"] = "PRIVATE"
            data = {"name": "黎明之剑", "author": "远瞳",
                "coverUrl": "http://127.0.0.1:1/generated-cover.svg", "intro": json.dumps(GUARD.DIAGNOSTICS)}
            if fault in ("name", "author", "coverUrl"):
                data[fault] = ""
            if fault == "snapshot":
                data["intro"] = json.dumps({key: False for key in GUARD.DIAGNOSTICS})
            if fault == "empty-shell":
                data = {}
            return {"isSuccess": True, "errorMsg": "", "data": data}
        with patch.object(SMOKE, "call", side_effect=call):
            value = SMOKE.exercise_metadata_reader(object(), "http://127.0.0.1:18890", fixture)
        self.assertEqual(["/reader3/saveBookSource", "/reader3/getBookSource", "/reader3/getBookInfo"], calls)
        return value

    def test_full_positive_helper_double_is_not_actual_browser_acceptance(self):
        GUARD.verify({"metadataReader": self.run_double()})

    def test_dropped_rule_empty_shell_missing_metadata_or_empty_snapshot_fails(self):
        for fault in ("lost-rule", "empty-shell", "name", "author", "coverUrl", "snapshot"):
            with self.assertRaises((RuntimeError, ValueError)):
                self.run_double(fault)

    def test_real_target_replay_or_nonempty_cookie_fails(self):
        for fault in ("replay", "cookie"):
            with self.assertRaises(ValueError):
                self.run_double(fault)


if __name__ == "__main__":
    unittest.main()
