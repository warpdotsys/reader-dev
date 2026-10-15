#!/usr/bin/env python3
"""Offline, fail-closed backup and restore for one explicit Reader candidate.

This deliberately does not know any historical Reader path.  It derives all
paths and the unit solely from the validated candidate profile, and is intended
to be called by the candidate lifecycle tool only while the corresponding
systemd unit is stopped; it verifies that fact rather than trusting a receipt.

The format is a directory snapshot with a canonical JSON manifest.  It is not
an archive format: avoiding archive extraction removes a surprisingly large
path traversal surface and makes a failed restore easy to inspect.
"""
from __future__ import annotations

import argparse
import contextlib
import dataclasses
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import secrets
import stat
import subprocess
import sys
import time
from typing import Callable, Iterator

try:  # The command is Linux-only, but importing it for static inspection is safe.
    import fcntl
except ImportError:  # pragma: no cover - non-Linux command path fails closed.
    fcntl = None  # type: ignore[assignment]

MAX_FILES = 250_000
MAX_FILE_BYTES = 8 * 1024 * 1024 * 1024
MAX_TOTAL_BYTES = 256 * 1024 * 1024 * 1024
MAX_MANIFEST_BYTES = 96 * 1024 * 1024
CHUNK = 1024 * 1024
ROOT_NAMES = ("storage", "license-data", "secrets")
PROFILE_MODULE_PATH = Path(__file__).with_name("candidate-profile.py")
PROFILE_SPEC = importlib.util.spec_from_file_location("reader_candidate_profile", PROFILE_MODULE_PATH)
if PROFILE_SPEC is None or PROFILE_SPEC.loader is None:
    raise RuntimeError("candidate profile validator is unavailable")
candidate_profile = importlib.util.module_from_spec(PROFILE_SPEC)
sys.modules[PROFILE_SPEC.name] = candidate_profile
PROFILE_SPEC.loader.exec_module(candidate_profile)


class BackupError(RuntimeError):
    pass


@dataclasses.dataclass(frozen=True)
class Profile:
    candidate_id: str
    data_root: Path
    roots: dict[str, Path]
    backups_root: Path
    service_unit: str
    lifecycle_lock: Path
    profile_sha256: str


def _json_load_regular(path: Path, maximum: int = MAX_MANIFEST_BYTES) -> object:
    value = path.lstat()
    if not stat.S_ISREG(value.st_mode) or value.st_nlink != 1 or value.st_size > maximum:
        raise BackupError("metadata must be a bounded, non-linked regular file")
    def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
        output: dict[str, object] = {}
        for key, value in pairs:
            if key in output:
                raise BackupError("duplicate JSON key")
            output[key] = value
        return output
    try:
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle, object_pairs_hook=reject_duplicates,
                             parse_constant=lambda _value: (_ for _ in ()).throw(BackupError("non-finite JSON")))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise BackupError("invalid JSON metadata") from exc


def _profile_sha(value: object) -> str:
    return hashlib.sha256(_canonical_json(value)).hexdigest()


def from_candidate_profile(value: object) -> Profile:
    """Derive every mutable path and the unit solely from CandidateProfile."""
    if not isinstance(value, candidate_profile.CandidateProfile):
        raise BackupError("candidate profile validator did not return CandidateProfile")
    canonical = candidate_profile.profile_to_dict(value)
    data_root = Path(value.mount_root) / "data"
    roots = {name: data_root / name for name in ROOT_NAMES}
    # This is candidate-derived but outside the mounted data tree, so snapshots
    # cannot be accidentally included in the next snapshot.
    backups_root = Path(value.work_dir) / "full-backups"
    return Profile(value.candidate_id, data_root, roots, backups_root, value.unit,
                   Path(value.work_dir) / "lifecycle.lock", _profile_sha(canonical))


def load_profile(path: Path) -> Profile:
    try:
        return from_candidate_profile(candidate_profile.load_profile(path))
    except candidate_profile.ProfileError as exc:
        raise BackupError("candidate profile rejected: " + str(exc)) from exc


