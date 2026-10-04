"""WebView Cookie differential using disposable accounts and a loopback fixture.

The archived JAR uses Vert.x's port-only listen call; passing bindAddress does
not make it loopback-only. Run that JAR only inside separately verified network
isolation. The safe mode launches only the restored JAR.
"""

import argparse
import hashlib
import http.cookiejar
import json
import os
import secrets
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest().upper()


class Fixture(ThreadingHTTPServer):
    def __init__(self, address, archived_renderer=False):
        super().__init__(address, FixtureHandler)
        self.archived_renderer = archived_renderer
        self.calls = []
        self.script_sources = []
        self.request_fields = []
        self.lock = threading.Lock()

    def reset(self):
        with self.lock:
            self.calls.clear()
            self.script_sources.clear()
            self.request_fields.clear()

    def snapshot(self):
        with self.lock:
            return list(self.calls)

    def script_snapshot(self):
        with self.lock:
            return list(self.script_sources)

    def request_snapshot(self):
        with self.lock:
            return list(self.request_fields)


class FixtureHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if not self.server.archived_renderer or not self.path.startswith("/search"):
            self.send_error(404)
            return
        self.serve_search("GET", None)

    def do_POST(self):
        if self.server.archived_renderer and self.path.startswith("/search"):
            length = int(self.headers.get("Content-Length", "0"))
            if length > 65536:
                self.send_error(400)
                return
            self.serve_search("POST", self.rfile.read(length).decode("utf-8"))
            return
        if self.path != "/render.html":
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length", "0"))
        if length <= 0 or length > 65536:
            self.send_error(400)
            return
        try:
            request = json.loads(self.rfile.read(length))
            target = request["url"]
            headers = request["headers"] or {}
            if not target.startswith(f"http://127.0.0.1:{self.server.server_port}/search"):
                raise ValueError("unexpected target")
            cookie = next((value for key, value in headers.items()
                           if key.lower() == "cookie"), "")
            fixture_header = next((value for key, value in headers.items()
                                   if key.lower() == "x-fixture"), None)
            with self.server.lock:
                self.server.calls.append(cookie)
                self.server.script_sources.append(request.get("js_source"))
                self.server.request_fields.append({
                    "httpMethod": request.get("http_method"),
                    "body": request.get("body"),
                    "testHeader": fixture_header,
                })
                call_number = len(self.server.calls)
        except (ValueError, KeyError, TypeError):
            self.send_error(400)
            return
        self.send_search_html(call_number)

    def serve_search(self, method, body):
        with self.server.lock:
            self.server.calls.append(self.headers.get("Cookie", ""))
            self.server.script_sources.append(None)
            self.server.request_fields.append({
                "httpMethod": method,
                "body": body,
                "testHeader": self.headers.get("X-Fixture"),
            })
            call_number = len(self.server.calls)
        self.send_search_html(call_number)

    def send_search_html(self, call_number):
        if self.server.archived_renderer:
            # The later probes require webJs to rewrite this marker. Parsing
            # the unmodified target HTML cannot accidentally count as success.
            name = "WebView脚本原始书" if call_number >= 4 else "WebView差分书"
            html = ("<html><head><title>WebView差分页</title></head>"
                    "<body><div class='book'><a href='/book'>"
                    f"<span class='name'>{name}</span></a>"
                    "<span class='author'>测试作者</span></div></body></html>")
        else:
            html = ("<html><div class='book'><a href='/book'>"
                    "<span class='name'>WebView差分书</span></a>"
                    "<span class='author'>测试作者</span></div></html>")
        data = html.encode("utf-8")
        self.send_response(200)
        if call_number == 1:
            self.send_header("Set-Cookie", "session=alpha==; Path=/; HttpOnly; SameSite=Lax")
        elif call_number == 2:
            self.send_header("Set-Cookie", "session=; Max-Age=0; Path=/")
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, _format, *_args):
        pass


