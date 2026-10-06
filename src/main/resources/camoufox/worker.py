"""One-request NDJSON worker for the Reader-managed Camoufox process."""

import contextlib
import json
import re
import secrets
import sys
import time
from email.utils import parsedate_to_datetime
from urllib.parse import urlsplit

# Stdout is a machine protocol. Keep all package/browser diagnostics on stderr.
protocol_out = sys.stdout
sys.stdout = sys.stderr
with contextlib.redirect_stdout(sys.stderr):
    from camoufox import Camoufox, DefaultAddons, NewContext


# Both values are byte limits, rather than character limits: Reader's parent
# protocol is UTF-8 and a page can otherwise expand several-fold at serialization.
MAX_BODY_UTF8_BYTES = 4 * 1024 * 1024
MAX_PROTOCOL_UTF8_BYTES = 8 * 1024 * 1024
MAX_COOKIES = 128
MAX_COOKIE_NAME_UTF8_BYTES = 256
MAX_COOKIE_VALUE_UTF8_BYTES = 8 * 1024
MAX_COOKIE_DOMAIN_UTF8_BYTES = 253
MAX_COOKIE_PATH_UTF8_BYTES = 2 * 1024
MAX_COOKIE_SAMESITE_UTF8_BYTES = 16


class ResponseBodyTooLarge(Exception):
    pass


class CookieLimitExceeded(Exception):
    pass


class ResponseTooLarge(Exception):
    pass


class BlockedScheme(Exception):
    pass


class SourceScriptTimeout(Exception):
    pass


class SourceScriptRejected(Exception):
    pass


class SourceScriptStateLost(Exception):
    pass


BROWSER_OPERATION_PHASES = frozenset({
    "initialNavigation", "snapshotEventPump", "snapshotLoadState",
    "snapshotContent", "snapshotStabilityPump", "sourceScriptStart", "sourceScriptRead",
})
SOURCE_STATE_FAILURE_KINDS = frozenset({
    "sourceStateMissing", "sourceStateTypeInvalid", "sourceStateBodyInvalid", "sourceStateStatusInvalid",
})


@contextlib.contextmanager
def browser_operation(phase):
    """Annotate the original exception, without swallowing, retrying or replaying.

    Only a fixed operation label survives. The existing NDJSON error class and
    public ReturnData stay unchanged; diagnostic text never includes page data.
    """
    if not isinstance(phase, str) or phase not in BROWSER_OPERATION_PHASES:
        raise ValueError("Unknown browser operation")
    try:
        yield
    except Exception as error:
        try:
            if getattr(error, "_reader_browser_operation", None) is None:
                error._reader_browser_operation = phase
        except Exception:
            # Diagnostics must not replace a fatal exception lacking writable attributes.
            pass
        raise


def browser_failure_diagnostic(error):
    """Return finite diagnostic labels, never an exception's untrusted message.

    Message matching is a diagnostic hint, not authority to retry a request or
    relax network policy. Unknown failures remain fatal and explicitly unknown.
    """
    phase = getattr(error, "_reader_browser_operation", None)
    if not isinstance(phase, str) or phase not in BROWSER_OPERATION_PHASES:
        return None
    name = type(error).__name__
    kind = "unclassified"
    if isinstance(error, SourceScriptStateLost):
        reason = getattr(error, "_reader_source_state_kind", None)
        observed = getattr(error, "_reader_main_frame_navigation_observed", None)
        if (phase == "sourceScriptRead" and isinstance(reason, str) and
                reason in SOURCE_STATE_FAILURE_KINDS and type(observed) is bool):
            return {"operation": phase, "kind": reason, "errorClass": "SourceScriptStateLost",
                    "mainFrameNavigationObserved": observed}
    if name == "Error":
        try:
            message = str(error)
        except Exception:
            message = ""
        if "NS_BINDING_ABORTED" in message or "is interrupted by another navigation" in message:
            kind = "navigationInterrupted"
        elif "Execution context was destroyed" in message:
            kind = "executionContextDestroyed"
        elif "Unable to retrieve content because the page is navigating" in message:
            kind = "documentChanging"
    return {"operation": phase, "kind": kind,
            "errorClass": name if name in {"Error", "TimeoutError", "TargetClosedError"} else "Other"}


