"""Generated adapter/guard regressions, not an original-JAR or runtime proof."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("guarded_baseline_api", ROOT / "scripts/probe-baseline-api-in-guarded-runtime.py")
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


class Response:
    def __init__(self, raw, status=200, url="http://127.0.0.1:18890/reader3/getBookshelf"):
        self.raw, self.status, self.url = raw, status, url
        self.headers = {"Content-Type": "application/json"}

    def __enter__(self): return self
    def __exit__(self, *args): return None
    def read(self, size): return self.raw[:size]
    def geturl(self): return self.url


class Opener:
    def __init__(self, response): self.response = response
    def open(self, request, timeout):
        self.request = request
        return self.response


class GuardedBaselineApiTest(unittest.TestCase):
    def test_exact_generated_unit_and_sha_are_accepted(self):
        PROBE.validate_inputs("reader-baseline-api-20261010-d.service", "e" * 64)

    def test_disabled_assertions_cannot_bypass_real_cli_guards(self):
        result = subprocess.run([sys.executable, "-O", str(ROOT / "scripts/probe-baseline-api-in-guarded-runtime.py"),
            "--unit", "reader-baseline-api-20261010-d.service", "--expected-restored-sha256", "e" * 64],
            capture_output=True, text=True, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("Refuses disabled safety assertions", result.stderr)

    def test_unsafe_main_inputs_are_rejected_before_identity_or_any_jar(self):
        with mock.patch.object(PROBE, "identity") as identity:
            with self.assertRaises(ValueError):
                PROBE.main(["--unit", "reader.service", "--expected-restored-sha256", "e" * 64])
            identity.assert_not_called()

    def test_unsafe_or_production_units_are_rejected(self):
        for unit in ("reader.service", "reader-baseline-api-20261010-d.service/child", "../reader.service", "", None):
            with self.subTest(unit=unit), self.assertRaises(ValueError):
                PROBE.validate_inputs(unit, "e" * 64)

    def test_malformed_or_original_as_candidate_sha_is_rejected(self):
        for sha in ("e" * 63, "E" * 64, "e" * 64 + "\n", PROBE.ORIGINAL_SHA, None):
            with self.subTest(sha=sha), self.assertRaises(ValueError):
                PROBE.validate_inputs("reader-baseline-api-20261010-d.service", sha)

    def test_redaction_removes_credentials_but_keeps_null_false_zero_defaults(self):
        raw = {"accessToken": "generated-secret", "nested": {"password": "generated-secret"},
               "enabledCookieJar": False, "cookieCount": 0, "token": None, "name": "中文默认值"}
        redacted = PROBE.redact(raw)
        self.assertNotIn("generated-secret", json.dumps(redacted))
        for name in ("enabledCookieJar", "cookieCount", "token", "name"):
            self.assertEqual(redacted[name], raw[name])

    def test_auth_defaults_keep_unknown_fields_and_do_not_mutate_input(self):
        raw = {"data": {"username": "generated", "accessToken": "generated-secret", "createdAt": 1,
                       "lastLoginAt": 2, "nestedUnknown": {"enabled": False, "null": None}}}
        before = json.dumps(raw)
        value = PROBE.authentication_defaults(raw, "generated")
        self.assertEqual(value, {"username": "generated", "nestedUnknown": {"enabled": False, "null": None}})
        self.assertEqual(before, json.dumps(raw))

    def test_missing_or_wrong_auth_transient_types_are_rejected(self):
        for override in ({"accessToken": ""}, {"accessToken": None}, {"createdAt": True},
                         {"lastLoginAt": "2"}, {"username": "other"}):
            document = {"data": {"username": "generated", "accessToken": "generated-secret", "createdAt": 1,
                                 "lastLoginAt": 2, **override}}
            with self.subTest(override=override), self.assertRaises(AssertionError):
                PROBE.authentication_defaults(document, "generated")

    def test_success_requires_actual_envelope_and_http_status(self):
        self.assertTrue(PROBE.successful({"status": 200, "document": {"isSuccess": True}}))
        for value in ({"status": 404, "document": {"isSuccess": True}},
                      {"status": 200, "document": {"isSuccess": 1}}, {"status": 200, "document": []}):
            self.assertFalse(PROBE.successful(value))

    def test_non_json_preserves_status_but_does_not_expose_token_query_or_body(self):
        path = "/reader3/getBookshelf?accessToken=generated-secret"
        response = Response(b"<html>generated-private-error</html>", 404,
                            "http://127.0.0.1:18890" + path)
        with self.assertRaises(PROBE.NonJsonResponse) as caught:
            PROBE.request(Opener(response), "http://127.0.0.1:18890", path)
        observed = caught.exception.observation
        self.assertEqual(observed["path"], "/reader3/getBookshelf")
        self.assertEqual(observed["status"], 404)
        self.assertTrue(observed["looksHtml"])
        self.assertFalse(observed["responseBodyPublished"])
        self.assertNotIn("generated-secret", json.dumps(observed))
        self.assertNotIn("generated-private-error", json.dumps(observed))

    def test_request_keeps_literal_json_defaults(self):
        body = {"isSuccess": False, "errorMsg": "请登录后使用", "data": "NEED_LOGIN"}
        observed = PROBE.request(Opener(Response(json.dumps(body).encode())), "http://127.0.0.1:18890",
                                 "/reader3/getBookshelf")
        self.assertEqual(observed["document"], body)

    def test_login_json_post_matches_existing_wire_charset(self):
        opener = Opener(Response(b'{"isSuccess":true}'))
        PROBE.request(opener, "http://127.0.0.1:18890", "/reader3/login", {"isLogin": False})
        self.assertEqual(opener.request.get_method(), "POST")
        self.assertEqual(opener.request.get_header("Content-type"), "application/json; charset=utf-8")

    def test_case_sensitive_bookshelf_and_post_logout_are_in_real_probe(self):
        source = (ROOT / "scripts/probe-baseline-api-in-guarded-runtime.py").read_text()
        self.assertNotIn('"/reader3/getBookShelf"', source)
        self.assertIn('"/reader3/getBookshelf"', source)
        self.assertIn('"/reader3/logout", {})', source)


if __name__ == "__main__":
    unittest.main()
