#!/usr/bin/env python3
"""Read-only identity validation for an isolated Reader candidate profile.

This module deliberately does *not* prepare disks, mount filesystems, start a
process, or register a service.  It is the narrow boundary between an
operator-supplied JSON identity declaration and a later controller.  A
controller must validate the returned immutable profile and independently
measure every artifact before it performs any mutation.
"""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
from typing import Any, Mapping


SCHEMA_VERSION = 1
CPU_QUOTA_PERCENT = 200
MEMORY_MAX_BYTES = 2 * 1024 ** 3
MEMORY_SWAP_MAX_BYTES = 0
OLD_WORK_DIR = "/mnt/d/Reader-Pro-Local-Full-610-20261015-a"
OLD_MOUNT_ROOT = "/mnt/reader-local-full-610"
OLD_UNIT = "reader-local-full-610.service"
OLD_NAME = "reader-local-full-610"
RESERVED_PORTS = frozenset((18961, 18963, 18964))
_ID = re.compile(r"[a-z][a-z0-9-]{2,62}")
_SHA256 = re.compile(r"[0-9a-f]{64}")
_REVISION = re.compile(r"[0-9a-f]{7,64}")
_SEMVER = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?")


class ProfileError(ValueError):
    """A profile is malformed, unsafe, or belongs to the old fixed scope."""


class DuplicateKeyError(ProfileError):
    """Strict JSON parser error: ambiguous duplicate object key."""


@dataclass(frozen=True)
class CandidateProfile:
    candidate_id: str
    version: str
    revision: str
    image_id: str
    archive_sha256: str
    reader_jar_sha256: str
    license_jar_sha256: str
    software_manifest_sha256: str
    work_dir: str
    mount_root: str
    image_disk: str
    data_disk: str
    unit: str
    name: str
    reader_port: int
    license_port: int
    image_bytes: int
    data_bytes: int
    cpu_quota_percent: int
    memory_max_bytes: int
    memory_high_bytes: int
    memory_swap_max_bytes: int
    tasks_max: int
    log_current_max_bytes: int
    log_previous_files: int
    outer_log_max_bytes: int
    outer_log_retained_files: int


def _reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKeyError("duplicate JSON key: " + key)
        result[key] = value
    return result


def _json_constant(value: str) -> None:
    raise ProfileError("non-finite JSON number is prohibited: " + value)


def parse_profile_json(raw: bytes | str) -> CandidateProfile:
    """Strictly parse one profile; duplicate keys and unknown fields are errors."""
    try:
        value = json.loads(raw, object_pairs_hook=_reject_duplicates, parse_constant=_json_constant)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise ProfileError("invalid profile JSON") from error
    return validate_profile(value)


def _object(value: Any, keys: set[str], label: str) -> Mapping[str, Any]:
    if type(value) is not dict or set(value) != keys:
        raise ProfileError(label + " has unknown, missing, or duplicate-normalized fields")
    return value


def _text(value: Any, label: str, pattern: re.Pattern[str] | None = None) -> str:
    if type(value) is not str or not value:
        raise ProfileError(label + " must be a non-empty string")
    if pattern and not pattern.fullmatch(value):
        raise ProfileError(label + " has an invalid format")
    return value


def _integer(value: Any, label: str, minimum: int, maximum: int | None = None) -> int:
    if type(value) is not int or value < minimum or (maximum is not None and value > maximum):
        raise ProfileError(label + " is outside its permitted range")
    return value


def _sha(value: Any, label: str) -> str:
    return _text(value, label, _SHA256)


def _image_id(value: Any) -> str:
    text = _text(value, "artifacts.imageId")
    if not text.startswith("sha256:") or not _SHA256.fullmatch(text[7:]):
        raise ProfileError("artifacts.imageId has an invalid format")
    return text


def _semver_greater_than_baseline(value: Any) -> str:
    text = _text(value, "release.version", _SEMVER)
    match = _SEMVER.fullmatch(text)
    assert match is not None
    if tuple(map(int, match.group(1, 2, 3))) <= (6, 1, 0):
        raise ProfileError("release.version must be newer than v6.1.0")
    return text


def _expected_paths(candidate_id: str) -> tuple[str, str, str, str]:
    work = "/mnt/d/Reader-Pro-Candidate-" + candidate_id
    mount = "/mnt/reader-candidate-" + candidate_id
    return work, mount, work + "/image.ext4", work + "/data.ext4"


def _strict_posix_path(value: Any, label: str, expected: str) -> str:
    text = _text(value, label)
    path = PurePosixPath(text)
    if ("\\" in text or "//" in text or not path.is_absolute() or
            any(part in ("", ".", "..") for part in path.parts) or text != expected):
        raise ProfileError(label + " is not the canonical candidate-derived path")
    return text


def _identity(candidate_id: str, value: Any, label: str, expected: str) -> str:
    text = _text(value, label)
    if text != expected or candidate_id not in text:
        raise ProfileError(label + " is not derived from candidateId")
    return text


