"""Exercise an opt-in local WebView Reader against a loopback-only book source.

The target Reader must use an isolated work directory; this script registers a
synthetic source and disposable accounts there, then runs a bounded burst of
independent users. It refuses non-loopback Reader addresses.
"""

import argparse
import concurrent.futures
import http.cookiejar
import json
import secrets
import threading
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Fixture(ThreadingHTTPServer):
    def __init__(self, address):
        super().__init__(address, FixtureHandler)
        self.cookies = []
        self.requests = []
        self.async_marks = []
        self.lock = threading.Lock()


class FixtureHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if not self.path.startswith("/search"):
            self.send_error(404)
            return
        self.serve_search("GET", None)

    def do_POST(self):
        if self.path == "/async-mark":
            length = int(self.headers.get("Content-Length", "0"))
            if not 0 <= length <= 1024:
                self.send_error(400)
                return
            body = self.rfile.read(length).decode("utf-8")
            if body not in ("phase=get", "phase=post"):
                self.send_error(400)
                return
            with self.server.lock:
                self.server.async_marks.append({"body": body,
                    "cookie": self.headers.get("Cookie", "")})
            self.send_response(200)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        if self.path not in ("/search-post", "/search-async-post"):
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length", "0"))
        if not 0 <= length <= 65536:
            self.send_error(400)
            return
        self.serve_search("POST", self.rfile.read(length).decode("utf-8"))

    def serve_search(self, method, body):
        cookie = self.headers.get("Cookie", "")
        with self.server.lock:
            self.server.cookies.append(cookie)
            self.server.requests.append({"httpMethod": method, "body": body,
                                         "testHeader": self.headers.get("X-Fixture")})
            number = len(self.server.cookies)
        name = ("WebView脚本原始书" if self.path in ("/search-script", "/search-post", "/search-async", "/search-async-post")
                else "本地浏览器测试书")
        html = ("<html><div class='book'><a href='/book'>"
                f"<span class='name'>{name}</span></a>"
                "<span class='author'>测试作者</span></div></html>")
        if self.path in ("/search-async", "/search-async-post"):
            # A genuine DOM delay, not a server response containing the expected
            # book name. Only the source Promise can produce the accepted name.
            fragment = ("<div class='book'><a href='/book'><span class='name'>"
                        "WebView脚本原始书</span></a><span class='author'>测试作者</span></div>")
            html = ("<html><body><main id='generated-result'></main><script>"
                    "setTimeout(() => { document.querySelector('#generated-result').innerHTML = "
                    + json.dumps(fragment, ensure_ascii=False) + "; }, 300);</script></body></html>")
        data = html.encode("utf-8")
        self.send_response(200)
        if number == 1:
            self.send_header("Set-Cookie", "session=alpha==; Path=/; HttpOnly; SameSite=Lax")
        elif number == 3:
            self.send_header("Set-Cookie", "session=; Max-Age=0; Path=/")
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, _format, *_args):
        pass


def call(opener, base, path, body):
    payload = json.dumps(body, ensure_ascii=False).encode("utf-8")
    request = urllib.request.Request(
        base + path, data=payload, method="POST",
        headers={"Content-Type": "application/json; charset=utf-8"})
    with opener.open(request, timeout=35) as response:
        value = json.loads(response.read().decode("utf-8"))
        if response.status != 200 or value.get("isSuccess") is not True:
            raise RuntimeError(f"{path}: HTTP {response.status}, {value.get('errorMsg')}")
        return value


def get_json(opener, base, path):
    with opener.open(base + path, timeout=35) as response:
        value = json.loads(response.read().decode("utf-8"))
        if response.status != 200 or value.get("isSuccess") is not True:
            raise RuntimeError(f"{path}: HTTP {response.status}, {value.get('errorMsg')}")
        return value


def async_source_script(phase):
    if phase not in ("get", "post"):
        raise ValueError("Unexpected generated async phase")
    cookie = ("asyncOnly=generated; Path=/" if phase == "get" else
              "asyncOnly=; Max-Age=0; Path=/")
    return ("new Promise((resolve, reject) => { const started = performance.now(); "
            "const read = () => { const node = document.querySelector('.book .name'); "
            "if (!node) { if (performance.now() - started > 5000) { "
            "reject(new Error('GeneratedDomWaitTimedOut')); return; } "
            "setTimeout(read, 25); return; } "
            "node.textContent = 'WebView异步书'; document.cookie = " + json.dumps(cookie) + "; "
            "fetch('/async-mark', {method:'POST', body:" + json.dumps("phase=" + phase) + "})"
            ".then(response => { if (!response.ok) throw new Error('GeneratedMarkerFailed'); "
            "resolve(document.documentElement.outerHTML); }).catch(reject); }; "
            "setTimeout(read, 75); })")


