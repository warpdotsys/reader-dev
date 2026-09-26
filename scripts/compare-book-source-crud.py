#!/usr/bin/env python3
"""Compare isolated book-source CRUD against reader-pro-3.2.14.

The fixtures use .invalid URLs and are never fetched. Both servers use separate
temporary work directories and generated users; no production data is touched.
"""

import hashlib
import http.cookiejar
import json
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/book-source-crud-diff-latest.json"


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def request(opener, base, method, path, payload=None):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=25)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read()
        content_type = response.headers.get("Content-Type", "")
        if "json" not in content_type:
            return {"status": response.status, "contentType": content_type,
                    "length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        return {"status": response.status, "contentType": content_type,
                "body": json.loads(raw.decode("utf-8"))}


def expect(result, success, name, count=None):
    body = result.get("body", {})
    if result["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{name}: unexpected response: {result}")
    data = body.get("data")
    if count is not None and len(data) != count:
        raise AssertionError(f"{name}: expected {count} rows, got {data}")
    return data


def source(suffix, name, explore=False):
    result = {"bookSourceUrl": f"https://fixture.invalid/{suffix}",
              "bookSourceName": name, "bookSourceGroup": "Fixture"}
    if explore:
        result["exploreUrl"] = "https://fixture.invalid/explore"
    return result


