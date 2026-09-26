#!/usr/bin/env python3
"""Compare upload preview and loopback remote-source import in disposable workdirs."""

import hashlib
import http.cookiejar
import json
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/import-entrypoints-diff-latest.json"
DYNAMIC_FIELDS = {"latestChapterTime", "lastCheckTime", "durChapterTime"}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


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
        result = {}
        for key, item in value.items():
            if key in DYNAMIC_FIELDS:
                if not isinstance(item, int):
                    raise AssertionError(f"{key} must be numeric")
                result[key] = "<timestamp>"
            else:
                result[key] = normalize(item)
        return result
    return value


def send(opener, base, method, path, payload=None, upload=None, timeout=25):
    if upload is not None:
        filename, content = upload
        boundary = "reader-probe-" + secrets.token_hex(8)
        data = (f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; "
                f"filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n").encode()
        data += content + f"\r\n--{boundary}--\r\n".encode()
        headers = {"Content-Type": f"multipart/form-data; boundary={boundary}"}
    else:
        data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
        headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    request = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(request, timeout=timeout)
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


def expect(result, success, name):
    if result["status"] != 200 or result.get("body", {}).get("isSuccess") is not success:
        raise AssertionError(f"{name}: {result}")
    return result["body"].get("data")


def run_one(jar, workdir, username, other_user, password, fixture_url, txt):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    anonymous, owner, other = client(), client(), client()
    probes = []

    def add(name, session, method, path, payload=None, upload=None, timeout=25):
        try:
            result = send(session, base, method, path, payload, upload, timeout)
        except TimeoutError:
            result = {"timeoutAfterSeconds": timeout}
        probes.append({"probe": name, **result})
        return result

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited: {process.returncode}")
            try:
                if send(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")

        path = "/reader3/importBookPreview"
        remote = "/reader3/saveFromRemoteSource"
        expect(add("anonymous-preview", anonymous, "POST", path,
                   upload=("probe.txt", txt)), False, "anonymous-preview")
        expect(add("anonymous-remote", anonymous, "POST", remote,
                   {"url": fixture_url + "/source.json"}), False, "anonymous-remote")
        expect(add("anonymous-invalid-sources", anonymous, "POST",
                   "/reader3/getInvalidBookSources", {}), False, "anonymous-invalid-sources")
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                expect(send(session, base, "POST", "/reader3/login",
                            {"username": name, "password": password, "isLogin": login}),
                       True, "login")

        expect(add("preview-no-file", owner, "POST", path, {}), False, "preview-no-file")
        expect(add("preview-unsupported", owner, "POST", path,
                   upload=("probe.bin", b"binary fixture")), False, "preview-unsupported")
        preview = expect(add("preview-txt", owner, "POST", path,
                             upload=("probe.txt", txt)), True, "preview-txt")
        if len(preview) != 1 or len(preview[0]["chapters"]) != 2:
            raise AssertionError(f"preview did not parse two chapters: {preview}")
        expect(add("owner-shelf-after-preview", owner, "GET", "/reader3/getBookshelf"),
               True, "owner-shelf-after-preview")
        expect(add("other-shelf-after-preview", other, "GET", "/reader3/getBookshelf"),
               True, "other-shelf-after-preview")
        expect(add("other-preview-no-file", other, "POST", path, {}),
               False, "other-preview-no-file")
        if expect(add("owner-invalid-sources-empty", owner, "POST",
                      "/reader3/getInvalidBookSources", {}), True,
                  "owner-invalid-sources-empty") != []:
            raise AssertionError("Owner invalid-source cache should be empty")
        if expect(add("other-invalid-sources-empty", other, "POST",
                      "/reader3/getInvalidBookSources", {}), True,
                  "other-invalid-sources-empty") != []:
            raise AssertionError("Other user's invalid-source cache should be empty")

        expect(add("remote-no-url", owner, "POST", remote, {}), False, "remote-no-url")
        add("remote-http-404", owner, "POST", remote,
            {"url": fixture_url + "/not-found"}, timeout=5)
        expect(add("remote-valid", owner, "POST", remote,
                   {"url": fixture_url + "/source.json"}), True, "remote-valid")
        expect(add("owner-sources-after-import", owner, "GET", "/reader3/getBookSources"),
               True, "owner-sources-after-import")
        expect(add("other-sources-after-import", other, "GET", "/reader3/getBookSources"),
               True, "other-sources-after-import")

        asset = workdir / "storage/assets" / username / "book/probe.txt"
        other_asset = workdir / "storage/assets" / other_user / "book/probe.txt"
        source_file = workdir / "storage/data" / username / "bookSource.json"
        shelf_file = workdir / "storage/data" / username / "bookshelf.json"
        return {"probes": probes, "assetExists": asset.exists(),
                "assetSha256": digest(asset) if asset.exists() else None,
                "otherAssetExists": other_asset.exists(),
                "sourceFileExists": source_file.exists(),
                "sourceStorage": json.loads(source_file.read_text(encoding="utf-8"))
                if source_file.exists() else None,
                "shelfFileExists": shelf_file.exists(),
                "shelfStorage": json.loads(shelf_file.read_text(encoding="utf-8"))
                if shelf_file.exists() else None}
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
    username = "importprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    txt = "第一章 开始\n固定正文。\n第二章 继续\n结束。".encode("utf-8")
    fixture_port = free_port()
    fixture_url = f"http://127.0.0.1:{fixture_port}"
    fixture = subprocess.Popen(
        [sys.executable, "-B", str(ROOT / "scripts/mock-book-source.py"), "--port", str(fixture_port)],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if fixture.poll() is not None:
                raise RuntimeError("Fixture server exited")
            try:
                with urllib.request.urlopen(fixture_url + "/health", timeout=1) as response:
                    if response.status == 200:
                        break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.1)
        else:
            raise TimeoutError("Fixture startup timeout")
        with tempfile.TemporaryDirectory(prefix="reader-import-diff-") as temp:
            root = Path(temp)
            original = run_one(ORIGINAL, root / "original", username, other_user,
                               password, fixture_url, txt)
            restored = run_one(RESTORED, root / "restored", username, other_user,
                               password, fixture_url, txt)
    finally:
        fixture.terminate()
        try:
            fixture.wait(timeout=5)
        except subprocess.TimeoutExpired:
            fixture.kill()
            fixture.wait(timeout=5)

    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "accepted404Fix": (
                        left == {"probe": "remote-http-404", "timeoutAfterSeconds": 5}
                        and right == {"probe": "remote-http-404", "status": 200,
                                      "contentType": "application/json; charset=utf-8",
                                      "body": {"isSuccess": False,
                                               "errorMsg": "远程书源链接错误：HTTP 404"}}),
                    "original": left, "restored": right}
                   for left, right in zip(original["probes"], restored["probes"])]
    report = {"originalJarSha256": digest(ORIGINAL), "restoredJarSha256": digest(RESTORED),
              "fixtureSha256": hashlib.sha256(txt).hexdigest(), "comparisons": comparisons,
              "storage": {"original": {key: value for key, value in original.items() if key != "probes"},
                          "restored": {key: value for key, value in restored.items() if key != "probes"}}}
    by_name = {row["probe"]: row for row in comparisons}
    asset = report["storage"]["restored"]
    sources = asset["sourceStorage"]
    if (len(comparisons) != 16
            or [row["probe"] for row in comparisons if row["accepted404Fix"]] != ["remote-http-404"]
            or not all(row["equal"] or row["accepted404Fix"] for row in comparisons)
            or original["sourceStorage"] != sources
            or not asset["assetExists"] or asset["assetSha256"] != report["fixtureSha256"]
            or asset["otherAssetExists"] or not asset["sourceFileExists"]
            or asset["shelfFileExists"] or not isinstance(sources, list) or len(sources) != 1
            or sources[0]["bookSourceName"] != "Remote import fixture"
            or by_name["remote-valid"]["restored"]["body"]["data"] != ""
            or by_name["owner-shelf-after-preview"]["restored"]["body"]["data"] != []
            or by_name["other-sources-after-import"]["restored"]["body"]["data"] != []):
        raise AssertionError("Import response, persistence or user isolation differs")
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        verdict = "equal" if row["equal"] else "accepted old-JAR 404 timeout" if row["accepted404Fix"] else "DIFFERENT"
        print(f"{row['probe']}: {verdict}")
    print("storageEqual:", report["storage"]["original"] == report["storage"]["restored"])
    print("Report:", REPORT)


if __name__ == "__main__":
    main()
