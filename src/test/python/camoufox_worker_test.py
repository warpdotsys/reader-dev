"""Pure parser/snapshot tests; no Camoufox binary or browser download is required."""

import importlib.util
import io
import json
import os
import sys
import types
import unittest
from pathlib import Path
from urllib.parse import urlsplit
from unittest.mock import MagicMock, patch


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


class WorkerNumericalThreadPolicyTest(unittest.TestCase):
    def test_missing_library_options_are_single_threaded(self):
        environment = {}
        worker.prepare_numerical_runtime(environment)
        self.assertEqual({name: "1" for name in worker.NUMERICAL_THREAD_ENV}, environment)

    def test_inherited_host_sized_or_invalid_options_are_not_kept(self):
        for value in ("32", "64", "0", "invalid", ""):
            with self.subTest(value=value):
                environment = {name: value for name in worker.NUMERICAL_THREAD_ENV}
                worker.prepare_numerical_runtime(environment)
                self.assertTrue(all(value == "1" for value in environment.values()))

    def test_unrelated_runtime_network_and_account_options_are_unchanged(self):
        unrelated = {"HOME": "/generated/home", "PATH": "/generated/bin",
                     "READER_BROWSER_ALLOW_PRIVATE_NETWORKS": "false",
                     "READER_PROBE_NAMESPACE": "generated"}
        environment = dict(unrelated)
        worker.prepare_numerical_runtime(environment)
        worker.prepare_numerical_runtime(environment)
        self.assertEqual(unrelated, {key: environment[key] for key in unrelated})

    def test_policy_runs_before_the_first_camoufox_import(self):
        observed = []
        dependency = types.ModuleType("camoufox")

        def imported_attribute(name):
            if name not in ("Camoufox", "DefaultAddons", "NewContext"):
                raise AttributeError(name)
            observed.append({key: os.environ.get(key) for key in worker.NUMERICAL_THREAD_ENV})
            return types.SimpleNamespace(UBO="generated") if name == "DefaultAddons" else object

        dependency.__getattr__ = imported_attribute
        path = Path(__file__).parents[2] / "main" / "resources" / "camoufox" / "worker.py"
        spec = importlib.util.spec_from_file_location("generated_worker_import_order", path)
        module = importlib.util.module_from_spec(spec)
        with patch.dict(sys.modules, {"camoufox": dependency}), \
                patch.dict(os.environ, {name: "32" for name in worker.NUMERICAL_THREAD_ENV}), \
                patch.object(sys, "stdout", io.StringIO()), patch.object(sys, "stderr", io.StringIO()):
            spec.loader.exec_module(module)
        self.assertGreaterEqual(len(observed), 3)
        self.assertTrue(all(all(value == "1" for value in record.values()) for record in observed))


class WorkerInitialNavigationPolicyTest(unittest.TestCase):
    def test_empty_source_keeps_the_existing_dom_ready_snapshot_policy(self):
        for source in (None, ""):
            self.assertEqual("domcontentloaded", worker.initial_navigation_wait_until(source, None))

    def test_source_regex_keeps_its_dom_ready_resource_sniffing_policy(self):
        pattern = worker.re.compile("generated-resource")
        for source in (None, "", "generated-rule"):
            self.assertEqual("domcontentloaded", worker.initial_navigation_wait_until(source, pattern))

    def fixture(self):
        browser = MagicMock()
        context = MagicMock()
        context.cookies.return_value = []
        page = context.new_page.return_value
        payload = {"url": "https://generated.invalid/start", "timeoutMs": 1234,
                   "proxy": "http://127.0.0.1:1", "javaScript": "'generated-source'"}
        return browser, context, page, payload

    def test_render_uses_load_for_both_existing_goto_and_html_calls_then_runs_the_rule_once(self):
        # Mock protocol integration only. Actual browser navigation is accepted
        # separately by the mandatory generated HTTP contract on hosted runners.
        for html in (None, "<p>generated-only</p>"):
            with self.subTest(html_present=html is not None):
                browser, context, page, payload = self.fixture()
                if html is not None:
                    payload["html"] = html
                with patch.object(worker, "Camoufox", return_value=browser), \
                        patch.object(worker, "NewContext", return_value=context), \
                        patch.object(worker, "MainDocumentSnapshot") as snapshot, \
                        patch.object(worker, "evaluate_source_script", return_value="generated-result") as rule:
                    result = worker.render(payload)
                page.goto.assert_called_once_with(payload["url"], wait_until="load", timeout=1234)
                if html is None:
                    page.set_content.assert_not_called()
                else:
                    page.set_content.assert_called_once_with(html, wait_until="load", timeout=1234)
                snapshot.assert_not_called()
                rule.assert_called_once()
                self.assertEqual((page, payload["javaScript"], 1234), rule.call_args.args[:3])
                self.assertEqual({"body": "generated-result", "cookies": []}, result)
                context.close.assert_called_once()

    def test_initial_navigation_failure_is_fatal_without_rule_execution_or_request_replay(self):
        browser, context, page, payload = self.fixture()
        failure = TimeoutError("generated-load-timeout")
        page.goto.side_effect = failure
        with patch.object(worker, "Camoufox", return_value=browser), \
                patch.object(worker, "NewContext", return_value=context), \
                patch.object(worker, "evaluate_source_script") as rule:
            with self.assertRaises(TimeoutError) as raised:
                worker.render(payload)
        self.assertIs(failure, raised.exception)
        page.goto.assert_called_once_with(payload["url"], wait_until="load", timeout=1234)
        page.set_content.assert_not_called()
        rule.assert_not_called()
        context.close.assert_called_once()