def _safe_dir(path: Path) -> os.stat_result:
    item = path.lstat()
    if stat.S_ISLNK(item.st_mode) or not stat.S_ISDIR(item.st_mode):
        raise BackupError("unsafe directory: %s" % path)
    return item


def _safe_ancestry(path: Path) -> None:
    """Reject redirected profile paths before treating their endpoints as roots."""
    current = path
    while True:
        value = current.lstat()
        if stat.S_ISLNK(value.st_mode) or not stat.S_ISDIR(value.st_mode):
            raise BackupError("profile path has a symlink or non-directory ancestor")
        if current == current.parent:
            return
        current = current.parent


def _validate_profile_roots(profile: Profile, *, allow_missing_roots: bool = False) -> None:
    _safe_ancestry(profile.data_root)
    _safe_ancestry(profile.backups_root)
    data = _safe_dir(profile.data_root)
    _safe_dir(profile.backups_root)
    for root in profile.roots.values():
        _safe_ancestry(root.parent)
        try:
            root_stat = _safe_dir(root)
        except FileNotFoundError:
            if allow_missing_roots:
                continue
            raise BackupError("candidate root is absent")
        if root_stat.st_dev != data.st_dev:
            raise BackupError("candidate root is a cross-mount attachment")


def _assert_runtime_scope(fields: dict[str, str]) -> None:
    expected = {
        "ActiveState": "inactive",
        "SubState": "dead",
        "MainPID": "0",
    }
    if fields != expected:
        raise BackupError("service is not proven inactive in the system manager")


