#!/usr/bin/env python3
"""Compare isolated user administration with the original Reader JAR.

All accounts, credentials, and management keys are generated per run. Only
disposable local work directories are used; no production user is modified.
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
REPORT = ROOT / "reports/user-admin-lifecycle-diff-latest.json"


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def normalize(value, key=""):
    if key in {"created_at", "last_login_at", "createdAt", "lastLoginAt"}:
        if not isinstance(value, int) or value <= 0:
            raise AssertionError(f"{key} is not a positive timestamp: {value}")
        return "<timestamp>"
    if key in {"salt", "password", "token"}:
        if not isinstance(value, str) or not value:
            raise AssertionError(f"{key} is missing")
        return "<secret>"
    if key == "accessToken":
        if not isinstance(value, str) or ":" not in value:
            raise AssertionError("accessToken has no username prefix")
        username, token = value.split(":", 1)
        return username + (":<token>" if token else ":<empty>")
    if key == "token_map":
        if not isinstance(value, dict):
            raise AssertionError("token_map is not an object")
        if any(not isinstance(item, int) for item in value.values()):
            raise AssertionError("token_map has nonnumeric expiry")
        return {"count": len(value)}
    if isinstance(value, list):
        result = [normalize(item) for item in value]
        if result and all(isinstance(item, dict) and "username" in item for item in result):
            result.sort(key=lambda item: item["username"])
        return result
    if isinstance(value, dict):
        return {name: normalize(item, name) for name, item in value.items()}
    return value


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
                "body": normalize(json.loads(raw.decode("utf-8")))}


def expect(result, success, name, count=None):
    body = result.get("body", {})
    if result["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{name}: unexpected response: {result}")
    data = body.get("data")
    if count is not None and len(data) != count:
        raise AssertionError(f"{name}: expected {count} users, got {data}")
    return data


def run_one(jar, workdir, admin, target, password, new_password, secure_key):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         f"--reader.app.secureKey={secure_key}",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    anonymous, manager, target_client = client(), client(), client()
    probes = []
    manager_path = "?secureKey=" + urllib.parse.quote(secure_key, safe="")

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

        expect(add("anonymous-list", anonymous, "GET", "/reader3/getUserList" + manager_path),
               False, "anonymous-list")
        expect(add("anonymous-add", anonymous, "POST", "/reader3/addUser" + manager_path,
                   {"username": target, "password": password}), False, "anonymous-add")
        expect(request(manager, base, "POST", "/reader3/login",
                       {"username": admin, "password": password, "isLogin": False}),
               True, "register-manager")
        expect(request(manager, base, "POST", "/reader3/login",
                       {"username": admin, "password": password, "isLogin": True}),
               True, "login-manager")

        expect(add("list-no-key", manager, "GET", "/reader3/getUserList"),
               False, "list-no-key")
        expect(add("list-wrong-key", manager, "GET", "/reader3/getUserList?secureKey=wrong"),
               False, "list-wrong-key")
        expect(add("list-one", manager, "GET", "/reader3/getUserList" + manager_path),
               True, "list-one", 1)
        expect(add("add-no-key", manager, "POST", "/reader3/addUser",
                   {"username": target, "password": password}), False, "add-no-key")
        expect(add("add-short-password", manager, "POST", "/reader3/addUser" + manager_path,
                   {"username": target, "password": "short"}), False, "add-short-password")
        add_payload = {"username": target, "password": password,
                       "enableWebdav": True, "enableBookSource": False,
                       "bookSourceLimit": 7, "bookLimit": 11}
        expect(add("add-target", manager, "POST", "/reader3/addUser" + manager_path,
                   add_payload), True, "add-target", 2)
        expect(add("add-duplicate", manager, "POST", "/reader3/addUser" + manager_path,
                   add_payload), False, "add-duplicate")
        expect(add("list-two", manager, "GET", "/reader3/getUserList" + manager_path),
               True, "list-two", 2)

        update_payload = {"username": target, "enableWebdav": False,
                          "enableBookSource": True, "bookSourceLimit": 9, "bookLimit": 13}
        expect(add("update-no-key", manager, "POST", "/reader3/updateUser",
                   update_payload), False, "update-no-key")
        updated = expect(add("update-target", manager, "POST", "/reader3/updateUser" + manager_path,
                             update_payload), True, "update-target", 2)
        row = next(user for user in updated if user["username"] == target)
        if (row["enableWebdav"] is not False or row["bookSourceLimit"] != 9 or row["bookLimit"] != 13):
            raise AssertionError("updated user flags and limits were not persisted")
        expect(add("update-missing", manager, "POST", "/reader3/updateUser" + manager_path,
                   {"username": "absentprobe"}), False, "update-missing")

        reset_payload = {"username": target, "password": new_password}
        expect(add("reset-no-key", manager, "POST", "/reader3/resetPassword",
                   reset_payload), False, "reset-no-key")
        expect(add("reset-target", manager, "POST", "/reader3/resetPassword" + manager_path,
                   reset_payload), True, "reset-target")
        expect(add("login-old-password", target_client, "POST", "/reader3/login",
                   {"username": target, "password": password, "isLogin": True}),
               False, "login-old-password")
        expect(add("login-new-password", target_client, "POST", "/reader3/login",
                   {"username": target, "password": new_password, "isLogin": True}),
               True, "login-new-password")
        expect(request(target_client, base, "POST", "/reader3/saveUserConfig",
                       {"probe": "user-admin-lifecycle"}), True, "prepare-user-namespace")
        target_dir = workdir / "storage/data" / target
        if not target_dir.exists():
            raise AssertionError("target user namespace was not created")

        expect(add("clear-no-key", manager, "POST", "/reader3/clearInactiveUsers",
                   {"inactiveDay": 100000}), False, "clear-no-key")
        expect(add("clear-noop", manager, "POST", "/reader3/clearInactiveUsers" + manager_path,
                   {"inactiveDay": 100000}), True, "clear-noop", 2)
        expect(add("delete-no-key", manager, "POST", "/reader3/deleteUsers",
                   [target]), False, "delete-no-key")
        expect(add("delete-target", manager, "POST", "/reader3/deleteUsers" + manager_path,
                   [target]), True, "delete-target", 1)
        expect(add("login-after-delete", target_client, "POST", "/reader3/login",
                   {"username": target, "password": new_password, "isLogin": True}),
               False, "login-after-delete")
        expect(add("list-final", manager, "GET", "/reader3/getUserList" + manager_path),
               True, "list-final", 1)

        users_file = workdir / "storage/data/users.json"
        if not users_file.exists():
            raise AssertionError("users.json is missing")
        return {"probes": probes, "storage": normalize(json.loads(users_file.read_text(encoding="utf-8"))),
                "targetNamespaceExists": target_dir.exists()}
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
    admin = "adminprobe" + secrets.token_hex(5)
    target = "userprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    new_password = "Changed-" + secrets.token_hex(18)
    secure_key = "ManagerProbe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-user-admin-diff-") as temp:
        root = Path(temp)
        original = run_one(ORIGINAL, root / "original", admin, target, password,
                           new_password, secure_key)
        restored = run_one(RESTORED, root / "restored", admin, target, password,
                           new_password, secure_key)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("probe count differs")
    comparisons = [{"probe": left["probe"], "equal": left == right,
                    "original": left, "restored": right}
                   for left, right in zip(original["probes"], restored["probes"])]
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons,
              "storageEqual": original["storage"] == restored["storage"],
              "targetNamespaceRemoved": not original["targetNamespaceExists"]
              and not restored["targetNamespaceExists"],
              "storage": {"original": original["storage"], "restored": restored["storage"]}}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        print(f"{row['probe']}: {'equal' if row['equal'] else 'DIFFERENT'}")
    print(f"storageEqual: {report['storageEqual']}")
    print(f"targetNamespaceRemoved: {report['targetNamespaceRemoved']}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] for row in comparisons) or not all(
            report[key] for key in ("storageEqual", "targetNamespaceRemoved")):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