class WorkerPostHeaderPolicyTest(unittest.TestCase):
    def fixture(self, headers=None, body="q=post", post=True, browser_headers=None):
        browser, context, page, payload = WorkerInitialNavigationPolicyTest().fixture()
        payload.update(post=post, body=body, headers=headers or {})
        requests = []

        def navigate(*_args, **_kwargs):
            handle = context.route.call_args.args[1]
            for navigation, target in ((True, payload["url"]), (False, payload["url"] + "/asset"),
                                       (True, payload["url"])):
                route = MagicMock()
                route.request.url = target
                route.request.is_navigation_request.return_value = navigation
                route.request.headers = (browser_headers if browser_headers is not None else
                                         {"accept": "text/html", "user-agent": "generated-agent"})
                handle(route)
                requests.append(route.continue_.call_args.kwargs)

        page.goto.side_effect = navigate
        with patch.object(worker, "Camoufox", return_value=browser), \
                patch.object(worker, "NewContext", return_value=context), \
                patch.object(worker, "await_origin_header_policy") as readiness, \
                patch.object(worker, "evaluate_source_script", return_value="generated-result"):
            worker.render(payload)
            readiness.assert_called_once()
            if payload["headers"] or payload["post"]:
                self.assertIsNotNone(readiness.call_args.args[1])
                self.assertEqual(bool(post) and not any(name.lower()=="content-type" for name in payload["headers"]),
                                 readiness.call_args.args[1]["defaultFormPost"])
        context.set_extra_http_headers.assert_not_called()
        return requests, payload, context

    def test_default_post_has_the_observed_legacy_form_type_without_changing_body(self):
        requests, payload, context = self.fixture({"X-Fixture": "synthetic"})
        self.assertEqual("POST", requests[0]["method"])
        self.assertEqual("q=post", requests[0]["post_data"])
        self.assertNotIn("headers", requests[0])
        self.assertEqual({}, requests[1])
        self.assertEqual({}, requests[2])
        self.assertEqual({"X-Fixture": "synthetic"}, payload["headers"])
        context.set_extra_http_headers.assert_not_called()

    def test_explicit_content_type_in_any_case_is_not_overridden_or_rewritten(self):
        for name in ("Content-Type", "content-type", "CONTENT-TYPE"):
            for value in ("application/json; charset=utf-8", "", "application/custom"):
                with self.subTest(name=name, value=value):
                    requests, _payload, _context = self.fixture({name: value})
                    self.assertEqual({"method": "POST", "post_data": "q=post"}, requests[0])

    def test_default_post_does_not_freeze_cookie_or_transport_headers_across_redirects(self):
        managed = {"Host": "generated.invalid", "Cookie": "generated_session=NOT_A_REAL_COOKIE",
                   "Content-Length": "0", "Proxy-Authorization": "GENERATED_PROXY_ONLY",
                   "proxy-connection": "keep-alive", "CONNECTION": "close"}
        native = {**managed, "accept": "text/html", "user-agent": "generated-agent"}
        requests, _payload, _context = self.fixture(browser_headers=native)
        self.assertEqual({"method": "POST", "post_data": "q=post"}, requests[0])
        self.assertEqual({**managed, "accept": "text/html", "user-agent": "generated-agent"}, native)
        self.assertEqual([{}, {}], requests[1:])

    def test_get_never_injects_a_form_header_or_changes_the_method(self):
        requests, _payload, _context = self.fixture(post=False)
        self.assertEqual([{}, {}, {}], requests)

    def test_empty_body_keeps_legacy_empty_post_without_encoding_or_replay(self):
        for body in (None, ""):
            with self.subTest(body=body):
                requests, _payload, _context = self.fixture(body=body)
                self.assertEqual("", requests[0]["post_data"])
                self.assertNotIn("headers", requests[0])
                self.assertEqual([{}, {}], requests[1:])


