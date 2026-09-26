#!/usr/bin/env python3
"""Compare file batch deletion, import preview, and legacy asset endpoints.

Every path is inside the two disposable work directories. Traversal sentinels
are created only inside those directories and are never production files.
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
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/file-batch-assets-diff-latest.json"
DYNAMIC_BOOK_FIELDS = {"latestChapterTime", "lastCheckTime", "durChapterTime"}


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
            if key in DYNAMIC_BOOK_FIELDS:
                if not isinstance(item, int):
                    raise AssertionError(f"{key} is not numeric")
                result[key] = "<timestamp>"
            else:
                result[key] = normalize(item)
        return result
    return value


def read_response(response):
    with response:
        raw = response.read()
        content_type = response.headers.get("Content-Type", "")
        if "json" not in content_type:
            return {"status": response.status, "contentType": content_type,
                    "length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        return {"status": response.status, "contentType": content_type,
                "body": normalize(json.loads(raw.decode("utf-8")))}


def request(opener, base, method, path, payload=None):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=25)
    except urllib.error.HTTPError as error:
        response = error
    return read_response(response)


def upload(opener, base, path, filename, content):
    boundary = "reader-probe-" + secrets.token_hex(8)
    data = (f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; "
            f"filename=\"{filename}\"\r\nContent-Type: text/plain\r\n\r\n").encode("ascii")
    data += content + f"\r\n--{boundary}--\r\n".encode("ascii")
    req = urllib.request.Request(base + path, data=data, method="POST",
                                 headers={"Content-Type": f"multipart/form-data; boundary={boundary}"})
    try:
        response = opener.open(req, timeout=25)
    except urllib.error.HTTPError as error:
        response = error
    return read_response(response)


def expect(result, success, name, count=None):
    body = result.get("body", {})
    if result["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{name}: unexpected response: {result}")
    data = body.get("data")
    if count is not None and len(data) != count:
        raise AssertionError(f"{name}: expected {count} items, got {data}")
    return data


def accepted_asset_security_fix(left, right, username):
    common = {"status": 200, "contentType": "application/json; charset=utf-8"}
    if left["probe"] != right["probe"]:
        return False
    if left["probe"] == "asset-upload-parent-type":
        return (left == {"probe": left["probe"], **common,
                         "body": {"isSuccess": True, "errorMsg": "",
                                  "data": [f"/assets/{username}/../escaped-upload.txt"]}}
                and right == {"probe": right["probe"], **common,
                              "body": {"isSuccess": False, "errorMsg": "文件类型错误"}})
    if left["probe"] == "asset-delete-traversal":
        return (left == {"probe": left["probe"], **common,
                         "body": {"isSuccess": True, "errorMsg": "", "data": ""}}
                and right == {"probe": right["probe"], **common,
                              "body": {"isSuccess": False, "errorMsg": "文件链接错误"}})
    return False


def run_one(jar, workdir, username, other_user, password):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    anonymous, owner, other = client(), client(), client()
    probes = []

    def add(name, session, method, path, payload=None):
        result = request(session, base, method, path, payload)
        probes.append({"probe": name, **result})
        return result

    def add_upload(name, session, path, filename, content):
        result = upload(session, base, path, filename, content)
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

        expect(add("anonymous-delete-multi", anonymous, "POST", "/reader3/file/deleteMulti",
                   {"home": "__HOME__", "path": ["/batch/a.txt"]}),
               False, "anonymous-delete-multi")
        expect(add("anonymous-preview", anonymous, "POST", "/reader3/file/importPreview",
                   {"home": "__HOME__", "path": ["/books/preview.txt"]}),
               False, "anonymous-preview")
        expect(add_upload("anonymous-asset-upload", anonymous, "/reader3/uploadFile?type=images",
                          "asset.txt", b"asset fixture"), False, "anonymous-asset-upload")
        add_upload("anonymous-read-source-file", anonymous, "/reader3/readSourceFile",
                   "source.json", b'{"bookSourceUrl":"https://fixture.invalid"}')
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                expect(request(session, base, "POST", "/reader3/login",
                               {"username": name, "password": password, "isLogin": login}),
                       True, "login")

        expect(add("delete-multi-no-path", owner, "POST", "/reader3/file/deleteMulti",
                   {"home": "__HOME__"}), False, "delete-multi-no-path")
        expect(add("preview-no-path", owner, "POST", "/reader3/file/importPreview",
                   {"home": "__HOME__"}), False, "preview-no-path")
        for path, content in (("/batch/a.txt", "alpha"), ("/batch/b.txt", "beta"),
                              ("/batch/keep.txt", "keep"),
                              ("/books/preview.txt", "第一章 开始\n固定正文。\n第二章 继续\n结束。"),
                              ("/books/unsupported.bin", "binary fixture")):
            expect(request(owner, base, "POST", "/reader3/file/save",
                           {"home": "__HOME__", "path": path, "content": content}),
                   True, "prepare-files")
        preview = expect(add("preview-txt", owner, "POST", "/reader3/file/importPreview",
                             {"home": "__HOME__", "path": ["/books/preview.txt"]}),
                         True, "preview-txt", 1)
        if not preview[0]["book"]["bookUrl"] or not preview[0]["chapters"]:
            raise AssertionError("TXT import preview did not return book and chapters")
        expect(add("preview-unsupported", owner, "POST", "/reader3/file/importPreview",
                   {"home": "__HOME__", "path": ["/books/unsupported.bin"]}),
               False, "preview-unsupported")
        expect(add("other-preview-empty", other, "POST", "/reader3/file/importPreview",
                   {"home": "__HOME__", "path": ["/books/preview.txt"]}),
               True, "other-preview-empty", 0)

        data_root = workdir / "storage/data"
        data_root.mkdir(parents=True, exist_ok=True)
        file_guard = data_root / "guard-file-batch.txt"
        file_guard.write_text("outside-user-home-sentinel", encoding="utf-8")
        expect(add("delete-multi", owner, "POST", "/reader3/file/deleteMulti",
                   {"home": "__HOME__", "path": ["/batch/a.txt", "/batch/b.txt"]}),
               True, "delete-multi")
        expect(add("get-deleted-a", owner, "GET", "/reader3/file/get?home=__HOME__&path=/batch/a.txt"),
               False, "get-deleted-a")
        expect(add("get-deleted-b", owner, "GET", "/reader3/file/get?home=__HOME__&path=/batch/b.txt"),
               False, "get-deleted-b")
        expect(add("get-kept", owner, "GET", "/reader3/file/get?home=__HOME__&path=/batch/keep.txt"),
               True, "get-kept")
        expect(add("delete-multi-traversal", owner, "POST", "/reader3/file/deleteMulti",
                   {"home": "__HOME__", "path": ["/../guard-file-batch.txt"]}),
               True, "delete-multi-traversal")
        file_guard_survived = (file_guard.exists() and
                               file_guard.read_text(encoding="utf-8") == "outside-user-home-sentinel")

        expect(add("asset-upload-missing", owner, "POST", "/reader3/uploadFile?type=images", {}),
               False, "asset-upload-missing")
        asset_root = workdir / "storage/assets"
        asset_root.mkdir(parents=True, exist_ok=True)
        asset_guard = asset_root / "guard-asset.txt"
        asset_guard.write_text("outside-asset-user-sentinel", encoding="utf-8")
        add_upload("asset-upload-parent-type", owner, "/reader3/uploadFile?type=..",
                   "escaped-upload.txt", b"escaped fixture")
        escaped_asset_created = (asset_root / "escaped-upload.txt").exists()
        add("asset-delete-traversal", owner, "POST", "/reader3/deleteFile",
            {"url": f"/assets/{username}/images/../../guard-asset.txt"})
        asset_guard_survived = (asset_guard.exists() and
                                asset_guard.read_text(encoding="utf-8") == "outside-asset-user-sentinel")
        uploaded = expect(add_upload("asset-upload", owner, "/reader3/uploadFile?type=images",
                                     "asset.txt", b"asset fixture"), True, "asset-upload", 1)
        asset_url = uploaded[0]
        expected_asset_url = f"/assets/{username}/images/asset.txt"
        if asset_url != expected_asset_url:
            raise AssertionError(f"unexpected asset URL: {asset_url}")
        asset_file = workdir / "storage/assets" / username / "images/asset.txt"
        if asset_file.read_bytes() != b"asset fixture":
            raise AssertionError("uploaded asset bytes changed")
        expect(add("other-delete-asset", other, "POST", "/reader3/deleteFile",
                   {"url": asset_url}), False, "other-delete-asset")
        expect(add("delete-asset-missing-url", owner, "POST", "/reader3/deleteFile", {}),
               False, "delete-asset-missing-url")
        expect(add("delete-asset", owner, "POST", "/reader3/deleteFile",
                   {"url": asset_url}), True, "delete-asset")
        asset_removed = not asset_file.exists()
        expect(add("delete-asset-again", owner, "POST", "/reader3/deleteFile",
                   {"url": asset_url}), True, "delete-asset-again")

        expect(add_upload("read-source-file", owner, "/reader3/readSourceFile",
                          "source.json", b'{"bookSourceUrl":"https://fixture.invalid"}'),
               True, "read-source-file", 1)
        expect(add("read-source-no-file", owner, "POST", "/reader3/readSourceFile", {}),
               False, "read-source-no-file")
        return {"probes": probes, "fileGuardSurvived": file_guard_survived,
                "assetGuardSurvived": asset_guard_survived,
                "escapedAssetCreated": escaped_asset_created,
                "assetRemoved": asset_removed,
                "keptFile": (workdir / "storage/data" / username / "batch/keep.txt").read_text(encoding="utf-8")}
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
    username = "fileprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-file-batch-diff-") as temp:
        root = Path(temp)
        original = run_one(ORIGINAL, root / "original", username, other_user, password)
        restored = run_one(RESTORED, root / "restored", username, other_user, password)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("probe count differs")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "acceptedSecurityFix": accepted_asset_security_fix(left, right, username),
                    "original": left, "restored": right}
                   for left, right in zip(original["probes"], restored["probes"])]
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons,
              "fileGuardSurvived": {"original": original["fileGuardSurvived"],
                                    "restored": restored["fileGuardSurvived"]},
              "acceptedOldJarTraversalFix": (not original["fileGuardSurvived"]
                                              and restored["fileGuardSurvived"]),
              "assetGuardSurvived": {"original": original["assetGuardSurvived"],
                                     "restored": restored["assetGuardSurvived"]},
              "escapedAssetCreated": {"original": original["escapedAssetCreated"],
                                      "restored": restored["escapedAssetCreated"]},
              "acceptedOldJarAssetPathFix": (not original["assetGuardSurvived"]
                                              and restored["assetGuardSurvived"]
                                              and original["escapedAssetCreated"]
                                              and not restored["escapedAssetCreated"]),
              "assetRemoved": {"original": original["assetRemoved"],
                               "restored": restored["assetRemoved"]},
              "keptFileEqual": original["keptFile"] == restored["keptFile"] == "keep"}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        verdict = "equal" if row["equal"] else "accepted security fix" if row["acceptedSecurityFix"] else "DIFFERENT"
        print(f"{row['probe']}: {verdict}")
    for key in ("fileGuardSurvived", "acceptedOldJarTraversalFix", "assetGuardSurvived",
                "escapedAssetCreated", "acceptedOldJarAssetPathFix", "assetRemoved", "keptFileEqual"):
        print(f"{key}: {report[key]}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] or row["acceptedSecurityFix"] for row in comparisons) or not (
            report["acceptedOldJarTraversalFix"] and report["acceptedOldJarAssetPathFix"]
            and all(report["assetRemoved"].values())
            and report["keptFileEqual"]):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
