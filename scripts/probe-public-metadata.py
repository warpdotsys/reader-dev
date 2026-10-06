"""One anonymous public metadata request inside the exact tested Reader image.

No real account, external Cookie, chapter endpoint, raw HTML or error is accepted
or emitted. A success envelope without the expected metadata is a failed probe.
"""

import argparse
from datetime import datetime, timezone
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import threading
import time
import urllib.error
import urllib.parse
import urllib.request


SOURCE = "https://www.qidian.com"
BOOK = SOURCE + "/book/1010400217/"
BASE = "http://127.0.0.1:18893"
MAX_RESPONSE_BYTES = 64 * 1024
FIELDS = ("bookUrl", "tocUrl", "name", "author", "coverUrl", "type", "group",
          "totalChapterNum", "durChapterIndex", "durChapterPos", "canUpdate")
METADATA_DOM_WAIT_MS = 8000
PAGE_DIAGNOSTIC_KEYS = frozenset((
    "nameSelectorPresent", "authorSelectorPresent", "coverSelectorPresent",
    "bodyHasText", "bodyHasExpectedBookTitle", "bodyHasExpectedAuthor",
    "bodyHasSafetyPhrase", "captchaContainerPresent"))
PUBLIC_SOURCE_TEMPLATE = """{
  "bookSourceUrl": "https://www.qidian.com",
  "bookSourceName": "Anonymous metadata-only probe",
  "enabledCookieJar": false,
  "ruleToc": {},
  "ruleBookInfo": {
    "name": "#bookName@text",
    "author": ".book-info-top .book-meta .author@text",
    "coverUrl": "#bookImg img@src"
  }
}"""
# Parse only the already-returned snapshot. This Reader rule does not execute
# page scripts, make requests, inspect Cookie or change the metadata selectors.
# Structural hints are not a visibility/CAPTCHA/authentication verdict.
PAGE_DIAGNOSTIC_RULE = """@js:(function () {
  var doc = Packages.org.jsoup.Jsoup.parse(String(result));
  var text = String(doc.body().text());
  return JSON.stringify({
    nameSelectorPresent: doc.select('#bookName').size() > 0,
    authorSelectorPresent: doc.select('.book-info-top .book-meta .author').size() > 0,
    coverSelectorPresent: doc.select('#bookImg img').size() > 0,
    bodyHasText: text.trim().length > 0,
    bodyHasExpectedBookTitle: text.indexOf('黎明之剑') >= 0,
    bodyHasExpectedAuthor: text.indexOf('远瞳') >= 0,
    bodyHasSafetyPhrase: /拖动滑块|完成拼图|安全验证|访问验证|请完成验证/.test(text),
    captchaContainerPresent: doc.select('.geetest_panel,#nc_1_wrapper,#tcaptcha_transform_dy').size() > 0
  });
})()"""
# Return only the page's own HTML. No inserted metadata, request replay, Cookie
# access, external fetch, navigation, chapter operation or CAPTCHA interaction.
METADATA_DOM_SCRIPT = """new Promise(resolve => {
  const started = performance.now();
  const read = () => {
    const name = document.querySelector('#bookName');
    const author = document.querySelector('.book-info-top .book-meta .author');
    const cover = document.querySelector('#bookImg img');
    if ((name && name.textContent.trim() && author && author.textContent.trim()
         && cover && cover.getAttribute('src')) || performance.now() - started >= 8000) {
      resolve(document.documentElement.outerHTML);
      return;
    }
    setTimeout(read, 100);
  };
  read();
})"""


class ProbeFailure(Exception):
    """Only a fixed category, never a server message or credential-bearing URL."""


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ProbeFailure("UnexpectedReaderRedirect")


