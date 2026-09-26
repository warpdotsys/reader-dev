#!/usr/bin/env python3
"""Black-box differential for cached network-book chapters over SSE.

The book source listens only on loopback; both JARs use isolated accounts and
storage trees. No production service or user data is accessed.
"""

import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
import json
from pathlib import Path
import secrets
import subprocess
from threading import Thread
import time
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
SSE_SPEC = importlib.util.spec_from_file_location(
    "sse_base", Path(__file__).with_name("compare-sse-pagination.py"))
SSE = importlib.util.module_from_spec(SSE_SPEC)
SSE_SPEC.loader.exec_module(SSE)
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/cache-book-sse-diff-latest.json"
BOOK_PAGES = {
    "/book": "<html><h1>Cache Fixture</h1><span class='author'>Fixture Author</span>"
             "<a class='toc' href='/toc'>TOC</a></html>",
    "/toc": "<html><div class='chapter'><a href='/chapter/1'>Chapter One</a></div>"
            "<div class='chapter'><a href='/chapter/2'>Chapter Two</a></div></html>",
    "/chapter/1": "<html><div class='content'><p>Alpha cache body.</p></div></html>",
    "/chapter/2": "<html><div class='content'><p>Beta cache body.</p></div></html>",
}


class BookHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = BOOK_PAGES.get(urllib.parse.urlsplit(self.path).path)
        if body is None:
            self.send_error(404)
            return
        data = body.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, _format, *_args):
        pass


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source(base):
    return {
        "bookSourceUrl": base,
        "bookSourceName": "Cache fixture",
        "searchUrl": base + "/search?key={{key}}",
        "ruleSearch": {"bookList": ".book", "name": ".name@text",
                       "author": ".author@text", "bookUrl": "a@href"},
        "ruleBookInfo": {"name": "h1@text", "author": ".author@text",
                         "tocUrl": ".toc@href"},
        "ruleToc": {"chapterList": ".chapter", "chapterName": "a@text",
                    "chapterUrl": "a@href"},
        "ruleContent": {"content": ".content@html"},
    }


def read_cache_sse(opener, base, name, path):
    with opener.open(base + path, timeout=45) as response:
        raw = response.read(2_000_001)
        status = response.status
        content_type = response.headers.get("Content-Type", "")
        cache_control = response.headers.get("Cache-Control", "")
    if status != 200 or not content_type.startswith("text/event-stream") or len(raw) > 2_000_000:
        raise AssertionError(f"{name}: invalid SSE status, type or size")
    text = raw.decode("utf-8", errors="strict")
    if not text.endswith("\n\n"):
        raise AssertionError(f"{name}: incomplete final SSE frame")
    frames = []
    for block in text[:-2].split("\n\n"):
        lines = block.split("\n")
        event = next((line[7:] for line in lines if line.startswith("event: ")), "message")
        data_lines = [line[6:] for line in lines if line.startswith("data: ")]
        if not data_lines:
            raise AssertionError(f"{name}: SSE frame without data")
        frames.append({"event": event, "data": json.loads("\n".join(data_lines))})
    return {"probe": name, "status": status, "contentType": content_type,
            "cacheControl": cache_control, "frames": frames}


def run_one(jar, workdir, username, other_user, password, fixture_base):
    workdir.mkdir(parents=True)
    port = SSE.free_port()
    base = f"http://127.0.0.1:{port}"
    source_json = source(fixture_base)
    book_url = fixture_base + "/book"
    encoded_url = urllib.parse.quote(book_url, safe="")
    endpoint = "/reader3/cacheBookSSE"
    command = [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
               f"--reader.server.port={port}", "--reader.app.secure=true",
               "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"]
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        import http.cookiejar
        def client():
            return urllib.request.build_opener(
                urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        anonymous, owner, other = client(), client(), client()
        probes = []

        def add(name, session, params):
            path = endpoint + "?" + urllib.parse.urlencode(params)
            row = read_cache_sse(session, base, name, path)
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
            add("anonymous", anonymous, {"url": book_url})
            for session, name in ((owner, username), (other, other_user)):
                for is_login in (False, True):
                    SSE.require_success(SSE.get_json(session, base, "/reader3/login",
                                                     {"username": name, "password": password,
                                                      "isLogin": is_login}), "fixture login")
            add("missing-url", owner, {})
            add("unknown-book", owner, {"url": book_url})
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBookSource",
                                             source_json), "save fixture source")
            book_info = SSE.require_success(SSE.get_json(
                owner, base, "/reader3/getBookInfo",
                {"url": book_url, "bookSource": source_json}), "book info")
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBook",
                                             book_info), "save fixture book")
            # Removing the source after shelving checks the missing-source branch.
            SSE.require_success(SSE.get_json(owner, base, "/reader3/deleteBookSource",
                                             source_json), "remove fixture source")
            add("no-source", owner, {"url": book_url})
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBookSource",
                                             source_json), "restore fixture source")
            local_url = f"storage/data/{username}/reading/local.txt"
            local_file = workdir / local_url
            local_file.parent.mkdir(parents=True, exist_ok=True)
            local_file.write_text("Local fixture body.\n", encoding="utf-8")
            SSE.require_success(SSE.get_json(owner, base, "/reader3/saveBook",
                                             {"bookUrl": local_url, "originName": local_url,
                                              "origin": "loc_book", "name": "Local Fixture",
                                              "author": "", "type": 0}), "save local fixture")
            add("local-book-rejected", owner, {"url": local_url})
            chapter_list = SSE.require_success(SSE.get_json(
                owner, base, "/reader3/getChapterList?url=" + encoded_url), "chapter list")
            if len(chapter_list) != 2:
                raise AssertionError("Expected two fixture chapters")
            add("other-user-unknown", other, {"url": book_url})
            add("first-cache", owner, {"url": book_url, "concurrentCount": 1})
            probes.append(cache_snapshot(workdir, username, "first-storage"))
            add("repeat-cache", owner, {"url": book_url, "concurrentCount": 1})
            probes.append(cache_snapshot(workdir, username, "repeat-storage"))
            add("force-refresh", owner, {"url": book_url, "concurrentCount": 1,
                                         "refresh": 1})
            probes.append(cache_snapshot(workdir, username, "refresh-storage"))
            return probes
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def cache_snapshot(workdir, username, name):
    root = workdir / "storage/data" / username
    records = {}
    for index in (0, 1):
        matches = list(root.rglob(f"{index}.txt"))
        records[str(index)] = [{"text": path.read_text(encoding="utf-8"),
                                "sha256": sha256(path)} for path in matches]
    return {"probe": name, "chapters": records}