def request(opener, base, path, body=None):
    data = json.dumps(body, ensure_ascii=False).encode("utf-8") if body is not None else None
    headers = {"Content-Type": "application/json; charset=utf-8"} if data else {}
    req = urllib.request.Request(base + path, data=data, headers=headers,
                                 method="POST" if data else "GET")
    with opener.open(req, timeout=25) as response:
        return response.status, json.loads(response.read().decode("utf-8"))


def require_success(opener, base, path, body=None, include_return_data=False):
    status, value = request(opener, base, path, body)
    if status != 200 or value.get("isSuccess") is not True:
        raise RuntimeError(f"{path}: HTTP {status}, {value.get('errorMsg')}")
    return {"status": status, "isSuccess": True,
            "errorMsg": value.get("errorMsg", ""), "data": value.get("data"),
            **({"returnData": value} if include_return_data else {})}


def require_verified_private_loopback():
    """Recheck the root guard's namespace after dropping privileges."""
    try:
        verified = (os.name == "posix" and
                    str(os.stat("/proc/self/ns/net").st_ino) ==
                    os.environ.get("READER_PRIVATE_NETNS_INODE") and
                    [name for _, name in socket.if_nameindex()] == ["lo"])
    except OSError:
        verified = False
    if not verified:
        raise SystemExit("Archived/Camoufox probe requires a root-verified private loopback-only namespace")


