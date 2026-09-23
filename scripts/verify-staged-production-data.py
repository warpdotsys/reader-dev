"""Run a bounded auth/reading probe against an isolated copy on the production host.

Usage: pipe this file to `ssh root@host python3 - --namespace NAME --source-index N`.
The script never prints tokens, source rules, URLs, book titles or chapter text.
The production container and original storage are not mounted in the probe.
"""

import argparse
import hashlib
import http.cookiejar
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


APP_DIR = Path("/opt/reader-pro-restored")
PRODUCTION_DATA = APP_DIR / "storage/data"
IMAGE = "medwarp/reader-pro-restored:4.0.7"


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def write_private_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False), encoding="utf-8")
    os.chown(path, 10001, 10001)
    path.chmod(0o600)


def request(opener, base, path, body=None, timeout=45):
    payload = json.dumps(body, ensure_ascii=False).encode("utf-8") if body is not None else None
    headers = {"Content-Type": "application/json; charset=utf-8"} if payload else {}
    req = urllib.request.Request(base + path, data=payload, headers=headers,
                                 method="POST" if payload else "GET")
    with opener.open(req, timeout=timeout) as response:
        return response.status, json.loads(response.read().decode("utf-8"))


def checked(status, value, step):
    if status != 200 or value.get("isSuccess") is not True:
        raise RuntimeError(f"{step}: HTTP {status}, success={value.get('isSuccess')}")
    return value.get("data")


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--namespace", required=True)
    parser.add_argument("--source-index", type=int, required=True)
    parser.add_argument("--query", default="斗破苍穹")
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9_-]+", args.namespace):
        raise ValueError("Invalid storage namespace")
    if args.source_index < 0:
        raise ValueError("Source index must be nonnegative")

    deployed_image = subprocess.run(
        ["docker", "inspect", "-f", "{{.Image}}", "reader-pro-restored"],
        text=True, capture_output=True, timeout=10, check=True,
    ).stdout.strip()
    tagged_image = subprocess.run(
        ["docker", "image", "inspect", "-f", "{{.Id}}", IMAGE],
        text=True, capture_output=True, timeout=10, check=True,
    ).stdout.strip()
    if deployed_image != tagged_image:
        raise RuntimeError("Staging image differs from deployed container image")

    users_file = PRODUCTION_DATA / "users.json"
    source_file = PRODUCTION_DATA / args.namespace / "bookSource.json"
    shelf_file = PRODUCTION_DATA / args.namespace / "bookshelf.json"
    original_files = (users_file, source_file, shelf_file)
    original_hashes = {str(path): digest(path) for path in original_files}
    users = json.loads(users_file.read_text(encoding="utf-8"))
    user = users.get(args.namespace)
    if not user or not user.get("token"):
        raise RuntimeError("Selected account or active token is unavailable")
    sources = json.loads(source_file.read_text(encoding="utf-8"))
    source = sources[args.source_index]
    if any(source.get(key) for key in ("header", "loginUrl", "loginUi", "loginCheckJs")):
        raise RuntimeError("Selected source has login or header configuration")
    shelf = json.loads(shelf_file.read_text(encoding="utf-8"))

    stage = Path(tempfile.mkdtemp(prefix="auth-reading-", dir=APP_DIR))
    container = "reader-pro-verify-" + secrets.token_hex(4)
    launched = False
    result = {}
    try:
        os.chown(stage, 10001, 10001)
        stage.chmod(0o700)
        storage = stage / "storage"
        logs = stage / "logs"
        for directory in (storage, storage / "data", storage / "data" / args.namespace, logs):
            directory.mkdir(parents=True, exist_ok=True)
            os.chown(directory, 10001, 10001)
            directory.chmod(0o700)
        write_private_json(storage / "data/users.json", {args.namespace: user})
        write_private_json(storage / "data" / args.namespace / "bookSource.json", [source])
        write_private_json(storage / "data" / args.namespace / "bookshelf.json", shelf)

        port = free_port()
        base = f"http://127.0.0.1:{port}"
        launched = True
        subprocess.run([
            "docker", "run", "--pull=never", "--rm", "-d", "--name", container,
            "--cpus=2", "--memory=1g", "--security-opt=no-new-privileges",
            "--cap-drop=ALL", "--tmpfs", "/tmp:size=256m,mode=1777",
            "-p", f"127.0.0.1:{port}:8080",
            "-v", f"{storage}:/storage", "-v", f"{logs}:/logs",
            "-e", "SPRING_PROFILES_ACTIVE=prod", "-e", "READER_APP_WORKDIR=/",
            "-e", "READER_SERVER_PORT=8080", "-e", "READER_APP_SECURE=true",
            "-e", "READER_APP_LICENSECHECKENABLED=false", IMAGE,
        ], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
            timeout=30, check=True)
        opener = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            try:
                status, value = request(opener, base, "/reader3/getSystemInfo", timeout=2)
                if status == 200 and value.get("isSuccess"):
                    break
            except (OSError, ValueError, urllib.error.URLError):
                pass
            time.sleep(0.5)
        else:
            raise TimeoutError("Isolated reader did not become ready")

        token = args.namespace + ":" + user["token"]
        token_path = "/reader3/getUserInfo?" + urllib.parse.urlencode({"accessToken": token})
        status, value = request(opener, base, token_path)
        checked(status, value, "token-auth")
        token = None
        status, value = request(opener, base, "/reader3/getBookshelf")
        books = checked(status, value, "bookshelf")
        if len(books) != len(shelf):
            raise RuntimeError("Imported bookshelf count differs from source file")

        status, value = request(opener, base, "/reader3/searchBook",
                                {"key": args.query, "page": 1, "bookSource": source})
        found = checked(status, value, "search")
        if not found:
            raise RuntimeError("Real source returned no search results")
        book_url = found[0]["bookUrl"]
        status, value = request(opener, base, "/reader3/getBookInfo",
                                {"url": book_url, "bookSource": source})
        info = checked(status, value, "book-info")
        if not info.get("tocUrl"):
            raise RuntimeError("Book info has no TOC URL")
        status, value = request(opener, base, "/reader3/getChapterList",
                                {"url": book_url, "bookSource": source, "refresh": 1})
        chapters = checked(status, value, "chapter-list")
        if not chapters:
            raise RuntimeError("Chapter list is empty")
        status, value = request(opener, base, "/reader3/getBookContent",
                                {"url": book_url, "bookSource": source, "index": 0,
                                 "cache": 1, "refresh": 1}, timeout=60)
        content = checked(status, value, "content")
        if not isinstance(content, str) or not content:
            raise RuntimeError("Content is empty or not text")

        after_hashes = {str(path): digest(path) for path in original_files}
        if after_hashes != original_hashes:
            raise RuntimeError("Original production data changed during isolated probe")
        result = {
            "auth": "ok", "bookshelfCount": len(books), "searchCount": len(found),
            "chapterCount": len(chapters), "contentLength": len(content),
            "contentSha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
            "originalProductionDataUnchanged": True,
        }
    finally:
        try:
            if launched:
                subprocess.run(["docker", "rm", "-f", container],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=20)
        finally:
            resolved_stage = stage.resolve()
            if (resolved_stage.parent != APP_DIR.resolve() or
                    not resolved_stage.name.startswith("auth-reading-")):
                raise RuntimeError("Refusing to remove unexpected staging path")
            shutil.rmtree(resolved_stage)
    result["stagingCleaned"] = not stage.exists()
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
