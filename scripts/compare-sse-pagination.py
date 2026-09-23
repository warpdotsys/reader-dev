#!/usr/bin/env python3
"""Differential SSE lifecycle and two-source pagination with a loopback fixture."""

import argparse
import hashlib
import http.cookiejar
import json
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid


PROJECT = Path(__file__).resolve().parents[1]
MAX_SSE_BYTES = 2_000_000


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def get_json(opener, base, path, body=None, timeout=20):
    payload = json.dumps(body, ensure_ascii=False).encode() if body is not None else None
    req = urllib.request.Request(
        base + path, data=payload, method="POST" if payload else "GET",
        headers={"Content-Type": "application/json"} if payload else {},
    )
    with opener.open(req, timeout=timeout) as response:
        return response.status, json.loads(response.read().decode("utf-8"))


def require_success(response, step):
    status, value = response
    if status != 200 or value.get("isSuccess") is not True:
        raise AssertionError(f"{step} failed: HTTP {status}, {value.get('errorMsg')}")
    return value.get("data")


def read_sse(opener, base, name, endpoint, params, terminal, timeout=35):
    req = urllib.request.Request(base + endpoint + "?" + urllib.parse.urlencode(params))
    with opener.open(req, timeout=timeout) as response:
        raw = response.read(MAX_SSE_BYTES + 1)
        status = response.status
        content_type = response.headers.get("Content-Type", "")
        cache_control = response.headers.get("Cache-Control", "")
    if status != 200 or not content_type.startswith("text/event-stream") or len(raw) > MAX_SSE_BYTES:
        raise AssertionError(f"{name}: invalid HTTP status, content type, or SSE size")
    frames = parse_sse_frames(raw, name, terminal)
    books = [book for event, data in frames if event == "message"
             for book in data.get("data", [])]
    digest = hashlib.sha256(json.dumps(
        [(book.get("name"), book.get("author"), book.get("bookUrl"), book.get("origin"))
         for book in books], ensure_ascii=False, separators=(",", ":")
    ).encode()).hexdigest()
    result = {
        "probe": name, "status": status, "contentType": content_type,
        "cacheControl": cache_control, "frameCount": len(frames),
        "dataCount": len(books), "terminalEvent": frames[-1][0],
    }
    if terminal == "end":
        result.update(lastIndex=frames[-1][1].get("lastIndex"),
                      isEnd=frames[-1][1].get("isEnd"), resultSha256=digest)
    else:
        last = frames[-1][1]
        result.update(isSuccess=last.get("isSuccess"), errorMsg=last.get("errorMsg"),
                      data=last.get("data"))
    return result


def parse_sse_frames(raw, name, terminal):
    decoded = raw.decode("utf-8", errors="strict")
    if not decoded.endswith("\n\n"):
        raise AssertionError(f"{name}: incomplete final SSE frame")
    frames = []
    for block in decoded[:-2].split("\n\n"):
        lines = block.split("\n")
        event = next((line[7:] for line in lines if line.startswith("event: ")), "message")
        data_lines = [line[6:] for line in lines if line.startswith("data: ")]
        if not data_lines:
            raise AssertionError(f"{name}: SSE frame without data")
        frames.append((event, json.loads("\n".join(data_lines))))
    if not frames or frames[-1][0] != terminal:
        raise AssertionError(f"{name}: EOF without expected {terminal} terminal event")
    if terminal == "end" and any(event == "error" for event, _ in frames):
        raise AssertionError(f"{name}: unexpected error frame")
    if terminal == "error" and len(frames) != 1:
        raise AssertionError(f"{name}: error response contained unexpected frames")
    return frames


def assert_incomplete_eof_rejected():
    for raw in (b'data: {"data": []}\n\n',
                b'data: {"data": []}\n\nevent: end\ndata: {"isEnd": true}'):
        try:
            parse_sse_frames(raw, "synthetic-incomplete", "end")
        except AssertionError:
            continue
        raise AssertionError("SSE parser accepted EOF without a complete terminal frame")