def run_one(jar, workdir, username, other_user, password, secure_key):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         f"--reader.app.secureKey={secure_key}",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    anonymous, owner, other = client(), client(), client()
    probes = []

    def add(name, session, method, path, payload=None):
        result = request(session, base, method, path, payload)
        probes.append({"probe": name, **result})
        return result

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited during startup: {process.returncode}")
            try:
                if request(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")

        first = source("one", "One", explore=True)
        second = source("two", "Two")
        third = source("three", "Three")
        expect(add("anonymous-save-multi", anonymous, "POST", "/reader3/saveBookSources",
                   [first]), False, "anonymous-save-multi")
        expect(add("anonymous-delete-all", anonymous, "POST", "/reader3/deleteAllBookSources",
                   {}), False, "anonymous-delete-all")
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                expect(request(session, base, "POST", "/reader3/login",
                               {"username": name, "password": password, "isLogin": login}),
                       True, "login")

        expect(add("empty-get", owner, "GET", "/reader3/getBookSources"),
               True, "empty-get", 0)
        expect(add("save-two", owner, "POST", "/reader3/saveBookSources",
                   [first, second]), True, "save-two")
        initial = expect(add("get-two", owner, "GET", "/reader3/getBookSources"),
                         True, "get-two", 2)
        if [row["bookSourceUrl"] for row in initial] != [first["bookSourceUrl"], second["bookSourceUrl"]]:
            raise AssertionError("book sources were not saved in input order")
        simple = expect(add("get-simple", owner, "GET", "/reader3/getBookSources?simple=1"),
                        True, "get-simple", 2)
        if any(set(row) != {"bookSourceUrl", "bookSourceName", "bookSourceGroup", "exploreUrl"}
               for row in simple):
            raise AssertionError("simple source projection contains unexpected fields")
        expect(add("post-get", owner, "POST", "/reader3/getBookSources",
                   {"simple": 0}), True, "post-get", 2)
        expect(add("get-single", owner, "GET", "/reader3/getBookSource?bookSourceUrl="
                   + urllib.parse.quote(first["bookSourceUrl"], safe="")), True, "get-single")
        expect(add("other-user-empty", other, "GET", "/reader3/getBookSources"),
               True, "other-user-empty", 0)

        updated = dict(first, bookSourceName="One updated")
        expect(add("upsert-and-add", owner, "POST", "/reader3/saveBookSources",
                   [updated, third]), True, "upsert-and-add")
        after_upsert = expect(add("get-after-upsert", owner, "GET", "/reader3/getBookSources"),
                              True, "get-after-upsert", 3)
        if after_upsert[0]["bookSourceName"] != "One updated":
            raise AssertionError("bulk upsert did not replace the existing source")
        expect(add("delete-one", owner, "POST", "/reader3/deleteBookSource",
                   {"bookSourceUrl": second["bookSourceUrl"]}), True, "delete-one")
        expect(add("get-after-delete-one", owner, "GET", "/reader3/getBookSources"),
               True, "get-after-delete-one", 2)
        expect(add("delete-multi", owner, "POST", "/reader3/deleteBookSources",
                   [first, third]), True, "delete-multi")
        expect(add("get-after-delete-multi", owner, "GET", "/reader3/getBookSources"),
               True, "get-after-delete-multi", 0)
        expect(add("save-again", owner, "POST", "/reader3/saveBookSources",
                   [first, second]), True, "save-again")
        expect(add("delete-all", owner, "POST", "/reader3/deleteAllBookSources",
                   {}), True, "delete-all")
        expect(add("get-after-delete-all", owner, "GET", "/reader3/getBookSources"),
               True, "get-after-delete-all", 0)
        expect(add("save-for-file-delete", owner, "POST", "/reader3/saveBookSources",
                   [first]), True, "save-for-file-delete")
        expect(add("delete-source-file", owner, "POST", "/reader3/deleteBookSourcesFile",
                   {}), True, "delete-source-file")
        expect(add("get-after-file-delete", owner, "GET", "/reader3/getBookSources"),
               True, "get-after-file-delete", 0)
        expect(add("manager-default-denied", owner, "POST", "/reader3/setAsDefaultBookSources",
                   {"username": username}), False, "manager-default-denied")
        expect(add("manager-user-delete-denied", owner, "POST", "/reader3/deleteUserBookSource",
                   [username]), False, "manager-user-delete-denied")
        expect(add("other-user-still-empty", other, "GET", "/reader3/getBookSources"),
               True, "other-user-still-empty", 0)

        expect(add("save-for-default", owner, "POST", "/reader3/saveBookSources",
                   [first]), True, "save-for-default")
        manager_path = "?secureKey=" + urllib.parse.quote(secure_key, safe="")
        expect(add("manager-set-default", owner, "POST",
                   "/reader3/setAsDefaultBookSources" + manager_path,
                   {"username": username}), True, "manager-set-default")
        inherited = expect(add("other-user-default", other, "GET", "/reader3/getBookSources"),
                           True, "other-user-default", 1)
        if inherited[0]["bookSourceUrl"] != first["bookSourceUrl"]:
            raise AssertionError("other user did not inherit the new default source")
        expect(add("manager-delete-user-file", owner, "POST",
                   "/reader3/deleteUserBookSource" + manager_path,
                   [username]), True, "manager-delete-user-file")
        expect(add("owner-fallback-default", owner, "GET", "/reader3/getBookSources"),
               True, "owner-fallback-default", 1)

        user_dir = workdir / "storage/data" / username
        source_file = user_dir / "bookSource.json"
        default_file = workdir / "storage/data/default/bookSource.json"
        if not default_file.exists():
            raise AssertionError("authorized default source was not persisted")
        return {"probes": probes, "sourceFileExists": source_file.exists(),
                "sourceStorage": json.loads(source_file.read_text(encoding="utf-8"))
                if source_file.exists() else None,
                "defaultStorage": json.loads(default_file.read_text(encoding="utf-8"))}
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main():
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    username = "sourceprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    secure_key = "ManagerProbe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-source-crud-diff-") as temp:
        root = Path(temp)
        original = run_one(ORIGINAL, root / "original", username, other_user, password, secure_key)
        restored = run_one(RESTORED, root / "restored", username, other_user, password, secure_key)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("probe count differs")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "original": left, "restored": right}
                   for left, right in zip(original["probes"], restored["probes"])]
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons,
              "storageEqual": original["sourceStorage"] == restored["sourceStorage"],
              "defaultStorageEqual": original["defaultStorage"] == restored["defaultStorage"],
              "sourceFileExistsEqual": original["sourceFileExists"] == restored["sourceFileExists"],
              "storage": {"original": original["sourceStorage"],
                          "restored": restored["sourceStorage"],
                          "defaultOriginal": original["defaultStorage"],
                          "defaultRestored": restored["defaultStorage"]}}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'DIFFERENT'}")
    print(f"storageEqual: {report['storageEqual']}")
    print(f"defaultStorageEqual: {report['defaultStorageEqual']}")
    print(f"sourceFileExistsEqual: {report['sourceFileExistsEqual']}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] for row in comparisons) or not all(
            report[key] for key in ("storageEqual", "defaultStorageEqual", "sourceFileExistsEqual")):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
