"""Require the generated, browser-backed getBookInfo receipt; not real-site acceptance."""

import argparse
import hashlib
import json
from pathlib import Path


MAX_BYTES = 64 * 1024
SOURCE_SCRIPT_SHA256 = "045f640185166e2d9415e09b1e2d574ba45cf6119345ba180f4fe6e5a9bf4b04"
DIAGNOSTIC_RULE_SHA256 = "603a86357f45c3b850ec3c8cce2fe98266053aa1276fdf7178fbd5a4fe61166a"
DIAGNOSTICS = {"nameSelectorPresent": True, "authorSelectorPresent": True,
    "coverSelectorPresent": True, "bodyHasText": True, "bodyHasExpectedBookTitle": True,
    "bodyHasExpectedAuthor": True, "bodyHasSafetyPhrase": False, "captchaContainerPresent": False}
ROUNDTRIP_KEYS = {"dataIsObject", "sourceUrlMatches", "cookieJarDisabled",
                  "modernTocMarkerPresent", "metadataRulesMatch", "passed"}
KEYS = {"schemaVersion", "scope", "bookInfoApiCalls", "status", "isSuccess", "errorMsg",
    "dataIsObject", "nameMatches", "authorMatches", "coverMatches", "pageDiagnostics",
    "sourceDefinitionRoundtrip", "sourceScriptSha256", "diagnosticRuleSha256",
    "targetRequestCount", "targetRequest"}


class MetadataSmokeRejected(ValueError):
    pass


def verify(value):
    result = value.get("metadataReader") if isinstance(value, dict) else None
    if not isinstance(result, dict) or set(result) != KEYS:
        raise MetadataSmokeRejected("Generated metadata observation missing or unexpected")
    for key, expected in (("schemaVersion", 1), ("status", 200),
                          ("bookInfoApiCalls", 1), ("targetRequestCount", 1)):
        if type(result[key]) is not int or result[key] != expected:
            raise MetadataSmokeRejected("Generated metadata status or single-call count failed")
    if (result["scope"] != "generated-delayed-dom" or result["errorMsg"] != "" or
            any(result[key] is not True for key in
                ("isSuccess", "dataIsObject", "nameMatches", "authorMatches", "coverMatches")) or
            result["sourceScriptSha256"] != SOURCE_SCRIPT_SHA256 or
            result["diagnosticRuleSha256"] != DIAGNOSTIC_RULE_SHA256):
        raise MetadataSmokeRejected("Generated metadata result or exact probe definitions failed")
    diagnostic = result["pageDiagnostics"]
    if (not isinstance(diagnostic, dict) or set(diagnostic) != set(DIAGNOSTICS) or
            any(diagnostic[key] is not expected for key, expected in DIAGNOSTICS.items())):
        raise MetadataSmokeRejected("Generated returned-snapshot structure failed")
    roundtrip = result["sourceDefinitionRoundtrip"]
    if (not isinstance(roundtrip, dict) or set(roundtrip) != ROUNDTRIP_KEYS or
            any(value is not True for value in roundtrip.values())):
        raise MetadataSmokeRejected("Generated source rules were not preserved")
    if result["targetRequest"] != {"httpMethod": "GET", "body": None, "testHeader": None, "cookie": ""}:
        raise MetadataSmokeRejected("Generated metadata target or empty Cookie failed")
    return {"generatedBrowserBackedMetadataCases": 1, "savedSourceAndDetailParserVerified": True,
            "realSiteMetadataOrAuthenticationProven": False}


def unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise MetadataSmokeRejected("Duplicate report key")
        result[key] = value
    return result


def verify_file(path):
    with Path(path).open("rb") as stream:
        raw = stream.read(MAX_BYTES + 1)
    if len(raw) > MAX_BYTES:
        raise MetadataSmokeRejected("Report exceeds byte limit")
    try:
        value = json.loads(raw, object_pairs_hook=unique_keys)
    except (ValueError, UnicodeError):
        raise MetadataSmokeRejected("Malformed report") from None
    return {**verify(value), "jsonSha256": hashlib.sha256(raw).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    try:
        result = verify_file(args.report)
    except (OSError, MetadataSmokeRejected):
        parser.exit(1, "Generated metadata report rejected; no raw response or Cookie is echoed.\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
