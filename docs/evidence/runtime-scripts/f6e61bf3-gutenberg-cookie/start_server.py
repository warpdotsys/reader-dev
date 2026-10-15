"""Launch only a fresh owned public-domain scope; controller and broker inside total budget."""
import json
import os
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
AUTH = Path('/var/tmp/reader-generated-fragments-20261003.RMZay2')
if ROOT.parent != AUTH or not re.fullmatch(r'live-gutenberg-20261014\.[A-Za-z0-9]+', ROOT.name) or ROOT.resolve(strict=True) != ROOT:
    raise ValueError('Unexpected authorized scope')
if os.geteuid() != 0 or (ROOT / 'launch.json').exists():
    raise ValueError('Preserve existing launch')
start = time.monotonic()
while True:
    memory = dict(row.split(':', 1) for row in Path('/proc/meminfo').read_text().splitlines())
    if int(memory['MemAvailable'].split()[0]) >= 2097152:
        break
    if time.monotonic() - start >= 55:
        raise RuntimeError('Unchanged preflight memory threshold not satisfied; no Reader started')
    time.sleep(2)
prefix = 'readerall' + os.urandom(6).hex()
parent, anchor, unit = prefix + '.slice', prefix + '-anchor.service', prefix + '-controller.service'


def command(*args):
    return subprocess.run(args, check=True, capture_output=True, text=True, timeout=20)


intent = dict(unit=unit, parentSlice=parent, anchor=anchor, root=str(ROOT),
    scope='owned fixed-public-page cookie/script probe', productionChanged=False)
with (ROOT / 'launch-intent.json').open('x') as output:
    json.dump(intent, output, indent=2)
# Preserve exact ownership handles before any externally visible start. A lost
# SSH response must be reconciled against these handles, never blindly retried.
command('systemd-run', '--unit=' + anchor, '--slice=' + parent, '--property=Type=oneshot', '--property=RemainAfterExit=yes', '/bin/true')
command('systemctl', 'set-property', '--runtime', parent, 'CPUQuota=200%', 'MemoryHigh=1610612736', 'MemoryMax=2147483648', 'MemorySwapMax=0', 'TasksMax=512')
command('systemd-run', '--unit=' + unit, '--slice=' + parent, '--property=Type=exec', '--property=RuntimeMaxSec=480',
    '--property=KillMode=mixed', '/usr/bin/python3', str(ROOT / 'scripts/controller.py'), '--root', str(ROOT), '--slice', parent)
receipt = dict(unit=unit, parentSlice=parent, anchor=anchor, root=str(ROOT),
    controllerAndProxyIncludedInTotalBudget=True, publicDomain='www.gutenberg.org',
    noPublishedPorts=True, privateBooksOrCredentialsCopied=False, productionChanged=False)
with (ROOT / 'launch.json').open('x') as output:
    json.dump(receipt, output, indent=2)
print(json.dumps(receipt), flush=True)