def run_jar(java, jar, workdir, port, fixture_base, fixture,
            exercise_script=False, exercise_post=False, renderer_base=None,
            camoufox_python=None, include_data=False):
    if renderer_base is not None or camoufox_python is not None:
        require_verified_private_loopback()
    base = f"http://127.0.0.1:{port}"
    workdir.mkdir(parents=True, exist_ok=True)
    launch = [str(java), "-Xms128m", "-Xmx768m", "-jar", str(jar),
              f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
              "--reader.server.bindAddress=127.0.0.1",
              "--reader.app.webviewRenderer=" + ("camoufox" if camoufox_python else "remote"),
              f"--reader.app.remote-webview-api={renderer_base or fixture_base}",
              "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
              "--spring.profiles.active=prod"]
    environment = os.environ.copy()
    environment.pop("READER_BROWSER_ALLOW_PRIVATE_NETWORKS", None)
    if camoufox_python is not None:
        launch.append(f"--reader.app.camoufoxPythonExecutable={camoufox_python}")
        # Only enabled inside the independently verified loopback-only netns.
        # The generated target must be reachable; no external interface exists.
        environment["READER_BROWSER_ALLOW_PRIVATE_NETWORKS"] = "true"
    flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
    log_path = workdir / "reader.log"
    log_output = log_path.open("wb")
    try:
        process = subprocess.Popen(launch, cwd=workdir, stdout=log_output,
                                   stderr=subprocess.STDOUT, creationflags=flags,
                                   env=environment)
    except BaseException:
        log_output.close()
        raise
    opener = urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited at startup: {process.returncode}")
            try:
                if require_success(opener, base, "/reader3/getSystemInfo"):
                    break
            except (OSError, ValueError, urllib.error.URLError):
                time.sleep(0.5)
        else:
            raise TimeoutError("Reader startup timed out")

        username = "webviewprobe" + secrets.token_hex(5)
        password = "Probe-" + secrets.token_hex(12)
        for is_login in (False, True):
            require_success(opener, base, "/reader3/login",
                            {"username": username, "password": password,
                             "isLogin": is_login})
        source = {
            "bookSourceUrl": fixture_base,
            "bookSourceName": "Loopback WebView Cookie fixture",
            "enabledCookieJar": True,
            "searchUrl": fixture_base + "/search, {\"webView\": true}",
            "ruleSearch": {"bookList": ".book", "name": ".name@text",
                           "author": ".author@text", "bookUrl": "a@href"},
            "ruleToc": {"chapterList": ".chapter"},
            "ruleContent": {"content": ".content@html"},
        }
        require_success(opener, base, "/reader3/saveBookSource", source)
        saved = require_success(opener, base, "/reader3/getBookSource",
                                {"bookSourceUrl": fixture_base})["data"]
        if saved.get("searchUrl") != source["searchUrl"]:
            raise RuntimeError(f"Saved source changed WebView rule: keys={list(saved.keys())}, "
                               f"value={saved.get('searchUrl')!r}")
        probes = []
        for key in ("first", "second", "third"):
            result = require_success(opener, base, "/reader3/searchBook",
                                     {"key": key, "page": 1,
                                      "bookSourceUrl": fixture_base}, include_return_data=include_data)
            books = result["data"]
            if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "WebView差分书":
                names = [book.get("name") for book in books] if isinstance(books, list) else None
                raise RuntimeError(f"Unexpected search result for {key}: "
                                   f"names={names}, renderCalls={len(fixture.snapshot())}")
            probes.append({"status": result["status"], "isSuccess": True,
                           "errorMsg": result["errorMsg"], "count": len(books),
                           **({"data": books, "returnData": result["returnData"]} if include_data else {})})
        if exercise_script:
            scripted_source = dict(source)
            script = ("document.documentElement.outerHTML.replace('WebView脚本原始书','WebView差分书')" if renderer_base
                      else "document.title")
            scripted_source["searchUrl"] = (
                fixture_base + '/search, {"webView": true, "webJs": "' + script + '"}'
            )
            require_success(opener, base, "/reader3/saveBookSource", scripted_source)
            result = require_success(opener, base, "/reader3/searchBook", {
                "key": "script", "page": 1, "bookSourceUrl": fixture_base},
                include_return_data=include_data)
            books = result["data"]
            if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "WebView差分书":
                raise RuntimeError("Script-bearing synthetic source did not parse one book")
            probes.append({"status": result["status"], "isSuccess": True,
                           "errorMsg": result["errorMsg"], "count": len(books),
                           **({"data": books, "returnData": result["returnData"]} if include_data else {})})
        if exercise_post:
            post_source = dict(source)
            script = ("document.documentElement.outerHTML.replace('WebView脚本原始书','WebView差分书')" if renderer_base
                      else "document.title")
            post_source["searchUrl"] = (
                fixture_base + '/search, {"webView": true, "method": "POST", '
                '"body": "q=post", "headers": {"X-Fixture": "synthetic"}, '
                '"webJs": "' + script + '"}'
            )
            require_success(opener, base, "/reader3/saveBookSource", post_source)
            result = require_success(opener, base, "/reader3/searchBook", {
                "key": "post", "page": 1, "bookSourceUrl": fixture_base},
                include_return_data=include_data)
            books = result["data"]
            if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "WebView差分书":
                raise RuntimeError("POST-bearing synthetic source did not parse one book")
            probes.append({"status": result["status"], "isSuccess": True,
                           "errorMsg": result["errorMsg"], "count": len(books),
                           **({"data": books, "returnData": result["returnData"]} if include_data else {})})
        return {"searches": probes, "renderCookieHeaders": fixture.snapshot(),
                "renderScriptSources": fixture.script_snapshot(),
                "renderRequestFields": fixture.request_snapshot()}
    except Exception:
        log_output.flush()
        log_lines = log_path.read_text(encoding="utf-8", errors="replace").splitlines()
        print("Reader diagnostics (local fixture only):", *log_lines[-40:], sep="\n")
        raise
    finally:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        log_output.close()


def same_json(left, right):
    # Python considers False == 0 and True == 1. JSON types must not collapse.
    def encoded(value):
        return json.dumps(value, ensure_ascii=False, sort_keys=True,
                          separators=(",", ":"), allow_nan=False)
    return encoded(left) == encoded(right)


def write_report(path, report):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8") as output:
        output.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")


