#!/usr/bin/env python3
"""Verify all deterministic differential reports use the same current JARs."""

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPECS = (
    ("auth-lifecycle-diff-latest.json", 11, "semanticEqual", None),
    ("file-lifecycle-diff-latest.json", 30, "semanticEqual", "acceptedDivergence"),
    ("webdav-lifecycle-diff-latest.json", 19, "equal", None),
    ("sse-pagination-diff-latest.json", 11, "equal", "acceptedCursorFix"),
    ("web-source-reading-diff-latest.json", 28, "semanticEqual", "acceptedDivergence"),
    ("metadata-lifecycle-diff-latest.json", 44, "equal", "acceptedDivergence"),
    ("tts-lifecycle-diff-latest.json", 20, "equal", "acceptedDivergence"),
    ("book-group-lifecycle-diff-latest.json", 22, "equal", "acceptedDivergence"),
    ("backup-lifecycle-diff-latest.json", 21, "equal", None),
    ("book-source-crud-diff-latest.json", 29, "equal", None),
    ("user-admin-lifecycle-diff-latest.json", 23, "equal", None),
    ("local-shelf-delete-diff-latest.json", 31, "equal", "acceptedDivergence"),
    ("pdf-reading-diff-latest.json", 11, "equal", "acceptedDivergence"),
    ("pdf-multipage-diff-latest.json", 23, "equal", "acceptedDirectImportFix"),
    ("pdf-book-config-diff-latest.json", 21, "equal", "acceptedMissingUrlFix"),
    ("cache-book-sse-diff-latest.json", 12, "equal", None),
    ("search-json-diff-latest.json", 23, "equal", "acceptedMissingUrlFix"),
    ("file-batch-assets-diff-latest.json", 24, "equal", "acceptedSecurityFix"),
    ("import-entrypoints-diff-latest.json", 16, "equal", "accepted404Fix"),
    ("explore-book-diff-latest.json", 9, "equal", None),
    ("book-source-debug-sse-diff-latest.json", 7, "equal", None),
    ("epub-reading-diff-latest.json", 24, "equal", "acceptedDivergence"),
)
EXPECTED_BREAKDOWN = {
    "auth-lifecycle-diff-latest.json": (11, 0),
    "file-lifecycle-diff-latest.json": (27, 3),
    "webdav-lifecycle-diff-latest.json": (19, 0),
    "sse-pagination-diff-latest.json": (7, 4),
    "web-source-reading-diff-latest.json": (22, 6),
    "metadata-lifecycle-diff-latest.json": (38, 6),
    "tts-lifecycle-diff-latest.json": (19, 1),
    "book-group-lifecycle-diff-latest.json": (17, 5),
    "backup-lifecycle-diff-latest.json": (21, 0),
    "book-source-crud-diff-latest.json": (29, 0),
    "user-admin-lifecycle-diff-latest.json": (23, 0),
    "local-shelf-delete-diff-latest.json": (30, 1),
    "pdf-reading-diff-latest.json": (7, 4),
    "pdf-multipage-diff-latest.json": (13, 10),
    "pdf-book-config-diff-latest.json": (20, 1),
    "cache-book-sse-diff-latest.json": (12, 0),
    "search-json-diff-latest.json": (22, 1),
    "file-batch-assets-diff-latest.json": (22, 2),
    "import-entrypoints-diff-latest.json": (15, 1),
    "explore-book-diff-latest.json": (9, 0),
    "book-source-debug-sse-diff-latest.json": (7, 0),
    "epub-reading-diff-latest.json": (11, 13),
}


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main():
    original = sha256(ROOT / "reference/original/reader-pro-3.2.14.original.jar")
    restored = sha256(ROOT / "build/libs/reader-4.0.7.jar")
    totals = [0, 0, 0]
    for filename, expected_count, equal_key, accepted_key in SPECS:
        report = json.loads((ROOT / "reports" / filename).read_text(encoding="utf-8"))
        if report["originalJarSha256"].lower() != original or report["restoredJarSha256"].lower() != restored:
            raise AssertionError(f"{filename}: report JAR hash does not match current artifact")
        rows = report["comparisons"]
        if len(rows) != expected_count:
            raise AssertionError(f"{filename}: expected {expected_count} probes, got {len(rows)}")
        exact = sum(row[equal_key] is True for row in rows)
        accepted = sum(row.get(accepted_key) is True for row in rows) if accepted_key else 0
        if exact + accepted != len(rows):
            raise AssertionError(f"{filename}: unknown divergence in {len(rows) - exact - accepted} probes")
        if (exact, accepted) != EXPECTED_BREAKDOWN[filename]:
            raise AssertionError(f"{filename}: expected {EXPECTED_BREAKDOWN[filename]} exact/accepted, "
                                 f"got {(exact, accepted)}")
        if any(row[equal_key] is True and accepted_key and row.get(accepted_key) is True for row in rows):
            raise AssertionError(f"{filename}: probe marked both exact and divergent")
        if filename == "metadata-lifecycle-diff-latest.json":
            snapshots = report["snapshotComparisons"]
            if len(snapshots) != 3 or any(not (row["equal"] or row["acceptedDivergence"]) for row in snapshots):
                raise AssertionError("metadata storage snapshots have unknown differences")
            if (not report["storageEqual"]
                    or not report["unicodeRule"]["acceptedOldJarDuplicateOnChineseName"]
                    or not report["defaultParser"]["acceptedOldJarParserFix"]):
                raise AssertionError("metadata storage, Chinese-name rule, or default RSS parser boundary failed")
        if filename == "tts-lifecycle-diff-latest.json" and not report["storageEqual"]:
            raise AssertionError("TTS storage differs after delete lifecycle")
        if filename == "book-group-lifecycle-diff-latest.json" and not (
                report["groupStorageAcceptedDivergence"] and report["bookshelfStorageEqual"]):
            raise AssertionError("book-group storage has unknown differences")
        if filename == "backup-lifecycle-diff-latest.json" and not all(
                report[key] for key in ("webdavEntriesEqual", "downloadEntriesEqual", "storageEqual")):
            raise AssertionError("backup ZIP entries or restored storage differ")
        if filename == "book-source-crud-diff-latest.json" and not all(
                report[key] for key in ("storageEqual", "defaultStorageEqual", "sourceFileExistsEqual")):
            raise AssertionError("book-source CRUD storage or default inheritance differs")
        if filename == "user-admin-lifecycle-diff-latest.json" and not all(
                report[key] for key in ("storageEqual", "targetNamespaceRemoved")):
            raise AssertionError("user administration storage or namespace deletion differs")
        if filename == "local-shelf-delete-diff-latest.json" and not (
                report["storageEqual"] and report["sourceFilesRemainEqual"]
                and report["sourceFilesRemain"]["original"] == [True, True]
                and report["sourceFilesRemain"]["restored"] == [True, True]):
            raise AssertionError("local shelf deletion changed storage or removed source files")
        if filename == "pdf-reading-diff-latest.json":
            reviewed = {row["probe"] for row in rows if row["acceptedDivergence"]}
            if reviewed != {"pdf-chapters", "pdf-content", "pdf-image-http", "pdf-image-storage"}:
                raise AssertionError("PDF direct-import fix differs outside its reviewed boundary")
            by_name = {row["probe"]: row for row in rows}
            if (by_name["pdf-image-storage"]["restored"]["sha256"]
                    != by_name["legacy-image-storage"]["original"]["sha256"]):
                raise AssertionError("Direct PDF rendering differs from original directory rendering")
        if filename == "pdf-multipage-diff-latest.json":
            by_name = {row["probe"]: row for row in rows}
            expected_fixes = {"direct-chapters"}
            for index in range(3):
                expected_fixes.update({f"direct-content-{index}",
                                       f"direct-image-http-{index}",
                                       f"direct-image-storage-{index}"})
                if (by_name[f"direct-image-storage-{index}"]["restored"]["sha256"]
                        != by_name[f"legacy-image-storage-{index}"]["original"]["sha256"]):
                    raise AssertionError(f"PDF page {index} differs from the original renderer")
            if {row["probe"] for row in rows if row["acceptedDirectImportFix"]} != expected_fixes:
                raise AssertionError("Multipage PDF fixes differ outside the reviewed boundary")
            cache = report["cacheChecks"]
            if cache["original"]["direct"] is not None:
                raise AssertionError("Original JAR unexpectedly read directly imported PDF")
            checks = (cache["original"]["legacy"], cache["restored"]["legacy"],
                      cache["restored"]["direct"])
            if not checks[0] == checks[1] == checks[2]:
                raise AssertionError("PDF cache/refresh differs across original and restored layouts")
            check = checks[0]
            if (not check["sameContentResponses"] or check["refreshedHttpStatus"] != 200
                    or check["initialSha256"] != check["cacheHitSha256"]
                    or check["initialSha256"] != check["sourceChangedButUnrefreshedSha256"]
                    or check["initialSha256"] == check["refreshedSha256"]
                    or check["refreshedSha256"] != check["refreshedHttpSha256"]):
                raise AssertionError("PDF cache/refresh image invariant failed")
        if filename == "pdf-book-config-diff-latest.json":
            by_name = {row["probe"]: row for row in rows}
            if [row["probe"] for row in rows if row["acceptedMissingUrlFix"]] != ["missing-url"]:
                raise AssertionError("PDF config error fix differs outside the reviewed boundary")
            if report["fixtureSha256"] != json.loads(
                    (ROOT / "reports/pdf-reading-diff-latest.json").read_text(
                        encoding="utf-8"))["fixtureSha256"]:
                raise AssertionError("PDF config fixture differs from inspected reading fixture")
            for width, height in ((400, 499), (1200, 1500)):
                storage = by_name[f"width-{width}-image-storage"]["restored"]
                http = by_name[f"width-{width}-image-http"]["restored"]
                if (storage["size"] != [width, height] or storage["sha256"] != http["sha256"]
                        or by_name[f"save-width-{width}"]["restored"]["body"]["data"]["readConfig"]["pdfImageWidth"] != width
                        or by_name[f"shelf-width-{width}"]["restored"]["body"]["data"][0]["readConfig"]["pdfImageWidth"] != width):
                    raise AssertionError(f"PDF width {width} response, shelf or PNG differs")
            if (by_name["bookshelf-storage"]["restored"]["data"][0]["readConfig"]["pdfImageWidth"] != 1200
                    or by_name["other-user-shelf"]["restored"]["body"]["data"] != []
                    or by_name["other-user-denied"]["restored"]["body"]["errorMsg"] != "书籍信息错误"):
                raise AssertionError("PDF width persistence or user isolation failed")
        if filename == "cache-book-sse-diff-latest.json":
            by_name = {row["probe"]: row["restored"] for row in rows}
            for name, expected in (("first-cache", (2, 2)),
                                   ("repeat-cache", (2, 0)),
                                   ("force-refresh", (2, 2))):
                result = by_name[name]
                if (result["cacheControl"] != "no-cache"
                        or result["frames"][-1] != {"event": "end", "data": {
                            "cachedCount": expected[0], "successCount": expected[1],
                            "failedCount": 0}}):
                    raise AssertionError(f"Network book cache SSE contract failed: {name}")
            storage = by_name["first-storage"]["chapters"]
            if (storage != by_name["repeat-storage"]["chapters"]
                    or storage != by_name["refresh-storage"]["chapters"]
                    or len(storage["0"]) != 1 or len(storage["1"]) != 1):
                raise AssertionError("Network book chapter cache storage differs")
        if filename == "search-json-diff-latest.json":
            by_name = {row["probe"]: row for row in rows}
            if [row["probe"] for row in rows if row["acceptedMissingUrlFix"]] != ["missing-url-post"]:
                raise AssertionError("JSON search fixes differ outside the reviewed error boundary")
            if (by_name["missing-url-post"]["restored"]["body"]
                    != by_name["missing-url-get"]["restored"]["body"]):
                raise AssertionError("JSON search POST missing URL differs from GET")
            success_names = {"multi-one-get", "multi-one-post", "alternate-one-get",
                             "alternate-one-post", "multi-two-get", "multi-second-page",
                             "multi-group-a", "alternate-two-get", "alternate-second-page"}
            for row in rows:
                original_times = row["originalElapsedMs"]
                restored_times = row["restoredElapsedMs"]
                data = row["restored"]["body"].get("data")
                books = data.get("list", []) if isinstance(data, dict) else []
                if (len(original_times) != len(restored_times)
                        or len(original_times) != len(books)
                        or any(book.get("time") != "<elapsed-ms>" for book in books)
                        or any(type(value) is not int or not (0 <= value <= 120_000)
                               for value in original_times + restored_times)):
                    raise AssertionError(f"JSON search elapsed field normalization failed: {row['probe']}")
                if (row["probe"] in success_names) != (row["restored"]["body"].get("isSuccess") is True):
                    raise AssertionError(f"JSON search success/error branch changed: {row['probe']}")
            if (by_name["multi-one-get"]["restored"]["body"]
                    != by_name["multi-one-post"]["restored"]["body"]
                    or by_name["alternate-one-get"]["restored"]["body"]
                    != by_name["alternate-one-post"]["restored"]["body"]):
                raise AssertionError("JSON search GET/POST response semantics differ")
        if filename == "file-batch-assets-diff-latest.json" and not (
                report["acceptedOldJarTraversalFix"] and report["acceptedOldJarAssetPathFix"]
                and report["keptFileEqual"]
                and all(report["assetRemoved"].values())
                and report["fileGuardSurvived"] == {"original": False, "restored": True}
                and report["assetGuardSurvived"] == {"original": False, "restored": True}
                and report["escapedAssetCreated"] == {"original": True, "restored": False}):
            raise AssertionError("file batch/asset storage or traversal safety differs unexpectedly")
        if filename == "import-entrypoints-diff-latest.json":
            original_storage = report["storage"]["original"]
            restored_storage = report["storage"]["restored"]
            by_name = {row["probe"]: row for row in rows}
            if (original_storage != restored_storage
                    or [row["probe"] for row in rows if row["accepted404Fix"]] != ["remote-http-404"]
                    or by_name["remote-http-404"]["original"] != {
                        "probe": "remote-http-404", "timeoutAfterSeconds": 5}
                    or by_name["remote-http-404"]["restored"]["body"] != {
                        "isSuccess": False, "errorMsg": "远程书源链接错误：HTTP 404"}
                    or by_name["owner-invalid-sources-empty"]["restored"]["body"]["data"] != []
                    or by_name["other-invalid-sources-empty"]["restored"]["body"]["data"] != []
                    or not restored_storage["assetExists"]
                    or restored_storage["assetSha256"] != report["fixtureSha256"]
                    or restored_storage["otherAssetExists"]
                    or not restored_storage["sourceFileExists"]
                    or restored_storage["shelfFileExists"]
                    or len(restored_storage["sourceStorage"]) != 1
                    or by_name["remote-valid"]["restored"]["body"]["data"] != ""
                    or by_name["owner-shelf-after-preview"]["restored"]["body"]["data"] != []
                    or by_name["other-sources-after-import"]["restored"]["body"]["data"] != []):
                raise AssertionError("Import endpoint storage, response or isolation differs")
        if filename == "explore-book-diff-latest.json":
            by_name = {row["probe"]: row for row in rows}
            if report["fixturePaths"] != (["/explore?page=1", "/explore?page=1",
                                           "/explore?page=2", "/explore?page=2"] * 2):
                raise AssertionError("Discovery fixture did not receive both pages")
            for word, title, number in (("one", "Explore One", "1"),
                                        ("two", "Explore Two", "2")):
                get_body = by_name[f"page-{word}-get"]["restored"]["body"]
                post_body = by_name[f"page-{word}-post"]["restored"]["body"]
                books = get_body.get("data")
                if (get_body != post_body or get_body.get("isSuccess") is not True
                        or not isinstance(books, list) or len(books) != 1
                        or books[0].get("name") != title
                        or not books[0].get("bookUrl", "").endswith("/book/" + number)):
                    raise AssertionError(f"Discovery page {number} data differs")
            if by_name["other-no-source-after"]["restored"]["body"] != {
                    "isSuccess": False, "errorMsg": "未配置书源"}:
                raise AssertionError("Discovery source crossed user namespace")
        if filename == "book-source-debug-sse-diff-latest.json":
            by_name = {row["probe"]: row["restored"] for row in rows}
            if {name for name, row in by_name.items() if row["frames"][-1]["event"] == "error"} != {
                    "anonymous", "missing-source-url", "missing-keyword",
                    "unknown-source", "other-user-source-denied"}:
                raise AssertionError("Debugger error frames changed")
            stream = by_name["no-match-debug"]
            if (stream["contentType"] != "text/event-stream"
                    or stream["cacheControl"] != "no-cache"
                    or stream["frames"][-1] != {"event": "end", "data": {"end": True}}
                    or not any("未获取到书籍" in frame["data"].get("msg", "")
                               for frame in stream["frames"])):
                raise AssertionError("Debugger no-match stream incomplete")
            hit = by_name["one-hit-debug"]
            messages = [frame["data"].get("msg", "") for frame in hit["frames"]]
            if (hit["status"] != 200 or hit["contentType"] != "text/event-stream"
                    or hit["cacheControl"] != "no-cache" or len(hit["frames"]) < 100
                    or hit["frames"][-1] != {"event": "end", "data": {"end": True}}
                    or not any("<created-ms>" in msg for msg in messages)
                    or not any("第一段，中文与 UTF-8。" in msg for msg in messages)):
                raise AssertionError("Debugger full-hit stream incomplete")
        if filename == "epub-reading-diff-latest.json":
            by_name = {row["probe"]: row for row in rows}
            if ({row["probe"] for row in rows if row["acceptedDivergence"]}
                    != {"upload-preview", "owner-shelf", "book-info", "chapter-list",
                        "content-0", "asset-0", "html-0", "content-1", "asset-1", "html-1",
                        "shelf-after-delete", "prior-record-chapters", "prior-record-content"}
                    or not report["storageAcceptedEpubFix"] or report["storageEqual"]
                    or not report["previewMetadataCorrect"] or not report["directReadable"]
                    or report["fixtureSha256"] !=
                    "762e305bf87dee9124c75f063aaf9730b1c25d03df18f580099f9ab34795481b"):
                raise AssertionError("Native EPUB direct-import fix changed outside its reviewed boundary")
            direct_error = {"isSuccess": False, "errorMsg": "本地书籍源文件不存在"}
            if (by_name["chapter-list"]["original"]["body"] != direct_error
                    or by_name["chapter-list"]["restored"]["body"].get("isSuccess") is not True):
                raise AssertionError("Direct-file EPUB was not restored")
            for index, expected in enumerate(("第一段，中文 EPUB 内容。", "第二段，标点 &amp; 空格。")):
                probe = by_name[f"legacy-html-{index}"]["restored"]
                if (probe["body"].get("isSuccess") is not True
                        or expected not in probe["body"]["data"]["content"]):
                    raise AssertionError(f"Legacy EPUB chapter {index} HTML changed")
        totals[0] += len(rows)
        totals[1] += exact
        totals[2] += accepted
        print(f"{filename}: {len(rows)} probes, {exact} exact, {accepted} accepted divergence")
    print(f"Original JAR: {original}")
    print(f"Restored JAR: {restored}")
    print(f"TOTAL: {totals[0]} probes, {totals[1]} exact, {totals[2]} accepted divergence")


if __name__ == "__main__":
    main()
