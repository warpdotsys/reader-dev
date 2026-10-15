"""Generated pure guards only; these never count as actual browser acceptance."""
import copy
import contextlib
import hashlib
import importlib.util
import json
import io
from pathlib import Path
import threading
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
def load(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
m = load("header_security_test_subject", "reader_header_security_differential.py")
c = load("header_security_comparator_subject", "compare-webview-cookie.py")

def row(case, actor, role, method="GET", authorization=True):
    body = m.POST_BODY.encode() if method == "POST" else b""
    pairs = [["User-Agent", m.USER_AGENT], ["X-Generated-Trace", m.TRACE]]
    if authorization:
        pairs.append(["Authorization", m.AUTHORIZATION])
    return {"case":case,"actor":actor,"role":role,"method":method,"headers":pairs,
        "bodyByteCount":len(body),"bodySha256":hashlib.sha256(body).hexdigest()}

def observation(follow_post=False, cross_authorization=True):
    rows = []
    results = []
    for case in m.CASES:
        if case == "resources":
            rows += [row(case,"primary","page"), row(case,"primary","same.js"),
                row(case,"secondary","cross.js",authorization=cross_authorization)]
        else:
            rows.append(row(case,"primary","start","POST" if case == "cross-post" else "GET"))
            if case != "cross-post" or follow_post:
                actor = "primary" if case == "same-get" else "secondary"
                rows.append(row(case,actor,"end",authorization=True if actor == "primary" else cross_authorization))
        results.append({"case":case,"status":200,"sourceDefinitionRoundtrip":True,
            "returnData":{"isSuccess":True,"errorMsg":"","data":[{"name":"认证头差分书-"+case}]}})
    return {"generatedOnly":True,"realCredentialsImported":False,"httpsTested":False,
        "cases":results,"targetRequests":rows,"rejections":0}

class HeaderSecurityDifferentialTest(unittest.TestCase):
    def test_isolation_failure_precedes_bind_or_thread_start(self):
        def guard(): raise SystemExit("no private namespace")
        with mock.patch.object(m,"ThreadingHTTPServer") as server, mock.patch.object(m.threading,"Thread") as thread:
            with self.assertRaises(SystemExit): m.HeaderFixture(guard,c.bounded_target_headers)
        server.assert_not_called()
        thread.assert_not_called()

    def test_definitions_contain_only_fixed_generated_headers_and_checked_resource_script(self):
        fixture = object.__new__(m.HeaderFixture)
        fixture.primary, fixture.secondary = "http://127.0.0.1:1234", "http://127.0.0.2:1235"
        for case in m.CASES:
            definition = fixture.definition(case)
            options = json.loads(definition["searchUrl"].split(", ",1)[1])
            self.assertEqual({"Authorization":m.AUTHORIZATION,"X-Generated-Trace":m.TRACE,"User-Agent":m.USER_AGENT},options["headers"])
            self.assertEqual(case == "cross-post", options.get("method") == "POST")
            self.assertIn("认证头差分书-"+case,options["webJs"])
        script = json.loads(fixture.definition("resources")["searchUrl"].split(", ",1)[1])["webJs"]
        self.assertIn("generatedSameResource !== 'executed'",script)
        self.assertIn("generatedCrossResource !== 'executed'",script)
        self.assertIn("throw new Error",script)
        with self.assertRaises(ValueError): fixture.definition("https://untrusted.test")

    def test_complete_insecure_historical_observation_is_never_security_or_parity_green(self):
        actual = observation()
        original = copy.deepcopy(actual)
        result = m.characterize(actual)
        self.assertTrue(result["observationComplete"])
        self.assertFalse(result["crossOriginAuthorizationAbsentForAllObservedRequests"])
        self.assertFalse(result["strictParityAccepted"])
        self.assertFalse(result["realAuthenticationAccepted"])
        self.assertFalse(result["httpsAccepted"])
        self.assertEqual(actual,original)
        self.assertFalse(result["cases"][2]["redirectFollowed"])

    def test_safe_observed_cross_headers_do_not_grant_unexecuted_auth_or_https_parity(self):
        result = m.characterize(observation(True,False))
        self.assertTrue(result["crossOriginAuthorizationAbsentForAllObservedRequests"])
        self.assertTrue(result["cases"][2]["redirectFollowed"])
        self.assertFalse(result["strictParityAccepted"])
        self.assertFalse(result["realAuthenticationAccepted"])

    def test_missing_navigation_or_script_resource_cannot_count_as_absent_secret(self):
        for index in (0,1,2,3,4,5,6,7):
            actual = observation()
            del actual["targetRequests"][index]
            with self.subTest(index=index), self.assertRaises(RuntimeError): m.characterize(actual)

    def test_wrong_body_method_actor_duplicate_or_unbounded_pairs_rejected(self):
        changes = ({"bodyByteCount":True}, {"bodyByteCount":65537}, {"bodySha256":"bad"},
            {"method":"DELETE"},{"actor":"external"},{"case":"unknown"},
            {"headers":[["X","x"]]*65},{"headers":[["X","x"*8192]]},{"headers":[["X",1]]})
        for change in changes:
            actual = observation()
            actual["targetRequests"][0].update(change)
            with self.subTest(change=list(change)), self.assertRaises(RuntimeError): m.characterize(actual)
        actual = observation()
        actual["targetRequests"].append(copy.deepcopy(actual["targetRequests"][0]))
        with self.assertRaises(RuntimeError): m.characterize(actual)

    def test_http_boolean_status_failed_json_empty_book_and_wrong_script_name_rejected(self):
        for change in ({"status":True},{"status":500},{"sourceDefinitionRoundtrip":False},
                {"returnData":None},{"returnData":{"isSuccess":False,"data":[]}},
                {"returnData":{"isSuccess":True,"data":[{"name":"认证头原始书-same-get"}]}}):
            actual = observation()
            actual["cases"][0].update(change)
            with self.subTest(change=list(change)), self.assertRaises(RuntimeError): m.characterize(actual)

    def test_finite_target_record_rejects_overflow_without_truncating_or_mutating(self):
        fixture = object.__new__(m.HeaderFixture)
        fixture.lock,fixture.rows,fixture.rejections = threading.RLock(),[],0
        fixture.bounded_headers = c.bounded_target_headers
        headers = mock.Mock()
        headers.raw_items.return_value = [("X-Duplicate","a"),("x-duplicate","b")]
        for _ in range(32): fixture.record("same-get","primary","start","GET",headers,b"")
        before = fixture.snapshot()
        with self.assertRaises(ValueError): fixture.record("same-get","primary","start","GET",headers,b"")
        self.assertEqual(before,fixture.snapshot())
        self.assertEqual([["X-Duplicate","a"],["x-duplicate","b"]],before["targetRequests"][0]["headers"])
        before["targetRequests"].clear()
        self.assertEqual(32,len(fixture.snapshot()["targetRequests"]))

    def test_direct_jar_probe_refuses_header_fixture_without_actual_renderer_before_process_start(self):
        with mock.patch.object(c.subprocess,"Popen") as start:
            with self.assertRaises(SystemExit):
                c.run_jar(Path("java"),Path("jar"),Path("work"),1,"http://127.0.0.1",mock.Mock(),header_security_fixture=mock.Mock())
        start.assert_not_called()

    def test_cli_refuses_incomplete_or_mixed_scope_before_any_bind_or_process_start(self):
        for args in (["--restored-only","--characterize-header-security"],
                ["--original-network-isolated","--characterize-header-security","--observe-target-headers"]):
            with mock.patch("sys.argv",["compare","--report","never-created.json",*args]), \
                    mock.patch.object(c,"Fixture") as fixture,mock.patch.object(c.subprocess,"Popen") as start, \
                    contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as raised: c.main()
            self.assertEqual(2,raised.exception.code)
            fixture.assert_not_called()
            start.assert_not_called()

if __name__ == "__main__": unittest.main()
