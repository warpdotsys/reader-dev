"""No-network safety checks, not a substitute for a real default-browser run."""

import importlib.util
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
from types import SimpleNamespace
import tempfile
import threading
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("public_metadata_probe", ROOT / "scripts/probe-public-metadata.py")
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


class MetadataSummaryTest(unittest.TestCase):
    def value(self):
        return {"isSuccess": True, "errorMsg": "", "data": {
            "name": "黎明之剑", "author": "远瞳",
            "intro": json.dumps({key: False for key in PROBE.PAGE_DIAGNOSTIC_KEYS}),
            "coverUrl": "https://bookcover.yuewen.com/qdbimg/349573/1010400217/180"}}

    def test_expected_metadata_is_required_beyond_success_envelope(self):
        self.assertTrue(PROBE.summarize(200, self.value())["passed"])
        self.assertFalse(PROBE.summarize(200, {"isSuccess": True, "errorMsg": "", "data": {}})["passed"])

    def test_http_boolean_and_error_are_not_coerced(self):
        for status, success, error in ((503, True, ""), (200, "true", ""),
                                       (200, 1, ""), (200, True, None), (200, True, "failure")):
            value = self.value()
            value.update(isSuccess=success, errorMsg=error)
            self.assertFalse(PROBE.summarize(status, value)["passed"])

    def test_names_and_cover_are_not_faked_or_widened(self):
        for name, author, cover in (("wrong", "远瞳", "https://bookcover.yuewen.com/a"),
                                   ("黎明之剑", "wrong", "https://bookcover.yuewen.com/a"),
                                   ("黎明之剑", "远瞳", "http://bookcover.yuewen.com/a"),
                                   ("黎明之剑", "远瞳", "https://127.0.0.1/a"),
                                   ("黎明之剑", "远瞳", "https://token@bookcover.yuewen.com/a")):
            value = self.value()
            value["data"].update(name=name, author=author, coverUrl=cover)
            self.assertFalse(PROBE.summarize(200, value)["passed"])

    def test_raw_errors_urls_unknown_fields_and_metadata_values_do_not_escape(self):
        value = self.value()
        value["errorMsg"] = "PRIVATE_COOKIE_VALUE reason=DNS_FAILURE hostSha256=ad4a9c600a0e9679 https://x/?ticket=PRIVATE_TICKET"
        value["data"]["PRIVATE_FIELD_VALUE"] = "PRIVATE_BODY_VALUE"
        result = PROBE.summarize(200, value)
        encoded = json.dumps(result, ensure_ascii=False)
        for text in ("PRIVATE", "https://", "ticket=", "黎明之剑", "远瞳"):
            self.assertNotIn(text, encoded)
        self.assertEqual("DNS_FAILURE", result["policyReason"])
        self.assertEqual("ad4a9c600a0e9679", result["hostFingerprint"])

    def test_nonobject_metadata_is_a_failure_without_raw_echo(self):
        for data in (None, [], "PRIVATE_RESPONSE"):
            result = PROBE.summarize(200, {"isSuccess": True, "errorMsg": "", "data": data})
            self.assertFalse(result["passed"])
            self.assertNotIn("PRIVATE_RESPONSE", json.dumps(result))

    def test_fixed_worker_error_classes_are_classified_without_copying_the_wrapper(self):
        for category in ("SourceScriptStateLost", "SourceScriptTimeout", "SourceScriptRejected",
                         "TimeoutError", "ResponseBodyTooLarge", "ResponseTooLarge",
                         "CookieLimitExceeded", "Error"):
            for prefix in ("", "java.lang.IllegalStateException: "):
                value = {"isSuccess": False, "errorMsg": prefix + "Camoufox 渲染失败 (" + category + ")", "data": None}
                result = PROBE.summarize(200, value)
                self.assertEqual(category, result["workerErrorCategory"])
                self.assertFalse(result["passed"])
                self.assertNotIn("Camoufox 渲染失败", json.dumps(result, ensure_ascii=False))

    def test_unknown_or_credential_bearing_error_text_is_not_classified_or_echoed(self):
        for error in ("Camoufox 渲染失败 (PRIVATE_CLASS)",
                      "https://example.org/?ticket=PRIVATE Camoufox 渲染失败 (SourceScriptStateLost)",
                      "Camoufox 渲染失败 (SourceScriptRejected) PRIVATE_COOKIE",
                      "Camoufox 渲染失败 (SourceScriptTimeout)\nPRIVATE_BODY"):
            result = PROBE.summarize(200, {"isSuccess": False, "errorMsg": error, "data": None})
            self.assertIsNone(result["workerErrorCategory"])
            self.assertNotIn("PRIVATE", json.dumps(result))

    def test_successful_metadata_does_not_gain_an_inferred_worker_error(self):
        self.assertIsNone(PROBE.summarize(200, self.value())["workerErrorCategory"])

    def test_nonempty_mismatch_is_distinct_from_empty_metadata_without_echo(self):
        value = self.value()
        value["data"].update(name="PRIVATE_NAME", author="PRIVATE_AUTHOR", coverUrl="PRIVATE_COVER")
        result = PROBE.summarize(200, value)
        for key in ("nameHasText", "authorHasText", "coverHasText"):
            self.assertIs(result[key], True)
        for key in ("nameMatches", "authorMatches", "coverHasExpectedPublicOrigin"):
            self.assertIs(result[key], False)
        self.assertNotIn("PRIVATE", json.dumps(result))
        for blank in (None, 1, [], " \n "):
            value["data"].update(name=blank, author=blank, coverUrl=blank)
            result = PROBE.summarize(200, value)
            self.assertFalse(any(result[key] for key in ("nameHasText", "authorHasText", "coverHasText")))

    def test_diagnostics_accept_only_exact_fixed_boolean_fields(self):
        fields = {key: False for key in PROBE.PAGE_DIAGNOSTIC_KEYS}
        self.assertEqual(fields, PROBE.page_diagnostics(json.dumps(fields)))
        for invalid in (None, {}, "PRIVATE_HTML", "[]", "x" * 1025, "\ud800",
                        json.dumps({**fields, "PRIVATE_BODY": "PRIVATE_VALUE"}),
                        json.dumps({key: 0 for key in fields}),
                        json.dumps({key: "true" for key in fields}),
                        json.dumps({key: value for key, value in fields.items() if key != "bodyHasText"}),
                        json.dumps(fields)[:-1] + ',"bodyHasText":true}'):
            self.assertIsNone(PROBE.page_diagnostics(invalid))

    def test_safety_hints_never_turn_invalid_metadata_into_success(self):
        fields = {key: True for key in PROBE.PAGE_DIAGNOSTIC_KEYS}
        result = PROBE.summarize(200, {"isSuccess": True, "errorMsg": "", "data": {
            "intro": json.dumps(fields)}})
        self.assertEqual(fields, result["pageDiagnostics"])
        self.assertFalse(result["passed"])


