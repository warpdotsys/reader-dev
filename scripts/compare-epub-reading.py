#!/usr/bin/env python3
"""Compare deterministic native EPUB import and reading in isolated workdirs."""

import hashlib
import importlib.util
import json
import re
from copy import deepcopy
from pathlib import Path
import secrets
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
from xml.etree import ElementTree
from zipfile import ZIP_STORED, ZipFile, ZipInfo


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "base_diff", Path(__file__).with_name("compare-import-entrypoints.py"))
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
REPORT = ROOT / "reports/epub-reading-diff-latest.json"

CONTAINER = b'''<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>'''
OPF = '''<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="uid" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">urn:uuid:5a9561b3-4e07-4853-9c19-6ee7499b3f56</dc:identifier>
    <dc:title>EPUB 差分测试书</dc:title><dc:creator>测试作者</dc:creator>
    <dc:language>zh-CN</dc:language><dc:description>本地固定夹具</dc:description>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="one" href="Text/one.xhtml" media-type="application/xhtml+xml"/>
    <item id="two" href="Text/two.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine toc="ncx"><itemref idref="one"/><itemref idref="two"/></spine>
</package>'''.encode("utf-8")
NCX = '''<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head><meta name="dtb:uid" content="urn:uuid:5a9561b3-4e07-4853-9c19-6ee7499b3f56"/></head>
  <docTitle><text>EPUB 差分测试书</text></docTitle>
  <navMap>
    <navPoint id="one" playOrder="1"><navLabel><text>第一章</text></navLabel><content src="Text/one.xhtml"/></navPoint>
    <navPoint id="two" playOrder="2"><navLabel><text>第二章</text></navLabel><content src="Text/two.xhtml"/></navPoint>
  </navMap>
</ncx>'''.encode("utf-8")


def chapter(title, text):
    return (f'<?xml version="1.0" encoding="UTF-8"?>\n'
            f'<html xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title></head>'
            f'<body><h1>{title}</h1><p>{text}</p></body></html>').encode("utf-8")


