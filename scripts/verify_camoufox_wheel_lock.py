"""Fail closed unless every pinned Camoufox requirement has SHA-256 hashes."""

import re
import sys
from pathlib import Path


ENTRY = re.compile(r"([A-Za-z0-9_.-]+)==([A-Za-z0-9_.-]+)((?:\s+--hash=sha256:[0-9a-f]{64})+)")
HASH = re.compile(r"--hash=sha256:[0-9a-f]{64}")


def logical_lines(path):
    pending = ""
    for physical in path.read_text(encoding="utf-8").splitlines():
        line = physical.strip()
        if not line or line.startswith("#"):
            continue
        if line.endswith("\\"):
            pending += line[:-1].rstrip() + " "
            continue
        yield pending + line
        pending = ""
    if pending:
        raise ValueError("Unterminated requirement continuation")


def verify(path):
    names = set()
    count = 0
    for line in logical_lines(path):
        match = ENTRY.fullmatch(line)
        if not match:
            raise ValueError(f"Unpinned, unhashed or unsupported requirement: {line[:120]}")
        name = match.group(1).lower().replace("_", "-").replace(".", "-")
        if name in names:
            raise ValueError(f"Duplicate requirement: {name}")
        names.add(name)
        count += len(HASH.findall(match.group(3)))
    if len(names) < 38 or count < len(names):
        raise ValueError(f"Incomplete Camoufox wheel lock: {len(names)} packages, {count} hashes")
    print(f"Verified {len(names)} pinned packages and {count} SHA-256 hashes")


if __name__ == "__main__":
    verify(Path(sys.argv[1]))
