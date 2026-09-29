"""Record the archived remote WebView's synthetic and public-page behavior.

Run only beside an isolated, pinned container. This is a reference-implementation
probe, not proof that the image matches a previously deployed remote service.
No page body, cookie value, credentials, or book-source configuration is logged.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import http.client
import json
import socket
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


MAX_BODY = 4 * 1024 * 1024


def fixture_body(marker):
    return (
        "<!doctype html><html><body>"
        f"<main id='result'>{marker}</main><span id='script'>pending</span>"
        "<script>document.getElementById('script').textContent='script-ok'</script>"
        "</body></html>"
    ).encode("utf-8")


class FixtureHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/regex-page":
            self.respond("<script>fetch('/regex-resource')</script>")
            return
        if self.path == "/regex-resource":
            self.respond("matched-resource-body")
            return
        self.respond("get-ok")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        if length > 1024:
            self.send_error(413)
            return
        self.rfile.read(length)
        self.respond("post-ok")

    def respond(self, marker):
        body = fixture_body(marker)
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Set-Cookie", "fixture=opaque; Path=/; HttpOnly; SameSite=Lax")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
        with self.server.lock:
            self.server.methods.append(self.command)
            self.server.paths.append(self.path)

    def log_message(self, *_args):
        pass


class Fixture(ThreadingHTTPServer):
    def __init__(self):
        super().__init__(("0.0.0.0", 0), FixtureHandler)
        self.lock = threading.Lock()
        self.methods = []
        self.paths = []


def wait_port(base, timeout=60):
    parsed = urllib.parse.urlparse(base)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection((parsed.hostname, parsed.port), timeout=1):
                return
        except OSError:
            time.sleep(0.5)
    raise TimeoutError("Archived remote WebView did not open its loopback port")


def render(base, url, method="GET", body=None, js_source=None, source_regex=None,
           timeout=45, expect_integer=False, expect_projection=False):
    payload = {
        "url": url,
        "html": None,
        "headers": {},
        "js_source": js_source,
        "proxy": None,
        "http_method": method,
        "body": body,
        "encode": None,
        "tag": None,
        "sourceRegex": source_regex,
    }
    request = urllib.request.Request(
        base.rstrip("/") + "/render.html",
        data=json.dumps(payload, ensure_ascii=False).encode("utf-8"),
        headers={"Content-Type": "application/json; charset=utf-8"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        raw = response.read(MAX_BODY + 1)
        if len(raw) > MAX_BODY:
            raise RuntimeError("Archived WebView response exceeded the 4 MiB probe limit")
        result = {
            "status": response.status,
            "contentType": response.headers.get("Content-Type", ""),
            "cookieNames": sorted({value.split("=", 1)[0]
                                   for value in response.headers.get_all("Set-Cookie", [])}),
            "bodySha256": hashlib.sha256(raw).hexdigest(),
            "bodyLength": len(raw),
            "hasGetMarker": b"get-ok" in raw,
            "hasPostMarker": b"post-ok" in raw,
            "hasScriptMarker": b"script-ok" in raw,
        }
        if expect_integer:
            number = raw.strip()
            if not number.isascii() or not number.isdigit() or len(number) > 3:
                raise RuntimeError("Archived WebView did not return a bounded integer")
            result["integerValue"] = int(number)
        if expect_projection:
            projection = json.loads(raw.decode("utf-8"))
            if (not isinstance(projection, list) or not 1 <= len(projection) <= 100 or
                    any(not isinstance(pair, list) or len(pair) != 2 or
                        not isinstance(pair[0], str) or not pair[0] or len(pair[0]) > 256 or
                        not isinstance(pair[1], str) or not pair[1].startswith(url) or
                        len(pair[1]) > 1024 for pair in projection)):
                raise RuntimeError("Archived WebView returned an invalid book projection")
            result["projectionCount"] = len(projection)
            result["projectionSha256"] = hashlib.sha256(json.dumps(
                projection, ensure_ascii=False, separators=(",", ":")).encode("utf-8")).hexdigest()
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--remote-base", default="http://127.0.0.1:18050")
    parser.add_argument("--fixture-host", default="host.docker.internal")
    parser.add_argument("--public-url")
    args = parser.parse_args()
    parsed = urllib.parse.urlparse(args.remote_base)
    if parsed.scheme != "http" or parsed.hostname not in ("localhost", "127.0.0.1"):
        parser.error("Remote WebView must be exposed only on loopback")
    if args.public_url:
        public = urllib.parse.urlparse(args.public_url)
        if public.scheme != "https" or public.username or public.password or public.query:
            parser.error("Public test URL must be an HTTPS page without credentials or query")

    fixture = Fixture()
    thread = threading.Thread(target=fixture.serve_forever, daemon=True)
    thread.start()
    try:
        wait_port(args.remote_base)
        # A TCP listener can appear before the old service has initialized its
        # browser. Retry only this first request on a reset; later failures
        # remain visible as actual render failures.
        time.sleep(3)
        fixture_url = f"http://{args.fixture_host}:{fixture.server_port}/page"
        try:
            for attempt in range(1, 4):
                try:
                    get_result = render(args.remote_base, fixture_url)
                    break
                except (ConnectionResetError, http.client.RemoteDisconnected):
                    print(json.dumps({"stage": "synthetic-get", "resetAttempt": attempt}),
                          flush=True)
                    if attempt == 3:
                        raise
                    time.sleep(2)
        except Exception as exc:
            with fixture.lock:
                methods = list(fixture.methods)
            print(json.dumps({"stage": "synthetic-get", "errorType": type(exc).__name__,
                              "fixtureMethods": methods}, sort_keys=True), flush=True)
            raise
        post_result = render(args.remote_base, fixture_url, "POST", "probe=1")
        with fixture.lock:
            methods = list(fixture.methods)
        print(json.dumps({"synthetic": {"get": get_result, "post": post_result,
                                        "fixtureMethods": methods}},
                         ensure_ascii=False, sort_keys=True), flush=True)
        if get_result["status"] != 200 or not get_result["hasGetMarker"]:
            raise RuntimeError("Archived WebView did not return the synthetic GET marker")
        if post_result["status"] != 200 or not post_result["hasPostMarker"]:
            raise RuntimeError("Archived WebView did not return the synthetic POST marker")
        if args.public_url:
            try:
                public_result = render(args.remote_base, args.public_url, timeout=45)
            except (OSError, RuntimeError, urllib.error.URLError) as exc:
                public_result = {"errorType": type(exc).__name__}
            print(json.dumps({"public": public_result}, ensure_ascii=False,
                             sort_keys=True), flush=True)
            try:
                public_count = render(
                    args.remote_base, args.public_url,
                    js_source="document.querySelectorAll('.booklist_a .list_a').length",
                    timeout=45, expect_integer=True,
                )
                public_count_result = {
                    "url": args.public_url,
                    "status": public_count["status"],
                    "count": public_count["integerValue"],
                    "observedAt": datetime.now(timezone.utc).isoformat(),
                }
            except (OSError, RuntimeError, urllib.error.URLError) as exc:
                public_count_result = {"url": args.public_url, "errorType": type(exc).__name__}
            print(json.dumps({"publicDomBookListCount": public_count_result},
                             ensure_ascii=False, sort_keys=True), flush=True)
            try:
                projection = render(
                    args.remote_base, args.public_url,
                    js_source=(
                        "Array.from(document.querySelectorAll('.booklist_a .list_a'), row => {"
                        " const main = row.querySelector('.main');"
                        " const name = main && main.querySelector('strong');"
                        " const link = main && main.querySelector('a');"
                        " return [name ? name.textContent.trim() : '', link ? link.href : ''];"
                        " })"
                    ),
                    timeout=45, expect_projection=True,
                )
                projection_result = {
                    "url": args.public_url,
                    "status": projection["status"],
                    "count": projection["projectionCount"],
                    "projectionSha256": projection["projectionSha256"],
                    "observedAt": datetime.now(timezone.utc).isoformat(),
                }
            except (OSError, RuntimeError, ValueError, urllib.error.URLError) as exc:
                projection_result = {"url": args.public_url, "errorType": type(exc).__name__}
            print(json.dumps({"publicDomBookProjection": projection_result},
                             ensure_ascii=False, sort_keys=True), flush=True)
        try:
            js_result = render(args.remote_base, fixture_url,
                               # This archived implementation resolves the
                               # returned value of page.evaluate(js_source).
                               # A side-effect-only expression returns undefined
                               # and takes its 30-attempt retry branch instead.
                               js_source="(() => { document.body.setAttribute('data-probe','yes'); "
                                         "return document.body.getAttribute('data-probe') === 'yes' "
                                         "? 'script-result-ok' : 'script-result-failed'; })()",
                               timeout=20)
        except (OSError, RuntimeError, urllib.error.URLError) as exc:
            with fixture.lock:
                methods = list(fixture.methods)
            print(json.dumps({"syntheticJs": {"errorType": type(exc).__name__,
                                               "fixtureMethods": methods}},
                             sort_keys=True), flush=True)
            raise RuntimeError("Archived WebView script render did not complete") from exc
        print(json.dumps({"syntheticJs": js_result}, sort_keys=True), flush=True)
        if js_result["status"] != 200 or js_result["bodySha256"] != hashlib.sha256(
                b"script-result-ok").hexdigest():
            raise RuntimeError("Archived WebView returned an unexpected script result")
        # page.evaluate returns structured JavaScript values. The archived
        # service JSON-serializes non-string results before sending its body.
        for label, expression, expected in (
                ("object", "({answer: 42, ready: true})", b'{"answer":42,"ready":true}'),
                ("array", "['alpha', 7]", b'["alpha",7]'),
                ("number", "42", b"42"),
        ):
            result = render(args.remote_base, fixture_url, js_source=expression, timeout=20)
            print(json.dumps({"syntheticJsType": label, "result": result},
                             sort_keys=True), flush=True)
            if result["status"] != 200 or result["bodySha256"] != hashlib.sha256(expected).hexdigest():
                raise RuntimeError(f"Archived WebView returned unexpected {label} script output")
        try:
            resource = render(
                args.remote_base,
                f"http://{args.fixture_host}:{fixture.server_port}/regex-page",
                source_regex=r"/regex-resource$",
                timeout=20,
            )
        except TimeoutError:
            with fixture.lock:
                paths = list(fixture.paths)
            # This pinned reference has a known bug: its response listener
            # reads response.request().url.match(...) instead of url().match(...).
            # Confirm the resource was fetched before recording the hang.
            if "/regex-page" not in paths or "/regex-resource" not in paths:
                raise RuntimeError("sourceRegex timed out before the fixture resource was requested")
            print(json.dumps({"syntheticSourceRegex": {"knownDefect": "response-listener-hangs",
                                                        "fixturePaths": paths}},
                             sort_keys=True), flush=True)
        else:
            with fixture.lock:
                methods = list(fixture.methods)
                paths = list(fixture.paths)
            print(json.dumps({"syntheticSourceRegex": resource, "fixtureMethods": methods,
                              "fixturePaths": paths},
                             sort_keys=True), flush=True)
            if resource["status"] != 200 or resource["bodySha256"] != hashlib.sha256(
                    fixture_body("matched-resource-body")).hexdigest():
                raise RuntimeError("Archived WebView did not return the matched resource body")
    finally:
        fixture.shutdown()
        fixture.server_close()
        thread.join(timeout=5)


if __name__ == "__main__":
    main()