class MetadataWaitJavaScriptTest(unittest.TestCase):
    def test_exact_wait_script_with_generated_dom_and_clock_is_not_a_browser_proof(self):
        node = os.environ.get("READER_TEST_NODE") or shutil.which("node")
        self.assertTrue(node, "Node is required; exact probe JS must not be skipped")
        fixture = ROOT / "src/test/javascript/public-metadata-wait.test.mjs"
        completed = subprocess.run([node, "--test", "--test-reporter=tap", str(fixture)],
                                   capture_output=True, text=True, encoding="utf-8", timeout=10)
        self.assertEqual(0, completed.returncode, "Generated-only metadata JS checks failed")
        self.assertRegex(completed.stdout, re.compile(r"^# tests 8$", re.MULTILINE))
        for counter in ("fail", "cancelled", "skipped", "todo"):
            self.assertRegex(completed.stdout, re.compile(r"^# " + counter + r" 0$", re.MULTILINE))


class SnapshotStructureTest(unittest.TestCase):
    @staticmethod
    def envelope():
        return {"pageDiagnostics": {key: False for key in PROBE.PAGE_DIAGNOSTIC_KEYS},
                "snapshotStructure": {key: False for key in PROBE.SNAPSHOT_STRUCTURE_KEYS}}

    def test_default_control_rules_keep_their_exact_previously_tested_hashes(self):
        for rule, expected in ((PROBE.PAGE_DIAGNOSTIC_RULE,
                               "603a86357f45c3b850ec3c8cce2fe98266053aa1276fdf7178fbd5a4fe61166a"),
                              (PROBE.METADATA_DOM_SCRIPT,
                               "045f640185166e2d9415e09b1e2d574ba45cf6119345ba180f4fe6e5a9bf4b04")):
            self.assertEqual(expected, hashlib.sha256(rule.encode("utf-8")).hexdigest())

    def test_extended_envelope_is_strict_and_not_accepted_as_the_old_format(self):
        value = self.envelope()
        raw = json.dumps(value)
        self.assertEqual((value["pageDiagnostics"], value["snapshotStructure"]), PROBE.snapshot_observations(raw))
        self.assertIsNone(PROBE.page_diagnostics(raw))
        self.assertEqual((None, None), PROBE.snapshot_observations(json.dumps(value["pageDiagnostics"])))

    def test_unknown_missing_nonboolean_and_wrong_type_fields_are_rejected(self):
        for group in ("pageDiagnostics", "snapshotStructure"):
            for invalid in (None, [], "PRIVATE_HTML", {},
                            {**self.envelope()[group], "PRIVATE_FIELD": "PRIVATE_VALUE"},
                            {key: 1 for key in self.envelope()[group]},
                            {key: "false" for key in self.envelope()[group]}):
                value = self.envelope()
                value[group] = invalid
                self.assertEqual((None, None), PROBE.snapshot_observations(json.dumps(value)))
        for value in ({}, {"snapshotStructure": {}}, {**self.envelope(), "PRIVATE": "PRIVATE"}):
            self.assertEqual((None, None), PROBE.snapshot_observations(json.dumps(value)))

    def test_duplicate_nested_keys_utf8_limit_and_malformed_payloads_are_rejected(self):
        raw = json.dumps(self.envelope())
        duplicate = raw.replace('"rawEmpty": false', '"rawEmpty": false, "rawEmpty": true')
        for invalid in (None, "PRIVATE_HTML", "[]", "x" * 1025, "\ud800", duplicate,
                        raw[:-1] + ',"pageDiagnostics":{}}'):
            self.assertEqual((None, None), PROBE.snapshot_observations(invalid))

    def test_details_source_changes_only_intro_and_readback_requires_that_exact_rule(self):
        basic = PROBE.source_definition()
        details = PROBE.source_definition(True)
        self.assertEqual(PROBE.SNAPSHOT_STRUCTURE_RULE, details["ruleBookInfo"]["intro"])
        details["ruleBookInfo"]["intro"] = basic["ruleBookInfo"]["intro"]
        self.assertEqual(basic, details)
        self.assertTrue(PROBE.source_roundtrip((200, {"isSuccess": True, "data": PROBE.source_definition(True)}), True)["passed"])
        self.assertFalse(PROBE.source_roundtrip((200, {"isSuccess": True, "data": basic}), True)["passed"])
        for unsafe in ("fetch(", "ajax(", "document.cookie", "location.", "eval(", "outerHTML", "innerHTML"):
            self.assertNotIn(unsafe, PROBE.SNAPSHOT_STRUCTURE_RULE)

    def test_details_mode_is_not_coerced(self):
        for invalid in (None, 0, 1, "true", []):
            for operation in (lambda: PROBE.source_definition(invalid),
                              lambda: PROBE.source_roundtrip((200, {}), invalid),
                              lambda: PROBE.summarize(200, {}, invalid)):
                with self.assertRaisesRegex(PROBE.ProbeFailure, "InvalidSnapshotDetailsMode"):
                    operation()

    def test_structure_and_safety_hints_cannot_manufacture_metadata_success(self):
        value = self.envelope()
        value["snapshotStructure"] = {key: True for key in PROBE.SNAPSHOT_STRUCTURE_KEYS}
        result = PROBE.summarize(200, {"isSuccess": True, "errorMsg": "", "data": {
            "intro": json.dumps(value)}}, True)
        self.assertEqual(value["snapshotStructure"], result["snapshotStructure"])
        self.assertFalse(result["passed"])

    def test_untrusted_snapshot_or_metadata_content_is_never_echoed(self):
        value = MetadataSummaryTest().value()
        value["data"].update(intro="PRIVATE_HTML https://private/?ticket=PRIVATE", name="PRIVATE_NAME")
        result = PROBE.summarize(200, value, True)
        self.assertIsNone(result["snapshotStructure"])
        self.assertIsNone(result["pageDiagnostics"])
        self.assertNotIn("PRIVATE", json.dumps(result))

    def test_details_without_bounded_wait_fails_before_environment_or_requests(self):
        with patch("sys.argv", ["probe", "--expected-revision", "a" * 40, "--snapshot-details"]), \
                patch.object(PROBE, "require_environment") as environment, \
                patch.object(PROBE, "request_json") as requests:
            with self.assertRaisesRegex(PROBE.ProbeFailure, "SnapshotDetailsRequireBoundedWait"):
                PROBE.main()
        environment.assert_not_called()
        requests.assert_not_called()


