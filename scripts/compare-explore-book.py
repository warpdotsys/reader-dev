#!/usr/bin/env python3
"""Compare GET/POST discovery against a deterministic two-page loopback source."""

import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
import json
from pathlib import Path
import secrets
import subprocess
import tempfile
from threading import Thread
import time
from urllib.parse import parse_qs, urlsplit


ROOT = Path(__file__).resolve().parents[1]
BASE_SPEC = importlib.util.spec_from_file_location(
    "import_base", Path(__file__).with_name("compare-import-entrypoints.py"))
BASE = importlib.util.module_from_spec(BASE_SPEC)
BASE_SPEC.loader.exec_module(BASE)
REPORT = ROOT / "reports/explore-book-diff-latest.json"


class Handler(BaseHTTPRequestHandler):
    seen = []

    def do_GET(self):
        parsed = urlsplit(self.path)
        self.seen.append(self.path)
        if parsed.path != "/explore":
            self.send_error(404)
            return
        page = parse_qs(parsed.query).get("page", ["1"])[0]
        if page == "1":
            title, number = "Explore One", "1"
        elif page == "2":
            title, number = "Explore Two", "2"
        else:
            title, number = None, None
        body = ("<html><div class='book'><a href='/book/" + number + "'>"
                "<span class='name'>" + title + "</span></a>"
                "<span class='author'>Fixture Author</span></div></html>") if title else "<html></html>"
        data = body.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, format, *args):
        pass


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_one(jar, workdir, username, other_user, password, fixture_base):
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(BASE.JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    anonymous, owner, other = BASE.client(), BASE.client(), BASE.client()
    probes = []

    def add(name, session, method, payload=None):
        path = "/reader3/exploreBook"
        if method == "GET" and payload:
            from urllib.parse import urlencode
            path += "?" + urlencode(payload)
            payload = None
        result = BASE.send(session, base, method, path, payload)
        probes.append({"probe": name, **result})
        return result

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited: {process.returncode}")
            try:
                if BASE.send(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except OSError:
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")

        BASE.expect(add("anonymous-no-source-get", anonymous, "GET"),
                    False, "anonymous-no-source-get")
        BASE.expect(add("anonymous-no-source-post", anonymous, "POST", {}),
                    False, "anonymous-no-source-post")
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                BASE.expect(BASE.send(session, base, "POST", "/reader3/login",
                                      {"username": name, "password": password,
                                       "isLogin": login}), True, "login")
        BASE.expect(add("owner-no-source-get", owner, "GET"),
                    False, "owner-no-source-get")
        BASE.expect(add("other-no-source-post", other, "POST", {}),
                    False, "other-no-source-post")

        source = {
            "bookSourceUrl": fixture_base, "bookSourceName": "Explore fixture",
            "exploreUrl": fixture_base + "/explore?page=<1,2>",
            "ruleExplore": {"bookList": ".book", "name": ".name@text",
                            "author": ".author@text", "bookUrl": "a@href"},
            "ruleToc": {},
        }
        BASE.expect(BASE.send(owner, base, "POST", "/reader3/saveBookSource", source),
                    True, "save explore source")
        saved = BASE.send(owner, base, "GET", "/reader3/getBookSources")
        if saved["body"]["data"][0].get("ruleExplore", {}).get("bookList") != ".book":
            raise AssertionError("Explore rule was not saved")
        first = {"bookSourceUrl": fixture_base,
                 "ruleFindUrl": fixture_base + "/explore?page=<1,2>", "page": 1}
        second = dict(first, page=2)
        BASE.expect(add("page-one-get", owner, "GET", first), True, "page-one-get")
        BASE.expect(add("page-one-post", owner, "POST", first), True, "page-one-post")
        BASE.expect(add("page-two-get", owner, "GET", second), True, "page-two-get")
        BASE.expect(add("page-two-post", owner, "POST", second), True, "page-two-post")
        BASE.expect(add("other-no-source-after", other, "GET", first),
                    False, "other-no-source-after")
        return probes
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main():
    for path in (BASE.JAVA, BASE.ORIGINAL, BASE.RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    username = "exploreprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    Handler.seen = []
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    fixture_base = f"http://127.0.0.1:{server.server_port}"
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        with tempfile.TemporaryDirectory(prefix="reader-explore-diff-") as temp:
            root = Path(temp)
            original = run_one(BASE.ORIGINAL, root / "original", username,
                               other_user, password, fixture_base)
            restored = run_one(BASE.RESTORED, root / "restored", username,
                               other_user, password, fixture_base)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("Probe sequence differs")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "original": left, "restored": right}
                   for left, right in zip(original, restored)]
    by_name = {row["probe"]: row for row in comparisons}
    expected_paths = (["/explore?page=1", "/explore?page=1",
                       "/explore?page=2", "/explore?page=2"] * 2)
    if len(comparisons) != 9 or not all(row["equal"] for row in comparisons):
        raise AssertionError("Explore responses differ")
    if Handler.seen != expected_paths:
        raise AssertionError(f"Unexpected fixture requests: {Handler.seen}")
    for number, title in ((1, "Explore One"), (2, "Explore Two")):
        get_body = by_name[f"page-{('one' if number == 1 else 'two')}-get"]["restored"]["body"]
        post_body = by_name[f"page-{('one' if number == 1 else 'two')}-post"]["restored"]["body"]
        if get_body != post_body or get_body.get("isSuccess") is not True or get_body.get("errorMsg") != "":
            raise AssertionError(f"GET/POST discovery differs on page {number}")
        books = get_body.get("data")
        if (not isinstance(books, list) or len(books) != 1
                or books[0].get("bookUrl") != f"{fixture_base}/book/{number}"
                or books[0].get("origin") != fixture_base
                or books[0].get("name") != title
                or books[0].get("author") != "Fixture Author"):
            raise AssertionError(f"Discovery page {number} did not return its expected book")
    if by_name["other-no-source-after"]["restored"]["body"] != {
            "isSuccess": False, "errorMsg": "未配置书源"}:
        raise AssertionError("Discovery source crossed user namespace")
    report = {"originalJarSha256": sha256(BASE.ORIGINAL),
              "restoredJarSha256": sha256(BASE.RESTORED),
              "fixturePaths": Handler.seen, "comparisons": comparisons}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                      encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'DIFFERENT'}")
    print("Report:", REPORT)


if __name__ == "__main__":
    main()
