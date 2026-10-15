"""Generated-only Reader thread-group diagnostic, never an original/private-data probe."""
import sys

if sys.flags.optimize:
    raise RuntimeError("Refuses disabled safety assertions")

import argparse
from collections import Counter
import hashlib
import http.cookiejar
import json
import os
from pathlib import Path
import re
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

ORIGINAL_SHA = "b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c"
GROUPS = ("vertxEventLoop", "vertxInternalWorker", "vertxWorker", "coroutineDispatcher",
    "readerBrowser", "okHttp", "forkJoin", "scheduler", "jvmRuntime", "other")
STARTUP_MARKERS = ("Read-only file system", "Permission denied", "NoSuchFileException",
    "BeanCreationException", "Application run failed", "OutOfMemoryError", "unable to create native thread")


def startup_markers(raw):
    if not isinstance(raw, bytes) or len(raw) > 4 * 1024**2:
        raise ValueError("Unbounded private startup log")
    return {name: name.encode("ascii") in raw for name in STARTUP_MARKERS}


def thread_group(name):
    if not isinstance(name, str) or not name or len(name.encode("utf-8")) > 64:
        raise ValueError("Invalid bounded thread name")
    for prefix, group in (("vert.x-eventloo", "vertxEventLoop"), ("vert.x-interna", "vertxInternalWorker"),
            ("vert.x-worker", "vertxWorker"), ("DefaultDispat", "coroutineDispatcher"),
            ("reader-browser", "readerBrowser"), ("reader-camoufox", "readerBrowser"),
            ("OkHttp", "okHttp"), ("ForkJoinPool", "forkJoin"),
            ("scheduling-", "scheduler"), ("pool-", "scheduler")):
        if name.startswith(prefix): return group
    if name == "java" or name.startswith(("GC Thread", "G1 ", "VM Thread", "VM Periodic", "C1 Compiler",
            "C2 Compiler", "Sweeper", "Signal Dispatch", "Reference Handl", "Finalizer", "Common-Cleaner",
            "Service Thread", "Attach Listener")):
        return "jvmRuntime"
    return "other"


def summarize_threads(task_root):
    counts = Counter({key: 0 for key in GROUPS})
    disappeared = 0
    tasks = list(task_root.iterdir())
    if not 1 <= len(tasks) <= 512: raise RuntimeError("Unbounded thread inventory")
    for task in tasks:
        if not task.name.isdigit() or task.is_symlink(): raise RuntimeError("Unexpected task inventory")
        try:
            with (task / "comm").open("rb") as stream: raw = stream.read(65)
        except FileNotFoundError:
            disappeared += 1
            continue
        if not raw or len(raw) > 64: raise RuntimeError("Invalid bounded thread observation")
        counts[thread_group(raw.decode("utf-8").rstrip("\n"))] += 1
    return {"observedThreads": sum(counts.values()), "disappearedDuringRead": disappeared,
        "groups": dict(counts), "rawThreadNamesExported": False}


def validate_inputs(unit, expected_sha):
    if not isinstance(unit, str) or re.fullmatch(r"reader-thread-profile-[0-9]{8}-[a-z][.]service", unit) is None:
        raise ValueError("Expected an owned isolated thread-profile unit")
    if not isinstance(expected_sha, str) or re.fullmatch(r"[0-9a-f]{64}", expected_sha) is None or expected_sha == ORIGINAL_SHA:
        raise ValueError("Expected independently verified restored JAR, never the original")