def source(fixture_base, suffix):
    delay = "&delayMs=800" if suffix == "source-three" else "&delayMs=500"
    return {
        "bookSourceUrl": fixture_base + "/" + suffix,
        "bookSourceName": "SSE fixture " + suffix,
        "searchUrl": fixture_base + "/search?key={{key}}" + delay,
        "ruleSearch": {"bookList": ".book", "name": ".name@text",
                       "author": ".author@text", "bookUrl": "a@href"},
        "ruleBookInfo": {"name": "h1@text", "author": ".author@text",
                         "tocUrl": ".toc@href"},
        "ruleToc": {"chapterList": ".chapter", "chapterName": "a@text",
                    "chapterUrl": "a@href"},
        "ruleContent": {"content": ".content@html"},
    }


def fixture_stats(base, reset=False):
    suffix = "/stats?reset=1" if reset else "/stats"
    with urllib.request.urlopen(base + suffix, timeout=5) as response:
        return json.loads(response.read().decode("utf-8"))


def abort_stream_then_probe(opener, base, fixture_base):
    endpoint = "/reader3/searchBookMultiSSE"
    params = urllib.parse.urlencode({"key": "差分", "lastIndex": -1,
                                     "searchSize": 3, "concurrentCount": 2})
    req = urllib.request.Request(base + endpoint + "?" + params)
    with opener.open(req, timeout=10) as response:
        first_frame = bytearray()
        while len(first_frame) < 100_000:
            line = response.readline()
            if not line:
                raise AssertionError("SSE ended before the first data frame")
            first_frame.extend(line)
            if first_frame.endswith(b"\n\n"):
                break
        else:
            raise AssertionError("First SSE frame exceeded 100 KB")
        parse_sse_frames(bytes(first_frame), "abort-first-frame", "message")
        deadline = time.monotonic() + 0.5
        source_active = False
        while time.monotonic() < deadline:
            source_active = fixture_stats(fixture_base)["activeSearches"] > 0
            if source_active:
                break
            time.sleep(0.02)
    # The third source remains in flight when the client's response closes.
    time.sleep(1.0)
    try:
        status, value = get_json(opener, base, "/reader3/getSystemInfo", timeout=5)
        healthy = status == 200 and value.get("isSuccess") is True
        repeat = read_sse(opener, base, "after-abort", endpoint,
                          {"key": "差分", "lastIndex": -1,
                           "searchSize": 1, "concurrentCount": 1}, "end", timeout=8)
        return {"probe": "abort-recovery", "sourceActiveAtClose": source_active,
                "systemInfoHealthy": healthy,
                "repeatSseComplete": repeat["dataCount"] == 1 and repeat["lastIndex"] == 0,
                "repeatTerminalEvent": repeat["terminalEvent"]}
    except (OSError, ValueError, urllib.error.URLError, AssertionError) as error:
        return {"probe": "abort-recovery", "sourceActiveAtClose": source_active,
                "systemInfoHealthy": False,
                "repeatSseComplete": False, "failureType": type(error).__name__}


