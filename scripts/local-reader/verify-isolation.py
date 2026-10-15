#!/usr/bin/env python3
"""Verify the local two-role runtime without reading credentials or private books."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
import service

PROBE = '''import json, os
from pathlib import Path
role = ROLE
mounts = {line.split()[4] for line in Path('/proc/self/mountinfo').read_text().splitlines()}
status = dict(line.split(':', 1) for line in Path('/proc/self/status').read_text().splitlines() if ':' in line)
foreign = ['/local-secrets/license-admin-token', '/license-data/legacy-rsa-private.pkcs8', '/license-data/signing-key.pkcs8'] if role == 'reader' else ['/local-secrets/reader-admin-key']
own = '/local-secrets/' + ('reader-admin-key' if role == 'reader' else 'license-admin-token')
print(json.dumps({'uid': os.getuid(), 'gid': os.getgid(),
 'capabilitiesEmpty': all(int(status[key].strip(), 16) == 0 for key in ('CapEff', 'CapPrm', 'CapBnd')),
 'noNewPrivileges': status['NoNewPrivs'].strip() == '1',
 'foreignSecretsAbsent': all(not os.path.lexists(path) for path in foreign),
 'ownSecretFilePresent': Path(own).is_file(),
 'foreignDataNotMounted': ('/license-data' if role == 'reader' else '/storage') not in mounts}))
'''


def accepted_probe(value):
    return (value.get('uid') == value.get('gid') == 10001 and all(value.get(key) is True for key in
        ('capabilitiesEmpty', 'noNewPrivileges', 'foreignSecretsAbsent', 'ownSecretFilePresent', 'foreignDataNotMounted')))


def execute():
    output = service.WORK / 'isolation-acceptance.json'
    if output.exists():
        raise ValueError('existing acceptance is immutable')
    service.verified_receipt()
    state = service.metadata(service.WORK / 'service-state.json')
    if (state.get('status') != 'running' or state.get('roles') != ['license', 'reader']
            or state.get('generatedNetworkTesting') is not False):
        raise ValueError('two-role service with default network policy required')
    service.command('systemctl', 'is-active', '--quiet', service.UNIT)
    runroot = Path(state['runRoot'])
    if runroot.parent != service.MOUNT / 'image/service-runs' or not re.fullmatch(r'run-[0-9a-f]{16}', runroot.name):
        raise ValueError('unsafe runtime identity')
    # This diagnostic intentionally runs from source, not as the durable
    # service controller. Verify its two executable dependencies independently.
    software = service.WORK / 'runtime-software'
    tools = {'runc': software / 'runc/bin/runc', 'library': software / 'runc/lib/libseccomp.so.2'}
    for name, digest in (('runc', service.RUNC_SHA), ('library', service.SECCOMP_SHA)):
        if tools[name].is_symlink() or service.prepare.sha256_file(tools[name]) != digest:
            raise ValueError('diagnostic runtime software mismatch')
    env = dict(os.environ, LD_LIBRARY_PATH=str(tools['library'].parent), GOMAXPROCS='2')
    probes, namespaces = {}, {}
    for role in ('license', 'reader'):
        prefix = [tools['runc'], '--root', runroot / role / 'runc-state']
        container = service.NAME + '-' + role
        observed = json.loads(service.command(*prefix, 'state', container, env=env).stdout)
        if observed.get('id') != container or observed.get('status') != 'running' or not isinstance(observed.get('pid'), int):
            raise ValueError('role is not running')
        pid = observed['pid']
        namespaces[role] = {name: os.readlink('/proc/' + str(pid) + '/ns/' + name) for name in ('pid', 'mnt')}
        result = service.command(*prefix, 'exec', container, '/usr/bin/python3', '-c', PROBE.replace('ROLE', repr(role)), env=env)
        if len(result.stdout) > 2048:
            raise ValueError('probe output bound exceeded')
        probes[role] = json.loads(result.stdout)
        if not accepted_probe(probes[role]):
            raise ValueError('role isolation was not accepted')
    if any(namespaces['license'][name] == namespaces['reader'][name] for name in ('pid', 'mnt')):
        raise ValueError('roles share process or mount namespace')
    spec = importlib.util.spec_from_file_location('local_persistence_identity', Path(__file__).with_name('verify-persistence.py'))
    persistence = importlib.util.module_from_spec(spec); spec.loader.exec_module(persistence)
    current = persistence.identity()
    prior = service.metadata(service.WORK / 'restart-after.json')['hashes']
    if current != prior:
        raise ValueError('known generated data or license identity changed')
    report = dict(accepted=True, roles=probes, pidAndMountNamespacesDistinct=True,
        originalGeneratedDataAndLicenseHashesRetained=True, defaultPrivateNetworkDeny=True,
        secretContentsRead=False, privateBookContentsRead=False,
        softwareJarSha256=service.prepare.LOCK['jarSha256'])
    with output.open('x', encoding='utf-8') as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
    print(json.dumps(report))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execute', action='store_true')
    if not parser.parse_args(argv).execute:
        print(json.dumps({'planned': True, 'writes': False, 'secretContentsRead': False}))
        return 0
    execute()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
