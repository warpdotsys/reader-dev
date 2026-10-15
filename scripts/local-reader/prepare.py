#!/usr/bin/env python3
"""Prepare the exact v6.1.0 Reader image on explicitly-owned local disks.

This is deliberately a *preparation* tool: it never starts Reader.  Planning
and static validation are the default; filesystem changes require --execute.
The operator must supply two newly-created, marker-bearing regular files on D:
(an image disk and a persistent data disk).  Existing disks are never
reformatted by this program.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tarfile
import time
from typing import Any
from email.parser import Parser

sys.dont_write_bytecode = True

LOCK = {
    "schemaVersion": 1,
    "architecture": "amd64",
    "version": "6.1.0",
    "revision": "cc4593a1f707b30366005974f45eb1436da9b8fb",
    "archiveSha256": "70538a4f3e8b145d26ec39e8df183c347f4d470c5f07d06f73a7ccb0fd815d51",
    "imageId": "sha256:57403ef4e3d7412b049636985e4f3d2c950e3aaf94d917b21520c6c8309421d9",
    "jarSha256": "b4954ac70229f4514146b8e077909874bfc5aeee779f71fde903622638ed1ee0",
    "layerCount": 14,
}
TASK = "reader-local-full-610-20261015-a"
WORK_DIR = "/mnt/d/Reader-Pro-Local-Full-610-20261015-a"
MOUNT_ROOT = "/mnt/reader-local-full-610"
IMAGE_BYTES = 16 * 1024**3
DATA_BYTES = 4 * 1024**3
CPU_QUOTA = "200%"
MEMORY_MAX = 2 * 1024**3
MEMORY_HIGH = int(1.5 * 1024**3)
TASKS_MAX = 512


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def atomic_json(path: Path, value: dict[str, Any]) -> None:
    """Create one evidence file, never overwrite evidence from another run."""
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def require_regular(path: Path, label: str) -> os.stat_result:
    assert path.is_file() and not path.is_symlink(), f"{label} must be a regular non-symlink file"
    return path.stat()


def marker_path(disk: Path) -> Path:
    return disk.with_name(disk.name + "." + TASK + ".marker.json")


def load_owned_marker(disk: Path, purpose: str, size: int) -> dict[str, Any]:
    """Accept only a fresh, explicit, empty task disk created by the operator."""
    info = require_regular(disk, purpose + " disk")
    marker = marker_path(disk)
    assert marker.is_file() and not marker.is_symlink(), "missing task ownership marker: " + str(marker)
    value = json.loads(marker.read_text(encoding="utf-8"))
    assert value == {
        "schemaVersion": 1,
        "task": TASK,
        "purpose": purpose,
        "bytes": size,
        "state": "blank-for-first-format",
    }, "disk marker does not prove this is a new owned blank disk"
    # The Windows-side creator leaves an *empty* sparse file.  This exact zero
    # size is important: a non-empty file may contain a filesystem or user
    # data, and is never expanded or formatted by this tool.
    assert info.st_size == 0, f"{purpose} disk is not a new zero-byte owned file"
    assert disk.parent.resolve().as_posix().startswith("/mnt/d/"), "owned disks must be on D:"
    return value


def fixed_paths(args: argparse.Namespace) -> tuple[Path, Path, Path]:
    """Only the reviewed D: directory and its two explicit file names are valid."""
    work, image, data = Path(args.work_dir), Path(args.image_disk), Path(args.data_disk)
    assert str(work).replace("\\", "/") == WORK_DIR, "work-dir is fixed for this preparation"
    assert str(image).replace("\\", "/") == WORK_DIR + "/image.ext4", "image disk path is fixed"
    assert str(data).replace("\\", "/") == WORK_DIR + "/data.ext4", "data disk path is fixed"
    assert args.mount_root == MOUNT_ROOT, "mount root is fixed"
    return work, image, data


def open_empty_owned_disk(disk: Path) -> int:
    """No-follow open plus inode/size check immediately before ftruncate."""
    before = disk.lstat()
    assert stat.S_ISREG(before.st_mode) and before.st_size == 0 and not disk.is_symlink()
    descriptor = os.open(disk, os.O_RDWR | getattr(os, "O_NOFOLLOW", 0))
    try:
        after, opened = disk.lstat(), os.fstat(descriptor)
        assert (before.st_dev, before.st_ino) == (after.st_dev, after.st_ino) == (opened.st_dev, opened.st_ino), "disk path changed during open"
        assert stat.S_ISREG(opened.st_mode) and opened.st_size == 0, "disk is not a fresh empty regular file"
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def validate_config(config: dict[str, Any]) -> dict[str, Any]:
    """Pinned OCI runtime identity; copied as a pure stable verifier, not imported from build evidence."""
    assert config.get("architecture") == "amd64" and config.get("os") == "linux", "image platform mismatch"
    runtime = config.get("config", {})
    assert runtime.get("User") == "10001:10001", "image user mismatch"
    assert runtime.get("Entrypoint") == ["/usr/local/bin/reader-entrypoint"], "image entrypoint mismatch"
    assert runtime.get("WorkingDir") == "/app", "image working directory mismatch"
    expected = {
        "READER_RELEASE_VERSION": LOCK["version"], "READER_BUILD_REVISION": LOCK["revision"],
        "READER_APP_WEBVIEWRENDERER": "camoufox", "READER_APP_WEBUI": "vue3",
        "READER_APP_CAMOUFOXPYTHONEXECUTABLE": "/usr/bin/python3",
        "READER_APP_CAMOUFOXBROWSERVERSION": "152.0.4-beta.30",
    }
    for key, value in expected.items():
        matches = [item for item in runtime.get("Env", []) if isinstance(item, str) and item.startswith(key + "=")]
        assert matches == [key + "=" + value], "image environment identity mismatch: " + key
    return runtime


def read_blob(layout: Path, descriptor: dict[str, Any], maximum: int | None = None) -> Path:
    algorithm, digest = descriptor["digest"].split(":", 1)
    assert algorithm == "sha256" and len(digest) == 64 and all(c in "0123456789abcdef" for c in digest), "unsafe OCI descriptor"
    path = layout / "blobs/sha256" / digest
    assert path.is_file() and not path.is_symlink() and path.stat().st_size == descriptor["size"], "OCI blob size/type mismatch"
    assert maximum is None or descriptor["size"] <= maximum, "OCI metadata exceeds limit"
    assert sha256_file(path) == digest, "OCI blob checksum mismatch"
    return path


def original_config_bytes(archive: Path) -> bytes:
    digest = LOCK["imageId"].removeprefix("sha256:")
    with tarfile.open(archive, mode="r|gz") as stream:
        for member in stream:
            if member.name.removeprefix("./") in (digest + ".json", "blobs/sha256/" + digest):
                assert member.isfile() and member.size <= 1024**2, "invalid original Docker config member"
                data = stream.extractfile(member).read()
                assert hashlib.sha256(data).hexdigest() == digest, "original Docker config checksum mismatch"
                return data
    raise AssertionError("original Docker config member absent")


def preserve_original_config(layout: Path, destination: Path, data: bytes, expected_digest: str) -> dict[str, Any]:
    assert "sha256:" + hashlib.sha256(data).hexdigest() == expected_digest
    index = json.loads((layout / "index.json").read_text())
    assert len(index.get("manifests", [])) == 1
    manifest_path = read_blob(layout, index["manifests"][0], 1024**2)
    manifest = json.loads(manifest_path.read_text())
    converted = read_blob(layout, manifest["config"], 1024**2)
    canonical = lambda value: json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False)
    assert canonical(json.loads(data)) == canonical(json.loads(converted.read_bytes())), "OCI config semantics differ; restoration prohibited"
    destination.mkdir(mode=0o700)
    target = destination / "blobs/sha256"; target.mkdir(mode=0o700, parents=True)
    for blob in (layout / "blobs/sha256").iterdir():
        assert blob.is_file() and not blob.is_symlink() and len(blob.name) == 64 and all(c in "0123456789abcdef" for c in blob.name)
        os.link(blob, target / blob.name)
    def put(content: bytes) -> str:
        digest = hashlib.sha256(content).hexdigest(); path = target / digest
        if path.exists(): assert path.read_bytes() == content
        else:
            with path.open("xb") as stream: stream.write(content)
        return "sha256:" + digest
    prior = {"convertedConfigDigest": manifest["config"]["digest"], "convertedManifestDigest": index["manifests"][0]["digest"]}
    manifest["config"].update({"digest": put(data), "size": len(data)})
    manifest_bytes = json.dumps(manifest, separators=(",", ":")).encode()
    index["manifests"][0].update({"digest": put(manifest_bytes), "size": len(manifest_bytes)})
    with (destination / "index.json").open("x") as stream: json.dump(index, stream)
    with (destination / "oci-layout").open("x") as stream: json.dump({"imageLayoutVersion": "1.0.0"}, stream)
    return {**prior, "restoredConfigDigest": manifest["config"]["digest"], "configSemanticsExactlyEqualBeforeRestoration": True, "originalDockerConfigBytesRestoredInNewWrapper": True, "filesystemLayersModified": False}


def installed_package_version(rootfs: Path, package: str, expected: str) -> dict[str, str]:
    records = []
    for directory in (rootfs / "usr/local/lib", rootfs / "usr/lib"):
        for pattern in ("python*/dist-packages/", "python*/site-packages/"):
            for path in directory.glob(pattern + package + "-*.dist-info/METADATA"):
                assert path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(rootfs) and path.stat().st_size <= 1024**2
                metadata = Parser().parsestr(path.read_text(encoding="utf-8"))
                assert metadata.get("Name", "").lower() == package and metadata.get("Version") == expected
                records.append({"package": package, "version": expected, "metadataPath": str(path.relative_to(rootfs))})
    assert len(records) == 1, "expected exactly one installed distribution: " + package
    return records[0]


def verify_archive(archive: Path) -> dict[str, Any]:
    """Use the existing production-grade parser, including original config bytes."""
    before = require_regular(archive, "image archive")
    assert sha256_file(archive) == LOCK["archiveSha256"], "archive SHA-256 differs from locked release"
    original = original_config_bytes(archive)
    config = json.loads(original)
    validate_config(config)
    after = archive.stat()
    assert (before.st_size, before.st_mtime_ns) == (after.st_size, after.st_mtime_ns), "archive changed while read"
    return {
        "archive": str(archive),
        "bytes": before.st_size,
        "sha256": LOCK["archiveSha256"],
        "originalDockerConfigSha256": hashlib.sha256(original).hexdigest(),
        "originalDockerConfigBytes": len(original),
    }


def tool_paths() -> dict[str, Path]:
    base = repo_root() / "build/local-camoufox-image-20261004-a/tools/root/usr"
    values = {name: base / "bin" / name for name in ("skopeo", "umoci", "runc")}
    for name, path in values.items():
        assert path.is_file() and not path.is_symlink(), "missing bundled " + name
    return values


def isolated_environment(temp: Path, cache: Path, tools: dict[str, Path]) -> dict[str, str]:
    """Every known conversion cache/temp location remains under the D: mount."""
    for path in (temp, cache):
        # Keep this check usable in Windows unit tests as well as WSL runtime;
        # runtime paths are still required to use the exact Linux mount root.
        assert str(path).replace("\\", "/").startswith("/mnt/reader-local-full-610/"), "temp/cache escaped owned mount"
    temp_text, cache_text = str(temp).replace("\\", "/"), str(cache).replace("\\", "/")
    env = os.environ.copy()
    env.update({
        "LD_LIBRARY_PATH": str(tools["skopeo"].parent.parent / "lib/x86_64-linux-gnu"),
        "GOMAXPROCS": "2",
        "TMPDIR": temp_text, "TMP": temp_text, "TEMP": temp_text,
        "XDG_CACHE_HOME": cache_text,
        "CONTAINERS_STORAGE_CONF": cache_text + "/containers-storage.conf",
    })
    return env


def plan(args: argparse.Namespace) -> dict[str, Any]:
    """Read-only validation and an explicit execution plan."""
    archive = Path(args.archive)
    work, image_disk, data_disk = fixed_paths(args)
    archive_info = verify_archive(archive)
    image_marker = load_owned_marker(image_disk, "image", IMAGE_BYTES)
    data_marker = load_owned_marker(data_disk, "data", DATA_BYTES)
    assert not work.exists() or work.is_dir(), "work-dir is not a directory"
    return {
        "accepted": True,
        "task": TASK,
        "executeRequested": bool(args.execute),
        "readerStarted": False,
        "archive": archive_info,
        "lock": LOCK,
        "imageDisk": str(image_disk),
        "dataDisk": str(data_disk),
        "imageMarker": image_marker,
        "dataMarker": data_marker,
        "mountRoot": args.mount_root,
        "workDir": str(work),
        "limits": {"cpuQuota": CPU_QUOTA, "memoryMax": MEMORY_MAX, "memoryHigh": MEMORY_HIGH,
                   "memorySwapMax": 0, "tasksMax": TASKS_MAX},
        "actionsWhenExecute": [
            "attach only the two marker-verified loop files",
            "format those new empty files once as ext4 and mount nodev,nosuid",
            "bind all skopeo temporary/cache paths to the image D: mount",
            "copy docker archive to OCI, preserve original config bytes, restore only semantics-equal config",
            "rootful umoci unpack with child umask 022, verify 14 layers, JAR, ownership and browser metadata",
            "write a receipt; do not start Reader or reformat either disk on later invocation",
        ],
    }


def run(args: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, check=True, text=True, timeout=kwargs.pop("timeout", 900), **kwargs)


def loop_for(disk: Path, descriptor: int) -> str:
    records = json.loads(run(["losetup", "--json", "--list", "--output", "NAME,BACK-FILE"], capture_output=True).stdout)
    assert not any(Path(row.get("back-file", "")).resolve() == disk.resolve() for row in records.get("loopdevices", [])), "owned disk is already attached"
    # Popen closes unrelated descriptors by default; use the still-open parent
    # descriptor explicitly, not /proc/self (which would refer to losetup).
    output = run(["losetup", "--find", "--show", "--nooverlap", "/proc/" + str(os.getpid()) + "/fd/" + str(descriptor)], capture_output=True).stdout.strip()
    assert output.startswith("/dev/loop") and output[9:].isdigit()
    return output


def format_and_mount(disk: Path, mount: Path, label: str, logical_size: int) -> str:
    """The caller has already checked the exact marker.  No reuse path exists."""
    assert not mount.exists(), "refuse to mount over an existing path"
    descriptor = open_empty_owned_disk(disk)
    try:
        os.ftruncate(descriptor, logical_size)
        os.fsync(descriptor)
        loop = loop_for(disk, descriptor)
    finally:
        os.close(descriptor)
    try:
        run(["mkfs.ext4", "-q", "-m", "0", "-L", label, loop], timeout=180)
        mount.mkdir(mode=0o700, parents=True)
        run(["mount", "-t", "ext4", "-o", "nodev,nosuid", loop, str(mount)])
        source = run(["findmnt", "-n", "-o", "SOURCE", "--mountpoint", str(mount)], capture_output=True).stdout.strip()
        assert source == loop, "mount source changed unexpectedly"
    except BaseException:
        try:
            unmount_owned(loop, mount)
        except BaseException:
            subprocess.run(["losetup", "--detach", loop], check=False, text=True)
        raise
    return loop


def unmount_owned(loop: str | None, mount: Path) -> bool:
    """Detach only a mount whose current source is our exact recorded loop."""
    if loop is None:
        return False
    probe = subprocess.run(["findmnt", "-n", "-o", "SOURCE", "--mountpoint", str(mount)], text=True, capture_output=True)
    if probe.returncode == 0:
        assert probe.stdout.strip() == loop, "refuse to unmount a different source"
        run(["umount", str(mount)])
    listing = run(["losetup", "--json", "--output", "NAME,BACK-FILE", loop], capture_output=True).stdout
    rows = json.loads(listing).get("loopdevices", [])
    assert len(rows) == 1 and rows[0].get("name") == loop, "recorded loop identity disappeared"
    run(["losetup", "--detach", loop])
    return True


def bounded(command: list[str], env: dict[str, str], deadline: float, reserve: Path, temp: Path, label: str) -> None:
    """Run conversion in a bounded transient unit with real /tmp binds.

    Environment variables alone are insufficient: skopeo and native helpers
    may write /tmp or /var/tmp directly.  Both are bind-mounted to the D:
    ext4 image disk for this child.
    """
    assert temp.is_dir() and temp.resolve().is_relative_to(reserve.resolve())
    assert label in ("skopeo", "umoci")
    unit = f"{TASK}-{label}-{os.getpid()}.service"
    initial = subprocess.run(["systemctl", "show", unit, "--property=LoadState"], text=True, capture_output=True, check=True).stdout
    assert "LoadState=not-found" in initial, "refuse to reuse a transient preparation unit"
    # systemd does not inherit this client's environment.  Every conversion
    # parameter is supplied explicitly to the manager-side child.
    forwarded = ("LD_LIBRARY_PATH", "GOMAXPROCS", "TMPDIR", "TMP", "TEMP", "XDG_CACHE_HOME", "CONTAINERS_STORAGE_CONF")
    unit_command = ["systemd-run", "--unit=" + unit, "--pipe", "--wait", "--collect", "--quiet",
                    "--property=CPUQuota=" + CPU_QUOTA,
                    "--property=MemoryMax=" + str(MEMORY_MAX),
                    "--property=MemoryHigh=" + str(MEMORY_HIGH),
                    "--property=MemorySwapMax=0", "--property=TasksMax=" + str(TASKS_MAX),
                    "--property=BindPaths=" + str(temp) + ":/tmp",
                    "--property=BindPaths=" + str(temp) + ":/var/tmp"]
    unit_command += ["--setenv=" + name + "=" + env[name] for name in forwarded]
    unit_command += ["--"] + command
    child = subprocess.Popen(unit_command, env=env, stdin=subprocess.DEVNULL, umask=0o022)
    try:
        while child.poll() is None:
            assert time.monotonic() < deadline, "preparation deadline exceeded"
            assert shutil.disk_usage(reserve).free > 512 * 1024**2, "image disk reserve below 512 MiB"
            time.sleep(1)
        assert child.returncode == 0, "preparation tool failed: " + str(child.returncode)
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill(); child.wait(timeout=10)
        # --collect normally removes the unit.  On a client timeout/error,
        # explicitly stop its named service before any mount may be detached.
        status = subprocess.run(["systemctl", "show", unit, "--property=LoadState,ActiveState,MainPID"], text=True, capture_output=True, check=True).stdout
        if "LoadState=not-found" not in status:
            subprocess.run(["systemctl", "stop", unit], text=True, check=True)
            stopped = subprocess.run(["systemctl", "show", unit, "--property=ActiveState,MainPID"], text=True, capture_output=True, check=True).stdout
            assert "MainPID=0" in stopped and "ActiveState=inactive" in stopped, "conversion unit did not stop"


def execute(args: argparse.Namespace, planned: dict[str, Any]) -> dict[str, Any]:
    assert os.geteuid() == 0, "--execute must run as root in WSL"
    assert args.execute
    # No resume/reformat behavior: both mounts and final receipt must be absent.
    work, image_disk, data_disk = fixed_paths(args)
    receipt = work / "prepare-receipt.json"
    assert not receipt.exists(), "existing receipt means preparation is immutable; use its result, do not rerun"
    mount_root = Path(args.mount_root)
    image_mount = mount_root / "image"
    data_mount = mount_root / "data"
    assert not mount_root.exists(), "refuse to reuse a previous mount root"
    mount_root.mkdir(mode=0o700, parents=True)
    image_loop = data_loop = None
    try:
        # Repeat marker checks in execute mode immediately before the secure
        # descriptor/zero-byte checks inside format_and_mount.
        load_owned_marker(image_disk, "image", IMAGE_BYTES)
        load_owned_marker(data_disk, "data", DATA_BYTES)
        image_loop = format_and_mount(image_disk, image_mount, "rdrfullimg610", IMAGE_BYTES)
        data_loop = format_and_mount(data_disk, data_mount, "rdrfulldata610", DATA_BYTES)
        temp, cache = image_mount / "private-tmp", image_mount / "private-cache"
        temp.mkdir(mode=0o1777); cache.mkdir(mode=0o700)
        tools = tool_paths()
        env = isolated_environment(temp, cache, tools)
        exact = image_mount / "exact-image"
        exact.mkdir(mode=0o700)
        policy, auth = exact / "policy.json", exact / "empty-auth.json"
        policy.write_text(json.dumps({"default": [{"type": "reject"}], "transports": {"docker-archive": {"": [{"type": "insecureAcceptAnything"}]}}}), encoding="utf-8")
        auth.write_text(json.dumps({"auths": {}}), encoding="utf-8")
        os.chmod(policy, 0o600); os.chmod(auth, 0o600)
        archive = Path(args.archive)
        before = archive.stat()
        original = original_config_bytes(archive)
        (work / "docker-config.original.json").parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        with (work / "docker-config.original.json").open("xb") as stream:
            stream.write(original)
        layout = exact / "oci"
        deadline = time.monotonic() + 1100
        bounded([str(tools["skopeo"]), "--tmpdir", str(temp), "--policy", str(policy), "copy", "--src-no-creds", "--dest-no-creds", "--authfile", str(auth), "--format", "oci", "--image-parallel-copies", "1", "docker-archive:" + str(archive), "oci:" + str(layout) + ":reader-tested"], env, deadline, image_mount, temp, "skopeo")
        assert (before.st_size, before.st_mtime_ns) == (archive.stat().st_size, archive.stat().st_mtime_ns)
        assert sha256_file(archive) == LOCK["archiveSha256"]
        index = json.loads((layout / "index.json").read_text())
        assert index["schemaVersion"] == 2 and len(index["manifests"]) == 1
        manifest = json.loads(read_blob(layout, index["manifests"][0], 1024**2).read_text())
        config = json.loads(read_blob(layout, manifest["config"], 1024**2).read_text())
        validate_config(config)
        restoration: Any = None
        if manifest["config"]["digest"] != LOCK["imageId"]:
            restored = exact / "oci-original-config"
            restoration = preserve_original_config(layout, restored, original, LOCK["imageId"])
            layout = restored
            index = json.loads((layout / "index.json").read_text())
            manifest = json.loads(read_blob(layout, index["manifests"][0], 1024**2).read_text())
            config = json.loads(read_blob(layout, manifest["config"], 1024**2).read_text())
        validate_config(config)
        assert manifest["config"]["digest"] == LOCK["imageId"]
        assert len(manifest["layers"]) == LOCK["layerCount"] == len(config["rootfs"]["diff_ids"])
        for descriptor in manifest["layers"]:
            read_blob(layout, descriptor)
        bundle = exact / "bundle"
        bounded([str(tools["umoci"]), "unpack", "--image", str(layout) + ":reader-tested", str(bundle)], env, deadline, image_mount, temp, "umoci")
        rootfs = bundle / "rootfs"
        jar = rootfs / "app/reader.jar"
        assert sha256_file(jar) == LOCK["jarSha256"] and not jar.is_symlink()
        expected = [(rootfs, 0, 0, 0o755), (rootfs / "app", 0, 0, None), (jar, 0, 0, None),
                    (rootfs / "home/reader", 10001, 10001, None), (rootfs / "usr/local/bin/reader-entrypoint", 0, 0, 0o755)]
        ownership = {}
        for path, uid, gid, mode in expected:
            item = path.lstat(); assert not stat.S_ISLNK(item.st_mode)
            assert (item.st_uid, item.st_gid) == (uid, gid)
            actual_mode = stat.S_IMODE(item.st_mode)
            assert not actual_mode & 0o022
            if mode is not None: assert actual_mode == mode
            ownership[str(path.relative_to(rootfs) or ".")] = {"uid": uid, "gid": gid, "mode": oct(actual_mode)}
        browser_metadata = []
        for path in (rootfs / "home/reader/.cache/camoufox").rglob("version.json"):
            assert not path.is_symlink() and path.stat().st_size < 65536
            value = json.loads(path.read_text())
            assert value["sha256"] == "5720d45b894ce1770543de024c6f10d514b38be560fa2dc3226b3d8586caf672"
            browser_metadata.append(str(path.relative_to(rootfs)))
        assert browser_metadata
        packages = [installed_package_version(rootfs, "camoufox", "0.5.6"),
                    installed_package_version(rootfs, "playwright", "1.62.0")]
        result = dict(planned, executed=True, readerStarted=False, rootfs=str(rootfs), imageLoop=image_loop,
                      dataLoop=data_loop, configRestoration=restoration, criticalOwnership=ownership,
                      browserMetadata=browser_metadata, installedPackages=packages,
                      originalConfigPath=str(work / "docker-config.original.json"),
                      paths={"work": str(work), "mountRoot": str(mount_root), "imageMount": str(image_mount), "dataMount": str(data_mount), "temporary": str(temp), "cache": str(cache)})
        atomic_json(receipt, result)
        return result
    except BaseException as error:
        # Never delete or reformat either disk.  After transient children have
        # been stopped by bounded(), unmount only the two exact loops created
        # in this invocation so a failed preparation cannot pin WSL storage.
        detached = []
        cleanup_errors = []
        for loop, mount, role in ((data_loop, data_mount, "data"), (image_loop, image_mount, "image")):
            try:
                if unmount_owned(loop, mount): detached.append(role)
            except BaseException as cleanup_error:
                cleanup_errors.append({"role": role, "error": repr(cleanup_error)})
        try:
            atomic_json(work / "prepare-failed.json", {"task": TASK, "readerStarted": False,
                "error": repr(error), "detachedOwnedLoops": detached, "cleanupErrors": cleanup_errors,
                "imageDisk": args.image_disk, "dataDisk": args.data_disk})
        except BaseException:
            pass
        raise


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", default="/mnt/d/Reader-Pro-Native-610-20261015-a/reader-image.tar.gz")
    parser.add_argument("--work-dir", default=WORK_DIR)
    parser.add_argument("--image-disk", required=True)
    parser.add_argument("--data-disk", required=True)
    parser.add_argument("--mount-root", default=MOUNT_ROOT)
    parser.add_argument("--execute", action="store_true", help="format only marker-verified new disks; never starts Reader")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    result = plan(args)
    if args.execute:
        result = execute(args, result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
