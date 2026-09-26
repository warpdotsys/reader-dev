#!/usr/bin/env python3
"""Compare three-page local PDF reading against the original JAR.

Uses only generated fixtures and isolated work directories. This remains a
separate coverage report; it does not change the 14-suite baseline counts.
"""

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import secrets
import shutil
import subprocess
import time
import urllib.parse

from PIL import Image
from pypdf import PdfReader
from reportlab.pdfgen import canvas


ROOT = Path(__file__).resolve().parents[1]
BASE_SPEC = importlib.util.spec_from_file_location(
    "pdf_reading_base", Path(__file__).with_name("compare-pdf-reading.py"))
BASE = importlib.util.module_from_spec(BASE_SPEC)
BASE_SPEC.loader.exec_module(BASE)
REPORT = ROOT / "reports/pdf-multipage-diff-latest.json"
PAGE_SIZES = ((288, 360), (360, 288), (300, 500))


def fixture(path, revised=False):
    path.parent.mkdir(parents=True, exist_ok=True)
    pdf = canvas.Canvas(str(path), pagesize=PAGE_SIZES[0], invariant=1,
                        pageCompression=0)
    pdf.setTitle("Reader three-page PDF differential fixture")
    for index, size in enumerate(PAGE_SIZES):
        pdf.setPageSize(size)
        width, height = size
        pdf.rect(22, 22, width - 44, height - 44)
        pdf.setFont("Helvetica-Bold", 16)
        label = f"PDF PAGE {index + 1}" + (" UPDATED" if revised and index == 1 else "")
        pdf.drawString(38, height - 66, label)
        pdf.setFont("Helvetica", 11)
        pdf.drawString(38, height - 92, f"Page {index + 1} of 3 - {width} x {height} pt")
        pdf.line(38, height - 105, width - 38, height - 105)
        pdf.showPage()
    pdf.save()
    reader = PdfReader(str(path))
    assert len(reader.pages) == 3
    for index, page in enumerate(reader.pages):
        assert f"PDF PAGE {index + 1}" in page.extract_text()


def image_row(path, name):
    if not path.is_file():
        return {"probe": name, "exists": False}
    with Image.open(path) as image:
        return {"probe": name, "exists": True, "size": list(image.size),
                "format": image.format, "sha256": BASE.sha256(path)}