def fixture(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    def fixed_entry(name, data):
        entry = ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
        entry.compress_type = ZIP_STORED
        return entry, data

    with ZipFile(path, "w") as archive:
        for name, data in (
                ("mimetype", b"application/epub+zip"),
                ("META-INF/container.xml", CONTAINER),
                ("OEBPS/content.opf", OPF),
                ("OEBPS/toc.ncx", NCX),
                ("OEBPS/Text/one.xhtml", chapter("第一章", "第一段，中文 EPUB 内容。")),
                ("OEBPS/Text/two.xhtml", chapter("第二章", "第二段，标点 &amp; 空格。"))):
            entry, body = fixed_entry(name, data)
            archive.writestr(entry, body)
    with ZipFile(path) as archive:
        if archive.namelist()[0] != "mimetype" or archive.read("mimetype") != b"application/epub+zip":
            raise AssertionError("EPUB mimetype is invalid")
        for name in ("META-INF/container.xml", "OEBPS/content.opf", "OEBPS/toc.ncx",
                     "OEBPS/Text/one.xhtml", "OEBPS/Text/two.xhtml"):
            ElementTree.fromstring(archive.read(name))


def run_one(jar, workdir, username, other_user, password, data):
    source = workdir / "storage/data" / username / "reading/probe.epub"
    source.parent.mkdir(parents=True, exist_ok=True)
    source.write_bytes(data)
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    log_file = (workdir / "reader.log").open("w", encoding="utf-8")
    process = subprocess.Popen(
        [str(BASE.JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=log_file, stderr=subprocess.STDOUT)
    anonymous, owner, other = BASE.client(), BASE.client(), BASE.client()
    probes = []

    def add(name, session, method, path, payload=None, upload=None):
        result = BASE.send(session, base, method, path, payload, upload)
        probes.append({"probe": name, **result})
        return result

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited during startup: {process.returncode}")
            try:
                if BASE.send(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")

        for session, name in ((owner, username), (other, other_user)):
            for is_login in (False, True):
                BASE.expect(BASE.send(session, base, "POST", "/reader3/login",
                                      {"username": name, "password": password,
                                       "isLogin": is_login}), True, "login")
        preview = add("upload-preview", owner, "POST", "/reader3/importBookPreview",
                      upload=("preview.epub", data))
        BASE.expect(preview, True, "upload-preview")
        imported = add("import-epub", owner, "GET",
                       "/reader3/file/parse?home=__HOME__&path=/reading&import=1")
        BASE.expect(imported, True, "import-epub")
        shelf = add("owner-shelf", owner, "GET", "/reader3/getBookshelf")
        books = BASE.expect(shelf, True, "owner-shelf")
        if len(books) != 1:
            raise AssertionError(f"Expected one imported EPUB, got {len(books)}")
        book_url = books[0]["bookUrl"]
        escaped = urllib.parse.quote(book_url, safe="")
        if BASE.expect(add("other-shelf", other, "GET", "/reader3/getBookshelf"),
                       True, "other-shelf") != []:
            raise AssertionError("EPUB crossed user namespace")
        add("book-info", owner, "GET", "/reader3/getBookInfo?url=" + escaped)
        chapters = add("chapter-list", owner, "GET", "/reader3/getChapterList?url=" + escaped)
        if chapters.get("body", {}).get("isSuccess"):
            chapter_data = BASE.expect(chapters, True, "chapter-list")
            if len(chapter_data) != 2:
                raise AssertionError(f"Expected two EPUB chapters, got {len(chapter_data)}")
            for index in range(2):
                content = add(f"content-{index}", owner, "GET",
                              "/reader3/getBookContent?url=" + escaped + f"&index={index}")
                url = content.get("body", {}).get("data")
                if isinstance(url, str) and url.startswith("/book-assets/"):
                    add(f"asset-{index}", owner, "GET", urllib.parse.quote(url, safe="/"))
                else:
                    probes.append({"probe": f"asset-{index}", "skipped": True})
                add(f"html-{index}", owner, "GET",
                    "/reader3/getBookContent?url=" + escaped
                    + f"&index={index}&epubContent=1")
        else:
            for index in range(2):
                probes.extend({"probe": f"{kind}-{index}", "skipped": True}
                              for kind in ("content", "asset", "html"))

        legacy_dir = workdir / "storage/data" / username / "reading/legacy.epub"
        legacy_dir.mkdir(parents=True)
        (legacy_dir / "index.epub").write_bytes(data)
        legacy_url = f"storage/data/{username}/reading/legacy.epub"
        add("save-legacy-layout", owner, "POST", "/reader3/saveBook",
            {"bookUrl": legacy_url, "originName": legacy_url, "origin": "loc_book",
             "name": "legacy", "author": "", "type": 0})
        legacy_escaped = urllib.parse.quote(legacy_url, safe="")
        legacy_chapters = add("legacy-chapters", owner, "GET",
                              "/reader3/getChapterList?url=" + legacy_escaped)
        if legacy_chapters.get("body", {}).get("isSuccess"):
            if len(BASE.expect(legacy_chapters, True, "legacy-chapters")) != 2:
                raise AssertionError("Legacy EPUB did not expose two chapters")
            for index in range(2):
                add(f"legacy-content-{index}", owner, "GET",
                    "/reader3/getBookContent?url=" + legacy_escaped + f"&index={index}")
                add(f"legacy-html-{index}", owner, "GET",
                    "/reader3/getBookContent?url=" + legacy_escaped
                    + f"&index={index}&epubContent=1")
        else:
            for index in range(2):
                probes.extend([{"probe": f"legacy-content-{index}", "skipped": True},
                               {"probe": f"legacy-html-{index}", "skipped": True}])
        shelf_file = workdir / "storage/data" / username / "bookshelf.json"
        if not shelf_file.is_file():
            raise AssertionError("Imported EPUB did not persist in bookshelf.json")
        storage = BASE.normalize(json.loads(shelf_file.read_text(encoding="utf-8")))
        original_bytes = source.read_bytes()
        deleted = add("delete-direct", owner, "POST", "/reader3/deleteBook",
                      {"bookUrl": book_url, "name": books[0]["name"],
                       "author": books[0]["author"]})
        BASE.expect(deleted, True, "delete-direct")
        if source.read_bytes() != original_bytes:
            raise AssertionError("Deleting the shelf entry modified the directly imported EPUB")
        cache_root = workdir / "storage/data" / username / (books[0]["name"] + "_" + books[0]["author"])
        if cache_root.exists():
            raise AssertionError("Deleting the shelf entry left its extracted EPUB cache")
        remaining = add("shelf-after-delete", owner, "GET", "/reader3/getBookshelf")
        if [item["bookUrl"] for item in BASE.expect(remaining, True, "shelf-after-delete")] != [legacy_url]:
            raise AssertionError("Deleting the direct EPUB damaged the legacy shelf entry")
        # Simulate an entry written before the fix: its title is the filename,
        # while originName still points to the single EPUB file.
        prior = {"bookUrl": book_url, "originName": books[0]["originName"],
                 "origin": "loc_book", "name": "probe", "author": "", "type": 0}
        BASE.expect(add("save-prior-record", owner, "POST", "/reader3/saveBook", prior),
                    True, "save-prior-record")
        prior_chapters = add("prior-record-chapters", owner, "GET",
                             "/reader3/getChapterList?url=" + escaped)
        if prior_chapters.get("body", {}).get("isSuccess"):
            add("prior-record-content", owner, "GET",
                "/reader3/getBookContent?url=" + escaped + "&index=0")
        else:
            probes.append({"probe": "prior-record-content", "skipped": True})
        BASE.expect(add("delete-prior-record", owner, "POST", "/reader3/deleteBook",
                        {"bookUrl": book_url, "name": "probe", "author": ""}),
                    True, "delete-prior-record")
        if source.read_bytes() != original_bytes:
            raise AssertionError("Deleting the prior-layout shelf entry modified its EPUB source")
        if (workdir / "storage/data" / username / "probe_").exists():
            raise AssertionError("Deleting the prior-layout shelf entry left its EPUB cache")
        return probes, storage
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        log_file.close()


def accepted_preview(left, right):
    if left["probe"] != right["probe"] or left["probe"] != "upload-preview":
        return False
    old, new = deepcopy(left), deepcopy(right)
    try:
        old_book = old["body"]["data"][0]["book"]
        new_book = new["body"]["data"][0]["book"]
        for field, wanted in (("name", "EPUB 差分测试书"), ("author", "测试作者")):
            before, after = old_book.pop(field), new_book.pop(field)
            if after != wanted or not before or before == after:
                return False
    except (KeyError, IndexError, TypeError):
        return False
    return old == new


def normalize_direct_book(old_book, new_book, after_reading=False):
    if old_book.pop("name", None) != "probe" or new_book.pop("name", None) != "EPUB 差分测试书":
        return False
    if old_book.pop("author", None) != "" or new_book.pop("author", None) != "测试作者":
        return False
    if old_book.pop("coverUrl", None) is not None:
        return False
    cover = new_book.pop("coverUrl", None)
    if not isinstance(cover, str) or not re.fullmatch(r"/assets/covers/[0-9a-f]{16,32}\.jpg", cover):
        return False
    for field in ("intro", "displayIntro"):
        before, after = old_book.pop(field, None), new_book.pop(field, None)
        if field == "displayIntro" and before is None and after is None:
            continue
        if not isinstance(before, str) or not before or before == "本地固定夹具" or after != "本地固定夹具":
            return False
    if old_book.pop("displayCover", None) is not None or new_book.pop("displayCover", cover) != cover:
        return False
    if after_reading:
        expected = {"latestChapterTitle": "第二章", "lastCheckCount": 2,
                    "totalChapterNum": 2, "durChapterTitle": "第二章", "durChapterIndex": 1}
        old_expected = {"latestChapterTitle": None, "lastCheckCount": 0,
                        "totalChapterNum": 0, "durChapterTitle": None, "durChapterIndex": 0}
        for field, wanted in expected.items():
            if old_book.pop(field, None) != old_expected[field] or new_book.pop(field, None) != wanted:
                return False
    return old_book == new_book


def accepted_owner_shelf(left, right):
    if left["probe"] != right["probe"] or left["probe"] != "owner-shelf":
        return False
    old, new = deepcopy(left), deepcopy(right)
    try:
        old_books, new_books = old["body"]["data"], new["body"]["data"]
        if len(old_books) != 1 or len(new_books) != 1:
            return False
        if not normalize_direct_book(old_books[0], new_books[0]):
            return False
    except (KeyError, IndexError, TypeError):
        return False
    return old == new


def accepted_book_info(left, right, restored_shelf):
    if left["probe"] != right["probe"] or left["probe"] != "book-info":
        return False
    return (left == {"probe": "book-info", "status": 200,
                    "contentType": "application/json; charset=utf-8",
                    "body": {"isSuccess": False, "errorMsg": "未配置书源"}}
            and right["status"] == 200
            and right["contentType"] == "application/json; charset=utf-8"
            and right["body"] == {"isSuccess": True, "errorMsg": "",
                                  "data": restored_shelf["body"]["data"][0]})


def normalize_legacy_book(old, new):
    for field in ("latestChapterTitle", "durChapterTitle"):
        before, after = old.pop(field, None), new.pop(field, None)
        if not isinstance(before, str) or not before or before == after or after != "第二章":
            return False
    return old == new


def accepted_storage_encoding(old, new):
    if not isinstance(old, list) or not isinstance(new, list) or len(old) != 2 or len(new) != 2:
        return False
    old, new = deepcopy(old), deepcopy(new)
    return (normalize_direct_book(old[0], new[0], after_reading=True)
            and normalize_legacy_book(old[1], new[1]))


def accepted_epub_fix(left, right, shelf):
    name = left["probe"]
    if name != right["probe"]:
        return False
    if name == "upload-preview":
        return accepted_preview(left, right)
    if name == "owner-shelf":
        return accepted_owner_shelf(left, right)
    if name == "book-info":
        return accepted_book_info(left, right, shelf)
    if name in ("chapter-list", "prior-record-chapters"):
        data = right.get("body", {}).get("data", [])
        return (left.get("body") == {"isSuccess": False, "errorMsg": "本地书籍源文件不存在"}
                and right.get("status") == 200 and right.get("body", {}).get("isSuccess") is True
                and len(data) == 2
                and [(row.get("url"), row.get("title"), row.get("index")) for row in data]
                == [("Text/one.xhtml", "第一章", 0), ("Text/two.xhtml", "第二章", 1)]
                and all(row.get("bookUrl") == shelf["body"]["data"][0]["bookUrl"] for row in data))
    if re.fullmatch(r"(content|asset|html)-[01]", name):
        if left != {"probe": name, "skipped": True}:
            return False
        index = int(name[-1])
        body = right.get("body", {})
        if name.startswith("content"):
            url = body.get("data")
            owner = shelf["body"]["data"][0]["bookUrl"].split("/")[2]
            return (right.get("status") == 200 and body.get("isSuccess") is True
                    and isinstance(url, str) and url.startswith(f"/book-assets/{owner}/")
                    and url.endswith(f"/OEBPS/Text/{('one', 'two')[index]}.xhtml"))
        if name.startswith("asset"):
            return (right.get("status") == 200 and right.get("contentType") == "application/xhtml+xml"
                    and right.get("length", 0) > 100
                    and re.fullmatch(r"[0-9a-f]{64}", right.get("sha256", "")) is not None)
        data = body.get("data", {})
        return (right.get("status") == 200 and body.get("isSuccess") is True
                and isinstance(data, dict) and isinstance(data.get("url"), str)
                and data["url"].startswith("__API_ROOT__/book-assets/")
                and f"第{('一', '二')[index]}章" in data.get("content", "")
                and "reader-inject-javascript" in data.get("content", ""))
    if name == "prior-record-content":
        owner = shelf["body"]["data"][0]["bookUrl"].split("/")[2]
        url = right.get("body", {}).get("data")
        return (left == {"probe": name, "skipped": True}
                and right.get("status") == 200 and right.get("body", {}).get("isSuccess") is True
                and isinstance(url, str) and url.startswith(f"/book-assets/{owner}/probe_/")
                and url.endswith("/epub-index/OEBPS/Text/one.xhtml"))
    if name == "shelf-after-delete":
        old, new = deepcopy(left), deepcopy(right)
        try:
            old_books, new_books = old["body"]["data"], new["body"]["data"]
            if len(old_books) != 1 or len(new_books) != 1 or not normalize_legacy_book(old_books[0], new_books[0]):
                return False
        except (KeyError, TypeError):
            return False
        return old == new
    return False


def main():
    for path in (BASE.JAVA, BASE.ORIGINAL, BASE.RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    username = "epubprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-epub-diff-") as temp:
        root = Path(temp)
        book_file = root / "fixture.epub"
        fixture(book_file)
        data = book_file.read_bytes()
        original, original_storage = run_one(BASE.ORIGINAL, root / "original",
                                             username, other_user, password, data)
        restored, restored_storage = run_one(BASE.RESTORED, root / "restored",
                                             username, other_user, password, data)
        if not next(row for row in restored if row["probe"] == "chapter-list")["body"].get("isSuccess"):
            log_lines = (root / "restored/reader.log").read_text(encoding="utf-8", errors="replace").splitlines()
            print("Restored EPUB log tail:\n" + "\n".join(log_lines[-50:]))
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("EPUB probe sequence differs")
    by_name = {row["probe"]: row for row in restored}
    comparisons = []
    for left, right in zip(original, restored):
        accepted = accepted_epub_fix(left, right, by_name["owner-shelf"])
        comparisons.append({"probe": left["probe"], "equal": left == right,
                            "acceptedDivergence": accepted,
                            "original": left, "restored": right})
    preview = by_name["upload-preview"]["body"]["data"]
    if len(preview) != 1 or len(preview[0]["chapters"]) != 2:
        raise AssertionError("EPUB upload preview did not parse two chapters")
    direct = by_name["chapter-list"]["body"]
    direct_readable = (direct.get("isSuccess") is True
                       and len(direct.get("data", [])) == 2)
    for index in range(2):
        html = by_name[f"legacy-html-{index}"]["body"]
        if (html.get("isSuccess") is not True or not isinstance(html.get("data"), dict)
                or f"第{('一', '二')[index]}章" not in html["data"].get("content", "")):
            raise AssertionError(f"Legacy EPUB chapter {index} did not return source HTML")
    storage_accepted = accepted_storage_encoding(original_storage, restored_storage)
    for index in range(2):
        url = by_name[f"content-{index}"]["body"]["data"]
        html = by_name[f"html-{index}"]["body"]["data"]
        asset = by_name[f"asset-{index}"]
        encoded = html["content"].encode("utf-8")
        if (html["url"] != "__API_ROOT__" + url or asset["length"] != len(encoded)
                or asset["sha256"] != hashlib.sha256(encoded).hexdigest()):
            raise AssertionError(f"Direct EPUB asset and HTML content differ for chapter {index}")
    report = {"originalJarSha256": BASE.digest(BASE.ORIGINAL),
              "restoredJarSha256": BASE.digest(BASE.RESTORED),
              "fixtureSha256": hashlib.sha256(data).hexdigest(),
              "comparisons": comparisons,
              "storageEqual": original_storage == restored_storage,
              "storageAcceptedEpubFix": storage_accepted,
              "previewMetadataCorrect": (preview[0]["book"].get("name") == "EPUB 差分测试书"
                                         and preview[0]["book"].get("author") == "测试作者"),
              "directReadable": direct_readable,
              "originalStorage": original_storage,
              "restoredStorage": restored_storage}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                      encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'accepted' if row['acceptedDivergence'] else 'DIFFERENT'}")
    print("storageEqual:", report["storageEqual"],
          "storageAcceptedEpubFix:", storage_accepted,
          "previewMetadataCorrect:", report["previewMetadataCorrect"],
          "directReadable:", direct_readable)
    print("Report:", REPORT)
    if not (storage_accepted and direct_readable and report["previewMetadataCorrect"]) or not all(
            row["equal"] or row["acceptedDivergence"] for row in comparisons):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
