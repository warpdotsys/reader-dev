#!/usr/bin/env python3
"""Compare book-source debugger SSE with a loopback-only HTML fixture."""

import argparse
import hashlib
from email.utils import parsedate_to_datetime
import importlib.util
import json
from pathlib import Path
import re
import secrets
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
BASE_SPEC = importlib.util.spec_from_file_location(
    "import_base", Path(__file__).with_name("compare-import-entrypoints.py"))
BASE = importlib.util.module_from_spec(BASE_SPEC)
BASE_SPEC.loader.exec_module(BASE)
SSE_SPEC = importlib.util.spec_from_file_location(
    "sse_base", Path(__file__).with_name("compare-sse-pagination.py"))
SSE = importlib.util.module_from_spec(SSE_SPEC)
SSE_SPEC.loader.exec_module(SSE)
REPORT = ROOT / "reports/book-source-debug-sse-diff-latest.json"
TIME_PREFIX = re.compile(r"^\[\d{2}:\d{2}\.\d{3}\] ")
HTTP_DURATION = re.compile(
    r"^(<elapsed> <-- 200 OK http://127\.0\.0\.1:\d+/[^ ]+) \((\d+)ms\)$")
BOOK_CREATED_FIELDS = ("latestChapterTime", "lastCheckTime", "durChapterTime")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalize_book_creation_times(msg):
    """Only elide constructor timestamps in the full-hit Book debug object."""
    if not msg.startswith("<elapsed> └{"):
        return msg
    book = json.loads(msg.removeprefix("<elapsed> └"))
    if not isinstance(book, dict) or "bookUrl" not in book:
        raise AssertionError("Unexpected object in Book debugger frame")
    now_ms = int(time.time() * 1000)
    for field in BOOK_CREATED_FIELDS:
        value = book.get(field)
        if type(value) is not int or abs(now_ms - value) > 180_000:
            raise AssertionError(f"Unexpected Book constructor timestamp: {field}={value}")
        msg, count = re.subn(r'(\"' + field + r'\": )(\d+)',
                             r'\g<1><created-ms>', msg)
        if count != 1:
            raise AssertionError(f"Unexpected Book timestamp format: {field}")
    return msg


