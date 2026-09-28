"""Record the archived remote WebView's synthetic and public-page behavior.

Run only beside an isolated, pinned container. This is a reference-implementation
probe, not proof that the image matches a previously deployed remote service.
No page body, cookie value, credentials, or book-source configuration is logged.
"""

import argparse
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


class FixtureHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.respond("get-ok")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        if length > 1024:
            self.send_error(413)
            return
        self.rfile.read(length)
        self.respond("post-ok")

    def respond(self, marker):
        body = (
            "<!doctype html><html><body>"
            f"<main id='result'>{marker}</main><span id='script'>pending</span>"
            "<script>document.getElementById('script').textContent='script-ok'</script>"
            "</body></html>"
        ).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Set-Cookie", "fixture=opaque; Path=/; HttpOnly; SameSite=Lax")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
        with self.server.lock:
            self.server.methods.append(self.command)

    def log_message(self, *_args):
        pass


class Fixture(ThreadingHTTPServer):
    def __init__(self):
        super().__init__(("0.0.0.0", 0), FixtureHandler)
        self.lock = threading.Lock()
        self.methods = []


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


def render(base, url, method="GET", body=None, js_source=None, timeout=45):
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
        "sourceRegex": None,
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
        return {
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
            js_result = render(args.remote_base, fixture_url,
                               js_source="document.body.setAttribute('data-probe','yes')",
                               timeout=20)
        except (OSError, RuntimeError, urllib.error.URLError) as exc:
            with fixture.lock:
                methods = list(fixture.methods)
            print(json.dumps({"syntheticJs": {"errorType": type(exc).__name__,
                                               "fixtureMethods": methods}},
                             sort_keys=True), flush=True)
            raise RuntimeError("Archived WebView script render did not complete") from exc
        print(json.dumps({"syntheticJs": js_result}, sort_keys=True), flush=True)
    finally:
        fixture.shutdown()
        fixture.server_close()
        thread.join(timeout=5)


if __name__ == "__main__":
    main()
