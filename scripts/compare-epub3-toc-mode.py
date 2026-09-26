#!/usr/bin/env python3
"""Compare explicit EPUB 3 TOC mode against spine mode using nested navigation."""

import hashlib
import importlib.util
import json
import secrets
import subprocess
import tempfile
import time
import urllib.error
from pathlib import Path
from xml.etree import ElementTree
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("epub3_diff", Path(__file__).with_name("compare-epub3-reading.py"))
EPUB3 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EPUB3)
BASE = EPUB3.BASE
REPORT = ROOT / "reports/epub3-toc-mode-diff-latest.json"
NESTED_NAV = '''<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head><title>目录</title></head><body><nav epub:type="toc"><ol>
    <li><a href="Text/two.xhtml">导航第二章</a><ol>
      <li><a href="Text/one.xhtml">导航第一章</a></li>
    </ol></li>
  </ol></nav></body>
</html>'''.encode("utf-8")


def fixture(path):
    base = path.with_name("flat.epub")
    EPUB3.fixture(base)
    with ZipFile(base) as source, ZipFile(path, "w") as target:
        for entry in source.infolist():
            data = NESTED_NAV if entry.filename == "OEBPS/nav.xhtml" else source.read(entry)
            target.writestr(entry, data)
    ElementTree.fromstring(NESTED_NAV)


def run(jar, workdir, data, username, password):
    legacy_url = f"storage/data/{username}/reading/legacy.epub"
    source = workdir / legacy_url / "index.epub"
    source.parent.mkdir(parents=True)
    source.write_bytes(data)
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    log = (workdir / "reader.log").open("w", encoding="utf-8")
    process = subprocess.Popen(
        [str(BASE.JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
    )
    session = BASE.client()
    result = {}
    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited during startup: {process.returncode}")
            try:
                if BASE.send(session, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")
        for is_login in (False, True):
            BASE.expect(BASE.send(session, base, "POST", "/reader3/login",
                                  {"username": username, "password": password,
                                   "isLogin": is_login}), True, "login")
        for mode in ("toc", "spin"):
            book = {"bookUrl": legacy_url, "originName": legacy_url,
                    "origin": "loc_book", "name": "epub3 nested", "author": "",
                    "type": 0, "tocUrl": mode}
            saved = BASE.send(session, base, "POST", "/reader3/saveBook", book)
            BASE.expect(saved, True, f"save-{mode}")
            chapters = BASE.send(session, base, "GET",
                                 f"/reader3/getChapterList?url={legacy_url}&refresh=1")
            body = chapters.get("body", {})
            rows = body.get("data", []) if isinstance(body.get("data"), list) else []
            result[mode] = {"status": chapters.get("status"),
                            "isSuccess": body.get("isSuccess"),
                            "errorMsg": body.get("errorMsg"),
                            "chapters": [{"index": row.get("index"),
                                          "title": row.get("title"),
                                          "url": row.get("url")} for row in rows]}
        return result
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        log.close()


def main():
    with tempfile.TemporaryDirectory(prefix="reader-epub3-toc-") as temp:
        root = Path(temp)
        source = root / "nested.epub"
        fixture(source)
        data = source.read_bytes()
        username = "tocprobe" + secrets.token_hex(5)
        password = "Probe-" + secrets.token_hex(18)
        original = run(BASE.ORIGINAL, root / "original", data, username, password)
        restored = run(BASE.RESTORED, root / "restored", data, username, password)
    report = {"originalJarSha256": BASE.digest(BASE.ORIGINAL),
              "restoredJarSha256": BASE.digest(BASE.RESTORED),
              "fixtureSha256": hashlib.sha256(data).hexdigest(),
              "original": original, "restored": restored,
              "equal": original == restored}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    expected = {
        "toc": [{"index": 0, "title": "导航第二章", "url": "Text/two.xhtml"},
                {"index": 1, "title": "导航第一章", "url": "Text/one.xhtml"}],
        "spin": [{"index": 0, "title": "导航第一章", "url": "Text/one.xhtml"},
                 {"index": 1, "title": "导航第二章", "url": "Text/two.xhtml"}],
    }
    for label, value in (("original", original), ("restored", restored)):
        for mode, chapters in expected.items():
            if value[mode] != {"status": 200, "isSuccess": True,
                               "errorMsg": "", "chapters": chapters}:
                raise AssertionError(f"{label}: {mode} did not return the expected chapter order")
    if not report["equal"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
