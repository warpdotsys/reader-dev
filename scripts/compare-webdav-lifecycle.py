#!/usr/bin/env python3
"""Compare isolated WebDAV behavior of the original and restored JARs.

Only generated accounts and files in temporary work directories are used.
The temporary directories are deleted after both Java processes have stopped.
"""

import argparse
import base64
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
import xml.etree.ElementTree as ET


PROJECT = Path(__file__).resolve().parents[1]
DAV = "{DAV:}"


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def request(base, method, path, body=None, headers=None):
    req = urllib.request.Request(base + path, data=body, method=method, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=20) as response:
            return response.status, dict(response.headers), response.read()
    except urllib.error.HTTPError as error:
        return error.code, dict(error.headers), error.read()


def webdav_summary(name, response):
    status, headers, body = response
    result = {"probe": name, "status": status, "contentType": headers.get("Content-Type", "")}
    if name.startswith("propfind-") and status == 207:
        root = ET.fromstring(body)
        entries = []
        for entry in root.findall(DAV + "response"):
            href = entry.findtext(DAV + "href", "")
            prop = entry.find(DAV + "propstat/" + DAV + "prop")
            if prop is None:
                raise AssertionError("PROPFIND response lacks DAV:prop")
            entries.append({
                "path": urllib.parse.unquote(urllib.parse.urlsplit(href).path),
                "displayName": prop.findtext(DAV + "displayname", ""),
                "collection": prop.find(DAV + "resourcetype/" + DAV + "collection") is not None,
                "contentLength": prop.findtext(DAV + "getcontentlength", ""),
                "hasModified": bool(prop.findtext(DAV + "getlastmodified")),
            })
        result["entries"] = sorted(entries, key=lambda item: item["path"])
    elif name == "lock-file" and status == 200:
        root = ET.fromstring(body)
        token = root.findtext(".//" + DAV + "locktoken/" + DAV + "href", "")
        lock_root = root.findtext(".//" + DAV + "lockroot/" + DAV + "href", "")
        result["tokenIsUuid"] = token.startswith("urn:uuid:") and len(token) == 45
        result["headerMatchesToken"] = headers.get("Lock-Token", "") == token
        result["lockRootPath"] = urllib.parse.urlsplit(lock_root).path
        result["timeout"] = root.findtext(".//" + DAV + "timeout", "")
    elif name == "unlock-file" and status == 204:
        result["hasLockTokenHeader"] = bool(headers.get("Lock-Token"))
    elif name.startswith("get-") and status == 200:
        result["length"] = len(body)
        result["sha256"] = hashlib.sha256(body).hexdigest()
        result["disposition"] = headers.get("Content-Disposition", "")
        result["cacheControl"] = headers.get("Cache-Control", "")
    else:
        result["bodyLength"] = len(body)
    if name in ("options-anonymous", "propfind-anonymous", "propfind-wrong-password"):
        result["davHeader"] = headers.get("DAV", "")
        result["allowHeader"] = headers.get("Allow", "")
    return result