# Evaluate a rule once and return immediately, even when it yields a Promise.
# Python owns the monotonic budget: a page timer or Playwright's evaluate call
# cannot bound a Promise that never settles. Only serialized results cross IPC;
# rejected values/messages may contain credentials and must remain in the page.
SOURCE_SCRIPT_START = """({source, key}) => {
    const state = {status: 'pending'};
    Object.defineProperty(globalThis, key, {value: state, configurable: true});
    const stringify = JSON.stringify;
    const resolve = Promise.resolve.bind(Promise);
    const reject = () => { state.status = 'rejected'; };
    const finish = (value) => {
        try {
            const body = value == null ? '' :
                typeof value === 'string' ? value : stringify(value);
            state.body = body == null ? '' : body;
            state.status = 'done';
        } catch (_) { reject(); }
    };
    try {
        const value = (0, eval)(source);
        // Native adoption reads a thenable's getter only once and follows
        // nested Promises. Do not inspect value.then before Promise.resolve.
        resolve(value).then(finish, reject);
    } catch (_) { reject(); }
} """

SOURCE_SCRIPT_READ = """(key) => {
    const state = globalThis[key];
    if (!state) return null;
    return state.status === 'done' ? {status: 'done', body: state.body} :
        {status: state.status};
} """

SOURCE_SCRIPT_CLEAR = """(key) => { delete globalThis[key]; } """


def evaluate_source_script(page, source, timeout_ms, check_allowed, monotonic=time.monotonic):
    """Wait for one rule result without replaying the rule, goto, or original POST.

    Navigation/transport failures stay fatal. A replacement document loses the
    random transient state and must not silently rerun a potentially mutating
    rule. Context teardown remains the final cleanup authority. Synchronous JS
    that blocks Firefox's event loop is still bounded by the parent watchdog.
    """
    if timeout_ms <= 0:
        raise ValueError("timeoutMs must be positive")
    deadline = monotonic() + timeout_ms / 1000.0
    key = "__reader_source_" + secrets.token_hex(16)
    navigation_observed = False

    def frame_navigated(frame):
        nonlocal navigation_observed
        if frame == page.main_frame:
            navigation_observed = True

    def state_lost(kind):
        error = SourceScriptStateLost()
        error._reader_browser_operation = "sourceScriptRead"
        error._reader_source_state_kind = kind
        # An event is not proof of a replacement document (same-document
        # navigation can also emit it). Never claim a unique cause from this bit.
        error._reader_main_frame_navigation_observed = navigation_observed
        return error

    def remaining_ms():
        remaining = (deadline - monotonic()) * 1000.0
        if remaining <= 0:
            raise SourceScriptTimeout()
        return remaining

    try:
        page.on("framenavigated", frame_navigated)
        check_allowed()
        with browser_operation("sourceScriptStart"):
            page.evaluate(SOURCE_SCRIPT_START, {"source": source, "key": key})
        while True:
            check_allowed()
            remaining_ms()
            with browser_operation("sourceScriptRead"):
                result = page.evaluate(SOURCE_SCRIPT_READ, key)
            check_allowed()
            remaining_ms()
            if result is None:
                raise state_lost("sourceStateMissing")
            if not isinstance(result, dict):
                raise state_lost("sourceStateTypeInvalid")
            status = result.get("status")
            if status == "done" and set(result) == {"status", "body"}:
                if not isinstance(result["body"], str):
                    raise state_lost("sourceStateBodyInvalid")
                require_utf8_limit(result["body"], MAX_BODY_UTF8_BYTES, ResponseBodyTooLarge)
                return result["body"]
            if status == "rejected" and set(result) == {"status"}:
                raise SourceScriptRejected()
            if status != "pending" or set(result) != {"status"}:
                raise state_lost("sourceStateStatusInvalid")
            # Pump actual browser events; sleeping Python would hide completion.
            page.wait_for_timeout(min(50, remaining_ms()))
    finally:
        try:
            page.remove_listener("framenavigated", frame_navigated)
        except Exception:
            pass
        try:
            page.evaluate(SOURCE_SCRIPT_CLEAR, key)
        except Exception:
            # Preserve the original failure. render() always closes this fresh
            # context; a closed or replaced document need not accept cleanup JS.
            pass


