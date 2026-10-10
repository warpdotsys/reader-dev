"""Read exactly five authorized business JSONs with both JARs, without egress.

Run as root under `unshare --net --fork` on Linux. The original JAR ignores
bindAddress, so this program independently verifies a private, loopback-only
kernel network namespace before dropping privileges and starting either JAR.
Only aggregate comparisons are persisted; private responses and credentials
stay in memory, and no source search, refresh, or book download is requested.
"""

import argparse
import hashlib
import http.cookiejar
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


ORIGINAL_SHA256 = "b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c"
BUSINESS_FILES = ("bookshelf.json", "bookGroup.json", "bookSource.json",
                  "bookSourceMap.json", "replaceRule.json")
READ_ROUTES = ("getBookshelf?refresh=0", "getBookGroups", "getBookSources?simple=0",
               "getReplaceRules")


def check_namespace(current, initial, interfaces):
    if current == initial or interfaces != ["lo"]:
        raise SystemExit("Refusing to start JARs: private loopback-only netns required")


def isolate_then_drop():
    if sys.platform != "linux" or os.geteuid() != 0:
        raise SystemExit("Refusing to start JARs: run as root in a private Linux netns")
    check_namespace(os.stat("/proc/self/ns/net").st_ino,
                    os.stat("/proc/1/ns/net").st_ino,
                    [name for _, name in socket.if_nameindex()])
    subprocess.run(["ip", "link", "set", "lo", "up"], check=True)
    # Do not let an environment acknowledgment replace the actual kernel check.
    check_namespace(os.stat("/proc/self/ns/net").st_ino,
                    os.stat("/proc/1/ns/net").st_ino,
                    [name for _, name in socket.if_nameindex()])
    os.setgroups([])
    os.setgid(65534)
    os.setuid(65534)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def new_opener():
    # Do not inherit host HTTP proxy settings, including localhost proxy URLs.
    return urllib.request.build_opener(urllib.request.ProxyHandler({}),
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def call(opener, base, route, body=None):
    payload = None if body is None else json.dumps(body).encode("utf-8")
    request = urllib.request.Request(base + "/reader3/" + route, data=payload,
        headers={"Content-Type": "application/json"} if payload else {},
        method="POST" if payload else "GET")
    try:
        response = opener.open(request, timeout=30)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read()
        value = json.loads(raw.decode("utf-8"))
        return {"status": response.status, "value": value,
                "replacementCharacter": "\ufffd" in raw.decode("utf-8")}


def success(reply):
    if reply["status"] != 200 or reply["value"].get("isSuccess") is not True:
        # Never interpolate response data/error text: it can contain private data.
        raise RuntimeError("Isolated Reader business request did not succeed")
    return reply["value"].get("data")


def same_json(left, right):
    # Python equality conflates False/0 and True/1. Preserve JSON value types
    # (and numeric representation) while ignoring object member ordering.
    options = {"sort_keys": True, "ensure_ascii": False, "separators": (",", ":")}
    return json.dumps(left, **options) == json.dumps(right, **options)


def run_reader(java, jar, workdir, business, username, password):
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    base = f"http://127.0.0.1:{port}"
    workdir.mkdir()
    command = [str(java), "-Xms128m", "-Xmx768m", "-jar", str(jar),
        f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
        "--reader.server.bindAddress=127.0.0.1", "--reader.app.secure=true",
        "--reader.app.licenseCheckEnabled=false", "--reader.app.shelfUpdateInteval=0",
        "--reader.app.remoteBookSourceUpdateInterval=0",
        "--reader.app.autoBackupUserData=false", "--reader.app.autoClearInactiveUser=0"]
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(command, cwd=workdir, stdout=log, stderr=log)
        try:
            account = new_opener()
            deadline = time.monotonic() + 75
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError("Isolated Reader exited before readiness")
                try:
                    success(call(account, base, "getSystemInfo"))
                    break
                except (OSError, ValueError, RuntimeError):
                    time.sleep(0.25)
            else:
                raise TimeoutError("Isolated Reader startup timed out")

            success(call(account, base, "login", {"username": username,
                "password": password, "isLogin": False}))
            login = success(call(account, base, "login", {"username": username,
                "password": password, "isLogin": True}))
            namespace = workdir / "storage" / "data" / username
            namespace.mkdir(parents=True, exist_ok=True)
            for name, content in business.items():
                target = namespace / name
                if target.exists():
                    raise RuntimeError("Fresh isolated business destination unexpectedly exists")
                target.write_bytes(content)

            replies = {route: call(account, base, route) for route in READ_ROUTES}
            for reply in replies.values():
                success(reply)

            token = login.get("accessToken", "") if isinstance(login, dict) else ""
            token_reply = call(new_opener(), base, "getBookshelf?refresh=0&accessToken="
                               + urllib.parse.quote(token, safe=""))
            anonymous = call(new_opener(), base, "getBookshelf?refresh=0")
            other = new_opener()
            success(call(other, base, "login", {"username": username + "b",
                "password": password, "isLogin": False}))
            other_shelf = success(call(other, base, "getBookshelf?refresh=0"))
            facts = {
                "loginSucceeded": True,
                "accessTokenNamespaced": token.startswith(username + ":"),
                "tokenOnlyShelfEqual": token_reply["status"] == 200
                    and token_reply["value"].get("isSuccess") is True
                    and same_json(token_reply["value"].get("data"),
                                  replies[READ_ROUTES[0]]["value"].get("data")),
                "anonymousShelfDenied": anonymous["value"].get("isSuccess") is False
                    and anonymous["value"].get("data") == "NEED_LOGIN",
                "otherNamespaceEmpty": other_shelf == [],
                "businessFilesUnchanged": all((namespace / name).read_bytes() == content
                                               for name, content in business.items()),
            }
            if not all(facts.values()):
                raise RuntimeError("Isolated login, namespace, or read-only assertion failed")
            return replies, facts
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def aggregate(left, right):
    a, b = left["value"], right["value"]
    da, db = a.get("data"), b.get("data")
    return {
        "originalStatus": left["status"], "restoredStatus": right["status"],
        "statusEqual": left["status"] == right["status"],
        "isSuccessEqual": same_json(a.get("isSuccess"), b.get("isSuccess")),
        "errorMsgEqual": same_json(a.get("errorMsg"), b.get("errorMsg")),
        "envelopeFieldsEqual": set(a) == set(b),
        "originalCount": len(da) if isinstance(da, list) else None,
        "restoredCount": len(db) if isinstance(db, list) else None,
        "dataEqual": same_json(da, db),
        "jsonTypesComparedStrictly": True,
        "replacementCharacterObserved": left["replacementCharacter"] or right["replacementCharacter"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", required=True, type=Path)
    parser.add_argument("--original", required=True, type=Path)
    parser.add_argument("--restored", required=True, type=Path)
    parser.add_argument("--business-directory", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    isolate_then_drop()  # Fail before reading private data or executing any JAR.
    if digest(args.original) != ORIGINAL_SHA256:
        parser.error("Original JAR does not match the read-only baseline")
    report_path = args.report.resolve()
    if not report_path.is_relative_to(Path("/var/tmp")) or report_path.exists():
        parser.error("Report must be a new file beneath /var/tmp")
    business = {name: (args.business_directory / name).read_bytes() for name in BUSINESS_FILES}
    for content in business.values():
        if not isinstance(json.loads(content.decode("utf-8")), (list, dict)):
            parser.error("Expected business JSON arrays or objects")
    original_hash, restored_hash = digest(args.original), digest(args.restored)
    username, password = "storageprobe" + secrets.token_hex(5), "Probe-" + secrets.token_hex(12)
    with tempfile.TemporaryDirectory(prefix="reader-private-storage-diff-") as temporary:
        root = Path(temporary)
        original, original_facts = run_reader(args.java, args.original, root / "original",
                                             business, username, password)
        restored, restored_facts = run_reader(args.java, args.restored, root / "restored",
                                             business, username, password)
    source_unchanged = all((args.business_directory / name).read_bytes() == content
                           for name, content in business.items())
    if not source_unchanged or digest(args.original) != original_hash or digest(args.restored) != restored_hash:
        raise RuntimeError("Read-only input changed during comparison")
    report = {
        "originalJarSha256": original_hash, "restoredJarSha256": restored_hash,
        "originalExecuted": True, "network": "private-loopback-only-netns",
        "privateDataPublished": False, "sourceUnchanged": source_unchanged,
        "inputFileCount": len(business), "inputBytes": sum(map(len, business.values())),
        "originalFacts": original_facts, "restoredFacts": restored_facts,
        "routes": {route: aggregate(original[route], restored[route]) for route in READ_ROUTES},
        "limitations": ["No book files, production accounts, saved Cookie jars or caches",
                        "No searches, WebView renders, source logins, SSE or downloads",
                        "A synthetic account is used; this is not production-user authentication"],
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False))


if __name__ == "__main__":
    main()
