"""Pure parser/snapshot tests; no Camoufox binary or browser download is required."""

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from urllib.parse import urlsplit
from unittest.mock import patch


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


class GeneratedPage:
    """Deterministic event-loop double, not a real browser acceptance claim."""

    def __init__(self):
        self.now = 0.0
        self.main_frame = object()
        self.events = []
        self.handlers = {}
        self.body = "generated-old"
        self.content_calls = 0
        self.on_content = None
        self.on_load_state = None

    def on(self, name, callback):
        self.handlers[name] = callback

    def emit(self, name, item):
        self.handlers[name](item)

    def request(self, main=True, navigation=True):
        return types.SimpleNamespace(is_navigation_request=lambda: navigation,
                                     frame=self.main_frame if main else object())

    def schedule(self, when, callback):
        self.events.append((when, callback))
        self.events.sort(key=lambda event: event[0])

    def wait_for_timeout(self, milliseconds):
        end = self.now + milliseconds / 1000.0
        while self.events and self.events[0][0] <= end:
            self.now, callback = self.events.pop(0)
            callback()
        self.now = end

    def wait_for_load_state(self, state, timeout):
        assert state == "domcontentloaded" and timeout > 0
        if self.on_load_state:
            callback, self.on_load_state = self.on_load_state, None
            callback()

    def content(self):
        self.content_calls += 1
        value = self.body
        if self.on_content:
            callback, self.on_content = self.on_content, None
            callback()
        return value


class WorkerDocumentSnapshotTest(unittest.TestCase):
    def snapshot(self, page):
        return worker.MainDocumentSnapshot(page, monotonic=lambda: page.now)

    def test_pending_main_navigation_cannot_return_the_old_document(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        request = page.request()
        # SimpleNamespace is unhashable; real Playwright Request is identity-hashable.
        request = type("GeneratedRequest", (), {"frame": request.frame,
                       "is_navigation_request": lambda self: True})()
        page.emit("request", request)
        def complete():
            page.body = "generated-final"
            page.emit("framenavigated", page.main_frame)
            page.emit("requestfinished", request)
        page.schedule(0.4, complete)
        self.assertEqual("generated-final", snapshot.read(2000, lambda: None))
        self.assertGreaterEqual(page.now, 0.6)
        self.assertEqual(1, page.content_calls)

    def test_commit_during_snapshot_discards_only_the_obsolete_html(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        def commit():
            page.body = "generated-final"
            page.emit("framenavigated", page.main_frame)
        page.on_content = commit
        self.assertEqual("generated-final", snapshot.read(2000, lambda: None))
        self.assertEqual(2, page.content_calls)

    def test_commit_during_load_state_check_prevents_an_obsolete_snapshot(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        page.on_load_state = lambda: page.emit("framenavigated", page.main_frame)
        self.assertEqual("generated-old", snapshot.read(2000, lambda: None))
        self.assertGreaterEqual(page.now, 0.4)
        self.assertEqual(1, page.content_calls)

    def test_iframe_and_non_navigation_streams_do_not_reset_main_quiet_window(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        page.emit("request", page.request(main=False))
        page.emit("request", page.request(navigation=False))
        page.emit("framenavigated", object())
        self.assertEqual("generated-old", snapshot.read(2000, lambda: None))
        self.assertLess(page.now, 0.4)
        self.assertFalse(snapshot.pending)

    def test_endless_main_navigation_has_one_monotonic_budget(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        for step in range(1, 20):
            page.schedule(step / 20, lambda: page.emit("framenavigated", page.main_frame))
        with self.assertRaises(TimeoutError):
            snapshot.read(500, lambda: None)
        self.assertLessEqual(page.now, 0.500001)
        self.assertEqual(0, page.content_calls)

    def test_network_rejection_during_event_pump_is_fatal_before_snapshot(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        denied = []
        page.schedule(0.05, lambda: denied.append(True))
        def check_allowed():
            if denied:
                raise worker.BlockedScheme()
        with self.assertRaises(worker.BlockedScheme):
            snapshot.read(2000, check_allowed)
        self.assertEqual(0, page.content_calls)

    def test_unknown_content_failure_is_not_hidden_or_retried(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        def fail():
            raise RuntimeError("generated-closed-page")
        page.on_content = fail
        with self.assertRaisesRegex(RuntimeError, "generated-closed-page"):
            snapshot.read(2000, lambda: None)
        self.assertEqual(1, page.content_calls)

    def test_utf8_body_limit_is_enforced_before_accepting_a_snapshot(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        page.body = "生成"
        with patch.object(worker, "MAX_BODY_UTF8_BYTES", 4):
            with self.assertRaises(worker.ResponseBodyTooLarge):
                snapshot.read(2000, lambda: None)

    def test_non_positive_snapshot_budget_is_rejected(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        with self.assertRaises(ValueError):
            snapshot.read(0, lambda: None)
        self.assertEqual(0, page.content_calls)


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