class MainDocumentSnapshot:
    """Observe finite main-frame navigation without replaying requests or scripts.

    DOMContentLoaded belongs to one document, not to a chain of client redirects.
    A bounded quiet window covers pending main-document requests and commits;
    subresources/SSE never reset it. It is not an arbitrary delayed-script oracle.
    """

    QUIET_SECONDS = 0.2

    def __init__(self, page, monotonic=time.monotonic):
        self.page = page
        self.monotonic = monotonic
        self.pending = set()
        self.generation = 0
        self.last_change = monotonic()
        page.on("request", self.request_started)
        page.on("requestfinished", self.request_finished)
        page.on("requestfailed", self.request_finished)
        page.on("framenavigated", self.frame_navigated)

    def changed(self):
        self.generation += 1
        self.last_change = self.monotonic()

    def request_started(self, request):
        if request.is_navigation_request() and request.frame == self.page.main_frame:
            self.pending.add(request)
            self.changed()

    def request_finished(self, request):
        if request in self.pending:
            self.pending.remove(request)
            self.changed()

    def frame_navigated(self, frame):
        if frame == self.page.main_frame:
            self.changed()

    def read(self, timeout_ms, check_allowed):
        if timeout_ms <= 0:
            raise ValueError("timeoutMs must be positive")
        # One snapshot budget, not a fresh timeout for each document in a chain.
        deadline = self.monotonic() + timeout_ms / 1000.0

        def remaining_ms():
            remaining = (deadline - self.monotonic()) * 1000.0
            if remaining <= 0:
                raise TimeoutError("Main document did not settle within snapshot budget")
            return remaining

        while True:
            check_allowed()
            # Pump the sync driver's event loop; time.sleep would hide commits.
            with browser_operation("snapshotEventPump"):
                self.page.wait_for_timeout(min(50, remaining_ms()))
            check_allowed()
            remaining_ms()
            if self.pending or self.monotonic() - self.last_change < self.QUIET_SECONDS:
                continue
            generation = self.generation
            with browser_operation("snapshotLoadState"):
                self.page.wait_for_load_state("domcontentloaded", timeout=remaining_ms())
            check_allowed()
            remaining_ms()
            if self.pending or self.generation != generation:
                continue
            # Unknown browser, closed-page and transport exceptions stay fatal;
            # never treat them as transient or replay the original POST/goto.
            with browser_operation("snapshotContent"):
                body = self.page.content()
            require_utf8_limit(body, MAX_BODY_UTF8_BYTES, ResponseBodyTooLarge)
            with browser_operation("snapshotStabilityPump"):
                self.page.wait_for_timeout(min(50, remaining_ms()))
            check_allowed()
            remaining_ms()
            if not self.pending and self.generation == generation:
                return body


def utf8_length_at_most(value, maximum):
    """Count UTF-8 bytes without allocating a second full copy of an untrusted body."""
    if not isinstance(value, str):
        value = str(value)
    total = 0
    for start in range(0, len(value), 4096):
        total += len(value[start:start + 4096].encode("utf-8"))
        if total > maximum:
            return False
    return True


def require_utf8_limit(value, maximum, error_type):
    if not utf8_length_at_most(value, maximum):
        raise error_type()


