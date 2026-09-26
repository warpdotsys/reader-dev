"""One-request NDJSON worker for the Reader-managed Camoufox process."""

import contextlib
import json
import re
import sys
import time
from urllib.parse import urlsplit

# Stdout is a machine protocol. Keep all package/browser diagnostics on stderr.
protocol_out = sys.stdout
sys.stdout = sys.stderr
with contextlib.redirect_stdout(sys.stderr):
    from camoufox import Camoufox, DefaultAddons, NewContext


def render(payload):
    url = payload["url"]
    timeout_ms = int(payload["timeoutMs"])
    source_pattern = re.compile(payload["sourceRegex"]) if payload.get("sourceRegex") else None
    state = {"matched_url": None, "post_sent": False, "blocked": False}
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
                cookies = payload.get("cookies") or {}
                if cookies:
                    context.add_cookies([
                        {"name": name, "value": value, "url": url}
                        for name, value in cookies.items()
                    ])

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
                try:
                    page.goto(url, wait_until="domcontentloaded", timeout=timeout_ms)
                except Exception:
                    if state["matched_url"] is None:
                        raise

                html = payload.get("html")
                if html is not None and state["matched_url"] is None:
                    page.set_content(html, wait_until="domcontentloaded", timeout=timeout_ms)

                source = payload.get("javaScript")
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
                    value = page.evaluate(
                        "(source) => { const value = (0, eval)(source); "
                        "return value == null ? '' : value.toString(); }",
                        source,
                    )
                    body = value or ""
                else:
                    body = page.content()

                cookie_values = [
                    {"name": item["name"], "value": item["value"]}
                    for item in context.cookies(url)
                ]
                if state["blocked"]:
                    return {"error": "BlockedScheme"}
                return {"body": body, "cookies": cookie_values}
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
        protocol_out.write(json.dumps(response, ensure_ascii=False, separators=(",", ":")) + "\n")
        protocol_out.flush()


if __name__ == "__main__":
    main()
