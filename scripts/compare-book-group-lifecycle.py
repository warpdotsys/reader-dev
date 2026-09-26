#!/usr/bin/env python3
"""Compare book-group CRUD and shelf assignment with the original JAR.

Uses one generated two-chapter TXT, disposable users, and isolated local storage.
No production data or external source is accessed.
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
REPORT = ROOT / "reports/book-group-lifecycle-diff-latest.json"
DYNAMIC_BOOK_FIELDS = {"latestChapterTime", "lastCheckTime", "durChapterTime"}
DEFAULT_GROUP_NAMES = ["全部", "本地", "音频", "未分组", "更新错误"]


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


def different_paths(left, right, prefix=""):
    if isinstance(left, dict) and isinstance(right, dict):
        paths = []
        for key in sorted(left.keys() | right.keys()):
            path = f"{prefix}.{key}" if prefix else key
            if key not in left or key not in right:
                paths.append(path)
            else:
                paths.extend(different_paths(left[key], right[key], path))
        return paths
    if isinstance(left, list) and isinstance(right, list):
        if len(left) != len(right):
            return [prefix + ".length"]
        return [path for index, (a, b) in enumerate(zip(left, right))
                for path in different_paths(a, b, f"{prefix}[{index}]")]
    return [] if left == right else [prefix]


def accepted_default_group_name_corruption(left, right, prefix):
    expected = {f"{prefix}[{index}].groupName" for index in range(5)}
    paths = different_paths(left, right)
    rows = right["body"]["data"] if prefix == "body.data" else right
    accepted = (set(paths) == expected and len(rows) >= 5
                and [row["groupName"] for row in rows[:5]] == DEFAULT_GROUP_NAMES)
    return accepted, paths


def request(opener, base, method, path, payload=None):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=20)
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
    if count is not None and len(body.get("data", [])) != count:
        raise AssertionError(f"{name}: expected {count} rows, got {body.get('data')}")
    return body.get("data")


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

        add("anonymous-groups", anonymous, "GET", "/reader3/getBookGroups")
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                response = request(session, base, "POST", "/reader3/login",
                                   {"username": name, "password": password, "isLogin": login})
                expect(response, True, "login")

        defaults = expect(add("default-groups", owner, "GET", "/reader3/getBookGroups"),
                          True, "default-groups", 5)
        if [row["groupId"] for row in defaults] != [-1, -2, -3, -4, -5]:
            raise AssertionError("default group IDs differ from the original contract")
        expect(add("invalid-group", owner, "POST", "/reader3/saveBookGroup",
                   {"groupId": 0, "groupName": ""}), False, "invalid-group")
        expect(add("save-first", owner, "POST", "/reader3/saveBookGroup",
                   {"groupId": 0, "groupName": "Alpha"}), True, "save-first")
        first_list = expect(add("after-first", owner, "GET", "/reader3/getBookGroups"),
                            True, "after-first", 6)
        first_id = next(row["groupId"] for row in first_list if row["groupName"] == "Alpha")
        expect(add("save-second", owner, "POST", "/reader3/saveBookGroup",
                   {"groupId": 0, "groupName": "Beta"}), True, "save-second")
        second_list = expect(add("after-second", owner, "GET", "/reader3/getBookGroups"),
                             True, "after-second", 7)
        second_id = next(row["groupId"] for row in second_list if row["groupName"] == "Beta")
        if first_id <= 0 or second_id <= 0 or first_id == second_id:
            raise AssertionError("custom group IDs are not unique positive bit flags")
        expect(add("update-first", owner, "POST", "/reader3/saveBookGroup",
                   {"groupId": first_id, "groupName": "Alpha-updated", "show": False}),
               True, "update-first")
        expect(add("after-update", owner, "GET", "/reader3/getBookGroups"),
               True, "after-update", 7)
        expect(add("save-order", owner, "POST", "/reader3/saveBookGroupOrder",
                   {"order": [{"groupId": first_id, "order": 50},
                              {"groupId": second_id, "order": 40}]}), True, "save-order")
        ordered = expect(add("after-order", owner, "GET", "/reader3/getBookGroups"),
                         True, "after-order", 7)
        orders = {row["groupId"]: row["order"] for row in ordered}
        if orders[first_id] != 50 or orders[second_id] != 40:
            raise AssertionError("group order was not persisted")
        expect(add("other-user-defaults", other, "GET", "/reader3/getBookGroups"),
               True, "other-user-defaults", 5)

        reading_text = "第一章 开始\n这是第一章。\n第二章 继续\n这是第二章。"
        expect(request(owner, base, "POST", "/reader3/file/save",
                       {"home": "__HOME__", "path": "/reading/group.txt",
                        "content": reading_text}), True, "prepare-local-txt")
        expect(request(owner, base, "GET", "/reader3/file/parse?home=__HOME__&path=/reading&import=1"),
               True, "prepare-import")
        shelf = expect(request(owner, base, "GET", "/reader3/getBookshelf"),
                       True, "prepare-shelf", 1)
        book_url = shelf[0]["bookUrl"]
        if not book_url:
            raise AssertionError("imported local book has no URL")
        expect(add("missing-book-url", owner, "POST", "/reader3/saveBookGroupId",
                   {"groupId": first_id}), False, "missing-book-url")
        expect(add("invalid-group-id", owner, "POST", "/reader3/saveBookGroupId",
                   {"bookUrl": book_url, "groupId": 0}), False, "invalid-group-id")
        assign = expect(add("assign-first", owner, "POST", "/reader3/saveBookGroupId",
                            {"bookUrl": book_url, "groupId": first_id}), True, "assign-first")
        if assign["group"] != first_id:
            raise AssertionError("single-book group assignment did not set the group bit")
        after_assign = expect(add("shelf-after-assign", owner, "GET", "/reader3/getBookshelf"),
                              True, "shelf-after-assign", 1)
        if after_assign[0]["group"] != first_id:
            raise AssertionError("assigned group was not persisted")
        book_list = [{"bookUrl": book_url}]
        expect(add("add-group-multi", owner, "POST", "/reader3/addBookGroupMulti",
                   {"groupId": second_id, "bookList": book_list}), True, "add-group-multi")
        after_add = expect(add("shelf-after-add", owner, "GET", "/reader3/getBookshelf"),
                           True, "shelf-after-add", 1)
        if after_add[0]["group"] != first_id | second_id:
            raise AssertionError("bulk group add did not set both bits")
        expect(add("remove-group-multi", owner, "POST", "/reader3/removeBookGroupMulti",
                   {"groupId": second_id, "bookList": book_list}), True, "remove-group-multi")
        after_remove = expect(add("shelf-after-remove", owner, "GET", "/reader3/getBookshelf"),
                              True, "shelf-after-remove", 1)
        if after_remove[0]["group"] != first_id:
            raise AssertionError("bulk group removal did not restore the first bit")
        expect(add("delete-second", owner, "POST", "/reader3/deleteBookGroup",
                   {"groupId": second_id, "groupName": "Beta"}), True, "delete-second")
        expect(add("after-delete", owner, "GET", "/reader3/getBookGroups"),
               True, "after-delete", 6)

        user_dir = workdir / "storage/data" / username
        group_file = user_dir / "bookGroup.json"
        shelf_file = user_dir / "bookshelf.json"
        groups = json.loads(group_file.read_text(encoding="utf-8"))
        books = normalize(json.loads(shelf_file.read_text(encoding="utf-8")))
        if len(groups) != 6 or len(books) != 1 or books[0]["group"] != first_id:
            raise AssertionError("group or bookshelf storage is inconsistent")
        return {"probes": probes, "groups": groups, "books": books}
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
    username = "groupprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-groups-diff-") as temp:
        root = Path(temp)
        original = run_one(ORIGINAL, root / "original", username, other_user, password)
        restored = run_one(RESTORED, root / "restored", username, other_user, password)
    if len(original["probes"]) != len(restored["probes"]):
        raise AssertionError("probe count differs")
    comparisons = []
    for left, right in zip(original["probes"], restored["probes"]):
        accepted, paths = accepted_default_group_name_corruption(
            left, right, "body.data") if left["probe"] in {
                "after-first", "after-second", "after-update", "after-order", "after-delete"
            } else (False, different_paths(left, right))
        comparisons.append({"probe": left["probe"], "equal": left == right,
                            "acceptedDivergence": accepted, "differentPaths": paths,
                            "original": left, "restored": right})
    group_storage_accepted, group_storage_paths = accepted_default_group_name_corruption(
        original["groups"], restored["groups"], "")
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons,
              "groupStorageEqual": original["groups"] == restored["groups"],
              "groupStorageAcceptedDivergence": group_storage_accepted,
              "groupStorageDifferentPaths": group_storage_paths,
              "bookshelfStorageEqual": original["books"] == restored["books"],
              "storage": {"original": {"groups": original["groups"], "books": original["books"]},
                          "restored": {"groups": restored["groups"], "books": restored["books"]}}}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        outcome = "equal" if row["equal"] else "accepted old-JAR text corruption" if row["acceptedDivergence"] else "DIFFERENT"
        print(f"{row['probe']}: {outcome}")
    print(f"Group storage: {'equal' if report['groupStorageEqual'] else 'accepted old-JAR text corruption' if group_storage_accepted else 'DIFFERENT'}")
    print(f"Bookshelf storage: {'equal' if report['bookshelfStorageEqual'] else 'DIFFERENT'}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] or row["acceptedDivergence"] for row in comparisons) or not (
            (report["groupStorageEqual"] or group_storage_accepted)
            and report["bookshelfStorageEqual"]):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
