#!/usr/bin/env python3
"""Read-only live public-source discovery differential in disposable Reader data dirs.

No production storage, private source configuration, user credential, or remote
WebView service is used. The public page can change between requests, so the
script reports evidence rather than treating every content mismatch as a bug.
"""

import hashlib
import importlib.util
import json
from pathlib import Path
import secrets
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "import_base", Path(__file__).with_name("compare-import-entrypoints.py"))
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
SOURCE_URL = "https://m.jjjxsw.com"
PAGE_URL = SOURCE_URL + "/txt/"
SOURCE = {
    "bookSourceUrl": SOURCE_URL,
    "bookSourceName": "Public live discovery probe",
    "exploreUrl": PAGE_URL,
    "ruleExplore": {
        "bookList": ".booklist_a .list_a",
        "name": ".main strong@text",
        "author": ".main span a@text",
        "bookUrl": ".main a@href",
    },
    "ruleToc": {},
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(block)
    return checksum.hexdigest()


def upstream_snapshot():
    request = urllib.request.Request(
        PAGE_URL, headers={"User-Agent": "ReaderPublicCompatibilityProbe/1.0"})
    with urllib.request.urlopen(request, timeout=20) as response:
        body = response.read(4 * 1024 * 1024 + 1)
        if len(body) > 4 * 1024 * 1024:
            raise ValueError("Public page exceeded 4 MiB probe limit")
        return {"status": response.status, "sha256": digest(body),
                "length": len(body)}


def summary(result):
    body = result.get("body") or {}
    books = body.get("data")
    output = {"status": result["status"], "isSuccess": body.get("isSuccess"),
              "errorMsg": body.get("errorMsg")}
    if isinstance(books, list):
        canonical = json.dumps(books, sort_keys=True, ensure_ascii=False,
                               separators=(",", ":")).encode("utf-8")
        output.update({"bookCount": len(books), "dataSha256": digest(canonical),
                       "firstBookKeys": sorted(books[0]) if books else [],
                       "booksWithUrl": sum(bool(book.get("bookUrl")) for book in books)})
    else:
        output["dataType"] = type(books).__name__
    return output


def run_one(jar, workdir, username, password):
    port = BASE.free_port()
    base = f"http://127.0.0.1:{port}"
    workdir.mkdir(parents=True, exist_ok=True)
    with (workdir / "reader.log").open("wb") as logfile:
        process = subprocess.Popen(
            [str(BASE.JAVA), "-Xms128m", "-Xmx768m", "-jar", str(jar),
             f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
             "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
             "--spring.profiles.active=prod"],
            cwd=ROOT, stdout=logfile, stderr=subprocess.STDOUT,
            creationflags=(subprocess.CREATE_NO_WINDOW
                           if hasattr(subprocess, "CREATE_NO_WINDOW") else 0))
        session = BASE.client()
        try:
            deadline = time.monotonic() + 70
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"Reader exited during startup: {process.returncode}")
                try:
                    if BASE.send(session, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                        break
                except (OSError, urllib.error.URLError):
                    time.sleep(0.5)
            else:
                raise TimeoutError("Reader startup timed out")

            for is_login in (False, True):
                BASE.expect(BASE.send(session, base, "POST", "/reader3/login",
                                      {"username": username, "password": password,
                                       "isLogin": is_login}), True, "isolated login")
            BASE.expect(BASE.send(session, base, "POST", "/reader3/saveBookSource",
                                  SOURCE), True, "save public source")
            result = BASE.send(session, base, "POST", "/reader3/exploreBook",
                               {"bookSourceUrl": SOURCE_URL, "ruleFindUrl": PAGE_URL,
                                "page": 1}, timeout=45)
            return summary(result)
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def main():
    for path in (BASE.JAVA, BASE.ORIGINAL, BASE.RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    username = "publicprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-public-explore-") as temp:
        root = Path(temp)
        before = upstream_snapshot()
        original = run_one(BASE.ORIGINAL, root / "original", username, password)
        restored = run_one(BASE.RESTORED, root / "restored", username, password)
        after = upstream_snapshot()
    report = {
        "source": PAGE_URL,
        "originalJarSha256": file_digest(BASE.ORIGINAL),
        "restoredJarSha256": file_digest(BASE.RESTORED),
        "upstreamBefore": before,
        "upstreamAfter": after,
        "upstreamByteStable": before == after,
        "original": original,
        "restored": restored,
        "sameEnvelope": all(original.get(key) == restored.get(key)
                            for key in ("status", "isSuccess", "errorMsg")),
        "sameDataDigest": original.get("dataSha256") == restored.get("dataSha256"),
    }
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if not (original["status"] == restored["status"] == 200
            and original["isSuccess"] is restored["isSuccess"] is True
            and original.get("bookCount", 0) > 0 and restored.get("bookCount", 0) > 0):
        raise RuntimeError("Public source was not successfully parsed by both JARs")
    if before != after:
        raise RuntimeError("Public page changed; the live differential is inconclusive")
    if not report["sameEnvelope"] or not report["sameDataDigest"]:
        raise RuntimeError("Original and restored JARs differ on a stable public page")


if __name__ == "__main__":
    from original_jar_safety import require_original_jar_isolation

    require_original_jar_isolation()
    main()
