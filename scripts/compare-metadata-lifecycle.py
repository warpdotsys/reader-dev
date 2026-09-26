#!/usr/bin/env python3
"""Differential test for RSS-source, bookmark, and replacement-rule metadata.

Both JARs run with generated users and independent temporary storage. No
production credentials, servers, source rules, or storage files are used.
"""

import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import http.cookiejar
import json
from pathlib import Path
import re
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid


ROOT = Path(__file__).resolve().parents[1]


class RssFixture(BaseHTTPRequestHandler):
    def do_GET(self):
        path = urllib.parse.urlsplit(self.path).path
        if path == "/feed.xml":
            body = ("<html><div class='item'><span class='title'>Fixture article</span>"
                    "<a class='link' href='/article/1'>Article</a>"
                    "<span class='summary'>Fixed summary</span></div></html>")
            content_type = "text/html; charset=utf-8"
        elif path == "/default.rss":
            link = f"http://127.0.0.1:{self.server.server_port}/article/1"
            body = ("<?xml version='1.0' encoding='UTF-8'?><rss version='2.0'><channel>"
                    "<title>Fixture feed</title><item><title>Fixture article</title>"
                    f"<link>{link}</link><description>Fixed summary</description>"
                    "<pubDate>Tue, 10 Sep 2024 00:00:00 GMT</pubDate>"
                    "</item></channel></rss>")
            content_type = "application/rss+xml; charset=utf-8"
        elif path == "/article/1":
            body = "<html><div class='article'>RSS body fixed.</div></html>"
            content_type = "text/html; charset=utf-8"
        else:
            self.send_error(404)
            return
        data = body.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, format, *args):
        pass
KNOWN_TEXT_DIFFERENCES = {
    "rss-one": {"body.data[0].sourceName": "测试 RSS", "body.data[0].tag": "测试 RSS"},
    "rss-updated": {"body.data[0].sourceName": "更新 RSS", "body.data[0].tag": "更新 RSS"},
    "rss-two": {"body.data[0].sourceName": "更新 RSS", "body.data[0].tag": "更新 RSS"},
    "bookmark-one": {
        "body.data[0].bookName": "书签测试书", "body.data[0].bookAuthor": "作者",
        "body.data[0].chapterName": "第一章",
    },
    "bookmark-updated": {
        "body.data[0].bookName": "书签测试书", "body.data[0].bookAuthor": "作者",
        "body.data[0].chapterName": "第二章",
    },
    "bookmark-two": {
        "body.data[0].bookName": "书签测试书", "body.data[0].bookAuthor": "作者",
        "body.data[0].chapterName": "第二章",
    },
}
KNOWN_STORAGE_TEXT_DIFFERENCES = {
    "rss-two": {"[0].sourceName": "更新 RSS", "[0].tag": "更新 RSS"},
    "bookmark-two": {
        "[0].bookName": "书签测试书", "[0].bookAuthor": "作者",
        "[0].chapterName": "第二章",
    },
}


def different_paths(left, right, prefix=""):
    if isinstance(left, dict) and isinstance(right, dict):
        result = []
        for key in sorted(left.keys() | right.keys()):
            path = f"{prefix}.{key}" if prefix else key
            if key not in left or key not in right:
                result.append(path)
            else:
                result.extend(different_paths(left[key], right[key], path))
        return result
    if isinstance(left, list) and isinstance(right, list):
        if len(left) != len(right):
            return [prefix + ".length"]
        return [path for index, (a, b) in enumerate(zip(left, right))
                for path in different_paths(a, b, f"{prefix}[{index}]")]
    return [] if left == right else [prefix]


def value_at(document, path):
    value = document
    for segment in re.findall(r"\[\d+\]|[^.\[\]]+", path):
        value = value[int(segment[1:-1])] if segment.startswith("[") else value[segment]
    return value