def systemd_probe(unit: str) -> dict[str, str]:
    try:
        result = subprocess.run(
            ["systemctl", "--system", "show", unit, "--no-page", "--property=ActiveState", "--property=SubState", "--property=MainPID"],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            timeout=5, check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise BackupError("cannot query systemd service state") from exc
    if result.returncode != 0 or result.stderr or len(result.stdout) > 1024:
        raise BackupError("cannot verify systemd service state")
    try:
        fields = dict(line.split("=", 1) for line in result.stdout.decode("ascii").splitlines() if line)
    except ValueError as exc:
        raise BackupError("invalid systemd service response") from exc
    _assert_runtime_scope(fields)
    return fields


def systemd_runtime_lock_check(profile: Profile) -> None:
    """Fail closed until the candidate controller emits a verifiable receipt.

    RuntimeScope is not a unit property on supported systemd versions.  A
    future adapter must prove both the shared flock lifetime and invocation of
    ``startup_guard``; rendered ExecStart text alone is intentionally rejected.
    """
    # Deliberately reject until the candidate controller supplies a verifiable
    # launcher receipt.  Matching words in systemd's rendered ExecStart is not
    # proof that the process holds the lock or invokes startup_guard.
    raise BackupError(
        "candidate lifecycle shared-lock/startup-guard integration is not yet verified; "
        "offline execute is fail-closed")


@contextlib.contextmanager
def maintenance_lock(profile: Profile) -> Iterator[None]:
    """Exclusive side of the candidate lifecycle lock protocol.

    The candidate unit must hold a shared flock on this exact path for its
    whole runtime.  This maintenance side is not by itself an authorization to
    race a future ``systemctl start``; executing calls check unit integration.
    """
    if fcntl is None:
        raise BackupError("Linux advisory locking is unavailable")
    _safe_ancestry(profile.lifecycle_lock.parent)
    lock = profile.lifecycle_lock
    flags = os.O_RDWR | os.O_CREAT
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(str(lock), flags, 0o600)
    except OSError as exc:
        raise BackupError("cannot open candidate maintenance lock") from exc
    try:
        value = os.fstat(descriptor)
        if not stat.S_ISREG(value.st_mode) or value.st_nlink != 1:
            raise BackupError("maintenance lock is unsafe")
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as exc:
            raise BackupError("candidate maintenance is already locked") from exc
        yield
    finally:
        try:
            os.close(descriptor)
        except OSError:
            pass


def _safe_name(name: str) -> None:
    if not name or name in {".", ".."} or "/" in name or "\\" in name or "\x00" in name:
        raise BackupError("unsafe filesystem entry name")


def _entry_path(root_name: str, relative: PurePosixPath) -> str:
    return root_name if str(relative) == "." else root_name + "/" + relative.as_posix()


def _walk_root(root_name: str, root: Path) -> list[dict[str, object]]:
    root_stat = _safe_dir(root)
    device = root_stat.st_dev
    entries: list[dict[str, object]] = [{
        "path": root_name, "mode": stat.S_IMODE(root_stat.st_mode),
        "uid": root_stat.st_uid, "gid": root_stat.st_gid,
        "type": "directory", "size": 0, "sha256": None,
    }]
    total = 0

    def visit(directory: Path, relative: PurePosixPath) -> None:
        nonlocal total
        directory_stat = _safe_dir(directory)
        if directory_stat.st_dev != device:
            raise BackupError("cross-mount directory rejected")
        try:
            children = sorted(os.scandir(directory), key=lambda item: item.name)
        except OSError as exc:
            raise BackupError("cannot enumerate source directory") from exc
        for child in children:
            _safe_name(child.name)
            child_path = directory / child.name
            value = child_path.lstat()
            if value.st_dev != device:
                raise BackupError("cross-mount entry rejected")
            child_relative = relative / child.name
            mode = stat.S_IMODE(value.st_mode)
            base = {"path": _entry_path(root_name, child_relative), "mode": mode,
                    "uid": value.st_uid, "gid": value.st_gid}
            if stat.S_ISDIR(value.st_mode):
                entries.append({**base, "type": "directory", "size": 0, "sha256": None})
                visit(child_path, child_relative)
            elif stat.S_ISREG(value.st_mode):
                if value.st_nlink != 1:
                    raise BackupError("hard-linked source file rejected")
                if value.st_size > MAX_FILE_BYTES:
                    raise BackupError("source file exceeds maximum")
                total += value.st_size
                if total > MAX_TOTAL_BYTES:
                    raise BackupError("source data exceeds total maximum")
                entries.append({**base, "type": "file", "size": value.st_size,
                                "sha256": None, "_source": child_path, "_stat": value})
            else:
                raise BackupError("symlink or special source file rejected")
            if len(entries) > MAX_FILES:
                raise BackupError("source has too many entries")
    visit(root, PurePosixPath("."))
    return entries


def _open_source(path: Path, expected: os.stat_result) -> int:
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    descriptor = os.open(str(path), flags)
    value = os.fstat(descriptor)
    if (not stat.S_ISREG(value.st_mode) or value.st_nlink != 1 or
            (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns) !=
            (expected.st_dev, expected.st_ino, expected.st_size, expected.st_mtime_ns)):
        os.close(descriptor)
        raise BackupError("source changed or became unsafe during backup")
    return descriptor


def _mkdir(path: Path, mode: int, uid: int, gid: int) -> None:
    path.mkdir(mode=mode)
    os.chmod(path, mode)
    if (os.geteuid(), os.getegid()) != (uid, gid):
        os.chown(path, uid, gid)


def _copy_file(source: Path, source_stat: os.stat_result, target: Path, entry: dict[str, object]) -> str:
    descriptor = _open_source(source, source_stat)
    out_flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_NOFOLLOW"):
        out_flags |= os.O_NOFOLLOW
    out = os.open(str(target), out_flags, int(entry["mode"]))
    digest = hashlib.sha256()
    count = 0
    try:
        while True:
            block = os.read(descriptor, CHUNK)
            if not block:
                break
            count += len(block)
            if count > int(entry["size"]):
                raise BackupError("source grew during backup")
            digest.update(block)
            offset = 0
            while offset < len(block):
                written = os.write(out, block[offset:])
                if written <= 0:
                    raise BackupError("short write while creating snapshot")
                offset += written
        if count != int(entry["size"]):
            raise BackupError("source shrank during backup")
        os.fsync(out)
    finally:
        os.close(descriptor)
        os.close(out)
    after = source.lstat()
    if (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) != (source_stat.st_dev, source_stat.st_ino, source_stat.st_size, source_stat.st_mtime_ns):
        raise BackupError("source changed during backup")
    os.chmod(target, int(entry["mode"]))
    if (os.geteuid(), os.getegid()) != (int(entry["uid"]), int(entry["gid"])):
        os.chown(target, int(entry["uid"]), int(entry["gid"]))
    metadata_fd = os.open(str(target), os.O_RDONLY)
    try:
        os.fsync(metadata_fd)
    finally:
        os.close(metadata_fd)
    return digest.hexdigest()


def _canonical_json(value: object) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def _write_manifest(snapshot: Path, manifest: dict[str, object]) -> None:
    destination = snapshot / "manifest.json"
    data = _canonical_json(manifest)
    descriptor = os.open(str(destination), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        os.write(descriptor, data)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _safe_snapshot_name(profile: Profile) -> str:
    return "%s-%s-%s" % (profile.candidate_id, time.strftime("%Y%m%dT%H%M%SZ", time.gmtime()), secrets.token_hex(6))


def plan(profile: Profile) -> dict[str, object]:
    _validate_profile_roots(profile)
    entries = []
    for name in ROOT_NAMES:
        entries.extend(_walk_root(name, profile.roots[name]))
    total = sum(int(item["size"]) for item in entries if item["type"] == "file")
    return {"candidate_id": profile.candidate_id, "profile_sha256": profile.profile_sha256, "roots": list(ROOT_NAMES),
            "files_and_directories": len(entries), "total_bytes": total,
            "backup_root": str(profile.backups_root)}


def backup(profile: Profile, execute: bool, probe: Callable[[str], dict[str, str]] = systemd_probe,
           runtime_lock_check: Callable[[Profile], None] = systemd_runtime_lock_check) -> dict[str, object]:
    result = plan(profile)
    result["operation"] = "backup"
    result["executed"] = False
    if not execute:
        return result
    with maintenance_lock(profile):
        _assert_runtime_scope(probe(profile.service_unit))
        runtime_lock_check(profile)
        if _read_journal(profile) is not None:
            raise BackupError("unfinished transaction exists; run recover --execute before backup")
        # Re-enumerate under the lock after service state has been proven.
        collections = {name: _walk_root(name, profile.roots[name]) for name in ROOT_NAMES}
        name = _safe_snapshot_name(profile)
        temporary = profile.backups_root / ("." + name + ".incomplete")
        final = profile.backups_root / name
        if temporary.exists() or final.exists():
            raise BackupError("snapshot collision")
        temporary.mkdir(mode=0o700)
        try:
            final_entries: list[dict[str, object]] = []
            for root_name in ROOT_NAMES:
                for raw in collections[root_name]:
                    entry = {key: value for key, value in raw.items() if key not in {"_source", "_stat"}}
                    _root, rel = _relative_from_entry(entry["path"])
                    target = temporary / root_name / Path(*rel.parts)
                    if entry["type"] == "directory":
                        _mkdir(target, int(entry["mode"]), int(entry["uid"]), int(entry["gid"]))
                    else:
                        entry["sha256"] = _copy_file(raw["_source"], raw["_stat"], target, entry)  # type: ignore[arg-type]
                    final_entries.append(entry)
            manifest = {"format": 2, "candidate_id": profile.candidate_id,
                        "profile_sha256": profile.profile_sha256,
                        "roots": list(ROOT_NAMES), "entries": final_entries}
            _write_manifest(temporary, manifest)
            _fsync_tree(temporary)
            os.replace(temporary, final)
            parent_fd = os.open(str(profile.backups_root), os.O_RDONLY)
            try:
                os.fsync(parent_fd)
            finally:
                os.close(parent_fd)
        except Exception:
            # Intentionally retain the incomplete tree for forensic inspection;
            # it is never a valid restore input because it lacks a final name.
            raise
    result.update({"executed": True, "snapshot": str(final), "manifest_sha256": hashlib.sha256(_canonical_json(manifest)).hexdigest()})
    return result


def _relative_from_entry(value: object) -> tuple[str, PurePosixPath]:
    if not isinstance(value, str) or value.startswith("/") or "\\" in value or "\x00" in value:
        raise BackupError("manifest path unsafe")
    if value in ROOT_NAMES:
        return value, PurePosixPath(".")
    if "/" not in value:
        raise BackupError("manifest path unsafe")
    root, remaining = value.split("/", 1)
    if root not in ROOT_NAMES or not remaining:
        raise BackupError("manifest root unsafe")
    relative = PurePosixPath(remaining)
    if relative.is_absolute() or any(part in {"", ".", ".."} for part in relative.parts):
        raise BackupError("manifest path escaped")
    return root, relative


def _load_manifest(snapshot: Path, profile: Profile) -> dict[str, object]:
    _safe_dir(snapshot)
    if snapshot.parent != profile.backups_root or snapshot.name.startswith("."):
        raise BackupError("snapshot is outside the explicit backup root")
    manifest = _json_load_regular(snapshot / "manifest.json")
    if not isinstance(manifest, dict) or manifest.get("format") != 2 or manifest.get("candidate_id") != profile.candidate_id or manifest.get("profile_sha256") != profile.profile_sha256 or manifest.get("roots") != list(ROOT_NAMES):
        raise BackupError("manifest identity does not match candidate")
    entries = manifest.get("entries")
    if not isinstance(entries, list) or len(entries) > MAX_FILES:
        raise BackupError("manifest entries invalid")
    seen = set()
    total = 0
    for entry in entries:
        if not isinstance(entry, dict):
            raise BackupError("manifest entry invalid")
        root, relative = _relative_from_entry(entry.get("path"))
        key = (root, relative.as_posix())
        if key in seen:
            raise BackupError("manifest duplicate path")
        seen.add(key)
        kind = entry.get("type")
        if kind not in {"file", "directory"} or not isinstance(entry.get("mode"), int) or not isinstance(entry.get("uid"), int) or not isinstance(entry.get("gid"), int):
            raise BackupError("manifest metadata invalid")
        if kind == "file":
            if not isinstance(entry.get("size"), int) or not isinstance(entry.get("sha256"), str) or len(entry["sha256"]) != 64 or entry["size"] < 0 or entry["size"] > MAX_FILE_BYTES:
                raise BackupError("manifest file metadata invalid")
            total += entry["size"]
        elif entry.get("size") != 0 or entry.get("sha256") is not None:
            raise BackupError("manifest directory metadata invalid")
    if total > MAX_TOTAL_BYTES:
        raise BackupError("manifest total exceeds maximum")
    return manifest


def _verify_snapshot(snapshot: Path, manifest: dict[str, object]) -> None:
    expected = {"manifest.json", *ROOT_NAMES}
    snapshot_device = _safe_dir(snapshot).st_dev
    entries = manifest["entries"]  # validated by _load_manifest
    for entry in entries:  # type: ignore[union-attr]
        root, rel = _relative_from_entry(entry["path"])
        if str(rel) != ".":
            expected.add(root + "/" + rel.as_posix())
        source = snapshot / root / Path(*rel.parts)
        value = source.lstat()
        if value.st_dev != snapshot_device:
            raise BackupError("snapshot contains a cross-mount entry")
        if stat.S_ISLNK(value.st_mode) or (entry["type"] == "file" and (not stat.S_ISREG(value.st_mode) or value.st_nlink != 1)) or (entry["type"] == "directory" and not stat.S_ISDIR(value.st_mode)):
            raise BackupError("snapshot entry type unsafe")
        if stat.S_IMODE(value.st_mode) != entry["mode"] or value.st_uid != entry["uid"] or value.st_gid != entry["gid"]:
            raise BackupError("snapshot permissions changed")
        if entry["type"] == "file":
            if value.st_size != entry["size"]:
                raise BackupError("snapshot size changed")
            digest = hashlib.sha256()
            with source.open("rb") as input_file:
                for block in iter(lambda: input_file.read(CHUNK), b""):
                    digest.update(block)
            if digest.hexdigest() != entry["sha256"]:
                raise BackupError("snapshot hash changed")
    actual = set()
    for base, directories, files in os.walk(snapshot, followlinks=False):
        for filename in directories + files:
            point = Path(base) / filename
            relative = point.relative_to(snapshot).as_posix()
            value = point.lstat()
            if value.st_dev != snapshot_device:
                raise BackupError("snapshot contains a cross-mount entry")
            if stat.S_ISLNK(value.st_mode) or not (stat.S_ISDIR(value.st_mode) or stat.S_ISREG(value.st_mode)):
                raise BackupError("snapshot has unsafe extra entry")
            actual.add(relative)
    if actual != expected:
        raise BackupError("snapshot manifest collection is not exact")


def _stage_restore(snapshot: Path, manifest: dict[str, object], profile: Profile, stage: Path) -> None:
    stage.mkdir(mode=0o700)
    ordered = sorted(manifest["entries"], key=lambda e: (str(e["path"]).count("/"), str(e["path"])))  # type: ignore[index]
    for entry in ordered:
        root, rel = _relative_from_entry(entry["path"])
        target = stage / root / Path(*rel.parts)
        source = snapshot / root / Path(*rel.parts)
        if entry["type"] == "directory":
            _mkdir(target, int(entry["mode"]), int(entry["uid"]), int(entry["gid"]))
        else:
            source_stat = source.lstat()
            got = _copy_file(source, source_stat, target, entry)
            if got != entry["sha256"]:
                raise BackupError("staged file hash differs from manifest")
    # Verify the stage using the same manifest checker after a private manifest
    # copy, before any live candidate root is renamed.
    _write_manifest(stage, manifest)
    _verify_snapshot(stage, manifest)
    (stage / "manifest.json").unlink()
    _fsync_tree(stage)


def _fsync_directory(path: Path) -> None:
    descriptor = os.open(str(path), os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _fsync_tree(path: Path) -> None:
    """Persist metadata after chmod/chown and before publishing a stage."""
    for base, directories, _files in os.walk(path, topdown=False, followlinks=False):
        for name in directories:
            value = Path(base) / name
            if value.is_symlink():
                raise BackupError("cannot fsync redirected transaction directory")
        _fsync_directory(Path(base))


def _journal_path(profile: Profile) -> Path:
    return profile.data_root / ".full-backup-transaction.json"


def _write_journal(profile: Profile, value: dict[str, object]) -> None:
    """Durably publish a transition intent before its associated rename."""
    target = _journal_path(profile)
    temporary = profile.data_root / ("." + target.name + ".new")
    if temporary.exists():
        raise BackupError("stale transaction journal temporary exists")
    payload = _canonical_json(value)
    descriptor = os.open(str(temporary), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        os.write(descriptor, payload)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    os.replace(temporary, target)
    _fsync_directory(profile.data_root)


def _read_journal(profile: Profile) -> dict[str, object] | None:
    target = _journal_path(profile)
    if not os.path.lexists(target):
        return None
    raw = _json_load_regular(target, 256 * 1024)
    required = {"format", "candidate_id", "profile_sha256", "snapshot", "stage", "quarantine", "phase"}
    if not isinstance(raw, dict) or set(raw) != required or raw.get("format") != 1 or raw.get("candidate_id") != profile.candidate_id or raw.get("profile_sha256") != profile.profile_sha256:
        raise BackupError("transaction journal identity is invalid")
    for key in ("stage", "quarantine", "snapshot"):
        if not isinstance(raw.get(key), str):
            raise BackupError("transaction journal is incomplete")
    phase = raw.get("phase")
    allowed = {"staged", "committed"} | {prefix + name for prefix in ("move-old:", "install-new:") for name in ROOT_NAMES}
    if phase not in allowed:
        raise BackupError("transaction journal phase is invalid")
    return raw


def _same_child(parent: Path, raw: str, prefix: str) -> Path:
    value = Path(raw)
    if value.parent != parent or not value.name.startswith(prefix) or value.is_symlink():
        raise BackupError("transaction journal path escaped candidate data root")
    return value


def _transaction_preflight(profile: Profile, stage: Path, quarantine: Path) -> None:
    """Validate every present transition root before moving any of them."""
    data_device = _safe_dir(profile.data_root).st_dev
    for directory in (stage, quarantine):
        value = _safe_dir(directory)
        if value.st_dev != data_device:
            raise BackupError("transaction directory crossed candidate data mount")
    for name in ROOT_NAMES:
        for point in (profile.roots[name], stage / name, quarantine / name):
            if not os.path.lexists(point):
                continue
            value = point.lstat()
            if stat.S_ISLNK(value.st_mode) or not stat.S_ISDIR(value.st_mode) or value.st_dev != data_device:
                raise BackupError("transaction root is unsafe before recovery: " + name)


def recover_transaction(profile: Profile, execute: bool) -> dict[str, object]:
    """Idempotently roll back an interrupted three-root restore transaction.

    A journal is an admission guard for the future candidate controller too:
    its startup path must refuse to serve while this file exists, then invoke
    this explicit maintenance recovery under the exclusive lifecycle lock.
    """
    _validate_profile_roots(profile, allow_missing_roots=True)
    journal = _read_journal(profile)
    if journal is None:
        return {"operation": "recover", "candidate_id": profile.candidate_id, "executed": False, "pending": False}
    stage = _same_child(profile.data_root, journal["stage"], ".restore-")
    quarantine = _same_child(profile.data_root, journal["quarantine"], ".quarantine-")
    if stage.name[len(".restore-"):] != quarantine.name[len(".quarantine-"):]:
        raise BackupError("transaction journal stage/quarantine token mismatch")
    result = {"operation": "recover", "candidate_id": profile.candidate_id, "executed": False,
              "pending": True, "stage": str(stage), "quarantine": str(quarantine)}
    if not execute:
        return result
    if not stage.is_dir() or not quarantine.is_dir():
        raise BackupError("interrupted transaction directories are missing")
    _transaction_preflight(profile, stage, quarantine)
    # This state table is independent of the last journal write: a SIGKILL can
    # happen after a rename but before the following durable journal update.
    for name in ROOT_NAMES:
        live, old, staged = profile.roots[name], quarantine / name, stage / name
        present = (os.path.lexists(live), os.path.lexists(old), os.path.lexists(staged))
        if present == (True, False, True):
            continue                         # no old-root move occurred
        if present == (False, True, True):
            os.replace(old, live)             # old root was moved, new not installed
            _fsync_directory(profile.data_root)
            _fsync_directory(quarantine)
            continue
        if present == (True, True, False):
            os.replace(live, staged)          # new root installed; preserve it
            _fsync_directory(profile.data_root)
            _fsync_directory(stage)
            os.replace(old, live)              # restore the complete old root
            _fsync_directory(profile.data_root)
            _fsync_directory(quarantine)
            continue
        raise BackupError("interrupted transaction has ambiguous root state: " + name)
    _validate_profile_roots(profile)
    # Keep stage/quarantine for forensic rollback.  Only remove the admission
    # journal after all original roots are back and the parent is durable.
    _journal_path(profile).unlink()
    _fsync_directory(profile.data_root)
    result["executed"] = True
    return result


def startup_guard(profile: Profile) -> None:
    """Candidate runtime integration point: refuse start on unfinished restore."""
    if _read_journal(profile) is not None:
        raise BackupError("unfinished full-backup transaction requires offline recovery")


def restore(profile: Profile, snapshot: Path, execute: bool, probe: Callable[[str], dict[str, str]] = systemd_probe,
            runtime_lock_check: Callable[[Profile], None] = systemd_runtime_lock_check,
            failpoint: Callable[[str], None] | None = None) -> dict[str, object]:
    _validate_profile_roots(profile)
    manifest = _load_manifest(snapshot, profile)
    _verify_snapshot(snapshot, manifest)
    result = {"operation": "restore", "candidate_id": profile.candidate_id,
              "snapshot": str(snapshot), "executed": False}
    if not execute:
        return result
    with maintenance_lock(profile):
        _assert_runtime_scope(probe(profile.service_unit))
        runtime_lock_check(profile)
        if _read_journal(profile) is not None:
            raise BackupError("unfinished transaction exists; run recover --execute before restore")
        # Revalidate immediately before changing anything.
        _verify_snapshot(snapshot, manifest)
        token = "%s-%s-%s" % (profile.candidate_id, time.strftime("%Y%m%dT%H%M%SZ", time.gmtime()), secrets.token_hex(6))
        stage = profile.data_root / (".restore-" + token)
        quarantine = profile.data_root / (".quarantine-" + token)
        if stage.exists() or quarantine.exists():
            raise BackupError("restore transaction collision")
        _stage_restore(snapshot, manifest, profile, stage)
        quarantine.mkdir(mode=0o700)
        _fsync_directory(profile.data_root)
        _transaction_preflight(profile, stage, quarantine)
        transaction = {"format": 1, "candidate_id": profile.candidate_id,
                       "profile_sha256": profile.profile_sha256, "snapshot": str(snapshot),
                       "stage": str(stage), "quarantine": str(quarantine), "phase": "staged"}
        _write_journal(profile, transaction)
        for root in ROOT_NAMES:
            transaction["phase"] = "move-old:" + root
            _write_journal(profile, transaction)
            os.replace(profile.roots[root], quarantine / root)
            _fsync_directory(profile.data_root)
            _fsync_directory(quarantine)
            if failpoint:
                failpoint("old-moved:" + root)
        for root in ROOT_NAMES:
            transaction["phase"] = "install-new:" + root
            _write_journal(profile, transaction)
            os.replace(stage / root, profile.roots[root])
            _fsync_directory(profile.data_root)
            _fsync_directory(stage)
            if failpoint:
                failpoint("new-installed:" + root)
        transaction["phase"] = "committed"
        _write_journal(profile, transaction)
        if failpoint:
            failpoint("committed")
        _journal_path(profile).unlink()
        _fsync_directory(profile.data_root)
        # A crash after journal removal leaves only an empty, candidate-owned
        # stage directory.  It is deliberately non-critical and not removed
        # here, avoiding a committed-window recovery ambiguity.
    result.update({"executed": True, "quarantine": str(quarantine)})
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--execute", action="store_true", help="perform writes; otherwise emit a validation plan")
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("backup")
    commands.add_parser("recover")
    restore_parser = commands.add_parser("restore")
    restore_parser.add_argument("--snapshot", required=True, type=Path)
    args = parser.parse_args(argv)
    if sys.platform != "linux":
        raise SystemExit("full backup/restore is Linux-only and fails closed outside Linux")
    try:
        profile = load_profile(args.profile)
        if args.command == "backup":
            result = backup(profile, args.execute)
        elif args.command == "restore":
            result = restore(profile, args.snapshot, args.execute)
        else:
            if args.execute:
                with maintenance_lock(profile):
                    _assert_runtime_scope(systemd_probe(profile.service_unit))
                    systemd_runtime_lock_check(profile)
                    result = recover_transaction(profile, True)
            else:
                result = recover_transaction(profile, False)
        print(json.dumps(result, ensure_ascii=False, sort_keys=True))
        return 0
    except BackupError as exc:
        print(json.dumps({"accepted": False, "error": str(exc)}, ensure_ascii=False), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
