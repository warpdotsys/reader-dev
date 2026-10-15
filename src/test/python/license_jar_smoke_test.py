import importlib.util
import socket
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "smoke-license-jar.py"
SPEC = importlib.util.spec_from_file_location("license_jar_smoke", SCRIPT)
SMOKE = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(SMOKE)


class LicenseJarSmokeTest(unittest.TestCase):
    def test_bounded_regular_jar_hash(self):
        with tempfile.TemporaryDirectory() as temporary:
            jar = Path(temporary) / "fixture.jar"
            jar.write_bytes(b"synthetic-fat-jar")
            digest = SMOKE.bounded_file_hash(jar)
            self.assertRegex(digest, r"^[0-9a-f]{64}$")
            jar.unlink()
            with self.assertRaises(SMOKE.SmokeFailure):
                SMOKE.bounded_file_hash(jar)

    def test_runtime_environment_discards_inherited_license_variables(self):
        environment = SMOKE.isolated_java_environment(
            {"PATH": "/bin", "SystemRoot": "C:\\Windows", "WINDIR": "C:\\Windows", "LICENSE_ADMIN_TOKEN": "production", "LICENSE_SMTP_PASSWORD": "production", "OTHER": "ignored"},
            Path("/tmp/license-smoke"), 18181, 12525, "synthetic-admin-token",
        )
        self.assertEqual(environment["LICENSE_ADMIN_TOKEN"], "synthetic-admin-token")
        self.assertEqual(environment["LICENSE_SMTP_HOST"], "127.0.0.1")
        self.assertEqual(environment["SystemRoot"], "C:\\Windows")
        self.assertEqual(environment["WINDIR"], "C:\\Windows")
        self.assertNotIn("LICENSE_SMTP_PASSWORD", environment)
        self.assertNotIn("OTHER", environment)
        self.assertTrue(all(not name.startswith("LICENSE_") or name in {
            "LICENSE_BIND", "LICENSE_PORT", "LICENSE_DATA_DIR", "LICENSE_ADMIN_TOKEN", "LICENSE_ISSUER",
            "LICENSE_LEGACY_RSA_ENABLED", "LICENSE_SMTP_HOST", "LICENSE_SMTP_PORT", "LICENSE_SMTP_MODE",
            "LICENSE_SMTP_ALLOW_PLAINTEXT_LOOPBACK", "LICENSE_SMTP_FROM",
        } for name in environment))

    def test_safe_report_shape_contains_no_secrets(self):
        report = SMOKE.make_report("a" * 64, 1, True, True, True, True)
        self.assertTrue(report["passed"])
        self.assertEqual(report["smtpMessageCount"], 1)
        self.assertNotIn("key", report)
        self.assertNotIn("code", report)
        self.assertNotIn("mail", report)
        modified = dict(report)
        modified["token"] = "must-not-be-accepted"
        with self.assertRaises(SMOKE.SmokeFailure):
            SMOKE.validate_report(modified)

    def test_report_requires_every_claimed_outcome(self):
        report = SMOKE.make_report("b" * 64, 1, True, True, True, True)
        report["restartClaimRejected"] = False
        with self.assertRaises(SMOKE.SmokeFailure):
            SMOKE.validate_report(report)

    def test_fake_smtp_is_single_connection_and_extracts_code_from_body_only(self):
        with SMOKE.LoopbackSmtp() as smtp:
            with socket.create_connection(("127.0.0.1", smtp.port), timeout=1) as connection:
                stream = connection.makefile("rwb")
                self.assertTrue(stream.readline().startswith(b"220"))
                for command in (b"EHLO test\r\n", b"MAIL FROM:<sender@example.test>\r\n", b"RCPT TO:<trial@example.test>\r\n"):
                    stream.write(command)
                    stream.flush()
                    self.assertTrue(stream.readline().startswith(b"250"))
                    if command.startswith(b"EHLO"):
                        self.assertTrue(stream.readline().startswith(b"250"))
                stream.write(b"DATA\r\n")
                stream.flush()
                self.assertTrue(stream.readline().startswith(b"354"))
                stream.write(b"Date: 12345678\r\nSubject: no-code-here\r\n\r\nYour code is 87654321\r\n.\r\nQUIT\r\n")
                stream.flush()
                self.assertTrue(stream.readline().startswith(b"250"))
                self.assertTrue(stream.readline().startswith(b"221"))
            self.assertEqual(SMOKE.wait_for_single_code(smtp, time.monotonic() + 1), "87654321")

    def test_deadline_and_exited_child_are_rejected_before_an_http_request(self):
        with self.assertRaises(SMOKE.SmokeFailure):
            SMOKE.request_json("http://127.0.0.1:1", "/healthz", time.monotonic() - 0.1)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            child = subprocess.Popen([sys.executable, "-c", "raise SystemExit(7)"])
            self.assertEqual(child.wait(timeout=5), 7)
            with self.assertRaises(SMOKE.SmokeFailure) as rejected:
                SMOKE.wait_for_health("http://127.0.0.1:1", child, root, time.monotonic() + 1)
            self.assertEqual(rejected.exception.exit_code, 7)

    def test_root_and_report_paths_must_be_new_non_symlink_children(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            jar = base / "fixture.jar"
            jar.write_bytes(b"fixture")
            output = base / "out"
            output.mkdir()
            existing_root = base / "already-there"
            existing_root.mkdir()
            with self.assertRaises(SMOKE.SmokeFailure):
                SMOKE.validate_paths(jar, existing_root, output / "report.json")
            existing_report = output / "already.json"
            existing_report.touch()
            with self.assertRaises(SMOKE.SmokeFailure):
                SMOKE.validate_paths(jar, base / "runtime", existing_report)


if __name__ == "__main__":
    unittest.main()
