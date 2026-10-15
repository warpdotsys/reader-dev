#!/usr/bin/env python3
"""Bounded loopback smoke test for a freshly built license-server fat JAR.

It deliberately creates only synthetic mailboxes and keys.  Its JSON report
contains booleans, counts and the JAR digest only; it never writes a trial
code, license key, mail content, or generated admin token to stdout.
The caller owns deletion of the newly-created runtime root; this program never
deletes a path supplied by a caller.
"""

import argparse
import base64
from email import policy
from email.parser import BytesParser
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import signal
import socket
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
from urllib.error import HTTPError
from urllib.request import ProxyHandler, Request, build_opener


MAX_JAR_BYTES = 128 * 1024 * 1024
MAX_HTTP_BODY_BYTES = 128 * 1024
MAX_REQUEST_BYTES = 4 * 1024
MAX_SMTP_MESSAGE_BYTES = 64 * 1024
REQUEST_TIMEOUT_SECONDS = 5
STARTUP_TIMEOUT_SECONDS = 20
TOTAL_TIMEOUT_SECONDS = 45
CODE = re.compile(r"(?<![0-9])([0-9]{8})(?![0-9])")
SHA256 = re.compile(r"[0-9a-f]{64}")


class SmokeFailure(RuntimeError):
    def __init__(self, stage: str, message: str, exit_code: int | None = None) -> None:
        super().__init__(message)
        self.stage = stage
        self.exit_code = exit_code


ACTIVE_PROCESSES: set[subprocess.Popen] = set()


