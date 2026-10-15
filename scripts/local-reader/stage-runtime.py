#!/usr/bin/env python3
"""Freeze small verified local runtime dependencies on D:, without overwriting."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time
sys.dont_write_bytecode = True
import prepare

SERVICE_UNIT = 'reader-local-full-610.service'
SOURCE_NAMES = ('service.py', 'prepare.py', 'init.py', 'ReaderLocal.ps1')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--refresh-source', action='store_true', help='Back up and replace only previously verified controller source')
    args = parser.parse_args()
    if not args.execute:
        print(json.dumps({'planned': True, 'writes': False})); return
    work = Path(prepare.WORK_DIR)
    target = work / 'runtime-software'
    if args.refresh_source:
        refresh_source(target)
        return
    if target.exists():
        raise ValueError('durable software already exists; never overwrite it implicitly')
    repo = prepare.repo_root()
    old = repo / 'build/local-camoufox-image-20261004-a/tools/root/usr'
    inputs = [(old / 'bin/runc', 'runc/bin/runc', 'dc61317da24db892bd614ef8af0ec54f22b2b8235e0fcdcda51b3a82e4ee61c5'),
              (Path('/usr/lib/x86_64-linux-gnu/libseccomp.so.2.6.0'), 'runc/lib/libseccomp.so.2', 'df995d80c824876afc38528b5e71d258e1844eb524cd9d34f9a7898562fd99e7'),
              (repo / 'license-server/build/libs/reader-license-server-0.1.0-all.jar', 'license/server.jar', '7d348ecbb121b50ff76f96a8f84b1f4a01f1d74f3923026a55d533b138eb4d53')]
    source_root = Path(__file__).resolve().parent
    for name in ('service.py', 'prepare.py', 'init.py', 'ReaderLocal.ps1'):
        source = source_root / name
        inputs.append((source, 'source/' + name, prepare.sha256_file(source)))
    for source, relative, digest in inputs:
        if not source.is_file() or source.is_symlink() or prepare.sha256_file(source) != digest:
            raise ValueError('software input checksum mismatch')
    target.mkdir(mode=0o700)
    records = []
    for source, relative, digest in inputs:
        output = target / relative
        output.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        with source.open('rb') as original, output.open('xb') as copied:
            shutil.copyfileobj(original, copied)
        output.chmod(0o755 if relative == 'runc/bin/runc' else 0o644)
        if prepare.sha256_file(output) != digest:
            raise ValueError('durable copy checksum mismatch')
        records.append(dict(path=relative, bytes=output.stat().st_size, sha256=digest))
    with (target / 'runtime-software.lock.json').open('x') as stream:
        json.dump(dict(schemaVersion=1, files=records, requires='Ubuntu WSL2 with systemd and glibc',
                       independentOfRepositoryBuildCaches=True), stream, indent=2, sort_keys=True)
    print(json.dumps({'stagedFiles': len(records), 'bytes': sum(x['bytes'] for x in records)}))


def _regular(path, label):
    item = path.lstat()
    if not stat.S_ISREG(item.st_mode) or path.is_symlink():
        raise ValueError(label + ' must be a regular non-symlink file')
    return item


def _directory(path, label, create=False):
    if create and not path.exists():
        path.mkdir(mode=0o700, parents=True)
    item = path.lstat()
    if not stat.S_ISDIR(item.st_mode) or path.is_symlink():
        raise ValueError(label + ' must be a directory, never a symlink')
    return item


def _copy_atomic(source, destination, digest, mode=0o644):
    """Copy one previously-hashed source without exposing partial output."""
    before = _regular(source, 'source')
    reader_fd = os.open(source, os.O_RDONLY | getattr(os, 'O_NOFOLLOW', 0))
    opened = os.fstat(reader_fd); after = source.lstat()
    if ((before.st_dev, before.st_ino) != (after.st_dev, after.st_ino) or
            (before.st_dev, before.st_ino) != (opened.st_dev, opened.st_ino) or
            not stat.S_ISREG(opened.st_mode)):
        os.close(reader_fd); raise ValueError('source path changed during open')
    _directory(destination.parent, 'destination parent')
    temporary = destination.with_name(destination.name + '.next-' + str(os.getpid()) + '-' + str(time.time_ns()))
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_NOFOLLOW', 0), mode)
    try:
        copied = hashlib.sha256()
        with os.fdopen(reader_fd, 'rb', closefd=False) as reader, os.fdopen(descriptor, 'wb', closefd=False) as writer:
            for block in iter(lambda: reader.read(1024 * 1024), b''):
                copied.update(block); writer.write(block)
            writer.flush(); os.fsync(writer.fileno())
        if copied.hexdigest() != digest or prepare.sha256_file(temporary) != digest:
            raise ValueError('source changed while being copied')
        os.chmod(temporary, mode)
        # DrvFS may delay publishing a renamed file until its open handle is
        # closed. Close after fsync and before replace, on Linux as on Windows.
        os.close(descriptor); descriptor = None
        os.replace(temporary, destination)
        return destination.stat().st_size
    except BaseException:
        try: temporary.unlink()
        except FileNotFoundError: pass
        raise
    finally:
        try: os.close(reader_fd)
        except OSError: pass
        if descriptor is not None:
            try: os.close(descriptor)
            except OSError: pass


def _write_manifest_atomic(path, value):
    _directory(path.parent, 'software root')
    temporary = path.with_name(path.name + '.next-' + str(os.getpid()) + '-' + str(time.time_ns()))
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_NOFOLLOW', 0), 0o644)
    try:
        with os.fdopen(descriptor, 'w', encoding='utf-8', closefd=False) as stream:
            json.dump(value, stream, indent=2, sort_keys=True); stream.write('\n')
            stream.flush(); os.fsync(stream.fileno())
        _regular(temporary, 'new manifest')
        os.close(descriptor); descriptor = None
        os.replace(temporary, path)
    except BaseException:
        try: temporary.unlink()
        except FileNotFoundError: pass
        raise
    finally:
        if descriptor is not None:
            try: os.close(descriptor)
            except OSError: pass


def _unit_must_be_inactive(unit=SERVICE_UNIT):
    result = subprocess.run(['systemctl', 'is-active', unit], text=True, capture_output=True, check=False, timeout=5)
    if result.stdout.strip() != 'inactive':
        raise ValueError('local Reader service must be inactive before source refresh')


def _refresh_lock(work):
    _directory(work, 'work directory')
    lock = work / 'service-controller.lock'
    if lock.exists(): _regular(lock, 'service controller lock')
    descriptor = os.open(lock, os.O_RDWR | os.O_CREAT | getattr(os, 'O_NOFOLLOW', 0), 0o600)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BaseException:
        os.close(descriptor); raise ValueError('local Reader controller is busy; source refresh refused')
    return descriptor


def refresh_source(target, *, work=None, source_root=None, unit=SERVICE_UNIT):
    """Create an immutable old-source snapshot, then atomically publish new hashes.

    Source files become temporarily inconsistent with the old lock during a
    successful refresh; the old lock intentionally makes that detectable.
    The lock itself is replaced only after every copied source is verified.
    """
    work = Path(prepare.WORK_DIR) if work is None else Path(work)
    source_root = Path(__file__).parent if source_root is None else Path(source_root)
    target = Path(target)
    if target != work / 'runtime-software' or target.is_symlink():
        raise ValueError('unsafe software root')
    _directory(target, 'software root'); _directory(source_root, 'controller source directory')
    controller = _refresh_lock(work)
    try:
        _unit_must_be_inactive(unit)
        lock = target / 'runtime-software.lock.json'
        _regular(lock, 'runtime manifest')
        old = json.loads(lock.read_bytes())
        expected = {'runc/bin/runc', 'runc/lib/libseccomp.so.2', 'license/server.jar',
                    'source/service.py', 'source/prepare.py', 'source/init.py', 'source/ReaderLocal.ps1'}
        if {item['path'] for item in old['files']} != expected or len(old['files']) != len(expected):
            raise ValueError('unexpected software manifest')
        _directory(target / 'source', 'durable controller source')
        for item in old['files']:
            path = target / item['path']
            _regular(path, 'durable software')
            if prepare.sha256_file(path) != item['sha256']:
                raise ValueError('durable software was changed; refuse overwrite')
        # Validate all inputs before preserving/replacing anything, so a bad
        # late source cannot produce a half-refresh.
        new_sources = {}
        for name in SOURCE_NAMES:
            source = source_root / name
            _regular(source, 'controller source')
            new_sources[name] = (source, prepare.sha256_file(source), source.stat().st_size)
        history_root = target / 'source-history'
        _directory(history_root, 'source history', create=True)
        history = history_root / str(time.time_ns())
        history.mkdir(mode=0o700)
        _directory(history, 'new source history')
        _copy_atomic(lock, history / 'runtime-software.lock.json', prepare.sha256_file(lock))
        records = []
        old_by_path = {item['path']: item for item in old['files']}
        for name in SOURCE_NAMES:
            relative = 'source/' + name
            prior = target / relative
            _copy_atomic(prior, history / name, old_by_path[relative]['sha256'])
        # Every history file now exists and is verified.  Replacements can be
        # made one by one; until the final manifest swap, the mismatch is
        # intentionally fail-closed for later service launches.
        for item in old['files']:
            if item['path'].startswith('source/'):
                name = Path(item['path']).name
                source, digest, size = new_sources[name]
                _copy_atomic(source, target / item['path'], digest)
                records.append(dict(path=item['path'], bytes=size, sha256=digest))
            else:
                records.append(dict(item))
        old['files'] = sorted(records, key=lambda item: item['path'])
        _write_manifest_atomic(lock, old)
        print(json.dumps({'sourceRefreshed': True, 'previousSourcePreserved': True}))
    finally:
        os.close(controller)


if __name__ == '__main__':
    main()
