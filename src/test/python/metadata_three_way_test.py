"""Generated report, CLI and local fixture checks; never starts Java or a browser."""
import copy
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
import urllib.request
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/compare-webview-cookie.py"
spec = importlib.util.spec_from_file_location("metadata_three_way_compare", SCRIPT)
PROBE = importlib.util.module_from_spec(spec)
spec.loader.exec_module(PROBE)
METADATA = PROBE.metadata_helper()


def observations(cookies):
    base = "http://127.0.0.1:9"
    book = {key: 0 for key in ("type", "group", "totalChapterNum", "durChapterIndex", "durChapterPos")}
    book.update(name="黎明之剑", author="远瞳", bookUrl=METADATA.book_info_request(base)["url"],
        tocUrl="", canUpdate=True, coverUrl=base + "/generated-cover.svg",
        intro=json.dumps({key: key not in {"bodyHasSafetyPhrase", "captchaContainerPresent"}
            for key in METADATA.DEFINITION.PAGE_DIAGNOSTIC_KEYS}))
    raw = {"isSuccess": True, "errorMsg": "", "data": book}
    searches = [{"status": 200, "isSuccess": True, "errorMsg": "", "count": 1,
        "data": [{"name": "WebView差分书"}],
        "returnData": {"isSuccess": True, "errorMsg": "", "data": [{"name": "WebView差分书"}]}}
        for _ in range(5)]
    return {"searches": searches, "renderCookieHeaders": cookies,
        "renderRequestFields": PROBE.expected_request_fields(), "metadata": {
            "status": 200, "returnData": raw, "fixtureBase": base, "bookInfoApiCalls": 1,
            "sourceDefinitionRoundtrip": METADATA.source_roundtrip(METADATA.source_definition(base), base),
            "targetRequests": [copy.deepcopy(METADATA.TARGET_FIELDS)],
            "sourceScriptSha256": METADATA.SCRIPT_HASH, "diagnosticRuleSha256": METADATA.RULE_HASH}}


def clock_observations(cookies, started):
    value = observations(cookies)
    value["metadata"]["requestWindowMs"] = {"started": started, "completed": started + 100}
    value["metadata"]["returnData"]["data"].update({key: started + 10 for key in METADATA.CLOCK_FIELDS})
    return value


