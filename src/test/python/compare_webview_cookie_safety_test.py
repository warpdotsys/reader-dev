"""CLI safety checks; never starts the archived JAR or a network listener."""

import subprocess
import copy
import importlib.util
import json
import sys
import tempfile
import unittest
import os
import hashlib
import io
import threading
import contextlib
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "compare-webview-cookie.py"
SPEC = importlib.util.spec_from_file_location("webview_cookie_probe", SCRIPT)
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)


def invoke(*arguments):
    return subprocess.run(
        [sys.executable, "-B", str(SCRIPT), *map(str, arguments)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )


class WebviewCookieCliSafetyTest(unittest.TestCase):
    def test_existing_historical_observation_is_preserved_before_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "new-report.json"
            observation = Path(directory) / "new-report.historical-observation.json"
            observation.write_bytes(b"preserve completed observations")
            result = invoke("--original-network-isolated", "--archived-renderer-base",
                            "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                            "--camoufox-python", sys.executable, "--exercise-encoding", "--report", report)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Historical observation already exists", result.stderr)
            self.assertEqual(b"preserve completed observations", observation.read_bytes())
            self.assertFalse(report.exists())

    def test_existing_encoding_failure_evidence_is_preserved_before_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "new-report.json"
            failure = Path(directory) / "new-report.original-failed.json"
            failure.write_bytes(b"preserve prior evidence")
            result = invoke("--original-network-isolated", "--archived-renderer-base",
                            "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                            "--camoufox-python", sys.executable, "--exercise-encoding", "--report", report)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Encoding failure report already exists", result.stderr)
            self.assertEqual(b"preserve prior evidence", failure.read_bytes())
            self.assertFalse(report.exists())

    def test_encoding_requires_actual_three_way_mode_before_inputs_or_listener(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "new-report.json"
            for mode in (["--restored-only"], ["--original-network-isolated"],
                         ["--original-network-isolated", "--archived-renderer-base",
                          "http://127.0.0.1:8050", "--exercise-script", "--exercise-post"]):
                with self.subTest(mode=mode):
                    result = invoke(*mode, "--exercise-encoding", "--report", report)
                    self.assertNotEqual(0, result.returncode)
                    self.assertIn("Encoding probe requires actual isolated three-way", result.stderr)
                    self.assertFalse(report.exists())

    def test_direct_encoding_launch_cannot_fall_back_to_a_fake_renderer(self):
        with mock.patch.object(PROBE.subprocess, "Popen") as process:
            with self.assertRaisesRegex(SystemExit, "actual isolated renderer"):
                PROBE.run_jar(None, None, None, 1, None, None, exercise_encoding=True)
            process.assert_not_called()

    def test_phase_handoff_cannot_be_used_without_three_way_mode(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            result = invoke("--restored-only", "--report", directory / "report.json",
                            "--phase-handoff-dir", directory / "phases")
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Phase handoff requires", result.stderr)
            self.assertFalse((directory / "phases").exists())

    def test_three_way_requires_every_explicit_mode_before_inputs(self):
        required = ["--original-network-isolated", "--archived-renderer-base",
                    "http://127.0.0.1:8050", "--exercise-script", "--exercise-post"]
        variants = [["--restored-only"],
                    [arg for arg in required if arg != "--exercise-script"],
                    [arg for arg in required if arg != "--exercise-post"],
                    [arg for arg in required if arg not in
                     ("--archived-renderer-base", "http://127.0.0.1:8050")]]
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "new-report.json"
            for mode in variants:
                with self.subTest(mode=mode):
                    result = invoke(*mode, "--camoufox-python", sys.executable, "--report", report)
                    self.assertNotEqual(0, result.returncode)
                    self.assertIn("Three-way Camoufox mode requires", result.stderr)
                    self.assertFalse(report.exists())

    def test_direct_camoufox_launch_rechecks_namespace_before_any_process(self):
        with mock.patch.dict(os.environ, {}, clear=True), \
                mock.patch.object(PROBE.subprocess, "Popen") as process:
            with self.assertRaisesRegex(SystemExit, "root-verified private"):
                PROBE.run_jar(None, None, None, 1, None, None, camoufox_python=Path(sys.executable))
            process.assert_not_called()

    def test_failed_python_preflight_never_starts_a_jar_or_listener(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "new-report.json"
            arguments = [str(SCRIPT), "--java", sys.executable, "--original", SCRIPT,
                         "--restored", SCRIPT, "--report", report,
                         "--original-network-isolated", "--archived-renderer-base",
                         "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                         "--camoufox-python", sys.executable]
            for failure in (OSError("cannot execute"), subprocess.TimeoutExpired("generated", 15), None):
                with self.subTest(failure=type(failure).__name__), \
                        mock.patch.object(sys, "argv", list(map(str, arguments))), \
                        mock.patch.dict(sys.modules, {"original_jar_safety": mock.Mock()}), \
                        mock.patch.object(PROBE, "require_verified_private_loopback"), \
                        mock.patch.object(PROBE.subprocess, "run", side_effect=failure,
                                          return_value=subprocess.CompletedProcess([], 1)), \
                        mock.patch.object(PROBE, "Fixture") as listener, \
                        mock.patch.object(PROBE.subprocess, "Popen") as process:
                    with self.assertRaises(SystemExit):
                        PROBE.main()
                    listener.assert_not_called()
                    process.assert_not_called()
                    self.assertFalse(report.exists())

    def test_refuses_to_start_without_an_explicit_network_mode(self):
        with tempfile.TemporaryDirectory(prefix="reader-webview-cli-") as directory:
            report = Path(directory) / "new-report.json"
            result = invoke("--report", report)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("--restored-only", result.stderr)
            self.assertIn("--original-network-isolated", result.stderr)
            self.assertFalse(report.exists())


def executed_results(cookie_headers):
    books = [{"name": "WebView差分书", "author": "测试作者", "bookUrl": "http://127.0.0.1:9/book",
              "type": 0, "enabled": False, "coverUrl": None, "intro": ""}]
    searches = [{"status": 200, "isSuccess": True, "errorMsg": "", "count": 1,
                 "data": books, "returnData": {"isSuccess": True, "errorMsg": "", "data": books}}
                for _ in range(5)]
    return copy.deepcopy({
        "searches": searches, "renderCookieHeaders": cookie_headers,
        "renderRequestFields": [{"httpMethod": "GET", "body": None, "testHeader": None}] * 4 +
                               [{"httpMethod": "POST", "body": "q=post", "testHeader": "synthetic"}],
    })


def encoding_results(cookie_headers):
    result = executed_results(cookie_headers[:5])
    search = copy.deepcopy(result["searches"][0])
    search["data"][0]["name"] = PROBE.ENCODING_BOOK
    result["searches"].append(search)
    result["renderCookieHeaders"] = list(cookie_headers)
    result["renderRequestFields"].append(PROBE.encoding_request_fields())
    return result


class EncodingReportValidationTest(unittest.TestCase):
    """Generated fixture validation, not actual Reader/browser observations."""

    def setUp(self):
        self.original = encoding_results([""] * 6)
        self.remote = encoding_results([""] * 6)
        self.camoufox = encoding_results(["", "session=alpha==", "", "", "", ""])

    def validate(self):
        PROBE.validate_three_way(self.original, self.remote, self.camoufox, exercise_encoding=True)

    def test_six_actual_shapes_are_accepted_without_normalization(self):
        before = json.dumps([self.original, self.remote, self.camoufox])
        self.validate()
        self.assertEqual(before, json.dumps([self.original, self.remote, self.camoufox]))

    def test_five_case_report_cannot_claim_six_case_encoding_acceptance(self):
        with self.assertRaisesRegex(RuntimeError, "Missing executed"):
            PROBE.validate_three_way(executed_results([""] * 5), self.remote, self.camoufox, True)
        with self.assertRaisesRegex(RuntimeError, "Missing executed"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox)

    def test_body_bytes_digest_count_text_and_type_are_independently_required(self):
        for key, value in (("bodySha256", "0" * 64), ("bodyByteCount", True),
                           ("bodyByteCount", len(PROBE.ENCODING_BODY)),
                           ("body", PROBE.ENCODING_BODY.replace("𠮷", "?")),
                           ("testHeader", "synthetic")):
            with self.subTest(key=key, value=value):
                changed = copy.deepcopy(self.camoufox)
                changed["renderRequestFields"][-1][key] = value
                with self.assertRaisesRegex(RuntimeError, "GET/POST"):
                    PROBE.validate_three_way(self.original, self.remote, changed, True)
        changed = copy.deepcopy(self.camoufox)
        del changed["renderRequestFields"][-1]["bodySha256"]
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            PROBE.validate_three_way(self.original, self.remote, changed, True)

    def test_matching_corruption_on_all_sides_is_not_an_encoding_pass(self):
        for result in (self.original, self.remote, self.camoufox):
            result["searches"][-1]["data"][0]["name"] = "WebView编码原始书"
        with self.assertRaisesRegex(RuntimeError, "UTF-8 response or script"):
            self.validate()

    def test_sixth_case_does_not_replace_old_post_or_cookie_assertions(self):
        self.camoufox["renderRequestFields"][4]["body"] = "q=unexpected"
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            self.validate()
        self.setUp()
        self.camoufox["renderCookieHeaders"][5] = "session=alpha=="
        with self.assertRaisesRegex(RuntimeError, "replay/deletion"):
            self.validate()

    def handler(self, raw):
        handler = object.__new__(PROBE.FixtureHandler)
        handler.server = mock.Mock(archived_renderer=True, exercise_encoding=True,
                                   calls=[], script_sources=[], request_fields=[], lock=threading.Lock())
        handler.path = "/search-utf8"
        handler.headers = {"Content-Length": str(len(raw)), "X-Fixture": "synthetic-utf8"}
        handler.rfile = io.BytesIO(raw)
        handler.send_search_html = mock.Mock()
        handler.send_error = mock.Mock()
        return handler

    def test_target_handler_hashes_original_bytes_not_reencoded_summary(self):
        raw = PROBE.ENCODING_BODY.encode("utf-8")
        handler = self.handler(raw)
        handler.do_POST()
        self.assertEqual([{"httpMethod": "POST", "body": PROBE.ENCODING_BODY,
                           "testHeader": "synthetic-utf8", "bodyByteCount": len(raw),
                           "bodySha256": hashlib.sha256(raw).hexdigest()}], handler.server.request_fields)
        self.assertGreater(len(raw), len(PROBE.ENCODING_BODY))
        handler.send_error.assert_not_called()

    def test_target_handler_rejects_invalid_utf8_without_replacement(self):
        handler = self.handler(b'{"query":"\xff"}')
        handler.do_POST()
        handler.send_error.assert_called_once_with(400)
        self.assertEqual([], handler.server.request_fields)
        handler.send_search_html.assert_not_called()

    def test_actual_mode_forwards_encoding_to_all_three_sides_and_handoff(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "three-way.json"
            phases = Path(temporary) / "phases"
            arguments = [str(SCRIPT), "--java", sys.executable, "--original", SCRIPT,
                         "--restored", SCRIPT, "--report", report,
                         "--original-network-isolated", "--archived-renderer-base",
                         "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                         "--camoufox-python", sys.executable, "--exercise-encoding",
                         "--phase-handoff-dir", phases]
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
            for call, side in zip(run.call_args_list, ("original", "restored", "camoufox")):
                self.assertIs(True, call.kwargs["exercise_encoding"])
                self.assertIs(True, call.kwargs["include_data"])
                self.assertEqual(report.with_name("three-way." + side + "-failed.json"), call.kwargs["failure_report"])
            handoff.assert_called_once_with(phases, self.original, self.remote, exercise_encoding=True)
            value = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(PROBE.encoding_request_fields(), value["encodingProbe"]["expectedRequestFields"])
            self.assertEqual(6, len(value["expectedCamoufoxCookieSequence"]))
            observation = json.loads(report.with_name("three-way.historical-observation.json").read_text(encoding="utf-8"))
            self.assertEqual(self.original, observation["original"])
            self.assertEqual(self.remote, observation["restored"])
            self.assertIs(False, observation["acceptanceEvaluatedAtCapture"])
            self.assertIs(False, observation["camoufoxExecutedAtCapture"])

    def test_failed_historical_handoff_preserves_raw_observation_without_running_camoufox(self):
        self.original["renderRequestFields"][-1]["bodyByteCount"] = 44
        self.remote["renderRequestFields"][-1]["bodyByteCount"] = 44
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "three-way.json"
            phases = Path(temporary) / "phases"
            arguments = [str(SCRIPT), "--java", sys.executable, "--original", SCRIPT,
                         "--restored", SCRIPT, "--report", report,
                         "--original-network-isolated", "--archived-renderer-base",
                         "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                         "--camoufox-python", sys.executable, "--exercise-encoding",
                         "--phase-handoff-dir", phases]
            with mock.patch.object(sys, "argv", list(map(str, arguments))), \
                    mock.patch.dict(sys.modules, {"original_jar_safety": mock.Mock()}), \
                    mock.patch.object(PROBE, "require_verified_private_loopback"), \
                    mock.patch.object(PROBE.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)), \
                    mock.patch.object(PROBE, "Fixture"), mock.patch.object(PROBE, "free_port", return_value=9), \
                    mock.patch.object(PROBE.threading, "Thread"), \
                    mock.patch.object(PROBE, "run_jar", side_effect=[self.original, self.remote]) as run, \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(RuntimeError, "GET/POST"):
                    PROBE.main()
            self.assertEqual(2, run.call_count)
            observation = json.loads(report.with_name("three-way.historical-observation.json").read_text(encoding="utf-8"))
            self.assertEqual(self.original, observation["original"])
            self.assertEqual(self.remote, observation["restored"])
            self.assertIs(False, observation["acceptanceEvaluatedAtCapture"])
            self.assertIs(False, observation["camoufoxExecutedAtCapture"])
            self.assertFalse(report.exists())
            self.assertFalse(phases.exists())


class HistoricalUtf8CharacterizationTest(unittest.TestCase):
    """Generated report fixtures only; no Reader or browser execution claims."""

    def setUp(self):
        self.original = encoding_results([""] * 6)
        self.remote = encoding_results([""] * 6)
        self.camoufox = encoding_results(["", "session=alpha==", "", "", "", ""])
        for side in (self.original, self.remote):
            side["renderRequestFields"][-1] = PROBE.historical_utf8_truncation_fields()

    def test_exact_observed_defect_has_distinct_false_parity_and_does_not_rewrite_data(self):
        before = json.dumps([self.original, self.remote, self.camoufox])
        observation = PROBE.validate_utf8_characterization(self.original, self.remote, self.camoufox)
        self.assertIs(True, observation["observationsComplete"])
        self.assertIs(False, observation["strictSixCaseParityAccepted"])
        self.assertIs(False, observation["historicalUtf8BodyCorrect"])
        self.assertIs(True, observation["camoufoxUtf8BodyCorrect"])
        self.assertEqual(44, observation["historicalExpectedDefectFields"]["bodyByteCount"])
        self.assertEqual("c61ca3e561f770d4fc8551da0bddb98afd0be801801502504a73221ed11ec5d9",
                         observation["historicalExpectedDefectFields"]["bodySha256"])
        with self.assertRaises(json.JSONDecodeError):
            json.loads(observation["historicalExpectedDefectFields"]["body"])
        self.assertEqual(60, observation["correctUtf8Fields"]["bodyByteCount"])
        self.assertEqual(before, json.dumps([self.original, self.remote, self.camoufox]))

    def test_default_strict_validator_still_rejects_the_observed_defect(self):
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            PROBE.validate_three_way(self.original, self.remote, self.camoufox, True)

    def test_fixed_or_unknown_historical_fields_cannot_be_called_the_known_defect(self):
        for fields in (PROBE.encoding_request_fields(), {},
                       {**PROBE.historical_utf8_truncation_fields(), "bodyByteCount": True},
                       {**PROBE.historical_utf8_truncation_fields(), "bodyByteCount": 43},
                       {**PROBE.historical_utf8_truncation_fields(), "bodySha256": "0" * 64},
                       {**PROBE.historical_utf8_truncation_fields(), "body": ""},
                       {**PROBE.historical_utf8_truncation_fields(), "testHeader": "other"}):
            with self.subTest(fields=fields):
                side = copy.deepcopy(self.remote)
                side["renderRequestFields"][-1] = fields
                with self.assertRaisesRegex(RuntimeError, "GET/POST"):
                    PROBE.validate_utf8_characterization(self.original, side, self.camoufox)

    def test_first_five_requests_and_all_response_fields_remain_required(self):
        for key, value in (("body", "other"), ("httpMethod", "GET")):
            with self.subTest(key=key):
                side = copy.deepcopy(self.remote)
                side["renderRequestFields"][4][key] = value
                with self.assertRaisesRegex(RuntimeError, "GET/POST"):
                    PROBE.validate_utf8_characterization(self.original, side, self.camoufox)
        for side in (self.remote, self.camoufox):
            for index in range(6):
                with self.subTest(side="camoufox" if side is self.camoufox else "remote", index=index):
                    changed = copy.deepcopy(side)
                    changed["searches"][index]["returnData"]["extra"] = None
                    pair = (self.original, changed, self.camoufox) if side is self.remote else \
                           (self.original, self.remote, changed)
                    with self.assertRaisesRegex(RuntimeError, "full Reader JSON differs"):
                        PROBE.validate_utf8_characterization(*pair)

    def test_current_engine_must_send_all_sixty_bytes_not_follow_the_defect(self):
        self.camoufox["renderRequestFields"][-1] = PROBE.historical_utf8_truncation_fields()
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            PROBE.validate_utf8_characterization(self.original, self.remote, self.camoufox)

    def test_cookie_sequences_and_six_real_response_shapes_are_not_relaxed(self):
        for original, remote, camoufox in (
                (executed_results([""] * 5), self.remote, self.camoufox),
                (self.original, self.remote, executed_results([""] * 5))):
            with self.assertRaisesRegex(RuntimeError, "Missing executed"):
                PROBE.validate_utf8_characterization(original, remote, camoufox)
        self.remote["renderCookieHeaders"][5] = "session=unexpected"
        with self.assertRaisesRegex(RuntimeError, "historical renderer"):
            PROBE.validate_utf8_characterization(self.original, self.remote, self.camoufox)
        self.setUp()
        self.camoufox["renderCookieHeaders"][2] = "session=alpha=="
        with self.assertRaisesRegex(RuntimeError, "replay/deletion"):
            PROBE.validate_utf8_characterization(self.original, self.remote, self.camoufox)

    def test_characterization_handoff_still_checks_namespace_and_port_refused(self):
        for connection_error in (ConnectionRefusedError, None):
            with self.subTest(connection_error=connection_error), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary) / "phases"
                with mock.patch.object(PROBE, "require_verified_private_loopback") as guard, \
                        mock.patch.object(PROBE.time, "sleep", side_effect=lambda _: \
                                          (directory / "camoufox-permitted").touch()), \
                        mock.patch.object(PROBE.socket, "create_connection", side_effect=connection_error,
                                          return_value=mock.MagicMock()):
                    if connection_error is None:
                        with self.assertRaisesRegex(RuntimeError, "still running"):
                            PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote,
                                exercise_encoding=True, characterize_historical_utf8=True)
                    else:
                        PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote,
                            exercise_encoding=True, characterize_historical_utf8=True)
                    self.assertEqual(2, guard.call_count)

    def test_characterization_cannot_create_markers_without_six_cases(self):
        with tempfile.TemporaryDirectory() as temporary, \
                mock.patch.object(PROBE, "require_verified_private_loopback"):
            directory = Path(temporary) / "phases"
            with self.assertRaisesRegex(RuntimeError, "all six probes"):
                PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote,
                                               characterize_historical_utf8=True)
            self.assertFalse(directory.exists())

    def test_cli_requires_explicit_six_case_mode_and_guarded_handoff_before_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "report.json"
            for extra in ([], ["--exercise-encoding"], ["--phase-handoff-dir", Path(temporary) / "phases"]):
                with self.subTest(extra=extra):
                    result = invoke("--restored-only", "--report", report,
                                    "--characterize-historical-utf8", *extra)
                    self.assertNotEqual(0, result.returncode)
                    self.assertIn("requires six-case mode and guarded renderer handoff", result.stderr)
                    self.assertFalse(report.exists())

    def test_completed_cli_preserves_all_raw_sides_but_cannot_exit_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "report.json"
            phases = Path(temporary) / "phases"
            arguments = [str(SCRIPT), "--java", sys.executable, "--original", SCRIPT,
                         "--restored", SCRIPT, "--report", report,
                         "--original-network-isolated", "--archived-renderer-base",
                         "http://127.0.0.1:8050", "--exercise-script", "--exercise-post",
                         "--camoufox-python", sys.executable, "--exercise-encoding",
                         "--phase-handoff-dir", phases, "--characterize-historical-utf8"]
            with mock.patch.object(sys, "argv", list(map(str, arguments))), \
                    mock.patch.dict(sys.modules, {"original_jar_safety": mock.Mock()}), \
                    mock.patch.object(PROBE, "require_verified_private_loopback"), \
                    mock.patch.object(PROBE.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)), \
                    mock.patch.object(PROBE, "Fixture"), mock.patch.object(PROBE, "free_port", return_value=9), \
                    mock.patch.object(PROBE.threading, "Thread"), \
                    mock.patch.object(PROBE, "wait_for_camoufox_handoff") as handoff, \
                    mock.patch.object(PROBE, "run_jar", side_effect=[self.original, self.remote, self.camoufox]) as run, \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(RuntimeError, "strict six-case parity NOT accepted"):
                    PROBE.main()
            self.assertEqual(3, run.call_count)
            handoff.assert_called_once_with(phases, self.original, self.remote,
                                           exercise_encoding=True, characterize_historical_utf8=True)
            value = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(self.original, value["original"])
            self.assertEqual(self.remote, value["restored"])
            self.assertEqual(self.camoufox, value["camoufox"])
            self.assertIs(False, value["characterization"]["strictSixCaseParityAccepted"])
            self.assertIs(False, value["characterization"]["historicalUtf8BodyCorrect"])


