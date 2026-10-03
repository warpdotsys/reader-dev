"""Fail closed before launching the original JAR from comparison scripts.

The original binary ignores the restored server's bind-address setting. The
caller must independently provide inbound network isolation; this module does
not create a sandbox or change firewall rules.
"""

import os


ISOLATION_ACK = "READER_ORIGINAL_JAR_NETWORK_ISOLATED"


def require_original_jar_isolation():
    if os.environ.get(ISOLATION_ACK) != "confirmed":
        raise SystemExit(
            "Refusing to start the original JAR: it listens on all interfaces, "
            "and this host may allow inbound Java traffic. First verify inbound "
            "isolation outside this script (for example, in a network-isolated "
            "VM/container); then set " + ISOLATION_ACK + "=confirmed. "
            "The environment variable is only an acknowledgment, not isolation."
        )