class MetadataThreeWayTest(unittest.TestCase):
    def setUp(self):
        self.original = observations([""] * 5)
        self.remote = copy.deepcopy(self.original)
        self.camoufox = observations(["", "session=alpha==", "", "", ""])

    def test_exact_generated_metadata_and_old_five_cases_can_pass(self):
        PROBE.validate_three_way(self.original, self.remote, self.camoufox, exercise_metadata=True)

    def test_metadata_requires_actual_three_way_and_guarded_handoff_before_any_input(self):
        with tempfile.TemporaryDirectory() as directory:
            for mode in (["--restored-only"], ["--original-network-isolated"],
                         ["--original-network-isolated", "--exercise-script", "--exercise-post",
                          "--archived-renderer-base", "http://127.0.0.1:8050", "--camoufox-python", sys.executable]):
                result = subprocess.run([sys.executable, "-B", str(SCRIPT), *mode,
                    "--exercise-metadata", "--report", str(Path(directory) / "report.json")],
                    capture_output=True, text=True, timeout=10)
                self.assertNotEqual(0, result.returncode)
                self.assertIn("Metadata probe requires actual isolated three-way", result.stderr)
                self.assertEqual([], list(Path(directory).iterdir()))

    def test_direct_launch_cannot_fall_back_to_a_fake_renderer(self):
        with mock.patch.object(PROBE.subprocess, "Popen") as process:
            with self.assertRaisesRegex(SystemExit, "Metadata probe requires actual isolated renderer"):
                PROBE.run_jar(None, None, None, 1, None, None, exercise_metadata=True)
        process.assert_not_called()

    def test_old_five_case_report_cannot_claim_metadata_acceptance(self):
        for side in (self.original, self.remote, self.camoufox):
            side.pop("metadata")
        with self.assertRaisesRegex(RuntimeError, "Missing actual generated metadata"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox, exercise_metadata=True)

    def test_source_identity_changes_only_generated_host_and_name_not_rules(self):
        source = METADATA.source_definition("http://127.0.0.1:9")
        source["bookSourceUrl"] = METADATA.DEFINITION.SOURCE
        source["bookSourceName"] = METADATA.DEFINITION.source_definition()["bookSourceName"]
        self.assertEqual(METADATA.DEFINITION.source_definition(), source)
        request = METADATA.book_info_request("http://127.0.0.1:9")
        options = json.loads(request["url"].split(", ", 1)[1])
        self.assertEqual({"webView": True, "webJs": METADATA.DEFINITION.METADATA_DOM_SCRIPT}, options)
        for base in ("https://www.qidian.com", "http://127.0.0.1:9/", "http://localhost:9", "http://127.0.0.1:65536"):
            with self.assertRaises(ValueError):
                METADATA.source_definition(base)

    def test_matching_missing_or_corrupt_defaults_on_all_sides_are_rejected(self):
        for key in METADATA.DEFINITION.FIELDS:
            value = copy.deepcopy(self.original["metadata"])
            value["returnData"]["data"].pop(key)
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                METADATA.validate(value)
        for key, bad in (("type", False), ("group", True), ("durChapterPos", 1),
                         ("canUpdate", 1), ("canUpdate", False), ("tocUrl", None)):
            value = copy.deepcopy(self.original["metadata"])
            value["returnData"]["data"][key] = bad
            with self.subTest(key=key, bad=bad), self.assertRaises(RuntimeError):
                METADATA.validate(value)

    def test_full_actual_return_data_extra_field_and_type_differences_are_not_normalized(self):
        for side in ("remote", "camoufox"):
            for bad in (None, False, 0, ""):
                original, remote, camoufox = map(copy.deepcopy, (self.original, self.remote, self.camoufox))
                original["metadata"]["returnData"]["data"]["extraDefault"] = None
                remote["metadata"]["returnData"]["data"]["extraDefault"] = None
                camoufox["metadata"]["returnData"]["data"]["extraDefault"] = None
                chosen = remote if side == "remote" else camoufox
                if bad is None:
                    chosen["metadata"]["returnData"]["data"].pop("extraDefault")
                else:
                    chosen["metadata"]["returnData"]["data"]["extraDefault"] = bad
                with self.assertRaisesRegex(RuntimeError, "complete metadata JSON differs"):
                    PROBE.validate_three_way(original, remote, camoufox, exercise_metadata=True)

    def test_url_options_are_preserved_in_the_actual_book_not_stripped_for_comparison(self):
        value = copy.deepcopy(self.original["metadata"])
        value["returnData"]["data"]["bookUrl"] = value["fixtureBase"] + "/book-info"
        with self.assertRaises(RuntimeError):
            METADATA.validate(value)

    def test_source_diagnostic_status_calls_target_and_cookie_are_strict(self):
        for key, bad in (("status", True), ("status", 503), ("bookInfoApiCalls", True),
                         ("bookInfoApiCalls", 2), ("sourceScriptSha256", "a" * 64),
                         ("diagnosticRuleSha256", "a" * 64), ("targetRequests", []),
                         ("targetRequests", [{**METADATA.TARGET_FIELDS, "cookie": "session=alpha=="}])):
            value = copy.deepcopy(self.original["metadata"])
            value[key] = bad
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                METADATA.validate(value)
        for success, error in ((1, ""), (True, None), (True, "generated failure")):
            value = copy.deepcopy(self.original["metadata"])
            value["returnData"].update(isSuccess=success, errorMsg=error)
            with self.assertRaises(RuntimeError):
                METADATA.validate(value)

    def test_changed_source_or_missing_page_diagnostics_cannot_hide_behind_correct_names(self):
        for raw in (None, "PRIVATE_HTML", "{}", json.dumps({key: 1 for key in METADATA.DEFINITION.PAGE_DIAGNOSTIC_KEYS})):
            value = copy.deepcopy(self.original["metadata"])
            value["returnData"]["data"]["intro"] = raw
            with self.assertRaises(RuntimeError):
                METADATA.validate(value)
        for key in METADATA.ROUNDTRIP_KEYS:
            value = copy.deepcopy(self.original["metadata"])
            value["sourceDefinitionRoundtrip"][key] = 1
            with self.assertRaises(RuntimeError):
                METADATA.validate(value)

    def test_metadata_never_replaces_old_post_or_cookie_assertions(self):
        self.camoufox["renderRequestFields"][-1]["body"] = "changed"
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox, exercise_metadata=True)
        self.camoufox = observations([""] * 5)
        with self.assertRaisesRegex(RuntimeError, "Cookie replay"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox, exercise_metadata=True)

    def test_generated_http_fixture_has_delayed_dom_no_cookie_and_separate_observations(self):
        fixture = PROBE.Fixture(("127.0.0.1", 0), archived_renderer=True, exercise_metadata=True)
        worker = threading.Thread(target=fixture.serve_forever, daemon=True)
        worker.start()
        try:
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
            base = "http://127.0.0.1:" + str(fixture.server_port)
            with opener.open(base + "/book-info", timeout=3) as response:
                self.assertEqual(200, response.status)
                self.assertIsNone(response.headers.get("Set-Cookie"))
                self.assertEqual(METADATA.generated_html(), response.read())
            self.assertEqual([METADATA.TARGET_FIELDS], fixture.metadata_requests)
            self.assertEqual([], fixture.snapshot())
            with opener.open(base + "/generated-cover.svg", timeout=3) as response:
                self.assertEqual(200, response.status)
            self.assertEqual(1, len(fixture.metadata_requests))
            fixture.reset()
            self.assertEqual([], fixture.metadata_requests)
        finally:
            fixture.shutdown()
            fixture.server_close()
            worker.join(3)
        self.assertFalse(worker.is_alive())

    def test_failed_metadata_pair_never_creates_handoff_markers(self):
        self.remote["metadata"]["returnData"]["data"].pop("group")
        with tempfile.TemporaryDirectory() as directory, mock.patch.object(PROBE, "require_verified_private_loopback"):
            phases = Path(directory) / "phases"
            with self.assertRaises(RuntimeError):
                PROBE.wait_for_camoufox_handoff(phases, self.original, self.remote, exercise_metadata=True)
            self.assertFalse(phases.exists())

    def test_metadata_mode_rejects_encoding_combinations_before_host_or_files(self):
        with tempfile.TemporaryDirectory() as directory:
            arguments = ["--original", "absent", "--restored", "absent", "--runtime-image", "sha256:" + "a" * 64,
                "--output", directory + "/output", "--exercise-metadata", "--exercise-encoding"]
            result = subprocess.run([sys.executable, str(ROOT / "scripts/run-three-way-webview-in-docker.py"), *arguments],
                capture_output=True, text=True, timeout=10)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("does not replace or relax", result.stderr)
            self.assertFalse((Path(directory) / "output").exists())

    def test_cli_forwards_metadata_to_all_sides_and_preserves_raw_pair_before_handoff(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "three-way.json"
            phases = Path(directory) / "phases"
            arguments = [str(SCRIPT), "--java", sys.executable, "--original", SCRIPT, "--restored", SCRIPT,
                "--report", report, "--original-network-isolated", "--archived-renderer-base", "http://127.0.0.1:8050",
                "--exercise-script", "--exercise-post", "--camoufox-python", sys.executable,
                "--exercise-metadata", "--phase-handoff-dir", phases]
            with mock.patch.object(sys, "argv", list(map(str, arguments))), \
                    mock.patch.dict(sys.modules, {"original_jar_safety": mock.Mock()}), \
                    mock.patch.object(PROBE, "require_verified_private_loopback"), \
                    mock.patch.object(PROBE.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)), \
                    mock.patch.object(PROBE, "Fixture"), mock.patch.object(PROBE, "free_port", return_value=9), \
                    mock.patch.object(PROBE.threading, "Thread"), \
                    mock.patch.object(PROBE, "wait_for_camoufox_handoff") as handoff, \
                    mock.patch.object(PROBE, "run_jar", side_effect=[self.original, self.remote, self.camoufox]) as run, \
                    contextlib.redirect_stdout(io.StringIO()):
                PROBE.main()
            self.assertEqual(3, run.call_count)
            for call in run.call_args_list:
                self.assertIs(call.kwargs["exercise_metadata"], True)
                self.assertIs(call.kwargs["exercise_encoding"], False)
            handoff.assert_called_once_with(phases, self.original, self.remote, exercise_encoding=False, exercise_metadata=True)
            saved = json.loads(report.read_text())
            self.assertEqual(self.original["metadata"], saved["original"]["metadata"])
            observed = json.loads(report.with_name("three-way.historical-observation.json").read_text())
            self.assertEqual(self.remote["metadata"], observed["restored"]["metadata"])
            self.assertIs(observed["acceptanceEvaluatedAtCapture"], False)
            self.assertNotIn("encodingProbe", saved)

    def test_failed_historical_metadata_is_preserved_and_camoufox_never_runs(self):
        self.remote["metadata"]["returnData"].update(isSuccess=False, errorMsg="generated failure", data=None)
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "three-way.json"
            phases = Path(directory) / "phases"
            arguments = [str(SCRIPT), "--java", sys.executable, "--original", SCRIPT, "--restored", SCRIPT,
                "--report", report, "--original-network-isolated", "--archived-renderer-base", "http://127.0.0.1:8050",
                "--exercise-script", "--exercise-post", "--camoufox-python", sys.executable,
                "--exercise-metadata", "--phase-handoff-dir", phases]
            with mock.patch.object(sys, "argv", list(map(str, arguments))), \
                    mock.patch.dict(sys.modules, {"original_jar_safety": mock.Mock()}), \
                    mock.patch.object(PROBE, "require_verified_private_loopback"), \
                    mock.patch.object(PROBE.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)), \
                    mock.patch.object(PROBE, "Fixture"), mock.patch.object(PROBE, "free_port", return_value=9), \
                    mock.patch.object(PROBE.threading, "Thread"), \
                    mock.patch.object(PROBE, "run_jar", side_effect=[self.original, self.remote]) as run, \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(RuntimeError, "ReturnData"):
                    PROBE.main()
            self.assertEqual(2, run.call_count)
            observed = json.loads(report.with_name("three-way.historical-observation.json").read_text())
            self.assertEqual(self.remote["metadata"]["returnData"], observed["restored"]["metadata"]["returnData"])
            self.assertFalse(phases.exists())
            self.assertFalse(report.exists())


