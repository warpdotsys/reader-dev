#!/usr/bin/env python3
"""Compare user-file path handling through a disposable directory link.

The link and its target both live under a temporary test workdir. No production
data, existing storage, or third-party service is accessed.
"""

import hashlib
import http.cookiejar
import json
import os
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
REPORT = ROOT / "reports/file-link-boundary-diff-latest.json"
SENTINEL = "outside-user-home-sentinel"


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def request(opener, base, path, body=None):
    data = json.dumps(body, ensure_ascii=False).encode("utf-8") if body is not None else None
    req = urllib.request.Request(
        base + path, data=data, method="POST" if body is not None else "GET",
        headers={"Content-Type": "application/json; charset=utf-8"} if data else {},
    )
    try:
        response = opener.open(req, timeout=25)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        return {"status": response.status,
                "body": json.loads(response.read().decode("utf-8"))}


def make_directory_link(link, target, workdir, marker):
    # Both ends must remain inside this test's disposable workdir. The target is
    # outside the synthetic user's home, which is the boundary under test.
    link.relative_to(workdir)
    target.relative_to(workdir)
    if link.exists() or link.is_symlink():
        raise RuntimeError("test link already exists")
    try:
        os.symlink(target, link, target_is_directory=True)
    except OSError:
        if os.name != "nt":
            raise
        env = os.environ.copy()
        env["CODEX_TEST_LINK"] = str(link)
        env["CODEX_TEST_TARGET"] = str(target)
        subprocess.run(
            ["pwsh", "-NoProfile", "-Command",
             "New-Item -ItemType Junction -Path $env:CODEX_TEST_LINK "
             "-Target $env:CODEX_TEST_TARGET -ErrorAction Stop | Out-Null"],
            env=env, check=True, capture_output=True, text=True,
        )
    if not link.is_dir() or not (link / marker).is_file():
        raise RuntimeError("directory link did not resolve to fixture")


def run_one(jar, workdir, username, password):
    workdir.mkdir()
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
    log_path = workdir / "reader.log"
    with log_path.open("wb") as output:
        process = subprocess.Popen(
            [str(JAVA), "-Xms128m", "-Xmx768m", "-jar", str(jar),
             f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
             "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
             "--spring.profiles.active=prod"],
            cwd=ROOT, stdout=output, stderr=subprocess.STDOUT, creationflags=flags,
        )
        opener = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        try:
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"reader exited during startup: {process.returncode}")
                try:
                    if request(opener, base, "/reader3/getSystemInfo")["status"] == 200:
                        break
                except (OSError, ValueError, urllib.error.URLError):
                    pass
                time.sleep(0.4)
            else:
                raise TimeoutError("reader startup timeout")

            for is_login in (False, True):
                result = request(opener, base, "/reader3/login",
                                 {"username": username, "password": password,
                                  "isLogin": is_login})
                if result["status"] != 200 or result["body"].get("isSuccess") is not True:
                    raise RuntimeError(f"fixture login failed: {result}")

            home = workdir / "storage" / "data" / username
            home.mkdir(parents=True, exist_ok=True)
            outside = workdir / "storage" / "data" / "link-target-fixture"
            outside.mkdir(parents=True, exist_ok=True)
            (outside / "sentinel.txt").write_text(SENTINEL, encoding="utf-8")
            make_directory_link(home / "linked", outside, workdir, "sentinel.txt")
            internal = home / "internal-target"
            internal.mkdir()
            (internal / "ordinary.txt").write_text("inside-user-home", encoding="utf-8")
            make_directory_link(home / "internal-link", internal, workdir, "ordinary.txt")

            encoded = urllib.parse.quote("/linked/sentinel.txt", safe="/")
            get_result = request(opener, base,
                                 f"/reader3/file/get?home=__HOME__&path={encoded}")
            save_result = request(opener, base, "/reader3/file/save",
                                  {"home": "__HOME__", "path": "/linked/created.txt",
                                   "content": "write-through-link"})
            internal_get = request(opener, base,
                                   "/reader3/file/get?home=__HOME__&path=/internal-link/ordinary.txt")
            internal_save = request(opener, base, "/reader3/file/save",
                                    {"home": "__HOME__", "path": "/internal-link/new.txt",
                                     "content": "inside-link-write"})
            created = outside / "created.txt"
            return {"get": get_result, "save": save_result,
                    "internalGet": internal_get, "internalSave": internal_save,
                    "internalWrite": (internal / "new.txt").read_text(encoding="utf-8")
                    if (internal / "new.txt").exists() else None,
                    "sentinelIntact": (outside / "sentinel.txt").read_text(encoding="utf-8") == SENTINEL,
                    "wroteOutsideHome": created.exists(),
                    "writtenValue": created.read_text(encoding="utf-8") if created.exists() else None}
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def main():
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    username = "linkprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-file-link-diff-") as temporary:
        root = Path(temporary)
        original = run_one(ORIGINAL, root / "original", username, password)
        restored = run_one(RESTORED, root / "restored", username, password)
    report = {"originalJarSha256": sha256(ORIGINAL),
              "restoredJarSha256": sha256(RESTORED),
              "original": original, "restored": restored}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Original link read/write: {original['get']['body'].get('isSuccess')}/"
          f"{original['wroteOutsideHome']}")
    print(f"Restored link read/write: {restored['get']['body'].get('isSuccess')}/"
          f"{restored['wroteOutsideHome']}")
    print(f"Report: {REPORT}")
    if not (original["get"]["body"].get("data") == SENTINEL
            and original["wroteOutsideHome"] and original["sentinelIntact"]
            and restored["get"]["body"].get("isSuccess") is False
            and not restored["wroteOutsideHome"] and restored["sentinelIntact"]
            and all(result["internalGet"]["body"].get("data") == "inside-user-home"
                    and result["internalSave"]["body"].get("isSuccess") is True
                    and result["internalWrite"] == "inside-link-write"
                    for result in (original, restored))):
        raise SystemExit("Unexpected directory-link boundary behavior")


if __name__ == "__main__":
    main()