class WorkerOriginHeaderPolicyTest(unittest.TestCase):
    def test_form_post_without_rule_headers_still_gets_private_native_policy(self):
        with worker.origin_header_addon("https://generated.invalid/book", {}, post=True) as policy:
            self.assertTrue(policy["defaultFormPost"])
            source=(Path(policy["addons"][0])/"policy.js").read_text(encoding="utf-8")
            self.assertIn('"defaultFormPost": true',source)
            self.assertIn('"headers": {}',source)
        for name in ("Content-Type","CONTENT-TYPE"):
            with worker.origin_header_addon("https://generated.invalid/book", {name:""}, post=True) as policy:
                self.assertFalse(policy["defaultFormPost"])

    def test_empty_headers_have_no_addon_or_bootstrap(self):
        with patch.object(worker.tempfile, "TemporaryDirectory") as files:
            with worker.origin_header_addon("https://generated.invalid", {}) as policy:
                self.assertIsNone(policy)
            files.assert_not_called()
        context = MagicMock()
        worker.await_origin_header_policy(context, None, 1000)
        context.new_page.assert_not_called()

    def test_normalization_preserves_values_and_does_not_mutate_input(self):
        headers = {"Authorization": "Bearer GENERATED", "X-Trace": "中文", "Content-Type": ""}
        self.assertEqual({"authorization": "Bearer GENERATED", "x-trace": "中文", "content-type": ""},
                         worker.bounded_rule_headers(headers))
        self.assertEqual(["Authorization", "X-Trace", "Content-Type"], list(headers))

    def test_invalid_headers_fail_before_files_or_browser_launch(self):
        invalid = [[], "bad", {"A B": "x"}, {"X": "a\rb"}, {"X": "a\nb"},
                   {"X": "a\x00b"}, {"X": 7}, {7: "x"}, {"X": "a\x7fb"},
                   {"X": "one", "x": "two"}]
        for headers in invalid:
            with self.subTest(kind=type(headers).__name__), \
                    patch.object(worker.tempfile, "TemporaryDirectory") as files:
                with self.assertRaises(worker.HeaderPolicyError):
                    with worker.origin_header_addon("https://generated.invalid", headers):
                        self.fail("Invalid policy yielded")
                files.assert_not_called()

    def test_managed_credentials_and_transport_headers_cannot_enter_rule_addon(self):
        for name in ("Cookie", "User-Agent", "HOST", "Content-Length", "Proxy-Authorization"):
            with self.subTest(name=name), self.assertRaises(worker.HeaderPolicyError):
                worker.bounded_rule_headers({name: "GENERATED"})

    def test_header_pair_and_utf8_limits_are_enforced_without_truncation(self):
        allowed = {"X" + str(index): "v" for index in range(worker.MAX_RULE_HEADERS)}
        self.assertEqual(64, len(worker.bounded_rule_headers(allowed)))
        with self.assertRaises(worker.HeaderPolicyError):
            worker.bounded_rule_headers({**allowed, "Extra": "v"})
        self.assertEqual("a" * 8191, worker.bounded_rule_headers({"X": "a" * 8191})["x"])
        for value in ("a" * 8192, "中" * 2731):
            with self.assertRaises(worker.HeaderPolicyError):
                worker.bounded_rule_headers({"X": value})

    def test_addon_is_private_not_web_accessible_and_removed_after_success(self):
        with worker.origin_header_addon("https://generated.invalid/book", {"Authorization": "GENERATED_ONLY"}) as policy:
            directory = Path(policy["addons"][0])
            manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
            self.assertNotIn("web_accessible_resources", manifest)
            self.assertNotIn("GENERATED_ONLY", policy["bootstrapUrl"])
            self.assertNotIn("GENERATED_ONLY", (directory / "ready.js").read_text(encoding="utf-8"))
            self.assertIn('"authorization": "GENERATED_ONLY"', (directory / "policy.js").read_text(encoding="utf-8"))
            self.assertEqual({"policy.js", "manifest.json", "ready.js"}, {file.name for file in directory.iterdir()})
            if os.name == "posix":
                self.assertEqual(0o700, directory.stat().st_mode & 0o777)
                self.assertTrue(all(file.stat().st_mode & 0o777 == 0o600 for file in directory.iterdir()))
        self.assertFalse(directory.exists())

    def test_addon_files_are_removed_on_browser_failure(self):
        with self.assertRaises(RuntimeError):
            with worker.origin_header_addon("https://generated.invalid", {"X": "GENERATED"}) as policy:
                directory = Path(policy["addons"][0])
                raise RuntimeError("generated failure")
        self.assertFalse(directory.exists())

    def test_ready_bootstrap_is_fulfilled_without_headers_and_always_closed(self):
        context = MagicMock()
        policy = {"bootstrapUrl": "http://reader-header-generated.invalid/ready", "nonce": "GENERATED_NONCE"}
        with patch.object(worker.time, "monotonic", side_effect=[0, .1, .2]):
            worker.await_origin_header_policy(context, policy, 1000)
        context.route.assert_called_once()
        self.assertEqual(policy["bootstrapUrl"], context.route.call_args.args[0])
        route = MagicMock()
        context.route.call_args.args[1](route)
        self.assertEqual(200, route.fulfill.call_args.kwargs["status"])
        self.assertNotIn("headers", route.fulfill.call_args.kwargs)
        context.unroute.assert_called_once_with(policy["bootstrapUrl"])
        context.new_page.return_value.close.assert_called_once()

    def test_only_private_bootstrap_can_retry_with_a_decreasing_deadline(self):
        context = MagicMock()
        page = context.new_page.return_value
        page.goto.side_effect = [RuntimeError("generated unavailable"), None]
        policy = {"bootstrapUrl": "http://reader-header-generated.invalid/ready", "nonce": "GENERATED_NONCE"}
        with patch.object(worker.time, "monotonic", side_effect=[0, .1, .2, .3, .4]):
            worker.await_origin_header_policy(context, policy, 1000)
        self.assertEqual(2, page.goto.call_count)
        self.assertTrue(all(call.args == (policy["bootstrapUrl"],) for call in page.goto.call_args_list))
        self.assertGreater(page.goto.call_args_list[0].kwargs["timeout"], page.goto.call_args_list[1].kwargs["timeout"])
        page.close.assert_called_once()

    def test_readiness_failure_closes_context_before_cookies_or_business_navigation(self):
        browser, context, page, payload = WorkerInitialNavigationPolicyTest().fixture()
        payload["headers"] = {"Authorization": "GENERATED_ONLY"}
        with patch.object(worker, "Camoufox", return_value=browser) as launch, \
                patch.object(worker, "NewContext", return_value=context), \
                patch.object(worker, "await_origin_header_policy", side_effect=worker.HeaderPolicyError()), \
                patch.object(worker, "evaluate_source_script") as rule:
            with self.assertRaises(worker.HeaderPolicyError):
                worker.render(payload)
        page.goto.assert_not_called()
        context.add_cookies.assert_not_called()
        context.set_extra_http_headers.assert_not_called()
        context.close.assert_called_once()
        self.assertTrue(launch.call_args.kwargs["firefox_user_prefs"]["network.proxy.allow_hijacking_localhost"])
        self.assertEqual({"server": payload["proxy"]}, launch.call_args.kwargs["proxy"])
        rule.assert_not_called()

    def test_expired_bootstrap_is_fatal_and_does_not_echo_policy_values(self):
        context = MagicMock()
        policy = {"bootstrapUrl": "http://reader-header-generated.invalid/ready", "nonce": "GENERATED_NONCE"}
        with patch.object(worker.time, "monotonic", side_effect=[0, 1.01]):
            with self.assertRaises(worker.HeaderPolicyError) as failure:
                worker.await_origin_header_policy(context, policy, 1000)
        self.assertEqual("", str(failure.exception))
        context.new_page.return_value.goto.assert_not_called()
        context.new_page.return_value.close.assert_called_once()