def exercise_async_reader(opener, reader_base, fixture, source):
    """Actual Reader API calls; generated target/marker counts detect replays."""
    fixture_base = source["bookSourceUrl"]
    cases = []
    start = len(fixture.requests)
    for phase, path, method, body, header, cookie in (
            ("get", "/search-async", "GET", None, None, ""),
            ("post", "/search-async-post", "POST", "q=async", "async", "asyncOnly=generated")):
        options = {"webView": True, "webJs": async_source_script(phase)}
        if method == "POST":
            options.update(method=method, body=body, headers={"X-Fixture": header})
        updated = dict(source)
        updated["searchUrl"] = fixture_base + path + ", " + json.dumps(options)
        call(opener, reader_base, "/reader3/saveBookSource", updated)
        value = call(opener, reader_base, "/reader3/searchBook", {
            "key": "async-" + phase, "page": 1, "bookSourceUrl": fixture_base})
        books = value.get("data")
        if (value.get("isSuccess") is not True or not isinstance(books, list) or len(books) != 1 or
                books[0].get("name") != "WebView异步书" or books[0].get("author") != "测试作者" or
                books[0].get("bookUrl") != fixture_base + "/book" or value.get("errorMsg") != ""):
            raise RuntimeError("Reader did not parse the generated async result")
        target = {"httpMethod": method, "body": body, "testHeader": header}
        expected_requests = start + len(cases) + 1
        marks = [mark for mark in fixture.async_marks if mark["body"] == "phase=" + phase]
        mark_cookie = "asyncOnly=generated" if phase == "get" else ""
        if (len(fixture.requests) != expected_requests or fixture.requests[-1] != target or
                fixture.cookies[-1] != cookie or len(marks) != 1 or marks[0]["cookie"] != mark_cookie):
            raise RuntimeError("Async target/script replay or generated Cookie mismatch")
        cases.append({"case": phase, "status": 200, "isSuccess": value["isSuccess"],
            "errorMsg": value["errorMsg"], "count": len(books), "nameMatches": True,
            "authorMatches": True, "bookUrlMatches": True, "targetRequest": target,
            "targetCookie": fixture.cookies[-1], "scriptMarkCount": len(marks),
            "scriptMarkCookie": marks[0]["cookie"]})
    call(opener, reader_base, "/reader3/saveBookSource", source)
    value = call(opener, reader_base, "/reader3/searchBook", {
        "key": "async-cleanup-followup", "page": 1, "bookSourceUrl": fixture_base})
    books = value.get("data")
    if (value.get("isSuccess") is not True or value.get("errorMsg") != "" or
            not isinstance(books, list) or len(books) != 1 or
            books[0].get("name") != "本地浏览器测试书" or
            len(fixture.requests) != start + 3 or fixture.cookies[-1] != ""):
        raise RuntimeError("Async Cookie deletion did not persist into the same user's next Reader call")
    return cases, {"sameUserNextRequestCookie": fixture.cookies[-1],
                   "sameUserNextRequestCount": len(books), "deletedVerified": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reader-base", default="http://127.0.0.1:18890")
    parser.add_argument("--concurrent-requests", type=int, default=4)
    args = parser.parse_args()
    if not 1 <= args.concurrent_requests <= 8:
        parser.error("Concurrent request count must stay between 1 and 8")
    if args.reader_base not in ("http://127.0.0.1:18890", "http://127.0.0.1:18891"):
        parser.error("Reader must use an isolated CI smoke endpoint, never the user's Reader")

    fixture = Fixture(("127.0.0.1", 0))
    fixture_base = f"http://127.0.0.1:{fixture.server_port}"
    thread = threading.Thread(target=fixture.serve_forever, daemon=True)
    thread.start()
    try:
        def create_account():
            account = urllib.request.build_opener(
                urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
            username = "localwebview" + secrets.token_hex(5)
            password = "Probe-" + secrets.token_hex(12)
            for is_login in (False, True):
                call(account, args.reader_base, "/reader3/login", {
                    "username": username, "password": password, "isLogin": is_login})
            return account

        alice = create_account()
        bob = create_account()
        if get_json(alice, args.reader_base, "/reader3/getUserInfo").get("data", {}).get("secure") is not True:
            raise RuntimeError("User isolation fixture requires READER_APP_SECURE=true")
        source = {
            "bookSourceUrl": fixture_base,
            "bookSourceName": "Local browser loopback fixture",
            "enabledCookieJar": True,
            "searchUrl": fixture_base + "/search, {\"webView\": true}",
            "ruleSearch": {"bookList": ".book", "name": ".name@text",
                           "author": ".author@text", "bookUrl": "a@href"},
            "ruleToc": {"chapterList": ".chapter"},
            "ruleContent": {"content": ".content@html"},
        }
        call(alice, args.reader_base, "/reader3/saveBookSource", source)
        call(bob, args.reader_base, "/reader3/saveBookSource", source)
        searches = []
        for account, key in ((alice, "alice-first"), (bob, "bob-first"),
                             (alice, "alice-second"), (alice, "alice-third")):
            value = call(account, args.reader_base, "/reader3/searchBook", {
                "key": key, "page": 1, "bookSourceUrl": fixture_base})
            books = value.get("data")
            if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "本地浏览器测试书":
                raise RuntimeError(f"Unexpected result for {key}: {books!r}")
            searches.append({"key": key, "count": len(books)})
        expected = ["", "", "session=alpha==", ""]
        if fixture.cookies != expected:
            raise RuntimeError(f"Cookie sequence {fixture.cookies!r}, expected {expected!r}")

        # These cases use the same script marker and POST fields as the local
        # original-JAR / archived-WebKit differential. Only an evaluated script
        # can change the raw target book name into the accepted result.
        script = "document.documentElement.outerHTML.replace('WebView脚本原始书','WebView差分书')"
        reference_cases = []
        for name, path, options in (
                ("script", "/search-script", {"webView": True, "webJs": script}),
                ("post", "/search-post", {"webView": True, "method": "POST",
                                          "body": "q=post", "headers": {"X-Fixture": "synthetic"},
                                          "webJs": script})):
            reference_source = dict(source)
            reference_source["searchUrl"] = fixture_base + path + ", " + json.dumps(options)
            call(alice, args.reader_base, "/reader3/saveBookSource", reference_source)
            value = call(alice, args.reader_base, "/reader3/searchBook", {
                "key": name, "page": 1, "bookSourceUrl": fixture_base})
            books = value.get("data")
            if (not isinstance(books, list) or len(books) != 1 or
                    books[0].get("name") != "WebView差分书" or value.get("errorMsg") != ""):
                raise RuntimeError(f"Bundled-browser {name} reference case failed")
            reference_cases.append({"case": name, "status": 200, "isSuccess": True,
                                    "errorMsg": "", "count": len(books),
                                    "targetRequest": fixture.requests[-1]})
        expected_reference_requests = [
            {"httpMethod": "GET", "body": None, "testHeader": None},
            {"httpMethod": "POST", "body": "q=post", "testHeader": "synthetic"},
        ]
        if fixture.requests[4:] != expected_reference_requests or fixture.cookies != expected + ["", ""]:
            raise RuntimeError("Bundled-browser reference method, body, header, or Cookie mismatch")

        async_cases, async_cleanup = exercise_async_reader(alice, args.reader_base, fixture, source)
        expected_before_burst = expected + ["", "", "", "asyncOnly=generated", ""]
        if fixture.cookies != expected_before_burst:
            raise RuntimeError("Generated async Cookie lifecycle or initial reference sequence changed")

        # A small simultaneous burst exercises the Reader queue and isolated
        # browser contexts under the image's 2 GiB / 256 PID budget. Each
        # account owns a separate HTTP cookie jar and saved source.
        burst_accounts = [create_account() for _ in range(args.concurrent_requests)]
        for account in burst_accounts:
            call(account, args.reader_base, "/reader3/saveBookSource", source)

        def burst_search(index):
            value = call(burst_accounts[index], args.reader_base, "/reader3/searchBook", {
                "key": f"burst-{index}", "page": 1, "bookSourceUrl": fixture_base})
            books = value.get("data")
            if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "本地浏览器测试书":
                raise RuntimeError(f"Concurrent search {index} returned unexpected books")
            return len(books)

        with concurrent.futures.ThreadPoolExecutor(max_workers=args.concurrent_requests) as pool:
            burst_counts = list(pool.map(burst_search, range(args.concurrent_requests)))
        if fixture.cookies != expected_before_burst + [""] * args.concurrent_requests:
            raise RuntimeError("Concurrent WebView requests leaked or changed a user Cookie")
        print(json.dumps({"searches": searches, "cookieSequence": expected,
                          "legacyReferenceCases": reference_cases,
                          "asyncReaderCases": async_cases, "asyncCookieCleanup": async_cleanup,
                          "concurrentRequests": args.concurrent_requests,
                          "concurrentBookCounts": burst_counts},
                         ensure_ascii=False))
    finally:
        fixture.shutdown()
        fixture.server_close()
        thread.join(timeout=5)


if __name__ == "__main__":
    main()
