#!/usr/bin/env python3
"""Compare non-SSE multi-source search and alternative-source search.

The source is a loopback-only HTML fixture. Both JARs use isolated accounts.
"""

import hashlib
from http.server import ThreadingHTTPServer
import http.cookiejar
import importlib.util
import json
from copy import deepcopy
from pathlib import Path
import secrets
import subprocess
from threading import Thread
import time
import urllib.parse
import urllib.request
import urllib.error


ROOT = Path(__file__).resolve().parents[1]
SSE_SPEC = importlib.util.spec_from_file_location(
    "sse_base", Path(__file__).with_name("compare-sse-pagination.py"))
SSE = importlib.util.module_from_spec(SSE_SPEC)
SSE_SPEC.loader.exec_module(SSE)
CACHE_SPEC = importlib.util.spec_from_file_location(
    "cache_base", Path(__file__).with_name("compare-cache-book-sse.py"))
CACHE = importlib.util.module_from_spec(CACHE_SPEC)
CACHE_SPEC.loader.exec_module(CACHE)
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/search-json-diff-latest.json"


class SearchHandler(CACHE.BookHandler):
    def do_GET(self):
        parsed = urllib.parse.urlsplit(self.path)
        if parsed.path != "/search":
            return super().do_GET()
        key = urllib.parse.parse_qs(parsed.query).get("key", [""])[0]
        body = ("<html><div class='book'><a href='/book'>"
                "<span class='name'>Cache Fixture</span></a>"
                "<span class='author'>Fixture Author</span></div></html>"
                if key in ("Cache", "Cache Fixture") else "<html></html>")
        data = body.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def json_request(session, base, route, method, payload=None):
    if method == "GET":
        path = route + ("?" + urllib.parse.urlencode(payload) if payload else "")
        request = urllib.request.Request(base + path, method="GET")
    else:
        body = json.dumps(payload or {}, ensure_ascii=False).encode("utf-8")
        request = urllib.request.Request(
            base + route, data=body, method="POST",
            headers={"Content-Type": "application/json; charset=utf-8"})
    try:
        response = session.open(request, timeout=35)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        return {"status": response.status,
                "contentType": response.headers.get("Content-Type", ""),
                "body": json.loads(response.read().decode("utf-8"))}


def normalize_elapsed(row):
    normalized = deepcopy(row)
    observed = []
    data = normalized["body"].get("data")
    if isinstance(data, dict) and isinstance(data.get("list"), list):
        for book in data["list"]:
            if "time" in book:
                value = book["time"]
                if type(value) is not int or not (0 <= value <= 120_000):
                    raise AssertionError(f"Invalid search elapsed time: {value}")
                observed.append(value)
                book["time"] = "<elapsed-ms>"
    return normalized, observed


def accepted_missing_url_fix(left, right):
    old = left["body"]
    new = right["body"]
    return (left["probe"] == right["probe"] == "missing-url-post"
            and left["status"] == right["status"] == 200
            and left["contentType"] == right["contentType"] == "application/json; charset=utf-8"
            and old == {"isSuccess": False,
                        "errorMsg": 'java.lang.NullPointerException: context.bodyAsJson.getString("url") must not be null'}
            and new == {"isSuccess": False, "errorMsg": "请输入书籍链接"})


