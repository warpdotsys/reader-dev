"""Retain only finite worker stderr labels, never Reader logs or page content."""

import json
from pathlib import Path
import sys


PREFIX = b"READER_BROWSER_FAILURE "
MAX_LINE_BYTES = 4096
MAX_INPUT_BYTES = 256 * 1024
MAX_RECORDS = 8
PHASES = frozenset(("initialNavigation", "snapshotEventPump", "snapshotLoadState",
                   "snapshotContent", "snapshotStabilityPump", "sourceScriptStart", "sourceScriptRead"))
KINDS = frozenset(("unclassified", "navigationInterrupted", "executionContextDestroyed", "documentChanging"))
STATE_KINDS = frozenset(("sourceStateMissing", "sourceStateTypeInvalid",
                         "sourceStateBodyInvalid", "sourceStateStatusInvalid"))


def unique_fields(pairs):
    fields = {}
    for key, value in pairs:
        if key in fields:
            raise ValueError("Duplicate label")
        fields[key] = value
    return fields


def diagnostic(line):
    if not line.startswith(PREFIX) or len(line) > MAX_LINE_BYTES:
        return None
    try:
        value = json.loads(line[len(PREFIX):].decode("utf-8"), object_pairs_hook=unique_fields)
    except (ValueError, UnicodeError):
        return None
    if not isinstance(value, dict):
        return None
    if (type(value.get("operation")) is not str or value["operation"] not in PHASES or
            type(value.get("kind")) is not str or type(value.get("errorClass")) is not str):
        return None
    if set(value) == {"operation", "kind", "errorClass"}:
        return value if value["kind"] in KINDS and value["errorClass"] in {
            "Error", "TimeoutError", "TargetClosedError", "Other"} else None
    if (set(value) == {"operation", "kind", "errorClass", "mainFrameNavigationObserved"} and
            value["operation"] == "sourceScriptRead" and value["kind"] in STATE_KINDS and
            value["errorClass"] == "SourceScriptStateLost" and
            type(value["mainFrameNavigationObserved"]) is bool):
        return value
    return None


def collect(stream):
    records, total, discarding, truncated = [], 0, False, False
    while True:
        line = stream.readline(MAX_LINE_BYTES + 1)
        if not line:
            break
        total += len(line)
        if total > MAX_INPUT_BYTES:
            truncated = True
            break
        if discarding:
            discarding = not line.endswith(b"\n")
            continue
        if len(line) > MAX_LINE_BYTES:
            discarding = not line.endswith(b"\n")
            continue
        value = diagnostic(line)
        if value is not None:
            if len(records) == MAX_RECORDS:
                truncated = True
                break
            records.append(value)
    return {"schemaVersion": 1, "scope": "finite worker stderr labels; not raw logs or page content",
            "records": records, "truncated": truncated, "rawLogsPersisted": False}


def main():
    if len(sys.argv) != 2 or Path(sys.argv[1]).name != "WORKER_DIAGNOSTICS.json":
        raise SystemExit("Invalid diagnostic output")
    report = collect(sys.stdin.buffer)
    try:
        with Path(sys.argv[1]).open("x", encoding="utf-8") as output:
            json.dump(report, output, sort_keys=True, indent=2)
            output.write("\n")
    except OSError:
        raise SystemExit("Diagnostic output unavailable") from None
    print(json.dumps({"diagnosticRecords": len(report["records"]), "truncated": report["truncated"]}))


if __name__ == "__main__":
    main()