def run_reader(jar, java, workdir, port, fixture_base, username, password):
    base = f"http://127.0.0.1:{port}"
    opener = urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    command = [str(java), "-jar", str(jar), f"--reader.app.workDir={workdir}",
               f"--reader.server.port={port}", "--reader.app.secure=true",
               "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"]
    with (workdir / "stdout.log").open("wb") as stdout, (workdir / "stderr.log").open("wb") as stderr:
        process = subprocess.Popen(command, cwd=workdir, stdout=stdout, stderr=stderr)
        try:
            deadline = time.monotonic() + 90
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"JAR exited during startup: {process.returncode}")
                try:
                    if get_json(opener, base, "/reader3/getSystemInfo", timeout=2)[1].get("isSuccess"):
                        break
                except (OSError, ValueError, urllib.error.URLError):
                    pass
                time.sleep(0.5)
            else:
                raise TimeoutError("Reader startup timeout")

            multi = "/reader3/searchBookMultiSSE"
            source_sse = "/reader3/searchBookSourceSSE"
            probes = [read_sse(opener, base, "anonymous", multi, {"key": "差分"}, "error")]
            credentials = {"username": username, "password": password}
            require_success(get_json(opener, base, "/reader3/login",
                                     dict(credentials, isLogin=False)), "register")
            require_success(get_json(opener, base, "/reader3/login",
                                     dict(credentials, isLogin=True)), "login")
            probes.append(read_sse(opener, base, "no-source", multi,
                                   {"key": "差分"}, "error"))
            first_source = source(fixture_base, "source-one")
            require_success(get_json(opener, base, "/reader3/saveBookSource", first_source),
                            "save first source")
            probes.append(read_sse(opener, base, "missing-key", multi, {}, "error"))
            probes.append(read_sse(opener, base, "one-source", multi,
                                   {"key": "差分", "lastIndex": -1,
                                    "searchSize": 1, "concurrentCount": 1}, "end"))
            book_url = fixture_base + "/book"
            probes.append(read_sse(opener, base, "unknown-book", source_sse,
                                   {"url": fixture_base + "/unknown"}, "error"))
            require_success(get_json(opener, base, "/reader3/getBookInfo",
                                     {"url": book_url, "bookSource": first_source}), "book info")
            probes.append(read_sse(opener, base, "one-source-switch", source_sse,
                                   {"url": book_url, "lastIndex": -1, "searchSize": 1}, "end"))
            require_success(get_json(opener, base, "/reader3/saveBookSource",
                                     source(fixture_base, "source-two")), "save second source")
            first_page = read_sse(opener, base, "two-source-page-1", multi,
                                  {"key": "差分", "lastIndex": -1,
                                   "searchSize": 1, "concurrentCount": 1}, "end")
            probes.append(first_page)
            probes.append(read_sse(opener, base, "two-source-page-2", multi,
                                   {"key": "差分", "lastIndex": first_page["lastIndex"],
                                    "searchSize": 1, "concurrentCount": 1}, "end"))
            fixture_stats(fixture_base, reset=True)
            probes.append(read_sse(opener, base, "two-source-concurrent", multi,
                                   {"key": "差分", "lastIndex": -1,
                                    "searchSize": 2, "concurrentCount": 2}, "end"))
            probes[-1]["maxFixtureConcurrency"] = fixture_stats(fixture_base)["maxActiveSearches"]
            probes.append(read_sse(opener, base, "exhausted", multi,
                                   {"key": "差分", "lastIndex": 1}, "error"))
            require_success(get_json(opener, base, "/reader3/saveBookSource",
                                     source(fixture_base, "source-three")), "save third source")
            probes.append(abort_stream_then_probe(opener, base, fixture_base))
            disconnect_lines = [line for file in (workdir / "logs").glob("reader-*.log")
                                for line in file.read_bytes().splitlines()
                                if b"searchBookMultiSSE" in line]
            # The original JAR writes local Windows logs in GBK; match only the
            # ASCII logger/method markers rather than assuming a log encoding.
            probes[-1]["serverObservedDisconnect"] = any(
                b"BookController - " in line and b"searchBookMultiSSE" in line
                for line in disconnect_lines
            )
            return probes
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


def accepted_cursor_fix(left, right, source_count):
    if left == right:
        return False
    revised = dict(left, isEnd=True)
    return (left.get("isEnd") is False and right.get("isEnd") is True and
            left.get("lastIndex") == source_count - 1 and revised == right)


