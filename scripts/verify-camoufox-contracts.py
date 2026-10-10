"""Fail closed on the actual default-engine JUnit report before release export.

This validates a report, not a browser. Only a real runtime execution can produce
acceptance evidence; the unit-test XML fixtures are explicitly synthetic.
"""

import argparse
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET


SUITE = "com.htmake.reader.utils.CamoufoxWebviewRendererTest"
MAX_REPORT_BYTES = 256 * 1024
NAVIGATION_CONTRACT = "generatedClientNavigationReturnsTheFinalDocumentWithoutReplayingPost"
EARLY_NAVIGATION_CONTRACT = "generatedNavigationBeforeDomReadyReturnsFinalDocumentWithoutReplayingPost"
ENDLESS_NAVIGATION_CONTRACT = "endlessGeneratedNavigationTimesOutAndTheNextRenderRecovers"
PROMISE_CONTRACT = "scriptPromiseFailuresAreBoundedAndTheNextRenderRecovers"
STATE_DELETION_CONTRACT = "generatedSourceStateDeletionFailsWithoutReplayingSideEffectsAndRecovers"
SOURCE_NAVIGATION_CONTRACT = "generatedSourceRuleRunsOnceInTheDocumentThatCompletesLoad"
NUMERICAL_THREAD_CONTRACT = "numericalLibraryImportsDoNotAllocateTheHostCpuThreadPool"
UTF8_POST_CONTRACT = "generatedUtf8PostPreservesRawBytesAndSourceScriptResult"
ORIGIN_HEADER_CONTRACT = "generatedRuleHeadersStayOnTheirOriginAcrossRedirectsAndScripts"
POST_REDIRECT_HEADER_CONTRACT = "generatedPostRedirectsPreserveBodyCookiesAndOriginHeaders"
SCRIPT_REDIRECT_HEADER_CONTRACT = "generatedScriptAndResourceRedirectsRespectOriginHeaderPolicy"
LOOPBACK_PROXY_CONTRACT = "generatedLoopbackRedirectCannotBypassTheEgressProxy"
CONTRACTS = frozenset((
    "pageJavaScriptDeletionBeforeDomReadyDoesNotResurrectCookies",
    "sourceJavaScriptDeletionOverridesSameResponseSetCookie",
    "importedHostOnlyCookieRetainsPathForSubresources",
    "cookiesArePersistedPerReaderNamespace",
    "sourceRegexReturnsTheResourceUrlWithoutFetchingIt",
    "unmatchedSourceRegexTimesOutAndTheNextRenderRecovers",
    "existingHttpOnlyCookieCanBeRenewed",
    NAVIGATION_CONTRACT,
    EARLY_NAVIGATION_CONTRACT,
    ENDLESS_NAVIGATION_CONTRACT,
    PROMISE_CONTRACT,
    STATE_DELETION_CONTRACT,
    SOURCE_NAVIGATION_CONTRACT,
    NUMERICAL_THREAD_CONTRACT,
    UTF8_POST_CONTRACT,
    ORIGIN_HEADER_CONTRACT,
    POST_REDIRECT_HEADER_CONTRACT,
    SCRIPT_REDIRECT_HEADER_CONTRACT,
    LOOPBACK_PROXY_CONTRACT,
    "javaScriptStructuredResultsUseTheArchivedWebviewResponseFormat",
    "quotedCookieReplayMatchesWhatTheBrowserActuallyAccepted",
    "stalledMainNavigationFailsInsteadOfReturningProxyErrorPage",
    "importedNetscapeCookiesRetainScopeInRealBrowserRequests",
    "getPostScriptsAndSubresourcesUseTheBrowser",
))


class ContractReportError(ValueError):
    pass


def verify_report(path):
    # Read with a hard byte cap rather than trusting the file's stat or contents.
    with Path(path).open("rb") as stream:
        raw = stream.read(MAX_REPORT_BYTES + 1)
    if len(raw) > MAX_REPORT_BYTES:
        raise ContractReportError("Default-engine report exceeds the byte limit")
    try:
        suite = ET.fromstring(raw)
    except ET.ParseError as error:
        raise ContractReportError("Malformed default-engine report") from error
    if suite.tag != "testsuite" or suite.get("name") != SUITE:
        raise ContractReportError("Wrong default-engine test suite")
    counters = {}
    for key in ("tests", "failures", "errors", "skipped"):
        value = suite.get(key, "")
        if not 1 <= len(value) <= 10 or not value.isascii() or not value.isdecimal():
            raise ContractReportError("Missing or invalid default-engine counters")
        counters[key] = int(value)
    if counters["tests"] != len(CONTRACTS) or any(counters[key] for key in ("failures", "errors", "skipped")):
        raise ContractReportError("Default-engine contracts failed, skipped, or incomplete")
    cases = suite.findall("testcase")
    names = [case.get("name") for case in cases]
    if len(cases) != len(CONTRACTS) or set(names) != CONTRACTS:
        raise ContractReportError("Missing, duplicate, or unexpected default-engine contracts")
    if any(case.get("classname") != SUITE for case in cases):
        raise ContractReportError("Wrong default-engine case class")
    if any(case.find(tag) is not None for case in cases for tag in ("failure", "error", "skipped")):
        raise ContractReportError("Default-engine case failures contradict suite counters")
    return {
        "suite": SUITE,
        **counters,
        "navigationContractPresent": True,
        "earlyNavigationContractPresent": True,
        "endlessNavigationContractPresent": True,
        "promiseContractPresent": True,
        "stateDeletionContractPresent": True,
        "sourceNavigationContractPresent": True,
        "numericalThreadContractPresent": True,
        "utf8PostContractPresent": True,
        "originHeaderContractPresent": True,
        "postRedirectHeaderContractPresent": True,
        "scriptRedirectHeaderContractPresent": True,
        "loopbackProxyContractPresent": True,
        "xmlSha256": hashlib.sha256(raw).hexdigest(),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    try:
        verified = verify_report(args.report)
    except (OSError, ContractReportError):
        # Failed XML may contain page URLs/cookies in exception text; never echo it.
        parser.exit(1, "Default-engine contract report rejected; inspect the preserved generated-only XML.\n")
    print(json.dumps(verified, sort_keys=True))


if __name__ == "__main__":
    main()
