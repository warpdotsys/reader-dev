#!/usr/bin/env python3
"""Run the original/restored WebView Cookie probe in a private Linux netns.

Invoke this program *under* `unshare --net --fork` as root. It refuses to start
either JAR unless its network namespace differs from PID 1 and has only `lo`.
The namespace has no route to production or the LAN; the HTTP fixture and both
Reader processes share its private loopback. The original JAR is never edited.
"""

import argparse
import hashlib
import os
from pathlib import Path
import socket
import subprocess
import sys


ORIGINAL_SHA256 = "b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c"
ISOLATION_ACK = "READER_ORIGINAL_JAR_NETWORK_ISOLATED"
SCRIPT = Path(__file__).resolve().with_name("compare-webview-cookie.py")


def require_private_loopback():
    if sys.platform != "linux" or os.geteuid() != 0:
        raise SystemExit("Refusing to start JARs: a root-owned Linux network namespace is required")
    current = os.stat("/proc/self/ns/net").st_ino
    initial = os.stat("/proc/1/ns/net").st_ino
    if current == initial or [name for _, name in socket.if_nameindex()] != ["lo"]:
        raise SystemExit("Refusing to start JARs: not in a private loopback-only network namespace")
    subprocess.run(["ip", "link", "set", "lo", "up"], check=True)
    if [name for _, name in socket.if_nameindex()] != ["lo"]:
        raise SystemExit("Refusing to start JARs: the private namespace gained another interface")
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java", required=True, type=Path)
    parser.add_argument("--original", required=True, type=Path)
    parser.add_argument("--restored", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--exercise-post", action="store_true",
                        help="Include the synthetic POST request-shape comparison")
    args = parser.parse_args()

    # Isolation is checked before reading any JAR or granting the comparison
    # script's acknowledgment. A bare environment variable cannot bypass it.
    require_private_loopback()
    for path in (args.java, args.original, args.restored):
        if not path.is_file():
            parser.error(f"Required input does not exist: {path}")
    if sha256(args.original) != ORIGINAL_SHA256:
        parser.error("Original JAR SHA-256 does not match the read-only baseline")
    report = args.report.resolve()
    if not report.is_relative_to(Path("/var/tmp")) or report.exists():
        parser.error("Report must be a new file beneath /var/tmp")
    # No child needs to administer its namespace. Drop root and capabilities
    # before Java starts; /tmp remains writable for disposable probe data.
    os.setgroups([])
    os.setgid(65534)
    os.setuid(65534)
    if not all(path.is_file() and os.access(path, os.R_OK) for path in
               (args.java, args.original, args.restored, SCRIPT)):
        parser.error("Unprivileged probe user cannot read all required inputs")
    env = os.environ.copy()
    env[ISOLATION_ACK] = "confirmed"
    probe = [
        sys.executable, str(SCRIPT),
        "--java", str(args.java),
        "--original", str(args.original),
        "--restored", str(args.restored),
        "--report", str(report),
        "--original-network-isolated", "--exercise-script",
    ]
    if args.exercise_post:
        probe.append("--exercise-post")
    os.execve(sys.executable, probe, env)


if __name__ == "__main__":
    main()
