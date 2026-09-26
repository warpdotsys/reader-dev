#!/usr/bin/env python3
"""Compare isolated backup ZIP and restore behavior with the original JAR.

Only generated accounts and synthetic RSS/rule data are used. Both JARs run
locally in separate disposable directories; no production backup is restored.
"""

import hashlib
import http.cookiejar
import io
import json
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import zipfile


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/backup-lifecycle-diff-latest.json"


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def zip_entries(data):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        result = []
        for info in sorted(archive.infolist(), key=lambda entry: entry.filename):
            if info.is_dir():
                continue
            raw = archive.read(info.filename)
            item = {"path": info.filename, "length": len(raw),
                    "sha256": hashlib.sha256(raw).hexdigest()}
            if info.filename in {"rssSources.json", "replaceRule.json"}:
                item["document"] = json.loads(raw.decode("utf-8"))
            result.append(item)
        return result


def semantic_entries(entries):
    """Preserve byte checks for opaque files; JSON object member order is not data."""
    return [{"path": item["path"], "length": item["length"],
             "content": item.get("document", item["sha256"])} for item in entries]


def semantic_response(response):
    if "zipEntries" not in response:
        return response
    return {key: value for key, value in response.items()
            if key not in {"length", "zipEntries"}} | {
                "zipEntries": semantic_entries(response["zipEntries"])}


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
        result = {"status": response.status, "contentType": content_type}
        if "json" in content_type:
            result["body"] = json.loads(raw.decode("utf-8"))
        else:
            result["length"] = len(raw)
            result["disposition"] = response.headers.get("Content-Disposition", "")
            result["cacheControl"] = response.headers.get("Cache-Control", "")
            if raw.startswith(b"PK\x03\x04"):
                result["zipEntries"] = zip_entries(raw)
            else:
                result["sha256"] = hashlib.sha256(raw).hexdigest()
        return result


def expect(response, success, name, count=None):
    body = response.get("body", {})
    if response["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{name}: unexpected response: {response}")
    data = body.get("data")
    if count is not None and len(data) != count:
        raise AssertionError(f"{name}: expected {count} items, got {data}")
    return data


def local_backup_files(workdir, username):
    user_dir = workdir / "storage/data" / username
    webdav_backup = sorted((user_dir / "webdav/legado").glob("backup*.zip"))
    download_backup = sorted((user_dir / "backup").glob("backup*.zip"))
    if len(webdav_backup) != 1 or len(download_backup) != 1:
        raise AssertionError("Expected exactly one WebDAV and one download backup ZIP")
    return webdav_backup[0], download_backup[0]


