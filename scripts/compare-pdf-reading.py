#!/usr/bin/env python3
"""Compare one-page local PDF import and reading against the original JAR.

The PDF and both storage roots are generated locally. No production data is read.
Run with the bundled Python runtime, which provides reportlab, pypdf, and Pillow.
"""

import argparse
import hashlib
import http.cookiejar
import json
from pathlib import Path
import re
import secrets
import shutil
import socket
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request

from PIL import Image
from pypdf import PdfReader
from reportlab.pdfgen import canvas


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/pdf-reading-diff-latest.json"
DYNAMIC_BOOK_FIELDS = {"latestChapterTime", "lastCheckTime", "durChapterTime"}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def make_fixture(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    page = canvas.Canvas(str(path), pagesize=(288, 360), invariant=1,
                         pageCompression=0)
    page.setTitle("Reader PDF differential fixture")
    page.setStrokeColorRGB(0, 0, 0)
    page.rect(24, 24, 240, 312)
    page.setFont("Helvetica-Bold", 16)
    page.drawString(42, 290, "PDF PAGE ONE")
    page.setFont("Helvetica", 11)
    page.drawString(42, 260, "Deterministic local reading fixture")
    page.line(42, 245, 246, 245)
    page.showPage()
    page.save()
    reader = PdfReader(str(path))
    if len(reader.pages) != 1 or "PDF PAGE ONE" not in reader.pages[0].extract_text():
        raise AssertionError("Generated PDF fixture is not a readable one-page document")


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
        return {key: "<timestamp>" if key in DYNAMIC_BOOK_FIELDS and isinstance(item, int)
                else normalize(item) for key, item in value.items()}
    return value


def request(opener, base, method, path, payload=None):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=35)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read()
        content_type = response.headers.get("Content-Type", "")
        if "json" in content_type:
            return {"status": response.status, "contentType": content_type,
                    "body": normalize(json.loads(raw.decode("utf-8")))}
        return {"status": response.status, "contentType": content_type,
                "length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def run_one(jar, workdir, username, password, fixture):
    source = workdir / "storage/data" / username / "reading/probe.pdf"
    source.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(fixture, source)
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(
            [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
             f"--reader.server.port={port}", "--reader.app.secure=true",
             "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
            cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        session = client()
        probes = []

        def add(name, method, path, payload=None):
            result = request(session, base, method, path, payload)
            probes.append({"probe": name, **result})
            return result

        try:
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"Reader exited during startup: {process.returncode}")
                try:
                    if request(session, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                        break
                except (OSError, urllib.error.URLError):
                    pass
                time.sleep(0.4)
            else:
                raise TimeoutError("Reader startup timeout")

            for login in (False, True):
                result = request(session, base, "POST", "/reader3/login",
                                 {"username": username, "password": password, "isLogin": login})
                if result["status"] != 200 or result["body"]["isSuccess"] is not True:
                    raise AssertionError("Fixture account login failed")

            add("import-pdf", "GET", "/reader3/file/parse?home=__HOME__&path=/reading&import=1")
            shelf = add("pdf-shelf", "GET", "/reader3/getBookshelf")
            books = shelf.get("body", {}).get("data", [])
            if len(books) == 1:
                book_url = books[0]["bookUrl"]
                escaped = urllib.parse.quote(book_url, safe="")
                add("pdf-chapters", "GET", "/reader3/getChapterList?url=" + escaped)
                content = add("pdf-content", "GET",
                              "/reader3/getBookContent?url=" + escaped + "&index=0")
                html = content.get("body", {}).get("data", "")
                image_match = re.fullmatch(
                    r"<img src='__API_ROOT__(/book-assets/[^']+/index/output-0\.png)' />",
                    html) if isinstance(html, str) else None
                if image_match:
                    add("pdf-image-http", "GET", image_match.group(1))
                else:
                    probes.append({"probe": "pdf-image-http", "skipped": True})
                image_path = (workdir / "storage/data" /
                              image_match.group(1).removeprefix("/book-assets/")) if image_match else None
                if image_path is not None and image_path.is_file():
                    with Image.open(image_path) as image:
                        probes.append({"probe": "pdf-image-storage", "exists": True,
                                       "size": list(image.size), "format": image.format,
                                       "sha256": sha256(image_path)})
                else:
                    probes.append({"probe": "pdf-image-storage", "exists": False})
            legacy_dir = workdir / "storage/data" / username / "reading/legacy.pdf"
            legacy_dir.mkdir(parents=True)
            shutil.copyfile(fixture, legacy_dir / "index.pdf")
            legacy_url = f"storage/data/{username}/reading/legacy.pdf"
            add("save-legacy-layout", "POST", "/reader3/saveBook",
                {"bookUrl": legacy_url, "originName": legacy_url, "origin": "loc_book",
                 "name": "legacy", "author": "", "type": 0})
            legacy_escaped = urllib.parse.quote(legacy_url, safe="")
            add("legacy-chapters", "GET", "/reader3/getChapterList?url=" + legacy_escaped)
            legacy_content = add("legacy-content", "GET",
                                 "/reader3/getBookContent?url=" + legacy_escaped + "&index=0")
            legacy_html = legacy_content.get("body", {}).get("data", "")
            legacy_match = re.fullmatch(
                r"<img src='__API_ROOT__(/book-assets/[^']+/index/output-0\.png)' />",
                legacy_html) if isinstance(legacy_html, str) else None
            if legacy_match:
                add("legacy-image-http", "GET", legacy_match.group(1))
            else:
                probes.append({"probe": "legacy-image-http", "skipped": True})
            legacy_image = legacy_dir / "index/output-0.png"
            if legacy_image.is_file():
                with Image.open(legacy_image) as image:
                    probes.append({"probe": "legacy-image-storage", "exists": True,
                                   "size": list(image.size), "format": image.format,
                                   "sha256": sha256(legacy_image)})
            else:
                probes.append({"probe": "legacy-image-storage", "exists": False})
            return probes
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def accepted_direct_pdf_fix(left, right, username, run_root, legacy_http, legacy_storage):
    name = left["probe"]
    if right["probe"] != name:
        return False
    content_type = "application/json; charset=utf-8"
    if name == "pdf-chapters":
        expected_url = f"storage/data/{username}/reading/probe.pdf"
        return (left == {"probe": name, "status": 200, "contentType": content_type,
                        "body": {"isSuccess": False,
                                 "errorMsg": "本地书籍源文件不存在"}}
                and right == {"probe": name, "status": 200, "contentType": content_type,
                              "body": {"isSuccess": True, "errorMsg": "", "data": [{
                                  "url": "output-0.png", "title": "output-0.png",
                                  "isVolume": False, "baseUrl": "", "bookUrl": expected_url,
                                  "index": 0, "start": 0, "end": 0}]}})
    if name == "pdf-content":
        error_prefix = "java.io.FileNotFoundException: " + str(
            run_root / "original/storage/data" / username / "reading/probe.pdf/index.pdf")
        old_body = left.get("body", {})
        new_body = right.get("body", {})
        new_html = new_body.get("data", "")
        return (left.get("status") == right.get("status") == 200
                and left.get("contentType") == right.get("contentType") == content_type
                and old_body.get("isSuccess") is False
                and old_body.get("errorMsg", "").startswith(error_prefix)
                and new_body.get("isSuccess") is True and new_body.get("errorMsg") == ""
                and isinstance(new_html, str)
                and re.fullmatch(
                    rf"<img src='__API_ROOT__/book-assets/{username}/probe_/"
                    r"[0-9a-f]{32}/index/output-0\.png' />", new_html) is not None)
    if name == "pdf-image-http":
        return (left == {"probe": name, "skipped": True}
                and {key: value for key, value in right.items() if key != "probe"} ==
                {key: value for key, value in legacy_http.items() if key != "probe"}
                and right.get("status") == 200 and right.get("contentType") == "image/png")
    if name == "pdf-image-storage":
        return (left == {"probe": name, "exists": False}
                and {key: value for key, value in right.items() if key != "probe"} ==
                {key: value for key, value in legacy_storage.items() if key != "probe"}
                and right.get("exists") is True and right.get("format") == "PNG"
                and right.get("size") == [800, 999])
    return False


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture-only", type=Path,
                        help="Create one PDF fixture at this path, then exit for visual QA")
    parser.add_argument("--fixture", type=Path,
                        help="Reuse an inspected one-page PDF fixture")
    args = parser.parse_args()
    if args.fixture_only:
        make_fixture(args.fixture_only)
        print(args.fixture_only.resolve())
        return
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    run_root = ROOT / ".tools" / ("pdf-reading-diff-" + secrets.token_hex(8))
    run_root.mkdir(parents=True)
    fixture = args.fixture or run_root / "fixture/one-page.pdf"
    if args.fixture is None:
        make_fixture(fixture)
    if len(PdfReader(str(fixture)).pages) != 1:
        raise AssertionError("Expected a one-page PDF fixture")
    username = "pdfprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    original = run_one(ORIGINAL, run_root / "original", username, password, fixture)
    restored = run_one(RESTORED, run_root / "restored", username, password, fixture)
    if [row["probe"] for row in original] != [row["probe"] for row in restored]:
        raise AssertionError("Original and restored PDF probe sequences differ")
    legacy_http = next(row for row in original if row["probe"] == "legacy-image-http")
    legacy_storage = next(row for row in original if row["probe"] == "legacy-image-storage")
    if (legacy_http.get("status") != 200 or legacy_http.get("contentType") != "image/png"
            or legacy_storage.get("exists") is not True):
        raise AssertionError("The original JAR did not render the legacy directory layout")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "acceptedDivergence": left != right and accepted_direct_pdf_fix(
                        left, right, username, run_root, legacy_http, legacy_storage),
                    "original": left, "restored": right}
                   for left, right in zip(original, restored)]
    report = {"originalJarSha256": sha256(ORIGINAL),
              "restoredJarSha256": sha256(RESTORED),
              "fixtureSha256": sha256(fixture), "comparisons": comparisons}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'reviewed fix' if row['acceptedDivergence'] else 'UNKNOWN DIFFERENCE'}")
    print(f"Report: {REPORT}")
    print(f"Isolated data: {run_root}")
    if not all(row["equal"] or row["acceptedDivergence"] for row in comparisons):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
