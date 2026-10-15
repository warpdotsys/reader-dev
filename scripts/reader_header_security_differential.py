"""Generated-only HTTP header characterization; not real-account or HTTPS proof.

The caller must independently verify a private loopback-only network namespace
BEFORE constructing this fixture. Never accepts URLs, tokens or headers from a
file/user. Historic engines may be insecure: observation is not acceptance.
"""
import copy
import hashlib
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

CASES = ("same-get", "cross-get", "cross-post", "resources")
AUTHORIZATION = "Bearer GENERATED_NOT_A_REAL_CREDENTIAL"
TRACE = "GENERATED_HEADER_TRACE_ONLY"
USER_AGENT = "ReaderGeneratedHeaderProbe/1.0"
POST_BODY = "q=generated-header-only"


def header_values(row, name):
    return [value for key, value in row["headers"] if key.lower() == name.lower()]


class HeaderFixture:
    def __init__(self, require_isolation, bounded_headers):
        # No binds, threads, input reads or process starts before this guard.
        require_isolation()
        self.bounded_headers = bounded_headers
        self.lock = threading.RLock()
        self.rows = []
        self.rejections = 0
        self.servers = []
        self.threads = []
        try:
            for host, actor in (("127.0.0.1", "primary"), ("127.0.0.2", "secondary")):
                server = ThreadingHTTPServer((host, 0), HeaderHandler)
                server.fixture = self
                server.actor = actor
                self.servers.append(server)
            self.primary = "http://127.0.0.1:" + str(self.servers[0].server_port)
            self.secondary = "http://127.0.0.2:" + str(self.servers[1].server_port)
            for server in self.servers:
                thread = threading.Thread(target=server.serve_forever, daemon=True)
                thread.start()
                self.threads.append(thread)
        except BaseException:
            self.close()
            raise

    def close(self):
        for index, server in enumerate(self.servers):
            if index < len(self.threads):
                server.shutdown()
            server.server_close()
        for thread in self.threads:
            thread.join(timeout=5)
            if thread.is_alive():
                raise RuntimeError("Generated header fixture did not stop")

    def reset(self):
        with self.lock:
            self.rows.clear()
            self.rejections = 0

    def snapshot(self):
        with self.lock:
            return {"targetRequests": copy.deepcopy(self.rows), "rejections": self.rejections}

    def record(self, case, actor, role, method, headers, body):
        if case not in CASES or actor not in ("primary", "secondary"):
            raise ValueError("Unknown generated header case")
        pairs = self.bounded_headers(headers)
        if len(body) > 65536:
            raise ValueError("Generated body exceeds finite budget")
        with self.lock:
            if len(self.rows) >= 128 or sum(row["case"] == case for row in self.rows) >= 32:
                raise ValueError("Generated request count exceeds finite budget")
            self.rows.append({"case": case, "actor": actor, "role": role,
                "method": method, "headers": pairs, "bodyByteCount": len(body),
                "bodySha256": hashlib.sha256(body).hexdigest()})

    def html(self, case, resources=False):
        scripts = ("<script src='" + self.primary + "/probe/resources/same.js'></script>"
                   "<script src='" + self.secondary + "/probe/resources/cross.js'></script>"
                   if resources else "")
        # Absolute book URL deliberately stays identical across all three sides.
        return ("<!doctype html><meta charset='UTF-8'>" + scripts +
            "<div class='book'><a href='" + self.primary + "/book/" + case + "'>书</a>"
            "<span class='name'>认证头原始书-" + case + "</span>"
            "<span class='author'>生成作者</span></div>").encode("utf-8")

    def definition(self, case):
        if case not in CASES:
            raise ValueError("Unknown generated case")
        script = ("document.documentElement.outerHTML.replace('认证头原始书-" + case +
                  "','认证头差分书-" + case + "')")
        if case == "resources":
            script = ("(() => { if(document.documentElement.dataset.generatedSameResource !== 'executed' || "
                "document.documentElement.dataset.generatedCrossResource !== 'executed') "
                "throw new Error('GENERATED_RESOURCE_NOT_EXECUTED'); return " + script + "; })()")
        options = {"webView": True, "webJs": script, "headers": {
            "Authorization": AUTHORIZATION, "X-Generated-Trace": TRACE, "User-Agent": USER_AGENT}}
        if case == "cross-post":
            options.update({"method": "POST", "body": POST_BODY})
        source_url = self.primary + "/source/" + case
        return {"bookSourceUrl": source_url, "bookSourceName": "Generated header " + case,
            "enabledCookieJar": True, "searchUrl": self.primary + "/probe/" + case +
                ("/page" if case == "resources" else "/start") + ", " + json.dumps(options, ensure_ascii=False),
            "ruleSearch": {"bookList": ".book", "name": ".name@text", "author": ".author@text", "bookUrl": "a@href"},
            "ruleToc": {"chapterList": ".chapter"}, "ruleContent": {"content": ".content@html"}}

    def exercise(self, opener, base, phase, require_success, request):
        self.reset()
        results = []
        for case in CASES:
            source = self.definition(case)
            phase.call("headerSourceSave", require_success, opener, base, "/reader3/saveBookSource", source)
            saved = phase.call("headerSourceRead", require_success, opener, base, "/reader3/getBookSource",
                {"bookSourceUrl": source["bookSourceUrl"]})["data"]
            rules = saved.get("ruleSearch")
            retained = (all(saved.get(key) == source[key] for key in ("bookSourceUrl", "searchUrl")) and
                isinstance(rules, dict) and all(rules.get(key) == value for key, value in source["ruleSearch"].items()))
            if not retained:
                raise RuntimeError("Generated header source did not round-trip")
            # Preserve the actual failure JSON too; do not fabricate one successful book.
            status, value = phase.call("headerSearch", request, opener, base, "/reader3/searchBook",
                {"key": "generated", "page": 1, "bookSourceUrl": source["bookSourceUrl"]})
            results.append({"case": case, "status": status, "returnData": value,
                "sourceDefinitionRoundtrip": retained})
        return {"generatedOnly": True, "realCredentialsImported": False, "httpsTested": False,
            "primary": self.primary, "secondary": self.secondary, "cases": results, **self.snapshot()}


