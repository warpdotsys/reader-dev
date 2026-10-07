"""Generated-only metadata fixture and strict Reader observation checks.

No main, network client, account or real content. The caller must independently
verify the private loopback namespace before launching any Reader or renderer.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import re

spec = importlib.util.spec_from_file_location("metadata_differential_definition",
    Path(__file__).with_name("probe-public-metadata.py"))
DEFINITION = importlib.util.module_from_spec(spec)
spec.loader.exec_module(DEFINITION)
SCRIPT_HASH = "045f640185166e2d9415e09b1e2d574ba45cf6119345ba180f4fe6e5a9bf4b04"
RULE_HASH = "603a86357f45c3b850ec3c8cce2fe98266053aa1276fdf7178fbd5a4fe61166a"
assert hashlib.sha256(DEFINITION.METADATA_DOM_SCRIPT.encode()).hexdigest() == SCRIPT_HASH
assert hashlib.sha256(DEFINITION.PAGE_DIAGNOSTIC_RULE.encode()).hexdigest() == RULE_HASH
ROUNDTRIP_KEYS = {"dataIsObject", "sourceUrlMatches", "cookieJarDisabled", "modernTocMarkerPresent",
                  "metadataRulesMatch", "passed"}
TARGET_FIELDS = {"httpMethod": "GET", "body": None, "testHeader": None, "cookie": ""}
CLOCK_FIELDS = frozenset(("latestChapterTime", "lastCheckTime", "durChapterTime"))


def require_base(base):
    match = re.fullmatch(r"http://127\.0\.0\.1:([1-9][0-9]{0,4})", base or "")
    if match is None or int(match[1]) > 65535:
        raise ValueError("Metadata fixture must use exact private loopback")


def source_definition(base):
    require_base(base)
    source = DEFINITION.source_definition()
    source["bookSourceUrl"] = base + "/metadata-source"
    source["bookSourceName"] = "Generated delayed metadata differential"
    return source


def source_roundtrip(data, base):
    expected = source_definition(base)
    source = data if isinstance(data, dict) else {}
    rules = source.get("ruleBookInfo")
    result = {"dataIsObject": isinstance(data, dict),
        "sourceUrlMatches": source.get("bookSourceUrl") == expected["bookSourceUrl"],
        "cookieJarDisabled": source.get("enabledCookieJar") is False,
        "modernTocMarkerPresent": isinstance(source.get("ruleToc"), dict),
        "metadataRulesMatch": isinstance(rules, dict) and all(
            rules.get(key) == value for key, value in expected["ruleBookInfo"].items())}
    result["passed"] = all(result.values())
    return result


def book_info_request(base):
    require_base(base)
    return {"url": base + "/book-info, " + json.dumps({"webView": True,
        "webJs": DEFINITION.METADATA_DOM_SCRIPT}), "bookSourceUrl": base + "/metadata-source"}


def generated_html():
    # The initial DOM has no metadata selectors. Page-owned delayed insertion
    # must finish before the unchanged read-only source script can return them.
    return """<!doctype html><html><head><title>Generated metadata fixture</title></head><body>