def bounded_cookie(cookie):
    """Validate a portable cookie record before it enters either browser or stdout state."""
    if not isinstance(cookie, dict):
        raise CookieLimitExceeded()
    for key, maximum in (
        ("name", MAX_COOKIE_NAME_UTF8_BYTES),
        ("value", MAX_COOKIE_VALUE_UTF8_BYTES),
        ("domain", MAX_COOKIE_DOMAIN_UTF8_BYTES),
        ("path", MAX_COOKIE_PATH_UTF8_BYTES),
        ("sameSite", MAX_COOKIE_SAMESITE_UTF8_BYTES),
    ):
        value = cookie.get(key)
        if value is not None:
            require_utf8_limit(value, maximum, CookieLimitExceeded)
    return cookie


def normalized_domain(value):
    """Return a safe cookie domain without the optional leading dot."""
    value = (value or "").strip().lstrip(".").rstrip(".").lower()
    if not value or any(character.isspace() or character in "/;" for character in value):
        return None
    return value


def default_cookie_path(response_path):
    if not response_path or not response_path.startswith("/") or response_path.count("/") <= 1:
        return "/"
    return response_path.rsplit("/", 1)[0] or "/"


def domain_matches(host, domain):
    return host == domain or host.endswith("." + domain)


def same_origin(left, right):
    return (
        left.scheme.lower() == right.scheme.lower()
        and (left.hostname or "").lower() == (right.hostname or "").lower()
        and left.port == right.port
    )


def split_set_cookie_fields(raw_header):
    """Split attributes without treating a semicolon inside quotes as a boundary."""
    fields = []
    start = 0
    quoted = False
    escaped = False
    for index, character in enumerate(raw_header):
        if escaped:
            escaped = False
        elif character == "\\" and quoted:
            escaped = True
        elif character == '"':
            quoted = not quoted
        elif character == ";" and not quoted:
            fields.append(raw_header[start:index])
            start = index + 1
    if quoted:
        return None
    fields.append(raw_header[start:])
    return fields


def parse_set_cookie(raw_header, response_url):
    """Parse one response Set-Cookie into the portable Reader cookie record.

    The worker must retain attributes here because parent-side persistence is
    intentionally independent of the transient Firefox profile. This is a
    deliberately small RFC 6265 parser: unknown attributes are ignored, while
    invalid Domain values fail closed instead of becoming a host-only cookie.
    """
    # Header parsing is a fallback for a known Camoufox snapshot race. It must
    # not become an unbounded alternate cookie parser.
    if not utf8_length_at_most(raw_header, MAX_COOKIE_VALUE_UTF8_BYTES + MAX_COOKIE_PATH_UTF8_BYTES + 4096):
        return None
    pieces = split_set_cookie_fields(raw_header)
    if pieces is None:
        return None
    pair = pieces[0].strip() if pieces else ""
    name, separator, value = pair.partition("=")
    name = name.strip()
    if not separator or not name or any(character.isspace() or character in ";=" for character in name):
        return None
    origin = urlsplit(response_url)
    host = normalized_domain(origin.hostname)
    if not host:
        return None
    attributes = {}
    flags = set()
    for piece in pieces[1:]:
        key, has_value, attribute_value = piece.strip().partition("=")
        key = key.strip().lower()
        if not key:
            continue
        if has_value:
            attributes[key] = attribute_value.strip()
        else:
            flags.add(key)

    requested_domain = attributes.get("domain")
    host_only = not bool(requested_domain)
    domain = host if host_only else normalized_domain(requested_domain)
    if not domain or (not host_only and not domain_matches(host, domain)):
        return None
    path = attributes.get("path") or default_cookie_path(origin.path)
    if not path.startswith("/"):
        path = default_cookie_path(origin.path)

    expires = -1.0
    deleted = False
    max_age = attributes.get("max-age")
    max_age_seconds = None
    if max_age is not None:
        try:
            max_age_seconds = int(max_age)
            if max_age_seconds <= 0:
                deleted = True
            else:
                expires = time.time() + max_age_seconds
        except ValueError:
            # RFC 6265: an invalid Max-Age is ignored, so a valid Expires still
            # controls the record rather than accidentally creating a session.
            max_age_seconds = None
    if max_age_seconds is None and attributes.get("expires"):
        try:
            expires = parsedate_to_datetime(attributes["expires"]).timestamp()
            deleted = expires <= time.time()
        except (TypeError, ValueError, OverflowError):
            pass
    same_site = attributes.get("samesite", "").lower()
    same_site = {"lax": "Lax", "strict": "Strict", "none": "None"}.get(same_site)
    secure = "secure" in flags
    # A Secure attribute on an HTTP response is ignored by browsers. The raw
    # fallback must not persist a cookie the real browser would reject.
    if secure and origin.scheme.lower() != "https":
        return None
    if name.startswith("__Secure-") and (not secure or origin.scheme.lower() != "https"):
        return None
    if name.startswith("__Host-") and (
        not secure or origin.scheme.lower() != "https" or not host_only or path != "/"
    ):
        return None
    return bounded_cookie({
        "name": name,
        "value": value.strip(),
        "domain": domain,
        "path": path,
        "hostOnly": host_only,
        "secure": secure,
        "httpOnly": "httponly" in flags,
        "sameSite": same_site,
        "expires": expires,
        # `flag=` is a valid empty-valued cookie. Deletion is an explicit expiry
        # operation, never an inference from the value.
        "deleted": deleted,
    })


