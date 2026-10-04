"""CLI safety checks; never starts the archived JAR or a network listener."""

import subprocess
import copy
import importlib.util
import json
import sys
import tempfile
import unittest
import os
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