def validate_generated_searches(result):
    """Reject missing or normalized-away actual generated Reader responses."""
    expected_fields = [{"httpMethod": "GET", "body": None, "testHeader": None}] * 4 + [
        {"httpMethod": "POST", "body": "q=post", "testHeader": "synthetic"}]
    if not isinstance(result, dict) or not isinstance(result.get("searches"), list) or \
            len(result["searches"]) != 5:
        raise RuntimeError("Missing executed three-way search results")
    for search in result["searches"]:
        if not isinstance(search, dict) or type(search.get("status")) is not int or \
                search["status"] != 200 or search.get("isSuccess") is not True or \
                search.get("errorMsg") != "" or search.get("count") != 1 or \
                type(search.get("count")) is not int or \
                not isinstance(search.get("data"), list) or len(search["data"]) != 1:
            raise RuntimeError("Missing actual Reader JSON in three-way report")
        raw = search.get("returnData")
        if not isinstance(raw, dict) or raw.get("isSuccess") is not True or \
                raw.get("errorMsg") != "" or not same_json(raw.get("data"), search["data"]):
            raise RuntimeError("Missing exact ReturnData response in three-way report")
    if result.get("renderRequestFields") != expected_fields:
        raise RuntimeError("Three-way target GET/POST request fields differ")


def validate_remote_pair(original, remote):
    for result in (original, remote):
        validate_generated_searches(result)
    if not same_json(original["searches"], remote["searches"]):
        raise RuntimeError("Three-way full Reader JSON differs; inspect the preserved report")
    if original.get("renderCookieHeaders") != [""] * 5 or remote.get("renderCookieHeaders") != [""] * 5:
        raise RuntimeError("Unreviewed historical renderer target Cookie behavior")


def validate_three_way(original, remote, camoufox):
    """Compare actual target requests and full generated Reader JSON results."""
    validate_remote_pair(original, remote)
    validate_generated_searches(camoufox)
    if not same_json(original["searches"], camoufox["searches"]):
        raise RuntimeError("Three-way full Reader JSON differs; inspect the preserved report")
    if camoufox.get("renderCookieHeaders") != ["", "session=alpha==", "", "", ""]:
        raise RuntimeError("Camoufox target Cookie replay/deletion differs")