class HistoricalRendererHandoffTest(unittest.TestCase):
    """Generated rendezvous only; no real Reader or browser is launched."""

    def setUp(self):
        self.original = executed_results([""] * 5)
        self.remote = executed_results([""] * 5)

    def test_handoff_requires_private_namespace_before_creating_markers(self):
        with tempfile.TemporaryDirectory() as temporary, \
                mock.patch.dict(os.environ, {}, clear=True):
            directory = Path(temporary) / "phases"
            with self.assertRaisesRegex(SystemExit, "root-verified private"):
                PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote)
            self.assertFalse(directory.exists())

    def test_invalid_actual_remote_pair_never_signals_completion(self):
        self.remote["searches"][0]["returnData"]["unexpected"] = None
        with tempfile.TemporaryDirectory() as temporary, \
                mock.patch.object(PROBE, "require_verified_private_loopback"):
            directory = Path(temporary) / "phases"
            with self.assertRaisesRegex(RuntimeError, "full Reader JSON differs"):
                PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote)
            self.assertFalse(directory.exists())

    def test_missing_host_acknowledgement_times_out_without_camoufox(self):
        with tempfile.TemporaryDirectory() as temporary, \
                mock.patch.object(PROBE, "require_verified_private_loopback"), \
                mock.patch.object(PROBE.socket, "create_connection") as connection:
            directory = Path(temporary) / "phases"
            with self.assertRaisesRegex(TimeoutError, "not acknowledged"):
                PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote, timeout=0)
            self.assertEqual(b"", (directory / "remote-complete").read_bytes())
            connection.assert_not_called()

    def test_acknowledgement_rechecks_namespace_and_requires_port_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "phases"
            with mock.patch.object(PROBE, "require_verified_private_loopback") as guard, \
                    mock.patch.object(PROBE.time, "sleep", side_effect=lambda _:
                                      (directory / "camoufox-permitted").touch()), \
                    mock.patch.object(PROBE.socket, "create_connection",
                                      side_effect=ConnectionRefusedError) as connection:
                PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote)
                self.assertEqual(2, guard.call_count)
                connection.assert_called_once_with(("127.0.0.1", 8050), timeout=2)

    def test_live_historical_port_cannot_be_misreported_as_stopped(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "phases"
            with mock.patch.object(PROBE, "require_verified_private_loopback"), \
                    mock.patch.object(PROBE.time, "sleep", side_effect=lambda _:
                                      (directory / "camoufox-permitted").touch()), \
                    mock.patch.object(PROBE.socket, "create_connection", return_value=mock.MagicMock()):
                with self.assertRaisesRegex(RuntimeError, "still running"):
                    PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote)

    def test_nonempty_acknowledgement_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "phases"
            with mock.patch.object(PROBE, "require_verified_private_loopback"), \
                    mock.patch.object(PROBE.time, "sleep", side_effect=lambda _:
                                      (directory / "camoufox-permitted").write_text("unexpected")), \
                    mock.patch.object(PROBE.socket, "create_connection") as connection:
                with self.assertRaisesRegex(RuntimeError, "Invalid historical"):
                    PROBE.wait_for_camoufox_handoff(directory, self.original, self.remote)
                connection.assert_not_called()