def assert_contract(comparisons, fixture_base):
    by_name = {row["probe"]: row for row in comparisons}
    errors = {
        "anonymous-multi-get": "请登录后使用",
        "anonymous-multi-post": "请登录后使用",
        "anonymous-source-get": "请登录后使用",
        "anonymous-source-post": "请登录后使用",
        "no-source-multi": "未配置书源",
        "no-source-alternate": "未配置书源",
        "missing-key-get": "请输入搜索关键字",
        "missing-key-post": "请输入搜索关键字",
        "missing-url-get": "请输入书籍链接",
        "missing-url-post": 'java.lang.NullPointerException: context.bodyAsJson.getString("url") must not be null',
        "unknown-book": "书籍信息错误",
        "other-user-no-source": "未配置书源",
        "multi-exhausted": "没有更多了",
        "alternate-exhausted": "没有更多了",
    }
    for name, message in errors.items():
        row = by_name[name]["original"]
        body = row["body"]
        if (row["status"] != 200 or row["contentType"] != "application/json; charset=utf-8"
                or body.get("isSuccess") is not False or body.get("errorMsg") != message
                or (name.startswith("anonymous-") and body.get("data") != "NEED_LOGIN")):
            raise AssertionError(f"Unexpected JSON search error: {name}: {row}")
    fixed = by_name["missing-url-post"]["restored"]["body"]
    if fixed != by_name["missing-url-get"]["restored"]["body"]:
        raise AssertionError("POST missing URL is not aligned with the GET error")
    expected = {
        "multi-one-get": (0, ["source-one"]),
        "multi-one-post": (0, ["source-one"]),
        "alternate-one-get": (0, ["source-one"]),
        "alternate-one-post": (0, ["source-one"]),
        "multi-two-get": (0, ["source-one"]),
        "multi-second-page": (1, ["source-two"]),
        "multi-group-a": (0, ["source-one"]),
        "alternate-two-get": (1, ["source-one", "source-two"]),
        "alternate-second-page": (1, ["source-two"]),
    }
    for name, (cursor, suffixes) in expected.items():
        comparison = by_name[name]
        row = comparison["original"]
        body = row["body"]
        data = body.get("data", {})
        books = data.get("list", []) if isinstance(data, dict) else []
        if (row["status"] != 200 or row["contentType"] != "application/json; charset=utf-8"
                or body.get("isSuccess") is not True or body.get("errorMsg") != ""
                or data.get("lastIndex") != cursor or len(books) != len(suffixes)
                or len(comparison["originalElapsedMs"]) != len(books)
                or len(comparison["restoredElapsedMs"]) != len(books)):
            raise AssertionError(f"Unexpected JSON search success: {name}: {row}")
        for book, suffix in zip(books, suffixes):
            if (book.get("origin") != fixture_base + "/" + suffix
                    or book.get("bookUrl") != fixture_base + "/book"
                    or book.get("name") != "Cache Fixture"
                    or book.get("author") != "Fixture Author"
                    or book.get("time") != "<elapsed-ms>"):
                raise AssertionError(f"Unexpected JSON search book: {name}: {book}")
    for left, right in (("multi-one-get", "multi-one-post"),
                        ("alternate-one-get", "alternate-one-post")):
        if by_name[left]["original"]["body"] != by_name[right]["original"]["body"]:
            raise AssertionError(f"GET and POST search payloads differ: {left}, {right}")


def source(fixture_base, suffix, group):
    result = SSE.source(fixture_base, suffix)
    result["bookSourceGroup"] = group
    return result


