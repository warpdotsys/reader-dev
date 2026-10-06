"""Require actual async Reader API observations; synthetic unit JSON is not acceptance."""

import argparse
import hashlib
import json
from pathlib import Path


MAX_BYTES = 64 * 1024
CASE_KEYS = {"case", "status", "isSuccess", "errorMsg", "count", "nameMatches",
             "authorMatches", "bookUrlMatches", "targetRequest", "targetCookie",
             "scriptMarkCount", "scriptMarkCookie"}


class AsyncSmokeRejected(ValueError):
    pass


def verify(value):
    if not isinstance(value, dict):
        raise AsyncSmokeRejected("Unexpected report")
    cases = value.get("asyncReaderCases")
    if not isinstance(cases, list) or len(cases) != 2:
        raise AsyncSmokeRejected("Async Reader observations missing")
    for case, phase in zip(cases, ("get", "post")):
        if (not isinstance(case, dict) or set(case) != CASE_KEYS or
                case.get("case") != phase or type(case.get("status")) is not int or case["status"] != 200 or
                case.get("isSuccess") is not True or case.get("errorMsg") != "" or
                type(case.get("count")) is not int or case["count"] != 1 or
                any(case.get(key) is not True for key in ("nameMatches", "authorMatches", "bookUrlMatches")) or
                type(case.get("scriptMarkCount")) is not int or case["scriptMarkCount"] != 1):
            raise AsyncSmokeRejected("Async Reader observation failed")
        target = ({"httpMethod": "GET", "body": None, "testHeader": None} if phase == "get" else
                  {"httpMethod": "POST", "body": "q=async", "testHeader": "async"})
        cookie = "" if phase == "get" else "asyncOnly=generated"
        marker_cookie = "asyncOnly=generated" if phase == "get" else ""
        if case["targetRequest"] != target or case["targetCookie"] != cookie or case["scriptMarkCookie"] != marker_cookie:
            raise AsyncSmokeRejected("Generated async request or Cookie mismatch")
    cleanup = value.get("asyncCookieCleanup")
    if (not isinstance(cleanup, dict) or
            set(cleanup) != {"sameUserNextRequestCookie", "sameUserNextRequestCount", "deletedVerified"} or
            cleanup.get("sameUserNextRequestCookie") != "" or
            type(cleanup.get("sameUserNextRequestCount")) is not int or cleanup["sameUserNextRequestCount"] != 1 or
            cleanup.get("deletedVerified") is not True):
        raise AsyncSmokeRejected("Async Cookie deletion followup missing or failed")
    return {"asyncReaderCases": 2, "singleExecutionAndPostVerified": True,
            "sameUserAsyncCookieDeletionVerified": True}


def verify_file(path):
    with Path(path).open("rb") as stream:
        raw = stream.read(MAX_BYTES + 1)
    if len(raw) > MAX_BYTES:
        raise AsyncSmokeRejected("Report exceeds byte limit")
    try:
        value = json.loads(raw)
    except (ValueError, UnicodeError):
        raise AsyncSmokeRejected("Malformed report") from None
    return {**verify(value), "jsonSha256": hashlib.sha256(raw).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    try:
        verified = verify_file(args.report)
    except (OSError, AsyncSmokeRejected):
        parser.exit(1, "Async Reader report rejected; no raw response or Cookie is echoed.\n")
    print(json.dumps(verified, sort_keys=True))


if __name__ == "__main__":
    main()