def load_helper(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require_environment(base, revision, output):
    if base != BASE or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ProbeFailure("InvalidProbeInput")
    if (not hasattr(os, "getuid") or os.getuid() != 10001 or
            os.environ.get("READER_BUILD_REVISION") != revision or
            os.environ.get("READER_APP_WEBVIEWRENDERER") != "camoufox" or
            os.environ.get("READER_BROWSER_ALLOW_PRIVATE_NETWORKS") != "false" or
            os.environ.get("READER_SERVER_BINDADDRESS") != "127.0.0.1" or
            os.environ.get("READER_APP_SECURE") != "true"):
        raise ProbeFailure("UnexpectedRuntimeIdentityOrGuard")
    if output.resolve() != Path("/verification-output") or not output.is_dir():
        raise ProbeFailure("UnexpectedReportDirectory")
    if any((output / name).exists() for name in
           ("PUBLIC_METADATA_REPORT.json", "RUNTIME_RESOURCE_BUDGET.json")):
        raise ProbeFailure("ExistingReport")


def read_response(response):
    raw = response.read(MAX_RESPONSE_BYTES + 1)
    if len(raw) > MAX_RESPONSE_BYTES:
        raise ProbeFailure("ReaderResponseTooLarge")
    try:
        value = json.loads(raw.decode("utf-8"))
    except (ValueError, UnicodeError):
        raise ProbeFailure("MalformedReaderJson") from None
    if not isinstance(value, dict):
        raise ProbeFailure("UnexpectedReaderJsonType")
    return int(response.status), value


def request_json(opener, path, body=None):
    # The only source operation is one getBookInfo. No chapter/source-login API.
    if path not in ("/getSystemInfo", "/login", "/saveBookSource", "/getBookSource", "/getBookInfo",
                    "/getBookSourceCookie", "/setBookSourceCookie", "/logout"):
        raise ProbeFailure("UnexpectedReaderEndpoint")
    data = json.dumps(body, ensure_ascii=False).encode("utf-8") if body is not None else None
    request = urllib.request.Request(
        BASE + "/reader3" + path, data=data,
        headers={"Content-Type": "application/json; charset=utf-8"} if data else {},
        method="POST" if body is not None else "GET")
    try:
        with opener.open(request, timeout=35) as response:
            return read_response(response)
    except urllib.error.HTTPError as error:
        with error:
            return read_response(error)


def require_success(result):
    status, value = result
    if status != 200 or value.get("isSuccess") is not True:
        raise ProbeFailure("ReaderSetupOrCleanupRejected")
    return value


def page_diagnostics(raw):
    if not isinstance(raw, str):
        return None
    try:
        if len(raw.encode("utf-8")) > 1024:
            return None
    except UnicodeError:
        return None

    def unique_fields(pairs):
        fields = {}
        for key, value in pairs:
            if key in fields:
                raise ValueError("Duplicate diagnostic key")
            fields[key] = value
        return fields

    try:
        fields = json.loads(raw, object_pairs_hook=unique_fields)
    except (ValueError, TypeError):
        return None
    if (not isinstance(fields, dict) or set(fields) != PAGE_DIAGNOSTIC_KEYS or
            any(type(value) is not bool for value in fields.values())):
        return None
    return fields


def summarize(status, value):
    success = value.get("isSuccess")
    error = value.get("errorMsg")
    data = value.get("data")
    text = error if isinstance(error, str) else ""
    reason = re.search(r"reason=(DNS_FAILURE|ADDRESS_NOT_PUBLIC|INVALID_HOST|INVALID_URL|UNSUPPORTED_SCHEME)\b", text)
    fingerprint = re.search(r"hostSha256=([0-9a-f]{16})\b", text)
    # Only the exact readable JVM wrapper around a fixed worker exception is
    # classified. Never preserve arbitrary exception names, messages or URLs.
    worker_error = re.fullmatch(
        r"(?:java\.lang\.IllegalStateException: )?Camoufox 渲染失败 "
        r"\((SourceScriptStateLost|SourceScriptTimeout|SourceScriptRejected|TimeoutError|"
        r"ResponseBodyTooLarge|ResponseTooLarge|CookieLimitExceeded|Error)\)", text)
    book = data if isinstance(data, dict) else {}
    name = book.get("name")
    author = book.get("author")
    cover = book.get("coverUrl")
    try:
        parsed_cover = urllib.parse.urlsplit(cover) if isinstance(cover, str) else None
        cover_ok = bool(parsed_cover and parsed_cover.scheme == "https" and
                        parsed_cover.hostname == "bookcover.yuewen.com" and
                        not parsed_cover.username and not parsed_cover.password)
    except ValueError:
        cover_ok = False
    name_ok = isinstance(name, str) and name.strip() == "黎明之剑"
    author_ok = isinstance(author, str) and author.strip() == "远瞳"
    return {
        "httpStatus": status,
        "isSuccess": success if isinstance(success, bool) else None,
        "isSuccessIsBoolean": isinstance(success, bool),
        "errorMsgIsString": isinstance(error, str), "errorMsgEmpty": error == "",
        "errorMessageLength": len(text),
        "policyReason": reason.group(1) if reason else None,
        "hostFingerprint": fingerprint.group(1) if fingerprint else None,
        "workerErrorCategory": worker_error.group(1) if worker_error else None,
        "dataIsNull": data is None, "dataIsObject": isinstance(data, dict),
        "knownFieldsPresent": {key: key in book for key in FIELDS},
        "nameHasText": isinstance(name, str) and bool(name.strip()),
        "authorHasText": isinstance(author, str) and bool(author.strip()),
        "coverHasText": isinstance(cover, str) and bool(cover.strip()),
        "pageDiagnostics": page_diagnostics(book.get("intro")),
        "nameMatches": name_ok, "authorMatches": author_ok,
        "coverHasExpectedPublicOrigin": cover_ok,
        "passed": status == 200 and success is True and error == "" and
                  name_ok and author_ok and cover_ok,
    }


def source_definition():
    # Existing SourceAnalyzer recognizes the nested format only when ruleToc
    # is non-null. This empty format marker does not enable a chapter operation.
    source = json.loads(PUBLIC_SOURCE_TEMPLATE)
    source["ruleBookInfo"]["intro"] = PAGE_DIAGNOSTIC_RULE
    return source


def source_roundtrip(result):
    saved = require_success(result).get("data")
    source = saved if isinstance(saved, dict) else {}
    rules = source.get("ruleBookInfo")
    expected = source_definition()
    observation = {
        "dataIsObject": isinstance(saved, dict),
        "sourceUrlMatches": source.get("bookSourceUrl") == SOURCE,
        "cookieJarDisabled": source.get("enabledCookieJar") is False,
        "modernTocMarkerPresent": isinstance(source.get("ruleToc"), dict),
        "metadataRulesMatch": isinstance(rules, dict) and all(
            rules.get(key) == value for key, value in expected["ruleBookInfo"].items()),
    }
    observation["passed"] = all(observation.values())
    return observation


def book_info_request(wait_dom=False):
    if type(wait_dom) is not bool:
        raise ProbeFailure("InvalidMetadataWaitMode")
    options = {"webView": True}
    if wait_dom:
        options["webJs"] = METADATA_DOM_SCRIPT
    return {"url": BOOK + ", " + json.dumps(options), "bookSourceUrl": SOURCE}


def cookie_count(result):
    data = require_success(result).get("data")
    if not isinstance(data, list):
        raise ProbeFailure("UnexpectedCookieIndexType")
    return len(data)


def verify_cookie_session_logged_out(logout, protected_followup):
    # UserController chains setErrorMsg(...).setData(NEED_LOGIN); setData resets
    # success=true and errorMsg="". The protected followup chains them in the
    # opposite order and must be false. Preserve both actual legacy contracts.
    status, value = logout
    if status != 200 or value.get("isSuccess") is not True or value.get("data") != "NEED_LOGIN" or value.get("errorMsg") != "":
        raise ProbeFailure("UnexpectedLegacyLogoutContract")
    status, value = protected_followup
    if status != 200 or value.get("isSuccess") is not False or value.get("data") != "NEED_LOGIN":
        raise ProbeFailure("GeneratedCookieSessionNotRevoked")


def auth_observation(result):
    status, value = result
    return {"httpStatus": status,
            "isSuccess": value.get("isSuccess") if isinstance(value.get("isSuccess"), bool) else None,
            "dataIsNeedLogin": value.get("data") == "NEED_LOGIN",
            "errorMsgEmpty": value.get("errorMsg") == ""}


def write_new(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, sort_keys=True, ensure_ascii=False, indent=2)
        stream.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reader-base", default=BASE)
    parser.add_argument("--expected-revision", required=True)
    parser.add_argument("--output", type=Path, default=Path("/verification-output"))
    parser.add_argument("--wait-dom", action="store_true",
                        help="Wait at most 8 seconds for fixed public metadata selectors, without altering the page")
    args = parser.parse_args()
    require_environment(args.reader_base, args.expected_revision, args.output)
    cgroup = load_helper("public_metadata_cgroup", "report-browser-cgroup.py")
    process_helper = load_helper("public_metadata_processes", "soak-bundled-browser.py")
    report = {"schemaVersion": 1, "scope": "anonymous single-image public metadata only",
              "expectedImageRevision": args.expected_revision, "source": BOOK,
              "observedAt": datetime.now(timezone.utc).isoformat(),
              "realCredentialsImported": False, "chapterBodyRequested": False,
              "originalJarOrArchivedParity": False, "realAuthenticationProven": False,
              "allDeviceTokenRevocationProven": False,
              "configuredRenderer": "camoufox", "runtimeUid": os.getuid(),
              "privateNetworkGuardEnabled": True, "bookInfoApiCalls": 0,
              "sourceScriptMode": "boundedMetadataDom" if args.wait_dom else "domContentLoadedOnly",
              "pageCaptureMode": "bounded-dom" if args.wait_dom else "snapshot-only",
              "metadataDomWaitBudgetMs": METADATA_DOM_WAIT_MS if args.wait_dom else 0,
              "sourceScriptSynthesizesMetadata": False,
              "pageDiagnosticsScope": "returnedSnapshotStructureNotVisibilityOrAuthentication",
              "pageDiagnosticRuleRequested": True,
              "rawErrorHtmlCookieAndMetadataValuesNotPersisted": True, "passed": False}
    opener = urllib.request.build_opener(
        urllib.request.ProxyHandler({}), NoRedirect(),
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    logged_in = False
    stop = threading.Event()
    maximum = {"worker": 0, "browser": 0, "driver": 0}
    monitor_failures = []

    def monitor():
        try:
            while not stop.is_set():
                found = process_helper.browser_processes()
                for kind in maximum:
                    maximum[kind] = max(maximum[kind], len(found[kind]))
                stop.wait(0.05)
        except Exception:
            monitor_failures.append(True)

    watcher = threading.Thread(target=monitor, daemon=True)
    try:
        cgroup.verify_report(cgroup.collect_report(), require_no_swap=True)
        require_success(request_json(opener, "/getSystemInfo"))
        username, password = "anonmetadata" + secrets.token_hex(5), "Generated-" + secrets.token_hex(18)
        require_success(request_json(opener, "/login", {
            "username": username, "password": password, "isLogin": False}))
        username = password = None
        logged_in = True
        report["cookieRowsBefore"] = cookie_count(request_json(opener, "/getBookSourceCookie"))
        if report["cookieRowsBefore"] != 0:
            raise ProbeFailure("NonemptyFreshCookieIndex")
        require_success(request_json(opener, "/saveBookSource", source_definition()))
        report["sourceDefinitionRoundtrip"] = source_roundtrip(request_json(
            opener, "/getBookSource", {"bookSourceUrl": SOURCE}))
        if not report["sourceDefinitionRoundtrip"]["passed"]:
            raise ProbeFailure("ProbeSourceRulesNotRetained")
        watcher.start()
        started = time.monotonic()
        report["bookInfoApiCalls"] += 1
        result = request_json(opener, "/getBookInfo", book_info_request(args.wait_dom))
        report["metadata"] = summarize(*result)
        report["requestSeconds"] = round(time.monotonic() - started, 3)
        result = None
    except ProbeFailure as failure:
        report["failureCategory"] = str(failure)
    except Exception:
        report["failureCategory"] = "ProbeTransportOrRuntimeFailure"
    except SystemExit:
        report["failureCategory"] = "ResourceGuardFailed"
    finally:
        stop.set()
        if watcher.ident is not None:
            watcher.join(timeout=3)
        report["maximumObservedProcesses"] = maximum
        report["processMonitorPassed"] = not monitor_failures and not watcher.is_alive()
        report["defaultBrowserProcessObserved"] = maximum["worker"] > 0 and maximum["browser"] > 0
        if logged_in:
            try:
                require_success(request_json(opener, "/setBookSourceCookie", {"bookSource": SOURCE, "cookie": ""}))
                report["cookieRowsAfter"] = cookie_count(request_json(opener, "/getBookSourceCookie"))
                logout = request_json(opener, "/logout", {})
                followup = request_json(opener, "/getBookSourceCookie")
                report["logoutObservation"] = auth_observation(logout)
                report["postLogoutProtectedObservation"] = auth_observation(followup)
                verify_cookie_session_logged_out(logout, followup)
                report["generatedCookieSessionRevokedVerified"] = True
            except Exception:
                report["cleanupCategory"] = "GeneratedSessionCleanupFailed"
        try:
            budget = cgroup.collect_report("one anonymous default-Camoufox metadata request")
            write_new(args.output / "RUNTIME_RESOURCE_BUDGET.json", budget)
            cgroup.verify_report(budget, require_no_swap=True)
            report["resourceGuardPassed"] = True
        except (Exception, SystemExit):
            report["resourceGuardPassed"] = False
        report["passed"] = bool(report.get("metadata", {}).get("passed") and
            report.get("sourceDefinitionRoundtrip", {}).get("passed") and
            report.get("metadata", {}).get("pageDiagnostics") is not None and
            report["resourceGuardPassed"] and report["defaultBrowserProcessObserved"] and
            report["processMonitorPassed"] and report.get("cookieRowsAfter") == 0 and
            report.get("generatedCookieSessionRevokedVerified") and not report.get("failureCategory"))
        write_new(args.output / "PUBLIC_METADATA_REPORT.json", report)
    print(json.dumps({"passed": report["passed"], "bookInfoApiCalls": report["bookInfoApiCalls"],
                      "metadataPassed": report.get("metadata", {}).get("passed", False),
                      "resourceGuardPassed": report["resourceGuardPassed"]}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