def remaining_seconds(deadline: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise SmokeFailure("deadline", "total smoke time budget exceeded")
    return remaining


def reject_symlink_chain(path: Path, label: str) -> None:
    current = path.absolute()
    for item in (current, *current.parents):
        if item.exists() and item.is_symlink():
            raise SmokeFailure("preflight", label + " contains a symbolic link")


def validate_paths(jar: Path, root: Path, report: Path) -> None:
    if root.exists() or root.is_symlink() or not root.parent.is_dir():
        raise SmokeFailure("preflight", "smoke root must be a new child of an existing directory")
    reject_symlink_chain(root.parent, "smoke root parent")
    if report.exists() or report.is_symlink() or not report.parent.is_dir():
        raise SmokeFailure("preflight", "report path must be new under an existing directory")
    reject_symlink_chain(report.parent, "report parent")
    if jar.is_symlink():
        raise SmokeFailure("preflight", "fat JAR may not be a symbolic link")


def bounded_file_hash(path: Path) -> str:
    if not path.is_file() or path.is_symlink():
        raise SmokeFailure("preflight", "fat JAR is not a regular file")
    size = path.stat().st_size
    if not 0 < size <= MAX_JAR_BYTES:
        raise SmokeFailure("preflight", "fat JAR exceeds its size budget")
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def free_loopback_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


class MailSink:
    def __init__(self) -> None:
        self.messages = []
        self.lock = threading.Lock()

    def add(self, message: bytes) -> None:
        if len(message) > MAX_SMTP_MESSAGE_BYTES:
                raise SmokeFailure("smtp", "fake SMTP message exceeds its size budget")
        with self.lock:
            if len(self.messages) >= 2:
                raise SmokeFailure("smtp", "fake SMTP received too many messages")
            self.messages.append(message)


class FakeSmtpHandler(socketserver.StreamRequestHandler):
    def setup(self) -> None:
        super().setup()
        self.request.settimeout(REQUEST_TIMEOUT_SECONDS)

    def send(self, line: bytes) -> None:
        self.wfile.write(line + b"\r\n")
        self.wfile.flush()

    def handle(self) -> None:
        try:
            self.send(b"220 loopback test SMTP")
            in_data = False
            message = bytearray()
            while True:
                line = self.rfile.readline(MAX_SMTP_MESSAGE_BYTES + 1)
                if not line or len(line) > MAX_SMTP_MESSAGE_BYTES:
                    return
                line = line.rstrip(b"\r\n")
                if in_data:
                    if line == b".":
                        self.server.sink.add(bytes(message))
                        in_data = False
                        self.send(b"250 queued")
                    else:
                        if len(message) + len(line) + 2 > MAX_SMTP_MESSAGE_BYTES:
                            return
                        message.extend(line + b"\r\n")
                    continue
                upper = line.upper()
                if upper.startswith(b"EHLO"):
                    self.send(b"250-loopback")
                    self.send(b"250 SIZE 65536")
                elif upper.startswith(b"HELO") or upper.startswith(b"MAIL FROM") or upper.startswith(b"RCPT TO"):
                    self.send(b"250 ok")
                elif upper == b"DATA":
                    in_data = True
                    self.send(b"354 end with dot")
                elif upper == b"QUIT":
                    self.send(b"221 bye")
                    return
                else:
                    self.send(b"250 ok")
        except (OSError, socket.timeout):
            return


class LoopbackSmtp(socketserver.TCPServer):
    allow_reuse_address = True

    def __init__(self) -> None:
        self.sink = MailSink()
        super().__init__(("127.0.0.1", 0), FakeSmtpHandler)
        self.thread = threading.Thread(target=self.serve_forever, daemon=True)

    @property
    def port(self) -> int:
        return self.server_address[1]

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.shutdown()
        self.server_close()
        self.thread.join(timeout=REQUEST_TIMEOUT_SECONDS)


def isolated_java_environment(base: dict, root: Path, http_port: int, smtp_port: int, admin_token: str) -> dict:
    inherited = {name: base[name] for name in ("PATH", "HOME", "TMPDIR", "TEMP", "TMP", "SystemRoot", "WINDIR") if name in base}
    inherited.update({
        "LICENSE_BIND": "127.0.0.1",
        "LICENSE_PORT": str(http_port),
        "LICENSE_DATA_DIR": str(root / "data"),
        "LICENSE_ADMIN_TOKEN": admin_token,
        "LICENSE_ISSUER": "https://license.invalid",
        "LICENSE_LEGACY_RSA_ENABLED": "true",
        "LICENSE_SMTP_HOST": "127.0.0.1",
        "LICENSE_SMTP_PORT": str(smtp_port),
        "LICENSE_SMTP_MODE": "PLAINTEXT",
        "LICENSE_SMTP_ALLOW_PLAINTEXT_LOOPBACK": "true",
        "LICENSE_SMTP_FROM": "sender@example.test",
        "JAVA_TOOL_OPTIONS": "-Djava.net.useSystemProxies=false",
    })
    return inherited


def request_json(base: str, path: str, deadline: float, payload: dict | None = None) -> tuple[int, dict]:
    encoded = b"" if payload is None else json.dumps(payload, separators=(",", ":")).encode("utf-8")
    if len(encoded) > MAX_REQUEST_BYTES:
        raise SmokeFailure("http", "request exceeds its size budget")
    request = Request(base + path, data=encoded if payload is not None else None,
                      headers={"Content-Type": "application/json"} if payload is not None else {},
                      method="POST" if payload is not None else "GET")
    opener = build_opener(ProxyHandler({}))
    try:
        with opener.open(request, timeout=min(REQUEST_TIMEOUT_SECONDS, remaining_seconds(deadline))) as response:
            status = response.status
            raw = response.read(MAX_HTTP_BODY_BYTES + 1)
    except HTTPError as error:
        with error:
            status = error.code
            raw = error.read(MAX_HTTP_BODY_BYTES + 1)
    if len(raw) > MAX_HTTP_BODY_BYTES:
        raise SmokeFailure("http", "response exceeds its size budget")
    try:
        decoded = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SmokeFailure("http", "response is not bounded JSON") from error
    if not isinstance(decoded, dict):
        raise SmokeFailure("http", "response JSON must be an object")
    return status, decoded


def wait_for_health(base: str, process: subprocess.Popen, root: Path, deadline: float) -> None:
    while time.monotonic() < deadline:
        try:
            if process.poll() is not None:
                raise SmokeFailure("startup", "license process exited", process.returncode)
            status, health = request_json(base, "/healthz", deadline)
            if status == 200 and health.get("status") == "ok":
                status, document = request_json(base, "/.well-known/reader-license/legacy-public-key", deadline)
                generated = root / "data" / "legacy-rsa-public.spki"
                if status != 200 or not generated.is_file() or not isinstance(document.get("publicKey"), str):
                    raise SmokeFailure("startup", "server identity document did not match its data root")
                try:
                    published = base64.b64decode(document["publicKey"], validate=True)
                except ValueError as error:
                    raise SmokeFailure("startup", "server identity document was malformed") from error
                if not secrets.compare_digest(published, generated.read_bytes()):
                    raise SmokeFailure("startup", "health endpoint did not belong to this process")
                return
        except OSError:
            if process.poll() is not None:
                raise SmokeFailure("startup", "license process exited", process.returncode)
        time.sleep(0.2)
    raise SmokeFailure("startup", "license server did not become healthy in time", process.poll())


def wait_for_single_code(smtp: LoopbackSmtp, deadline: float) -> str:
    while time.monotonic() < deadline:
        with smtp.sink.lock:
            if smtp.sink.messages:
                raw = smtp.sink.messages[0]
                break
        time.sleep(0.05)
    else:
        raise SmokeFailure("smtp", "fake SMTP received no test message")
    message = BytesParser(policy=policy.default).parsebytes(raw)
    body_part = message.get_body(preferencelist=("plain",))
    if body_part is None:
        body = message.get_content() if not message.is_multipart() else ""
    else:
        body = body_part.get_content()
    matches = CODE.findall(body)
    if len(matches) != 1:
        raise SmokeFailure("smtp", "fake SMTP body did not contain exactly one eight digit code")
    return matches[0]


def start_server(java: str, jar: Path, root: Path, http_port: int, smtp_port: int) -> subprocess.Popen:
    environment = isolated_java_environment(os.environ, root, http_port, smtp_port, secrets.token_urlsafe(32))
    stderr = (root / "java.stderr").open("ab")
    command = [java, "-Xmx192m", "-XX:ActiveProcessorCount=2", "-jar", str(jar)]
    options = {"stdin": subprocess.DEVNULL, "stdout": subprocess.DEVNULL, "stderr": stderr, "env": environment}
    if os.name == "nt":
        options["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    else:
        options["start_new_session"] = True
    try:
        process = subprocess.Popen(command, **options)
    except Exception:
        stderr.close()
        raise
    process._reader_smoke_stderr = stderr
    ACTIVE_PROCESSES.add(process)
    return process


def stop_server(process: subprocess.Popen) -> None:
    try:
        if process.poll() is None:
            try:
                if os.name != "nt":
                    os.killpg(process.pid, signal.SIGTERM)
                else:
                    process.terminate()
            except (AttributeError, ProcessLookupError):
                process.terminate()
            try:
                process.wait(timeout=REQUEST_TIMEOUT_SECONDS)
            except subprocess.TimeoutExpired:
                try:
                    if os.name != "nt":
                        os.killpg(process.pid, signal.SIGKILL)
                    else:
                        process.kill()
                except (AttributeError, ProcessLookupError):
                    process.kill()
                process.wait(timeout=REQUEST_TIMEOUT_SECONDS)
    finally:
        ACTIVE_PROCESSES.discard(process)
        stderr = getattr(process, "_reader_smoke_stderr", None)
        if stderr is not None:
            stderr.close()


def require_trial_success(response: dict) -> None:
    if response.get("isSuccess") is not True or response.get("data") != "":
        raise SmokeFailure("trial-send", "trial code request did not return the expected success shape")


def require_trial_rejection(response: dict) -> None:
    if response.get("isSuccess") is not False or response.get("data") is not None:
        raise SmokeFailure("trial-replay", "trial claim was not rejected")


def make_report(jar_sha256: str, smtp_messages: int, claimed: bool, activated: bool,
                duplicate_rejected: bool, restart_rejected: bool) -> dict:
    report = {
        "schemaVersion": 1,
        "jarSha256": jar_sha256,
        "smtpMessageCount": smtp_messages,
        "eightDigitCodeReceived": True,
        "licenseSupplied": claimed,
        "licenseActivated": activated,
        "duplicateClaimRejected": duplicate_rejected,
        "restartClaimRejected": restart_rejected,
        "passed": claimed and activated and duplicate_rejected and restart_rejected,
    }
    validate_report(report)
    return report


def validate_report(report: dict) -> None:
    allowed = {"schemaVersion", "jarSha256", "smtpMessageCount", "eightDigitCodeReceived",
               "licenseSupplied", "licenseActivated", "duplicateClaimRejected",
               "restartClaimRejected", "passed"}
    if set(report) != allowed or report.get("schemaVersion") != 1:
        raise SmokeFailure("report", "invalid smoke report shape")
    if not isinstance(report.get("jarSha256"), str) or not SHA256.fullmatch(report["jarSha256"]):
        raise SmokeFailure("report", "invalid smoke report digest")
    if report.get("smtpMessageCount") != 1 or report.get("eightDigitCodeReceived") is not True:
        raise SmokeFailure("report", "invalid SMTP observation")
    for key in ("licenseSupplied", "licenseActivated", "duplicateClaimRejected", "restartClaimRejected", "passed"):
        if report.get(key) is not True:
            raise SmokeFailure("report", "smoke report did not complete")


def run_smoke(java: str, jar: Path, root: Path) -> dict:
    jar_sha256 = bounded_file_hash(jar)
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    started = time.monotonic()
    with LoopbackSmtp() as smtp:
        port = free_loopback_port()
        base = "http://127.0.0.1:" + str(port)
        first = start_server(java, jar, root, port, smtp.port)
        try:
            wait_for_health(base, first, root, min(started + STARTUP_TIMEOUT_SECONDS, started + TOTAL_TIMEOUT_SECONDS))
            status, sent = request_json(base, "/reader3/sendCodeToEmail", started + TOTAL_TIMEOUT_SECONDS, {"email": "trial@example.test"})
            if status != 200:
                raise SmokeFailure("trial-send", "trial code request did not return HTTP 200")
            require_trial_success(sent)
            code = wait_for_single_code(smtp, started + TOTAL_TIMEOUT_SECONDS)
            status, supplied = request_json(base, "/reader3/supplyLicense", started + TOTAL_TIMEOUT_SECONDS, {"email": "trial@example.test", "code": code})
            key = supplied.get("data", {}).get("key") if isinstance(supplied.get("data"), dict) else None
            if status != 200 or supplied.get("isSuccess") is not True or not isinstance(key, str) or not key or len(key) > MAX_HTTP_BODY_BYTES:
                raise SmokeFailure("trial-supply", "trial license was not supplied")
            status, activation = request_json(base, "/reader3/activateLicense", started + TOTAL_TIMEOUT_SECONDS, {"content": key})
            if status != 200 or activation.get("isSuccess") is not True:
                raise SmokeFailure("activation", "supplied license did not activate")
            status, replay = request_json(base, "/reader3/supplyLicense", started + TOTAL_TIMEOUT_SECONDS, {"email": "trial@example.test", "code": code})
            if status != 200:
                raise SmokeFailure("trial-replay", "duplicate claim did not return HTTP 200")
            require_trial_rejection(replay)
        finally:
            stop_server(first)
        if time.monotonic() > started + TOTAL_TIMEOUT_SECONDS:
            raise SmokeFailure("deadline", "first server exceeded total time budget")
        second_port = free_loopback_port()
        second_base = "http://127.0.0.1:" + str(second_port)
        second = start_server(java, jar, root, second_port, smtp.port)
        try:
            wait_for_health(second_base, second, root, started + TOTAL_TIMEOUT_SECONDS)
            status, restarted = request_json(second_base, "/reader3/sendCodeToEmail", started + TOTAL_TIMEOUT_SECONDS, {"email": "trial@example.test"})
            if status != 200:
                raise SmokeFailure("restart", "restart claim did not return HTTP 200")
            require_trial_rejection(restarted)
            with smtp.sink.lock:
                count = len(smtp.sink.messages)
            remaining_seconds(started + TOTAL_TIMEOUT_SECONDS)
            return make_report(jar_sha256, count, True, True, True, True)
        finally:
            stop_server(second)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--java", default="java")
    args = parser.parse_args(argv)
    validate_paths(args.jar, args.root, args.report)
    report = run_smoke(args.java, args.jar.resolve(), args.root)
    with args.report.open("x", encoding="utf-8") as stream:
        json.dump(report, stream, sort_keys=True)
        stream.write("\n")
    print(json.dumps(report, sort_keys=True))
    return 0


def signal_cleanup(_signum, _frame) -> None:
    for process in tuple(ACTIVE_PROCESSES):
        stop_server(process)
    raise KeyboardInterrupt


if __name__ == "__main__":
    try:
        signal.signal(signal.SIGTERM, signal_cleanup)
        raise SystemExit(main(sys.argv[1:]))
    except (OSError, SmokeFailure, subprocess.SubprocessError, KeyboardInterrupt) as error:
        stage = error.stage if isinstance(error, SmokeFailure) else "interrupted"
        exit_code = error.exit_code if isinstance(error, SmokeFailure) else None
        receipt = {"schemaVersion": 1, "passed": False, "stage": stage}
        if isinstance(exit_code, int):
            receipt["exitCode"] = exit_code
        print(json.dumps(receipt, sort_keys=True), file=sys.stderr)
        raise SystemExit(1)
