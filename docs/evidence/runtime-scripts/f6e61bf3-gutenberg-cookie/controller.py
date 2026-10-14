"""Owned public-domain three-way controller; no direct container WAN, no host TCP listener."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time

import allowlist_transport as transport

AUTH_ROOT = Path('/var/tmp/reader-generated-fragments-20261003.RMZay2')
SOFTWARE = AUTH_ROOT / 'accurate-native-f6e61bf3-20261013.uzcq04it/software'
LOAD = json.loads((SOFTWARE / 'load-receipt.json').read_bytes())
if LOAD.get('accepted') is not True or LOAD.get('oldSmokeAliasRestored') is not True:
    raise ValueError('Exact new image must be loaded and old alias restored first')
IMAGE = LOAD['loadedDockerImageId']
CONFIG = 'sha256:c76a847715d1efe04f84e92c320465412db1f23d147db61eff6c235b35318850'
REVISION = '27512eca06d9889df4bf217a7e1291aed589210f'
JAR = '1f114993a273eb2d5dc30d47ff3b10446ce30fccd04bb6a49e46f42cef2138c5'


def helper(directory, name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), directory / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', required=True, type=Path)
    parser.add_argument('--slice', required=True)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    if root.parent != AUTH_ROOT or not re.fullmatch(r'live-gutenberg-20261014\.[A-Za-z0-9]+', root.name) or args.root.is_symlink():
        raise ValueError('Fresh authorized test subdirectory required')
    scripts = root / 'scripts'
    base = helper(scripts, 'run-three-way-webview-in-docker')
    membership = Path('/proc/self/cgroup').read_text().strip()
    base.require_outer_controller(args.slice, membership)
    group = Path('/sys/fs/cgroup') / args.slice
    snapshot = lambda: base.budget_snapshot(group, 1610612736)
    snapshot()
    original = AUTH_ROOT / 'bundle/reference/original/reader-pro-3.2.14.original.jar'
    restored = SOFTWARE / 'reader-pro-v6.1.0.jar'
    if base.digest(original) != base.ORIGINAL_SHA or base.digest(restored) != JAR:
        raise ValueError('Keep exact original and candidate')
    identity = base.IDENTITY.inspect_verified(IMAGE, CONFIG, 'amd64', REVISION, JAR)
    base.command('docker', 'image', 'inspect', base.ARCHIVED)
    if base.command('docker', 'info', '--format', '{{.CgroupDriver}}').stdout.strip() != 'systemd':
        raise ValueError('Actual systemd cgroup driver required')
    token = args.slice.removesuffix('.slice')
    containers, broker, legacy, runtime = [], None, None, None
    results = root / 'results'
    results.mkdir(mode=0o750)
    os.chown(results, 10001, 10001)
    egress = root / 'egress'
    egress.mkdir(mode=0o750)
    os.chown(egress, 0, 10001)
    provenance = dict(schemaVersion=1, originalJarSha256=base.digest(original), restoredJarSha256=base.digest(restored),
        runtimeImageId=IMAGE, runtimeConfigIdentity=identity, archivedRenderer=base.ARCHIVED,
        controllerInitialMembership=membership, controllerIncludedOuterSlice=args.slice,
        publicDomainAuthorization='www.gutenberg.org fixed public book only',
        productionChanged=False, privateBooksRead=0, realCredentialsImported=False,
        originalProductionRendererVersionProven=False, historicalRendererStoppedBeforeCamoufox=False,
        overallAccepted=False, budgetAccepted=False, containerCgroups=[], resourceSamples=[])
    started = time.monotonic()
    common = ['--cgroup-parent', args.slice, '--cpus=2', '--memory=2g', '--memory-swap=2g',
        '--pids-limit=256', '--shm-size=512m', '--cap-drop=ALL', '--security-opt=no-new-privileges:true',
        '--restart=no', '--label', 'com.medwarp.reader.generated-test=' + token]

    def start(name, arguments):
        cid = base.command('docker', 'run', '-d', '--name', token + '-' + name, *common, *arguments).stdout.strip()
        if not re.fullmatch('[0-9a-f]{64}', cid):
            raise RuntimeError('Unexpected owned container ID')
        containers.append(cid)
        state = base.owned_container(cid, token)
        pid = state['State']['Pid']
        member = Path('/proc', str(pid), 'cgroup').read_text().strip()
        if not member.startswith('0::/' + args.slice + '/docker-') or state['HostConfig']['PortBindings']:
            raise RuntimeError('Owned runtime escaped budget or published a port')
        if state['HostConfig']['Privileged'] or state['HostConfig']['CapAdd']:
            raise RuntimeError('No privileged components permitted')
        provenance['containerCgroups'].append(member)
        return cid, pid

    def sample(phase):
        provenance['resourceSamples'].append(base.resource_sample(group, phase, started))

    try:
        broker = transport.Broker(egress)
        hosts = root / 'hosts.generated'
        with hosts.open('x') as output:
            output.write('127.0.0.1 localhost\n::1 localhost\n' + ''.join(ip + ' www.gutenberg.org\n' for ip in broker.addresses))
        os.chmod(hosts, 0o444)
        hosts_mount = ['--mount', 'type=bind,src=' + str(hosts) + ',dst=/etc/hosts,readonly']
        legacy, legacy_pid = start('legacy', ['--network=none', *hosts_mount, base.ARCHIVED])
        # Only this newly owned netns changes: no new interface/default route,
        # host address, firewall or archived implementation is modified.
        base.command('nsenter', '--target', legacy_pid, '--net', '--', 'ip', 'addr', 'add', broker.addresses[0] + '/32', 'dev', 'lo')
        provenance['ownedNetnsLoopbackPublicAlias'] = broker.addresses[0]
        provenance['transparentTlsBridgeRequiresExactGutenbergSni'] = True
        verify = ("import importlib.util,os,sys; s=importlib.util.spec_from_file_location('guard',sys.argv[1]);"
            "m=importlib.util.module_from_spec(s);s.loader.exec_module(m);m.require_private_loopback();"
            "print(os.stat('/proc/self/ns/net').st_ino)")
        guard = scripts / 'run-webview-cookie-in-linux-netns.py'
        inode = base.command('nsenter', '--target', legacy_pid, '--net', '--', 'python3', '-c', verify, guard).stdout.strip()
        if not inode.isdigit():
            raise RuntimeError('Independent loopback namespace guard failed')
        provenance['privateNetworkNamespaceInode'] = int(inode)
        runtime, runtime_pid = start('runtime', ['--network=container:' + legacy, '--user=10001:10001', '--init',
            '--tmpfs', '/tmp:size=512m,mode=1777', '--entrypoint=/usr/bin/python3', *hosts_mount,
            '--mount', 'type=bind,src=' + str(scripts) + ',dst=/probe,readonly',
            '--mount', 'type=bind,src=' + str(egress) + ',dst=/egress,readonly',
            '--mount', 'type=bind,src=' + str(original) + ',dst=/inputs/original.jar,readonly',
            '--mount', 'type=bind,src=' + str(restored) + ',dst=/inputs/restored.jar,readonly',
            '--mount', 'type=bind,src=' + str(results) + ',dst=/results',
            IMAGE, '-c', 'import time;time.sleep(600)'])
        if os.stat('/proc/' + str(runtime_pid) + '/ns/net').st_ino != int(inode):
            raise RuntimeError('Runtime does not share guarded namespace')
        packaged = base.command('docker', 'exec', runtime, 'sha256sum', '/app/reader.jar').stdout.split()
        if packaged != [JAR, '/app/reader.jar']:
            raise RuntimeError('Actual packaged JAR differs')
        provenance['packagedJarSha256'] = packaged[0]
        provenance['packagedJarBytesIndependentlyRead'] = True
        ready = "import socket;socket.create_connection(('127.0.0.1',8050),timeout=2).close()"
        for _ in range(30):
            if base.command('docker', 'exec', runtime, 'python3', '-c', ready, check=False).returncode == 0:
                break
            time.sleep(1)
        else:
            raise TimeoutError('Owned archived renderer was not ready')
        probe = ['docker', 'exec', '--user=10001:10001', '--env=READER_ORIGINAL_JAR_NETWORK_ISOLATED=confirmed',
            '--env=READER_PRIVATE_NETNS_INODE=' + inode, runtime, 'python3', '/probe/compare_cookie_script.py',
            '--java', '/opt/java/openjdk/bin/java', '--original', '/inputs/original.jar', '--restored', '/inputs/restored.jar',
            '--expected-restored-sha256', JAR, '--report', '/results/REAL_COOKIE_SCRIPT.json',
            '--public-bridge-ip', broker.addresses[0],
            '--phase-handoff-dir', '/results/phases']

        def handoff():
            base.require_unpressured_budget(snapshot())
            base.owned_container(legacy, token)
            base.command('docker', 'stop', '--time=10', legacy, timeout=30)
            if base.owned_container(legacy, token)['State']['Running']:
                raise RuntimeError('Reference did not stop before Camoufox')
            checked = base.command('nsenter', '--target', runtime_pid, '--net', '--', 'python3', '-c', verify, guard).stdout.strip()
            if checked != inode:
                raise RuntimeError('Namespace changed during handoff')
            provenance['historicalRendererStoppedBeforeCamoufox'] = True
            sample('historical-renderer-stopped')

        sample('probe-start')
        code = base.run_probe_with_handoff(probe, root / 'probe.log', results / 'phases', handoff, sample)
        provenance['probeExitCode'] = code
        sample('probe-finished')
        base.require_probe_acceptance(code)
        report = json.loads((results / 'REAL_COOKIE_SCRIPT.json').read_bytes())
        if not report.get('passed') or not report.get('realSiteScriptCookieThreeWayProven'):
            raise RuntimeError('Actual complete real-page script and cookie proof required')
        if broker.facts()['deniedRequests'] < 4 or not broker.facts()['tunnels']:
            raise RuntimeError('Actual negative proxy and upstream observations required')
        base.require_unpressured_budget(snapshot())
        provenance['overallAccepted'] = provenance['budgetAccepted'] = True
    except Exception as failure:
        provenance['failureType'] = type(failure).__name__
    finally:
        if broker is not None:
            broker.close()
            provenance['proxyObservations'] = broker.facts()
            provenance['ownedBrokerStoppedAndSocketRemoved'] = not broker.socket_path.exists()
        if legacy is not None:
            provenance['archivedRendererDiagnostics'] = base.preserve_owned_renderer_diagnostics(legacy, token, root)
        removed = []
        for cid in reversed(containers):
            base.owned_container(cid, token)
            base.command('docker', 'rm', '--force', cid)
            removed.append(cid)
        provenance['removedOwnedTestContainers'] = removed
        provenance['outerAggregateBudget'] = snapshot()
        try:
            base.require_unpressured_budget(provenance['outerAggregateBudget'])
        except Exception:
            provenance['overallAccepted'] = provenance['budgetAccepted'] = False
        provenance['originalJarUnchanged'] = base.digest(original) == base.ORIGINAL_SHA
        provenance['restoredJarUnchanged'] = base.digest(restored) == JAR
        provenance['seconds'] = round(time.monotonic() - started, 3)
        base.write_new(root / 'provenance.json', provenance)
    print(json.dumps({key: provenance.get(key) for key in ('overallAccepted', 'probeExitCode', 'failureType', 'seconds')}), flush=True)
    return 0 if provenance['overallAccepted'] else 1


if __name__ == '__main__':
    def interrupted(_signal, _frame):
        raise RuntimeError('Owned controller interrupted; mandatory cleanup')
    signal.signal(signal.SIGTERM, interrupted)
    raise SystemExit(main())