def assert_contract(probes):
    by_name = {row["probe"]: row for row in probes}
    errors = {"anonymous": ("请登录后使用", "NEED_LOGIN"),
              "missing-url": ("请输入书籍链接", None),
              "unknown-book": ("请先加入书架", None),
              "no-source": ("未配置书源", None),
              "local-book-rejected": ("本地书籍无需缓存", None),
              "other-user-unknown": ("请先加入书架", None)}
    for name, (message, data) in errors.items():
        row = by_name[name]
        frames = row["frames"]
        if (row["cacheControl"] != "no-cache" or len(frames) != 1
                or frames[0]["event"] != "error"
                or frames[0]["data"].get("errorMsg") != message
                or frames[0]["data"].get("isSuccess") is not False
                or (data is not None and frames[0]["data"].get("data") != data)):
            raise AssertionError(f"Unexpected cache SSE error: {name}: {row}")
    expected = {"first-cache": (2, 2), "repeat-cache": (2, 0),
                "force-refresh": (2, 2)}
    for name, (cached, success) in expected.items():
        row = by_name[name]
        frames = row["frames"]
        last = frames[-1]
        if (row["cacheControl"] != "no-cache" or len(frames) < 2
                or last["event"] != "end"
                or last["data"] != {"cachedCount": cached, "successCount": success,
                                    "failedCount": 0}):
            raise AssertionError(f"Unexpected cache SSE terminal frame: {name}: {row}")
    for name in ("first-storage", "repeat-storage", "refresh-storage"):
        chapters = by_name[name]["chapters"]
        if (len(chapters["0"]) != 1 or len(chapters["1"]) != 1
                or "Alpha cache body." not in chapters["0"][0]["text"]
                or "Beta cache body." not in chapters["1"][0]["text"]):
            raise AssertionError(f"Unexpected cached chapter files: {name}: {chapters}")
    if not (by_name["first-storage"]["chapters"] ==
            by_name["repeat-storage"]["chapters"] ==
            by_name["refresh-storage"]["chapters"]):
        raise AssertionError("Cache files changed unexpectedly between repeat and refresh")


def main():
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    run_root = ROOT / ".tools" / ("cache-book-sse-diff-" + secrets.token_hex(8))
    run_root.mkdir(parents=True)
    server = ThreadingHTTPServer(("127.0.0.1", 0), BookHandler)
    fixture_base = f"http://127.0.0.1:{server.server_port}"
    server_thread = Thread(target=server.serve_forever, daemon=True)
    server_thread.start()
    try:
        username = "cacheprobe" + secrets.token_hex(5)
        other_user = "cacheother" + secrets.token_hex(5)
        password = "Probe-" + secrets.token_hex(18)
        original = run_one(ORIGINAL, run_root / "original", username, other_user,
                           password, fixture_base)
        restored = run_one(RESTORED, run_root / "restored", username, other_user,
                           password, fixture_base)
    finally:
        server.shutdown()
        server.server_close()
        server_thread.join(timeout=5)
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("Original and restored cache probe sequences differ")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "original": left, "restored": right}
                   for left, right in zip(original, restored)]
    report = {"originalJarSha256": sha256(ORIGINAL),
              "restoredJarSha256": sha256(RESTORED), "comparisons": comparisons}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'DIFFERENT'}")
    print(f"Report: {REPORT}")
    print(f"Isolated data: {run_root}")
    assert_contract(original)
    assert_contract(restored)
    if not all(row["equal"] for row in comparisons):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
