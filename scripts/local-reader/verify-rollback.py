#!/usr/bin/env python3
"""Bounded rollback rehearsal for one newly generated local Reader namespace."""
import argparse
import hashlib
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import secrets
import shutil
import stat
import subprocess
import sys
import urllib.parse
import urllib.request

sys.dont_write_bytecode = True
WORK = Path('/mnt/d/Reader-Pro-Local-Full-610-20261015-a')
MOUNT = Path('/mnt/reader-local-full-610')
DATA = MOUNT / 'data'
NAMESPACES = DATA / 'storage' / 'data'
ROOT = DATA / 'rollback-test'
SECRETS = ROOT / 'secrets'
SNAPSHOTS = ROOT / 'snapshots'
QUARANTINE = ROOT / 'quarantine'
BASE = 'http://127.0.0.1:18963'
MAX_BYTES = 10 * 1024 * 1024
MAX_FILES = 128
MAX_JSON = 256 * 1024
UNIT = 'reader-local-full-610.service'


def _lstat_or_none(path):
    try:
        return path.lstat()
    except FileNotFoundError:
        return None


def safe_directory(path, owner=None):
    value = path.lstat()
    if stat.S_ISLNK(value.st_mode) or not stat.S_ISDIR(value.st_mode):
        raise ValueError('unsafe generated directory')
    if owner is not None and value.st_uid != owner:
        raise ValueError('unexpected generated directory owner')
    return value


def safe_regular(path, mode=None, owner=None):
    value = path.lstat()
    if (not stat.S_ISREG(value.st_mode) or value.st_nlink != 1
            or (mode is not None and stat.S_IMODE(value.st_mode) != mode)
            or (owner is not None and value.st_uid != owner)):
        raise ValueError('unsafe generated file')
    return value


def read_json(path):
    safe_regular(path)
    if path.stat().st_size > MAX_JSON: raise ValueError('metadata exceeds bound')
    return json.loads(path.read_text(encoding='utf-8'))


def layout(create_rollback=False):
    """Validate only the fixed local mount and our own generated subtree."""
    safe_directory(MOUNT, 0)
    # WSL can stop naturally between explicit stopped-service phases. Reattach
    # only the already prepared exact data disk; never create/format a disk.
    source_directory = str(Path(__file__).resolve().parent)
    if source_directory not in sys.path:
        sys.path.insert(0, source_directory)
    import service
    service.verified_receipt()
    service.ensure_mount(WORK / 'data.ext4', DATA, service.prepare.DATA_BYTES)
    safe_directory(DATA, 0)
    safe_directory(DATA / 'storage', 10001)
    safe_directory(NAMESPACES, 10001)
    root_stat = _lstat_or_none(ROOT)
    if root_stat is None:
        if not create_rollback:
            raise ValueError('generated rollback root absent')
        ROOT.mkdir(mode=0o700)
    safe_directory(ROOT, 0)
    for directory in (SECRETS, SNAPSHOTS, QUARANTINE):
        value = _lstat_or_none(directory)
        if value is None:
            if create_rollback:
                directory.mkdir(mode=0o700)
            continue
        safe_directory(directory, 0)
    return True


def ensure_private_directory(path):
    """Create one direct child only after its fixed parent has been checked."""
    safe_directory(path.parent, 0)
    if _lstat_or_none(path) is not None:
        safe_directory(path, 0)
        return
    path.mkdir(mode=0o700)
    safe_directory(path, 0)


def safe_parent_chain(path):
    """Reject a redirected existing parent anywhere on the destination path."""
    current = path
    while True:
        safe_directory(current)
        if current == current.parent:
            return
        current = current.parent


def write_secret_json(path, value):
    safe_directory(path.parent, 0)
    descriptor = os.open(str(path), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, 'w', encoding='utf-8') as output:
            json.dump(value, output, separators=(',', ':'))
            output.flush()
            os.fsync(output.fileno())
    finally:
        # fdopen owns the descriptor when it succeeds; this is only for a
        # failure before that ownership transfer.
        try: os.close(descriptor)
        except OSError: pass
    safe_regular(path, 0o600, 0)


