#!/usr/bin/env python3
"""Local WSL service for the pinned Reader image. Never formats or deletes data."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import signal
import socket
import stat
import subprocess
import sys
import time
import urllib.request

sys.dont_write_bytecode = True
import prepare

WORK = Path('/mnt/d/Reader-Pro-Local-Full-610-20261015-a')
MOUNT = Path('/mnt/reader-local-full-610')
UNIT = 'reader-local-full-610.service'
NAME = 'reader-local-full-610'
LICENSE_SHA = '7d348ecbb121b50ff76f96a8f84b1f4a01f1d74f3923026a55d533b138eb4d53'
RUNC_SHA = 'dc61317da24db892bd614ef8af0ec54f22b2b8235e0fcdcda51b3a82e4ee61c5'
SECCOMP_SHA = 'df995d80c824876afc38528b5e71d258e1844eb524cd9d34f9a7898562fd99e7'
MIB = 1024 ** 2


def command(*args, timeout=30, check=True, env=None):
    return subprocess.run([str(x) for x in args], capture_output=True, text=True,
                          check=check, timeout=timeout, env=env)


def runtime_software():
    """Startup uses durable D: software, never a disposable repository build cache."""
    root = WORK / 'runtime-software'
    paths = {'runc': root / 'runc/bin/runc', 'library': root / 'runc/lib/libseccomp.so.2',
             'license': root / 'license/server.jar'}
    expected = {'runc': RUNC_SHA, 'library': SECCOMP_SHA, 'license': LICENSE_SHA}
    required = {'runc/bin/runc', 'runc/lib/libseccomp.so.2', 'license/server.jar',
                'source/service.py', 'source/prepare.py', 'source/init.py', 'source/ReaderLocal.ps1'}
    manifest = metadata(root / 'runtime-software.lock.json')
    if manifest.get('schemaVersion') != 1 or not isinstance(manifest.get('files'), list):
        raise ValueError('durable runtime software manifest mismatch')
    records = manifest['files']
    if len(records) != len(required) or {item.get('path') for item in records if isinstance(item, dict)} != required:
        raise ValueError('durable runtime software manifest mismatch')
    for record in records:
        path = record.get('path')
        digest = record.get('sha256')
        size = record.get('bytes')
        if (not isinstance(path, str) or not isinstance(digest, str) or not re.fullmatch(r'[0-9a-f]{64}', digest) or
                not isinstance(size, int) or size < 0):
            raise ValueError('durable runtime software manifest mismatch')
        item = (root / path).lstat()
        if (not stat.S_ISREG(item.st_mode) or item.st_nlink != 1 or item.st_size != size or
                prepare.sha256_file(root / path) != digest):
            raise ValueError('durable runtime software identity mismatch')
    for name, path in paths.items():
        item = path.lstat()
        if not stat.S_ISREG(item.st_mode) or item.st_nlink != 1 or prepare.sha256_file(path) != expected[name]:
            raise ValueError('durable runtime software identity mismatch')
    if (Path(__file__).resolve() != (root / 'source/service.py').resolve() or
            Path(prepare.__file__).resolve() != (root / 'source/prepare.py').resolve() or
            Path(__file__).with_name('init.py').resolve() != (root / 'source/init.py').resolve()):
        raise ValueError('durable runtime controller source mismatch')
    return paths


def metadata(path):
    value = path.lstat()
    if not stat.S_ISREG(value.st_mode) or value.st_nlink != 1 or value.st_size > MIB:
        raise ValueError('unsafe metadata file')
    return json.loads(path.read_bytes())


def write_state(value):
    temporary = WORK / ('state-' + secrets.token_hex(6) + '.tmp')
    with temporary.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
    temporary.replace(WORK / 'service-state.json')


def verified_receipt():
    value = metadata(WORK / 'prepare-receipt.json')
    if value.get('accepted') is not True or value.get('executed') is not True or value.get('lock') != prepare.LOCK:
        raise ValueError('preparation identity mismatch')
    expected = {'imageDisk': str(WORK / 'image.ext4'), 'dataDisk': str(WORK / 'data.ext4'),
                'mountRoot': str(MOUNT), 'workDir': str(WORK),
                'rootfs': str(MOUNT / 'image/exact-image/bundle/rootfs'),
                'originalConfigPath': str(WORK / 'docker-config.original.json')}
    if any(value.get(k) != v for k, v in expected.items()):
        raise ValueError('preparation path mismatch')
    config_path = WORK / 'docker-config.original.json'
    if prepare.sha256_file(config_path) != prepare.LOCK['imageId'].split(':')[1]:
        raise ValueError('original image config mismatch')
    return value


def ensure_mount(disk, target, size):
    """Reattach only a previously prepared exact disk; no mkfs path exists."""
    item = disk.lstat()
    if not stat.S_ISREG(item.st_mode) or item.st_size != size:
        raise ValueError('prepared disk missing or wrong size')
    if target.is_symlink():
        raise ValueError('mount target is a link')
    target.mkdir(mode=0o700, parents=True, exist_ok=True)
    loops = json.loads(command('losetup', '--json', '--list', '--output', 'NAME,BACK-FILE').stdout)['loopdevices']
    matches = [x['name'] for x in loops if x['back-file'] == str(disk)]
    if len(matches) > 1:
        raise ValueError('ambiguous disk loop attachment')
    mounted = command('mountpoint', '-q', target, check=False).returncode == 0
    if mounted:
        source = command('findmnt', '-n', '-o', 'SOURCE', '--mountpoint', target).stdout.strip()
        if matches != [source]:
            raise ValueError('existing mount does not belong to prepared disk')
    else:
        if any(target.iterdir()):
            raise ValueError('refuse to cover existing files')
        source = matches[0] if matches else command('losetup', '--find', '--show', disk).stdout.strip()
        if not re.fullmatch(r'/dev/loop[0-9]+', source):
            raise ValueError('invalid loop device')
        command('mount', '-t', 'ext4', '-o', 'nodev,nosuid', source, target)
    if command('findmnt', '-n', '-o', 'FSTYPE', '--mountpoint', target).stdout.strip() != 'ext4':
        raise ValueError('expected ext4')


def owned_directory(path, uid=10001, mode=0o700):
    if not path.exists():
        path.mkdir(mode=mode)
        os.chown(path, uid, uid)
        path.chmod(mode)
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != uid or stat.S_IMODE(info.st_mode) != mode:
        raise ValueError('persistent directory ownership mismatch')


def persistent_data():
    data = MOUNT / 'data'
    for name in ('storage', 'logs', 'license-logs', 'license-data', 'secrets'):
        owned_directory(data / name)
    for name in ('license-admin-token', 'reader-admin-key'):
        target = data / 'secrets' / name
        if not target.exists():
            descriptor = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(descriptor, 'w') as stream:
                stream.write(secrets.token_urlsafe(36) + '\n')
            os.chown(target, 10001, 10001)
        info = target.lstat()
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_uid != 10001 or stat.S_IMODE(info.st_mode) != 0o600:
            raise ValueError('secret permissions mismatch')


def runtime_env(original, generated_network=False):
    env = dict(item.split('=', 1) for item in original)
    for key in ('LICENSE_ADMIN_TOKEN', 'READER_APP_SECUREKEY', 'READER_APP_LICENSEPUBLICKEY'):
        env.pop(key, None)
    env.update(READER_APP_WORKDIR='/', READER_SERVER_BINDADDRESS='127.0.0.1', READER_SERVER_PORT='18963',
               READER_APP_SECURE='true', READER_APP_LICENSECHECKENABLED='true',
               READER_APP_SHELFUPDATEINTEVAL='0', READER_APP_REMOTEBOOKSOURCEUPDATEINTERVAL='0',
               READER_APP_AUTOBACKUPUSERDATA='false', READER_APP_AUTOCLEARINACTIVEUSER='0')
    # Only an explicit temporary test run may reach the generated loopback source.
    env['READER_BROWSER_ALLOW_PRIVATE_NETWORKS'] = 'true' if generated_network else 'false'
    return [key + '=' + value for key, value in env.items()]


def bind(destination, source, writable=False):
    return dict(destination=destination, type='bind', source=str(source),
                options=['rbind', 'rw' if writable else 'ro', 'nodev', 'nosuid', 'noexec'])


def tmpfs(destination, size, mode='1777'):
    return dict(destination=destination, type='tmpfs', source='tmpfs',
                options=['nosuid', 'nodev', 'size=' + str(size), 'mode=' + mode])


def config(rootfs, frozen, env, role, run_name):
    data = MOUNT / 'data'
    if role not in ('license', 'reader'):
        raise ValueError('invalid OCI role')
    mounts = [tmpfs('/tmp', 256 * MIB), tmpfs('/dev', 65536), tmpfs('/dev/shm', 512 * MIB),
              bind('/local-license', frozen),
              dict(destination='/proc', type='proc', source='proc', options=['nosuid', 'nodev', 'noexec']),
              dict(destination='/sys', type='sysfs', source='sysfs', options=['ro', 'nosuid', 'nodev', 'noexec']),
              dict(destination='/sys/fs/cgroup', type='cgroup', source='cgroup', options=['ro', 'nosuid', 'nodev', 'noexec'])]
    secret_name = 'license-admin-token' if role == 'license' else 'reader-admin-key'
    mounts.append(bind('/local-secrets/' + secret_name, data / 'secrets' / secret_name))
    if role == 'license':
        mounts.extend((bind('/license-data', data / 'license-data', True), bind('/logs', data / 'license-logs', True)))
    else:
        mounts.extend((bind('/storage', data / 'storage', True), bind('/logs', data / 'logs', True)))
    for device in ('/dev/null', '/dev/zero', '/dev/urandom'):
        mounts.append(dict(destination=device, type='bind', source=device, options=['bind', 'nosuid', 'noexec']))
    return dict(ociVersion='1.0.2', hostname=NAME, root=dict(path=str(rootfs), readonly=False),
        process=dict(terminal=False, user=dict(uid=10001, gid=10001, additionalGids=[]),
                     args=['/usr/bin/python3', '/local-license/init.py', '--role', role], cwd='/app', env=env,
                     noNewPrivileges=True, capabilities={key: [] for key in ('bounding', 'effective', 'permitted', 'inheritable', 'ambient')}),
        mounts=mounts, linux=dict(namespaces=[dict(type=name) for name in ('mount', 'pid', 'ipc', 'uts', 'cgroup')],
        cgroupsPath='/system.slice/' + UNIT + '/' + role,
        resources=dict(cpu=dict(quota=200000, period=100000, cpus='0-1'),
                       memory=dict(limit=2147483648, swap=2147483648), pids=dict(limit=256))))


def overlay(runroot, lower):
    info = lower.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != 0 or stat.S_IMODE(info.st_mode) != 0o755:
        raise ValueError('image root metadata mismatch')
    for name in ('upper', 'work', 'merged'):
        (runroot / name).mkdir(mode=0o700)
    (runroot / 'upper').chmod(0o755)
    merged = runroot / 'merged'
    command('mount', '-t', 'overlay', 'overlay', '-o',
            'lowerdir=' + str(lower) + ',upperdir=' + str(runroot / 'upper') + ',workdir=' + str(runroot / 'work'), merged)
    return merged


def setup():
    receipt = verified_receipt()
    if not MOUNT.exists():
        MOUNT.mkdir(mode=0o700)
    mount_info = MOUNT.lstat()
    if not stat.S_ISDIR(mount_info.st_mode) or mount_info.st_uid != 0 or stat.S_IMODE(mount_info.st_mode) != 0o700:
        raise ValueError('owned mount root metadata mismatch')
    ensure_mount(WORK / 'image.ext4', MOUNT / 'image', prepare.IMAGE_BYTES)
    ensure_mount(WORK / 'data.ext4', MOUNT / 'data', prepare.DATA_BYTES)
    lower = Path(receipt['rootfs'])
    if prepare.sha256_file(lower / 'app/reader.jar') != prepare.LOCK['jarSha256']:
        raise ValueError('image JAR mismatch')
    persistent_data()
    for port in (18963, 18964):
        with socket.socket() as check:
            # A clean shutdown may leave accepted connections in TIME_WAIT.
            # This still rejects an active listener, without misidentifying
            # TIME_WAIT as a second live service during immediate restart.
            check.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            check.bind(('127.0.0.1', port))
    runs = MOUNT / 'image/service-runs'
    runs.mkdir(mode=0o700, exist_ok=True)
    runroot = runs / ('run-' + secrets.token_hex(8)); runroot.mkdir(mode=0o700)
    frozen = runroot / 'frozen'; frozen.mkdir(mode=0o755)
    license_jar = runtime_software()['license']
    for source, name in ((license_jar, 'server.jar'), (Path(__file__).with_name('init.py'), 'init.py')):
        shutil.copyfile(source, frozen / name); (frozen / name).chmod(0o644)
    state = dict(schemaVersion=1, scope=NAME, status='starting', runRoot=str(runroot),
                 jarSha256=prepare.LOCK['jarSha256'], configSha256=prepare.LOCK['imageId'],
                 licenseJarSha256=LICENSE_SHA, supervisorSha256=prepare.sha256_file(frozen / 'init.py'),
                 url='http://127.0.0.1:18963/', network='WSL host network; both services bind loopback',
                 persistentDataPath=str(MOUNT / 'data'), privateDataRead=False)
    write_state(state)
    return lower, runroot, frozen, state


def worker(generated_network=False):
    if Path('/proc/self/cgroup').read_text().strip() != '0::/system.slice/' + UNIT:
        raise ValueError('worker outside owned limited unit')
    group = Path('/sys/fs/cgroup/system.slice') / UNIT
    controller = group / 'controller'; controller.mkdir()
    (controller / 'cgroup.procs').write_text(str(os.getpid()))
    (group / 'cgroup.subtree_control').write_text('+cpu +cpuset +memory +pids')
    lower, base_runroot, frozen, state = setup()
    state['generatedNetworkTesting'] = generated_network
    merged = None
    tools = runtime_software()
    env = dict(os.environ, LD_LIBRARY_PATH=str(tools['library'].parent), GOMAXPROCS='2')
    children = {}
    runroots = {}
    def runc(role, *args, **kw):
        return command(tools['runc'], '--root', runroots[role] / 'runc-state', *args, env=env, **kw)
    stopping = False
    def stop_signal(_signal, _frame):
        nonlocal stopping
        stopping = True
    signal.signal(signal.SIGTERM, stop_signal); signal.signal(signal.SIGINT, stop_signal)
    def start_role(role):
        runroot = base_runroot / role
        runroot.mkdir(mode=0o700)
        runroots[role] = runroot
        # Record this before mounting: failed overlay setup must still take
        # the same narrow cleanup path without indexing a missing child.
        children[role] = (None, None, runroot / 'merged')
        merged = overlay(runroot, lower)
        children[role] = (None, None, merged)
        # OCI bind targets must exist in this role's disposable overlay.  Each
        # role gets only its own single secret file, never the shared directory.
        (merged / 'local-secrets').mkdir(mode=0o755)
        original = metadata(WORK / 'docker-config.original.json')['config']['Env']
        bundle = runroot / 'bundle'; bundle.mkdir(mode=0o700)
        role_env = runtime_env(original, generated_network) if role == 'reader' else []
        (bundle / 'config.json').write_text(json.dumps(config(merged, frozen, role_env, role, NAME + '-' + role)))
        log = (runroot / 'lifecycle.log').open('xb')
        child = subprocess.Popen([str(tools['runc']), '--root', str(runroot / 'runc-state'), 'run', '--bundle', str(bundle), NAME + '-' + role],
                                 stdin=subprocess.DEVNULL, stdout=log, stderr=log, env=env)
        children[role] = (child, log, merged)
        return child

    def stop_role(role):
        item = children.get(role)
        if item is None:
            return
        child, log, _merged = item
        if child is None:
            return
        if child.poll() is None:
            runc(role, 'kill', NAME + '-' + role, 'TERM', check=False)
            try:
                child.wait(timeout=30)
            except subprocess.TimeoutExpired:
                runc(role, 'kill', '--all', NAME + '-' + role, 'KILL', check=False)
                try: child.wait(timeout=10)
                except subprocess.TimeoutExpired: pass
        runc(role, 'delete', '--force', NAME + '-' + role, check=False)
        if log is not None:
            log.close()

    try:
        license_child = start_role('license')
        # The reader PID 1 performs its own bounded health/key wait; this
        # parent only rejects an immediately failed isolated license process.
        deadline = time.monotonic() + 10
        while license_child.poll() is None and time.monotonic() < deadline:
            try:
                with urllib.request.build_opener(urllib.request.ProxyHandler({})).open('http://127.0.0.1:18964/healthz', timeout=1) as response:
                    if response.status == 200: break
            except OSError:
                time.sleep(.1)
        if license_child.poll() is not None:
            raise RuntimeError('isolated license process exited before reader start')
        reader_child = start_role('reader')
        state.update(status='running', startedAt=int(time.time()), roles=['license', 'reader']); write_state(state)
        while reader_child.poll() is None and license_child.poll() is None and not stopping:
            time.sleep(.5)
        if stopping and reader_child.poll() is None:
            runc('reader', 'kill', NAME + '-reader', 'TERM', check=False)
            try: reader_child.wait(timeout=30)
            except subprocess.TimeoutExpired: pass
        state['exitCode'] = reader_child.poll() if reader_child.poll() is not None else license_child.poll()
    finally:
        for role in ('reader', 'license'):
            stop_role(role)
        # Remove only this disposable overlay after its exact mount is gone.
        if base_runroot.parent != MOUNT / 'image/service-runs' or not re.fullmatch(r'run-[0-9a-f]{16}', base_runroot.name):
            raise ValueError('unsafe overlay cleanup root')
        # Also remove partially-created directories when overlay mount failed.
        for role, runroot in runroots.items():
            _child, _log, merged = children.get(role, (None, None, runroot / 'merged'))
            if command('mountpoint', '-q', merged, check=False).returncode == 0:
                command('umount', merged)
            for name in ('upper', 'work', 'merged'):
                target = runroot / name
                if target.is_symlink() or command('mountpoint', '-q', target, check=False).returncode == 0:
                    raise ValueError('unsafe overlay cleanup target')
                if target.exists(): shutil.rmtree(target)
        state.update(status='stopped', stoppedAt=int(time.time()), persistentDataRetained=True)
        state['memoryPeakBytes'] = int((group / 'memory.peak').read_text())
        state['memoryEvents'] = dict(line.split() for line in (group / 'memory.events').read_text().splitlines())
        write_state(state)
    return state.get('exitCode', 1)


def status():
    value = metadata(WORK / 'service-state.json') if (WORK / 'service-state.json').exists() else {'status': 'not-started'}
    value['unitActive'] = command('systemctl', 'is-active', UNIT, check=False).stdout.strip()
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    try:
        with opener.open('http://127.0.0.1:18963/reader3/getSystemInfo', timeout=3) as response:
            body = json.loads(response.read(131072))
            value['readerHttpStatus'] = response.status
            value['readerIsSuccess'] = body.get('isSuccess') is True
    except (OSError, ValueError):
        value['readerIsSuccess'] = False
    print(json.dumps(value, indent=2, sort_keys=True))
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('start', 'status', 'stop', '_worker'))
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--generated-network', action='store_true', help='Temporary generated-loopback test only; disabled by default')
    args = parser.parse_args()
    if args.action == 'status': return status()
    if not args.execute or os.geteuid() != 0:
        raise ValueError('mutating actions require WSL root and --execute')
    if args.action == '_worker': return worker(args.generated_network)
    if args.action == 'stop':
        verified_receipt()
        command('systemctl', 'stop', UNIT, timeout=55)
        return status()
    verified_receipt()
    with (WORK / 'service-controller.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if command('systemctl', 'is-active', '--quiet', UNIT, check=False).returncode == 0:
            raise ValueError('owned service already active')
        argv = ['systemd-run', '--unit=' + UNIT, '--pipe', '--wait', '--collect', '--quiet',
                '--property=Delegate=yes', '--property=AllowedCPUs=0-1', '--property=CPUQuota=200%',
                '--property=MemoryMax=2147483648', '--property=MemoryHigh=1610612736',
                '--property=MemorySwapMax=0', '--property=TasksMax=512',
                '--property=KillMode=mixed', '--property=TimeoutStopSec=45',
                '--', '/usr/bin/python3', str(Path(__file__).resolve()), '_worker', '--execute']
        if args.generated_network:
            argv.append('--generated-network')
        return subprocess.call(argv)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except Exception as error:
        # Do not serialize exception values or credentials into outer logs.
        print(json.dumps({'event': 'local-service-failed', 'errorType': type(error).__name__,
                          'errno': getattr(error, 'errno', None)}))
        raise SystemExit(1)
