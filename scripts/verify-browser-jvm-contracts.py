#!/usr/bin/env python3
"""Fail closed unless the two browser JUnit suites really ran.

The normal JVM unit-test job deliberately has no browser runtime.  Its XML may
therefore be green while all browser contracts were skipped.  This verifier is
intended for the dedicated browser job and deliberately validates testcase
records, not merely the testsuite summary attributes.
"""

from __future__ import annotations

import argparse
import json
import sys
import xml.etree.ElementTree as element_tree
from pathlib import Path
from typing import Any


MAX_XML_BYTES = 512 * 1024

EXPECTED: dict[str, tuple[str, ...]] = {
    "com.htmake.reader.utils.CamoufoxWebviewRendererTest": (
        "endlessGeneratedNavigationTimesOutAndTheNextRenderRecovers",
        "generatedNavigationBeforeDomReadyReturnsFinalDocumentWithoutReplayingPost",
        "pageJavaScriptDeletionBeforeDomReadyDoesNotResurrectCookies",
        "sourceJavaScriptDeletionOverridesSameResponseSetCookie",
        "importedHostOnlyCookieRetainsPathForSubresources",
        "scriptPromiseFailuresAreBoundedAndTheNextRenderRecovers",
        "cookiesArePersistedPerReaderNamespace",
        "sourceRegexReturnsTheResourceUrlWithoutFetchingIt",
        "unmatchedSourceRegexTimesOutAndTheNextRenderRecovers",
        "existingHttpOnlyCookieCanBeRenewed",
        "generatedScriptAndResourceRedirectsRespectOriginHeaderPolicy",
        "generatedClientNavigationReturnsTheFinalDocumentWithoutReplayingPost",
        "javaScriptStructuredResultsUseTheArchivedWebviewResponseFormat",
        "quotedCookieReplayMatchesWhatTheBrowserActuallyAccepted",
        "numericalLibraryImportsDoNotAllocateTheHostCpuThreadPool",
        "generatedSourceRuleRunsOnceInTheDocumentThatCompletesLoad",
        "stalledMainNavigationFailsInsteadOfReturningProxyErrorPage",
        "generatedRuleHeadersStayOnTheirOriginAcrossRedirectsAndScripts",
        "generatedPostRedirectsPreserveBodyCookiesAndOriginHeaders",
        "importedNetscapeCookiesRetainScopeInRealBrowserRequests",
        "generatedLoopbackRedirectCannotBypassTheEgressProxy",
        "generatedSourceStateDeletionFailsWithoutReplayingSideEffectsAndRecovers",
        "generatedUtf8PostPreservesRawBytesAndSourceScriptResult",
        "getPostScriptsAndSubresourcesUseTheBrowser",
    ),
    "com.htmake.reader.utils.LocalWebviewRendererTest": (
        "unknownHtmlCharsetFailsWithClearValidationError",
        "generatedPageDoesNotTriggerBrowserBackgroundNetworkRequests",
        "blocksPrivateRedirectsAtTheEgressProxy",
        "javascriptSubresourcesUseTheGuardedProxy",
        "htmlInputSupportsDeclaredNonUtf8Charset",
        "getAndPostExecuteInBrowser",
        "deniesPrivateTargetsByDefaultBeforeLaunchingChromium",
        "chunkedEventStreamsReachThePageBeforeTheOriginCloses",
        "readsHtmlAfterGeneratedClientNavigationWithoutReplayingTheRequest",
        "cookiesRemainInTheUserNamespace",
        "sourceRegexReturnsTheFirstMatchingResourceUrlAndAbortsItsFetch",
    ),
}


class VerificationError(ValueError):
    """A report is missing, malformed, skipped, or inconsistent."""


def _local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def _nonnegative_int(value: str | None, field: str, report: Path) -> int:
    try:
        number = int(value or "")
    except ValueError as error:
        raise VerificationError(f"{report.name}: invalid {field} summary") from error
    if number < 0:
        raise VerificationError(f"{report.name}: negative {field} summary")
    return number