def read_stream(opener, base, name, params, terminal):
    endpoint = "/reader3/bookSourceDebugSSE"
    request = urllib.request.Request(base + endpoint + "?" + urllib.parse.urlencode(params))
    with opener.open(request, timeout=25) as response:
        raw = response.read(2_000_001)
        status = response.status
        content_type = response.headers.get("Content-Type", "")
        cache_control = response.headers.get("Cache-Control", "")
    if len(raw) > 2_000_000:
        raise AssertionError(f"{name}: SSE exceeded byte limit")
    frames = SSE.parse_sse_frames(raw, name, terminal)
    normalized = []
    for event, data in frames:
        if event == "message":
            msg = data.get("msg")
            if not isinstance(msg, str) or not TIME_PREFIX.match(msg):
                raise AssertionError(f"{name}: malformed timed debugger message: {data}")
            msg = TIME_PREFIX.sub("<elapsed> ", msg, count=1)
            duration = HTTP_DURATION.fullmatch(msg)
            if duration:
                milliseconds = int(duration.group(2))
                if not 0 <= milliseconds <= 10_000:
                    raise AssertionError(f"{name}: unexpected fixture request duration: {milliseconds}")
                msg = duration.group(1) + " (<request-ms>)"
            elif msg.startswith("<elapsed> Date: "):
                parsedate_to_datetime(msg.removeprefix("<elapsed> Date: "))
                msg = "<elapsed> Date: <server-date>"
            elif msg.startswith("<elapsed> └{"):
                msg = normalize_book_creation_times(msg)
            data = {"msg": msg}
        normalized.append({"event": event, "data": data})
    return {"probe": name, "status": status, "contentType": content_type,
            "cacheControl": cache_control, "frames": normalized}


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

    def add(name, session, params, terminal):
        result = read_stream(session, base, name, params, terminal)
        probes.append(result)
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

        source_url = fixture_base + "/debug"
        add("anonymous", anonymous, {"bookSourceUrl": source_url,
                                     "keyword": "nomatch"}, "error")
        for session, name in ((owner, username), (other, other_user)):
            for is_login in (False, True):
                BASE.expect(BASE.send(session, base, "POST", "/reader3/login",
                                      {"username": name, "password": password,
                                       "isLogin": is_login}), True, "login")
        add("missing-source-url", owner, {"keyword": "nomatch"}, "error")
        add("missing-keyword", owner, {"bookSourceUrl": source_url}, "error")
        add("unknown-source", owner, {"bookSourceUrl": source_url,
                                      "keyword": "nomatch"}, "error")
        source = SSE.source(fixture_base, "debug")
        BASE.expect(BASE.send(owner, base, "POST", "/reader3/saveBookSource", source),
                    True, "save debug source")
        add("other-user-source-denied", other, {"bookSourceUrl": source_url,
                                                "keyword": "nomatch"}, "error")
        add("no-match-debug", owner, {"bookSourceUrl": source_url,
                                      "keyword": "nomatch"}, "end")
        add("one-hit-debug", owner, {"bookSourceUrl": source_url,
                                     "keyword": "差分"}, "end")
        return probes
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--include-hit", action="store_true",
                        help="Compatibility alias; the full-hit trace is now always checked")
    parser.parse_args()
    for path in (BASE.JAVA, BASE.ORIGINAL, BASE.RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    username = "debugprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    fixture_port = BASE.free_port()
    fixture_base = f"http://127.0.0.1:{fixture_port}"
    fixture = subprocess.Popen(
        [sys.executable, "-B", str(ROOT / "scripts/mock-book-source.py"),
         "--port", str(fixture_port), "--http11"], cwd=ROOT,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if fixture.poll() is not None:
                raise RuntimeError("Fixture exited")
            try:
                with urllib.request.urlopen(fixture_base + "/health", timeout=1) as response:
                    if response.status == 200:
                        break
            except OSError:
                pass
            time.sleep(0.1)
        else:
            raise TimeoutError("Fixture startup timeout")
        with tempfile.TemporaryDirectory(prefix="reader-debug-sse-diff-") as temp:
            root = Path(temp)
            original = run_one(BASE.ORIGINAL, root / "original", username,
                               other_user, password, fixture_base)
            restored = run_one(BASE.RESTORED, root / "restored", username,
                               other_user, password, fixture_base)
    finally:
        fixture.terminate()
        try:
            fixture.wait(timeout=5)
        except subprocess.TimeoutExpired:
            fixture.kill()
            fixture.wait(timeout=5)

    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("Probe sequence differs")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "original": left, "restored": right}
                   for left, right in zip(original, restored)]
    by_name = {row["probe"]: row for row in comparisons}
    if len(comparisons) != 7:
        raise AssertionError("Unexpected debugger probe count")
    expected_errors = {
        "anonymous": ("请登录后使用", "NEED_LOGIN"),
        "missing-source-url": ("未配置书源", None),
        "missing-keyword": ("请输入搜索关键词", None),
        "unknown-source": ("未配置书源", None),
        "other-user-source-denied": ("未配置书源", None),
    }
    for name, (message, data) in expected_errors.items():
        row = by_name[name]["restored"]
        if (row["status"] != 200 or row["contentType"] != "text/event-stream"
                or row["cacheControl"] != "no-cache" or len(row["frames"]) != 1
                or row["frames"][0]["event"] != "error"
                or row["frames"][0]["data"].get("errorMsg") != message
                or (data is not None and row["frames"][0]["data"].get("data") != data)):
            raise AssertionError(f"Debugger error frame changed: {name}")
    no_hit = by_name["no-match-debug"]["restored"]
    messages = [frame["data"]["msg"] for frame in no_hit["frames"]
                if frame["event"] == "message"]
    if (no_hit["status"] != 200 or no_hit["contentType"] != "text/event-stream"
            or no_hit["cacheControl"] != "no-cache"
            or no_hit["frames"][-1] != {"event": "end", "data": {"end": True}}
            or len(messages) < 20 or not any("未获取到书籍" in msg for msg in messages)
            or not any("GET " + fixture_base + "/search?key=nomatch" in msg for msg in messages)):
        raise AssertionError("Debugger no-match stream was not completed")
    report = {"originalJarSha256": sha256(BASE.ORIGINAL),
              "restoredJarSha256": sha256(BASE.RESTORED),
              "comparisons": comparisons}
    hit = by_name["one-hit-debug"]
    if (hit["original"]["status"] != 200 or hit["restored"]["status"] != 200
            or hit["original"]["frames"][-1] != {"event": "end", "data": {"end": True}}
            or hit["restored"]["frames"][-1] != {"event": "end", "data": {"end": True}}
            or len(hit["original"]["frames"]) < 100
            or len(hit["original"]["frames"]) != len(hit["restored"]["frames"])):
        raise AssertionError("Full-hit debugger stream incomplete")
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                      encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'DIFFERENT'}")
    print("Report:", REPORT)
    if not all(row["equal"] for row in comparisons):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
