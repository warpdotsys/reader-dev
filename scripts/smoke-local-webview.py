"""Exercise an opt-in local WebView Reader against a loopback-only book source.

The target Reader must use an isolated work directory; this script registers one
synthetic source and account there. It refuses non-loopback Reader addresses.
"""

import argparse
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
        self.lock = threading.Lock()


class FixtureHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if not self.path.startswith("/search"):
            self.send_error(404)
            return
        cookie = self.headers.get("Cookie", "")
        with self.server.lock:
            self.server.cookies.append(cookie)
            number = len(self.server.cookies)
        html = ("<html><div class='book'><a href='/book'>"
                "<span class='name'>本地浏览器测试书</span></a>"
                "<span class='author'>测试作者</span></div></html>")
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reader-base", default="http://127.0.0.1:18890")
    args = parser.parse_args()
    parsed = urllib.parse.urlparse(args.reader_base)
    if parsed.scheme != "http" or parsed.hostname not in ("127.0.0.1", "localhost"):
        parser.error("Reader must be a loopback HTTP endpoint in an isolated work directory")

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
        print(json.dumps({"searches": searches, "cookieSequence": fixture.cookies},
                         ensure_ascii=False))
    finally:
        fixture.shutdown()
        fixture.server_close()
        thread.join(timeout=5)


if __name__ == "__main__":
    main()
