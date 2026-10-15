#!/usr/bin/env python3
"""PID 1 supervisor for the exact local Reader image; it never issues licenses."""
import json
import os
from pathlib import Path
import re
import signal
import stat
import subprocess
import sys
import threading
import time
import urllib.request

LICENSE_BASE = "http://127.0.0.1:18964"
HEALTH_TIMEOUT_SECONDS = 30.0
CHILD_STOP_SECONDS = 10.0
POLL_SECONDS = 0.1
WAIT_NOHANG = getattr(os, "WNOHANG", 1)
LOG_BYTES = 2 * 1024 * 1024
PUBLIC_KEY_RE = re.compile(r"[A-Za-z0-9+/]{128,4096}={0,2}\Z")


class StartupError(Exception):
    pass


class LicenseChildExited(StartupError):
    pass


class Paths:
    """Fixed container paths, with injection only for generated-data tests."""
    entrypoint = Path("/usr/local/bin/reader-entrypoint")
    java = Path("/opt/java/openjdk/bin/java")
    license_jar = Path("/local-license/server.jar")
    license_data = Path("/license-data")
    token = Path("/local-secrets/license-admin-token")
    reader_admin_key = Path("/local-secrets/reader-admin-key")
    logs = Path("/logs")


def emit(event):
    """Only non-secret, bounded lifecycle event names reach stdout."""
    print(json.dumps({"event": event}, separators=(",", ":")), flush=True)


def _lstat(path):
    try:
        return os.lstat(path)
    except OSError as exc:
        raise StartupError("required-path-unavailable") from exc


def _regular(path, executable=False):
    value = _lstat(path)
    if not stat.S_ISREG(value.st_mode) or stat.S_ISLNK(value.st_mode):
        raise StartupError("unsafe-required-file")
    if executable and not (value.st_mode & stat.S_IXUSR):
        raise StartupError("entrypoint-not-executable")


def _directory(path, mode=None, owner=None):
    value = _lstat(path)
    if not stat.S_ISDIR(value.st_mode) or stat.S_ISLNK(value.st_mode):
        raise StartupError("unsafe-required-directory")
    if mode is not None and stat.S_IMODE(value.st_mode) != mode:
        raise StartupError("unsafe-token-parent-mode")
    if owner is not None and value.st_uid != owner:
        raise StartupError("unsafe-secret-owner")


def read_secret(path, getuid=getattr(os, "getuid", lambda: 10001)):
    expected_uid = getuid()
    if expected_uid != 10001:
        raise StartupError("unexpected-supervisor-uid")
    # The production roles receive one file bind under a root-owned 0755
    # directory.  The older combined test harness uses a private 0700
    # directory.  In both forms the secret file itself remains 0600/10001.
    parent = _lstat(path.parent)
    private_parent = (stat.S_ISDIR(parent.st_mode) and not stat.S_ISLNK(parent.st_mode)
                      and stat.S_IMODE(parent.st_mode) == 0o700 and parent.st_uid == expected_uid)
    isolated_file_parent = (stat.S_ISDIR(parent.st_mode) and not stat.S_ISLNK(parent.st_mode)
                            and stat.S_IMODE(parent.st_mode) == 0o755 and parent.st_uid == 0)
    if not (private_parent or isolated_file_parent):
        raise StartupError("unsafe-token-parent-mode")
    value = _lstat(path)
    if (not stat.S_ISREG(value.st_mode) or stat.S_ISLNK(value.st_mode) or
            value.st_nlink != 1 or value.st_uid != expected_uid or stat.S_IMODE(value.st_mode) != 0o600 or
            value.st_size > 4096):
        raise StartupError("unsafe-token-file")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError as exc:
        raise StartupError("token-unavailable") from exc
    try:
        current = os.fstat(descriptor)
        if current.st_ino != value.st_ino or current.st_dev != value.st_dev or current.st_nlink != 1:
            raise StartupError("unsafe-token-file")
        payload = os.read(descriptor, 4097)
    finally:
        os.close(descriptor)
    if len(payload) > 4096:
        raise StartupError("unsafe-token-file")
    try:
        token = payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise StartupError("invalid-token") from exc
    if token.endswith("\n"):
        token = token[:-1]
    if len(token) < 24 or "\n" in token or "\r" in token:
        raise StartupError("invalid-token")
    return token