def generated_book_url(value, username):
    prefix = 'storage/data/' + username + '/'
    if not isinstance(value, str) or not value.startswith(prefix):
        raise ValueError('generated book escaped its namespace')
    raw_components = value.split('/')
    decoded_components = urllib.parse.unquote(value).split('/')
    if any(part in {'', '.', '..'} or '\\' in part or '\x00' in part
           for part in raw_components + decoded_components):
        raise ValueError('unsafe generated book path')
    return value


def unit_state(run=subprocess.run):
    """Check only the fixed local systemd unit; never enumerate services."""
    try:
        result = run(['systemctl', 'is-active', UNIT], stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5, check=False)
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise ValueError('cannot verify fixed local service unit') from exc
    if len(result.stdout) > 256 or len(result.stderr) > 256:
        raise ValueError('unit query output exceeded bound')
    state = result.stdout.decode('ascii', 'replace').strip()
    if state == 'inactive' and result.returncode == 4:
        # systemd-run --collect removes the transient unit after a clean stop.
        # Do not confuse that documented absence with an arbitrary query error.
        observed = run(['systemctl', 'show', UNIT, '-p', 'LoadState', '-p', 'ActiveState', '-p', 'SubState', '-p', 'MainPID'],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5, check=False)
        if observed.returncode != 0 or len(observed.stdout) > 512 or observed.stderr:
            raise ValueError('cannot verify collected service unit')
        fields = dict(line.split('=', 1) for line in observed.stdout.decode('ascii').splitlines())
        if fields != {'LoadState': 'not-found', 'ActiveState': 'inactive', 'SubState': 'dead', 'MainPID': '0'}:
            raise ValueError('collected service unit is not proven stopped')
        return 'inactive'
    expected = {'active': 0, 'inactive': 3, 'failed': 3}
    if state not in expected or result.returncode != expected[state]:
        raise ValueError('unit state unknown, activating, or inconsistent')
    return state


def unit_active(run=subprocess.run):
    return unit_state(run) == 'active'


def service_state(required, active_check=unit_active):
    prepare = read_json(WORK / 'prepare-receipt.json'); service = read_json(WORK / 'service-state.json')
    lock = prepare.get('lock', {})
    if prepare.get('accepted') is not True or prepare.get('executed') is not True or lock.get('version') != '6.1.0':
        raise ValueError('exact local prepare identity absent')
    state = active_check() if active_check is not unit_active else unit_state()
    okay = state == 'active' if required == 'running' else state in {'inactive', 'failed'}
    if service.get('status') != required or service.get('url') != 'http://127.0.0.1:18963/' or not okay:
        raise ValueError('service must be externally placed in required state')


def receipt_path(): return SECRETS / 'rollback-receipt.json'
def manifest_path(): return SECRETS / 'snapshot-manifest.json'
def pending_path(): return SECRETS / 'pending-credentials.json'
def credentials_path(): return SECRETS / 'credentials.json'


def read_receipt():
    layout()
    parent = safe_directory(SECRETS, 0)
    if stat.S_IMODE(parent.st_mode) != 0o700: raise ValueError('unsafe receipt parent')
    safe_regular(receipt_path(), 0o600, 0)
    receipt = read_json(receipt_path())
    name = receipt.get('username', '')
    if not isinstance(name, str) or not name.startswith('rollback610') or not name[11:].isalnum():
        raise ValueError('unsafe generated namespace')
    if receipt.get('namespace') != 'storage/data/' + name: raise ValueError('receipt namespace mismatch')
    safe_regular(credentials_path(), 0o600, 0); credentials = read_json(credentials_path())
    if credentials.get('username') != name or not isinstance(credentials.get('password'), str) or not credentials['password']:
        raise ValueError('generated credential receipt mismatch')
    generated_book_url(receipt.get('bookUrl'), name)
    return {**receipt, 'password': credentials['password']}