def validate_profile(value: Any) -> CandidateProfile:
    """Validate schema and identity only; this function never touches artifacts."""
    root = _object(value, {"schemaVersion", "candidateId", "release", "artifacts", "paths", "runtime", "disks", "logging"}, "profile")
    if type(root["schemaVersion"]) is not int or root["schemaVersion"] != SCHEMA_VERSION:
        raise ProfileError("unsupported schemaVersion")
    candidate_id = _text(root["candidateId"], "candidateId", _ID)
    release = _object(root["release"], {"version", "revision"}, "release")
    version = _semver_greater_than_baseline(release["version"])
    revision = _text(release["revision"], "release.revision", _REVISION)
    artifacts = _object(root["artifacts"], {"imageId", "archiveSha256", "readerJarSha256", "licenseJarSha256", "softwareManifestSha256"}, "artifacts")
    paths = _object(root["paths"], {"workDir", "mountRoot", "imageDisk", "dataDisk"}, "paths")
    expected_work, expected_mount, expected_image, expected_data = _expected_paths(candidate_id)
    work = _strict_posix_path(paths["workDir"], "paths.workDir", expected_work)
    mount = _strict_posix_path(paths["mountRoot"], "paths.mountRoot", expected_mount)
    image_disk = _strict_posix_path(paths["imageDisk"], "paths.imageDisk", expected_image)
    data_disk = _strict_posix_path(paths["dataDisk"], "paths.dataDisk", expected_data)
    runtime = _object(root["runtime"], {"unit", "name", "readerPort", "licensePort", "cpuQuotaPercent", "memoryMaxBytes", "memoryHighBytes", "memorySwapMaxBytes", "tasksMax"}, "runtime")
    unit = _identity(candidate_id, runtime["unit"], "runtime.unit", "reader-candidate-" + candidate_id + ".service")
    name = _identity(candidate_id, runtime["name"], "runtime.name", "reader-candidate-" + candidate_id)
    reader_port = _integer(runtime["readerPort"], "runtime.readerPort", 1024, 65535)
    license_port = _integer(runtime["licensePort"], "runtime.licensePort", 1024, 65535)
    if reader_port == license_port or {reader_port, license_port} & RESERVED_PORTS:
        raise ProfileError("runtime ports must be distinct isolated non-legacy ports")
    cpu = _integer(runtime["cpuQuotaPercent"], "runtime.cpuQuotaPercent", 1)
    memory_max = _integer(runtime["memoryMaxBytes"], "runtime.memoryMaxBytes", 1)
    memory_high = _integer(runtime["memoryHighBytes"], "runtime.memoryHighBytes", 1)
    memory_swap = _integer(runtime["memorySwapMaxBytes"], "runtime.memorySwapMaxBytes", 0)
    tasks = _integer(runtime["tasksMax"], "runtime.tasksMax", 1, 512)
    if (cpu != CPU_QUOTA_PERCENT or memory_max != MEMORY_MAX_BYTES or
            memory_swap != MEMORY_SWAP_MAX_BYTES or memory_high > memory_max):
        raise ProfileError("runtime budget does not match the fixed 2 CPU / 2 GiB candidate limit")
    disks = _object(root["disks"], {"imageBytes", "dataBytes"}, "disks")
    image_bytes = _integer(disks["imageBytes"], "disks.imageBytes", 1024 ** 3, 64 * 1024 ** 3)
    data_bytes = _integer(disks["dataBytes"], "disks.dataBytes", 1024 ** 3, 64 * 1024 ** 3)
    logging = _object(root["logging"], {"currentMaxBytes", "previousFiles", "outerMaxBytes", "outerRetainedFiles"}, "logging")
    current = _integer(logging["currentMaxBytes"], "logging.currentMaxBytes", 65536, 64 * 1024 ** 2)
    previous = _integer(logging["previousFiles"], "logging.previousFiles", 1, 16)
    outer = _integer(logging["outerMaxBytes"], "logging.outerMaxBytes", 65536, 64 * 1024 ** 2)
    retained = _integer(logging["outerRetainedFiles"], "logging.outerRetainedFiles", 1, 32)
    forbidden = (OLD_WORK_DIR, OLD_MOUNT_ROOT, OLD_UNIT, OLD_NAME)
    if any(item in (work, mount, unit, name) for item in forbidden):
        raise ProfileError("candidate profile overlaps the old v6.1.0 instance")
    return CandidateProfile(candidate_id, version, revision, _image_id(artifacts["imageId"]),
                            _sha(artifacts["archiveSha256"], "artifacts.archiveSha256"),
                            _sha(artifacts["readerJarSha256"], "artifacts.readerJarSha256"),
                            _sha(artifacts["licenseJarSha256"], "artifacts.licenseJarSha256"),
                            _sha(artifacts["softwareManifestSha256"], "artifacts.softwareManifestSha256"),
                            work, mount, image_disk, data_disk, unit, name, reader_port, license_port,
                            image_bytes, data_bytes, cpu, memory_max, memory_high, memory_swap, tasks,
                            current, previous, outer, retained)


