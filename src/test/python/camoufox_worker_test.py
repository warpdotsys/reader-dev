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


class GeneratedScriptPage(GeneratedPage):
    """Controlled IPC/event-loop double; does not execute JavaScript."""

    def __init__(self):
        super().__init__()
        self.result = {"status": "pending"}
        self.started = 0
        self.reads = 0
        self.cleared = 0
        self.key = None
        self.on_start = None
        self.on_read = None
        self.on_clear = None

    def evaluate(self, script, argument):
        if script == worker.SOURCE_SCRIPT_START:
            self.started += 1
            self.key = argument["key"]
            if self.on_start:
                self.on_start()
        elif script == worker.SOURCE_SCRIPT_READ:
            assert argument == self.key
            self.reads += 1
            if self.on_read:
                self.on_read()
            return self.result
        elif script == worker.SOURCE_SCRIPT_CLEAR:
            assert argument == self.key
            self.cleared += 1
            if self.on_clear:
                self.on_clear()
            self.key = None
        else:
            raise AssertionError("Unexpected generated script operation")


class WorkerSourceScriptBudgetTest(unittest.TestCase):
    def evaluate(self, page, timeout=1000, check_allowed=lambda: None):
        return worker.evaluate_source_script(page, "generated-source", timeout,
                                             check_allowed, monotonic=lambda: page.now)

    def test_delayed_completion_is_read_without_replaying_the_rule(self):
        page = GeneratedScriptPage()
        page.schedule(0.12, lambda: setattr(page, "result", {"status": "done", "body": "generated"}))
        self.assertEqual("generated", self.evaluate(page))
        self.assertGreaterEqual(page.reads, 3)
        self.assertEqual(1, page.started)
        self.assertEqual(1, page.cleared)
        self.assertIsNone(page.key)

    def test_sync_and_empty_strings_are_preserved(self):
        for body in ("", "0", "[]", "{\"generated\":true}", "生成"):
            with self.subTest(body=body):
                page = GeneratedScriptPage()
                page.result = {"status": "done", "body": body}
                self.assertEqual(body, self.evaluate(page))
                self.assertEqual(1, page.reads)
                self.assertEqual(1, page.started)

    def test_pending_result_has_one_monotonic_budget_and_is_cleaned_up(self):
        page = GeneratedScriptPage()
        with self.assertRaises(worker.SourceScriptTimeout):
            self.evaluate(page, timeout=500)
        self.assertLessEqual(page.now, 0.500001)
        self.assertEqual(1, page.started)
        self.assertEqual(1, page.cleared)

    def test_start_operation_time_is_part_of_the_same_budget(self):
        page = GeneratedScriptPage()
        page.on_start = lambda: setattr(page, "now", 0.6)
        with self.assertRaises(worker.SourceScriptTimeout):
            self.evaluate(page, timeout=500)
        self.assertEqual(0, page.reads)
        self.assertEqual(1, page.started)
        self.assertEqual(1, page.cleared)

    def test_result_read_cannot_accept_completion_after_the_deadline(self):
        page = GeneratedScriptPage()
        page.result = {"status": "done", "body": "late"}
        page.on_read = lambda: setattr(page, "now", 0.6)
        with self.assertRaises(worker.SourceScriptTimeout):
            self.evaluate(page, timeout=500)
        self.assertEqual(1, page.started)

    def test_rejection_returns_only_a_sanitized_exception_class(self):
        page = GeneratedScriptPage()
        page.result = {"status": "rejected"}
        with self.assertRaises(worker.SourceScriptRejected) as caught:
            self.evaluate(page)
        self.assertEqual("", str(caught.exception))
        self.assertEqual(1, page.cleared)

    def test_replaced_document_fails_instead_of_reexecuting_the_rule(self):
        page = GeneratedScriptPage()
        page.schedule(0.05, lambda: setattr(page, "result", None))
        with self.assertRaises(worker.SourceScriptStateLost):
            self.evaluate(page)
        self.assertEqual(1, page.started)
        self.assertEqual(1, page.cleared)

    def test_unknown_transport_failure_is_not_hidden_or_retried(self):
        page = GeneratedScriptPage()
        def fail():
            raise RuntimeError("generated-transport")
        page.on_read = fail
        with self.assertRaisesRegex(RuntimeError, "generated-transport"):
            self.evaluate(page)
        self.assertEqual(1, page.reads)
        self.assertEqual(1, page.started)

    def test_cleanup_failure_does_not_replace_the_original_timeout(self):
        page = GeneratedScriptPage()
        def fail():
            raise RuntimeError("generated-closed-page")
        page.on_clear = fail
        with self.assertRaises(worker.SourceScriptTimeout):
            self.evaluate(page, timeout=100)
        self.assertEqual(1, page.cleared)

    def test_network_rejection_while_waiting_is_fatal(self):
        page = GeneratedScriptPage()
        blocked = []
        page.schedule(0.05, lambda: blocked.append(True))
        def check_allowed():
            if blocked:
                raise worker.BlockedScheme()
        with self.assertRaises(worker.BlockedScheme):
            self.evaluate(page, check_allowed=check_allowed)
        self.assertEqual(1, page.started)
        self.assertEqual(1, page.cleared)

    def test_invalid_ipc_state_and_non_string_body_are_rejected(self):
        for result in ([], {}, {"status": "unknown"}, {"status": "done", "body": 42},
                       {"status": "pending", "body": "unexpected"},
                       {"status": "rejected", "message": "generated-secret"}):
            with self.subTest(result=result):
                page = GeneratedScriptPage()
                page.result = result
                with self.assertRaises(worker.SourceScriptStateLost):
                    self.evaluate(page)
                self.assertEqual(1, page.cleared)

    def test_utf8_result_limit_is_enforced_before_accepting_a_value(self):
        page = GeneratedScriptPage()
        page.result = {"status": "done", "body": "生成"}
        with patch.object(worker, "MAX_BODY_UTF8_BYTES", 4):
            with self.assertRaises(worker.ResponseBodyTooLarge):
                self.evaluate(page)
        self.assertEqual(1, page.cleared)

    def test_non_positive_budget_never_executes_a_rule(self):
        page = GeneratedScriptPage()
        for timeout in (0, -1):
            with self.assertRaises(ValueError):
                self.evaluate(page, timeout=timeout)
        self.assertEqual(0, page.started)
        self.assertEqual(0, page.cleared)

    def test_each_call_uses_distinct_nonpersistent_random_state(self):
        keys = []
        for _ in range(2):
            page = GeneratedScriptPage()
            page.result = {"status": "done", "body": ""}
            page.on_start = lambda: keys.append(page.key)
            self.evaluate(page)
        self.assertNotEqual(keys[0], keys[1])
        self.assertTrue(all(key.startswith("__reader_source_") and len(key) == 48 for key in keys))


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