def tree(root, expected_uid=10001):
    """Inventory only one known generated namespace; reject every unsafe node."""
    root_stat = root.lstat()
    if not stat.S_ISDIR(root_stat.st_mode) or stat.S_ISLNK(root_stat.st_mode) or root_stat.st_uid != expected_uid:
        raise ValueError('unsafe namespace root')
    files, directory_records, total = [], [{'path': '', 'mode': stat.S_IMODE(root_stat.st_mode), 'uid': root_stat.st_uid,
        'gid': root_stat.st_gid}], 0
    for parent, walk_directories, names in os.walk(root, followlinks=False):
        current = Path(parent)
        for name in walk_directories + names:
            item = current / name; value = item.lstat(); relative = item.relative_to(root).as_posix()
            if (not relative or relative.startswith('../') or stat.S_ISLNK(value.st_mode)
                    or value.st_uid != expected_uid):
                raise ValueError('link or unsafe relative path')
            if stat.S_ISDIR(value.st_mode):
                directory_records.append({'path': relative, 'mode': stat.S_IMODE(value.st_mode), 'uid': value.st_uid, 'gid': value.st_gid})
                continue
            if not stat.S_ISREG(value.st_mode) or value.st_nlink != 1: raise ValueError('special or hardlinked namespace file')
            total += value.st_size
            if len(files) >= MAX_FILES or total > MAX_BYTES: raise ValueError('namespace backup bound exceeded')
            digest = hashlib.sha256(item.read_bytes()).hexdigest()
            files.append({'path': relative, 'sha256': digest, 'bytes': value.st_size,
                'mode': stat.S_IMODE(value.st_mode), 'uid': value.st_uid, 'gid': value.st_gid})
    return {'directories': sorted(directory_records, key=lambda item: item['path']),
        'files': sorted(files, key=lambda item: item['path']), 'bytes': total}


def copy_tree(source, target, manifest=None, chown=getattr(os, 'chown', None), expected_uid=10001):
    if chown is None: raise ValueError('namespace ownership restore requires Linux chown')
    inventory = tree(source, expected_uid)
    # All callers provide a fixed, validated direct parent.  Do not let
    # pathlib create arbitrary parent chains that a symlink could redirect.
    safe_parent_chain(target.parent)
    if _lstat_or_none(target) is not None: raise ValueError('refuse overwrite generated recovery path')
    target.mkdir(mode=0o700)
    safe_directory(target)
    expected = {item['path']: item for item in (manifest or inventory)['files']}
    if manifest is not None and inventory['directories'] != manifest.get('directories'):
        raise ValueError('snapshot directory metadata changed')
    for item in inventory['directories']:
        destination = target / item['path']
        if item['path']:
            safe_directory(destination.parent)
            destination.mkdir(mode=0o700)
        os.chmod(destination, item['mode']); chown(destination, item['uid'], item['gid'])
    for item in inventory['files']:
        if manifest is not None and expected.get(item['path']) != item: raise ValueError('snapshot source changed')
        destination = target / item['path']; safe_directory(destination.parent)
        shutil.copyfile(source / item['path'], destination, follow_symlinks=False)
        os.chmod(destination, item['mode']); chown(destination, item['uid'], item['gid'])
    result = tree(target, expected_uid)
    if result != (manifest or inventory): raise ValueError('copied generated namespace hash/mode mismatch')
    return result


def opener(): return urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def request(client, path, body=None):
    payload = None if body is None else json.dumps(body, separators=(',', ':')).encode()
    headers = {} if payload is None else {'Content-Type': 'application/json'}
    with client.open(urllib.request.Request(BASE + path, data=payload, method='POST' if payload is not None else 'GET', headers=headers), timeout=25) as response:
        raw = response.read(MAX_JSON + 1)
        if len(raw) > MAX_JSON: raise ValueError('Reader reply exceeds bound')
        value = json.loads(raw.decode())
        if response.status != 200 or value.get('isSuccess') is not True: raise RuntimeError('generated rollback API request rejected')
        return value.get('data')


def fixture():
    path = Path(__file__).resolve().parents[1] / 'epub-pressure' / 'fixture.py'
    spec = importlib.util.spec_from_file_location('rollback_generated_fixture', path)
    module = importlib.util.module_from_spec(spec); sys.modules[spec.name] = module; spec.loader.exec_module(module)
    return module