def _parse(report: Path) -> element_tree.Element:
    try:
        size = report.stat().st_size
    except FileNotFoundError as error:
        raise VerificationError(f"missing report: {report.name}") from error
    if size <= 0 or size > MAX_XML_BYTES:
        raise VerificationError(f"{report.name}: report size is outside the allowed bound")
    with report.open('rb') as stream:
        payload = stream.read(MAX_XML_BYTES + 1)
    if not payload or len(payload) > MAX_XML_BYTES:
        raise VerificationError(f"{report.name}: report changed or exceeded the size bound")
    try:
        decoded = payload.decode('utf-8-sig', errors='strict')
    except UnicodeDecodeError as error:
        raise VerificationError(f"{report.name}: expected UTF-8 JUnit XML") from error
    if '\x00' in decoded:
        raise VerificationError(f"{report.name}: NUL bytes or unsupported XML encoding")
    lowered = decoded.lower()
    if "<!doctype" in lowered or "<!entity" in lowered:
        raise VerificationError(f"{report.name}: DTD and external entities are forbidden")
    try:
        return element_tree.fromstring(decoded)
    except element_tree.ParseError as error:
        raise VerificationError(f"{report.name}: malformed XML") from error


def _verify_suite(directory: Path, class_name: str, required_names: tuple[str, ...]) -> dict[str, Any]:
    report = directory / f"TEST-{class_name}.xml"
    root = _parse(report)
    if _local_name(root.tag) != "testsuite" or root.attrib.get("name") != class_name:
        raise VerificationError(f"{report.name}: expected one testsuite named {class_name}")

    cases = [node for node in root if _local_name(node.tag) == "testcase"]
    declared_tests = _nonnegative_int(root.attrib.get("tests"), "tests", report)
    if declared_tests != len(cases):
        raise VerificationError(f"{report.name}: tests summary does not match testcase records")
    for field in ("skipped", "failures", "errors"):
        if _nonnegative_int(root.attrib.get(field), field, report) != 0:
            raise VerificationError(f"{report.name}: {field} summary is not zero")

    seen: set[tuple[str, str]] = set()
    named_cases: set[str] = set()
    for case in cases:
        case_name = case.attrib.get("name", "")
        case_class = case.attrib.get("classname", "")
        if not case_name or not case_class:
            raise VerificationError(f"{report.name}: testcase without a class name or test name")
        key = (case_class, case_name)
        if key in seen:
            raise VerificationError(f"{report.name}: duplicate testcase {case_class}#{case_name}")
        seen.add(key)
        if case_class == class_name:
            named_cases.add(case_name)
        statuses = (case.attrib.get("status", ""), case.attrib.get("result", ""))
        if any(value.lower() not in {"", "run", "passed", "success", "completed"} for value in statuses):
            raise VerificationError(f"{report.name}: testcase {case_name} is not executed")
        if case_class != class_name:
            raise VerificationError(f"{report.name}: testcase belongs to a different suite")
        for child in case:
            if _local_name(child.tag) in {"skipped", "failure", "error"}:
                raise VerificationError(f"{report.name}: testcase {case_name} has a non-passing result")

    expected = set(required_names)
    missing = sorted(expected - named_cases)
    if missing:
        raise VerificationError(f"{report.name}: missing required testcase(s): {', '.join(missing)}")
    # A renamed/removed browser test must be visible to reviewers, while new
    # extra tests remain allowed as long as they executed successfully.
    unexpected = sorted(named_cases - expected)
    return {
        "report": report.name,
        "suite": class_name,
        "requiredTests": len(required_names),
        "executedTests": len(cases),
        "extraTests": len(unexpected),
        "skipped": 0,
        "failures": 0,
        "errors": 0,
    }


def verify_results(results_dir: Path) -> dict[str, Any]:
    if not results_dir.is_dir():
        raise VerificationError("results directory does not exist")
    suites = [_verify_suite(results_dir, class_name, names) for class_name, names in EXPECTED.items()]
    return {
        "schemaVersion": 1,
        "requiredSuites": len(EXPECTED),
        "requiredTests": sum(len(names) for names in EXPECTED.values()),
        "executedTests": sum(suite["executedTests"] for suite in suites),
        "skipped": 0,
        "failures": 0,
        "errors": 0,
        "suites": suites,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results-dir", required=True, type=Path, help="directory containing JUnit TEST-*.xml files")
    parser.add_argument("--json-output", type=Path, help="optional output file; stdout is used when omitted")
    args = parser.parse_args()
    try:
        summary = verify_results(args.results_dir)
        encoded = json.dumps(summary, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(encoded + "\n", encoding="utf-8")
        else:
            print(encoded)
        return 0
    except VerificationError as error:
        print(f"browser JVM contract verification failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