class WorkerBrowserDiagnosticTest(unittest.TestCase):
    def test_annotating_a_failure_preserves_the_original_exception_and_protocol_class(self):
        error = RuntimeError("PRIVATE_BODY https://generated.invalid/?cookie=PRIVATE_COOKIE")
        with self.assertRaises(RuntimeError) as raised:
            with worker.browser_operation("snapshotContent"):
                raise error
        self.assertIs(error, raised.exception)
        self.assertEqual("RuntimeError", type(raised.exception).__name__)
        self.assertEqual({"operation": "snapshotContent", "kind": "unclassified", "errorClass": "Other"},
                         worker.browser_failure_diagnostic(error))

    def test_fixed_error_kinds_never_echo_the_message_or_credentials(self):
        Error = type("Error", (Exception,), {})
        cases = {
            "NS_BINDING_ABORTED": "navigationInterrupted",
            "is interrupted by another navigation": "navigationInterrupted",
            "Execution context was destroyed": "executionContextDestroyed",
            "Unable to retrieve content because the page is navigating and changing the content": "documentChanging",
            "unknown browser transport": "unclassified",
        }
        for message, expected in cases.items():
            error = Error(message + " PRIVATE_BODY https://generated.invalid/?ticket=PRIVATE_COOKIE")
            with self.assertRaises(Error):
                with worker.browser_operation("initialNavigation"):
                    raise error
            diagnostic = worker.browser_failure_diagnostic(error)
            self.assertEqual(expected, diagnostic["kind"])
            self.assertEqual("Error", diagnostic["errorClass"])
            self.assertNotIn("PRIVATE", str(diagnostic))
            self.assertNotIn("generated.invalid", str(diagnostic))

    def test_unknown_or_missing_phase_does_not_emit_a_diagnostic(self):
        error = RuntimeError("PRIVATE_BODY")
        self.assertIsNone(worker.browser_failure_diagnostic(error))
        error._reader_browser_operation = "PRIVATE_COOKIE"
        self.assertIsNone(worker.browser_failure_diagnostic(error))
        error._reader_browser_operation = {"PRIVATE_COOKIE": "PRIVATE_BODY"}
        self.assertIsNone(worker.browser_failure_diagnostic(error))
        with self.assertRaises(ValueError):
            with worker.browser_operation("PRIVATE_COOKIE"):
                self.fail("An untrusted operation must not run")

    def test_nested_operations_keep_the_innermost_failure_without_retrying(self):
        error = RuntimeError("generated-fatal")
        calls = []
        with self.assertRaises(RuntimeError):
            with worker.browser_operation("initialNavigation"):
                with worker.browser_operation("snapshotContent"):
                    calls.append(1)
                    raise error
        self.assertEqual([1], calls)
        self.assertEqual("snapshotContent", worker.browser_failure_diagnostic(error)["operation"])


    def test_diagnostics_cannot_replace_an_unprintable_failure(self):
        class Error(Exception):
            def __str__(self):
                raise RuntimeError("PRIVATE_BODY")
        error = Error()
        with self.assertRaises(Error) as raised:
            with worker.browser_operation("snapshotLoadState"):
                raise error
        self.assertIs(error, raised.exception)
        self.assertEqual("unclassified", worker.browser_failure_diagnostic(error)["kind"])

    def test_main_keeps_the_wire_error_class_and_emits_only_fixed_stderr_labels(self):
        Error = type("Error", (Exception,), {})
        error = Error("NS_BINDING_ABORTED PRIVATE_BODY https://generated.invalid/?cookie=PRIVATE_COOKIE")
        error._reader_browser_operation = "initialNavigation"
        output, diagnostic_output = io.StringIO(), io.StringIO()
        with patch.object(worker, "render", side_effect=error), \
                patch.object(worker, "protocol_out", output), \
                patch.object(worker.sys, "stdin", io.StringIO("{}\n")), \
                patch.object(worker.sys, "stderr", diagnostic_output):
            worker.main()
        self.assertEqual({"error": "Error"}, json.loads(output.getvalue()))
        self.assertEqual(1, len(diagnostic_output.getvalue().splitlines()))
        self.assertEqual('READER_BROWSER_FAILURE {"operation":"initialNavigation","kind":"navigationInterrupted","errorClass":"Error"}\n',
                         diagnostic_output.getvalue())
        self.assertNotIn("PRIVATE", output.getvalue() + diagnostic_output.getvalue())


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

    def remove_listener(self, name, callback):
        if self.handlers.get(name) is callback:
            del self.handlers[name]

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

    def changing_content_error(self):
        Error = type("Error", (Exception,), {})
        return Error("Page.content: Unable to retrieve content because the page is navigating and changing the content.")

    def test_known_read_only_content_race_waits_again_and_returns_only_the_new_document(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        def race():
            page.body = "generated-final"
            raise self.changing_content_error()
        page.on_content = race
        self.assertEqual("generated-final", snapshot.read(2000, lambda: None))
        self.assertEqual(2, page.content_calls)
        self.assertGreaterEqual(page.now, 0.4)

    def test_endless_content_races_cannot_renew_the_snapshot_deadline(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        def race():
            page.on_content = race
            raise self.changing_content_error()
        page.on_content = race
        with self.assertRaises(TimeoutError):
            snapshot.read(500, lambda: None)
        self.assertGreaterEqual(page.content_calls, 2)
        self.assertLessEqual(page.now, 0.500001)

    def test_content_race_after_deadline_cannot_start_another_read(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        def race():
            page.now = 1.0
            raise self.changing_content_error()
        page.on_content = race
        with self.assertRaises(TimeoutError):
            snapshot.read(500, lambda: None)
        self.assertEqual(1, page.content_calls)

    def test_network_rejection_during_content_race_stays_fatal_before_another_read(self):
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        denied = []
        def race():
            denied.append(True)
            raise self.changing_content_error()
        def check_allowed():
            if denied:
                raise worker.BlockedScheme()
        page.on_content = race
        with self.assertRaises(worker.BlockedScheme):
            snapshot.read(2000, check_allowed)
        self.assertEqual(1, page.content_calls)

    def test_lookalike_and_other_browser_errors_are_not_hidden_or_retried(self):
        Error = type("Error", (Exception,), {})
        errors = [RuntimeError(str(self.changing_content_error())),
                  Error("Execution context was destroyed"), Error("Target page has been closed")]
        for error in errors:
            with self.subTest(kind=type(error).__name__):
                page = GeneratedPage()
                snapshot = self.snapshot(page)
                def fail():
                    raise error
                page.on_content = fail
                with self.assertRaises(type(error)) as raised:
                    snapshot.read(2000, lambda: None)
                self.assertIs(error, raised.exception)
                self.assertEqual(1, page.content_calls)

    def test_unprintable_browser_failure_keeps_the_original_exception(self):
        def unprintable(_):
            raise RuntimeError("generated-message-access-failure")
        Error = type("Error", (Exception,), {"__str__": unprintable})
        error = Error()
        page = GeneratedPage()
        snapshot = self.snapshot(page)
        def fail():
            raise error
        page.on_content = fail
        with self.assertRaises(Error) as raised:
            snapshot.read(2000, lambda: None)
        self.assertIs(error, raised.exception)
        self.assertEqual(1, page.content_calls)

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


class GeneratedDocumentHandle:
    def __init__(self, page):
        self.page = page

    def dispose(self):
        self.page.handles_disposed += 1
        if self.page.on_dispose:
            self.page.on_dispose()


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
        self.document_observation = True
        self.document_reads = 0
        self.handles_disposed = 0
        self.on_document_read = None
        self.on_dispose = None

    def evaluate_handle(self, script, argument):
        assert script == worker.SOURCE_SCRIPT_START
        self.evaluate(script, argument)
        return GeneratedDocumentHandle(self)

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
        elif script == worker.SOURCE_SCRIPT_DOCUMENT:
            assert isinstance(argument, GeneratedDocumentHandle)
            self.document_reads += 1
            if self.on_document_read:
                self.on_document_read()
            return self.document_observation
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

    def test_lost_state_document_observation_is_finite_and_reads_once(self):
        for observed, expected in ((True, "sameDocument"), (False, "differentDocument"),
                                   (None, "unavailable"), ("PRIVATE_BODY", "unavailable"),
                                   (1, "unavailable")):
            page = GeneratedScriptPage()
            page.result = None
            page.document_observation = observed
            with self.assertRaises(worker.SourceScriptStateLost) as raised:
                self.evaluate(page)
            diagnostic = worker.browser_failure_diagnostic(raised.exception)
            self.assertEqual(expected, diagnostic["sourceDocumentObservation"])
            self.assertNotIn("PRIVATE", json.dumps(diagnostic))
            self.assertEqual(1, page.document_reads)
            self.assertEqual(1, page.handles_disposed)
            self.assertEqual(1, page.started)
            self.assertEqual(1, page.reads)

    def test_document_read_or_dispose_failure_never_replaces_lost_state(self):
        page = GeneratedScriptPage()
        page.result = None
        def fail():
            raise RuntimeError("PRIVATE_CLOSED_CONTEXT")
        page.on_document_read = fail
        page.on_dispose = fail
        with self.assertRaises(worker.SourceScriptStateLost) as raised:
            self.evaluate(page)
        diagnostic = worker.browser_failure_diagnostic(raised.exception)
        self.assertEqual("unavailable", diagnostic["sourceDocumentObservation"])
        self.assertEqual("", str(raised.exception))
        self.assertNotIn("PRIVATE", json.dumps(diagnostic))
        self.assertEqual(1, page.document_reads)
        self.assertEqual(1, page.handles_disposed)

    def test_document_observation_does_not_extend_budget_or_accept_late_boolean(self):
        page = GeneratedScriptPage()
        page.result = None
        page.on_document_read = lambda: setattr(page, "now", 0.6)
        with self.assertRaises(worker.SourceScriptStateLost) as raised:
            self.evaluate(page, timeout=500)
        self.assertEqual("unavailable", worker.browser_failure_diagnostic(raised.exception)["sourceDocumentObservation"])
        self.assertEqual(1, page.started)
        self.assertEqual(1, page.document_reads)
        self.assertEqual(1, page.handles_disposed)

    def test_document_handle_is_disposed_without_extra_read_for_success_or_timeout(self):
        for pending in (False, True):
            page = GeneratedScriptPage()
            if pending:
                with self.assertRaises(worker.SourceScriptTimeout):
                    self.evaluate(page, timeout=100)
            else:
                page.result = {"status": "done", "body": "generated"}
                self.assertEqual("generated", self.evaluate(page))
            self.assertEqual(0, page.document_reads)
            self.assertEqual(1, page.handles_disposed)
            self.assertEqual(1, page.started)

    def test_expired_or_denied_document_observation_does_not_read_page(self):
        for denied in (False, True):
            page = GeneratedScriptPage()
            holder = GeneratedDocumentHandle(page)
            def check_allowed():
                if denied:
                    raise worker.BlockedScheme()
            observed = worker.source_document_observation(page, holder, 0 if not denied else 1,
                                                          check_allowed, lambda: 0)
            self.assertEqual("unavailable", observed)
            self.assertEqual(0, page.document_reads)

    def test_unknown_document_observation_attribute_never_echoes_page_data(self):
        error = worker.SourceScriptStateLost()
        error._reader_browser_operation = "sourceScriptRead"
        error._reader_source_state_kind = "sourceStateMissing"
        error._reader_main_frame_navigation_observed = False
        for value in ("PRIVATE_BODY", {"PRIVATE_COOKIE": True}, 1, True):
            error._reader_source_document_observation = value
            diagnostic = worker.browser_failure_diagnostic(error)
            self.assertNotIn("sourceDocumentObservation", diagnostic)
            self.assertNotIn("PRIVATE", json.dumps(diagnostic))

    def test_four_lost_state_reasons_are_finite_and_do_not_echo_state_values(self):
        for result, reason in ((None, "sourceStateMissing"),
                               ("PRIVATE_BODY", "sourceStateTypeInvalid"),
                               ({"status": "done", "body": 42}, "sourceStateBodyInvalid"),
                               ({"status": "PRIVATE_COOKIE"}, "sourceStateStatusInvalid")):
            page = GeneratedScriptPage()
            page.result = result
            with self.assertRaises(worker.SourceScriptStateLost) as raised:
                self.evaluate(page)
            self.assertEqual("", str(raised.exception))
            diagnostic = worker.browser_failure_diagnostic(raised.exception)
            self.assertEqual({"operation": "sourceScriptRead", "kind": reason,
                              "errorClass": "SourceScriptStateLost", "mainFrameNavigationObserved": False,
                              "sourceDocumentObservation": "sameDocument"}, diagnostic)
            self.assertNotIn("PRIVATE", json.dumps(diagnostic))
            self.assertEqual(1, page.started)
            self.assertEqual(1, page.cleared)
            self.assertNotIn("framenavigated", page.handlers)

    def test_navigation_bit_observes_only_main_frame_events_during_this_rule(self):
        for main in (False, True):
            page = GeneratedScriptPage()
            def navigate():
                page.emit("framenavigated", page.main_frame if main else object())
                page.result = None
            page.schedule(0.05, navigate)
            with self.assertRaises(worker.SourceScriptStateLost) as raised:
                self.evaluate(page)
            diagnostic = worker.browser_failure_diagnostic(raised.exception)
            self.assertEqual("sourceStateMissing", diagnostic["kind"])
            self.assertIs(main, diagnostic["mainFrameNavigationObserved"])
            self.assertNotIn("framenavigated", page.handlers)

    def test_state_diagnostic_rejects_unknown_labels_and_non_boolean_navigation(self):
        error = worker.SourceScriptStateLost()
        error._reader_browser_operation = "sourceScriptRead"
        for reason, observed in (("PRIVATE_COOKIE", True), ({"PRIVATE": "BODY"}, True),
                                 ("sourceStateMissing", "PRIVATE_COOKIE"), ("sourceStateMissing", 1)):
            error._reader_source_state_kind = reason
            error._reader_main_frame_navigation_observed = observed
            self.assertEqual({"operation": "sourceScriptRead", "kind": "unclassified", "errorClass": "Other"},
                             worker.browser_failure_diagnostic(error))

    def test_main_preserves_lost_state_wire_class_and_only_finite_stderr_diagnostic(self):
        page = GeneratedScriptPage()
        page.result = None
        with self.assertRaises(worker.SourceScriptStateLost) as raised:
            self.evaluate(page)
        output, diagnostic_output = io.StringIO(), io.StringIO()
        with patch.object(worker, "render", side_effect=raised.exception), \
                patch.object(worker, "protocol_out", output), \
                patch.object(worker.sys, "stdin", io.StringIO("{}\n")), \
                patch.object(worker.sys, "stderr", diagnostic_output):
            worker.main()
        self.assertEqual({"error": "SourceScriptStateLost"}, json.loads(output.getvalue()))
        text = diagnostic_output.getvalue().removeprefix("READER_BROWSER_FAILURE ")
        self.assertEqual({"operation": "sourceScriptRead", "kind": "sourceStateMissing",
                          "errorClass": "SourceScriptStateLost", "mainFrameNavigationObserved": False,
                          "sourceDocumentObservation": "sameDocument"}, json.loads(text))

    def test_unknown_transport_failure_is_not_hidden_or_retried(self):
        page = GeneratedScriptPage()
        def fail():
            raise RuntimeError("generated-transport")
        page.on_read = fail
        with self.assertRaisesRegex(RuntimeError, "generated-transport"):
            self.evaluate(page)
        self.assertEqual(1, page.reads)
        self.assertEqual(1, page.started)
        self.assertEqual(0, page.document_reads)
        self.assertEqual(1, page.handles_disposed)

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