class HeaderHandler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def send(self, status, body=b"", content_type="text/html; charset=UTF-8", location=None):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        if location is not None:
            self.send_header("Location", location)
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self.handle_probe()

    def do_POST(self):
        self.handle_probe()

    def handle_probe(self):
        if self.path == "/favicon.ico" and self.command == "GET":
            self.send(204)
            return
        parts = self.path.split("/")
        if len(parts) != 4 or parts[1] != "probe" or parts[2] not in CASES:
            self.send(404)
            return
        case, role = parts[2:]
        fixture, actor = self.server.fixture, self.server.actor
        allowed = ((role == "start" and actor == "primary" and case != "resources") or
                   (role == "end" and case != "resources" and actor == ("primary" if case == "same-get" else "secondary")) or
                   (case == "resources" and ((role == "page" and actor == "primary") or
                       (role == "same.js" and actor == "primary") or (role == "cross.js" and actor == "secondary"))))
        if not allowed:
            self.send(404)
            return
        try:
            # Set timeout before reading: a valid length alone cannot bound a stalled peer.
            self.connection.settimeout(5)
            lengths = self.headers.get_all("Content-Length", [])
            if self.headers.get("Transfer-Encoding") or len(lengths) > 1:
                raise ValueError("Unsupported generated request framing")
            length = int(lengths[0]) if lengths else 0
            if length < 0 or length > 65536:
                raise ValueError("Generated body length out of budget")
            body = self.rfile.read(length)
            if len(body) != length:
                raise ValueError("Incomplete generated body")
            fixture.record(case, actor, role, self.command, self.headers, body)
        except (ValueError, OSError):
            with fixture.lock:
                fixture.rejections += 1
            self.send(431)
            return
        if role.endswith(".js"):
            flag = "Same" if actor == "primary" else "Cross"
            self.send(200, ("document.documentElement.dataset.generated" + flag + "Resource='executed';").encode(),
                "text/javascript; charset=UTF-8")
        elif role == "start":
            target = fixture.primary if case == "same-get" else fixture.secondary
            # Original Node POST reads this body without following Location. That
            # is a recorded compatibility difference, not a fabricated redirect.
            self.send(303, fixture.html(case), location=target + "/probe/" + case + "/end")
        else:
            self.send(200, fixture.html(case, case == "resources"))


