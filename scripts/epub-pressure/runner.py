#!/usr/bin/env python3
"""Linux/systemd: sequential fresh-image generated UI rounds, aggregate 2 CPU/2 GiB."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time

SOURCES = ('fixture.py', 'generated_account.py', 'main.py', 'ui.py', 'pressure.py',
           'storage_budget.py', 'worker_network.py', 'file_network.py', 'epub-frame-readiness.js')
SOURCE = Path(__file__).resolve().parent


def cmd(*args, timeout=30, check=True):
    return subprocess.run(list(map(str, args)), capture_output=True, text=True, check=check, timeout=timeout)


def write_new(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2)


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''):
            result.update(block)
    return result.hexdigest()


def generated_container_ids(token):
    assert re.fullmatch('readergen[0-9a-f]{12}', token)
    rows = cmd('docker', 'ps', '--no-trunc', '-aq', '--filter',
               'label=com.medwarp.reader.generated-pressure=' + token).stdout.splitlines()
    assert len(rows) == len(set(rows)) and all(re.fullmatch('[0-9a-f]{64}', row) for row in rows)
    return rows


def budget(group):
    quota, period = map(int, (group / 'cpu.max').read_text().split())
    result = dict(cpuQuota=quota, cpuPeriod=period,
                  memoryMaxBytes=int((group / 'memory.max').read_text()),
                  memoryHighBytes=int((group / 'memory.high').read_text()),
                  memoryPeakBytes=int((group / 'memory.peak').read_text()),
                  swapMaxBytes=int((group / 'memory.swap.max').read_text()),
                  pidsMax=int((group / 'pids.max').read_text()),
                  memoryEvents=dict((k, int(v)) for k, v in (line.split() for line in (group / 'memory.events').read_text().splitlines())),
                  pidsEvents=dict((k, int(v)) for k, v in (line.split() for line in (group / 'pids.events').read_text().splitlines())))
    assert period > 0 and quota == period * 2 and result['memoryMaxBytes'] == 2147483648
    assert result['memoryHighBytes'] == 1610612736 and result['swapMaxBytes'] == 0 and result['pidsMax'] == 512
    return result


def worker(config_path):
    config_path = config_path.resolve(strict=True)
    config = json.loads(config_path.read_bytes())
    root, token = config_path.parent, config['token']
    assert re.fullmatch('readergen[0-9a-f]{12}', token)
    assert root.name.startswith('reader-epub-pressure-') and not root.is_symlink()
    parent = token + '.slice'
    assert Path('/proc/self/cgroup').read_text().strip().startswith('0::/' + parent + '/')
    group = Path('/sys/fs/cgroup') / parent
    assert {name: digest(root / 'sources' / name) for name in config['sourceHashes']} == config['sourceHashes']
    data, output = root / 'data', root / 'output'
    for directory in (data, output):
        directory.mkdir(mode=0o700)
        os.chown(directory, 10001, 10001)
    provenance = dict(schemaVersion=1, scope='aggregate-default-image-generated-epub-pressure',
                      token=token, imageId=config['imageId'], revision=config['revision'],
                      jarSha256=config['jarSha256'], architecture=config['architecture'],
                      generatedLargeEpubAccepted=False, privateBookInputRead=False,
                      realCredentialsImported=False, sourceHashes=config['sourceHashes'],
                      controllerIncludedInAggregateBudget=True, ownedContainerRemoved=False,
                      generatedDataCleared=False, hostGlobalCachesChanged=False,
                      freshProcessAndStorage=True, hostPageCacheColdProven=False)
    cid, observation, exit_code = None, None, 2

    def owned():
        states = json.loads(cmd('docker', 'container', 'inspect', cid).stdout)
        assert len(states) == 1 and states[0]['Id'] == cid
        assert states[0]['Config']['Labels']['com.medwarp.reader.generated-pressure'] == token
        assert states[0]['Config']['Image'] == config['imageId']
        return states[0]

    try:
        budget(group)
        cid = cmd('docker', 'create', '--name', token, '--cgroup-parent', parent, '--network=none', '--init',
                  '--cpus=2', '--memory=2g', '--memory-swap=2g', '--pids-limit=256', '--shm-size=512m',
                  '--cap-drop=ALL', '--security-opt=no-new-privileges:true', '--restart=no', '--user=10001:10001',
                  '--label', 'com.medwarp.reader.generated-pressure=' + token,
                  '--tmpfs', '/tmp:size=256m,mode=1777',
                  '--tmpfs', '/storage:size=16m,uid=10001,gid=10001,mode=700',
                  '--mount', 'type=bind,src=' + str(root / 'sources') + ',dst=/verification,readonly',
                  '--mount', 'type=bind,src=' + str(data) + ',dst=/verification-data',
                  '--mount', 'type=bind,src=' + str(output) + ',dst=/verification-output',
                  '--entrypoint=/usr/bin/python3', config['imageId'], '/verification/main.py',
                  '--revision', config['revision'], '--architecture', config['architecture'],
                  '--jar-sha', config['jarSha256']).stdout.strip()
        assert re.fullmatch('[0-9a-f]{64}', cid)
        state = owned()
        assert state['HostConfig']['NetworkMode'] == 'none' and not state['HostConfig']['PortBindings']
        assert state['HostConfig']['MemorySwap'] == state['HostConfig']['Memory'] == 2147483648
        assert state['HostConfig']['NanoCpus'] == 2000000000 and state['HostConfig']['PidsLimit'] == 256
        mounts = state['Mounts']
        binds = [m for m in mounts if m['Type'] == 'bind']
        assert len(binds) == 3 and {m['Destination'] for m in binds} == {'/verification', '/verification-data', '/verification-output'}
        assert next(m for m in binds if m['Destination'] == '/verification')['RW'] is False
        assert all(m['Type'] == 'bind' or (m['Type'] == 'tmpfs' and m['Destination'] in ('/tmp', '/storage')) for m in mounts)
        started = time.monotonic()
        cmd('docker', 'start', cid)
        state = owned()
        assert Path('/proc/' + str(state['State']['Pid']) + '/cgroup').read_text().strip().startswith('0::/' + parent + '/')
        print(json.dumps(dict(phase='generated-ui-started', token=token)), flush=True)
        while owned()['State']['Running']:
            if time.monotonic() - started >= 220:
                raise TimeoutError('Generated UI aggregate execution expired')
            time.sleep(1)
        state = owned()
        provenance['containerExitCode'] = state['State']['ExitCode']
        provenance['containerOomKilled'] = state['State']['OOMKilled']
        path = output / 'observation.json'
        assert path.is_file() and not path.is_symlink() and path.stat().st_size <= 1048576
        observation = json.loads(path.read_bytes())
        write_new(root / 'observation.json', observation)
        provenance['aggregateBudget'] = budget(group)
        assert provenance['containerExitCode'] == 0 and provenance['containerOomKilled'] is False
        assert observation['passed'] is True and observation['readerEightScenesAccepted'] is True
        assert all(provenance['aggregateBudget']['memoryEvents'].get(k, 0) == 0 for k in ('max', 'oom', 'oom_kill'))
        assert provenance['aggregateBudget']['pidsEvents'].get('max', 0) == 0
        provenance['generatedLargeEpubAccepted'], exit_code = True, 0
    except Exception as error:
        provenance['failureType'] = type(error).__name__
    finally:
        if group.exists() and 'aggregateBudget' not in provenance:
            provenance['aggregateBudget'] = budget(group)
        if cid is not None:
            owned()
            cmd('docker', 'rm', '--force', cid)
            provenance['ownedContainerRemoved'] = generated_container_ids(token) == []
        # This newly minted volume contains generated data only. Refuse symlinks,
        # hardlinks, foreign owners or paths outside that exact disposable target.
        assert data.resolve(strict=True) == data and data.parent == root and data.name == 'data'
        size = count = 0
        for path in data.rglob('*'):
            assert not path.is_symlink() and path.resolve(strict=True).is_relative_to(data)
            info = path.stat()
            assert info.st_uid == 10001
            if path.is_file():
                assert info.st_nlink == 1
                size, count = size + info.st_size, count + 1
        shutil.rmtree(data)
        provenance['generatedDataCleared'] = not data.exists()
        provenance['generatedBytesRemoved'], provenance['generatedFilesRemoved'] = size, count
        provenance['workerExitCode'] = exit_code
        write_new(root / 'provenance.json', provenance)
    return exit_code


def dispatch(args):
    assert sys.platform == 'linux' and os.geteuid() == 0 and not sys.flags.optimize
    assert re.fullmatch('[0-9a-f]{40}', args.revision) and re.fullmatch('[0-9a-f]{64}', args.jar_sha)
    work = args.work_root.resolve(strict=True)
    assert work.is_dir() and not work.is_symlink() and len(work.parts) >= 3
    assert not args.report.exists() and not args.report.is_symlink()
    assert cmd('docker', 'info', '--format', '{{.CgroupDriver}}').stdout.strip() == 'systemd'
    images = json.loads(cmd('docker', 'image', 'inspect', args.image).stdout)
    assert len(images) == 1
    image = images[0]
    assert image['Architecture'] == args.architecture and image['Os'] == 'linux'
    assert image['Config']['User'] == '10001:10001' and image['Config']['WorkingDir'] == '/app'
    assert image['Config']['Entrypoint'] == ['/usr/local/bin/reader-entrypoint']
    assert re.fullmatch('sha256:[0-9a-f]{64}', image['Id'])
    for key, value in {'READER_BUILD_REVISION': args.revision, 'READER_APP_WEBUI': 'vue3',
                       'READER_APP_WEBVIEWRENDERER': 'camoufox',
                       'JAVA_TOOL_OPTIONS': '-Xms256m -Xmx768m -Dfile.encoding=UTF-8'}.items():
        assert [v for v in image['Config']['Env'] if v.startswith(key + '=')] == [key + '=' + value]
    rounds, successful = [], True
    for index in range(3):
        root = Path(tempfile.mkdtemp(prefix='reader-epub-pressure-', dir=work)).resolve(strict=True)
        root.chmod(0o750)
        os.chown(root, 0, 10001)
        frozen = root / 'sources'
        frozen.mkdir(mode=0o750)
        os.chown(frozen, 0, 10001)
        source_hashes = {}
        source_paths = {name: SOURCE / name for name in SOURCES}
        source_paths['verify-reader-default-ui.py'] = SOURCE.parent / 'verify-reader-default-ui.py'
        for name, source in source_paths.items():
            assert source.is_file() and not source.is_symlink() and source.stat().st_size < 65536
            raw = source.read_bytes()
            if name.endswith('.py'):
                compile(raw, name, 'exec')
            target = frozen / name
            with target.open('xb') as stream:
                stream.write(raw)
            target.chmod(0o640)
            os.chown(target, 0, 10001)
            source_hashes[name] = hashlib.sha256(raw).hexdigest()
        token = 'readergen' + os.urandom(6).hex()
        parent, anchor, unit = token + '.slice', token + '-anchor.service', token + '-controller.service'
        config = dict(token=token, imageId=image['Id'], revision=args.revision, jarSha256=args.jar_sha,
                      architecture=args.architecture, sourceHashes=source_hashes)
        write_new(root / 'config.json', config)
        row = dict(round=index + 1, freshProcessAndStorage=True, executionReturnCode=None, accepted=False)
        try:
            cmd('systemd-run', '--unit=' + anchor, '--slice=' + parent, '--property=Type=oneshot',
                '--property=RemainAfterExit=yes', '/bin/true')
            cmd('systemctl', 'set-property', '--runtime', parent, 'CPUQuota=200%',
                'MemoryHigh=1610612736', 'MemoryMax=2147483648', 'MemorySwapMax=0', 'TasksMax=512')
            budget(Path('/sys/fs/cgroup') / parent)
            finished = cmd('systemd-run', '--wait', '--pipe', '--collect', '--unit=' + unit,
                           '--slice=' + parent, '--property=Type=exec', '--property=RuntimeMaxSec=240',
                           '--property=KillMode=mixed', '--property=TimeoutStopSec=20', sys.executable, str(Path(__file__).resolve()),
                           '--worker-config', str(root / 'config.json'), timeout=260, check=False)
            row['executionReturnCode'] = finished.returncode
            row['provenance'] = json.loads((root / 'provenance.json').read_bytes())
            observation_path = root / 'observation.json'
            if observation_path.exists():
                row['observation'] = json.loads(observation_path.read_bytes())
            row['accepted'] = finished.returncode == 0 and row['provenance']['generatedLargeEpubAccepted'] is True
        except Exception as error:
            row['dispatchFailureType'] = type(error).__name__
        finally:
            cmd('systemctl', 'stop', unit, timeout=30, check=False)
            # RuntimeMaxSec/OOM may stop the worker before its own finally. Only
            # exact label + immutable image + frozen mounts authorize fallback.
            stranded = generated_container_ids(token)
            for cid in stranded:
                assert re.fullmatch('[0-9a-f]{64}', cid)
                state = json.loads(cmd('docker', 'container', 'inspect', cid).stdout)[0]
                assert state['Config']['Labels']['com.medwarp.reader.generated-pressure'] == token
                # Config.Image retains the exact immutable create argument even
                # on stores whose State.Image addresses a different object kind.
                assert state['Config']['Image'] == image['Id'] and state['HostConfig']['CgroupParent'] == parent
                assert next(m for m in state['Mounts'] if m['Destination'] == '/verification')['Source'] == str(frozen)
                cmd('docker', 'rm', '--force', cid)
            row['fallbackOwnedContainersRemoved'] = len(stranded)
            group = Path('/sys/fs/cgroup') / parent
            # Direct members may be empty while descendants are still populated.
            if group.exists():
                assert 'populated 0' in (group / 'cgroup.events').read_text().splitlines()
            cmd('systemctl', 'stop', anchor, parent, timeout=30, check=False)
            row['parentEmptyAndStopped'] = cmd('systemctl', 'is-active', parent, check=False).stdout.strip() in ('inactive', 'unknown')
        rounds.append(row)
        successful = successful and row['accepted'] and row['parentEmptyAndStopped']
        print(json.dumps(dict(round=index + 1, accepted=row['accepted'], parentStopped=row['parentEmptyAndStopped'])), flush=True)
        if not successful:
            break
    value = dict(schemaVersion=1, scope='three-fresh-default-image-generated-large-epub-rounds',
                 imageId=image['Id'], revision=args.revision, jarSha256=args.jar_sha, architecture=args.architecture,
                 accepted=successful and len(rounds) == 3, rounds=rounds,
                 privateBookInputRead=False, realAccountAuthenticationTested=False,
                 originalJarAccountCompatibilityTested=False, hostGlobalCachesChanged=False,
                 hostPageCacheColdProven=False, productionOrLongRunProven=False)
    write_new(args.report, value)
    return 0 if value['accepted'] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker-config', type=Path)
    parser.add_argument('--image')
    parser.add_argument('--revision')
    parser.add_argument('--jar-sha')
    parser.add_argument('--architecture', choices=('amd64', 'arm64'))
    parser.add_argument('--work-root', type=Path)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    if args.worker_config is not None:
        assert os.geteuid() == 0 and sys.platform == 'linux' and not sys.flags.optimize
        return worker(args.worker_config)
    if any(getattr(args, key) is None for key in ('image', 'revision', 'jar_sha', 'architecture', 'work_root', 'report')):
        parser.error('Exact image/revision/JAR/architecture and fresh output paths are required')
    return dispatch(args)


if __name__ == '__main__':
    raise SystemExit(main())