def cookie_identity(cookie):
    return "\0".join((cookie["name"], cookie["domain"], cookie["path"]))


def response_cookie_needs_fallback(cookie):
    """Never recreate a script-deleted non-HttpOnly cookie from response headers."""
    return cookie["deleted"] or (cookie["hostOnly"] and cookie["httpOnly"])


def request_matches_cookie(cookie, request_url):
    request = urlsplit(request_url)
    host = normalized_domain(request.hostname)
    if not host or cookie.get("deleted"):
        return False
    expires = cookie.get("expires", -1)
    if expires is not None and expires >= 0 and expires <= time.time():
        return False
    if cookie.get("secure") and request.scheme.lower() != "https":
        return False
    if cookie.get("hostOnly", True):
        if cookie.get("domain") != host:
            return False
    elif not domain_matches(host, cookie.get("domain", "")):
        return False
    path = request.path or "/"
    cookie_path = cookie.get("path") or "/"
    return path == cookie_path or (
        path.startswith(cookie_path)
        and (cookie_path.endswith("/") or path[len(cookie_path):].startswith("/"))
    )


def as_playwright_cookie(cookie, request_url):
    """Convert a Reader record without losing host-only semantics on import."""
    path = cookie["path"] or "/"
    result = {
        "name": cookie["name"],
        "value": cookie["value"],
        "secure": bool(cookie.get("secure")),
        "httpOnly": bool(cookie.get("httpOnly")),
    }
    if cookie.get("expires", -1) >= 0:
        result["expires"] = cookie["expires"]
    if cookie.get("sameSite"):
        result["sameSite"] = cookie["sameSite"]
    if cookie.get("hostOnly", True):
        # Playwright's URL form derives Path from the URL's *parent directory*.
        # A persisted Path=/account would silently become / if supplied as a URL.
        # A domain without the leading dot denotes a host-only cookie and keeps
        # its exact Path, per the Playwright add_cookies contract.
        result["domain"] = cookie["domain"]
        result["path"] = path
    else:
        result["domain"] = "." + cookie["domain"]
        result["path"] = path
    return result