def run_one(jar, java, workdir, username, password):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    stdout_path = workdir / "stdout.log"
    stderr_path = workdir / "stderr.log"
    command = [
        str(java), "-jar", str(jar),
        f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
        "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
        "--reader.app.defaultUserEnableWebdav=true", "--spring.profiles.active=prod",
    ]
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr, cwd=workdir)
        try:
            deadline = time.monotonic() + 90
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"JAR exited during startup: {process.returncode}")
                try:
                    status, _, body = request(base, "GET", "/reader3/getSystemInfo")
                    if status == 200 and json.loads(body)["isSuccess"]:
                        break
                except (OSError, ValueError, KeyError):
                    pass
                time.sleep(0.5)
            else:
                raise TimeoutError("JAR did not become ready in 90 seconds")

            registration = json.dumps({"username": username, "password": password, "isLogin": False}).encode()
            status, _, body = request(base, "POST", "/reader3/login", registration,
                                      {"Content-Type": "application/json"})
            if status != 200 or not json.loads(body).get("isSuccess"):
                raise AssertionError("Isolated WebDAV test account registration failed")
            user_file = workdir / "storage" / "data" / "users.json"
            users = json.loads(user_file.read_text(encoding="utf-8"))
            if not users[username]["enable_webdav"]:
                raise AssertionError("WebDAV permission was not enabled for isolated test account")

            base_path = "/reader3/webdav/"
            basic = base64.b64encode(f"{username}:{password}".encode()).decode("ascii")
            wrong = base64.b64encode(f"{username}:{password}wrong".encode()).decode("ascii")
            auth = {"Authorization": "Basic " + basic}
            wrong_auth = {"Authorization": "Basic " + wrong}
            probes = []

            def probe(name, method, path, body=None, headers=None):
                result = webdav_summary(name, request(base, method, path, body, headers))
                probes.append(result)
                return result

            probe("options-anonymous", "OPTIONS", base_path)
            probe("propfind-anonymous", "PROPFIND", base_path)
            probe("propfind-wrong-password", "PROPFIND", base_path, headers=wrong_auth)
            probe("propfind-root", "PROPFIND", base_path, headers=auth)
            probe("mkcol", "MKCOL", base_path + "probe", headers=auth)
            sample = "WebDAV 测试\n".encode()
            probe("put-file", "PUT", base_path + "probe/a.txt", sample, auth)
            probe("get-file", "GET", base_path + "probe/a.txt", headers=auth)
            unicode_path = base_path + "probe/" + urllib.parse.quote("章节-中文.txt")
            probe("put-unicode", "PUT", unicode_path, sample, auth)
            probe("get-unicode", "GET", unicode_path, headers=auth)
            probe("propfind-directory", "PROPFIND", base_path + "probe/", headers=auth)
            copy_headers = dict(auth, Destination=base + base_path + "probe/b.txt")
            probe("copy-file", "COPY", base_path + "probe/a.txt", headers=copy_headers)
            move_headers = dict(auth, Destination=base + base_path + "probe/c.txt")
            probe("move-file", "MOVE", base_path + "probe/b.txt", headers=move_headers)
            probe("get-moved", "GET", base_path + "probe/c.txt", headers=auth)
            lock_response = request(base, "LOCK", base_path + "probe/a.txt", headers=auth)
            probes.append(webdav_summary("lock-file", lock_response))
            lock_token = lock_response[1].get("Lock-Token", "")
            probe("unlock-file", "UNLOCK", base_path + "probe/a.txt",
                  headers=dict(auth, **{"Lock-Token": lock_token}))
            probe("delete-original", "DELETE", base_path + "probe/a.txt", headers=auth)
            probe("get-deleted", "GET", base_path + "probe/a.txt", headers=auth)
            probe("delete-unicode", "DELETE", unicode_path, headers=auth)
            probe("delete-directory", "DELETE", base_path + "probe", headers=auth)
            webdav_root = workdir / "storage" / "data" / username / "webdav"
            expected_status = {
                "options-anonymous": 200, "propfind-anonymous": 401,
                "propfind-wrong-password": 401, "propfind-root": 207,
                "mkcol": 201, "put-file": 201, "get-file": 200,
                "put-unicode": 201, "get-unicode": 200,
                "propfind-directory": 207, "copy-file": 201,
                "move-file": 201, "get-moved": 200,
                "lock-file": 200, "unlock-file": 204,
                "delete-original": 200, "get-deleted": 404,
                "delete-unicode": 200, "delete-directory": 200,
            }
            actual_status = {item["probe"]: item["status"] for item in probes}
            if actual_status != expected_status:
                raise AssertionError(f"Unexpected WebDAV status: {actual_status}")
            expected_hash = hashlib.sha256(sample).hexdigest()
            for name in ("get-file", "get-unicode", "get-moved"):
                downloaded = next(item for item in probes if item["probe"] == name)
                if downloaded["sha256"] != expected_hash:
                    raise AssertionError(f"WebDAV download differs from uploaded bytes: {name}")
            directory = next(item for item in probes if item["probe"] == "propfind-directory")
            names = {entry["displayName"] for entry in directory["entries"]}
            if names != {"", "a.txt", "章节-中文.txt"}:
                raise AssertionError(f"Unexpected WebDAV directory entries: {names}")
            lock = next(item for item in probes if item["probe"] == "lock-file")
            if not lock["tokenIsUuid"] or not lock["headerMatchesToken"]:
                raise AssertionError("WebDAV LOCK token is absent or malformed")
            if not webdav_root.is_dir() or any(webdav_root.iterdir()):
                raise AssertionError("WebDAV test data remained after deletion")
            return {"probes": probes, "webdavRootExists": webdav_root.is_dir(),
                    "remainingEntries": sorted(path.name for path in webdav_root.iterdir())}
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", type=Path, default=PROJECT / ".tools/jdk-11.0.8/bin/java.exe")
    parser.add_argument("--original", type=Path,
                        default=PROJECT / "reference/original/reader-pro-3.2.14.original.jar")
    parser.add_argument("--restored", type=Path, default=PROJECT / "build/libs/reader-4.0.7.jar")
    parser.add_argument("--output", type=Path, default=PROJECT / "reports/webdav-lifecycle-diff-latest.json")
    args = parser.parse_args()
    for path in (args.java, args.original, args.restored):
        if not path.is_file():
            parser.error(f"Required file not found: {path}")
    tools_dir = PROJECT / ".tools"
    tools_dir.mkdir(exist_ok=True)
    username = "probe" + uuid.uuid4().hex[:12]
    password = "Probe-" + uuid.uuid4().hex
    with tempfile.TemporaryDirectory(prefix="webdav-diff-", dir=tools_dir) as root_string:
        root = Path(root_string)
        original_dir = root / "original"
        restored_dir = root / "restored"
        original_dir.mkdir()
        restored_dir.mkdir()
        original = run_one(args.original, args.java, original_dir, username, password)
        restored = run_one(args.restored, args.java, restored_dir, username, password)
    comparisons = [
        {"probe": left["probe"], "equal": left == right, "original": left, "restored": right}
        for left, right in zip(original["probes"], restored["probes"])
    ]
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("Probe count differs")
    report = {
        "originalJarSha256": hashlib.sha256(args.original.read_bytes()).hexdigest(),
        "restoredJarSha256": hashlib.sha256(args.restored.read_bytes()).hexdigest(),
        "comparisons": comparisons,
        "storage": {"original": {key: value for key, value in original.items() if key != "probes"},
                    "restored": {key: value for key, value in restored.items() if key != "probes"}},
    }
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for item in comparisons:
        print(f"{item['probe']}: {'equal' if item['equal'] else 'DIFFERENT'}")
    print(f"Report: {args.output}")
    if any(not item["equal"] for item in comparisons) or report["storage"]["original"] != report["storage"]["restored"]:
        raise AssertionError("WebDAV lifecycle differs from original JAR; inspect report")


if __name__ == "__main__":
    main()
