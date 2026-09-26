#!/usr/bin/env python3
"""Compare non-UTF-8 OPF metadata in the original and restored import preview."""

import hashlib
import importlib.util
import json
import secrets
import subprocess
import tempfile
import time
import urllib.error
from pathlib import Path
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("epub_diff", Path(__file__).with_name("compare-epub-reading.py"))
EPUB = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EPUB)
BASE = EPUB.BASE
REPORT = ROOT / "reports/epub-opf-encoding-diff-latest.json"


def make_fixture(path):
    ordinary = path.with_name("utf8.epub")
    EPUB.fixture(ordinary)
    opf = EPUB.OPF.decode("utf-8")
    opf = (opf.replace('encoding="UTF-8"', 'encoding="ISO-8859-1"')
              .replace("EPUB 差分测试书", "Café déjà vu")
              .replace("测试作者", "Señor Pérez")
              .replace("本地固定夹具", "Latin-1 metadata fixture"))
    with ZipFile(ordinary) as source, ZipFile(path, "w") as target:
        for entry in source.infolist():
            data = opf.encode("iso-8859-1") if entry.filename == "OEBPS/content.opf" else source.read(entry)
            target.writestr(entry, data)


def run(jar, workdir, payload, username, password):
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    workdir.mkdir(parents=True)
    log = (workdir / "reader.log").open("w", encoding="utf-8")
    process = subprocess.Popen(
        [str(BASE.JAVA), "-Dfile.encoding=GBK", "-jar", str(jar),
         f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
    )
    session = BASE.client()
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
        result = BASE.send(session, base, "POST", "/reader3/importBookPreview",
                           upload=("latin1.epub", payload))
        body = result.get("body", {})
        books = body.get("data", []) if isinstance(body.get("data"), list) else []
        book = books[0].get("book", {}) if books else {}
        return {"status": result.get("status"), "isSuccess": body.get("isSuccess"),
                "errorMsg": body.get("errorMsg"), "name": book.get("name"),
                "author": book.get("author"),
                "chapterCount": len(books[0].get("chapters", [])) if books else None}
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        log.close()


def main():
    with tempfile.TemporaryDirectory(prefix="reader-epub-opf-encoding-") as temp:
        root = Path(temp)
        fixture = root / "latin1.epub"
        make_fixture(fixture)
        payload = fixture.read_bytes()
        username = "opfprobe" + secrets.token_hex(5)
        password = "Probe-" + secrets.token_hex(18)
        original = run(BASE.ORIGINAL, root / "original", payload, username, password)
        restored = run(BASE.RESTORED, root / "restored", payload, username, password)
    report = {"originalJarSha256": BASE.digest(BASE.ORIGINAL),
              "restoredJarSha256": BASE.digest(BASE.RESTORED),
              "fixtureSha256": hashlib.sha256(payload).hexdigest(),
              "original": original, "restored": restored}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    expected_original = {"status": 200, "isSuccess": True, "errorMsg": "",
                         "name": "Caf? déj? vu", "author": "Pérez, Señor", "chapterCount": 2}
    expected_restored = {**expected_original, "name": "Café déjà vu"}
    if original != expected_original or restored != expected_restored:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