class MetadataDefaultClockContractTest(unittest.TestCase):
    def setUp(self):
        self.original = clock_observations([""] * 5, 100000)
        self.remote = clock_observations([""] * 5, 200000)
        self.camoufox = clock_observations(["", "session=alpha==", "", "", ""], 300000)

    def test_explicit_contract_keeps_raw_differences_and_requires_fresh_values(self):
        before = copy.deepcopy((self.original, self.remote, self.camoufox))
        PROBE.validate_three_way(self.original, self.remote, self.camoufox,
            exercise_metadata=True, metadata_clock_contract=True)
        result = METADATA.compare(self.original["metadata"], self.remote["metadata"], True)
        self.assertIs(result["fullRawReturnDataEqual"], False)
        self.assertIs(result["nonClockFullJsonEqual"], True)
        self.assertIs(result["clockFieldsFreshInEachOwnRequestWindow"], True)
        self.assertEqual(before, (self.original, self.remote, self.camoufox))

    def test_original_literal_metadata_validator_still_rejects_different_default_times(self):
        for side in (self.original, self.remote, self.camoufox):
            side["metadata"].pop("requestWindowMs")
        with self.assertRaisesRegex(RuntimeError, "complete metadata JSON differs"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox, exercise_metadata=True)

    def test_identical_stale_missing_future_boolean_and_float_clocks_cannot_pass(self):
        for key in METADATA.CLOCK_FIELDS:
            for bad in (None, 0, 99999, 100101, True, 100010.0):
                value = copy.deepcopy(self.original["metadata"])
                value["returnData"]["data"][key] = bad
                with self.subTest(key=key, bad=bad), self.assertRaisesRegex(RuntimeError, "outside its own actual request window"):
                    METADATA.compare(value, copy.deepcopy(value), True)

    def test_unknown_extra_default_field_is_not_treated_as_a_clock(self):
        left = copy.deepcopy(self.original["metadata"])
        right = copy.deepcopy(self.remote["metadata"])
        left["returnData"]["data"]["otherTime"] = 100010
        right["returnData"]["data"]["otherTime"] = 200010
        with self.assertRaisesRegex(RuntimeError, "non-clock complete metadata JSON differs"):
            METADATA.compare(left, right, True)

    def test_actual_window_cannot_be_absent_coerced_reversed_or_unbounded(self):
        for bad in (None, {}, {"started": True, "completed": 100010},
                    {"started": 100000, "completed": 99999}, {"started": 100000, "completed": 135001},
                    {"started": 100000, "completed": 100100, "PRIVATE": 1}):
            value = copy.deepcopy(self.original["metadata"])
            value["requestWindowMs"] = bad
            with self.assertRaisesRegex(RuntimeError, "actual metadata request clock window"):
                METADATA.validate(value, True)

    def test_clock_contract_does_not_relax_earlier_post_or_cookie_rules(self):
        self.camoufox["renderRequestFields"][-1]["body"] = "changed"
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox,
                exercise_metadata=True, metadata_clock_contract=True)

    def test_cli_clock_contract_requires_metadata_before_reading_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            for script, arguments in ((SCRIPT, ["--restored-only", "--report", directory + "/report.json"]),
                                     (ROOT / "scripts/run-three-way-webview-in-docker.py", ["--original", "absent",
                                      "--restored", "absent", "--runtime-image", "sha256:" + "a" * 64,
                                      "--output", directory + "/output"])):
                result = subprocess.run([sys.executable, str(script), *arguments, "--metadata-clock-contract"],
                    capture_output=True, text=True, timeout=10)
                self.assertNotEqual(0, result.returncode)
                self.assertIn("requires --exercise-metadata", result.stderr)
                self.assertEqual([], list(Path(directory).iterdir()))

    def test_existing_metadata_failure_report_is_rejected_without_overwrite(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "three-way.json"
            old = report.with_name("three-way.original-failed.json")
            old.write_bytes(b"preserve old evidence")
            result = subprocess.run([sys.executable, str(SCRIPT), "--original-network-isolated",
                "--archived-renderer-base", "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                "--camoufox-python", sys.executable, "--exercise-metadata", "--phase-handoff-dir",
                str(Path(directory) / "phases"), "--report", str(report)], capture_output=True, text=True, timeout=10)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Metadata failure report already exists", result.stderr)
            self.assertEqual(b"preserve old evidence", old.read_bytes())
            self.assertFalse(report.exists())


if __name__ == "__main__":
    unittest.main()
