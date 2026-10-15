"""Real pinned Camoufox checks of finite document labels, with generated data only.

This is a helper-level browser gate, not an end-to-end Reader, authentication,
real-source or original-JAR parity claim. Only one owned loopback fixture is
allowed; no HTML, errors, URLs, Cookie or source result values enter the report.
"""

import argparse
import contextlib
import importlib.util
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import sys
import threading
import time
from urllib.parse import urlsplit


CASES = {
    "deletedStateSameDocument": ("sameDocument", False),
    "deletedStateSameDocumentAfterHashNavigation": ("sameDocument", True),
    "replacedDocumentReferenceUnavailable": ("unavailable", True),
}
CASE_FIELDS = {"case", "documentObservation", "mainFrameNavigationObserved", "sourceRuns", "passed"}


class GateFailure(Exception):
    def __init__(self, report):
        self.report = report


def verify_report(report):
    if (not isinstance(report, dict) or
            set(report) != {"schemaVersion", "scope", "cases", "passed", "networkDenials"} or
            type(report["schemaVersion"]) is not int or report["schemaVersion"] != 1 or
            report["scope"] != "generated helper-level pinned browser; not Reader parity" or
            type(report["networkDenials"]) is not int or report["networkDenials"] != 0 or
            report["passed"] is not True or not isinstance(report["cases"], list) or
            len(report["cases"]) != len(CASES)):
        raise ValueError("Invalid bounded document-observation report")
    seen = set()
    for case in report["cases"]:
        if not isinstance(case, dict) or set(case) != CASE_FIELDS:
            raise ValueError("Invalid document-observation case")
        name = case["case"]
        if type(name) is not str or name not in CASES or name in seen:
            raise ValueError("Missing or duplicate document-observation case")
        seen.add(name)
        observation, navigation = CASES[name]
        if (case["documentObservation"] != observation or
                type(case["mainFrameNavigationObserved"]) is not bool or
                case["mainFrameNavigationObserved"] is not navigation or
                type(case["sourceRuns"]) is not int or case["sourceRuns"] != 1 or
                case["passed"] is not True):
            raise ValueError("Document-observation contract failed")


def load_worker():
    path = Path(__file__).resolve().parents[1] / "src/main/resources/camoufox/worker.py"
    spec = importlib.util.spec_from_file_location("reader_document_observation_gate", path)
    module = importlib.util.module_from_spec(spec)
    # worker deliberately redirects its stdout at import. Preserve this gate's
    # machine-protocol stream, and never serialize the worker's returned holder.
    original_stdout = sys.stdout
    try:
        spec.loader.exec_module(module)
    finally:
        sys.stdout = original_stdout
    return module


def run_cases(worker, browser_version):
    class Fixture(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path not in ("/origin", "/replacement"):
                self.send_error(404)
                return
            body = b"<!doctype html><title>generated document fixture</title><p>generated</p>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()
    port = server.server_address[1]
    base = "http://127.0.0.1:" + str(port)
    report = {"schemaVersion": 1, "scope": "generated helper-level pinned browser; not Reader parity",
              "cases": [], "passed": False, "networkDenials": 0}
    try:
        with contextlib.redirect_stdout(sys.stderr), worker.Camoufox(
                headless=True, browser=browser_version, geoip=False, block_webrtc=True,
                exclude_addons=[worker.DefaultAddons.UBO], i_know_what_im_doing=True,
                firefox_user_prefs={"network.dns.disablePrefetch": True,
                    "network.prefetch-next": False, "network.predictor.enabled": False,
                    "network.http.speculative-parallel-limit": 0}) as browser:
            for name, (expected_document, expected_navigation) in CASES.items():
                context = worker.NewContext(browser, accept_downloads=False, service_workers="block")
                holder = None
                try:
                    def guard_route(route):
                        url = urlsplit(route.request.url)
                        if (url.scheme == "http" and url.hostname == "127.0.0.1" and
                                url.port == port and url.path in ("/origin", "/replacement")):
                            route.continue_()
                        else:
                            report["networkDenials"] += 1
                            route.abort()
                    context.route("**/*", guard_route)
                    context.set_default_timeout(2000)
                    context.set_default_navigation_timeout(2000)
                    page = context.new_page()
                    page.goto(base + "/origin", wait_until="load")
                    runs = [0]
                    def observe_console(message):
                        if message.text == "generated-source-once":
                            runs[0] += 1
                    page.on("console", observe_console)
                    if name == "replacedDocumentReferenceUnavailable":
                        # Deliberate fixture navigation, outside evaluate_source_script.
                        # This checks a real foreign/disposed handle, not state-loss recovery.
                        holder = page.evaluate_handle(worker.SOURCE_SCRIPT_START,
                            {"source": "console.log('generated-source-once'); new Promise(()=>{})",
                             "key": "__reader_source_generated_probe"})
                        navigations = [False]
                        page.on("framenavigated", lambda frame: navigations.__setitem__(0, True)
                                if frame == page.main_frame else None)
                        page.goto(base + "/replacement", wait_until="load")
                        document = worker.source_document_observation(page, holder,
                            time.monotonic() + 1, lambda: None, time.monotonic)
                        navigation = navigations[0]
                    else:
                        hash_navigation = "location.hash = 'generated';" if expected_navigation else ""
                        source = "console.log('generated-source-once'); " + hash_navigation + "new Promise(() => { " + \
                            "setTimeout(() => Object.getOwnPropertyNames(globalThis)" + \
                            ".filter(k => k.startsWith('__reader_source_'))" + \
                            ".forEach(k => { delete globalThis[k]; }), 150); })"
                        try:
                            worker.evaluate_source_script(page, source, 2000, lambda: None)
                        except worker.SourceScriptStateLost as error:
                            diagnostic = worker.browser_failure_diagnostic(error)
                            document = diagnostic["sourceDocumentObservation"]
                            navigation = diagnostic["mainFrameNavigationObserved"]
                        else:
                            raise AssertionError("Generated state deletion must stay fatal")
                        assert context.cookies() == []
                        assert worker.evaluate_source_script(page, "'generated-health'", 1000,
                                                             lambda: None) == "generated-health"
                    report["cases"].append({"case": name, "documentObservation": document,
                        "mainFrameNavigationObserved": navigation, "sourceRuns": runs[0],
                        "passed": document == expected_document and navigation is expected_navigation and runs[0] == 1})
                finally:
                    if holder is not None:
                        try:
                            holder.dispose()
                        except Exception:
                            pass
                    context.close()
        report["passed"] = True
        verify_report(report)
        return report
    except Exception:
        report["passed"] = False
        raise GateFailure(report) from None
    finally:
        server.shutdown()
        server.server_close()
        server_thread.join(timeout=2)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--browser-version", required=True)
    args = parser.parse_args()
    try:
        report = run_cases(load_worker(), args.browser_version)
    except GateFailure as error:
        print(json.dumps(error.report, sort_keys=True))
        raise SystemExit(1)
    except Exception:
        # A fatal result must not expose page/driver messages or pretend cases ran.
        report = {"schemaVersion": 1, "scope": "generated helper-level pinned browser; not Reader parity",
                  "cases": [], "passed": False, "networkDenials": 0}
        print(json.dumps(report, sort_keys=True))
        raise SystemExit(1)
    print(json.dumps(report, sort_keys=True))


if __name__ == "__main__":
    main()