def multipart(path):
    boundary = 'rollback-generated-610'; data = path.read_bytes()
    body = (('--' + boundary + '\r\nContent-Disposition: form-data; name="file"; filename="rollback.epub"\r\nContent-Type: application/epub+zip\r\n\r\n').encode()
        + data + b'\r\n--' + boundary.encode() + b'--\r\n')
    return body, {'Content-Type': 'multipart/form-data; boundary=' + boundary}


def seed():
    service_state('running')
    layout(create_rollback=True)
    ensure_private_directory(SECRETS)
    if _lstat_or_none(receipt_path()) is not None: raise ValueError('generated rollback receipt already exists')
    # A pending receipt means a former seed may already have reached the API.
    # Do not ever turn it into an authorization to reuse an existing namespace.
    if _lstat_or_none(pending_path()) is not None:
        safe_regular(pending_path(), 0o600, 0)
        raise ValueError('pending generated credentials require manual review before retry')
    username = 'rollback610' + secrets.token_hex(4); password = 'Rollback-' + secrets.token_urlsafe(15)
    namespace = NAMESPACES / username
    if _lstat_or_none(namespace) is not None:
        raise ValueError('refuse pre-existing generated namespace')
    pending = {'username': username, 'password': password, 'namespace': 'storage/data/' + username}
    write_secret_json(pending_path(), pending)
    client = opener()
    try:
        request(client, '/reader3/login', {'username': username, 'password': password, 'isLogin': True})
    except RuntimeError:
        request(client, '/reader3/login', {'username': username, 'password': password, 'isLogin': False})
        request(client, '/reader3/login', {'username': username, 'password': password, 'isLogin': True})
    epub = ROOT / 'generated-small.epub'; generated = fixture(); info = generated.create(epub, generated.SMALL)
    body, headers = multipart(epub)
    with client.open(urllib.request.Request(BASE + '/reader3/importBookPreview', data=body, method='POST', headers=headers), timeout=30) as response:
        preview = json.loads(response.read(MAX_JSON).decode())
    if response.status != 200 or preview.get('isSuccess') is not True or not isinstance(preview.get('data'), list): raise RuntimeError('preview failed')
    book = preview['data'][0].get('book', {})
    saved = request(client, '/reader3/saveBook', book)
    if not isinstance(saved, dict): raise RuntimeError('save generated book did not return metadata')
    name = generated_book_url(saved.get('bookUrl'), username)
    chapters = request(client, '/reader3/getChapterList?url=' + urllib.parse.quote(name, safe=''))
    if not isinstance(chapters, list) or len(chapters) != 3: raise RuntimeError('generated three chapter import failed')
    receipt = {'username': username, 'namespace': 'storage/data/' + username, 'bookUrl': name,
        'bookName': book.get('name', ''), 'bookAuthor': book.get('author', ''),
        'fixtureSha256': info['sha256'], 'chapters': 3}
    write_secret_json(receipt_path(), receipt)
    if _lstat_or_none(credentials_path()) is not None: raise ValueError('generated credential receipt collision')
    pending_path().replace(credentials_path())
    safe_regular(credentials_path(), 0o600, 0)
    print(json.dumps({'phase': 'seed', 'generated': True, 'chapters': 3}, separators=(',', ':')))


def snapshot():
    service_state('stopped'); layout(create_rollback=True); receipt = read_receipt(); namespace = NAMESPACES / receipt['username']
    ensure_private_directory(SNAPSHOTS)
    target = SNAPSHOTS / receipt['username']; inventory = copy_tree(namespace, target)
    write_secret_json(manifest_path(), inventory)
    print(json.dumps({'phase': 'snapshot', 'fileCount': len(inventory['files']), 'bytes': inventory['bytes']}, separators=(',', ':')))


def damage():
    service_state('running'); layout(); receipt = read_receipt(); client = opener()
    request(client, '/reader3/login', {'username': receipt['username'], 'password': receipt['password'], 'isLogin': True})
    request(client, '/reader3/deleteBook', {'bookUrl': receipt['bookUrl'], 'name': receipt['bookName'], 'author': receipt['bookAuthor']})
    shelf = request(client, '/reader3/getBookshelf')
    if any(isinstance(item, dict) and item.get('bookUrl') == receipt['bookUrl'] for item in shelf): raise RuntimeError('generated book remained after damage')
    print(json.dumps({'phase': 'damage', 'generatedBookAbsent': True}, separators=(',', ':')))