def accepted_text_difference(original, restored, expected):
    paths = different_paths(original, restored)
    if not expected or set(paths) != set(expected):
        return False, paths
    return all(value_at(restored, path) == value for path, value in expected.items()), paths


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def request(opener, base, method, path, payload=None, timeout=20):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=timeout)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read()
        try:
            body = json.loads(raw)
        except (UnicodeDecodeError, json.JSONDecodeError):
            body = {"nonJsonSha256": hashlib.sha256(raw).hexdigest(), "length": len(raw)}
        return {"status": response.status,
                "contentType": response.headers.get("Content-Type", ""),
                "body": body}


def opener():
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def require(result, success, name, count=None):
    body = result["body"]
    if result["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{name}: unexpected HTTP/ReturnData: {result}")
    if count is not None and len(body.get("data", [])) != count:
        raise AssertionError(f"{name}: expected {count} records, got {body.get('data')}")


def run_one(jar, java, workdir, username, password, fixture_base):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    command = [str(java), "-jar", str(jar),
               f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
               "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
               "--spring.profiles.active=prod"]
    with (workdir / "stdout.log").open("wb") as stdout, (workdir / "stderr.log").open("wb") as stderr:
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr, cwd=workdir)
        try:
            deadline = time.monotonic() + 90
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"JAR exited during startup: {process.returncode}")
                try:
                    ready = request(opener(), base, "GET", "/reader3/getSystemInfo", timeout=2)
                    if ready["status"] == 200 and ready["body"].get("isSuccess"):
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(0.5)
            else:
                raise TimeoutError("JAR did not become ready in 90 seconds")

            users = [opener(), opener()]
            for index, client in enumerate(users):
                identity = username + str(index)
                payload = {"username": identity, "password": password, "isLogin": False}
                require(request(client, base, "POST", "/reader3/login", payload), True, "register")
                payload["isLogin"] = True
                require(request(client, base, "POST", "/reader3/login", payload), True, "login")

            probes = []
            user_dir = workdir / "storage" / "data" / (username + "0")
            snapshots = {}

            def snapshot(name, filename):
                target = user_dir / filename
                if not target.is_file():
                    raise AssertionError(f"Missing expected storage file: {target}")
                snapshots[name] = json.loads(target.read_text(encoding="utf-8"))

            def probe(name, method, path, payload=None, user=0, success=True, count=None):
                response = request(users[user], base, method, path, payload)
                require(response, success, name, count)
                probes.append({"probe": name, **response})
                return response["body"].get("data")

            rss1 = {"sourceUrl": "https://example.invalid/rss/a", "sourceName": "测试 RSS"}
            rss2 = {"sourceUrl": "https://example.invalid/rss/b", "sourceName": "Other RSS"}
            probe("rss-empty", "GET", "/reader3/getRssSources", count=0)
            probe("rss-invalid", "POST", "/reader3/saveRssSource",
                  {"sourceUrl": rss1["sourceUrl"], "sourceName": ""}, success=False)
            probe("rss-save", "POST", "/reader3/saveRssSource", rss1)
            probe("rss-one", "GET", "/reader3/getRssSources", count=1)
            probe("rss-upsert", "POST", "/reader3/saveRssSource",
                  {**rss1, "sourceName": "更新 RSS"})
            probe("rss-updated", "GET", "/reader3/getRssSources", count=1)
            probe("rss-bulk", "POST", "/reader3/saveRssSources", [rss2])
            probe("rss-two", "GET", "/reader3/getRssSources", count=2)
            snapshot("rss-two", "rssSources.json")
            probe("rss-user-isolated", "GET", "/reader3/getRssSources", user=1, count=0)
            probe("rss-delete", "POST", "/reader3/deleteRssSource", rss1)
            probe("rss-after-delete", "GET", "/reader3/getRssSources", count=1)

            feed_url = fixture_base + "/feed.xml"
            article_url = fixture_base + "/article/1"
            rss_fixture = {"sourceUrl": feed_url, "sourceName": "Fixture feed",
                           "ruleArticles": ".item", "ruleTitle": ".title@text",
                           "ruleLink": ".link@href", "ruleDescription": ".summary@text",
                           "ruleContent": ".article@text"}
            query = urllib.parse.urlencode({"sourceUrl": feed_url, "sortName": "Fixture", "sortUrl": feed_url, "page": 1})
            probe("rss-articles-unknown-source", "GET", "/reader3/getRssArticles?" +
                  urllib.parse.urlencode({"sourceUrl": fixture_base + "/missing.xml"}), success=False)
            default_url = fixture_base + "/default.rss"
            default_source = {"sourceUrl": default_url, "sourceName": "Default RSS fixture"}
            require(request(users[0], base, "POST", "/reader3/saveRssSource", default_source),
                    True, "default-rss-save")
            default_query = urllib.parse.urlencode(
                {"sourceUrl": default_url, "sortName": "Fixture", "sortUrl": default_url, "page": 1})
            default_parser = request(users[0], base, "GET", "/reader3/getRssArticles?" + default_query)
            require(request(users[0], base, "POST", "/reader3/deleteRssSource", default_source),
                    True, "default-rss-delete")
            probe("rss-feed-save", "POST", "/reader3/saveRssSource", rss_fixture)
            probe("rss-articles-get", "GET", "/reader3/getRssArticles?" + query)
            probe("rss-articles-post", "POST", "/reader3/getRssArticles",
                  {"sourceUrl": feed_url, "sortName": "Fixture", "sortUrl": feed_url, "page": 1})
            content_query = urllib.parse.urlencode(
                {"sourceUrl": feed_url, "link": article_url, "origin": feed_url})
            probe("rss-content-get", "GET", "/reader3/getRssContent?" + content_query)
            probe("rss-content-post", "POST", "/reader3/getRssContent",
                  {"sourceUrl": feed_url, "link": article_url, "origin": feed_url})
            probe("rss-feed-delete", "POST", "/reader3/deleteRssSource", rss_fixture)

            bookmark1 = {"time": 1700000000001, "bookName": "书签测试书", "bookAuthor": "作者", "chapterName": "第一章"}
            bookmark2 = {"time": 1700000000002, "bookName": "Second book", "bookAuthor": "Author"}
            probe("bookmark-empty", "GET", "/reader3/getBookmarks", count=0)
            probe("bookmark-invalid", "POST", "/reader3/saveBookmark",
                  {"time": 1700000000000, "bookName": "", "bookAuthor": ""}, success=False)
            probe("bookmark-save", "POST", "/reader3/saveBookmark", bookmark1)
            probe("bookmark-one", "GET", "/reader3/getBookmarks", count=1)
            probe("bookmark-upsert", "POST", "/reader3/saveBookmark",
                  {**bookmark1, "chapterName": "第二章"})
            probe("bookmark-updated", "GET", "/reader3/getBookmarks", count=1)
            probe("bookmark-bulk", "POST", "/reader3/saveBookmarks", [bookmark2])
            probe("bookmark-two", "GET", "/reader3/getBookmarks", count=2)
            snapshot("bookmark-two", "bookmark.json")
            probe("bookmark-user-isolated", "GET", "/reader3/getBookmarks", user=1, count=0)
            probe("bookmark-delete", "POST", "/reader3/deleteBookmark", bookmark1)
            probe("bookmark-after-delete", "GET", "/reader3/getBookmarks", count=1)
            probe("bookmark-delete-bulk", "POST", "/reader3/deleteBookmarks", [bookmark2])
            probe("bookmark-after-bulk-delete", "GET", "/reader3/getBookmarks", count=0)

            rule1 = {"id": 1700000000001, "name": "Replace test", "pattern": "old", "replacement": "new"}
            rule2 = {"id": 1700000000002, "name": "Other rule", "pattern": "a", "replacement": "b"}
            probe("rule-empty", "GET", "/reader3/getReplaceRules", count=0)
            probe("rule-invalid", "POST", "/reader3/saveReplaceRule",
                  {"id": 1700000000000, "name": "", "pattern": "x"}, success=False)
            probe("rule-save", "POST", "/reader3/saveReplaceRule", rule1)
            probe("rule-one", "GET", "/reader3/getReplaceRules", count=1)
            probe("rule-upsert", "POST", "/reader3/saveReplaceRule",
                  {**rule1, "replacement": "updated"})
            probe("rule-updated", "GET", "/reader3/getReplaceRules", count=1)
            probe("rule-bulk", "POST", "/reader3/saveReplaceRules", [rule2])
            probe("rule-two", "GET", "/reader3/getReplaceRules", count=2)
            snapshot("rule-two", "replaceRule.json")
            probe("rule-user-isolated", "GET", "/reader3/getReplaceRules", user=1, count=0)
            probe("rule-delete", "POST", "/reader3/deleteReplaceRule", rule1)
            probe("rule-after-delete", "GET", "/reader3/getReplaceRules", count=1)
            probe("rule-delete-bulk", "POST", "/reader3/deleteReplaceRules", [rule2])
            probe("rule-after-bulk-delete", "GET", "/reader3/getReplaceRules", count=0)

            storage_root = workdir / "storage" / "data"
            other_dir = storage_root / (username + "1")
            stored = {}
            for filename in ("rssSources.json", "bookmark.json", "replaceRule.json"):
                target = user_dir / filename
                if not target.is_file():
                    raise AssertionError(f"Missing expected storage file: {target}")
                stored[filename] = json.loads(target.read_text(encoding="utf-8"))
                if (other_dir / filename).exists():
                    raise AssertionError(f"Second user's metadata file unexpectedly exists: {filename}")

            # The original JAR corrupts Chinese names on storage reads; its
            # name-keyed upsert consequently inserts a duplicate. Keep this
            # separate from the ASCII-key baseline and require the exact shape.
            unicode_rule = {"id": 1700000000003, "name": "中文替换", "pattern": "old", "replacement": "first"}
            first_save = request(users[0], base, "POST", "/reader3/saveReplaceRule", unicode_rule)
            require(first_save, True, "unicode-rule-save")
            first_list = request(users[0], base, "GET", "/reader3/getReplaceRules")
            require(first_list, True, "unicode-rule-one", count=1)
            update_save = request(users[0], base, "POST", "/reader3/saveReplaceRule",
                                  {**unicode_rule, "replacement": "updated"})
            require(update_save, True, "unicode-rule-upsert")
            after_update = request(users[0], base, "GET", "/reader3/getReplaceRules")
            require(after_update, True, "unicode-rule-after-upsert")
            unicode_storage = json.loads((user_dir / "replaceRule.json").read_text(encoding="utf-8"))
            return {"probes": probes, "storage": stored, "snapshots": snapshots,
                    "defaultParser": default_parser,
                    "unicodeRule": {"firstList": first_list, "afterUpdate": after_update,
                                    "storage": unicode_storage}}
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", type=Path, default=ROOT / ".tools/jdk-11.0.8/bin/java.exe")
    parser.add_argument("--original", type=Path, default=ROOT / "reference/original/reader-pro-3.2.14.original.jar")
    parser.add_argument("--restored", type=Path, default=ROOT / "build/libs/reader-4.0.7.jar")
    parser.add_argument("--output", type=Path, default=ROOT / "reports/metadata-lifecycle-diff-latest.json")
    args = parser.parse_args()
    for path in (args.java, args.original, args.restored):
        if not path.is_file():
            parser.error(f"Required file not found: {path}")
    tools_dir = ROOT / ".tools"
    tools_dir.mkdir(exist_ok=True)
    username = "meta" + uuid.uuid4().hex[:12]
    password = "Probe-" + uuid.uuid4().hex
    fixture = ThreadingHTTPServer(("127.0.0.1", 0), RssFixture)
    fixture_thread = threading.Thread(target=fixture.serve_forever, daemon=True)
    fixture_thread.start()
    try:
        fixture_base = f"http://127.0.0.1:{fixture.server_port}"
        with tempfile.TemporaryDirectory(prefix="metadata-diff-", dir=tools_dir) as root_string:
            root = Path(root_string)
            original_dir, restored_dir = root / "original", root / "restored"
            original_dir.mkdir()
            restored_dir.mkdir()
            original = run_one(args.original, args.java, original_dir, username, password, fixture_base)
            restored = run_one(args.restored, args.java, restored_dir, username, password, fixture_base)
    finally:
        fixture.shutdown()
        fixture.server_close()
        fixture_thread.join(timeout=5)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("Probe count differs")
    comparisons = []
    for left, right in zip(original["probes"], restored["probes"]):
        accepted, paths = accepted_text_difference(left, right, KNOWN_TEXT_DIFFERENCES.get(left["probe"]))
        comparisons.append({"probe": left["probe"], "equal": left == right,
                            "acceptedDivergence": accepted, "differentPaths": paths,
                            "original": left, "restored": right})
    snapshot_comparisons = []
    for name, left in original["snapshots"].items():
        right = restored["snapshots"][name]
        accepted, paths = accepted_text_difference(left, right, KNOWN_STORAGE_TEXT_DIFFERENCES.get(name))
        snapshot_comparisons.append({"name": name, "equal": left == right,
                                     "acceptedDivergence": accepted, "differentPaths": paths})
    report = {
        "originalJarSha256": hashlib.sha256(args.original.read_bytes()).hexdigest(),
        "restoredJarSha256": hashlib.sha256(args.restored.read_bytes()).hexdigest(),
        "comparisons": comparisons,
        "storageEqual": original["storage"] == restored["storage"],
        "storage": {"original": original["storage"], "restored": restored["storage"]},
        "snapshotEqual": original["snapshots"] == restored["snapshots"],
        "snapshotComparisons": snapshot_comparisons,
        "snapshots": {"original": original["snapshots"], "restored": restored["snapshots"]},
        "defaultParser": {"original": original["defaultParser"], "restored": restored["defaultParser"]},
        "unicodeRule": {"original": original["unicodeRule"], "restored": restored["unicodeRule"]},
    }
    old_default = original["defaultParser"]
    new_default = restored["defaultParser"]
    new_data = new_default["body"].get("data")
    new_articles = new_data.get("first", []) if isinstance(new_data, dict) else []
    default_fix_expected = (
        old_default["status"] == 200 and old_default["body"].get("isSuccess") is False
        and "No valid parser classes found" in old_default["body"].get("errorMsg", "")
        and new_default["status"] == 200 and new_default["body"].get("isSuccess") is True
        and len(new_articles) == 1 and new_articles[0].get("title") == "Fixture article"
        and new_articles[0].get("description") == "Fixed summary"
    )
    report["defaultParser"]["acceptedOldJarParserFix"] = default_fix_expected
    old_rows = original["unicodeRule"]["afterUpdate"]["body"]["data"]
    new_rows = restored["unicodeRule"]["afterUpdate"]["body"]["data"]
    unicode_divergence_expected = (
        len(old_rows) == 2 and len(new_rows) == 1
        and [row.get("replacement") for row in old_rows] == ["first", "updated"]
        and new_rows[0].get("name") == "中文替换"
        and new_rows[0].get("replacement") == "updated"
        and len(original["unicodeRule"]["storage"]) == 2
        and len(restored["unicodeRule"]["storage"]) == 1
    )
    report["unicodeRule"]["acceptedOldJarDuplicateOnChineseName"] = unicode_divergence_expected
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for item in comparisons:
        result = "equal" if item["equal"] else "accepted old-JAR text corruption" if item["acceptedDivergence"] else "DIFFERENT"
        print(f"{item['probe']}: {result}")
    print(f"Storage: {'equal' if report['storageEqual'] else 'DIFFERENT'}")
    for item in snapshot_comparisons:
        result = "equal" if item["equal"] else "accepted old-JAR text corruption" if item["acceptedDivergence"] else "DIFFERENT"
        print(f"Storage {item['name']}: {result}")
    print(f"Chinese-name rule upsert: {'accepted old-JAR duplicate' if unicode_divergence_expected else 'DIFFERENT'}")
    print(f"Default XML RSS parser: {'accepted old-JAR parser fix' if default_fix_expected else 'DIFFERENT'}")
    print(f"Report: {args.output}")
    if (any(not item["equal"] and not item["acceptedDivergence"] for item in comparisons)
            or any(not item["equal"] and not item["acceptedDivergence"] for item in snapshot_comparisons)
            or not report["storageEqual"] or not unicode_divergence_expected or not default_fix_expected):
        raise AssertionError("Metadata lifecycle differs from original JAR; inspect report")


if __name__ == "__main__":
    main()
