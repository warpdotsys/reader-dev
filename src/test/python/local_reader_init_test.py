"""No-network generated-data tests for the local Reader PID 1 supervisor."""
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import tempfile
import types
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("local_reader_init", ROOT / "scripts" / "local-reader" / "init.py")
INIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INIT)


class Child:
    next_pid = 100
    def __init__(self, result=None):
        self.pid = Child.next_pid; Child.next_pid += 1
        self.result = result; self.terminated = False; self.killed = False; self.signals = []
    def poll(self): return self.result
    def terminate(self): self.terminated = True; self.result = -15
    def kill(self): self.killed = True; self.result = -9
    def wait(self, timeout=None): return self.result
    def send_signal(self, value): self.signals.append(value); self.result = -value


class Popen:
    def __init__(self, children): self.children = list(children); self.calls = []
    def __call__(self, argv, env, **kwargs): self.calls.append((argv, env, kwargs)); return self.children.pop(0)


class LocalReaderInitTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); base = Path(self.temp.name)
        class TestPaths:
            entrypoint = base / "reader-entrypoint"; java = base / "java"; license_jar = base / "server.jar"
            license_data = base / "license-data"; token = base / "secrets" / "license-admin-token"; logs = base / "logs"
            reader_admin_key = base / "secrets" / "reader-admin-key"
        self.paths = TestPaths
        for file in (self.paths.entrypoint, self.paths.java, self.paths.license_jar): file.write_bytes(b"x")
        os.chmod(self.paths.entrypoint, 0o700); os.chmod(self.paths.java, 0o700)
        self.paths.license_data.mkdir(); self.paths.logs.mkdir(); self.paths.token.parent.mkdir(); os.chmod(self.paths.token.parent, 0o700)
        self.paths.token.write_text("a" * 24, encoding="utf-8"); os.chmod(self.paths.token, 0o600)
        self.paths.reader_admin_key.write_text("b" * 24, encoding="utf-8"); os.chmod(self.paths.reader_admin_key, 0o600)
        self.events = []
        self.real_lstat = INIT._lstat
        secret_parent = self.paths.token.parent
        def linux_lstat(path):
            value = self.real_lstat(path); path = Path(path); mode = value.st_mode
            if path == secret_parent: mode = stat.S_IFDIR | 0o700
            elif path in (self.paths.token, self.paths.reader_admin_key): mode = stat.S_IFREG | 0o600
            elif path in (self.paths.entrypoint, self.paths.java): mode = stat.S_IFREG | 0o700
            elif path == self.paths.license_jar: mode = stat.S_IFREG | 0o600
            elif path in (self.paths.license_data, self.paths.logs): mode = stat.S_IFDIR | 0o700
            return types.SimpleNamespace(st_mode=mode, st_nlink=value.st_nlink, st_size=value.st_size,
                st_ino=value.st_ino, st_dev=value.st_dev, st_uid=10001)
        self.linux_lstat = linux_lstat; INIT._lstat = linux_lstat
    def tearDown(self):
        INIT._lstat = self.real_lstat
        self.temp.cleanup()
    def fetch(self, url, timeout):
        if url.endswith("/healthz"): return 200, b'{"status":"ok"}'
        return 200, json.dumps({"publicKey": "A" * 344}).encode()
    def supervisor(self, children, fetch=None, proc_root=None, waitpid=os.waitpid):
        return INIT.Supervisor(self.paths, Popen(children), fetch or self.fetch, sleeper=lambda _seconds: None,
            proc_root=proc_root or Path(self.temp.name) / "proc", waitpid=waitpid, event=self.events.append,
            getuid=lambda: 10001)
    def test_reader_exit_code_is_preserved_and_token_never_reaches_reader(self):
        license_child, reader_child = Child(), Child(23); popen = Popen([license_child, reader_child])
        supervisor = INIT.Supervisor(self.paths, popen, self.fetch, sleeper=lambda _seconds: None, event=self.events.append,
            getuid=lambda: 10001)
        self.assertEqual(23, supervisor.run(install_signal_handlers=False))
        license_env, reader_env = popen.calls[0][1], popen.calls[1][1]
        self.assertEqual("a" * 24, license_env["LICENSE_ADMIN_TOKEN"])
        self.assertNotIn("LICENSE_ADMIN_TOKEN", reader_env)
        self.assertNotIn("READER_APP_SECUREKEY", license_env)
        self.assertEqual("b" * 24, reader_env["READER_APP_SECUREKEY"])
        self.assertEqual("true", reader_env["READER_APP_SECURE"])
        self.assertEqual("true", reader_env["READER_APP_LICENSECHECKENABLED"])
        self.assertTrue(license_child.terminated)
    def test_license_role_lifecycle_spawns_only_license(self):
        license_child = Child(0); popen = Popen([license_child])
        supervisor = INIT.Supervisor(self.paths, popen, self.fetch, sleeper=lambda _seconds: None,
            event=self.events.append, getuid=lambda: 10001, role="license")
        self.assertEqual(0, supervisor.run(install_signal_handlers=False))
        self.assertEqual(1, len(popen.calls))
        self.assertIn("LICENSE_ADMIN_TOKEN", popen.calls[0][1])
        self.assertNotIn("READER_APP_SECUREKEY", popen.calls[0][1])
    def test_reader_role_lifecycle_uses_external_license_and_file_bound_secret(self):
        original = INIT._lstat
        def file_bound_parent(path):
            value = original(path)
            if Path(path) == self.paths.reader_admin_key.parent:
                return types.SimpleNamespace(st_mode=stat.S_IFDIR | 0o755, st_nlink=value.st_nlink,
                    st_size=value.st_size, st_ino=value.st_ino, st_dev=value.st_dev, st_uid=0)
            return value
        INIT._lstat = file_bound_parent
        try:
            reader_child = Child(0); popen = Popen([reader_child])
            supervisor = INIT.Supervisor(self.paths, popen, self.fetch, sleeper=lambda _seconds: None,
                event=self.events.append, getuid=lambda: 10001, role="reader")
            self.assertEqual(0, supervisor.run(install_signal_handlers=False))
            self.assertEqual(1, len(popen.calls))
            self.assertEqual([str(self.paths.entrypoint)], popen.calls[0][0])
            self.assertNotIn("LICENSE_ADMIN_TOKEN", popen.calls[0][1])
            self.assertEqual("b" * 24, popen.calls[0][1]["READER_APP_SECUREKEY"])
        finally:
            INIT._lstat = original
    def test_license_exit_code_is_preserved_and_reader_is_stopped(self):
        license_child, reader_child = Child(17), Child(); popen = Popen([license_child, reader_child])
        def sleeper(_seconds): license_child.result = 17
        supervisor = INIT.Supervisor(self.paths, popen, self.fetch, sleeper=sleeper, event=self.events.append,
            getuid=lambda: 10001)
        # Keep it alive until after the health/key exchange, then return 17.
        license_child.result = None
        self.assertEqual(17, supervisor.run(install_signal_handlers=False))
        self.assertTrue(reader_child.terminated)
    def test_license_environment_is_limited_but_reader_defaults_are_preserved(self):
        previous = os.environ.get("JAVA_TOOL_OPTIONS"); os.environ["JAVA_TOOL_OPTIONS"] = "-Xmx9g"
        try:
            license_child, reader_child = Child(), Child(0); popen = Popen([license_child, reader_child])
            supervisor = INIT.Supervisor(self.paths, popen, self.fetch, sleeper=lambda _seconds: None, event=self.events.append,
                getuid=lambda: 10001)
            self.assertEqual(0, supervisor.run(install_signal_handlers=False))
            args, license_env, _kwargs = popen.calls[0]
            self.assertEqual([str(self.paths.java), "-Xms16m", "-Xmx96m", "-XX:ActiveProcessorCount=2", "-jar", str(self.paths.license_jar)], args)
            self.assertNotIn("JAVA_TOOL_OPTIONS", license_env)
            self.assertEqual("-Xmx9g", popen.calls[1][1]["JAVA_TOOL_OPTIONS"])
        finally:
            if previous is None: os.environ.pop("JAVA_TOOL_OPTIONS", None)
            else: os.environ["JAVA_TOOL_OPTIONS"] = previous
    def test_health_failure_does_not_start_reader_and_returns_nonzero(self):
        child = Child(); popen = Popen([child])
        ticks = iter([0.0, 31.0])
        def clock():
            try: return next(ticks)
            except StopIteration: return 31.0
        supervisor = INIT.Supervisor(self.paths, popen, lambda *_: (_ for _ in ()).throw(OSError()),
            clock=clock, sleeper=lambda _seconds: None, event=self.events.append, getuid=lambda: 10001)
        self.assertEqual(70, supervisor.run(install_signal_handlers=False))
        self.assertEqual(1, len(popen.calls)); self.assertTrue(child.terminated)
    def test_foreign_loopback_success_is_rejected_if_our_license_child_exited(self):
        license_child = Child(); popen = Popen([license_child])
        calls = {"count": 0}
        def fetch(url, timeout):
            calls["count"] += 1
            response = self.fetch(url, timeout)
            if calls["count"] == 2:
                # The key reply could be from a stale listener after our child
                # lost bind; the post-read poll must refuse to start Reader.
                license_child.result = 31
            return response
        supervisor = INIT.Supervisor(self.paths, popen, fetch, sleeper=lambda _seconds: None,
            event=self.events.append, getuid=lambda: 10001)
        self.assertEqual(31, supervisor.run(install_signal_handlers=False))
        self.assertEqual(1, len(popen.calls))
    def test_startup_signal_never_continues_to_reader(self):
        license_child = Child(); popen = Popen([license_child])
        supervisor = INIT.Supervisor(self.paths, popen, self.fetch, sleeper=lambda _seconds: None,
            event=self.events.append, getuid=lambda: 10001)
        def fetch(url, timeout):
            supervisor.forward_signal(15)
            return self.fetch(url, timeout)
        supervisor.fetch = fetch
        self.assertEqual(-15, supervisor.run(install_signal_handlers=False))
        self.assertEqual(1, len(popen.calls))
    def test_orphan_reaper_never_waits_direct_popen_child(self):
        proc = Path(self.temp.name) / "proc"; proc.mkdir()
        for pid in (101, 102):
            folder = proc / str(pid); folder.mkdir()
            (folder / "status").write_text("State:\tZ (zombie)\nPPid:\t1\n", encoding="utf-8")
        waited = []
        supervisor = self.supervisor([], proc_root=proc, waitpid=lambda pid, flag: (waited.append(pid) or (pid, 0)))
        supervisor._direct_pids.add(101)
        self.assertEqual(1, supervisor.reap_orphans()); self.assertEqual([102], waited)
    def test_token_permission_failure_and_signal_forwarding_are_nonsecret(self):
        original = INIT._lstat
        def insecure(path):
            value = original(path)
            if Path(path) == self.paths.token:
                return types.SimpleNamespace(st_mode=stat.S_IFREG | 0o644, st_nlink=value.st_nlink,
                    st_size=value.st_size, st_ino=value.st_ino, st_dev=value.st_dev, st_uid=10001)
            return value
        INIT._lstat = insecure
        with self.assertRaises(INIT.StartupError): INIT.read_admin_token(self.paths, lambda: 10001)
        INIT._lstat = original
        self.paths.reader_admin_key.unlink()
        with self.assertRaises(INIT.StartupError): INIT.read_secret(self.paths.reader_admin_key, lambda: 10001)
        supervisor = self.supervisor([]); supervisor.license, supervisor.reader = Child(), Child()
        supervisor.forward_signal(15)
        self.assertEqual([15], supervisor.license.signals); self.assertEqual([15], supervisor.reader.signals)
        self.assertNotIn("a" * 24, " ".join(self.events))
    def test_main_rejects_arguments_without_starting(self):
        self.assertEqual(64, INIT.main(["--not-supported"]))
    def test_java_alternatives_link_resolves_to_regular_terminal(self):
        link = Path(self.temp.name) / "java-alternative"
        try:
            link.symlink_to(self.paths.java)
        except OSError as exc:
            self.skipTest("local platform does not permit generated symlink: " + str(exc))
        self.assertEqual(self.paths.java.resolve(), INIT.resolve_java(link))
    def test_secret_owner_must_be_uid_10001(self):
        original = INIT._lstat
        def wrong_owner(path):
            value = original(path)
            if Path(path) == self.paths.reader_admin_key.parent:
                return types.SimpleNamespace(st_mode=value.st_mode, st_nlink=value.st_nlink, st_size=value.st_size,
                    st_ino=value.st_ino, st_dev=value.st_dev, st_uid=10000)
            return value
        INIT._lstat = wrong_owner
        try:
            with self.assertRaises(INIT.StartupError):
                INIT.read_secret(self.paths.reader_admin_key, lambda: 10001)
        finally:
            INIT._lstat = original
    def test_log_drain_error_is_recorded_without_propagating_thread_trace(self):
        class BrokenSink:
            def write(self, _chunk): raise OSError("generated failure")
            def close(self): pass
        supervisor = self.supervisor([])
        supervisor._drain(io.BytesIO(b"generated output"), BrokenSink())
        self.assertTrue(supervisor._log_error.is_set())
    def test_generated_child_log_rotates_at_bound_without_stdout_forwarding(self):
        logs = INIT.RotatingLog(self.paths.logs, "reader", "stdout", limit=8)
        logs.write(b"abcdefghijklmno"); logs.close()
        current = self.paths.logs / "reader.stdout.0.log"
        previous = self.paths.logs / "reader.stdout.1.log"
        self.assertLessEqual(current.stat().st_size, 8); self.assertLessEqual(previous.stat().st_size, 8)
        self.assertEqual(b"abcdefgh", previous.read_bytes())
        self.assertEqual(b"ijklmno", current.read_bytes())


if __name__ == "__main__": unittest.main()
