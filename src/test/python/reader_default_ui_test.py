import copy
import importlib.util
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import tempfile
import threading
import unittest
from zipfile import ZipFile


SCRIPT = Path(__file__).resolve().parents[3] / "scripts/verify-reader-default-ui.py"
SPEC = importlib.util.spec_from_file_location("reader_default_ui_gate", SCRIPT)
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)
HTML = ('<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8">'
        '<title>夜读 · Reader</title><script type="module" src="/static/index-generated.js"></script>'
        '<link rel="stylesheet" href="/static/index-generated.css"></head><body><div id="app"></div></body></html>').encode()
JS = "console.log('生成界面');".encode()
CSS = b"body{color:#123}"


class ReaderDefaultUiTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="reader-generated-default-ui-")
        self.addCleanup(self.temp.cleanup)
        self.jar = Path(self.temp.name) / "generated.jar"
        with ZipFile(self.jar, "w") as jar:
            jar.writestr(gate.ROOT + "index.html", HTML)
            jar.writestr(gate.ROOT + "static/index-generated.js", JS)
            jar.writestr(gate.ROOT + "static/index-generated.css", CSS)
            jar.writestr("BOOT-INF/classes/web/index.html", "<title>原版回退</title>")
        self.responses = {path: (200, "text/html; charset=UTF-8", HTML) for path in gate.ROUTES}
        self.responses.update({"/static/index-generated.js": (200, "application/javascript", JS),
                               "/static/index-generated.css": (200, "text/css", CSS)})
        self.redirect_destination_hits = 0
        owner = self
        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/redirect-destination":
                    owner.redirect_destination_hits += 1
                status, mime, body = owner.responses.get(self.path, (404, "text/plain", b"missing"))
                self.send_response(status)
                self.send_header("Content-Type", mime)
                self.send_header("Content-Length", str(len(body)))
                if status == 302:
                    self.send_header("Location", "/redirect-destination")
                self.end_headers()
                self.wfile.write(body)
            def log_message(self, *args):
                pass
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.close_server)
        self.base = "http://127.0.0.1:" + str(self.server.server_address[1])

    def close_server(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def test_actual_loopback_static_bytes_and_four_history_entries(self):
        report = gate.observe(self.base, self.jar)
        self.assertEqual(len(report["routes"]), 4)
        self.assertEqual(len(report["assets"]), 2)
        self.assertTrue(report["passed"])
        gate.verify_report(report, report["jarSha256"])

    def test_legacy_home_is_not_default_vue3(self):
        self.responses["/"] = (200, "text/html; charset=UTF-8", b"<title>legacy</title>")
        with self.assertRaises(ValueError):
            gate.observe(self.base, self.jar)

    def test_history_route_404_is_not_success(self):
        self.responses["/login"] = (404, "text/html; charset=UTF-8", HTML)
        with self.assertRaises(Exception):
            gate.observe(self.base, self.jar)

    def test_mojibake_html_is_rejected(self):
        self.responses["/"] = (200, "text/html; charset=UTF-8", HTML.decode().encode("latin1", errors="replace"))
        with self.assertRaises(ValueError):
            gate.observe(self.base, self.jar)

    def test_different_asset_or_html_fallback_is_rejected(self):
        for mime, body in (("application/javascript", b"different"), ("text/html; charset=UTF-8", HTML)):
            with self.subTest(mime=mime):
                self.responses["/static/index-generated.js"] = (200, mime, body)
                with self.assertRaises(ValueError):
                    gate.observe(self.base, self.jar)

    def test_redirect_is_not_followed(self):
        self.responses["/"] = (302, "text/plain", b"")
        with self.assertRaises(ValueError):
            gate.observe(self.base, self.jar)
        self.assertEqual(self.redirect_destination_hits, 0)

    def test_non_loopback_or_credentialed_base_is_rejected_before_http(self):
        for base in ("https://127.0.0.1:1", "http://localhost:1", "http://127.0.0.1:1/path",
                     "http://credential@127.0.0.1:1", "http://127.0.0.1:1?query", "http://example.com:1"):
            with self.subTest(base=base), self.assertRaises(ValueError):
                gate.observe(base, self.jar)

    def test_report_rejects_missing_duplicate_or_tampered_observations(self):
        valid = gate.observe(self.base, self.jar)
        mutations = (
            lambda r: r.update(passed=False), lambda r: r.update(legacyEntryDifferent=False),
            lambda r: r.update(schemaVersion=True), lambda r: r.update(jarSha256="f" * 64),
            lambda r: r["routes"].pop(), lambda r: r["routes"][0].update(status=True),
            lambda r: r["routes"][0].update(sha256="f" * 64),
            lambda r: r["assets"].append(copy.deepcopy(r["assets"][0])),
            lambda r: r["assets"][0].update(packagedBytesMatch=False),
            lambda r: r["assets"][0].update(path="https://example.com/script.js"),
            lambda r: r.update(unexpected="field"),
        )
        for mutate in mutations:
            report = copy.deepcopy(valid)
            mutate(report)
            with self.subTest(mutation=repr(mutate)), self.assertRaises(ValueError):
                gate.verify_report(report, valid["jarSha256"])

    def test_duplicate_report_fields_and_nonfinite_numbers_are_rejected(self):
        for text in ('{"passed": true, "passed": true}', '{"value": NaN}'):
            with self.assertRaises(ValueError):
                gate.strict_json(text)


if __name__ == "__main__":
    unittest.main()