class RequestBoundaryTest(unittest.TestCase):
    def test_baseline_still_uses_the_same_fixed_page_without_a_source_script(self):
        request = PROBE.book_info_request()
        url, raw = request["url"].split(", ", 1)
        self.assertEqual(PROBE.BOOK, url)
        self.assertEqual(PROBE.SOURCE, request["bookSourceUrl"])
        self.assertEqual({"webView": True}, json.loads(raw))

    def test_bounded_dom_wait_returns_unmodified_page_html_without_extra_requests(self):
        request = PROBE.book_info_request(True)
        url, raw = request["url"].split(", ", 1)
        options = json.loads(raw)
        self.assertEqual(PROBE.BOOK, url)
        self.assertEqual({"webView", "webJs"}, set(options))
        self.assertIs(options["webView"], True)
        self.assertEqual(PROBE.METADATA_DOM_SCRIPT, options["webJs"])
        script = options["webJs"]
        self.assertIn("new Promise", script)
        self.assertIn("performance.now() - started >= 8000", script)
        self.assertIn("resolve(document.documentElement.outerHTML)", script)
        self.assertIn("setTimeout(read, 100)", script)
        for unsafe in ("fetch(", "document.cookie", "location.", "innerHTML =", "textContent =",
                       "黎明之剑", "远瞳", "captcha", "geetest"):
            self.assertNotIn(unsafe, script)
        self.assertEqual(8000, PROBE.METADATA_DOM_WAIT_MS)

    def test_wait_mode_does_not_coerce_credentials_or_arbitrary_script_input(self):
        for value in (1, "true", "PRIVATE_COOKIE", {"webJs": "PRIVATE_SCRIPT"}, None):
            with self.assertRaisesRegex(PROBE.ProbeFailure, "^InvalidMetadataWaitMode$"):
                PROBE.book_info_request(value)

    def test_native_user_agent_is_only_an_empty_fixed_header_not_a_forged_profile(self):
        request = PROBE.book_info_request(browser_native_user_agent=True)
        url, raw = request['url'].split(', ', 1)
        self.assertEqual(PROBE.BOOK, url)
        self.assertEqual({'webView': True, 'headers': {'User-Agent': ''}}, json.loads(raw))
        for value in (1, 'Firefox/PRIVATE', {'Cookie': 'PRIVATE'}, None):
            with self.assertRaisesRegex(PROBE.ProbeFailure, '^InvalidUserAgentMode$'):
                PROBE.book_info_request(browser_native_user_agent=value)

    def test_native_user_agent_does_not_change_the_existing_bounded_script(self):
        request = PROBE.book_info_request(True, True)
        url, raw = request['url'].split(', ', 1)
        self.assertEqual(PROBE.BOOK, url)
        self.assertEqual({'webView': True, 'webJs': PROBE.METADATA_DOM_SCRIPT,
                          'headers': {'User-Agent': ''}}, json.loads(raw))

    def response(self, raw, status=200):
        response = io.BytesIO(raw)
        response.status = status
        return response

    def test_response_size_is_bounded_before_json_decode(self):
        with self.assertRaisesRegex(PROBE.ProbeFailure, "ReaderResponseTooLarge"):
            PROBE.read_response(self.response(b"x" * (PROBE.MAX_RESPONSE_BYTES + 1)))

    def test_malformed_json_is_not_echoed(self):
        with self.assertRaisesRegex(PROBE.ProbeFailure, "^MalformedReaderJson$"):
            PROBE.read_response(self.response(b"PRIVATE_BODY not json"))

    def test_nonobject_json_rejected(self):
        with self.assertRaisesRegex(PROBE.ProbeFailure, "^UnexpectedReaderJsonType$"):
            PROBE.read_response(self.response(b"[]"))

    def test_chapter_source_login_and_arbitrary_urls_are_not_allowed(self):
        for path in ("/getBookContent", "/getChapterList", "/setSourceLogin", "https://example.org/", "/getBookInfo?token=x"):
            with self.assertRaisesRegex(PROBE.ProbeFailure, "UnexpectedReaderEndpoint"):
                PROBE.request_json(None, path)

    def test_reader_redirect_never_sends_session_to_another_destination(self):
        with self.assertRaisesRegex(PROBE.ProbeFailure, "UnexpectedReaderRedirect"):
            PROBE.NoRedirect().redirect_request(None, None, 302, "ignored", {}, "https://example.org/")

    def test_fresh_cookie_index_must_have_expected_shape(self):
        self.assertEqual(0, PROBE.cookie_count((200, {"isSuccess": True, "data": []})))
        with self.assertRaises(PROBE.ProbeFailure):
            PROBE.cookie_count((200, {"isSuccess": True, "data": {}}))

    def test_legacy_logout_need_login_contract_and_actual_followup_are_required(self):
        logged_out = (200, {"isSuccess": True, "data": "NEED_LOGIN", "errorMsg": ""})
        protected_denied = (200, {"isSuccess": False, "data": "NEED_LOGIN", "errorMsg": "请登录后使用"})
        PROBE.verify_cookie_session_logged_out(logged_out, protected_denied)
        still_logged_in = (200, {"isSuccess": True, "data": []})
        with self.assertRaises(PROBE.ProbeFailure):
            PROBE.verify_cookie_session_logged_out(logged_out, still_logged_in)
        with self.assertRaises(PROBE.ProbeFailure):
            PROBE.verify_cookie_session_logged_out(still_logged_in, protected_denied)
        with self.assertRaises(PROBE.ProbeFailure):
            PROBE.verify_cookie_session_logged_out(protected_denied, protected_denied)

    def test_source_is_fixed_metadata_only_without_credentials_or_chapter_rules(self):
        source = PROBE.source_definition()
        self.assertEqual("https://www.qidian.com", source["bookSourceUrl"])
        self.assertFalse(source["enabledCookieJar"])
        self.assertEqual({}, source["ruleToc"])
        for key in ("ruleContent", "header", "loginUrl", "loginInfo", "searchUrl"):
            self.assertNotIn(key, source)
        rules = source["ruleBookInfo"]
        self.assertEqual("#bookName@text", rules["name"])
        self.assertEqual(".book-info-top .book-meta .author@text", rules["author"])
        self.assertEqual("#bookImg img@src", rules["coverUrl"])
        self.assertEqual(PROBE.PAGE_DIAGNOSTIC_RULE, rules["intro"])
        for unsafe in ("ajax(", "fetch(", "cookie", "location", "eval(", "outerHTML", "innerHTML"):
            self.assertNotIn(unsafe, rules["intro"])

    def test_only_exact_loopback_endpoint_and_revision_are_accepted(self):
        for base, revision in (("https://read.medwarp.cn", "a" * 40),
                               (PROBE.BASE + "/", "a" * 40),
                               (PROBE.BASE, "main")):
            with self.assertRaisesRegex(PROBE.ProbeFailure, "InvalidProbeInput"):
                PROBE.require_environment(base, revision, Path("/verification-output"))

    def test_source_roundtrip_requires_the_exact_rules_and_modern_marker(self):
        value = {"isSuccess": True, "data": PROBE.source_definition()}
        self.assertTrue(PROBE.source_roundtrip((200, value))["passed"])
        for key in ("name", "author", "coverUrl", "intro"):
            source = PROBE.source_definition()
            source["ruleBookInfo"][key] = "PRIVATE_DIFFERENT_RULE"
            report = PROBE.source_roundtrip((200, {"isSuccess": True, "data": source}))
            self.assertFalse(report["passed"])
            self.assertFalse(report["metadataRulesMatch"])
            self.assertNotIn("PRIVATE", json.dumps(report))
        for patch_value in ({"ruleToc": None}, {"ruleToc": "{}"},
                            {"enabledCookieJar": 0}, {"enabledCookieJar": True},
                            {"bookSourceUrl": "https://PRIVATE_URL"}, {"ruleBookInfo": None}):
            source = PROBE.source_definition()
            source.update(patch_value)
            report = PROBE.source_roundtrip((200, {"isSuccess": True, "data": source}))
            self.assertFalse(report["passed"])
            self.assertNotIn("PRIVATE", json.dumps(report))
        for data in (None, "PRIVATE_RESPONSE", [], {}):
            report = PROBE.source_roundtrip((200, {"isSuccess": True, "data": data}))
            self.assertFalse(report["passed"])
            self.assertNotIn("PRIVATE", json.dumps(report))

    def test_runtime_does_not_fall_back_to_root_or_private_network_access(self):
        env = {"READER_BUILD_REVISION": "a" * 40, "READER_APP_WEBVIEWRENDERER": "camoufox",
               "READER_BROWSER_ALLOW_PRIVATE_NETWORKS": "false", "READER_SERVER_BINDADDRESS": "127.0.0.1",
               "READER_APP_SECURE": "true"}
        with patch.dict(PROBE.os.environ, env, clear=True), patch.object(PROBE.os, "getuid", return_value=0, create=True):
            with self.assertRaisesRegex(PROBE.ProbeFailure, "UnexpectedRuntimeIdentityOrGuard"):
                PROBE.require_environment(PROBE.BASE, "a" * 40, Path("/verification-output"))
        env["READER_BROWSER_ALLOW_PRIVATE_NETWORKS"] = "true"
        with patch.dict(PROBE.os.environ, env, clear=True), patch.object(PROBE.os, "getuid", return_value=10001, create=True):
            with self.assertRaisesRegex(PROBE.ProbeFailure, "UnexpectedRuntimeIdentityOrGuard"):
                PROBE.require_environment(PROBE.BASE, "a" * 40, Path("/verification-output"))


