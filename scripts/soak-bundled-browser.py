"""Bounded generated-data soak inside one offline, isolated Reader container.

No production endpoint, external source, real account or real book is accepted.
The GitHub-hosted harness loads an already-tested native image without rebuilding.
"""

import argparse
import concurrent.futures
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import signal
import threading
import time
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


SPEC = importlib.util.spec_from_file_location(
    "soak_cgroup", Path(__file__).with_name("report-browser-cgroup.py"))
CGROUP = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CGROUP)
PROC = Path("/proc")


def process_kind(arguments):
    if len(arguments) >= 2 and re.fullmatch(
            r"/tmp/reader-camoufox-worker-[^/]+\.py", arguments[1]) and (
            Path(arguments[0]).name.startswith("python")):
        return "worker"
    if arguments and re.search(r"/camoufox/(?:[^/]+/)*[^/]+$", arguments[0]):
        return "browser"
    if arguments and arguments[0].endswith("/playwright/driver/node"):
        return "driver"
    return None


def browser_processes():
    found = {"worker": [], "browser": [], "driver": []}
    for directory in PROC.iterdir():
        if not directory.name.isdigit() or int(directory.name) <= 1:
            continue
        try:
            status = (directory / "status").read_text()
            uid = re.search(r"^Uid:\s+(\d+)", status, re.MULTILINE)
            if not uid or int(uid.group(1)) != 10001:
                continue
            arguments = [part.decode("utf-8", errors="replace") for part in
                         (directory / "cmdline").read_bytes().split(b"\0") if part]
            kind = process_kind(arguments)
            if kind:
                found[kind].append(int(directory.name))
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return found


def quiet_snapshot():
    deadline = time.monotonic() + 10
    while True:
        processes = browser_processes()
        if not any(processes.values()):
            report = CGROUP.collect_report("offline single-image generated soak; quiescent sample")
            CGROUP.verify_report(report, require_no_swap=True)
            if not isinstance(report["memoryCurrentBytes"], int) or report["memoryCurrentBytes"] < 0:
                raise RuntimeError("Missing current-memory observation for the soak trace")
            return {"memoryCurrentBytes": report["memoryCurrentBytes"],
                    "memoryPeakBytes": report["memoryPeakBytes"],
                    "pidsCurrent": report["pidsCurrent"], "pidsPeak": report["pidsPeak"],
                    "browserProcesses": {key: len(value) for key, value in processes.items()}}
        if time.monotonic() >= deadline:
            raise RuntimeError("Browser, driver or worker processes remained after a completed request")
        time.sleep(0.1)


def require_environment(base, revision, seconds, output):
    parsed = urllib.parse.urlsplit(base)
    if (parsed.scheme != "http" or parsed.hostname != "127.0.0.1" or
            parsed.port != 18892 or parsed.path not in ("", "/") or
            parsed.username or parsed.password or parsed.query or parsed.fragment):
        raise ValueError("Soak requires its dedicated loopback port 18892")
    if not re.fullmatch(r"[0-9a-f]{40}", revision) or not 60 <= seconds <= 3600:
        raise ValueError("Invalid expected image revision or bounded soak duration")
    if not hasattr(os, "getuid") or os.getuid() != 10001:
        raise RuntimeError("Soak must run as the image's non-root Reader user")
    if {entry.name for entry in Path("/sys/class/net").iterdir()} != {"lo"}:
        raise RuntimeError("Soak requires a network namespace containing only loopback")
    if output.resolve() != Path("/verification-output") or not output.is_dir():
        raise ValueError("Soak reports require the dedicated mounted output directory")
    if (output / "SOAK_REPORT.json").exists() or (output / "SOAK_TRACE.jsonl").exists():
        raise ValueError("Refusing to overwrite an earlier soak report")
    CGROUP.verify_report(CGROUP.collect_report(), require_no_swap=True)


def request_json(opener, base, path, body=None):
    data = None if body is None else json.dumps(body, ensure_ascii=False).encode("utf-8")
    request = urllib.request.Request(base + path, data=data,
                                     headers={"Content-Type": "application/json; charset=utf-8"})
    with opener.open(request, timeout=40) as response:
        raw = response.read(2 * 1024 * 1024 + 1)
        if response.status != 200 or len(raw) > 2 * 1024 * 1024:
            raise RuntimeError("Unexpected or excessive Reader HTTP response")
        return json.loads(raw.decode("utf-8"))


def successful(value):
    if value.get("isSuccess") is not True or value.get("errorMsg") != "":
        raise RuntimeError("Reader operation did not return a successful ReturnData")
    return value.get("data")


