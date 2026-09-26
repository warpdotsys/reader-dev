#!/usr/bin/env python3
"""Compare a deterministic EPUB 3 navigation document and reading lifecycle."""

import hashlib
import importlib.util
import json
import secrets
import tempfile
from pathlib import Path
from xml.etree import ElementTree
from zipfile import ZIP_STORED, ZipFile, ZipInfo


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("epub_diff", Path(__file__).with_name("compare-epub-reading.py"))
EPUB = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EPUB)
BASE = EPUB.BASE
REPORT = ROOT / "reports/epub3-reading-diff-latest.json"

OPF3 = '''<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">urn:uuid:5a9561b3-4e07-4853-9c19-6ee7499b3f56</dc:identifier>
    <dc:title>EPUB 差分测试书</dc:title><dc:creator>测试作者</dc:creator>
    <dc:language>zh-CN</dc:language><dc:description>本地固定夹具</dc:description>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="one" href="Text/one.xhtml" media-type="application/xhtml+xml"/>
    <item id="two" href="Text/two.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="one"/><itemref idref="two"/></spine>
</package>'''.encode("utf-8")
NAV3 = '''<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head><title>目录</title></head><body>
  <nav epub:type="toc"><h1>目录</h1><ol>
    <li><a href="Text/one.xhtml">第一章</a></li>
    <li><a href="Text/two.xhtml">第二章</a></li>
  </ol></nav></body>
</html>'''.encode("utf-8")


def fixture(path):
    entries = (
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", EPUB.CONTAINER),
        ("OEBPS/content.opf", OPF3),
        ("OEBPS/nav.xhtml", NAV3),
        ("OEBPS/Text/one.xhtml", EPUB.chapter("第一章", "第一段，中文 EPUB 内容。")),
        ("OEBPS/Text/two.xhtml", EPUB.chapter("第二章", "第二段，标点 &amp; 空格。")),
    )
    with ZipFile(path, "w") as archive:
        for name, data in entries:
            entry = ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_STORED
            archive.writestr(entry, data)
    with ZipFile(path) as archive:
        if archive.namelist()[0] != "mimetype":
            raise AssertionError("EPUB mimetype must be first")
        for name, _ in entries[1:]:
            ElementTree.fromstring(archive.read(name))


def main():
    username = "epub3probe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-epub3-diff-") as temp:
        root = Path(temp)
        source = root / "epub3.epub"
        fixture(source)
        data = source.read_bytes()
        original, original_storage = EPUB.run_one(BASE.ORIGINAL, root / "original",
                                                  username, other_user, password, data)
        restored, restored_storage = EPUB.run_one(BASE.RESTORED, root / "restored",
                                                  username, other_user, password, data)
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("Probe sequence differs")
    shelf = next(row for row in restored if row["probe"] == "owner-shelf")
    for name, rows in (("original", original), ("restored", restored)):
        by_name = {row["probe"]: row for row in rows}
        preview = by_name["upload-preview"]["body"]
        if (preview.get("isSuccess") is not True or len(preview.get("data", [])) != 1
                or len(preview["data"][0].get("chapters", [])) != 2):
            raise AssertionError(f"{name}: EPUB 3 navigation preview did not expose two chapters")
        legacy = by_name["legacy-chapters"]["body"]
        if legacy.get("isSuccess") is not True or len(legacy.get("data", [])) != 2:
            raise AssertionError(f"{name}: legacy-layout EPUB 3 navigation failed")
        for index, title in enumerate(("第一章", "第二章")):
            html = by_name[f"legacy-html-{index}"]["body"]
            if (html.get("isSuccess") is not True or
                    title not in html.get("data", {}).get("content", "")):
                raise AssertionError(f"{name}: EPUB 3 chapter {index} body failed")
    direct = next(row for row in restored if row["probe"] == "chapter-list")["body"]
    if direct.get("isSuccess") is not True or len(direct.get("data", [])) != 2:
        raise AssertionError("Restored directly imported EPUB 3 navigation failed")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "acceptedDivergence": EPUB.accepted_epub_fix(left, right, shelf),
                    "original": left, "restored": right}
                   for left, right in zip(original, restored)]
    report = {"originalJarSha256": BASE.digest(BASE.ORIGINAL),
              "restoredJarSha256": BASE.digest(BASE.RESTORED),
              "fixtureSha256": hashlib.sha256(data).hexdigest(),
              "comparisons": comparisons,
              "storageEqual": original_storage == restored_storage,
              "storageAcceptedEpubFix": EPUB.accepted_storage_encoding(original_storage, restored_storage)}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        status = "equal" if row["equal"] else "accepted" if row["acceptedDivergence"] else "UNKNOWN"
        print(f"{row['probe']}: {status}")
    print("storageAcceptedEpubFix:", report["storageAcceptedEpubFix"])
    print("Report:", REPORT)
    if (len(comparisons) != 24 or
            sum(row["equal"] for row in comparisons) != 11 or
            sum(row["acceptedDivergence"] for row in comparisons) != 13 or
            not report["storageAcceptedEpubFix"] or any(
            not row["equal"] and not row["acceptedDivergence"] for row in comparisons)):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