class LifecycleTest(unittest.TestCase):
    def run_generated_probe(self, metadata, fail_logout=False, transport_failure=False, wait_dom=False,
                            source_retained=True, snapshot_details=False, snapshot_only_details=False,
                            browser_native_user_agent=False):
        observed = threading.Event()
        paths = []
        logged_out = False
        details_requested = snapshot_details or snapshot_only_details

        def processes():
            observed.set()
            return {"worker": [1], "browser": [2], "driver": [3]}

        def request(opener, path, body=None):
            nonlocal logged_out
            paths.append(path)
            if path == "/saveBookSource":
                self.assertEqual(PROBE.source_definition(details_requested), body)
            if path == "/getBookSource":
                self.assertEqual({"bookSourceUrl": PROBE.SOURCE}, body)
                return 200, {"isSuccess": True, "data": PROBE.source_definition(details_requested) if source_retained else {}}
            if path == "/getBookInfo":
                expected_request = (PROBE.book_info_request(wait_dom, True) if browser_native_user_agent else
                                    PROBE.book_info_request(wait_dom))
                self.assertEqual(expected_request, body)
                self.assertTrue(observed.wait(1), "Generated process observer did not run")
                if transport_failure:
                    raise RuntimeError("PRIVATE_TRANSPORT_BODY https://x/?ticket=PRIVATE_TICKET")
                return 200, metadata
            if path == "/getBookSourceCookie":
                if logged_out:
                    return 200, {"isSuccess": False, "data": "NEED_LOGIN"}
                return 200, {"isSuccess": True, "data": []}
            if path == "/logout":
                if fail_logout:
                    return 200, {"isSuccess": False, "errorMsg": "PRIVATE_CLEANUP_BODY"}
                logged_out = True
                return 200, {"isSuccess": True, "data": "NEED_LOGIN", "errorMsg": ""}
            return 200, {"isSuccess": True, "data": {}}

        cgroup = SimpleNamespace(collect_report=lambda *args: {"scope": "generated unit fixture"},
                                 verify_report=lambda *args, **kwargs: None)
        helpers = [cgroup, SimpleNamespace(browser_processes=processes)]
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(PROBE, "require_environment"), \
                    patch.object(PROBE, "load_helper", side_effect=helpers), \
                    patch.object(PROBE, "request_json", side_effect=request), \
                    patch.object(PROBE.os, "getuid", return_value=10001, create=True), \
                    patch("sys.argv", ["probe", "--expected-revision", "a" * 40, "--output", directory]
                          + (["--wait-dom"] if wait_dom else [])
                          + (["--snapshot-details"] if snapshot_details else [])
                          + (["--snapshot-only-details"] if snapshot_only_details else [])
                          + (["--browser-native-user-agent"] if browser_native_user_agent else [])), \
                    patch("sys.stdout", new_callable=io.StringIO) as stdout:
                result = PROBE.main()
            report = json.loads((Path(directory) / "PUBLIC_METADATA_REPORT.json").read_text(encoding="utf-8"))
            encoded = stdout.getvalue() + json.dumps(report, ensure_ascii=False)
            self.assertNotIn("PRIVATE", encoded)
            self.assertEqual(1 if source_retained else 0, report["bookInfoApiCalls"])
            self.assertEqual(1 if source_retained else 0, paths.count("/getBookInfo"))
            self.assertEqual(1, paths.count("/getBookSource"))
            self.assertFalse(report["chapterBodyRequested"])
            self.assertFalse(report["realAuthenticationProven"])
            self.assertFalse(report["sourceScriptSynthesizesMetadata"])
            self.assertEqual('browser-native' if browser_native_user_agent else 'reader-default', report['userAgentMode'])
            self.assertIs(browser_native_user_agent, report['browserNativeUserAgentRequested'])
            self.assertFalse(report['browserUserAgentActuallyObserved'])
            self.assertEqual(8000 if wait_dom else 0, report["metadataDomWaitBudgetMs"])
            self.assertEqual("boundedMetadataDom" if wait_dom else "domContentLoadedOnly", report["sourceScriptMode"])
            self.assertEqual("snapshot-only-details" if snapshot_only_details else "bounded-dom-details" if snapshot_details else
                             "bounded-dom" if wait_dom else "snapshot-only", report["pageCaptureMode"])
            self.assertEqual(0, report["cookieRowsBefore"])
            self.assertEqual(0, report["cookieRowsAfter"])
            self.assertIn("/logout", paths)
            return result, report

    def test_rule_loss_is_a_red_setup_gate_before_any_public_navigation(self):
        result, report = self.run_generated_probe(MetadataSummaryTest().value(), source_retained=False)
        self.assertEqual(1, result)
        self.assertEqual("ProbeSourceRulesNotRetained", report["failureCategory"])
        self.assertFalse(report["sourceDefinitionRoundtrip"]["passed"])
        self.assertNotIn("metadata", report)
        self.assertFalse(report["defaultBrowserProcessObserved"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_success_shell_without_metadata_is_a_red_gate_after_cleanup(self):
        result, report = self.run_generated_probe({"isSuccess": True, "errorMsg": "", "data": {}})
        self.assertEqual(1, result)
        self.assertFalse(report["passed"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_native_user_agent_observation_keeps_one_request_and_all_original_guards(self):
        value = MetadataSummaryTest().value()
        value['data']['intro'] = json.dumps(SnapshotStructureTest.envelope())
        result, report = self.run_generated_probe(value, snapshot_only_details=True, browser_native_user_agent=True)
        self.assertEqual(0, result)
        self.assertEqual('browser-native', report['userAgentMode'])
        self.assertTrue(report['browserNativeUserAgentRequested'])
        self.assertFalse(report['browserUserAgentActuallyObserved'])
        self.assertEqual(1, report['bookInfoApiCalls'])
        self.assertEqual(0, report['metadataDomWaitBudgetMs'])

    def test_native_user_agent_does_not_turn_empty_metadata_into_a_pass(self):
        result, report = self.run_generated_probe({'isSuccess': True, 'errorMsg': '', 'data': {}},
                                                  snapshot_only_details=True, browser_native_user_agent=True)
        self.assertEqual(1, result)
        self.assertFalse(report['passed'])
        self.assertTrue(report['generatedCookieSessionRevokedVerified'])

    def test_no_wait_details_explicitly_require_structure_even_if_page_summary_is_valid(self):
        finite = {'passed': True, 'pageDiagnostics': {key: False for key in PROBE.PAGE_DIAGNOSTIC_KEYS},
                  'snapshotStructure': None}
        with patch.object(PROBE, 'summarize', return_value=finite):
            result, report = self.run_generated_probe(MetadataSummaryTest().value(), snapshot_only_details=True)
        self.assertEqual(1, result)
        self.assertFalse(report['passed'])
        self.assertTrue(report['generatedCookieSessionRevokedVerified'])

    def test_metadata_success_does_not_hide_session_cleanup_failure(self):
        result, report = self.run_generated_probe(MetadataSummaryTest().value(), fail_logout=True)
        self.assertEqual(1, result)
        self.assertTrue(report["metadata"]["passed"])
        self.assertFalse(report["passed"])

    def test_transport_failure_is_sanitized_and_still_clears_the_generated_session(self):
        result, report = self.run_generated_probe({}, transport_failure=True)
        self.assertEqual(1, result)
        self.assertEqual("ProbeTransportOrRuntimeFailure", report["failureCategory"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_expected_metadata_and_all_generated_safety_checks_can_pass(self):
        result, report = self.run_generated_probe(MetadataSummaryTest().value())
        self.assertEqual(0, result)
        self.assertTrue(report["passed"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_metadata_match_does_not_hide_missing_or_raw_page_diagnostics(self):
        for raw in (None, "PRIVATE_HTML"):
            value = MetadataSummaryTest().value()
            value["data"]["intro"] = raw
            result, report = self.run_generated_probe(value)
            self.assertEqual(1, result)
            self.assertTrue(report["metadata"]["passed"])
            self.assertFalse(report["passed"])
            self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_bounded_wait_still_rejects_empty_metadata_and_cleans_up(self):
        result, report = self.run_generated_probe({"isSuccess": True, "errorMsg": "", "data": {}}, wait_dom=True)
        self.assertEqual(1, result)
        self.assertFalse(report["passed"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_finite_details_and_metadata_require_all_original_safety_gates(self):
        value = MetadataSummaryTest().value()
        value["data"]["intro"] = json.dumps(SnapshotStructureTest.envelope())
        result, report = self.run_generated_probe(value, wait_dom=True, snapshot_details=True)
        self.assertEqual(0, result)
        self.assertTrue(report["snapshotStructureRequested"])
        self.assertEqual(hashlib.sha256(PROBE.SNAPSHOT_STRUCTURE_RULE.encode("utf-8")).hexdigest(),
                         report["snapshotStructureRuleSha256"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_details_mode_cannot_pass_with_old_or_missing_diagnostics(self):
        for raw in (MetadataSummaryTest().value()["data"]["intro"], None, "PRIVATE_HTML"):
            value = MetadataSummaryTest().value()
            value["data"]["intro"] = raw
            result, report = self.run_generated_probe(value, wait_dom=True, snapshot_details=True)
            self.assertEqual(1, result)
            self.assertTrue(report["metadata"]["passed"])
            self.assertFalse(report["passed"])
            self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_valid_structure_without_metadata_is_still_red_and_cleanup_runs(self):
        value = {"isSuccess": True, "errorMsg": "", "data": {
            "intro": json.dumps(SnapshotStructureTest.envelope())}}
        result, report = self.run_generated_probe(value, wait_dom=True, snapshot_details=True)
        self.assertEqual(1, result)
        self.assertIsNotNone(report["metadata"]["snapshotStructure"])
        self.assertFalse(report["passed"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_no_wait_details_preserve_one_request_zero_wait_and_all_original_guards(self):
        value = MetadataSummaryTest().value()
        value["data"]["intro"] = json.dumps(SnapshotStructureTest.envelope())
        result, report = self.run_generated_probe(value, snapshot_only_details=True)
        self.assertEqual(0, result)
        self.assertTrue(report["snapshotStructureRequested"])
        self.assertEqual("snapshot-only-details", report["pageCaptureMode"])
        self.assertEqual(0, report["metadataDomWaitBudgetMs"])
        self.assertNotIn("metadataDomScriptSha256", report)
        self.assertIsNotNone(report["metadata"]["snapshotStructure"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_no_wait_details_do_not_forge_empty_metadata_as_passed(self):
        value = {"isSuccess": True, "errorMsg": "", "data": {
            "intro": json.dumps(SnapshotStructureTest.envelope())}}
        result, report = self.run_generated_probe(value, snapshot_only_details=True)
        self.assertEqual(1, result)
        self.assertIsNotNone(report["metadata"]["snapshotStructure"])
        self.assertFalse(report["passed"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_no_wait_details_reject_old_missing_or_raw_diagnostics_and_still_cleanup(self):
        for raw in (MetadataSummaryTest().value()["data"]["intro"], None, "PRIVATE_HTML"):
            value = MetadataSummaryTest().value()
            value["data"]["intro"] = raw
            result, report = self.run_generated_probe(value, snapshot_only_details=True)
            self.assertEqual(1, result)
            self.assertTrue(report["metadata"]["passed"])
            self.assertIsNone(report["metadata"]["snapshotStructure"])
            self.assertFalse(report["passed"])
            self.assertTrue(report["generatedCookieSessionRevokedVerified"])

    def test_no_wait_details_refuse_conflicting_flags_before_any_runtime_or_account_action(self):
        for flags in (["--wait-dom"], ["--snapshot-details"], ["--wait-dom", "--snapshot-details"]):
            with patch("sys.argv", ["probe", "--expected-revision", "a" * 40,
                    "--snapshot-only-details"] + flags), \
                    patch.object(PROBE, "require_environment") as environment, \
                    patch.object(PROBE, "request_json") as request:
                with self.assertRaisesRegex(PROBE.ProbeFailure, "ConflictingSnapshotDetailsMode"):
                    PROBE.main()
                environment.assert_not_called()
                request.assert_not_called()


class HostedCaptureModeTest(unittest.TestCase):
    def test_user_agent_enum_is_fixed_and_rejected_before_artifact_or_container_actions(self):
        wrapper = (ROOT / 'scripts/probe-public-native-image.sh').read_text(encoding='utf-8')
        validation = 'case "$user_agent_mode" in\n  reader-default|browser-native) ;;\n  *) exit 1 ;;\nesac'
        self.assertIn(validation, wrapper)
        self.assertLess(wrapper.index(validation), wrapper.index('version=$(jq'))
        self.assertLess(wrapper.index(validation), wrapper.index('docker load'))
        self.assertIn('user_agent_mode="${5:-reader-default}"', wrapper)
        self.assertIn('test "$#" = 3 || test "$#" = 4 || test "$#" = 5', wrapper)
        self.assertIn('if [[ "$user_agent_mode" = browser-native ]]; then probe_args+=(--browser-native-user-agent); fi', wrapper)

    def test_workflow_user_agent_input_is_quoted_and_not_an_arbitrary_header(self):
        workflow = (ROOT / '.github/workflows/browser-image.yml').read_text(encoding='utf-8')
        self.assertIn('options: [reader-default, browser-native]', workflow)
        self.assertIn('default: reader-default', workflow)
        self.assertEqual(2, workflow.count('USER_AGENT_MODE: ${{ inputs.public_metadata_user_agent }}'))
        validation = '[[ "$USER_AGENT_MODE" = reader-default || "$USER_AGENT_MODE" = browser-native ]]'
        self.assertIn(validation, workflow)
        metadata_job = workflow.split('  public-metadata:\n', 1)[1]
        self.assertLess(metadata_job.index(validation), metadata_job.index('gh api "repos/$GITHUB_REPOSITORY/actions/runs/$NATIVE_RUN"'))
        self.assertIn('"$CAPTURE_MODE" "$USER_AGENT_MODE"', workflow)
        self.assertIn("${{ inputs.public_metadata_user_agent || 'reader-default' }}", workflow)
        self.assertNotIn('run: ${{ inputs.public_metadata_user_agent', workflow)

    def test_capture_enum_is_validated_before_any_container_or_artifact_action(self):
        wrapper = (ROOT / "scripts/probe-public-native-image.sh").read_text(encoding="utf-8")
        validation = 'case "$capture_mode" in\n  bounded-dom|snapshot-only|bounded-dom-details|snapshot-only-details) ;;\n  *) exit 1 ;;\nesac'
        self.assertIn(validation, wrapper)
        self.assertLess(wrapper.index(validation), wrapper.index("version=$(jq"))
        self.assertLess(wrapper.index(validation), wrapper.index("docker load"))
        self.assertIn('capture_mode="${4:-bounded-dom}"', wrapper)
        self.assertIn('test "$#" = 3 || test "$#" = 4', wrapper)

    def test_snapshot_mode_does_not_add_webjs_and_bounded_mode_keeps_its_budget(self):
        wrapper = (ROOT / "scripts/probe-public-native-image.sh").read_text(encoding="utf-8")
        self.assertIn('probe_args=(--expected-revision "$revision")', wrapper)
        self.assertIn('if [[ "$capture_mode" = bounded-dom ]]; then probe_args+=(--wait-dom); fi', wrapper)
        self.assertIn('if [[ "$capture_mode" = bounded-dom-details ]]; then probe_args+=(--wait-dom --snapshot-details); fi', wrapper)
        self.assertIn('if [[ "$capture_mode" = snapshot-only-details ]]; then probe_args+=(--snapshot-only-details); fi', wrapper)
        self.assertIn('"${probe_args[@]}"', wrapper)
        _, baseline = PROBE.book_info_request(False)["url"].split(", ", 1)
        _, bounded = PROBE.book_info_request(True)["url"].split(", ", 1)
        self.assertEqual({"webView": True}, json.loads(baseline))
        self.assertEqual(PROBE.METADATA_DOM_SCRIPT, json.loads(bounded)["webJs"])

    def test_workflow_passes_the_fixed_mode_without_shell_interpolation(self):
        workflow = (ROOT / ".github/workflows/browser-image.yml").read_text(encoding="utf-8")
        self.assertIn("options: [bounded-dom, snapshot-only, bounded-dom-details, snapshot-only-details]", workflow)
        self.assertIn("default: bounded-dom", workflow)
        self.assertEqual(2, workflow.count("CAPTURE_MODE: ${{ inputs.public_metadata_capture }}"))
        self.assertIn('[[ "$CAPTURE_MODE" = bounded-dom || "$CAPTURE_MODE" = snapshot-only || "$CAPTURE_MODE" = bounded-dom-details || "$CAPTURE_MODE" = snapshot-only-details ]]', workflow)
        self.assertIn('"$RUNNER_TEMP/reader-public-metadata-report" "$CAPTURE_MODE"', workflow)
        self.assertIn("${{ inputs.public_metadata_capture || 'bounded-dom' }}", workflow)
        self.assertNotIn('run: ${{ inputs.public_metadata_capture', workflow)


if __name__ == "__main__":
    unittest.main()
