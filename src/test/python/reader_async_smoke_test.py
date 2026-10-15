"""Generated guard/wire doubles only; real Reader/Camoufox acceptance runs in CI."""

import copy
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


SMOKE = load("reader_async_smoke", "scripts/smoke-local-webview.py")
GUARD = load("reader_async_guard", "scripts/verify-reader-async-smoke.py")


def report():
    cases = []
    for phase in ("get", "post"):
        cases.append({"case": phase, "status": 200, "isSuccess": True, "errorMsg": "",
                      "count": 1, "nameMatches": True, "authorMatches": True, "bookUrlMatches": True,
                      "targetRequest": {"httpMethod": "GET" if phase == "get" else "POST",
                                        "body": None if phase == "get" else "q=async",
                                        "testHeader": None if phase == "get" else "async"},
                      "targetCookie": "" if phase == "get" else "asyncOnly=generated",
                      "scriptMarkCount": 1, "scriptMarkCookie": "asyncOnly=generated" if phase == "get" else ""})
    return {"asyncReaderCases": cases, "asyncCookieCleanup": {
        "sameUserNextRequestCookie": "", "sameUserNextRequestCount": 1, "deletedVerified": True}}


class ReaderAsyncGuardTest(unittest.TestCase):
    def test_complete_generated_json_checks_only_the_verifier(self):
        verified = GUARD.verify(report())
        self.assertEqual(2, verified["asyncReaderCases"])
        self.assertTrue(verified["singleExecutionAndPostVerified"])

    def test_old_synchronous_report_cannot_prove_async_reader_calls(self):
        with self.assertRaises(GUARD.AsyncSmokeRejected):
            GUARD.verify({"legacyReferenceCases": [{"isSuccess": True}]})

    def test_missing_extra_reversed_or_duplicate_cases_are_rejected(self):
        for cases in ([], report()["asyncReaderCases"][:1], report()["asyncReaderCases"] * 2,
                      list(reversed(report()["asyncReaderCases"])), [report()["asyncReaderCases"][0]] * 2):
            value = report()
            value["asyncReaderCases"] = cases
            with self.assertRaises(GUARD.AsyncSmokeRejected):
                GUARD.verify(value)

    def test_success_status_and_counts_are_not_coerced(self):
        for key, wrong in (("isSuccess", "true"), ("isSuccess", 1), ("status", "200"),
                           ("status", 503), ("count", True), ("count", 0),
                           ("scriptMarkCount", True), ("scriptMarkCount", 2),
                           ("errorMsg", None), ("errorMsg", "generated-failure")):
            value = report()
            value["asyncReaderCases"][0][key] = wrong
            with self.assertRaises(GUARD.AsyncSmokeRejected):
                GUARD.verify(value)

    def test_bad_metadata_or_raw_unexpected_fields_are_rejected(self):
        for key in ("nameMatches", "authorMatches", "bookUrlMatches", "rawHtml"):
            value = report()
            value["asyncReaderCases"][0][key] = "PRIVATE_BODY"
            with self.assertRaises(GUARD.AsyncSmokeRejected):
                GUARD.verify(value)

    def test_cookie_scope_creation_and_deletion_observations_are_required(self):
        for index, key, wrong in ((0, "targetCookie", "asyncOnly=generated"),
                                  (1, "targetCookie", ""), (0, "scriptMarkCookie", ""),
                                  (1, "scriptMarkCookie", "asyncOnly=generated")):
            value = report()
            value["asyncReaderCases"][index][key] = wrong
            with self.assertRaises(GUARD.AsyncSmokeRejected):
                GUARD.verify(value)

    def test_original_target_method_body_and_headers_are_required(self):
        value = report()
        for key, wrong in (("httpMethod", "GET"), ("body", "wrong"), ("testHeader", None)):
            changed = copy.deepcopy(value)
            changed["asyncReaderCases"][1]["targetRequest"][key] = wrong
            with self.assertRaises(GUARD.AsyncSmokeRejected):
                GUARD.verify(changed)

    def test_same_user_followup_cannot_be_replaced_by_another_users_empty_cookie(self):
        for key, wrong in (("sameUserNextRequestCookie", "asyncOnly=generated"),
                           ("sameUserNextRequestCount", True), ("deletedVerified", "true")):
            value = report()
            value["asyncCookieCleanup"][key] = wrong
            with self.assertRaises(GUARD.AsyncSmokeRejected):
                GUARD.verify(value)

    def test_file_reader_caps_bytes_and_rejects_malformed_json(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.json"
            for raw in (b"{", b"PRIVATE_BODY", b"x" * (GUARD.MAX_BYTES + 1)):
                path.write_bytes(raw)
                with self.assertRaises(GUARD.AsyncSmokeRejected):
                    GUARD.verify_file(path)
            path.write_text(json.dumps(report()), encoding="utf-8")
            self.assertEqual(64, len(GUARD.verify_file(path)["jsonSha256"]))


class ReaderAsyncFixtureTest(unittest.TestCase):
    def test_generated_script_uses_a_promise_bounded_dom_wait_and_one_marker(self):
        for phase in ("get", "post"):
            script = SMOKE.async_source_script(phase)
            self.assertIn("new Promise", script)
            self.assertIn("started > 5000", script)
            self.assertEqual(1, script.count("fetch('/async-mark'"))
            self.assertIn("document.documentElement.outerHTML", script)
        with self.assertRaises(ValueError):
            SMOKE.async_source_script("PRIVATE_URL")

    def test_delayed_wire_fixture_does_not_include_the_accepted_book_name(self):
        fixture = SMOKE.Fixture(("127.0.0.1", 0))
        thread = threading.Thread(target=fixture.serve_forever, daemon=True)
        thread.start()
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        base = f"http://127.0.0.1:{fixture.server_port}"
        try:
            with opener.open(base + "/search-async") as response:
                raw = response.read().decode("utf-8")
            self.assertIn("setTimeout", raw)
            self.assertIn("WebView脚本原始书", raw)
            self.assertNotIn("WebView异步书", raw)
            with opener.open(urllib.request.Request(base + "/async-mark", data=b"phase=get",
                                                  headers={"Cookie": "asyncOnly=generated"})) as response:
                self.assertEqual(200, response.status)
            self.assertEqual(1, len(fixture.requests))
            self.assertEqual([{"body": "phase=get", "cookie": "asyncOnly=generated"}], fixture.async_marks)
        finally:
            fixture.shutdown()
            fixture.server_close()
            thread.join(timeout=3)

    def run_double(self, fault=None):
        fixture = SimpleNamespace(requests=[], cookies=[], async_marks=[])
        source = {"bookSourceUrl": "http://127.0.0.1:1", "enabledCookieJar": True}
        options = {}
        def call(_opener, _base, path, body):
            nonlocal options
            if path == "/reader3/saveBookSource":
                if ", " in body.get("searchUrl", ""):
                    options = json.loads(body["searchUrl"].split(", ", 1)[1])
                return {"isSuccess": True, "errorMsg": "", "data": {}}
            phase = body["key"].removeprefix("async-")
            cleanup = phase == "cleanup-followup"
            post = phase == "post"
            target = {"httpMethod": "POST" if post else "GET", "body": "q=async" if post else None,
                      "testHeader": "async" if post else None}
            fixture.requests.append(target)
            fixture.cookies.append("asyncOnly=generated" if post else "")
            if not cleanup:
                self.assertTrue(options["webView"])
                self.assertIn("new Promise", options["webJs"])
                mark = {"body": "phase=" + phase, "cookie": "" if post else "asyncOnly=generated"}
                fixture.async_marks.append(mark)
                if fault == "target-replay":
                    fixture.requests.append(target)
                if fault == "script-replay":
                    fixture.async_marks.append(mark)
                if fault == "wrong-cookie":
                    fixture.cookies[-1] = "wrong"
            elif fault == "failed-deletion":
                fixture.cookies[-1] = "asyncOnly=generated"
            name = "本地浏览器测试书" if cleanup else "WebView异步书"
            data = [{"name": name, "author": "测试作者", "bookUrl": source["bookSourceUrl"] + "/book"}]
            if fault == "empty-result":
                data = []
            if fault == "raw-name" and not cleanup:
                data[0]["name"] = "WebView脚本原始书"
            return {"isSuccess": True, "errorMsg": "", "data": data}
        with patch.object(SMOKE, "call", side_effect=call):
            return SMOKE.exercise_async_reader(object(), "http://127.0.0.1:18890", fixture, source)

    def test_generated_double_exercises_full_positive_observation_shape_not_a_browser(self):
        cases, cleanup = self.run_double()
        GUARD.verify({"asyncReaderCases": cases, "asyncCookieCleanup": cleanup})

    def test_empty_or_unmodified_book_cannot_pass_the_reader_helper(self):
        for fault in ("empty-result", "raw-name"):
            with self.assertRaises(RuntimeError):
                self.run_double(fault)

    def test_target_or_source_replay_and_cookie_mismatch_are_actual_helper_failures(self):
        for fault in ("target-replay", "script-replay", "wrong-cookie", "failed-deletion"):
            with self.assertRaises(RuntimeError):
                self.run_double(fault)

    def test_user_reader_urls_are_rejected_before_account_or_fixture_creation(self):
        for base in ("http://127.0.0.1:18931", "https://read.medwarp.cn", "http://user:secret@127.0.0.1:18890",
                     "http://127.0.0.1:18890/?token=PRIVATE", "http://localhost:18890"):
            with patch("sys.argv", ["smoke", "--reader-base", base]), \
                    patch.object(SMOKE, "Fixture", side_effect=AssertionError("Must not create fixture")), \
                    patch("sys.stderr", new_callable=io.StringIO), self.assertRaises(SystemExit) as caught:
                SMOKE.main()
            self.assertEqual(2, caught.exception.code)


if __name__ == "__main__":
    unittest.main()
