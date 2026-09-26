#!/usr/bin/env python3
"""Compare PDF width configuration, rendered bytes, and user isolation.

Generates a deterministic one-page PDF fixture and separate temporary storage.
No production files or accounts are read or modified.
"""

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


ROOT = Path(__file__).resolve().parents[1]
BASE_SPEC = importlib.util.spec_from_file_location(
    "pdf_reading_base", Path(__file__).with_name("compare-pdf-reading.py"))
BASE = importlib.util.module_from_spec(BASE_SPEC)
BASE_SPEC.loader.exec_module(BASE)
REPORT = ROOT / "reports/pdf-book-config-diff-latest.json"


def image_record(path, name):
    if not path.is_file():
        return {"probe": name, "exists": False}
    with Image.open(path) as image:
        return {"probe": name, "exists": True, "format": image.format,
                "size": list(image.size), "sha256": BASE.sha256(path)}


def accepted_missing_url_fix(left, right):
    common = {"probe": "missing-url", "status": 200,
              "contentType": "application/json; charset=utf-8"}
    return (left == {**common, "body": {"isSuccess": False,
                                       "errorMsg": "java.lang.NullPointerException: context.bodyAsJson.getString(\"bookUrl\") must not be null"}}
            and right == {**common, "body": {"isSuccess": False,
                                             "errorMsg": "书籍链接不能为空"}})


