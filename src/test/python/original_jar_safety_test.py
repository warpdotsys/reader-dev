"""Check comparison-script guards without starting either JAR."""

import importlib.util
import os
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[3]
SCRIPTS = ROOT / "scripts"
MODULE = SCRIPTS / "original_jar_safety.py"


class OriginalJarSafetyTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location("original_jar_safety", MODULE)
        cls.safety = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.safety)

    def test_requires_explicit_isolation_acknowledgment(self):
        key = self.safety.ISOLATION_ACK
        for value in (None, "", "1", "true"):
            with self.subTest(value=value), patch.dict(os.environ, {}, clear=False):
                os.environ.pop(key, None)
                if value is not None:
                    os.environ[key] = value
                with self.assertRaisesRegex(SystemExit, "Refusing to start"):
                    self.safety.require_original_jar_isolation()
        with patch.dict(os.environ, {key: "confirmed"}):
            self.safety.require_original_jar_isolation()

    def test_every_legacy_comparison_has_a_guard(self):
        compare_scripts = sorted(SCRIPTS.glob("compare-*.py"))
        self.assertEqual(29, len(compare_scripts))
        for path in compare_scripts:
            body = path.read_text(encoding="utf-8")
            if path.name == "compare-generated-epub-fragments-in-netns.py":
                guard = body.index("BASE.isolate_then_drop()  #")
                self.assertLess(guard, body.index("if BASE.digest(args.original)"))
                self.assertLess(guard, body.index("fixtures = {label: fixture_bytes"))
                self.assertLess(guard, body.index("original = run(args.java"))
                continue
            if path.name == "compare-authorized-local-reading-in-netns.py":
                guard = body.index("BASE.isolate_then_drop()  #")
                self.assertLess(guard, body.index("if BASE.digest(args.original)"))
                self.assertLess(guard, body.index("shelf_bytes = args.shelf.read_bytes()"))
                self.assertLess(guard, body.index("original = run(args.java"))
                continue
            if path.name == "compare-authorized-storage-in-netns.py":
                # This comparison checks the kernel namespace itself, not an
                # environment acknowledgment. Its runtime negative tests also
                # verify that a normal host cannot read inputs or launch a JAR.
                guard = body.index("isolate_then_drop()  #")
                self.assertLess(guard, body.index("if digest(args.original)"))
                self.assertLess(guard, body.index("business = {name:"))
                self.assertLess(guard, body.index("original, original_facts = run_reader"))
                self.assertIn('os.stat("/proc/1/ns/net").st_ino', body)
                self.assertIn('interfaces != ["lo"]', body)
                continue
            self.assertIn("require_original_jar_isolation()", body, path.name)
            if path.name == "compare-webview-cookie.py":
                self.assertIn('mode.add_argument("--original-network-isolated"', body)
                self.assertLess(body.index("require_original_jar_isolation()"),
                                body.index("fixture_port = free_port()"))
            elif path.name not in ("compare-live-source-reading.py",
                                   "compare-pdf-multipage.py",
                                   "compare-pdf-reading.py"):
                self.assertIn("require_original_jar_isolation()\n    main()", body,
                              path.name)
            else:
                self.assertLess(body.index("require_original_jar_isolation()"),
                                body.index("for path in "))
        for path in SCRIPTS.glob("compare-*.ps1"):
            body = path.read_text(encoding="utf-8")
            if "$OriginalJar" in body:
                self.assertIn('READER_ORIGINAL_JAR_NETWORK_ISOLATED -ne "confirmed"',
                              body, path.name)
                self.assertLess(body.index("READER_ORIGINAL_JAR_NETWORK_ISOLATED"),
                                body.index("Start-Process"))

    def test_unacknowledged_script_exits_before_creating_a_report(self):
        environment = os.environ.copy()
        environment.pop(self.safety.ISOLATION_ACK, None)
        result = subprocess.run(
            [sys.executable, "-B", str(SCRIPTS / "compare-book-source-crud.py")],
            cwd=ROOT, env=environment, capture_output=True, text=True,
            timeout=10, check=False,
        )
        self.assertNotEqual(0, result.returncode)
        self.assertIn("Refusing to start the original JAR", result.stderr)


if __name__ == "__main__":
    unittest.main()
