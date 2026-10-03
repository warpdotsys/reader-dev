"""Download, hash-verify, and install the locked Camoufox browser build."""

import hashlib
import importlib.metadata
import json
import platform
import shutil
import sys
import tempfile
from pathlib import Path

import requests


def platform_key():
    if platform.system() != "Linux":
        raise RuntimeError("The production Camoufox image currently supports Linux only")
    machine = platform.machine().lower()
    if machine in ("x86_64", "amd64"):
        return "linux-x86_64"
    if machine in ("aarch64", "arm64"):
        return "linux-arm64"
    raise RuntimeError(f"Unsupported Camoufox CPU architecture: {machine}")


def download_verified(url, target, expected_size, expected_sha256):
    digest = hashlib.sha256()
    size = 0
    with requests.get(url, stream=True, timeout=(15, 120)) as response:
        response.raise_for_status()
        with target.open("wb") as output:
            for chunk in response.iter_content(chunk_size=1024 * 1024):
                if not chunk:
                    continue
                size += len(chunk)
                if size > expected_size:
                    raise RuntimeError("Camoufox release asset exceeds its locked size")
                digest.update(chunk)
                output.write(chunk)
    actual = digest.hexdigest()
    if size != expected_size:
        raise RuntimeError(f"Camoufox asset size mismatch: expected {expected_size}, got {size}")
    if actual != expected_sha256:
        raise RuntimeError(f"Camoufox asset SHA-256 mismatch: {actual}")
    return size, actual


def main():
    lock_path = Path(__file__).with_name("camoufox-browser.lock")
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    if importlib.metadata.version("camoufox") != lock["pythonPackageVersion"]:
        raise RuntimeError("Camoufox Python package does not match the browser lock")
    if importlib.metadata.version("playwright") != lock["playwrightVersion"]:
        raise RuntimeError("Playwright Python package does not match the browser lock")

    asset = lock["assets"][platform_key()]
    url = f"https://github.com/daijro/camoufox/releases/download/{lock['releaseTag']}/{asset['name']}"
    version = lock["releaseTag"].removeprefix("v").split("-", 1)
    version_object, build = version

    from camoufox.multiversion import get_active_path
    from camoufox.pkgman import AvailableVersion, CamoufoxFetcher, RepoConfig, Version

    selected = AvailableVersion(
        version=Version(build=build, version=version_object),
        url=url,
        is_prerelease=False,
        asset_size=asset["size"],
        sha256=asset["sha256"],
    )

    with tempfile.TemporaryDirectory(prefix="reader-camoufox-install-") as temp_dir:
        archive = Path(temp_dir) / asset["name"]
        size, digest = download_verified(url, archive, asset["size"], asset["sha256"])

        class VerifiedFetcher(CamoufoxFetcher):
            def __init__(self, *args, verified_archive, expected_url, **kwargs):
                self.verified_archive = verified_archive
                self.expected_url = expected_url
                super().__init__(*args, **kwargs)

            def download_file(self, file, requested_url):
                if requested_url != self.expected_url:
                    raise RuntimeError("Camoufox installer requested an unlocked URL")
                with self.verified_archive.open("rb") as source:
                    shutil.copyfileobj(source, file)
                file.seek(0)
                return file

        fetcher = VerifiedFetcher(
            RepoConfig.get_default(), selected,
            verified_archive=archive, expected_url=url,
        )
        fetcher.github_repo = "daijro/camoufox"
        fetcher.install()

    active = get_active_path()
    if active is None:
        raise RuntimeError("Camoufox installer did not activate the pinned browser")
    metadata = json.loads((active / "version.json").read_text(encoding="utf-8"))
    if metadata.get("sha256") != asset["sha256"]:
        raise RuntimeError("Installed Camoufox metadata does not match the locked SHA-256")
    print(json.dumps({
        "version": lock["releaseTag"],
        "asset": asset["name"],
        "size": size,
        "sha256": digest,
        "installPath": str(active),
    }, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Camoufox installation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise
