#!/usr/bin/env python3
"""Reject missing/skipped complete-image UI execution; generated screenshots only."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import xml.etree.ElementTree as ET

CLASS = "com.medwarp.reader.browserpoc.NativeImageDefaultUiTest"
METHOD = "defaultImageRegisterLoginImportReadReloadAndLogout"
STAGES = ("login", "shelf", "reading", "logout")
MAX_XML = 1024 * 1024
MAX_PNG = 8 * 1024 * 1024


class JourneyRejected(ValueError):
    pass


def bounded(path, maximum):
    if path.is_symlink() or not path.is_file():
        raise JourneyRejected("Expected a regular non-symlink evidence file")
    with path.open("rb") as stream:
        raw = stream.read(maximum + 1)
    if not raw or len(raw) > maximum:
        raise JourneyRejected("Empty or oversized evidence")
    return raw


def verify(xml_directory, screenshots, revision):
    if not isinstance(revision, str) or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise JourneyRejected("Expected exact tested revision")
    files = list(xml_directory.glob("TEST-*.xml"))
    expected = xml_directory / ("TEST-" + CLASS + ".xml")
    if files != [expected]:
        raise JourneyRejected("Expected only the explicitly selected UI suite")
    raw = bounded(expected, MAX_XML)
    if b"<!DOCTYPE" in raw.upper() or b"<!ENTITY" in raw.upper():
        raise JourneyRejected("DTD and entity declarations are forbidden")
    try:
        suite = ET.fromstring(raw)
        if suite.tag != "testsuite" or suite.get("name") != CLASS:
            raise JourneyRejected("Unexpected suite")
        for key, value in (("tests", "1"), ("failures", "0"), ("errors", "0"), ("skipped", "0")):
            if suite.get(key) != value:
                raise JourneyRejected("UI suite must execute exactly once without failures or skips")
        cases = suite.findall("testcase")
        if len(cases) != 1 or cases[0].get("classname") != CLASS or cases[0].get("name") != METHOD:
            raise JourneyRejected("Unexpected UI journey")
        duration = float(cases[0].get("time", ""))
        if not 0 < duration <= 180 or any(cases[0].find(tag) is not None for tag in ("failure", "error", "skipped")):
            raise JourneyRejected("UI journey missing actual successful execution")
    except (ET.ParseError, TypeError, ValueError) as failure:
        raise JourneyRejected("Invalid or failed UI report") from failure
    observations = []
    for stage in STAGES:
        name = "native-default-ui-generated-" + stage + ".png"
        pixels = bounded(screenshots / name, MAX_PNG)
        if len(pixels) < 33 or pixels[:8] != b"\x89PNG\r\n\x1a\n" or pixels[12:16] != b"IHDR":
            raise JourneyRejected("Invalid screenshot header")
        width, height = struct.unpack(">II", pixels[16:24])
        if (width, height) != (1280, 900):
            raise JourneyRejected("Unexpected actual client viewport")
        observations.append({"file": name, "sha256": hashlib.sha256(pixels).hexdigest(),
                             "bytes": len(pixels), "width": width, "height": height})
    return {"schemaVersion": 1, "scope": "complete-image-default-ui-generated-journey",
            "testedRevision": revision, "tests": 1, "failures": 0, "errors": 0, "skipped": 0,
            "seconds": duration, "xmlSha256": hashlib.sha256(raw).hexdigest(), "screenshots": observations,
            "clientRunsOutsideServerResourceBudget": True,
            "realAccountOrPrivateBooksTested": False, "productionOrLongRunProven": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--xml-directory", type=Path, required=True)
    parser.add_argument("--screenshots", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    try:
        result = verify(args.xml_directory, args.screenshots, args.revision)
    except (OSError, JourneyRejected):
        parser.exit(1, "Complete-image UI journey rejected; no response bodies or session tokens echoed.\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
