#!/usr/bin/env python3
"""Report bounded public-page observations from two isolated CI jobs.

The archived image is a historical reference, not the original production
remote WebView. A matching list count is not proof of response or API parity.
No page body, credentials, Cookie values, or book titles are read or emitted.
"""

import argparse
from datetime import datetime
import json
from pathlib import Path
from urllib.parse import urlparse


MAX_LOG_BYTES = 1024 * 1024
MAX_BOOKS = 100


def records(path):
    if path.stat().st_size > MAX_LOG_BYTES:
        raise ValueError(f"{path.name}: probe log exceeds 1 MiB")
    output = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError(f"{path.name}: JSON line is not an object")
            output.append(value)
    return output


def unique_record(items, key, label):
    matches = [item[key] for item in items if key in item]
    if len(matches) != 1 or not isinstance(matches[0], dict):
        raise ValueError(f"{label}: expected exactly one {key} object")
    return matches[0]


def positive_count(value, label):
    if isinstance(value, bool) or not isinstance(value, int) or not 1 <= value <= MAX_BOOKS:
        raise ValueError(f"{label}: expected a count from 1 to {MAX_BOOKS}")
    return value


def timestamp(value, label):
    if not isinstance(value, str):
        raise ValueError(f"{label}: missing observation timestamp")
    parsed = datetime.fromisoformat(value)
    if parsed.tzinfo is None or parsed.utcoffset() is None:
        raise ValueError(f"{label}: timestamp is not timezone-aware")
    return parsed


def compare(archived_log, camoufox_log):
    archived = records(archived_log)
    camoufox = records(camoufox_log)
    public = unique_record(archived, "public", "archived reference")
    dom = unique_record(archived, "publicDomBookListCount", "archived reference")
    projection = unique_record(archived, "publicDomBookProjection", "archived reference")
    candidates = [item for item in camoufox if "source" in item]
    if len(candidates) != 1:
        raise ValueError("bundled Camoufox: expected exactly one source result")
    candidate = candidates[0]

    url = dom.get("url")
    parsed = urlparse(url) if isinstance(url, str) else None
    if (parsed is None or parsed.scheme != "https" or not parsed.hostname or
            parsed.username or parsed.password or parsed.query or parsed.fragment or
            candidate.get("source") != url or projection.get("url") != url):
        raise ValueError("probe outputs do not identify the same credential-free HTTPS page")
    if public.get("status") != 200 or dom.get("status") != 200 or projection.get("status") != 200:
        raise ValueError("archived reference did not return HTTP 200 for all public probes")
    if candidate.get("status") != 200 or candidate.get("isSuccess") is not True:
        raise ValueError("bundled Camoufox public search did not succeed")
    if candidate.get("errorMsg") not in (None, ""):
        raise ValueError("bundled Camoufox returned a non-empty error message")
    archived_count = positive_count(dom.get("count"), "archived DOM")
    projection_count = positive_count(projection.get("count"), "archived projection")
    candidate_count = positive_count(candidate.get("bookCount"), "Reader search")
    old_time = timestamp(dom.get("observedAt"), "archived DOM")
    projection_time = timestamp(projection.get("observedAt"), "archived projection")
    new_time = timestamp(candidate.get("observedAt"), "Reader search")
    if not isinstance(public.get("bodySha256"), str) or len(public["bodySha256"]) != 64:
        raise ValueError("archived reference is missing its public response checksum")
    if not isinstance(candidate.get("projectionSha256"), str) or len(candidate["projectionSha256"]) != 64:
        raise ValueError("bundled Camoufox is missing its book projection checksum")
    if not isinstance(projection.get("projectionSha256"), str) or len(projection["projectionSha256"]) != 64:
        raise ValueError("archived reference is missing its book projection checksum")

    equal = archived_count == candidate_count
    projection_equal = projection["projectionSha256"] == candidate["projectionSha256"]
    return {
        "scope": "archived-reference-vs-bundled-camoufox-public-page",
        "url": url,
        "originalJarCompared": False,
        "referenceProvenProduction": False,
        "fullResponseParityProven": False,
        "archivedReference": {
            "directPageStatus": public["status"],
            "directPageSha256": public["bodySha256"],
            "domListStatus": dom["status"],
            "domListCount": archived_count,
            "observedAt": dom["observedAt"],
            "projectionCount": projection_count,
            "projectionSha256": projection["projectionSha256"],
            "projectionObservedAt": projection["observedAt"],
        },
        "bundledCamoufox": {
            "readerStatus": candidate["status"],
            "readerIsSuccess": candidate["isSuccess"],
            "readerErrorMsgEmpty": True,
            "bookCount": candidate_count,
            "bookProjectionSha256": candidate["projectionSha256"],
            "observedAt": candidate["observedAt"],
        },
        "observationDeltaSeconds": round(abs((new_time - old_time).total_seconds()), 3),
        "projectionDeltaSeconds": round(abs((new_time - projection_time).total_seconds()), 3),
        "countEqual": equal,
        "referenceInternalCountEqual": archived_count == projection_count,
        "projectionEqual": projection_equal,
        "interpretation": (
            "matching-projection-with-time-skew" if projection_equal else
            "divergent-projection-needs-investigation"
        ),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archived-log", required=True, type=Path)
    parser.add_argument("--camoufox-log", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    result = compare(args.archived_log, args.camoufox_log)
    serialized = json.dumps(result, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    with args.report.open("x", encoding="utf-8") as output:
        output.write(serialized)
    print(serialized, end="")
    if not result["countEqual"]:
        print("::warning::The archived DOM and Reader search counts differ; "
              "the public page may also have changed between observations.")
    if not result["projectionEqual"]:
        print("::warning::The archived and Reader book projections differ; "
              "inspect site drift and parsing before attributing this to the browser.")


if __name__ == "__main__":
    main()