def assert_contract(probes):
    by_name = {item["probe"]: item for item in probes}
    errors = {
        "anonymous": ("请登录后使用", "NEED_LOGIN"),
        "no-source": ("未配置书源", None),
        "missing-key": ("请输入搜索关键字", None),
        "unknown-book": ("书籍信息错误", None),
        "exhausted": ("没有更多了", None),
    }
    for name, (message, data) in errors.items():
        result = by_name[name]
        if (result["terminalEvent"] != "error" or result["frameCount"] != 1 or
                result["isSuccess"] is not False or result["errorMsg"] != message or
                (data is not None and result["data"] != data)):
            raise AssertionError(f"Unexpected SSE error contract: {name}")
    pages = {
        "one-source": (0, 1),
        "one-source-switch": (0, 1),
        "two-source-page-1": (0, 1),
        "two-source-page-2": (1, 1),
        "two-source-concurrent": (1, 2),
    }
    for name, (cursor, count) in pages.items():
        result = by_name[name]
        if (result["terminalEvent"] != "end" or result["frameCount"] != 2 or
                result["lastIndex"] != cursor or result["dataCount"] != count or
                not isinstance(result["isEnd"], bool) or
                result["cacheControl"] != "no-cache"):
            raise AssertionError(f"Unexpected SSE page contract: {name}: {result}")
    if by_name["two-source-page-1"]["isEnd"] is not False:
        raise AssertionError("First of two source pages must not be terminal")
    if by_name["one-source"]["resultSha256"] != by_name["two-source-page-1"]["resultSha256"]:
        raise AssertionError("First paginated result changed when adding the second source")
    if by_name["two-source-page-1"]["resultSha256"] == by_name["two-source-page-2"]["resultSha256"]:
        raise AssertionError("Second source page repeated the first source result")
    if by_name["two-source-concurrent"].get("maxFixtureConcurrency", 0) < 2:
        raise AssertionError("The fixture did not observe two overlapping source requests")
    recovery = by_name["abort-recovery"]
    if (recovery.get("repeatSseComplete") is not True or
            recovery.get("systemInfoHealthy") is not True or
            recovery.get("sourceActiveAtClose") is not True or
            recovery.get("serverObservedDisconnect") is not True):
        raise AssertionError(f"New SSE failed after client disconnect: {recovery}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", type=Path, default=PROJECT / ".tools/jdk-11.0.8/bin/java.exe")
    parser.add_argument("--original", type=Path,
                        default=PROJECT / "reference/original/reader-pro-3.2.14.original.jar")
    parser.add_argument("--restored", type=Path, default=PROJECT / "build/libs/reader-4.0.7.jar")
    parser.add_argument("--output", type=Path, default=PROJECT / "reports/sse-pagination-diff-latest.json")
    args = parser.parse_args()
    assert_incomplete_eof_rejected()
    for file in (args.java, args.original, args.restored):
        if not file.is_file():
            parser.error(f"Required file not found: {file}")
    tools_dir = PROJECT / ".tools"
    tools_dir.mkdir(exist_ok=True)
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    fixture_port, original_port, restored_port = sorted(ports)
    fixture_base = f"http://127.0.0.1:{fixture_port}"
    username = "sseprobe" + uuid.uuid4().hex[:10]
    password = "Probe-" + uuid.uuid4().hex
    with tempfile.TemporaryDirectory(prefix="sse-diff-", dir=tools_dir) as root_string:
        root = Path(root_string)
        first = root / "original"
        second = root / "restored"
        first.mkdir()
        second.mkdir()
        with (root / "fixture.log").open("wb") as fixture_log:
            fixture = subprocess.Popen(
                [sys.executable, "-B", str(PROJECT / "scripts/mock-book-source.py"),
                 "--port", str(fixture_port)], cwd=root,
                stdout=fixture_log, stderr=fixture_log,
            )
            try:
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if fixture.poll() is not None:
                        raise RuntimeError("Book source fixture exited during startup")
                    try:
                        with urllib.request.urlopen(fixture_base + "/health", timeout=2) as response:
                            if response.status == 200:
                                break
                    except OSError:
                        pass
                    time.sleep(0.2)
                else:
                    raise TimeoutError("Book source fixture startup timeout")
                original = run_reader(args.original, args.java, first, original_port,
                                      fixture_base, username, password)
                restored = run_reader(args.restored, args.java, second, restored_port,
                                      fixture_base, username, password)
            finally:
                fixture.terminate()
                try:
                    fixture.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    fixture.kill()
                    fixture.wait(timeout=5)
    assert_contract(original)
    assert_contract(restored)
    if len(original) != len(restored):
        raise AssertionError("SSE probe counts differ")
    comparisons = []
    for left, right in zip(original, restored):
        if left["probe"] != right["probe"]:
            raise AssertionError("SSE probe ordering differs")
        source_count = 2 if left["probe"] in ("two-source-page-2", "two-source-concurrent") else 1
        allowed = accepted_cursor_fix(left, right, source_count)
        comparisons.append({"probe": left["probe"], "equal": left == right,
                            "acceptedCursorFix": allowed, "original": left, "restored": right})
    report = {"originalJarSha256": hashlib.sha256(args.original.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(args.restored.read_bytes()).hexdigest(),
              "comparisons": comparisons}
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for item in comparisons:
        print(f"{item['probe']}: " + ("equal" if item["equal"] else
              "accepted cursor fix" if item["acceptedCursorFix"] else "DIFFERENT"))
    print(f"Report: {args.output}")
    if any(not item["equal"] and not item["acceptedCursorFix"] for item in comparisons):
        raise AssertionError("Unreviewed SSE behavior difference; inspect report")


if __name__ == "__main__":
    main()