def read_admin_token(paths, getuid=getattr(os, "getuid", lambda: 10001)):
    return read_secret(paths.token, getuid)


def resolve_java(path):
    if not Path(path).is_absolute():
        raise StartupError("java-path-not-absolute")
    try:
        terminal = Path(path).resolve(strict=True)
    except OSError as exc:
        raise StartupError("java-path-unavailable") from exc
    if not terminal.is_absolute():
        raise StartupError("java-path-not-absolute")
    value = _lstat(terminal)
    if not stat.S_ISREG(value.st_mode) or stat.S_ISLNK(value.st_mode) or not (value.st_mode & stat.S_IXUSR):
        raise StartupError("unsafe-java-terminal")
    return terminal


def validate_paths(paths, role="combined"):
    if role in ("combined", "reader"):
        _regular(paths.entrypoint, executable=True)
    if role in ("combined", "license"):
        _regular(paths.license_jar)
        _directory(paths.license_data)
    _directory(paths.logs)
    return resolve_java(paths.java)


class RotatingLog:
    """Write child output in bounded chunks, retaining only current+previous."""
    def __init__(self, directory, name, stream, limit=LOG_BYTES):
        self.directory = Path(directory)
        self.current = self.directory / (name + "." + stream + ".0.log")
        self.previous = self.directory / (name + "." + stream + ".1.log")
        self.limit = limit
        self.handle = None

    def _open(self):
        if self.handle is not None:
            return
        try:
            if self.current.exists():
                value = os.lstat(self.current)
                if not stat.S_ISREG(value.st_mode) or stat.S_ISLNK(value.st_mode) or value.st_nlink != 1:
                    raise StartupError("unsafe-log-file")
            flags = os.O_WRONLY | os.O_CREAT | os.O_APPEND | getattr(os, "O_NOFOLLOW", 0)
            descriptor = os.open(self.current, flags, 0o600)
            self.handle = os.fdopen(descriptor, "ab", buffering=0)
        except OSError as exc:
            raise StartupError("log-unavailable") from exc

    def _rotate(self):
        if self.handle is not None:
            self.handle.close(); self.handle = None
        try:
            if self.current.exists():
                os.replace(self.current, self.previous)
        except OSError as exc:
            raise StartupError("log-rotation-failed") from exc
        self._open()

    def write(self, payload):
        view = memoryview(payload)
        while view:
            self._open()
            used = os.fstat(self.handle.fileno()).st_size
            if used >= self.limit:
                self._rotate()
                continue
            count = min(len(view), self.limit - used)
            self.handle.write(view[:count])
            view = view[count:]

    def close(self):
        if self.handle is not None:
            self.handle.close(); self.handle = None


def default_fetch(url, timeout):
    request = urllib.request.Request(url, method="GET")
    with urllib.request.urlopen(request, timeout=timeout) as response:
        payload = response.read(8193)
        if len(payload) > 8192:
            raise StartupError("license-response-too-large")
        return response.status, payload


def wait_for_license(fetch, clock=time.monotonic, sleeper=time.sleep, timeout=HEALTH_TIMEOUT_SECONDS,
        should_abort=lambda: False, child_status=lambda: None):
    deadline = clock() + timeout
    health_ok = False
    while clock() < deadline:
        if should_abort():
            raise StartupError("child-log-failed")
        if child_status() is not None:
            raise LicenseChildExited("license-child-exited")
        try:
            health_status, health_body = fetch(LICENSE_BASE + "/healthz", min(2.0, max(0.1, deadline - clock())))
            if health_status == 200 and json.loads(health_body.decode("utf-8")).get("status") == "ok":
                health_ok = True
                key_status, key_body = fetch(LICENSE_BASE + "/.well-known/reader-license/legacy-public-key", 2.0)
                if key_status == 200:
                    public_key = json.loads(key_body.decode("utf-8")).get("publicKey", "")
                    if isinstance(public_key, str) and PUBLIC_KEY_RE.fullmatch(public_key):
                        # Do not trust a successful loopback response unless
                        # the child started by this supervisor is still alive.
                        # This rejects a stale/foreign listener after bind loss.
                        if child_status() is not None:
                            raise LicenseChildExited("license-child-exited")
                        return public_key
        except LicenseChildExited:
            raise
        except (OSError, ValueError, UnicodeDecodeError, json.JSONDecodeError, StartupError):
            pass
        sleeper(POLL_SECONDS)
    raise StartupError("license-health-or-public-key-unavailable" if health_ok else "license-health-unavailable")


