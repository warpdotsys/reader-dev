#!/usr/bin/env python3
"""Compare local TXT shelf lookup, refresh, cache guards, and deletion.

Uses two generated TXT files, two generated users, and isolated temporary
storage for each JAR. No network source or production storage is accessed.
"""

import hashlib
from copy import deepcopy
import http.cookiejar
import json
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/local-shelf-delete-diff-latest.json"
DYNAMIC_BOOK_FIELDS = {"latestChapterTime", "lastCheckTime", "durChapterTime"}


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def normalize(value):
    if isinstance(value, list):
        return [normalize(item) for item in value]
    if isinstance(value, dict):
        result = {}
        for key, item in value.items():
            if key in DYNAMIC_BOOK_FIELDS:
                if not isinstance(item, int):
                    raise AssertionError(f"{key} is not numeric")
                result[key] = "<timestamp>"
            else:
                result[key] = normalize(item)
        return result
    return value


def request(opener, base, method, path, payload=None, normalize_json=True):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=25)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read()
        content_type = response.headers.get("Content-Type", "")
        if "json" not in content_type:
            return {"status": response.status, "contentType": content_type,
                    "length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        value = json.loads(raw.decode("utf-8"))
        return {"status": response.status, "contentType": content_type,
                "body": normalize(value) if normalize_json else value}


def expect(result, success, name, count=None):
    body = result.get("body", {})
    if result["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{name}: unexpected response: {result}")
    data = body.get("data")
    if count is not None and len(data) != count:
        raise AssertionError(f"{name}: expected {count} books, got {data}")
    return data


def accepted_cached_search_title_difference(left, right):
    if left["probe"] != "search-local-custom-content" or right["probe"] != left["probe"]:
        return False
    original_title = left["body"]["data"]["list"][0]["chapterTitle"]
    restored_title = right["body"]["data"]["list"][0]["chapterTitle"]
    if original_title != "绗竴绔� first" or restored_title != "第一章 first":
        return False
    projected = deepcopy(left)
    projected["body"]["data"]["list"][0]["chapterTitle"] = restored_title
    return projected == right


def run_one(jar, workdir, username, other_user, password):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    anonymous, owner, other = client(), client(), client()
    probes = []

    def add(name, session, method, path, payload=None):
        result = request(session, base, method, path, payload)
        probes.append({"probe": name, **result})
        return result

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited during startup: {process.returncode}")
            try:
                if request(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")

        expect(add("anonymous-shelf-book", anonymous, "GET", "/reader3/getShelfBook?url=missing"),
               False, "anonymous-shelf-book")
        expect(add("anonymous-delete", anonymous, "POST", "/reader3/deleteBook",
                   {"bookUrl": "missing"}), False, "anonymous-delete")
        expect(add("anonymous-cache-delete", anonymous, "POST", "/reader3/deleteBookCache",
                   {"url": "missing"}), False, "anonymous-cache-delete")
        expect(add("anonymous-chapters-by-rule", anonymous, "POST",
                   "/reader3/getChapterListByRule", {}), False, "anonymous-chapters-by-rule")
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                expect(request(session, base, "POST", "/reader3/login",
                               {"username": name, "password": password, "isLogin": login}),
                       True, "login")

        expect(add("shelf-book-missing-url", owner, "GET", "/reader3/getShelfBook"),
               False, "shelf-book-missing-url")
        expect(add("shelf-book-unknown", owner, "GET", "/reader3/getShelfBook?url=missing"),
               False, "shelf-book-unknown")
        expect(add("refresh-missing-url", owner, "POST", "/reader3/refreshLocalBook", {}),
               False, "refresh-missing-url")
        expect(add("refresh-unknown", owner, "POST", "/reader3/refreshLocalBook",
                   {"bookUrl": "missing"}), False, "refresh-unknown")
        expect(add("cache-delete-missing-url", owner, "POST", "/reader3/deleteBookCache", {}),
               False, "cache-delete-missing-url")
        expect(add("cache-server-empty", owner, "POST", "/reader3/cacheBookOnServer",
                   {"bookUrlList": []}), False, "cache-server-empty")
        expect(add("chapters-by-rule-no-origin", owner, "POST",
                   "/reader3/getChapterListByRule", {}), False, "chapters-by-rule-no-origin")
        expect(add("chapters-by-rule-not-local", owner, "POST",
                   "/reader3/getChapterListByRule", {"origin": "https://fixture.invalid"}),
               False, "chapters-by-rule-not-local")

        for name in ("first", "second"):
            expect(request(owner, base, "POST", "/reader3/file/save",
                           {"home": "__HOME__", "path": f"/reading/{name}.txt",
                            "content": f"第一章 {name}\n固定正文。\n第二章 继续\n结束。"}),
                   True, "prepare-local-txt")
        expect(request(owner, base, "GET", "/reader3/file/parse?home=__HOME__&path=/reading&import=1"),
               True, "prepare-import")
        shelf = expect(add("shelf-two", owner, "GET", "/reader3/getBookshelf"),
                       True, "shelf-two", 2)
        urls = [book["bookUrl"] for book in shelf]
        if len(set(urls)) != 2:
            raise AssertionError("imported books do not have distinct URLs")
        first, second = urls
        raw_shelf = expect(request(owner, base, "GET", "/reader3/getBookshelf",
                                   normalize_json=False), True, "raw-shelf", 2)
        by_rule = expect(add("chapters-by-rule-local-txt", owner, "POST",
                             "/reader3/getChapterListByRule", raw_shelf[0]),
                         True, "chapters-by-rule-local-txt")
        if by_rule["book"]["bookUrl"] != first or len(by_rule["chapters"]) != 2:
            raise AssertionError("local TXT rule did not return the same book and two chapters")
        expect(add("get-first", owner, "GET", "/reader3/getShelfBook?url="
                   + urllib.parse.quote(first, safe="")), True, "get-first")
        expect(add("get-cache-info", owner, "GET", "/reader3/getShelfBookWithCacheInfo"),
               True, "get-cache-info", 2)
        expect(add("other-user-unknown", other, "GET", "/reader3/getShelfBook?url="
                   + urllib.parse.quote(first, safe="")), False, "other-user-unknown")
        expect(add("cache-delete-local", owner, "POST", "/reader3/deleteBookCache",
                   {"url": first}), False, "cache-delete-local")
        expect(add("refresh-first", owner, "POST", "/reader3/refreshLocalBook",
                   {"bookUrl": first}), True, "refresh-first")
        expect(add("shelf-after-refresh", owner, "GET", "/reader3/getBookshelf"),
               True, "shelf-after-refresh", 2)
        expect(add("cache-server-local", owner, "POST", "/reader3/cacheBookOnServer",
                   {"bookUrlList": [first]}), True, "cache-server-local")
        search = expect(add("search-local-content", owner, "POST", "/reader3/searchBookContent",
                            {"url": first, "keyword": "固定正文", "lastIndex": -1, "size": 10}),
                        True, "search-local-content")
        if len(search["list"]) != 1 or search["list"][0]["chapterIndex"] != 0:
            raise AssertionError("local TXT content search did not find chapter zero")
        custom_content = "\n自定义固定正文。"
        expect(add("save-local-custom-content", owner, "POST", "/reader3/saveBookContent",
                   {"url": first, "index": 0, "content": custom_content}),
               True, "save-local-custom-content")
        custom_search = expect(add("search-local-custom-content", owner, "POST",
                                   "/reader3/searchBookContent",
                                   {"url": first, "keyword": "自定义固定",
                                    "lastIndex": -1, "size": 10}),
                               True, "search-local-custom-content")
        if (len(custom_search["list"]) != 1
                or custom_search["list"][0]["resultText"] != custom_content
                or custom_search["list"][0]["queryIndexInChapter"] != 1
                or custom_search["list"][0]["queryIndexInResult"] != 1):
            raise AssertionError("custom local chapter cache was not searched verbatim")
        user_dir = workdir / "storage/data" / username
        zero_files = list(user_dir.rglob("0.txt"))
        custom_files = [path for path in zero_files if path.parent.name == "custom"]
        chapter_files = [path for path in zero_files if path.parent.name != "custom"
                         and path.read_text(encoding="utf-8") == custom_content]
        if (len(custom_files) != 1
                or custom_files[0].read_text(encoding="utf-8") != custom_content
                or not chapter_files):
            raise AssertionError("custom local content was not persisted in both cache locations")
        probes.append({"probe": "custom-local-cache-storage",
                       "contentSha256": hashlib.sha256(
                           custom_files[0].read_bytes()).hexdigest(),
                       "matchingChapterCache": True})
        expect(add("delete-first", owner, "POST", "/reader3/deleteBook",
                   {"bookUrl": first}), True, "delete-first")
        expect(add("shelf-after-delete-one", owner, "GET", "/reader3/getBookshelf"),
               True, "shelf-after-delete-one", 1)
        expect(add("delete-first-again", owner, "POST", "/reader3/deleteBook",
                   {"bookUrl": first}), False, "delete-first-again")
        expect(add("delete-second-bulk", owner, "POST", "/reader3/deleteBooks",
                   [{"bookUrl": second}]), True, "delete-second-bulk")
        expect(add("shelf-empty", owner, "GET", "/reader3/getBookshelf"),
               True, "shelf-empty", 0)
        expect(add("other-shelf-empty", other, "GET", "/reader3/getBookshelf"),
               True, "other-shelf-empty", 0)

        shelf_file = user_dir / "bookshelf.json"
        source_files = [user_dir / "reading/first.txt", user_dir / "reading/second.txt"]
        source_files_remain = [path.exists() for path in source_files]
        if source_files_remain != [True, True]:
            raise AssertionError("deleting shelf entries removed the original TXT files")
        return {"probes": probes,
                "shelfStorage": normalize(json.loads(shelf_file.read_text(encoding="utf-8"))),
                "sourceFilesRemain": source_files_remain}
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
    username = "shelfprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-local-shelf-diff-") as temp:
        root = Path(temp)
        original = run_one(ORIGINAL, root / "original", username, other_user, password)
        restored = run_one(RESTORED, root / "restored", username, other_user, password)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("probe count differs")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "acceptedDivergence": left != right and
                    accepted_cached_search_title_difference(left, right),
                    "original": left, "restored": right}
                   for left, right in zip(original["probes"], restored["probes"])]
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons,
              "storageEqual": original["shelfStorage"] == restored["shelfStorage"],
              "sourceFilesRemainEqual": original["sourceFilesRemain"] == restored["sourceFilesRemain"],
              "sourceFilesRemain": {"original": original["sourceFilesRemain"],
                                    "restored": restored["sourceFilesRemain"]},
              "storage": {"original": original["shelfStorage"],
                          "restored": restored["shelfStorage"]}}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'reviewed difference' if row['acceptedDivergence'] else 'UNKNOWN DIFFERENCE'}")
    print(f"storageEqual: {report['storageEqual']}")
    print(f"sourceFilesRemainEqual: {report['sourceFilesRemainEqual']}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] or row["acceptedDivergence"] for row in comparisons) or not all(
            report[key] for key in ("storageEqual", "sourceFilesRemainEqual")):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
