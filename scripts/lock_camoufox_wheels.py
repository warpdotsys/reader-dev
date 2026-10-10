"""Print a reviewed pip hash lock for the Reader Camoufox Linux image.

The image uses Ubuntu 22.04 / CPython 3.10. This script selects only PyPI
wheels compatible with both linux/amd64 and linux/arm64 at glibc 2.35; it
does not install or write packages. Review the printed diff before applying.
"""

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

from pip._internal.utils.compatibility_tags import get_supported
from pip._vendor.packaging.markers import default_environment
from pip._vendor.packaging.requirements import Requirement
from pip._vendor.packaging.utils import canonicalize_name
from pip._vendor.packaging.utils import parse_wheel_filename


ARCHES = ("x86_64", "aarch64")
VERSION = "310"


def target_tags(arch):
    platforms = [f"manylinux_2_{minor}_{arch}" for minor in range(35, 16, -1)]
    platforms.extend((f"manylinux2014_{arch}", f"manylinux2010_{arch}", f"manylinux1_{arch}"))
    return set(get_supported(version=VERSION, platforms=platforms, impl="cp", abis=["cp310"]))


def lock_entry(name, version, arch_tags):
    url = f"https://pypi.org/pypi/{name}/{version}/json"
    with urllib.request.urlopen(url, timeout=20) as response:
        metadata = json.load(response)
    if metadata["info"]["version"] != version:
        raise ValueError(f"PyPI returned unexpected version for {name}: {metadata['info']['version']}")
    selected = {}
    covered = set()
    for item in metadata["urls"]:
        if item["packagetype"] != "bdist_wheel" or item.get("yanked"):
            continue
        filename = item["filename"]
        _, wheel_version, _, tags = parse_wheel_filename(filename)
        if str(wheel_version) != version:
            continue
        supported = {arch for arch, target in arch_tags.items() if tags & target}
        if not supported:
            continue
        digest = item["digests"]["sha256"]
        if not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError(f"Invalid SHA-256 for {filename}")
        selected[filename] = digest
        covered.update(supported)
    if covered != set(arch_tags):
        raise ValueError(f"No compatible wheel for {name}=={version}: missing {set(arch_tags) - covered}")
    if not selected:
        raise ValueError(f"No compatible wheel for {name}=={version}")
    lines = [f"# PyPI wheels: {', '.join(sorted(selected))}", f"{name}=={version} \\"]
    items = sorted(selected.items())
    for index, (_, digest) in enumerate(items):
        continuation = " \\" if index < len(items) - 1 else ""
        lines.append(f"    --hash=sha256:{digest}{continuation}")
    return "\n".join(lines), len(selected), metadata["info"].get("requires_dist") or []


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("requirements", type=Path)
    args = parser.parse_args()
    entries = []
    for line in args.requirements.read_text(encoding="utf-8").splitlines():
        match = re.fullmatch(r"([A-Za-z0-9_.-]+)==([A-Za-z0-9_.-]+)(?:\s+\\)?", line.strip())
        if match:
            entries.append(match.groups())
    if not entries:
        raise ValueError("No pinned requirements found")
    arch_tags = {arch: target_tags(arch) for arch in ARCHES}
    locked = {canonicalize_name(name): version for name, version in entries}
    blocks = [
        "# Generated from official PyPI release metadata for Ubuntu 22.04 / CPython 3.10.",
        "# Supports linux/amd64 and linux/arm64; every allowed wheel has a SHA-256.",
        "# Regenerate with scripts/lock_camoufox_wheels.py and review the diff.",
    ]
    count = 0
    dependencies = {}
    for name, version in entries:
        block, selected_count, requires_dist = lock_entry(name, version, arch_tags)
        blocks.append(block)
        count += selected_count
        dependencies[name] = requires_dist
    for arch in ARCHES:
        env = default_environment()
        env.update(python_version="3.10", python_full_version="3.10.12",
                   sys_platform="linux", platform_system="Linux",
                   platform_machine=arch, extra="")
        for package, requirements in dependencies.items():
            for value in requirements:
                required = Requirement(value)
                if required.marker and not required.marker.evaluate(env):
                    continue
                dependency = canonicalize_name(required.name)
                if dependency not in locked:
                    raise ValueError(f"{package} needs unlocked dependency {required.name} on {arch}")
                if required.specifier and not required.specifier.contains(locked[dependency]):
                    raise ValueError(f"{package} requires {value}, locked {locked[dependency]} on {arch}")
    print("\n".join(blocks) + "\n")
    print(f"Locked {len(entries)} packages and {count} wheels", file=sys.stderr)


if __name__ == "__main__":
    main()