class Fixture(ThreadingHTTPServer):
    def __init__(self):
        super().__init__(("127.0.0.1", 0), Handler)
        self.lock = threading.Lock()
        self.release_stalls = threading.Event()
        self.crash_requested = threading.Event()
        self.markers = [secrets.token_hex(12) for _ in range(4)]
        self.seen = [0] * 4
        self.errors = []
        self.methods = {"GET": 0, "POST": 0}


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.serve("GET")

    def do_POST(self):
        self.serve("POST")

    def serve(self, method):
        url = urllib.parse.urlsplit(self.path)
        query = urllib.parse.parse_qs(url.query)
        if url.path not in ("/healthy", "/stall", "/crash") or query.get("account") not in (
                ["0"], ["1"], ["2"], ["3"]):
            self.send_error(404)
            return
        index = int(query["account"][0])
        length = int(self.headers.get("Content-Length", "0"))
        if not 0 <= length <= 65536:
            self.send_error(400)
            return
        body = self.rfile.read(length).decode("utf-8") if method == "POST" else None
        with self.server.lock:
            expected_cookie = ("soak_session=" + self.server.markers[index]
                               if self.server.seen[index] else "")
            if self.headers.get("Cookie", "") != expected_cookie:
                self.server.errors.append("cookieIsolationOrReplayMismatch")
            if (self.headers.get("X-Soak-Account") != str(index) or
                    (method == "POST" and body != f"account={index}")):
                self.server.errors.append("requestShapeMismatch")
            self.server.seen[index] += 1
            self.server.methods[method] += 1
        if url.path in ("/stall", "/crash"):
            if url.path == "/crash":
                self.server.crash_requested.set()
            self.server.release_stalls.wait(45)
        data = (f"<html><div class='book'><a href='/book/{index}'>"
                f"<span class='name'>SoakUser{index}</span></a>"
                "<span class='author'>生成测试</span></div></html>").encode("utf-8")
        try:
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Set-Cookie", "soak_session=" + self.server.markers[index] +
                             "; Path=/; HttpOnly; SameSite=Lax")
            self.end_headers()
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass  # The intentional stalled document has already been cancelled.

    def log_message(self, _format, *_args):
        pass


def source_for(fixture_base, index, method="GET", path="/healthy", hang_script=False):
    options = {"webView": True, "headers": {"X-Soak-Account": str(index)}}
    if method == "POST":
        options.update(method="POST", body=f"account={index}")
    if hang_script:
        options["webJs"] = "(() => { for (;;) {} })()"
    return {
        "bookSourceUrl": fixture_base, "bookSourceName": "Offline bounded browser soak",
        "enabledCookieJar": True,
        "searchUrl": fixture_base + path + f"?account={index}, " + json.dumps(options),
        "ruleSearch": {"bookList": ".book", "name": ".name@text",
                       "author": ".author@text", "bookUrl": "a@href"},
        "ruleToc": {"chapterList": ".chapter"}, "ruleContent": {"content": ".content@html"},
    }


