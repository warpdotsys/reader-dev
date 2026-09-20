#!/usr/bin/env python3
"""Redact legacy Reader request secrets without retaining an unredacted copy."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import stat
import tempfile


ALLOWED_ROOT = Path("/opt/reader-pro-restored/logs")
REQUEST_BODY = re.compile(rb"(Request body:\s*).*(\r?\n)?$")
READER_QUERY = re.compile(rb"(/reader3/[^\s?\"']*)\?[^\s\"']*")
QUERY_SECRET = re.compile(
    rb"((?:accessToken|secureKey|password)=)[^&\s\"']*", re.IGNORECASE
)
JSON_SECRET = re.compile(
    rb"(\"(?:password|accessToken|secureKey)\"\s*:\s*\")[^\"]*(\")",
    re.IGNORECASE,
)
ESCAPED_JSON_SECRET = re.compile(
    rb"(\\\"(?:password|accessToken|secureKey)\\\"\s*:\s*\\\")[^\\\"]*(\\\")",
    re.IGNORECASE,
)


def redact_line(line: bytes) -> tuple[bytes, int]:
    replacements = 0

    def replace(pattern: re.Pattern[bytes], replacement: bytes) -> None:
        nonlocal line, replacements
        line, count = pattern.subn(replacement, line)
        replacements += count

    replace(REQUEST_BODY, rb"\1<redacted historical entry>\2")
    replace(READER_QUERY, rb"\1?<redacted>")
    replace(QUERY_SECRET, rb"\1<redacted>")
    replace(JSON_SECRET, rb"\1<redacted>\2")
    replace(ESCAPED_JSON_SECRET, rb"\1<redacted>\2")
    return line, replacements


def validate_path(raw_path: str) -> Path:
    path = Path(raw_path)
    resolved = path.resolve(strict=True)
    if resolved.parent != ALLOWED_ROOT or not resolved.name.startswith("reader-"):
        raise ValueError(f"refusing path outside {ALLOWED_ROOT}: {resolved}")
    if not resolved.name.endswith(".log") or not resolved.is_file():
        raise ValueError(f"refusing non-log file: {resolved}")
    if path.is_symlink():
        raise ValueError(f"refusing symbolic link: {path}")
    return resolved


def redact_file(path: Path) -> int:
    metadata = path.stat()
    replacements = 0
    temp_path: Path | None = None
    try:
        with path.open("rb") as source, tempfile.NamedTemporaryFile(
            mode="wb", prefix=f".{path.name}.", suffix=".redacting", dir=path.parent, delete=False
        ) as target:
            temp_path = Path(target.name)
            for line in source:
                redacted, count = redact_line(line)
                target.write(redacted)
                replacements += count
            target.flush()
            os.fsync(target.fileno())

        if replacements == 0:
            temp_path.unlink()
            return 0

        os.chmod(temp_path, stat.S_IMODE(metadata.st_mode))
        os.chown(temp_path, metadata.st_uid, metadata.st_gid)
        os.replace(temp_path, path)
        temp_path = None
        os.utime(path, ns=(metadata.st_atime_ns, metadata.st_mtime_ns))
        directory_fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
        return replacements
    finally:
        if temp_path is not None and temp_path.exists():
            temp_path.unlink()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="+", help="explicit Reader log paths to redact")
    args = parser.parse_args()

    total = 0
    for raw_path in args.paths:
        path = validate_path(raw_path)
        replacements = redact_file(path)
        total += replacements
        print(f"{path.name}: {replacements} replacement(s)")
    print(f"total: {total} replacement(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