def portable_snapshot_cookie(item, document_origin, response_cookies, request_cookie_metadata):
    """Turn one Playwright snapshot entry into the Reader record without widening scope."""
    raw_domain = item.get("domain") or ""
    domain = normalized_domain(raw_domain)
    document_host = normalized_domain(document_origin.hostname)
    if not domain or not document_host or not domain_matches(document_host, domain):
        return None
    identity = "\0".join((item.get("name", ""), domain, item.get("path") or "/"))
    remembered = response_cookies.get(identity) or request_cookie_metadata.get(identity)
    return bounded_cookie({
        "name": item.get("name", ""),
        "value": item.get("value", ""),
        "domain": domain,
        "path": item.get("path") or "/",
        # response metadata; Playwright encodes Domain cookies with a leading
        # dot, which is the only reliable fallback signal when no response
        # header metadata was observed.
        "hostOnly": remembered.get("hostOnly", True) if remembered else not raw_domain.startswith("."),
        "secure": bool(item.get("secure")),
        "httpOnly": bool(item.get("httpOnly")),
        "sameSite": item.get("sameSite") or None,
        "expires": item.get("expires", -1),
        "deleted": False,
    })


def cookie_changed_from_initial(cookie, initial_cookies, response_seen):
    """Only return browser state that can safely change Reader's persistent jar."""
    identity = cookie_identity(cookie)
    if identity in response_seen:
        # A response explicitly touched this identity. The browser snapshot is the
        # authority for Domain-cookie acceptance; returning it preserves a genuine
        # same-value refresh without re-persisting unrelated imported cookies.
        return True
    return initial_cookies.get(identity) != cookie


def missing_initial_cookie_tombstones(initial_visible, final_visible, response_fallbacks):
    """Propagate JavaScript/expiry deletions without guessing about unseen cookies."""
    return {
        identity: bounded_cookie(dict(cookie, value="", expires=0, deleted=True))
        for identity, cookie in initial_visible.items()
        if identity not in final_visible and identity not in response_fallbacks
    }


def merge_response_cookie_fallbacks(cookie_values, final_visible, response_cookies):
    """Let the browser snapshot win when it saw a live response cookie."""
    for identity, fallback in response_cookies.items():
        if fallback["deleted"] or identity not in final_visible:
            cookie_values[identity] = fallback
    return cookie_values