def load_profile(path: Path) -> CandidateProfile:
    """Read one regular, non-symlink JSON profile; no directory is created."""
    descriptor, _size = _open_regular_no_follow(path, 1024 * 1024, "profile")
    with os.fdopen(descriptor, "rb") as stream:
        return parse_profile_json(stream.read())


def profile_to_dict(profile: CandidateProfile) -> dict[str, Any]:
    """Canonical JSON-safe representation for receipts; callers must not mutate profile."""
    return {
        "schemaVersion": SCHEMA_VERSION, "candidateId": profile.candidate_id,
        "release": {"version": profile.version, "revision": profile.revision},
        "artifacts": {"imageId": profile.image_id, "archiveSha256": profile.archive_sha256,
                      "readerJarSha256": profile.reader_jar_sha256, "licenseJarSha256": profile.license_jar_sha256,
                      "softwareManifestSha256": profile.software_manifest_sha256},
        "paths": {"workDir": profile.work_dir, "mountRoot": profile.mount_root,
                  "imageDisk": profile.image_disk, "dataDisk": profile.data_disk},
        "runtime": {"unit": profile.unit, "name": profile.name, "readerPort": profile.reader_port,
                    "licensePort": profile.license_port, "cpuQuotaPercent": profile.cpu_quota_percent,
                    "memoryMaxBytes": profile.memory_max_bytes, "memoryHighBytes": profile.memory_high_bytes,
                    "memorySwapMaxBytes": profile.memory_swap_max_bytes, "tasksMax": profile.tasks_max},
        "disks": {"imageBytes": profile.image_bytes, "dataBytes": profile.data_bytes},
        "logging": {"currentMaxBytes": profile.log_current_max_bytes, "previousFiles": profile.log_previous_files,
                    "outerMaxBytes": profile.outer_log_max_bytes, "outerRetainedFiles": profile.outer_log_retained_files},
    }


def verify_profile_paths(profile: CandidateProfile) -> None:
    """Reject an existing symlink in a candidate-owned path, without creating it.

    This is deliberately separate from parsing: a profile can be reviewed on
    Windows before its WSL mount paths exist.  A mutating controller calls it
    immediately before every filesystem action.
    """
    for raw in (profile.work_dir, profile.mount_root, profile.image_disk, profile.data_disk):
        # The direct host probe also makes this guard useful in tests and for
        # callers that have translated the approved WSL path to a host path.
        if os.path.lexists(raw) and Path(raw).is_symlink():
            raise ProfileError("candidate path contains a symlink")
        current = Path("/")
        for part in PurePosixPath(raw).parts[1:]:
            current /= part
            if not os.path.lexists(current):
                break
            if current.is_symlink():
                raise ProfileError("candidate path contains a symlink")


def _open_regular_no_follow(path: Path, maximum: int | None, label: str) -> tuple[int, int]:
    """Open a stable regular file without accepting a checked-path swap."""
    before = path.lstat()
    if (not stat.S_ISREG(before.st_mode) or path.is_symlink() or before.st_nlink != 1 or
            (maximum is not None and before.st_size > maximum)):
        raise ProfileError(label + " must be a single-link regular non-symlink file")
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
    try:
        opened, after = os.fstat(descriptor), path.lstat()
        if ((before.st_dev, before.st_ino, before.st_size) != (opened.st_dev, opened.st_ino, opened.st_size) or
                (before.st_dev, before.st_ino, before.st_size) != (after.st_dev, after.st_ino, after.st_size) or
                not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1):
            raise ProfileError(label + " changed while opened")
        return descriptor, opened.st_size
    except BaseException:
        os.close(descriptor)
        raise


def sha256_regular_file(path: Path) -> str:
    descriptor, _size = _open_regular_no_follow(path, None, "artifact")
    digest = hashlib.sha256()
    with os.fdopen(descriptor, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_artifact_files(profile: CandidateProfile, *, archive: Path, reader_jar: Path,
                          license_jar: Path, software_manifest: Path,
                          observed_image_id: str) -> None:
    """Compare independently measured artifacts with the immutable profile.

    ``observed_image_id`` must come from an OCI parser or other verifier.  It
    is intentionally not derived from any profile value: an archive hash alone
    cannot prove an OCI image identity.
    """
    measured = {
        "archive": sha256_regular_file(archive), "reader": sha256_regular_file(reader_jar),
        "license": sha256_regular_file(license_jar), "software": sha256_regular_file(software_manifest),
    }
    expected = {"archive": profile.archive_sha256, "reader": profile.reader_jar_sha256,
                "license": profile.license_jar_sha256, "software": profile.software_manifest_sha256}
    if measured != expected:
        raise ProfileError("artifact hash does not match candidate profile")
    if observed_image_id != profile.image_id:
        raise ProfileError("independently observed image identity does not match candidate profile")