def wait_for_camoufox_handoff(directory, original, remote, timeout=60):
    """Keep this fixture alive while the host stops its owned historical renderer."""
    require_verified_private_loopback()
    validate_remote_pair(original, remote)
    directory.mkdir()
    with (directory / "remote-complete").open("xb"):
        pass
    permit = directory / "camoufox-permitted"
    deadline = time.monotonic() + timeout
    while not permit.exists() and not permit.is_symlink():
        if time.monotonic() >= deadline:
            raise TimeoutError("Historical renderer handoff was not acknowledged")
        time.sleep(0.1)
    if permit.is_symlink() or not permit.is_file() or permit.stat().st_size != 0:
        raise RuntimeError("Invalid historical renderer handoff acknowledgement")
    require_verified_private_loopback()
    try:
        with socket.create_connection(("127.0.0.1", 8050), timeout=2):
            pass
    except ConnectionRefusedError:
        return
    raise RuntimeError("Historical renderer is still running before Camoufox")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", type=Path, default=ROOT / ".tools/jdk-11.0.8/bin/java.exe")
    parser.add_argument("--original", type=Path,
                        default=ROOT / "reference/original/reader-pro-3.2.14.original.jar")
    parser.add_argument("--restored", type=Path,
                        default=ROOT / "build/libs/reader-4.0.7.jar")
    parser.add_argument("--report", type=Path, required=True,
                        help="New report path; existing files are never overwritten")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--restored-only", action="store_true",
                      help="Do not launch the original JAR on this host")
    mode.add_argument("--original-network-isolated", action="store_true",
                      help="Run both JARs only after independently verifying inbound isolation")
    parser.add_argument("--exercise-script", action="store_true",
                        help="Also verify a synthetic webJs rule is sent as js_source")
    parser.add_argument("--exercise-post", action="store_true",
                        help="Also verify a synthetic WebView POST method, body, and header")
    parser.add_argument("--archived-renderer-base",
                        help="Use the actual archived renderer on private loopback, not a synthetic /render.html")
    parser.add_argument("--camoufox-python", type=Path,
                        help="Additionally run the same restored JAR with real Camoufox in the same private netns")
    parser.add_argument("--phase-handoff-dir", type=Path,
                        help="New directory next to the report for host-coordinated renderer shutdown")
    args = parser.parse_args()
    if args.report.exists() or args.report.is_symlink():
        parser.error(f"Report already exists; choose a new path: {args.report}")
    if args.camoufox_python and not (args.original_network_isolated and args.archived_renderer_base
                                    and args.exercise_script and args.exercise_post):
        parser.error("Three-way Camoufox mode requires isolated original-JAR mode, actual archived renderer, script and POST probes")
    if args.phase_handoff_dir and (not args.camoufox_python or
            not args.phase_handoff_dir.is_absolute() or args.phase_handoff_dir.exists() or
            args.phase_handoff_dir.is_symlink() or
            args.phase_handoff_dir.parent.resolve() != args.report.parent.resolve()):
        parser.error("Phase handoff requires three-way mode and a new absolute directory next to the report")
    if args.original_network_isolated:
        from original_jar_safety import require_original_jar_isolation

        require_original_jar_isolation()
    if args.archived_renderer_base:
        if not args.original_network_isolated or args.archived_renderer_base != "http://127.0.0.1:8050":
            parser.error("Archived renderer requires isolated original-JAR mode and private 127.0.0.1:8050")
        require_verified_private_loopback()
    for path in (args.java, args.original, args.restored):
        if not path.is_file():
            parser.error(f"Required file not found: {path}")
    if args.camoufox_python:
        if not args.camoufox_python.is_absolute() or not args.camoufox_python.is_file() or \
                not os.access(args.camoufox_python, os.X_OK):
            parser.error("Camoufox Python must be an existing absolute executable path")
        try:
            preflight = subprocess.run([str(args.camoufox_python), "-c",
                                        "import camoufox.sync_api; import playwright.sync_api"],
                                       capture_output=True, timeout=15, check=False)
        except (OSError, subprocess.TimeoutExpired):
            parser.error("Camoufox Python preflight failed; refusing to start any JAR")
        if preflight.returncode != 0:
            parser.error("Camoufox/Playwright imports unavailable; refusing to start any JAR")
    fixture_port = free_port()
    fixture = Fixture(("127.0.0.1", fixture_port),
                      archived_renderer=bool(args.archived_renderer_base))
    fixture_base = f"http://127.0.0.1:{fixture_port}"
    worker = threading.Thread(target=fixture.serve_forever, daemon=True)
    worker.start()
    try:
        with tempfile.TemporaryDirectory(prefix="reader-webview-diff-") as directory:
            root = Path(directory)
            original = None
            camoufox = None
            if args.original_network_isolated:
                fixture.reset()
                original = run_jar(args.java, args.original, root / "original",
                                   free_port(), fixture_base, fixture,
                                   args.exercise_script, args.exercise_post,
                                   args.archived_renderer_base,
                                   include_data=bool(args.camoufox_python))
            fixture.reset()
            restored = run_jar(args.java, args.restored, root / "restored",
                               free_port(), fixture_base, fixture,
                               args.exercise_script, args.exercise_post,
                               args.archived_renderer_base,
                               include_data=bool(args.camoufox_python))
            if args.camoufox_python:
                if args.phase_handoff_dir:
                    wait_for_camoufox_handoff(args.phase_handoff_dir, original, restored)
                fixture.reset()
                camoufox = run_jar(args.java, args.restored, root / "camoufox",
                                   free_port(), fixture_base, fixture, True, True,
                                   args.archived_renderer_base, args.camoufox_python,
                                   include_data=True)
    finally:
        fixture.shutdown()
        fixture.server_close()
        worker.join(timeout=5)

    expected_original = ["", "", ""] + ([""] if args.exercise_script else []) + \
                        ([""] if args.exercise_post else [])
    expected_restored = (["", "", ""] if args.archived_renderer_base else
                         ["", "session=alpha==", ""]) + \
                        ([""] if args.exercise_script else []) + ([""] if args.exercise_post else [])
    script = ("document.documentElement.outerHTML.replace('WebView脚本原始书','WebView差分书')" if args.archived_renderer_base
              else "document.title")
    expected_scripts = [None, None, None] + \
                       ([script] if args.exercise_script else []) + \
                       ([script] if args.exercise_post else [])
    expected_post = {"httpMethod": "POST", "body": "q=post", "testHeader": "synthetic"}
    report = {
        "originalJarSha256": sha256(args.original),
        "originalExecuted": original is not None,
        "restoredJarSha256": sha256(args.restored),
        "rendererMode": "archived-webkit" if args.archived_renderer_base else "synthetic-render-response",
        "observedRequestEndpoint": "target /search" if args.archived_renderer_base else "synthetic /render.html",
        "jsSourceDirectlyObserved": not bool(args.archived_renderer_base),
        "scriptValidatedByResult": bool(args.archived_renderer_base and args.exercise_script),
        "original": original,
        "restored": restored,
        "expectedOriginalCookieSequence": expected_original,
        "expectedRestoredCookieSequence": expected_restored,
        "expectedScriptSources": expected_scripts,
    }
    if args.exercise_post:
        report["expectedPostRequestFields"] = expected_post
    if args.camoufox_python:
        report.update({"comparisonMode": "same-run-original-remote-camoufox",
                       "camoufox": camoufox,
                       "camoufoxJarSha256": report["restoredJarSha256"],
                       "expectedCamoufoxCookieSequence": ["", "session=alpha==", "", "", ""],
                       "fullGeneratedReaderJsonRecorded": True,
                       "historicalRendererStoppedBeforeCamoufox": bool(args.phase_handoff_dir),
                       "originalProductionRendererVersionProven": False,
                       "realAuthenticatedSourceTested": False})
    write_report(args.report, report)
    endpoint = "target" if args.archived_renderer_base else "render"
    if original is not None:
        print(f"Original {endpoint} Cookie sequence: {original['renderCookieHeaders']}")
    else:
        print("Original JAR not executed; recorded original expectations are not a current observation")
    print(f"Restored {endpoint} Cookie sequence: {restored['renderCookieHeaders']}")
    if original is not None and not args.archived_renderer_base:
        print(f"Original render script sequence: {original['renderScriptSources']}")
    if not args.archived_renderer_base:
        print(f"Restored render script sequence: {restored['renderScriptSources']}")
    if args.exercise_post:
        if original is not None:
            print(f"Original POST request fields: {original['renderRequestFields'][-1]}")
        print(f"Restored POST request fields: {restored['renderRequestFields'][-1]}")
    print(f"Report: {args.report}")
    if args.camoufox_python:
        validate_three_way(original, restored, camoufox)
    expected_fields = ([{"httpMethod": "GET", "body": None, "testHeader": None}] *
                       (3 + int(args.exercise_script)) +
                       ([expected_post] if args.exercise_post else []))
    if restored["renderCookieHeaders"] != expected_restored or \
            (args.archived_renderer_base and restored["renderRequestFields"] != expected_fields) or \
            (not args.archived_renderer_base and restored["renderScriptSources"] != expected_scripts) or \
            (args.exercise_post and restored["renderRequestFields"][-1] != expected_post) or \
            (original is not None and (
                original["renderCookieHeaders"] != expected_original or
                original["searches"] != restored["searches"] or
                (args.archived_renderer_base and original["renderRequestFields"] != expected_fields) or
                (not args.archived_renderer_base and original["renderScriptSources"] != expected_scripts) or
                (args.exercise_post and (
                    original["renderRequestFields"][-1] != expected_post or
                    original["renderRequestFields"] != restored["renderRequestFields"])))):
        raise RuntimeError("Unreviewed WebView Cookie differential")


if __name__ == "__main__":
    main()