def run_one(jar, workdir, username, other_user, password, fixture_base):
    workdir.mkdir(parents=True)
    port = SSE.free_port()
    base = f"http://127.0.0.1:{port}"
    book_url = fixture_base + "/book"
    first_source = source(fixture_base, "source-one", "GroupA")
    second_source = source(fixture_base, "source-two", "GroupB")
    command = [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
               f"--reader.server.port={port}", "--reader.app.secure=true",
               "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"]
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        anonymous, owner, other = client(), client(), client()
        probes = []

        def add(name, session, route, method, payload=None):
            row = {"probe": name, **json_request(session, base, route, method, payload)}
            probes.append(row)
            return row

        try:
            deadline = time.monotonic() + 65
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"Reader exited during startup: {process.returncode}")
                try:
                    if SSE.get_json(anonymous, base, "/reader3/getSystemInfo")[1].get("isSuccess"):
                        break
                except OSError:
                    pass
                time.sleep(0.4)
            else:
                raise TimeoutError("Reader startup timeout")
            multi, alternate = "/reader3/searchBookMulti", "/reader3/searchBookSource"
            add("anonymous-multi-get", anonymous, multi, "GET", {"key": "Cache"})
            add("anonymous-multi-post", anonymous, multi, "POST", {"key": "Cache"})
            add("anonymous-source-get", anonymous, alternate, "GET", {"url": book_url})
            add("anonymous-source-post", anonymous, alternate, "POST", {"url": book_url})
            for session, name in ((owner, username), (other, other_user)):
                for is_login in (False, True):
                    SSE.require_success(SSE.get_json(session, base, "/reader3/login",
                                                     {"username": name, "password": password,
                                                      "isLogin": is_login}), "fixture login")
            add("no-source-multi", owner, multi, "GET", {"key": "Cache"})
            add("no-source-alternate", owner, alternate, "GET", {"url": book_url})
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBookSource",
                                             first_source), "save source one")
            add("missing-key-get", owner, multi, "GET")
            add("missing-key-post", owner, multi, "POST", {})
            add("missing-url-get", owner, alternate, "GET")
            add("missing-url-post", owner, alternate, "POST", {})
            add("unknown-book", owner, alternate, "GET", {"url": fixture_base + "/none"})
            book_info = SSE.require_success(SSE.get_json(
                owner, base, "/reader3/getBookInfo",
                {"url": book_url, "bookSource": first_source}), "book info")
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBook",
                                             book_info), "save book")
            one = {"key": "Cache", "lastIndex": -1, "searchSize": 1,
                   "concurrentCount": 1}
            add("multi-one-get", owner, multi, "GET", one)
            add("multi-one-post", owner, multi, "POST", one)
            search = {"url": book_url, "lastIndex": -1, "searchSize": 1}
            add("alternate-one-get", owner, alternate, "GET", search)
            add("alternate-one-post", owner, alternate, "POST", search)
            add("other-user-no-source", other, multi, "GET", {"key": "Cache"})
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBookSource",
                                             second_source), "save source two")
            add("multi-two-get", owner, multi, "GET", one)
            add("multi-second-page", owner, multi, "GET",
                {"key": "Cache", "lastIndex": 0, "searchSize": 1,
                 "concurrentCount": 1})
            add("multi-group-a", owner, multi, "GET",
                {"key": "Cache", "bookSourceGroup": "GroupA", "lastIndex": -1,
                 "searchSize": 1, "concurrentCount": 1})
            add("alternate-two-get", owner, alternate, "GET", search)
            add("alternate-second-page", owner, alternate, "GET",
                {"url": book_url, "lastIndex": 0, "searchSize": 1})
            add("multi-exhausted", owner, multi, "GET",
                {"key": "Cache", "lastIndex": 1})
            add("alternate-exhausted", owner, alternate, "GET",
                {"url": book_url, "lastIndex": 1})
            return probes
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def main():
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    run_root = ROOT / ".tools" / ("search-json-diff-" + secrets.token_hex(8))
    run_root.mkdir(parents=True)
    server = ThreadingHTTPServer(("127.0.0.1", 0), SearchHandler)
    fixture_base = f"http://127.0.0.1:{server.server_port}"
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        username = "jsonsearch" + secrets.token_hex(5)
        other_user = "jsonother" + secrets.token_hex(5)
        password = "Probe-" + secrets.token_hex(18)
        original = run_one(ORIGINAL, run_root / "original", username, other_user,
                           password, fixture_base)
        restored = run_one(RESTORED, run_root / "restored", username, other_user,
                           password, fixture_base)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("JSON search probe sequences differ")
    comparisons = []
    for left, right in zip(original, restored):
        normalized_left, left_times = normalize_elapsed(left)
        normalized_right, right_times = normalize_elapsed(right)
        comparisons.append({"probe": left["probe"],
                            "equal": normalized_left == normalized_right,
                            "acceptedMissingUrlFix": normalized_left != normalized_right
                            and accepted_missing_url_fix(normalized_left, normalized_right),
                            "originalElapsedMs": left_times,
                            "restoredElapsedMs": right_times,
                            "original": normalized_left, "restored": normalized_right})
    report = {"originalJarSha256": sha256(ORIGINAL),
              "restoredJarSha256": sha256(RESTORED), "comparisons": comparisons}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        outcome = ('equal' if row['equal'] else 'reviewed fix' if row['acceptedMissingUrlFix']
                   else 'UNKNOWN DIFFERENCE')
        print(f"{row['probe']}: {outcome}")
    print(f"Report: {REPORT}")
    print(f"Isolated data: {run_root}")
    assert_contract(comparisons, fixture_base)
    if not all(row["equal"] or row["acceptedMissingUrlFix"] for row in comparisons):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