def reader_command(jar, store):
    # Spring 5.1.8 rejects --name= with an empty value. Keep the packaged
    # empty invite-code default instead of passing an invalid empty option.
    return ["/opt/java/openjdk/bin/java", "-Xms256m", "-Xmx768m", "-jar", str(jar),
        "--reader.app.workDir=" + str(store), "--reader.server.port=18890",
        "--reader.server.bindAddress=127.0.0.1", "--reader.app.secure=true",
        "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod",
        "--reader.app.webviewRenderer=camoufox", "--reader.app.camoufoxPythonExecutable=/usr/bin/python3"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args):
        raise RuntimeError("Unexpected redirect in local generated diagnostic")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--unit", required=True)
    parser.add_argument("--expected-jar-sha256", required=True)
    args = parser.parse_args(argv)
    validate_inputs(args.unit, args.expected_jar_sha256)
    st = dict(line.split(":", 1) for line in Path("/proc/self/status").read_text().splitlines() if ":" in line)
    assert os.getuid() == os.getgid() == 10001 and os.getgroups() == []
    assert st["NoNewPrivs"].strip() == "1" and all(int(st[k].strip(), 16) == 0 for k in
        ("CapEff", "CapPrm", "CapBnd", "CapInh", "CapAmb"))
    assert os.sched_getaffinity(0) == {0, 1} and sorted(n for _, n in socket.if_nameindex()) == ["lo"]
    assert Path("/proc/self/cgroup").read_text().strip().split("::", 1)[1] == "/system.slice/" + args.unit + "/server"
    jar = Path("/verification/reader.jar")
    value = hashlib.sha256()
    with jar.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""): value.update(block)
    assert value.hexdigest() == args.expected_jar_sha256
    result = {"scope": "current Reader generated HTTP baseline thread groups only", "passed": False,
        "jarSha256": value.hexdigest(), "samples": [], "browserStarted": False,
        "privateDataRead": False, "rawThreadNamesExported": False, "runtimePolicyChanged": False,
        "realCredentialsImported": False, "productFixAccepted": False, "failureCode": None, "phase": "startup",
        "rendererSelection": "camoufox", "generatedWorkingDirectoryOnly": True}
    cookies = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(), urllib.request.HTTPCookieProcessor(cookies))
    base = "http://127.0.0.1:18890"

    def request(path, data=None):
        raw = None if data is None else json.dumps(data).encode("utf-8")
        req = urllib.request.Request(base + path, data=raw, headers={} if raw is None else {"Content-Type": "application/json"})
        with opener.open(req, timeout=10) as response:
            body = response.read(131073)
            assert response.status == 200 and len(body) <= 131072 and response.geturl() == base + path
            return json.loads(body)

    def sample(child, phase):
        assert child.poll() is None and len(result["samples"]) < 12
        observed = summarize_threads(Path("/proc") / str(child.pid) / "task")
        result["samples"].append({"phase": phase, **observed})

    with tempfile.TemporaryDirectory(prefix="reader-generated-thread-profile-", dir="/tmp") as directory:
        work = Path(directory)
        store = work / "store"
        store.mkdir(mode=0o700)
        log_path = work / "private-startup.log"
        child = None
        try:
            with log_path.open("xb") as log:
                child = subprocess.Popen(reader_command(jar, store),
                    cwd=work, stdin=subprocess.DEVNULL, stdout=log, stderr=log)
                result["phase"] = "readiness"
                deadline = time.monotonic() + 35
                while time.monotonic() < deadline:
                    assert child.poll() is None and log_path.stat().st_size <= 4 * 1024**2
                    try:
                        health = request("/reader3/getSystemInfo")
                        assert health["isSuccess"] is True
                        break
                    except (OSError, urllib.error.URLError): time.sleep(.1)
                else: raise TimeoutError("Owned generated Reader readiness deadline")
                sample(child, "ready")
                username, password = "generatedthread" + secrets.token_hex(5), "Generated-" + secrets.token_hex(12)
                for phase, is_login in (("registered", False), ("loggedIn", True)):
                    result["phase"] = phase
                    reply = request("/reader3/login", {"username": username, "password": password, "isLogin": is_login})
                    assert reply["isSuccess"] is True and reply["errorMsg"] == ""
                    sample(child, phase)
                for phase, path in (("emptyShelf", "/reader3/getBookshelf"), ("defaultGroups", "/reader3/getBookGroups")):
                    result["phase"] = phase
                    reply = request(path)
                    assert reply["isSuccess"] is True and reply["errorMsg"] == ""
                    sample(child, phase)
                time.sleep(5)
                result["phase"] = "idleAfterFiveSeconds"
                sample(child, "idleAfterFiveSeconds")
                result["phase"] = "logout"
                reply = request("/reader3/logout", {})
                assert reply["isSuccess"] is True and request("/reader3/getBookshelf")["isSuccess"] is False
                sample(child, "loggedOut")
                result["generatedAuthenticationAndLogoutAccepted"] = True
                result["passed"] = True
                result["phase"] = "complete"
        except Exception as error:
            result["failureCode"] = "timeout" if isinstance(error, (TimeoutError, subprocess.TimeoutExpired)) else "validation" if isinstance(error, (AssertionError, ValueError, TypeError)) else "other"
        finally:
            if child is not None and child.poll() is None:
                child.terminate()
                try: child.wait(timeout=8)
                except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=5)
            result["ownedJavaStopped"] = child is not None and child.poll() is not None
            result["ownedJavaExitCode"] = child.returncode if child is not None else None
            result["privateStartupLogBytes"] = log_path.stat().st_size if log_path.exists() else 0
            assert result["privateStartupLogBytes"] <= 4 * 1024**2
            result["finiteStartupMarkers"] = startup_markers(log_path.read_bytes()) if log_path.exists() else {}
            result["rawStartupLogExported"] = False
    print(json.dumps({"readerThreadObservation": result}), flush=True)
    return 0 if result["passed"] and result["ownedJavaStopped"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
