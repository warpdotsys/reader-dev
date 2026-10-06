"""No-network safety checks, not a substitute for a real default-browser run."""

import importlib.util
import io
import json
from pathlib import Path
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


class RequestBoundaryTest(unittest.TestCase):
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
        logged_out = (200, {"isSuccess": False, "data": "NEED_LOGIN", "errorMsg": "请重新登录"})
        PROBE.verify_cookie_session_logged_out(logged_out, logged_out)
        still_logged_in = (200, {"isSuccess": True, "data": []})
        with self.assertRaises(PROBE.ProbeFailure):
            PROBE.verify_cookie_session_logged_out(logged_out, still_logged_in)
        with self.assertRaises(PROBE.ProbeFailure):
            PROBE.verify_cookie_session_logged_out(still_logged_in, logged_out)

    def test_source_is_fixed_metadata_only_without_credentials_or_chapter_rules(self):
        source = PROBE.source_definition()
        self.assertEqual("https://www.qidian.com", source["bookSourceUrl"])
        self.assertFalse(source["enabledCookieJar"])
        for key in ("ruleContent", "ruleToc", "header", "loginUrl", "loginInfo", "searchUrl"):
            self.assertNotIn(key, source)

    def test_only_exact_loopback_endpoint_and_revision_are_accepted(self):
        for base, revision in (("https://read.medwarp.cn", "a" * 40),
                               (PROBE.BASE + "/", "a" * 40),
                               (PROBE.BASE, "main")):
            with self.assertRaisesRegex(PROBE.ProbeFailure, "InvalidProbeInput"):
                PROBE.require_environment(base, revision, Path("/verification-output"))

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
    def run_generated_probe(self, metadata, fail_logout=False, transport_failure=False):
        observed = threading.Event()
        paths = []
        logged_out = False

        def processes():
            observed.set()
            return {"worker": [1], "browser": [2], "driver": [3]}

        def request(opener, path, body=None):
            nonlocal logged_out
            paths.append(path)
            if path == "/getBookInfo":
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
                return 200, {"isSuccess": False, "data": "NEED_LOGIN"}
            return 200, {"isSuccess": True, "data": {}}

        cgroup = SimpleNamespace(collect_report=lambda *args: {"scope": "generated unit fixture"},
                                 verify_report=lambda *args, **kwargs: None)
        helpers = [cgroup, SimpleNamespace(browser_processes=processes)]
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(PROBE, "require_environment"), \
                    patch.object(PROBE, "load_helper", side_effect=helpers), \
                    patch.object(PROBE, "request_json", side_effect=request), \
                    patch.object(PROBE.os, "getuid", return_value=10001, create=True), \
                    patch("sys.argv", ["probe", "--expected-revision", "a" * 40, "--output", directory]), \
                    patch("sys.stdout", new_callable=io.StringIO) as stdout:
                result = PROBE.main()
            report = json.loads((Path(directory) / "PUBLIC_METADATA_REPORT.json").read_text(encoding="utf-8"))
            encoded = stdout.getvalue() + json.dumps(report, ensure_ascii=False)
            self.assertNotIn("PRIVATE", encoded)
            self.assertEqual(1, report["bookInfoApiCalls"])
            self.assertEqual(1, paths.count("/getBookInfo"))
            self.assertFalse(report["chapterBodyRequested"])
            self.assertFalse(report["realAuthenticationProven"])
            self.assertEqual(0, report["cookieRowsBefore"])
            self.assertEqual(0, report["cookieRowsAfter"])
            self.assertIn("/logout", paths)
            return result, report

    def test_success_shell_without_metadata_is_a_red_gate_after_cleanup(self):
        result, report = self.run_generated_probe({"isSuccess": True, "errorMsg": "", "data": {}})
        self.assertEqual(1, result)
        self.assertFalse(report["passed"])
        self.assertTrue(report["generatedCookieSessionRevokedVerified"])

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


if __name__ == "__main__":
    unittest.main()
