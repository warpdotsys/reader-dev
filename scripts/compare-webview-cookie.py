"""Loopback-only WebView Cookie differential for the original and restored JARs.

No production storage, third-party site, browser binary or real credentials are used.
"""

import argparse
import hashlib
import http.cookiejar
import json
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
    def __init__(self, address):
        super().__init__(address, FixtureHandler)
        self.calls = []
        self.lock = threading.Lock()

    def reset(self):
        with self.lock:
            self.calls.clear()

    def snapshot(self):
        with self.lock:
            return list(self.calls)


class FixtureHandler(BaseHTTPRequestHandler):
    def do_POST(self):
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
            with self.server.lock:
                self.server.calls.append(cookie)
                call_number = len(self.server.calls)
        except (ValueError, KeyError, TypeError):
            self.send_error(400)
            return
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


def require_success(opener, base, path, body=None):
    status, value = request(opener, base, path, body)
    if status != 200 or value.get("isSuccess") is not True:
        raise RuntimeError(f"{path}: HTTP {status}, {value.get('errorMsg')}")
    return {"status": status, "isSuccess": True,
            "errorMsg": value.get("errorMsg", ""), "data": value.get("data")}


def run_jar(java, jar, workdir, port, fixture_base, fixture):
    base = f"http://127.0.0.1:{port}"
    workdir.mkdir(parents=True, exist_ok=True)
    launch = [str(java), "-Xms128m", "-Xmx768m", "-jar", str(jar),
              f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
              f"--reader.app.remote-webview-api={fixture_base}",
              "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
              "--spring.profiles.active=prod"]
    flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
    log_path = workdir / "reader.log"
    log_output = log_path.open("wb")
    process = subprocess.Popen(launch, cwd=ROOT, stdout=log_output,
                               stderr=subprocess.STDOUT, creationflags=flags)
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
                                      "bookSourceUrl": fixture_base})
            books = result["data"]
            if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "WebView差分书":
                names = [book.get("name") for book in books] if isinstance(books, list) else None
                raise RuntimeError(f"Unexpected search result for {key}: "
                                   f"names={names}, renderCalls={len(fixture.snapshot())}")
            probes.append({"status": result["status"], "isSuccess": True,
                           "errorMsg": result["errorMsg"], "count": len(books)})
        return {"searches": probes, "renderCookieHeaders": fixture.snapshot()}
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", type=Path, default=ROOT / ".tools/jdk-11.0.8/bin/java.exe")
    parser.add_argument("--original", type=Path,
                        default=ROOT / "reference/original/reader-pro-3.2.14.original.jar")
    parser.add_argument("--restored", type=Path,
                        default=ROOT / "build/libs/reader-4.0.7.jar")
    parser.add_argument("--report", type=Path,
                        default=ROOT / "reports/webview-cookie-diff-latest.json")
    args = parser.parse_args()
    for path in (args.java, args.original, args.restored):
        if not path.is_file():
            parser.error(f"Required file not found: {path}")
    fixture_port = free_port()
    fixture = Fixture(("127.0.0.1", fixture_port))
    fixture_base = f"http://127.0.0.1:{fixture_port}"
    worker = threading.Thread(target=fixture.serve_forever, daemon=True)
    worker.start()
    try:
        with tempfile.TemporaryDirectory(prefix="reader-webview-diff-") as directory:
            root = Path(directory)
            fixture.reset()
            original = run_jar(args.java, args.original, root / "original",
                               free_port(), fixture_base, fixture)
            fixture.reset()
            restored = run_jar(args.java, args.restored, root / "restored",
                               free_port(), fixture_base, fixture)
    finally:
        fixture.shutdown()
        fixture.server_close()
        worker.join(timeout=5)

    expected_original = ["", "", ""]
    expected_restored = ["", "session=alpha==", ""]
    report = {
        "originalJarSha256": sha256(args.original),
        "restoredJarSha256": sha256(args.restored),
        "original": original,
        "restored": restored,
        "expectedOriginalCookieSequence": expected_original,
        "expectedRestoredCookieSequence": expected_restored,
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                           encoding="utf-8")
    print(f"Original render Cookie sequence: {original['renderCookieHeaders']}")
    print(f"Restored render Cookie sequence: {restored['renderCookieHeaders']}")
    print(f"Report: {args.report}")
    if original["renderCookieHeaders"] != expected_original or \
            restored["renderCookieHeaders"] != expected_restored or \
            original["searches"] != restored["searches"]:
        raise RuntimeError("Unreviewed WebView Cookie differential")


if __name__ == "__main__":
    main()