class Supervisor:
    def __init__(self, paths=Paths, popen=subprocess.Popen, fetch=default_fetch,
            clock=time.monotonic, sleeper=time.sleep, waitpid=os.waitpid, proc_root=Path("/proc"), event=emit,
            getuid=getattr(os, "getuid", lambda: 10001), role="combined"):
        if role not in ("combined", "license", "reader"):
            raise ValueError("invalid supervisor role")
        self.paths = paths
        self.popen = popen
        self.fetch = fetch
        self.clock = clock
        self.sleeper = sleeper
        self.waitpid = waitpid
        self.proc_root = Path(proc_root)
        self.event = event
        self.getuid = getuid
        self.role = role
        self._lock = threading.Lock()
        self._direct_pids = set()
        self.license = None
        self.reader = None
        self._drainers = []
        self._logs = None
        self._log_error = threading.Event()
        self._stop_requested = threading.Event()

    def _drain(self, stream, sink):
        failed = False
        try:
            while True:
                chunk = stream.read(4096)
                if not chunk:
                    return
                if not failed:
                    try:
                        sink.write(chunk)
                    except Exception:
                        # Keep consuming the pipe after a log error; otherwise
                        # a noisy Java child could block forever on its stdout.
                        failed = True; self._log_error.set()
        except Exception:
            self._log_error.set()
        finally:
            try:
                sink.close()
            except Exception:
                self._log_error.set()

    def _attach_logs(self, child, name):
        for stream_name in ("stdout", "stderr"):
            stream = getattr(child, stream_name, None)
            if stream is None:
                continue
            sink = RotatingLog(self.paths.logs, name, stream_name)
            thread = threading.Thread(target=self._drain, args=(stream, sink), daemon=True)
            thread.start(); self._drainers.append(thread)

    def _spawn(self, argv, env, name):
        # Registration occurs while holding the reaper lock, so an orphan
        # sweep can never steal a directly managed Popen child's exit status.
        with self._lock:
            child = self.popen(argv, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self._direct_pids.add(child.pid)
        try:
            self._attach_logs(child, name)
            return child
        except Exception:
            child.terminate()
            self._forget(child)
            raise

    def _forget(self, child):
        with self._lock:
            self._direct_pids.discard(child.pid)

    def _zombie_ppid_one(self):
        try:
            entries = list(self.proc_root.iterdir())
        except OSError:
            return []
        values = []
        for item in entries:
            if not item.name.isdigit():
                continue
            try:
                status_text = (item / "status").read_text(encoding="utf-8", errors="replace", newline=None)
            except (OSError, TypeError):
                # Python 3.10 Path.read_text has no newline argument.
                try:
                    with (item / "status").open("r", encoding="utf-8", errors="replace") as handle:
                        status_text = handle.read(8192)
                except OSError:
                    continue
            state = parent = None
            for line in status_text.splitlines():
                if line.startswith("State:"):
                    state = line.split()[1:2]
                elif line.startswith("PPid:"):
                    parent = line.split()[1:2]
            if state == ["Z"] and parent == ["1"]:
                values.append(int(item.name))
        return values

    def reap_orphans(self):
        reaped = 0
        with self._lock:
            for pid in self._zombie_ppid_one():
                if pid in self._direct_pids:
                    continue
                try:
                    found, _status = self.waitpid(pid, WAIT_NOHANG)
                except (ChildProcessError, OSError):
                    continue
                if found == pid:
                    reaped += 1
        return reaped

    def _license_env(self, token):
        env = dict(os.environ)
        env.pop("JAVA_TOOL_OPTIONS", None)
        env.pop("READER_APP_SECUREKEY", None)
        env.pop("LICENSE_ADMIN_TOKEN", None)
        env.update({"LICENSE_BIND": "127.0.0.1", "LICENSE_PORT": "18964",
            "LICENSE_DATA_DIR": "/license-data", "LICENSE_LEGACY_RSA_ENABLED": "true",
            "LICENSE_ADMIN_TOKEN": token})
        return env

    def _reader_env(self, public_key, admin_key):
        env = dict(os.environ)
        env.pop("LICENSE_ADMIN_TOKEN", None)
        env.update({"READER_APP_LICENSEPUBLICKEY": public_key,
            "READER_APP_LICENSESERVERURL": LICENSE_BASE,
            "READER_APP_LICENSECHECKENABLED": "true",
            "READER_APP_SECURE": "true", "READER_APP_SECUREKEY": admin_key})
        return env

    def _stop(self, child):
        if child is None or child.poll() is not None:
            return
        child.terminate()
        deadline = self.clock() + CHILD_STOP_SECONDS
        while child.poll() is None and self.clock() < deadline:
            self.sleeper(POLL_SECONDS)
        if child.poll() is None:
            child.kill()
        try:
            child.wait(timeout=CHILD_STOP_SECONDS)
        except (subprocess.TimeoutExpired, TypeError):
            pass
        self._forget(child)

    def _close_drainers(self):
        for thread in self._drainers:
            thread.join(timeout=CHILD_STOP_SECONDS)

    def forward_signal(self, signum, _frame=None):
        self._stop_requested.set()
        self.event("signal-forwarded")
        for child in (self.reader, self.license):
            if child is not None and child.poll() is None:
                child.send_signal(signum)

    def run(self, install_signal_handlers=True):
        try:
            java = validate_paths(self.paths, self.role)
            if install_signal_handlers:
                signal.signal(signal.SIGTERM, self.forward_signal)
                signal.signal(signal.SIGINT, self.forward_signal)
            if self.role in ("combined", "license"):
                token = read_admin_token(self.paths, self.getuid)
                self.license = self._spawn([str(java), "-Xms16m", "-Xmx96m",
                    "-XX:ActiveProcessorCount=2", "-jar", str(self.paths.license_jar)], self._license_env(token), "license")
                self.event("license-started")
                if self.role == "license":
                    while self.license.poll() is None and not self._stop_requested.is_set():
                        self.reap_orphans(); self.sleeper(POLL_SECONDS)
                    return self.license.poll() if self.license.poll() is not None else 0
            if self.role in ("combined", "reader"):
                admin_key = read_secret(self.paths.reader_admin_key, self.getuid)
                public_key = wait_for_license(self.fetch, self.clock, self.sleeper,
                    should_abort=lambda: self._log_error.is_set() or self._stop_requested.is_set(),
                    child_status=self.license.poll if self.license is not None else (lambda: None))
                if self._stop_requested.is_set() or (self.license is not None and self.license.poll() is not None):
                    raise StartupError("stop-requested-or-license-exited")
                self.reader = self._spawn([str(self.paths.entrypoint)], self._reader_env(public_key, admin_key), "reader")
                self.event("reader-started")
                if self.role == "reader":
                    while self.reader.poll() is None and not self._stop_requested.is_set():
                        self.reap_orphans()
                        if self._log_error.is_set():
                            self.event("child-log-failed")
                            return 74
                        self.sleeper(POLL_SECONDS)
                    if self.reader.poll() is not None:
                        self._forget(self.reader)
                        self.event("reader-exited")
                        return self.reader.poll()
                    return 0
            while True:
                self.reap_orphans()
                license_status = self.license.poll()
                reader_status = self.reader.poll()
                if license_status is not None:
                    self._forget(self.license)
                    self.event("license-exited")
                    self._stop(self.reader)
                    return license_status
                if reader_status is not None:
                    self._forget(self.reader)
                    self.event("reader-exited")
                    self._stop(self.license)
                    return reader_status
                if self._log_error.is_set():
                    self.event("child-log-failed")
                    self._stop(self.reader)
                    self._stop(self.license)
                    return 74
                self.sleeper(POLL_SECONDS)
        except StartupError:
            if self._log_error.is_set():
                self.event("child-log-failed")
            self.event("startup-failed")
            if self.license is not None and self.license.poll() is not None:
                return self.license.poll()
            return 70
        except Exception:
            self.event("supervisor-failed")
            return 71
        finally:
            # A setup error, exception, or signal never leaves the sibling
            # process running.  Normal direct-child exit codes are returned
            # above and are never replaced with zero.
            self._stop(self.reader)
            self._stop(self.license)
            self._close_drainers()


def main(argv=None):
    if argv is None:
        argv = sys.argv[1:]
    if len(argv) != 2 or argv[0] != "--role" or argv[1] not in ("license", "reader"):
        emit("invalid-arguments")
        return 64
    return Supervisor(role=argv[1]).run()


if __name__ == "__main__":
    sys.exit(main())