def validate_fault(value, kind):
    expected = "异常退出" if kind == "workerExit" else ("TimeoutError", "超时")
    message = value.get("errorMsg", "")
    if (value.get("isSuccess") is not False or not isinstance(message, str) or
            not any(part in message for part in ((expected,) if isinstance(expected, str) else expected))):
        raise RuntimeError("Injected fault did not produce the expected failed ReturnData")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reader-base", default="http://127.0.0.1:18892")
    parser.add_argument("--expected-revision", required=True)
    parser.add_argument("--seconds", type=int, default=1800)
    parser.add_argument("--output", type=Path, default=Path("/verification-output"))
    args = parser.parse_args()
    require_environment(args.reader_base, args.expected_revision, args.seconds, args.output)
    fixture = Fixture()
    thread = threading.Thread(target=fixture.serve_forever, daemon=True)
    thread.start()
    fixture_base = f"http://127.0.0.1:{fixture.server_port}"
    accounts = []
    state = {"scope": "one offline native full Reader image; generated data only",
             "expectedImageRevision": args.expected_revision, "requestedSeconds": args.seconds,
             "accounts": 4, "rounds": 0, "successfulSearches": 0, "faults": [],
             "samples": [], "passed": False}
    phase = "setup"
    started = time.monotonic()
    try:
        for index in range(4):
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}),
                urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
            login = {"username": "soak" + secrets.token_hex(8),
                     "password": "Probe-" + secrets.token_hex(16)}
            for is_login in (False, True):
                successful(request_json(opener, args.reader_base, "/reader3/login",
                                        dict(login, isLogin=is_login)))
            if successful(request_json(opener, args.reader_base, "/reader3/getUserInfo")).get("secure") is not True:
                raise RuntimeError("Soak requires secure, isolated Reader accounts")
            accounts.append(opener)
            successful(request_json(opener, args.reader_base, "/reader3/saveBookSource",
                                    source_for(fixture_base, index)))
        release = request_json(accounts[0], args.reader_base, "/assets/reader-release.json")
        if release.get("buildRevision") != args.expected_revision:
            raise RuntimeError("The running Reader revision differs from the expected tested image")
        state["releaseIdentity"] = release

        def search(index, check=True):
            began = time.monotonic()
            value = request_json(accounts[index], args.reader_base, "/reader3/searchBook",
                                 {"key": "generated", "page": 1, "bookSourceUrl": fixture_base})
            if check:
                books = successful(value)
                if not isinstance(books, list) or len(books) != 1 or books[0].get("name") != f"SoakUser{index}":
                    raise RuntimeError("User-specific generated book projection mismatch")
            return value, time.monotonic() - began

        phase = "cold-request"
        _, state["coldRequestSeconds"] = search(0)
        state["successfulSearches"] += 1
        state["coldQuiescent"] = quiet_snapshot()
        phase = "warmup"
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            list(pool.map(search, range(4)))
        state["successfulSearches"] += 4
        state["warmQuiescent"] = quiet_snapshot()

        for kind, path, hang in (("navigationTimeout", "/stall", False),
                                 ("scriptWatchdog", "/healthy", True),
                                 ("workerExit", "/crash", False)):
            phase = kind
            successful(request_json(accounts[0], args.reader_base, "/reader3/saveBookSource",
                                    source_for(fixture_base, 0, path=path, hang_script=hang)))
            killed = False
            if kind == "workerExit":
                with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
                    pending = pool.submit(search, 0, False)
                    if not fixture.crash_requested.wait(15):
                        raise RuntimeError("Crash fixture did not observe an active browser navigation")
                    workers = browser_processes()["worker"]
                    if len(workers) != 1:
                        raise RuntimeError("Refusing to kill anything other than one owned active worker")
                    # Only the non-root worker in this network-none container is killed.
                    os.kill(workers[0], signal.SIGKILL)
                    killed = True
                    value, seconds = pending.result(timeout=40)
            else:
                value, seconds = search(0, False)
            validate_fault(value, kind)
            quiet_snapshot()
            successful(request_json(accounts[0], args.reader_base, "/reader3/saveBookSource",
                                    source_for(fixture_base, 0)))
            search(0)
            state["successfulSearches"] += 1
            state["faults"].append({"kind": kind, "failedReturnDataVerified": True,
                                   "seconds": round(seconds, 3), "ownedWorkerKilled": killed,
                                   "healthyRecoveryVerified": True, "afterRecovery": quiet_snapshot()})

        phase = "continuous-rounds"
        soak_started = time.monotonic()
        with (args.output / "SOAK_TRACE.jsonl").open("x", encoding="utf-8") as trace:
            for round_index in range(1000):
                method = "GET" if round_index % 2 == 0 else "POST"
                for index, account in enumerate(accounts):
                    successful(request_json(account, args.reader_base, "/reader3/saveBookSource",
                                            source_for(fixture_base, index, method)))
                with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
                    results = list(pool.map(search, range(4)))
                snapshot = quiet_snapshot()
                if fixture.errors:
                    raise RuntimeError("Target observed a Cookie isolation or request shape mismatch")
                state["rounds"] += 1
                state["successfulSearches"] += 4
                elapsed = time.monotonic() - soak_started
                sample = {"round": state["rounds"], "method": method,
                          "elapsedSeconds": round(elapsed, 3),
                          "maxRequestSeconds": round(max(value[1] for value in results), 3), **snapshot}
                trace.write(json.dumps(sample, sort_keys=True) + "\n")
                trace.flush()
                state["samples"] = (state["samples"] + [sample])[-5:]
                if round_index % 10 == 0:
                    print(json.dumps({"soakProgress": sample}), flush=True)
                if elapsed >= args.seconds and state["rounds"] >= 20:
                    break
            else:
                raise RuntimeError("Round ceiling reached before the required continuous duration")
        state["continuousSeconds"] = round(time.monotonic() - soak_started, 3)
        state["finalQuiescent"] = quiet_snapshot()
        state["resources"] = CGROUP.collect_report(state["scope"])
        CGROUP.verify_report(state["resources"], require_no_swap=True)
        state["targetMethods"] = fixture.methods
        state["targetCookieOrShapeErrors"] = len(fixture.errors)
        state["passed"] = True
    except BaseException as error:
        state["failedPhase"] = phase
        state["failureType"] = type(error).__name__
        # No server payloads, account names, tokens or cookies are emitted on failure.
        state["resources"] = CGROUP.collect_report(state["scope"])
        state["remainingBrowserProcesses"] = {key: len(value) for key, value in browser_processes().items()}
        raise
    finally:
        state["totalSeconds"] = round(time.monotonic() - started, 3)
        state["targetCookieOrShapeErrors"] = len(fixture.errors)
        with (args.output / "SOAK_REPORT.json").open("x", encoding="utf-8") as output:
            json.dump(state, output, ensure_ascii=False, indent=2)
            output.write("\n")
        fixture.release_stalls.set()
        fixture.shutdown()
        fixture.server_close()
        thread.join(timeout=5)
    print(json.dumps({"soakCompleted": True, "rounds": state["rounds"],
                      "continuousSeconds": state["continuousSeconds"]}), flush=True)


if __name__ == "__main__":
    main()