<main id="generated-metadata"></main><script>
setTimeout(() => {
  document.querySelector('#generated-metadata').innerHTML = '<h1 id="bookName">黎明之剑</h1>'
    + '<section class="book-info-top"><span class="book-meta"><a class="author">远瞳</a></span></section>'
    + '<div id="bookImg"><img src="/generated-cover.svg"></div>';
}, 300);
</script></body></html>""".encode("utf-8")


def serve(handler):
    if handler.path == "/book-info":
        with handler.server.lock:
            handler.server.metadata_requests.append({"httpMethod": "GET", "body": None,
                "testHeader": handler.headers.get("X-Fixture"), "cookie": handler.headers.get("Cookie", "")})
        body, content_type = generated_html(), "text/html; charset=utf-8"
    elif handler.path == "/generated-cover.svg":
        body = b'<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"></svg>'
        content_type = "image/svg+xml"
    else:
        return False
    handler.send_response(200)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)
    return True


def validate(observation, clock_contract=False):
    if type(clock_contract) is not bool:
        raise RuntimeError("Invalid metadata clock contract mode")
    if not isinstance(observation, dict) or set(observation) != {
            "status", "returnData", "sourceDefinitionRoundtrip", "targetRequests",
            "fixtureBase", "sourceScriptSha256", "diagnosticRuleSha256", "bookInfoApiCalls"} | (
                {"requestWindowMs"} if clock_contract else set()):
        raise RuntimeError("Missing actual generated metadata observation")
    base = observation["fixtureBase"]
    require_base(base)
    if type(observation["status"]) is not int or observation["status"] != 200 or \
            type(observation["bookInfoApiCalls"]) is not int or observation["bookInfoApiCalls"] != 1:
        raise RuntimeError("Generated metadata HTTP/call contract differs")
    roundtrip = observation["sourceDefinitionRoundtrip"]
    if not isinstance(roundtrip, dict) or set(roundtrip) != ROUNDTRIP_KEYS or \
            any(value is not True for value in roundtrip.values()):
        raise RuntimeError("Generated metadata source rules were not retained")
    if observation["sourceScriptSha256"] != SCRIPT_HASH or observation["diagnosticRuleSha256"] != RULE_HASH:
        raise RuntimeError("Generated metadata rule identity differs")
    if observation["targetRequests"] != [TARGET_FIELDS] or \
            json.dumps(observation["targetRequests"], sort_keys=True) != json.dumps([TARGET_FIELDS], sort_keys=True):
        raise RuntimeError("Generated metadata target GET/Cookie fields differ")
    value = observation["returnData"]
    if not isinstance(value, dict) or value.get("isSuccess") is not True or value.get("errorMsg") != "":
        raise RuntimeError("Generated metadata ReturnData contract differs")
    book = value.get("data")
    if not isinstance(book, dict) or book.get("name") != "黎明之剑" or book.get("author") != "远瞳" or \
            book.get("bookUrl") != book_info_request(base)["url"] or book.get("coverUrl") != base + "/generated-cover.svg":
        raise RuntimeError("Generated metadata selectors or URLs differ")
    if not set(DEFINITION.FIELDS).issubset(book):
        raise RuntimeError("Generated metadata default fields are missing")
    for key in ("type", "group", "totalChapterNum", "durChapterIndex", "durChapterPos"):
        if type(book[key]) is not int or book[key] != 0:
            raise RuntimeError("Generated metadata default field type differs")
    if book["canUpdate"] is not True or type(book["tocUrl"]) is not str:
        raise RuntimeError("Generated metadata default boolean type differs")
    flags = DEFINITION.page_diagnostics(book.get("intro"))
    expected = {key: key not in {"bodyHasSafetyPhrase", "captchaContainerPresent"}
                for key in DEFINITION.PAGE_DIAGNOSTIC_KEYS}
    if flags is None or any(flags[key] is not expected[key] for key in expected):
        raise RuntimeError("Generated metadata page diagnostics differ")
    if clock_contract:
        window = observation["requestWindowMs"]
        if not isinstance(window, dict) or set(window) != {"started", "completed"} or \
                any(type(value) is not int for value in window.values()) or \
                not 0 < window["started"] <= window["completed"] <= window["started"] + 35000:
            raise RuntimeError("Missing actual metadata request clock window")
        for key in CLOCK_FIELDS:
            if type(book.get(key)) is not int or not window["started"] <= book[key] <= window["completed"]:
                raise RuntimeError("Metadata default clock outside its own actual request window")


def compare(left, right, clock_contract=False):
    """Never edit either original observation; explicitly qualify clock parity.

    This optional mode uses original bytecode-verified default clock fields. It
    is not literal equality and cannot accept stale, missing or future clocks.
    """
    validate(left, clock_contract)
    validate(right, clock_contract)

    def encoded(value):
        return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)

    if not clock_contract:
        if encoded(left) != encoded(right):
            raise RuntimeError("Three-way complete metadata JSON differs")
        return {"mode": "literal-full-json", "fullRawReturnDataEqual": True}

    def static_fields(value):
        # A comparison view only. Every other raw field, type, nested key and
        # ReturnData extra remains mandatory. The originals retain all clocks.
        result = {key: field for key, field in value.items() if key != "requestWindowMs"}
        result["returnData"] = dict(value["returnData"])
        result["returnData"]["data"] = {key: field for key, field in value["returnData"]["data"].items()
                                           if key not in CLOCK_FIELDS}
        return result

    if encoded(static_fields(left)) != encoded(static_fields(right)):
        raise RuntimeError("Three-way non-clock complete metadata JSON differs")
    return {"mode": "exact-static-json-and-bounded-default-clocks",
        "fullRawReturnDataEqual": encoded(left["returnData"]) == encoded(right["returnData"]),
        "nonClockFullJsonEqual": True, "clockFieldsFreshInEachOwnRequestWindow": True,
        "clockFields": {key: {"left": left["returnData"]["data"][key], "right": right["returnData"]["data"][key]}
                        for key in sorted(CLOCK_FIELDS)},
        "leftRequestWindowMs": left["requestWindowMs"], "rightRequestWindowMs": right["requestWindowMs"]}