def run_one(jar, workdir, username, other_user, password):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.defaultUserEnableWebdav=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    anonymous, owner, other = client(), client(), client()
    probes = []

    def add(name, user, method, path, payload=None):
        result = request(user, base, method, path, payload)
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

        add("anonymous-download", anonymous, "GET", "/reader3/user/downloadBackupFile")
        add("anonymous-webdav-backup", anonymous, "POST", "/reader3/backupToWebdav", {})
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                response = request(session, base, "POST", "/reader3/login",
                                   {"username": name, "password": password, "isLogin": login})
                expect(response, True, "login")

        rss = {"sourceUrl": "https://fixture.invalid/feed", "sourceName": "Backup RSS"}
        rule = {"id": 1700000000001, "name": "Backup rule",
                "pattern": "old", "replacement": "new"}
        expect(add("save-rss", owner, "POST", "/reader3/saveRssSource", rss), True, "save-rss")
        expect(add("save-rule", owner, "POST", "/reader3/saveReplaceRule", rule), True, "save-rule")
        rss_before = expect(add("rss-before", owner, "GET", "/reader3/getRssSources"),
                            True, "rss-before", 1)
        rule_before = expect(add("rule-before", owner, "GET", "/reader3/getReplaceRules"),
                             True, "rule-before", 1)
        expect(add("mongodb-backup-unconfigured", owner, "POST", "/reader3/backupToMongodb", {}),
               False, "mongodb-backup-unconfigured")
        expect(add("mongodb-restore-unconfigured", owner, "POST", "/reader3/restoreFromMongodb", {}),
               False, "mongodb-restore-unconfigured")
        expect(add("webdav-backup", owner, "POST", "/reader3/backupToWebdav", {}),
               True, "webdav-backup")

        downloaded = add("download-backup", owner, "GET", "/reader3/user/downloadBackupFile")
        if downloaded["status"] != 200 or not downloaded.get("zipEntries"):
            raise AssertionError("Backup download did not return a readable ZIP")
        webdav_file, download_file = local_backup_files(workdir, username)
        webdav_entries = zip_entries(webdav_file.read_bytes())
        download_entries = zip_entries(download_file.read_bytes())
        expected_names = {"rssSources.json", "replaceRule.json"}
        if not expected_names.issubset({item["path"] for item in download_entries}):
            raise AssertionError(f"Backup ZIP lacks synthetic data: {download_entries}")
        if downloaded["zipEntries"] != download_entries:
            raise AssertionError("Downloaded ZIP entries differ from the server-side file")
        user_dir = workdir / "storage/data" / username
        source_hashes = {
            name: hashlib.sha256((user_dir / name).read_bytes()).hexdigest()
            for name in expected_names
        }
        for entries in (webdav_entries, download_entries):
            if {item["path"]: item["sha256"] for item in entries} != source_hashes:
                raise AssertionError("Backup ZIP contents differ from the source JSON files")

        expect(add("delete-rss", owner, "POST", "/reader3/deleteRssSource", rss),
               True, "delete-rss")
        expect(add("delete-rule", owner, "POST", "/reader3/deleteReplaceRule", rule),
               True, "delete-rule")
        expect(add("rss-empty", owner, "GET", "/reader3/getRssSources"), True, "rss-empty", 0)
        expect(add("rule-empty", owner, "GET", "/reader3/getReplaceRules"), True, "rule-empty", 0)
        expect(add("restore-invalid-extension", owner, "POST", "/reader3/file/restore",
                   {"home": "__HOME__", "path": "/missing.txt"}),
               False, "restore-invalid-extension")
        expect(add("restore-missing-zip", owner, "POST", "/reader3/file/restore",
                   {"home": "__HOME__", "path": "/missing.zip"}),
               False, "restore-missing-zip")
        restore_path = "/backup/" + download_file.name
        expect(add("restore-backup", owner, "POST", "/reader3/file/restore",
                   {"home": "__HOME__", "path": restore_path}), True, "restore-backup")
        rss_after = expect(add("rss-after", owner, "GET", "/reader3/getRssSources"),
                           True, "rss-after", 1)
        rule_after = expect(add("rule-after", owner, "GET", "/reader3/getReplaceRules"),
                            True, "rule-after", 1)
        if rss_after != rss_before or rule_after != rule_before:
            raise AssertionError("Restored RSS/rule responses differ from their pre-backup values")
        expect(add("other-user-rss", other, "GET", "/reader3/getRssSources"),
               True, "other-user-rss", 0)
        expect(add("other-user-rule", other, "GET", "/reader3/getReplaceRules"),
               True, "other-user-rule", 0)

        storage = {name: json.loads((user_dir / name).read_text(encoding="utf-8"))
                   for name in expected_names}
        if len(storage["rssSources.json"]) != 1 or len(storage["replaceRule.json"]) != 1:
            raise AssertionError("Restored storage does not contain both synthetic entries")
        return {"probes": probes, "webdavEntries": webdav_entries,
                "downloadEntries": download_entries, "storage": storage}
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
    username = "backupprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-backup-diff-") as temp:
        root = Path(temp)
        original = run_one(ORIGINAL, root / "original", username, other_user, password)
        restored = run_one(RESTORED, root / "restored", username, other_user, password)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("probe count differs")
    comparisons = [{"probe": left["probe"],
                    "equal": semantic_response(left) == semantic_response(right),
                    "rawMetadataEqual": left == right,
                    "original": left, "restored": right}
                   for left, right in zip(original["probes"], restored["probes"])]
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons,
              "webdavEntriesEqual": semantic_entries(original["webdavEntries"]) == semantic_entries(restored["webdavEntries"]),
              "downloadEntriesEqual": semantic_entries(original["downloadEntries"]) == semantic_entries(restored["downloadEntries"]),
              "webdavEntriesRawEqual": original["webdavEntries"] == restored["webdavEntries"],
              "downloadEntriesRawEqual": original["downloadEntries"] == restored["downloadEntries"],
              "storageEqual": original["storage"] == restored["storage"],
              "archives": {"original": {"webdav": original["webdavEntries"],
                                        "download": original["downloadEntries"]},
                           "restored": {"webdav": restored["webdavEntries"],
                                        "download": restored["downloadEntries"]}},
              "storage": {"original": original["storage"], "restored": restored["storage"]}}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'DIFFERENT'}")
    for key in ("webdavEntriesEqual", "downloadEntriesEqual", "storageEqual"):
        print(f"{key}: {report[key]}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] for row in comparisons) or not all(
            report[key] for key in ("webdavEntriesEqual", "downloadEntriesEqual", "storageEqual")):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
