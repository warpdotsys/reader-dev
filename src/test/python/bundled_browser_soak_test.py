"""Generated-only unit checks; these do not claim a real Camoufox soak passed."""

import importlib.util
import json
from pathlib import Path
import tempfile
import threading
import unittest
import urllib.request
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "bundled_soak", Path(__file__).resolve().parents[3] / "scripts/soak-bundled-browser.py")
SOAK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SOAK)


class BundledBrowserSoakTest(unittest.TestCase):
    def test_only_owned_worker_arguments_can_be_killed(self):
        self.assertEqual("worker", SOAK.process_kind(["/usr/bin/python3", "/tmp/reader-camoufox-worker-123.py"]))
        self.assertIsNone(SOAK.process_kind(["/usr/bin/java", "/tmp/reader-camoufox-worker-123.py"]))
        self.assertIsNone(SOAK.process_kind(["/usr/bin/python3", "/srv/private-script.py"]))
        self.assertIsNone(SOAK.process_kind(["/usr/bin/python3", "/tmp/not-a-reader-worker.py"]))
        self.assertIsNone(SOAK.process_kind(["/usr/bin/python3", "/tmp/reader-camoufox-worker-dir/other.py"]))

    def test_browser_and_driver_children_are_not_ignored(self):
        self.assertEqual("browser", SOAK.process_kind(["/home/reader/.cache/camoufox/camoufox", "-contentproc"]))
        self.assertEqual("browser", SOAK.process_kind(["/home/reader/.cache/camoufox/camoufox-bin"]))
        self.assertEqual("browser", SOAK.process_kind(["/home/reader/.cache/camoufox/crashreporter"]))
        self.assertEqual("driver", SOAK.process_kind(["/usr/local/lib/python3.10/dist-packages/playwright/driver/node"]))
        self.assertIsNone(SOAK.process_kind(["/usr/bin/node", "reader-admin.js"]))

    def test_pid_one_and_other_uid_are_never_worker_candidates(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for pid, uid in ((1, 10001), (17, 10001), (18, 0), (19, 10002)):
                directory = root / str(pid)
                directory.mkdir()
                (directory / "status").write_text(f"Uid:\t{uid}\t{uid}\t{uid}\t{uid}\n")
                (directory / "cmdline").write_bytes(b"/usr/bin/python3\0/tmp/reader-camoufox-worker-123.py\0")
            with patch.object(SOAK, "PROC", root):
                self.assertEqual({"worker": [17], "browser": [], "driver": []}, SOAK.browser_processes())

    def test_non_soak_reader_endpoints_and_revision_are_rejected_before_any_request(self):
        for base in ("https://read.medwarp.cn", "http://127.0.0.1:18931",
                     "http://127.0.0.1:18892/?token=private", "http://user:password@127.0.0.1:18892"):
            with self.assertRaises(ValueError):
                SOAK.require_environment(base, "a" * 40, 1800, Path("/verification-output"))
        with self.assertRaises(ValueError):
            SOAK.require_environment("http://127.0.0.1:18892", "../revision", 1800, Path("/verification-output"))

    def test_input_duration_is_bounded(self):
        for seconds in (0, 59, 3601):
            with self.assertRaises(ValueError):
                SOAK.require_environment("http://127.0.0.1:18892", "a" * 40, seconds, Path("/verification-output"))

    def test_fault_is_not_passed_by_http_200_or_an_unrelated_failure(self):
        SOAK.validate_fault({"isSuccess": False, "errorMsg": "Camoufox 渲染失败 (TimeoutError)"}, "navigationTimeout")
        SOAK.validate_fault({"isSuccess": False, "errorMsg": "Camoufox WebView 超时"}, "scriptWatchdog")
        SOAK.validate_fault({"isSuccess": False, "errorMsg": "worker 异常退出 (-9)"}, "workerExit")
        for value in ({"isSuccess": True, "errorMsg": ""},
                      {"isSuccess": False, "errorMsg": "登录失效"},
                      {"isSuccess": False, "errorMsg": None}):
            with self.assertRaises(RuntimeError):
                SOAK.validate_fault(value, "navigationTimeout")

    def test_soak_generated_source_has_real_script_watchdog_and_post_fields(self):
        source = SOAK.source_for("http://127.0.0.1:1234", 2, "POST", hang_script=True)
        options = json.loads(source["searchUrl"].split(", ", 1)[1])
        self.assertTrue(options["webView"])
        self.assertEqual("POST", options["method"])
        self.assertEqual("account=2", options["body"])
        self.assertEqual("2", options["headers"]["X-Soak-Account"])
        self.assertIn("for (;;)", options["webJs"])

    def test_fixture_checks_cross_account_cookie_and_post_shape(self):
        fixture = SOAK.Fixture()
        thread = threading.Thread(target=fixture.serve_forever, daemon=True)
        thread.start()
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        url = f"http://127.0.0.1:{fixture.server_port}/healthy?account=0"
        try:
            with opener.open(urllib.request.Request(url, headers={"X-Soak-Account": "0"})) as response:
                self.assertIn(b"SoakUser0", response.read())
            with opener.open(urllib.request.Request(url, data=b"account=0", headers={
                    "X-Soak-Account": "0", "Cookie": "soak_session=" + fixture.markers[0]})) as response:
                response.read()
            self.assertEqual([], fixture.errors)
            with opener.open(urllib.request.Request(url, headers={"X-Soak-Account": "0",
                    "Cookie": "soak_session=" + fixture.markers[1]})) as response:
                response.read()
            self.assertEqual(["cookieIsolationOrReplayMismatch"], fixture.errors)
            self.assertEqual({"GET": 2, "POST": 1}, fixture.methods)
        finally:
            fixture.release_stalls.set()
            fixture.shutdown()
            fixture.server_close()
            thread.join(timeout=5)

    def test_finished_request_with_surviving_children_fails(self):
        with patch.object(SOAK, "browser_processes", return_value={"worker": [], "browser": [20], "driver": []}), \
                patch.object(SOAK.time, "monotonic", side_effect=(0, 11)):
            with self.assertRaisesRegex(RuntimeError, "processes remained"):
                SOAK.quiet_snapshot()


if __name__ == "__main__":
    unittest.main()
