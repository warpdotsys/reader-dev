#!/usr/bin/env python3
"""Retarget Debian OpenJDK config links inside a disposable /var/tmp tree.

`dpkg-deb -x` does not install the package's /etc files. This rewrites only
symlinks in the extracted JRE that already point at /etc/java-11-openjdk and
whose targets are present in that same extracted package. It never touches
the host's /etc or a source JAR.
"""

import argparse
import os
from pathlib import Path


def relocate(root):
    root = root.absolute()
    if (root.name != "jre" or root.parent.parent != Path("/var/tmp") or
            not root.parent.name.startswith("reader-jar-isolation.") or
            root.is_symlink() or root.parent.is_symlink() or root.resolve() != root):
        raise ValueError("JRE must be a real directory under /var/tmp/reader-jar-isolation.*")
    jvm = root / "usr/lib/jvm/java-11-openjdk-amd64"
    configs = root / "etc/java-11-openjdk"
    if not jvm.is_dir() or not configs.is_dir():
        raise ValueError("Extracted OpenJDK directories are missing")
    changes = []
    skipped = []
    for link in jvm.rglob("*"):
        if not link.is_symlink():
            continue
        target = os.readlink(link)
        if not target.startswith("/etc/java-11-openjdk/"):
            continue
        source = root / target.lstrip("/")
        if not source.resolve().is_relative_to(configs.resolve()):
            raise ValueError("Packaged JRE link escapes its configuration directory")
        if source.is_file():
            changes.append((link, source))
        else:
            skipped.append(link.relative_to(root).as_posix())
    for link, source in changes:
        link.unlink()
        link.symlink_to(os.path.relpath(source, link.parent))
    security = jvm / "conf/security/java.security"
    if not security.is_file():
        raise RuntimeError("java.security is still unreadable")
    return len(changes), skipped


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    changed, skipped = relocate(args.root)
    print(f"Relocated {changed} JRE config links inside the extracted package")
    print(f"Optional links without packaged targets: {len(skipped)}")


if __name__ == "__main__":
    main()
