"""Pure parser/snapshot tests; no Camoufox binary or browser download is required."""

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from urllib.parse import urlsplit


def load_worker():
    # worker.py imports Camoufox at module load. The cases here cover only pure
    # protocol helpers, so inject a minimal module rather than requiring it.
    camoufox = types.ModuleType("camoufox")
    camoufox.Camoufox = object
    camoufox.DefaultAddons = types.SimpleNamespace(UBO="ubo")
    camoufox.NewContext = object
    sys.modules["camoufox"] = camoufox
    path = Path(__file__).parents[2] / "main" / "resources" / "camoufox" / "worker.py"
    spec = importlib.util.spec_from_file_location("reader_camoufox_worker_test", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


worker = load_worker()


class WorkerCookieProtocolTest(unittest.TestCase):
    def test_visible_browser_cookie_wins_over_header_fallback(self):
        snapshot = {"name": "quoted", "value": '"alpha', "deleted": False}
        fallback = {"name": "quoted", "value": '"alpha;beta"', "deleted": False}
        values = {"quoted": snapshot}
        self.assertIs(snapshot, worker.merge_response_cookie_fallbacks(
            values, {"quoted"}, {"quoted": fallback}
        )["quoted"])
        self.assertIs(fallback, worker.merge_response_cookie_fallbacks(
            {}, set(), {"quoted": fallback}
        )["quoted"])
        deletion = {"name": "quoted", "value": "", "deleted": True}
        self.assertIs(deletion, worker.merge_response_cookie_fallbacks(
            {"quoted": snapshot}, {"quoted"}, {"quoted": deletion}
        )["quoted"])

    def test_quoted_semicolon_is_not_mistaken_for_an_attribute_separator(self):
        parsed = worker.parse_set_cookie(
            'quoted="alpha;beta"; Path=/; HttpOnly; '
            'Expires=Wed, 21 Oct 2037 07:28:00 GMT',
            "http://books.example.test/page",
        )
        self.assertEqual('"alpha;beta"', parsed["value"])
        self.assertTrue(parsed["httpOnly"])
        self.assertGreater(parsed["expires"], 0)
        self.assertIsNone(worker.parse_set_cookie(
            'quoted="unterminated; Path=/; HttpOnly',
            "http://books.example.test/page",
        ))

    def test_non_http_only_response_creation_cannot_override_script_deletion(self):
        public = worker.parse_set_cookie(
            "scripted=renewed; Path=/", "http://books.example.test/page"
        )
        hidden = worker.parse_set_cookie(
            "hidden=keep; Path=/; HttpOnly", "http://books.example.test/page"
        )
        deleted = worker.parse_set_cookie(
            "scripted=; Path=/; Max-Age=0", "http://books.example.test/page"
        )
        self.assertFalse(worker.response_cookie_needs_fallback(public))
        self.assertTrue(worker.response_cookie_needs_fallback(hidden))
        self.assertTrue(worker.response_cookie_needs_fallback(deleted))

    def test_domain_delete_fallback_is_forwarded_but_domain_creation_stays_browser_only(self):
        deleted = worker.parse_set_cookie(
            "sid=gone; Domain=example.test; Path=/; Max-Age=0",
            "https://books.example.test/logout",
        )
        self.assertFalse(deleted["hostOnly"])
        self.assertTrue(deleted["deleted"])

        # The caller accepts only host-only creations from raw headers. This is
        # what prevents the fallback from implementing public-suffix acceptance.
        self.assertFalse(deleted["hostOnly"] or deleted["deleted"] is False)

    def test_snapshot_leading_dot_remains_domain_scoped_without_header_metadata(self):
        cookie = worker.portable_snapshot_cookie(
            {"name": "wide", "value": "ok", "domain": ".example.test", "path": "/"},
            urlsplit("https://books.example.test/page"),
            {},
            {},
        )
        self.assertEqual("example.test", cookie["domain"])
        self.assertFalse(cookie["hostOnly"])

    def test_http_secure_fallback_is_rejected(self):
        self.assertIsNone(worker.parse_set_cookie(
            "sid=should-not-persist; Secure; Path=/",
            "http://books.example.test/login",
        ))

    def test_host_only_import_keeps_exact_path_without_widening(self):
        cookie = {
            "name": "session", "value": "alpha", "domain": "127.0.0.1",
            "path": "/scoped", "hostOnly": True, "secure": False,
        }
        imported = worker.as_playwright_cookie(cookie, "http://127.0.0.1:18890/echo")
        self.assertNotIn("url", imported)
        self.assertEqual("127.0.0.1", imported["domain"])
        self.assertEqual("/scoped", imported["path"])

    def test_unchanged_imported_cookie_is_transient_but_response_change_is_returned(self):
        imported = {
            "name": "once", "value": "header", "domain": "books.example.test",
            "path": "/", "hostOnly": True, "secure": False, "httpOnly": False,
            "sameSite": None, "expires": -1, "deleted": False,
        }
        key = worker.cookie_identity(imported)
        self.assertFalse(worker.cookie_changed_from_initial(imported, {key: imported}, set()))

        refreshed = dict(imported, value="server")
        self.assertTrue(worker.cookie_changed_from_initial(refreshed, {key: imported}, {key}))

    def test_response_domain_deletion_remains_a_tombstone(self):
        deleted = worker.parse_set_cookie(
            "sid=; Domain=example.test; Path=/; Max-Age=0",
            "https://books.example.test/logout",
        )
        key = worker.cookie_identity(deleted)
        self.assertTrue(deleted["deleted"])
        self.assertTrue(worker.cookie_changed_from_initial(deleted, {}, {key}))

    def test_only_an_initially_visible_cookie_can_be_deleted_by_snapshot_delta(self):
        cookie = {
            "name": "session", "value": "alpha", "domain": "books.example.test",
            "path": "/", "hostOnly": True, "secure": False, "httpOnly": False,
            "sameSite": None, "expires": -1, "deleted": False,
        }
        key = worker.cookie_identity(cookie)
        tombstones = worker.missing_initial_cookie_tombstones({key: cookie}, set(), set())
        self.assertTrue(tombstones[key]["deleted"])
        self.assertEqual("", tombstones[key]["value"])
        self.assertEqual({}, worker.missing_initial_cookie_tombstones({}, set(), set()))
        self.assertEqual({}, worker.missing_initial_cookie_tombstones({key: cookie}, {key}, set()))
        self.assertEqual({}, worker.missing_initial_cookie_tombstones({key: cookie}, set(), {key}))


if __name__ == "__main__":
    unittest.main()