class ThreeWayReportValidationTest(unittest.TestCase):
    """Generated report fixtures, not executed Reader/browser observations."""

    def setUp(self):
        self.original = executed_results([""] * 5)
        self.remote = executed_results([""] * 5)
        self.camoufox = executed_results(["", "session=alpha==", "", "", ""])

    def validate(self):
        PROBE.validate_three_way(self.original, self.remote, self.camoufox)

    def test_equal_full_responses_and_documented_cookie_difference_are_accepted(self):
        before = json.dumps([self.original, self.remote, self.camoufox])
        self.validate()
        self.assertEqual(before, json.dumps([self.original, self.remote, self.camoufox]))

    def test_missing_response_or_default_field_is_not_filled_from_summary(self):
        for key in ("data", "returnData"):
            with self.subTest(key=key):
                result = copy.deepcopy(self.camoufox)
                del result["searches"][0][key]
                with self.assertRaises(RuntimeError):
                    PROBE.validate_three_way(self.original, self.remote, result)
        for key in ("isSuccess", "errorMsg", "data"):
            with self.subTest(raw_key=key):
                result = copy.deepcopy(self.camoufox)
                del result["searches"][0]["returnData"][key]
                with self.assertRaises(RuntimeError):
                    PROBE.validate_three_way(self.original, self.remote, result)

    def test_changed_default_value_type_and_extra_response_field_are_rejected(self):
        for key, value in (("type", False), ("enabled", 0), ("coverUrl", ""), ("intro", None)):
            with self.subTest(key=key):
                result = copy.deepcopy(self.camoufox)
                result["searches"][0]["data"][0][key] = value
                with self.assertRaisesRegex(RuntimeError, "full Reader JSON differs"):
                    PROBE.validate_three_way(self.original, self.remote, result)
        self.camoufox["searches"][0]["returnData"]["unexpected"] = None
        with self.assertRaisesRegex(RuntimeError, "full Reader JSON differs"):
            self.validate()

    def test_summary_cannot_disagree_with_actual_data(self):
        self.camoufox["searches"][0]["data"] = [{"name": "not the recorded response"}]
        with self.assertRaisesRegex(RuntimeError, "exact ReturnData"):
            self.validate()

    def test_missing_probe_or_non_object_result_is_rejected(self):
        for result in (None, {}, {"searches": None}, {"searches": [None] * 5},
                       {"searches": self.camoufox["searches"][:4]}):
            with self.subTest(result=result), self.assertRaises(RuntimeError):
                PROBE.validate_three_way(self.original, self.remote, result)

    def test_target_post_fields_and_unreviewed_cookies_are_rejected(self):
        self.camoufox["renderRequestFields"][-1]["body"] = "q=unexpected"
        with self.assertRaisesRegex(RuntimeError, "GET/POST"):
            self.validate()
        self.setUp()
        self.remote["renderCookieHeaders"][1] = "session=alpha=="
        with self.assertRaisesRegex(RuntimeError, "historical renderer"):
            self.validate()
        self.setUp()
        self.camoufox["renderCookieHeaders"][2] = "session=alpha=="
        with self.assertRaisesRegex(RuntimeError, "replay/deletion"):
            self.validate()

    def test_count_boolean_is_not_the_integer_one(self):
        self.camoufox["searches"][0]["count"] = True
        with self.assertRaisesRegex(RuntimeError, "actual Reader JSON"):
            self.validate()

    def test_exclusive_report_creation_preserves_an_existing_file(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            PROBE.write_report(report, {"fixture": "中文"})
            before = report.read_bytes()
            with self.assertRaises(FileExistsError):
                PROBE.write_report(report, {"unexpected": True})
            self.assertEqual(before, report.read_bytes())

    @unittest.skipUnless(os.name == "posix", "POSIX symlink creation")
    def test_dangling_report_symlink_cannot_create_its_target(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "link.json"
            target = Path(directory) / "absent-target.json"
            report.symlink_to(target)
            with self.assertRaises(FileExistsError):
                PROBE.write_report(report, {"unexpected": True})
            self.assertFalse(target.exists())

    def test_refuses_to_overwrite_an_existing_report_before_looking_for_jars(self):
        with tempfile.TemporaryDirectory(prefix="reader-webview-cli-") as directory:
            report = Path(directory) / "user-report.json"
            original = b"keep this existing report unchanged\n"
            report.write_bytes(original)
            result = invoke("--restored-only", "--report", report)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Report already exists", result.stderr)
            self.assertEqual(original, report.read_bytes())

    def test_original_mode_also_requires_isolation_acknowledgment(self):
        with tempfile.TemporaryDirectory(prefix="reader-webview-cli-") as directory:
            report = Path(directory) / "new-report.json"
            environment = os.environ.copy()
            environment.pop("READER_ORIGINAL_JAR_NETWORK_ISOLATED", None)
            result = subprocess.run(
                [sys.executable, "-B", str(SCRIPT),
                 "--original-network-isolated", "--report", str(report)],
                cwd=ROOT, env=environment, capture_output=True, text=True,
                timeout=10, check=False,
            )
            self.assertNotEqual(0, result.returncode)
            self.assertIn("Refusing to start the original JAR", result.stderr)
            self.assertFalse(report.exists())


if __name__ == "__main__":
    unittest.main()