def restore():
    service_state('stopped'); layout(create_rollback=True); receipt = read_receipt()
    source = SNAPSHOTS / receipt['username']; safe_directory(SNAPSHOTS, 0)
    safe_regular(manifest_path(), 0o600, 0); manifest = read_json(manifest_path())
    namespace = NAMESPACES / receipt['username']; ensure_private_directory(QUARANTINE)
    if _lstat_or_none(namespace) is None: raise ValueError('generated namespace missing before recoverable quarantine')
    # Confirm the currently damaged namespace is still exactly our UID-owned
    # generated subtree before any rename can affect it.
    tree(namespace)
    quarantine = QUARANTINE / (receipt['username'] + '-' + secrets.token_hex(4))
    temporary = NAMESPACES / (receipt['username'] + '.restore-' + secrets.token_hex(4))
    if _lstat_or_none(quarantine) is not None or _lstat_or_none(temporary) is not None:
        raise ValueError('generated recovery name collision')
    # Fully construct and verify a private replacement before moving the
    # damaged namespace.  A failed second rename restores the original and
    # retains the independently verified temporary copy for review.
    copy_tree(source, temporary, manifest)
    try:
        namespace.rename(quarantine)
    except OSError as exc:
        raise ValueError('could not quarantine generated namespace') from exc
    try:
        temporary.rename(namespace)
    except OSError as exc:
        if _lstat_or_none(namespace) is None:
            try:
                quarantine.rename(namespace)
            except OSError as restore_exc:
                raise ValueError('replacement and old namespace restoration both failed') from restore_exc
        raise ValueError('could not install restored generated namespace; old namespace restored') from exc
    if tree(namespace) != manifest:
        raise ValueError('installed restored namespace validation failed')
    print(json.dumps({'phase': 'restore', 'quarantined': True, 'fileCount': len(manifest['files'])}, separators=(',', ':')))


def verify():
    service_state('running'); layout(); receipt = read_receipt(); client = opener()
    request(client, '/reader3/login', {'username': receipt['username'], 'password': receipt['password'], 'isLogin': True})
    shelf = request(client, '/reader3/getBookshelf')
    book = next((item for item in shelf if isinstance(item, dict) and item.get('bookUrl') == receipt['bookUrl']), None)
    if book is None: raise RuntimeError('restored generated book absent')
    chapters = request(client, '/reader3/getChapterList?url=' + urllib.parse.quote(receipt['bookUrl'], safe=''))
    if not isinstance(chapters, list) or len(chapters) != 3: raise RuntimeError('restored chapter count incorrect')
    generated = fixture()
    for index in range(3):
        content = request(client, '/reader3/getBookContent?url=' + urllib.parse.quote(receipt['bookUrl'], safe='') + '&index=' + str(index) + '&epubContent=1')
        if (not isinstance(content, dict) or not isinstance(content.get('content'), str)
                or generated.title(index) not in content['content'] or '\ufffd' in content['content']):
            raise RuntimeError('restored generated chapter content/title invalid')
    license_data = request(client, '/reader3/getLicense')
    license = license_data.get('license') if isinstance(license_data, dict) else None
    if not isinstance(license, dict) or license.get('verified') is not True: raise RuntimeError('generated local license not verified')
    print(json.dumps({'phase': 'verify', 'namespaceMatches': True, 'chaptersVerified': 3, 'titleVerified': True,
        'licenseVerified': True}, separators=(',', ':')))


PHASES = {'seed': seed, 'snapshot': snapshot, 'damage': damage, 'restore': restore, 'verify': verify}
def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('phase', choices=sorted(PHASES)); parser.add_argument('--execute', action='store_true')
    args = parser.parse_args(argv)
    if not args.execute:
        print(json.dumps({'phase': args.phase, 'planned': True, 'mutating': False, 'scope': 'one-generated-namespace'}, separators=(',', ':'))); return 0
    PHASES[args.phase](); return 0
if __name__ == '__main__': raise SystemExit(main())