def render(payload):
    url = payload["url"]
    timeout_ms = int(payload["timeoutMs"])
    source_pattern = re.compile(payload["sourceRegex"]) if payload.get("sourceRegex") else None
    state = {"matched_url": None, "post_sent": False, "blocked": False}
    document_origin = urlsplit(url)
    response_cookies = {}
    response_seen = set()
    proxy = {
        "server": payload["proxy"],
        # Playwright's special value removes Firefox's implicit loopback bypass.
        "bypass": "<-loopback>",
    }

    with contextlib.redirect_stdout(sys.stderr):
        with Camoufox(
            headless=True,
            browser=payload.get("browserVersion") or None,
            proxy=proxy,
            # Do not contact a GeoIP provider outside Reader's audited egress path.
            # The application proxy may be a local SSRF gate rather than a geographic exit.
            geoip=False,
            block_webrtc=True,
            exclude_addons=[DefaultAddons.UBO],
            # Network isolation is intentional: WebRTC is disabled and every HTTP(S)
            # request, including redirects, is sent to Reader's validated local proxy.
            i_know_what_im_doing=True,
            firefox_user_prefs={
                "network.dns.disablePrefetch": True,
                "network.prefetch-next": False,
                "network.predictor.enabled": False,
                "network.http.speculative-parallel-limit": 0,
            },
        ) as browser:
            context_options = {
                "accept_downloads": False,
                "service_workers": "block",
            }
            if payload.get("userAgent"):
                context_options["user_agent"] = payload["userAgent"]
            context = NewContext(browser, **context_options)
            try:
                context.set_default_timeout(timeout_ms)
                context.set_default_navigation_timeout(timeout_ms)
                if payload.get("headers"):
                    context.set_extra_http_headers(payload["headers"])
                cookies = payload.get("cookies") or []
                if len(cookies) > MAX_COOKIES:
                    raise CookieLimitExceeded()
                cookies = [bounded_cookie(cookie) for cookie in cookies]
                request_cookie_metadata = {
                    cookie_identity(cookie): cookie
                    for cookie in cookies
                    if request_matches_cookie(cookie, url)
                }
                # This is a complete pre-render baseline, including cookies derived
                # from an explicit Cookie request header. Such headers are a
                # one-request override and must never become durable merely because
                # Firefox reports the imported cookie in its final snapshot.
                initial_cookies = dict(request_cookie_metadata)
                if cookies:
                    context.add_cookies([
                        as_playwright_cookie(cookie, url)
                        for cookie in cookies
                        if request_matches_cookie(cookie, url)
                    ])
                # Only a cookie that Firefox actually accepted at import time
                # may later be declared deleted when it disappears. A raw input
                # record absent from both snapshots is not proof of deletion.
                initial_visible = {}
                for item in context.cookies():
                    imported = portable_snapshot_cookie(
                        item, document_origin, {}, request_cookie_metadata
                    )
                    if imported:
                        initial_visible[cookie_identity(imported)] = imported

                def remember_response_cookies(response):
                    """Keep a structured same-origin fallback for Camoufox snapshots."""
                    response_origin = urlsplit(response.url)
                    if not same_origin(response_origin, document_origin):
                        return
                    try:
                        for set_cookie in response.header_values("set-cookie"):
                            parsed = parse_set_cookie(set_cookie, response.url)
                            if parsed:
                                response_seen.add(cookie_identity(parsed))
                            # A browser snapshot is authoritative for non-HttpOnly
                            # creations: page scripts may have removed a cookie
                            # before navigation completes. Only host-only HttpOnly
                            # creations need the Camoufox snapshot-race fallback.
                            # Explicit deletion headers must always cross the race,
                            # including Domain-scoped deletions.
                            if parsed and response_cookie_needs_fallback(parsed):
                                if len(response_cookies) >= MAX_COOKIES and cookie_identity(parsed) not in response_cookies:
                                    raise CookieLimitExceeded()
                                response_cookies[cookie_identity(parsed)] = parsed
                    except Exception:
                        # Header access is advisory. The browser cookie snapshot remains
                        # the primary source and a response must not fail because a server
                        # returned an unusual header representation.
                        return

                context.on("response", remember_response_cookies)

                def route_request(route):
                    route_url = route.request.url
                    scheme = urlsplit(route_url).scheme.lower()
                    if scheme in ("about", "blob", "data"):
                        route.continue_()
                        return
                    if scheme not in ("http", "https", "ws", "wss"):
                        state["blocked"] = True
                        route.abort()
                        return
                    if source_pattern and source_pattern.fullmatch(route_url):
                        state["matched_url"] = route_url
                        route.abort()
                        return
                    if (payload.get("post") and route.request.is_navigation_request()
                            and route_url == url and not state["post_sent"]):
                        state["post_sent"] = True
                        route.continue_(method="POST", post_data=payload.get("body") or "")
                        return
                    route.continue_()

                context.route("**/*", route_request)
                page = context.new_page()
                snapshot = (MainDocumentSnapshot(page) if not source_pattern
                            and not payload.get("javaScript") else None)
                try:
                    with browser_operation("initialNavigation"):
                        page.goto(url, wait_until="domcontentloaded", timeout=timeout_ms)
                except Exception:
                    if state["matched_url"] is None:
                        raise

                html = payload.get("html")
                if html is not None and state["matched_url"] is None:
                    page.set_content(html, wait_until="domcontentloaded", timeout=timeout_ms)

                source = payload.get("javaScript")

                def check_allowed():
                    if state["blocked"]:
                        raise BlockedScheme()

                # A source rule may delete a non-HttpOnly cookie created during
                # navigation. Capture browser-accepted state before the rule runs:
                # the pre-navigation snapshot cannot observe that cookie.
                before_source_script = {}
                if source and state["matched_url"] is None:
                    for item in context.cookies():
                        visible = portable_snapshot_cookie(
                            item, document_origin, response_cookies, request_cookie_metadata
                        )
                        if visible and not visible["httpOnly"]:
                            before_source_script[cookie_identity(visible)] = visible
                if source_pattern:
                    if state["matched_url"] is None and source:
                        try:
                            page.evaluate("(source) => { (0, eval)(source); }", source)
                        except Exception:
                            if state["matched_url"] is None:
                                raise
                    deadline = time.monotonic() + timeout_ms / 1000.0
                    while state["matched_url"] is None and time.monotonic() < deadline:
                        page.wait_for_timeout(min(50, max(1, int((deadline - time.monotonic()) * 1000))))
                    body = state["matched_url"]
                    if body is None:
                        raise TimeoutError("sourceRegex resource was not observed before timeout")
                elif source:
                    # The archived /render.html reference returns strings as-is
                    # and JSON-serializes non-string results. Serialize after a
                    # returned Promise settles, not the Promise object itself.
                    body = evaluate_source_script(page, source, timeout_ms, check_allowed)
                else:
                    body = snapshot.read(timeout_ms, check_allowed)

                require_utf8_limit(body, MAX_BODY_UTF8_BYTES, ResponseBodyTooLarge)

                # Firefox normally exposes a complete snapshot. For a Linux
                # Camoufox release which omits a just-received HttpOnly cookie,
                # structured Set-Cookie records can fill a missing snapshot.
                # A visible browser cookie wins over our small header parser;
                # explicit response deletions still override stale snapshots.
                cookie_values = {}
                final_visible = set()
                for item in context.cookies():
                    cookie = portable_snapshot_cookie(
                        item, document_origin, response_cookies, request_cookie_metadata
                    )
                    if not cookie:
                        continue
                    final_visible.add(cookie_identity(cookie))
                    if not cookie_changed_from_initial(cookie, initial_cookies, response_seen):
                        continue
                    if len(cookie_values) >= MAX_COOKIES and cookie_identity(cookie) not in cookie_values:
                        raise CookieLimitExceeded()
                    cookie_values[cookie_identity(cookie)] = cookie
                cookie_values.update(missing_initial_cookie_tombstones(
                    initial_visible, final_visible, response_cookies
                ))
                merge_response_cookie_fallbacks(
                    cookie_values, final_visible, response_cookies
                )
                # An explicit source script's observed deletion wins over the
                # same-response Set-Cookie fallback. HttpOnly records are excluded
                # above because document.cookie cannot delete them.
                cookie_values.update(missing_initial_cookie_tombstones(
                    before_source_script, final_visible, set()
                ))
                if len(cookie_values) > MAX_COOKIES:
                    raise CookieLimitExceeded()
                if state["blocked"]:
                    return {"error": "BlockedScheme"}
                return {
                    "body": body,
                    "cookies": list(cookie_values.values()),
                }
            finally:
                context.close()


def main():
    for raw in sys.stdin:
        if not raw.strip():
            continue
        try:
            response = render(json.loads(raw))
        except Exception as error:
            # Return only the exception class; URLs, headers, and proxy credentials
            # can be present in Playwright exception text and must not be logged here.
            response = {"error": type(error).__name__}
            diagnostic = browser_failure_diagnostic(error)
            if diagnostic is not None:
                print("READER_BROWSER_FAILURE " + json.dumps(diagnostic, separators=(",", ":")), file=sys.stderr)
        encoded = json.dumps(response, ensure_ascii=False, separators=(",", ":"))
        if not utf8_length_at_most(encoded, MAX_PROTOCOL_UTF8_BYTES - 1):
            # Do not emit a partial JSON record or a huge response in diagnostics.
            encoded = '{"error":"ResponseTooLarge"}'
        protocol_out.write(encoded + "\n")
        protocol_out.flush()


if __name__ == "__main__":
    main()