def characterize(observation):
    """Validate finite executed facts, then report security WITHOUT granting parity."""
    if not isinstance(observation, dict) or observation.get("generatedOnly") is not True or \
            observation.get("realCredentialsImported") is not False or observation.get("httpsTested") is not False:
        raise RuntimeError("Missing generated header-security observation")
    results, rows = observation.get("cases"), observation.get("targetRequests")
    if not isinstance(results, list) or [item.get("case") for item in results] != list(CASES) or \
            not isinstance(rows, list) or not 7 <= len(rows) <= 128 or observation.get("rejections") != 0:
        raise RuntimeError("Incomplete generated header-security observation")
    for row in rows:
        if not isinstance(row, dict) or row.get("case") not in CASES or row.get("actor") not in ("primary", "secondary") or \
                row.get("role") not in ("start", "end", "page", "same.js", "cross.js") or row.get("method") not in ("GET", "POST"):
            raise RuntimeError("Unknown observed generated request")
        pairs, count, digest = row.get("headers"), row.get("bodyByteCount"), row.get("bodySha256")
        if not isinstance(pairs, list) or not 1 <= len(pairs) <= 64 or any(not isinstance(pair, list) or len(pair) != 2 or
                any(type(item) is not str for item in pair) for pair in pairs) or \
                sum(len(item.encode("utf-8")) for pair in pairs for item in pair) > 8192 or \
                type(count) is not int or not 0 <= count <= 65536 or type(digest) is not str or len(digest) != 64 or \
                any(character not in "0123456789abcdef" for character in digest):
            raise RuntimeError("Generated request observation violates finite field/body budget")
        if row["method"] == "GET" and (count != 0 or digest != hashlib.sha256(b"").hexdigest()):
            raise RuntimeError("Generated GET unexpectedly contains a body")
    diagnostics = []
    for case, result in zip(CASES, results):
        value = result.get("returnData")
        books = value.get("data") if isinstance(value, dict) else None
        if type(result.get("status")) is not int or result["status"] != 200 or \
                result.get("sourceDefinitionRoundtrip") is not True or not isinstance(value, dict) or value.get("isSuccess") is not True or \
                not isinstance(books, list) or len(books) != 1 or books[0].get("name") != "认证头差分书-" + case:
            raise RuntimeError("Generated header-security API/script outcome incomplete")
        actual = [row for row in rows if row.get("case") == case]
        expected_first = "page" if case == "resources" else "start"
        first = [row for row in actual if row.get("actor") == "primary" and row.get("role") == expected_first]
        if len(first) != 1 or first[0].get("method") != ("POST" if case == "cross-post" else "GET"):
            raise RuntimeError("Missing actual generated navigation")
        body = POST_BODY.encode() if case == "cross-post" else b""
        if first[0].get("bodyByteCount") != len(body) or first[0].get("bodySha256") != hashlib.sha256(body).hexdigest() or \
                header_values(first[0], "Authorization") != [AUTHORIZATION] or \
                header_values(first[0], "X-Generated-Trace") != [TRACE] or \
                header_values(first[0], "User-Agent") != [USER_AGENT]:
            raise RuntimeError("Generated initial headers/body not actually observed")
        if case == "resources":
            if len(actual) != 3 or {(row.get("actor"), row.get("role"), row.get("method")) for row in actual} != \
                    {("primary", "page", "GET"), ("primary", "same.js", "GET"), ("secondary", "cross.js", "GET")}:
                raise RuntimeError("Generated script subresources not both observed")
        elif case != "cross-post":
            if len(actual) != 2 or (actual[-1].get("actor"), actual[-1].get("role"), actual[-1].get("method")) != \
                    ("primary" if case == "same-get" else "secondary", "end", "GET"):
                raise RuntimeError("Generated GET redirect not actually followed")
        elif len(actual) not in (1, 2) or (len(actual) == 2 and
                (actual[-1].get("actor"), actual[-1].get("role"), actual[-1].get("method")) != ("secondary", "end", "GET")):
            raise RuntimeError("Unknown generated POST redirect behavior")
        cross = [row for row in actual if row["actor"] == "secondary"]
        same = [row for row in actual if row["actor"] == "primary"]
        diagnostics.append({"case": case, "requests": len(actual), "redirectFollowed":
            any(row["role"] == "end" for row in actual), "sameOriginAuthorizationRetained":
            all(header_values(row, "Authorization") == [AUTHORIZATION] for row in same),
            "crossOriginRequestsObserved": len(cross), "crossOriginAuthorizationAbsent":
            all(not header_values(row, "Authorization") for row in cross),
            "crossOriginAuthorizationSent": any(bool(header_values(row, "Authorization")) for row in cross)})
    return {"observationComplete": True, "cases": diagnostics, "strictParityAccepted": False,
        "crossOriginAuthorizationAbsentForAllObservedRequests": all(not item["crossOriginAuthorizationSent"] for item in diagnostics),
        "realAuthenticationAccepted": False, "httpsAccepted": False}