def run_one(jar, workdir, username, password, source_pdf, revised_pdf):
    source = workdir / "storage/data" / username / "reading/probe.pdf"
    source.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source_pdf, source)
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    probes = []
    cache_checks = {}
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(
            [str(BASE.JAVA), "-jar", str(jar),
             f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
             "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
             "--spring.profiles.active=prod"], cwd=ROOT, stdout=log,
            stderr=subprocess.STDOUT)
        session = BASE.client()

        def add(name, method, path, payload=None):
            result = BASE.request(session, base, method, path, payload)
            probes.append({"probe": name, **result})
            return result

        def read_pages(prefix, book_url):
            escaped = urllib.parse.quote(book_url, safe="")
            add(prefix + "-chapters", "GET", "/reader3/getChapterList?url=" + escaped)
            for page_index in range(3):
                content = add(prefix + f"-content-{page_index}", "GET",
                              "/reader3/getBookContent?url=" + escaped +
                              f"&index={page_index}")
                html = content.get("body", {}).get("data", "")
                match = re.fullmatch(
                    rf"<img src='__API_ROOT__(/book-assets/[^']+/index/output-{page_index}\.png)' />",
                    html) if isinstance(html, str) else None
                if match:
                    add(prefix + f"-image-http-{page_index}", "GET", match.group(1))
                    image_path = workdir / "storage/data" / match.group(1).removeprefix(
                        "/book-assets/")
                else:
                    probes.append({"probe": prefix + f"-image-http-{page_index}",
                                   "skipped": True})
                    image_path = Path("")
                probes.append(image_row(image_path, prefix + f"-image-storage-{page_index}"))

        def check_cache(prefix, book_url, pdf_path):
            initial = next(row for row in probes if row["probe"] == prefix + "-content-1")
            html = initial.get("body", {}).get("data", "")
            match = re.fullmatch(
                r"<img src='__API_ROOT__(/book-assets/[^']+/index/output-1\.png)' />",
                html) if isinstance(html, str) else None
            if match is None:
                return None
            image_path = workdir / "storage/data" / match.group(1).removeprefix("/book-assets/")
            escaped = urllib.parse.quote(book_url, safe="")
            content_path = "/reader3/getBookContent?url=" + escaped + "&index=1"
            before = BASE.sha256(image_path)
            hit = BASE.request(session, base, "GET", content_path)
            after_hit = BASE.sha256(image_path)
            shutil.copyfile(revised_pdf, pdf_path)
            stale = BASE.request(session, base, "GET", content_path)
            after_stale = BASE.sha256(image_path)
            refreshed = BASE.request(session, base, "GET", content_path + "&refresh=1")
            after_refresh = BASE.sha256(image_path)
            image_http = BASE.request(session, base, "GET", match.group(1))
            return {"initialSha256": before, "cacheHitSha256": after_hit,
                    "sourceChangedButUnrefreshedSha256": after_stale,
                    "refreshedSha256": after_refresh,
                    "sameContentResponses": initial == {"probe": initial["probe"], **hit}
                    and hit == stale == refreshed,
                    "refreshedHttpSha256": image_http["sha256"],
                    "refreshedHttpStatus": image_http["status"]}

        try:
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"Reader exited during startup: {process.returncode}")
                try:
                    if BASE.request(session, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                        break
                except OSError:
                    pass
                time.sleep(0.4)
            else:
                raise TimeoutError("Reader startup timeout")
            for is_login in (False, True):
                result = BASE.request(session, base, "POST", "/reader3/login",
                                      {"username": username, "password": password,
                                       "isLogin": is_login})
                if result["status"] != 200 or result["body"]["isSuccess"] is not True:
                    raise AssertionError("Fixture account login failed")
            add("import", "GET", "/reader3/file/parse?home=__HOME__&path=/reading&import=1")
            shelf = add("shelf", "GET", "/reader3/getBookshelf")
            books = shelf.get("body", {}).get("data", [])
            if len(books) != 1:
                raise AssertionError("Expected exactly one imported PDF")
            read_pages("direct", books[0]["bookUrl"])
            cache_checks["direct"] = check_cache("direct", books[0]["bookUrl"], source)
            legacy_dir = workdir / "storage/data" / username / "reading/legacy.pdf"
            legacy_dir.mkdir(parents=True)
            shutil.copyfile(source_pdf, legacy_dir / "index.pdf")
            legacy_url = f"storage/data/{username}/reading/legacy.pdf"
            add("save-legacy", "POST", "/reader3/saveBook",
                {"bookUrl": legacy_url, "originName": legacy_url,
                 "origin": "loc_book", "name": "legacy", "author": "", "type": 0})
            read_pages("legacy", legacy_url)
            cache_checks["legacy"] = check_cache("legacy", legacy_url,
                                                   legacy_dir / "index.pdf")
            return probes, cache_checks
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def verify(original, restored):
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("Original and restored PDF probe sequences differ")
    outcomes = []
    for left, right in zip(original, restored):
        name = left["probe"]
        equal = left == right
        fixed = False
        if name == "direct-chapters" and not equal:
            fixed = (left.get("body", {}).get("errorMsg") == "本地书籍源文件不存在"
                     and right.get("body", {}).get("isSuccess") is True
                     and [x.get("index") for x in right["body"].get("data", [])] == [0, 1, 2])
        elif name.startswith("direct-content-") and not equal:
            index = int(name.rsplit("-", 1)[1])
            fixed = (left.get("body", {}).get("isSuccess") is False
                     and "probe.pdf\\index.pdf" in left.get("body", {}).get("errorMsg", "")
                     and right.get("body", {}).get("isSuccess") is True
                     and f"/index/output-{index}.png" in right["body"].get("data", ""))
        elif name.startswith("direct-image-http-") and not equal:
            index = int(name.rsplit("-", 1)[1])
            legacy = next(x for x in original if x["probe"] == f"legacy-image-http-{index}")
            fixed = left.get("skipped") is True and right.get("status") == 200 and {
                k: v for k, v in right.items() if k != "probe"} == {
                k: v for k, v in legacy.items() if k != "probe"}
        elif name.startswith("direct-image-storage-") and not equal:
            index = int(name.rsplit("-", 1)[1])
            legacy = next(x for x in original if x["probe"] == f"legacy-image-storage-{index}")
            fixed = left.get("exists") is False and right.get("exists") is True and {
                k: v for k, v in right.items() if k != "probe"} == {
                k: v for k, v in legacy.items() if k != "probe"}
        outcomes.append({"probe": name, "equal": equal, "acceptedDirectImportFix": fixed,
                         "original": left, "restored": right})
    return outcomes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture-only", type=Path)
    parser.add_argument("--fixture", type=Path)
    args = parser.parse_args()
    if args.fixture_only:
        fixture(args.fixture_only)
        print(args.fixture_only.resolve())
        return
    for path in (BASE.JAVA, BASE.ORIGINAL, BASE.RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    run_root = ROOT / ".tools" / ("pdf-multipage-diff-" + secrets.token_hex(8))
    run_root.mkdir(parents=True)
    source_pdf = args.fixture or run_root / "fixture/three-page.pdf"
    if args.fixture is None:
        fixture(source_pdf)
    if len(PdfReader(str(source_pdf)).pages) != 3:
        raise AssertionError("Expected three PDF pages")
    revised_pdf = run_root / "fixture/three-page-revised.pdf"
    fixture(revised_pdf, revised=True)
    username = "pdfmulti" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    left, original_cache = run_one(BASE.ORIGINAL, run_root / "original", username,
                                   password, source_pdf, revised_pdf)
    right, restored_cache = run_one(BASE.RESTORED, run_root / "restored", username,
                                    password, source_pdf, revised_pdf)
    comparisons = verify(left, right)
    if original_cache["direct"] is not None:
        raise AssertionError("Original JAR unexpectedly cached a directly imported PDF")
    for name, check in (("original legacy", original_cache["legacy"]),
                        ("restored legacy", restored_cache["legacy"]),
                        ("restored direct", restored_cache["direct"])):
        if (check is None or not check["sameContentResponses"]
                or check["refreshedHttpStatus"] != 200
                or check["initialSha256"] != check["cacheHitSha256"]
                or check["initialSha256"] != check["sourceChangedButUnrefreshedSha256"]
                or check["initialSha256"] == check["refreshedSha256"]
                or check["refreshedSha256"] != check["refreshedHttpSha256"]):
            raise AssertionError(f"PDF cache/refresh invariant failed: {name}: {check}")
    if not (original_cache["legacy"] == restored_cache["legacy"] ==
            restored_cache["direct"]):
        raise AssertionError("PDF cache/refresh image hashes differ across layouts")
    report = {"originalJarSha256": BASE.sha256(BASE.ORIGINAL),
              "restoredJarSha256": BASE.sha256(BASE.RESTORED),
              "fixtureSha256": BASE.sha256(source_pdf),
              "revisedFixtureSha256": BASE.sha256(revised_pdf),
              "cacheChecks": {"original": original_cache, "restored": restored_cache},
              "comparisons": comparisons}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'reviewed fix' if row['acceptedDirectImportFix'] else 'UNKNOWN DIFFERENCE'}")
    print(f"Report: {REPORT}")
    print(f"Isolated data: {run_root}")
    if not all(x["equal"] or x["acceptedDirectImportFix"] for x in comparisons):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