def run_one(jar, workdir, username, other_user, password, fixture):
    legacy_dir = workdir / "storage/data" / username / "reading/config.pdf"
    legacy_dir.mkdir(parents=True)
    shutil.copyfile(fixture, legacy_dir / "index.pdf")
    book_url = f"storage/data/{username}/reading/config.pdf"
    escaped = urllib.parse.quote(book_url, safe="")
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    probes = []
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(
            [str(BASE.JAVA), "-jar", str(jar),
             f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
             "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
             "--spring.profiles.active=prod"], cwd=ROOT, stdout=log,
            stderr=subprocess.STDOUT)
        anonymous, owner, other = BASE.client(), BASE.client(), BASE.client()

        def add(name, session, method, path, payload=None):
            row = {"probe": name, **BASE.request(session, base, method, path, payload)}
            probes.append(row)
            return row

        def render(name, refresh=False):
            content = add(name + "-content", owner, "GET",
                          "/reader3/getBookContent?url=" + escaped + "&index=0" +
                          ("&refresh=1" if refresh else ""))
            html = content.get("body", {}).get("data", "")
            match = re.fullmatch(
                rf"<img src='__API_ROOT__(/book-assets/{username}/reading/config\.pdf/index/output-0\.png)' />",
                html) if isinstance(html, str) else None
            if match:
                add(name + "-image-http", owner, "GET", match.group(1))
            else:
                probes.append({"probe": name + "-image-http", "skipped": True})
            probes.append(image_record(legacy_dir / "index/output-0.png",
                                       name + "-image-storage"))

        try:
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"Reader exited during startup: {process.returncode}")
                try:
                    if BASE.request(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                        break
                except OSError:
                    pass
                time.sleep(0.4)
            else:
                raise TimeoutError("Reader startup timeout")
            add("anonymous-save-config", anonymous, "POST",
                "/reader3/book/saveBookConfig",
                {"bookUrl": book_url, "pdfImageWidth": 400})
            for session, name in ((owner, username), (other, other_user)):
                for is_login in (False, True):
                    result = BASE.request(session, base, "POST", "/reader3/login",
                                          {"username": name, "password": password,
                                           "isLogin": is_login})
                    if result["status"] != 200 or result["body"]["isSuccess"] is not True:
                        raise AssertionError("Fixture account login failed")
            add("missing-url", owner, "POST", "/reader3/book/saveBookConfig",
                {"pdfImageWidth": 400})
            add("unknown-book", owner, "POST", "/reader3/book/saveBookConfig",
                {"bookUrl": "missing", "pdfImageWidth": 400})
            add("save-legacy-book", owner, "POST", "/reader3/saveBook",
                {"bookUrl": book_url, "originName": book_url, "origin": "loc_book",
                 "name": "config", "author": "", "type": 0})
            add("initial-shelf", owner, "GET", "/reader3/getBookshelf")
            add("invalid-zero", owner, "POST", "/reader3/book/saveBookConfig",
                {"bookUrl": book_url, "pdfImageWidth": 0})
            add("invalid-negative", owner, "POST", "/reader3/book/saveBookConfig",
                {"bookUrl": book_url, "pdfImageWidth": -1})
            add("other-user-denied", other, "POST", "/reader3/book/saveBookConfig",
                {"bookUrl": book_url, "pdfImageWidth": 400})
            add("other-user-shelf", other, "GET", "/reader3/getBookshelf")
            add("save-width-400", owner, "POST", "/reader3/book/saveBookConfig",
                {"bookUrl": book_url, "pdfImageWidth": 400})
            add("shelf-width-400", owner, "GET", "/reader3/getBookshelf")
            add("chapter-list", owner, "GET", "/reader3/getChapterList?url=" + escaped)
            render("width-400")
            add("save-width-1200", owner, "POST", "/reader3/book/saveBookConfig",
                {"bookUrl": book_url, "pdfImageWidth": 1200})
            add("shelf-width-1200", owner, "GET", "/reader3/getBookshelf")
            render("width-1200", refresh=True)
            shelf_file = workdir / "storage/data" / username / "bookshelf.json"
            probes.append({"probe": "bookshelf-storage", "exists": shelf_file.is_file(),
                           "data": BASE.normalize(json.loads(shelf_file.read_text(encoding="utf-8")))
                           if shelf_file.is_file() else None})
            return probes
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def main():
    for path in (BASE.JAVA, BASE.ORIGINAL, BASE.RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    run_root = ROOT / ".tools" / ("pdf-book-config-diff-" + secrets.token_hex(8))
    run_root.mkdir(parents=True)
    fixture = run_root / "fixture/one-page.pdf"
    BASE.make_fixture(fixture)
    username = "pdfconfig" + secrets.token_hex(5)
    other_user = "pdfother" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    left = run_one(BASE.ORIGINAL, run_root / "original", username, other_user,
                   password, fixture)
    right = run_one(BASE.RESTORED, run_root / "restored", username, other_user,
                    password, fixture)
    if [row["probe"] for row in left] != [row["probe"] for row in right]:
        raise AssertionError("PDF config probe sequences differ")
    comparisons = [{"probe": a["probe"], "equal": a == b,
                    "acceptedMissingUrlFix": a != b and accepted_missing_url_fix(a, b),
                    "original": a, "restored": b} for a, b in zip(left, right)]
    report = {"originalJarSha256": BASE.sha256(BASE.ORIGINAL),
              "restoredJarSha256": BASE.sha256(BASE.RESTORED),
              "fixtureSha256": BASE.sha256(fixture), "comparisons": comparisons}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        outcome = ('equal' if row['equal'] else 'reviewed fix' if row['acceptedMissingUrlFix']
                   else 'UNKNOWN DIFFERENCE')
        print(f"{row['probe']}: {outcome}")
    print(f"Report: {REPORT}")
    print(f"Isolated data: {run_root}")
    if not all(row["equal"] or row["acceptedMissingUrlFix"] for row in comparisons):
        raise SystemExit(1)
    by_name = {row["probe"]: row["restored"] for row in comparisons}
    if not (by_name["width-400-image-storage"]["size"] == [400, 499]
            and by_name["width-1200-image-storage"]["size"] == [1200, 1500]
            and by_name["width-400-image-storage"]["sha256"]
            != by_name["width-1200-image-storage"]["sha256"]):
        raise AssertionError("PDF width setting did not change rendered dimensions")
    for width in (400, 1200):
        if (by_name[f"save-width-{width}"]["body"]["data"]["readConfig"]["pdfImageWidth"] != width
                or by_name[f"shelf-width-{width}"]["body"]["data"][0]["readConfig"]["pdfImageWidth"] != width
                or by_name[f"width-{width}-image-http"]["sha256"]
                != by_name[f"width-{width}-image-storage"]["sha256"]):
            raise AssertionError(f"PDF width {width} response, shelf or image bytes differ")
    if (by_name["bookshelf-storage"]["data"][0]["readConfig"]["pdfImageWidth"] != 1200
            or by_name["other-user-shelf"]["body"]["data"] != []
            or by_name["other-user-denied"]["body"]["errorMsg"] != "书籍信息错误"):
        raise AssertionError("PDF width storage or user isolation failed")


if __name__ == "__main__":
    main()
