#!/usr/bin/env python3
"""One-shot, generated-data acceptance for the locally prepared Reader image.

Default mode is a no-write plan.  ``--execute`` is deliberately required before
it creates the fixed generated account/book/license records.  It reads only
its generated book, never private books, browser profiles or production data.
"""
import argparse
import hashlib
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import stat
import sys
import urllib.request
import urllib.parse
import uuid
import threading

sys.dont_write_bytecode = True
WORK = Path('/mnt/d/Reader-Pro-Local-Full-610-20261015-a')
MOUNT = Path('/mnt/reader-local-full-610')
BASE = 'http://127.0.0.1:18963'
LICENSE = 'http://127.0.0.1:18964'
ACCOUNT = 'localfull610'
PASSWORD = 'Local-Test-610'
RESULT = WORK / 'verification-initial.json'
DATA = MOUNT / 'data' / 'generated-test'
SECRET_DIR = MOUNT / 'data' / 'secrets'
MAX_JSON = 256 * 1024


def regular(path, mode=None):
    value = path.lstat()
    if not stat.S_ISREG(value.st_mode) or value.st_nlink != 1 or (mode is not None and stat.S_IMODE(value.st_mode) != mode):
        raise ValueError('unsafe fixed file')
    return value


def secret(name):
    parent = SECRET_DIR.lstat()
    if not stat.S_ISDIR(parent.st_mode) or stat.S_IMODE(parent.st_mode) != 0o700:
        raise ValueError('unsafe secret directory')
    path = SECRET_DIR / name
    regular(path, 0o600)
    value = path.read_text(encoding='utf-8').rstrip('\n')
    if not 24 <= len(value) <= 4096 or '\n' in value or '\r' in value:
        raise ValueError('invalid local secret')
    return value


def read_json(path):
    regular(path)
    if path.stat().st_size > MAX_JSON:
        raise ValueError('metadata too large')
    return json.loads(path.read_text(encoding='utf-8'))


def verify_identity():
    """Do not turn a reachable port into an exact-image conclusion."""
    prepare = read_json(WORK / 'prepare-receipt.json')
    service = read_json(WORK / 'service-state.json')
    expected = {'accepted': True, 'executed': True}
    if any(prepare.get(key) != value for key, value in expected.items()):
        raise ValueError('prepare receipt not accepted')
    lock = prepare.get('lock', {})
    if lock.get('version') != '6.1.0' or lock.get('jarSha256') != 'b4954ac70229f4514146b8e077909874bfc5aeee779f71fde903622638ed1ee0':
        raise ValueError('prepared image identity mismatch')
    if (service.get('status') != 'running' or service.get('url') != 'http://127.0.0.1:18963/'
            or service.get('generatedNetworkTesting') is not True):
        raise ValueError('local service state not running on fixed loopback port')
    return {'prepareAccepted': True, 'serviceStateRunning': True, 'port': 18963,
            'jarSha256Matches': True}


def request(opener, url, method='GET', body=None, headers=None):
    payload = None if body is None else json.dumps(body, separators=(',', ':')).encode('utf-8')
    request_headers = {'Content-Type': 'application/json'} if payload is not None else {}
    request_headers.update(headers or {})
    with opener.open(urllib.request.Request(url, data=payload, method=method, headers=request_headers), timeout=35) as response:
        data = response.read(MAX_JSON + 1)
        if len(data) > MAX_JSON:
            raise ValueError('response too large')
        return response.status, json.loads(data.decode('utf-8'))


def local_opener(cookie_jar=None):
    handlers = [urllib.request.ProxyHandler({})]
    if cookie_jar is not None:
        handlers.append(urllib.request.HTTPCookieProcessor(cookie_jar))
    return urllib.request.build_opener(*handlers)


def success(response):
    status, value = response
    if status != 200 or value.get('isSuccess') is not True:
        raise RuntimeError('Reader API rejected generated verification request')
    return value.get('data')


def issue_and_import(opener, secure_key):
    installed = success(request(opener, BASE + '/reader3/getLicense'))
    if isinstance(installed, dict):
        prior = installed.get('license', {})
        if prior.get('host') == '127.0.0.1,localhost' and prior.get('type') == 'pro' and prior.get('verified') is True:
            return True
    token = secret('license-admin-token')
    _status, issued = request(local_opener(), LICENSE + '/v1/legacy/licenses', 'POST',
        {'host': '127.0.0.1,localhost', 'type': 'pro', 'userMaxLimit': 15, 'instances': 1,
         'code': 'local-' + uuid.uuid4().hex}, {'Authorization': 'Bearer ' + token})
    content = issued.get('content') if isinstance(issued, dict) else None
    if not isinstance(content, str) or not content:
        raise RuntimeError('local license service did not issue generated legacy content')
    data = success(request(opener, BASE + '/reader3/importLicense', 'POST', {'content': content},
        {'X-Reader-Secure-Key': secure_key}))
    return (isinstance(data, dict) and data.get('license', {}).get('verified') is True
            and data['license'].get('host') == '127.0.0.1,localhost')


def load_fixture():
    fixture_path = Path(__file__).resolve().parents[1] / 'epub-pressure' / 'fixture.py'
    spec = importlib.util.spec_from_file_location('generated_epub_fixture', fixture_path)
    module = importlib.util.module_from_spec(spec); sys.modules[spec.name] = module; spec.loader.exec_module(module)
    return module


def load_smoke():
    path = Path(__file__).resolve().parents[1] / 'smoke-local-webview.py'
    spec = importlib.util.spec_from_file_location('local_generated_smoke_definitions', path)
    module = importlib.util.module_from_spec(spec); sys.modules[spec.name] = module; spec.loader.exec_module(module)
    return module


def embedded_browser_once(opener):
    """One real Reader/Camoufox request against an existing generated fixture."""
    smoke = load_smoke()
    fixture = smoke.Fixture(('127.0.0.1', 0))
    thread = threading.Thread(target=fixture.serve_forever, daemon=True); thread.start()
    source_url = 'http://127.0.0.1:' + str(fixture.server_port)
    source = {'bookSourceUrl': source_url, 'bookSourceName': 'Generated local delayed JS',
        'enabledCookieJar': True,
        'searchUrl': source_url + '/search-async, ' + json.dumps({'webView': True, 'webJs': smoke.async_source_script('get')}),
        'ruleSearch': {'bookList': '.book', 'name': '.name@text', 'author': '.author@text', 'bookUrl': 'a@href'},
        'ruleToc': {'chapterList': '.chapter'}, 'ruleContent': {'content': '.content@html'}}
    try:
        success(request(opener, BASE + '/reader3/saveBookSource', 'POST', source))
        books = success(request(opener, BASE + '/reader3/searchBook', 'POST',
            {'key': 'generated-delayed-js', 'page': 1, 'bookSourceUrl': source_url}))
        if (not isinstance(books, list) or len(books) != 1 or books[0].get('name') != 'WebView异步书'
                or len(fixture.async_marks) != 1):
            raise RuntimeError('embedded browser did not complete generated delayed JS')
        return {'requested': True, 'resultCount': 1, 'delayedJsObserved': True, 'sourceCleaned': False}
    finally:
        try:
            success(request(opener, BASE + '/reader3/deleteBookSource', 'POST', source))
        finally:
            fixture.shutdown(); fixture.server_close(); thread.join(timeout=5)


def multipart_epub(path):
    boundary = 'reader-generated-610'
    raw = path.read_bytes()
    body = (('--' + boundary + '\r\nContent-Disposition: form-data; name="file"; filename="generated-small.epub"\r\n'
             'Content-Type: application/epub+zip\r\n\r\n').encode() + raw + b'\r\n--' + boundary.encode() + b'--\r\n')
    return body, {'Content-Type': 'multipart/form-data; boundary=' + boundary}


def post_multipart(opener, path, body, headers):
    with opener.open(urllib.request.Request(BASE + path, data=body, method='POST', headers=headers), timeout=30) as response:
        value = json.loads(response.read(MAX_JSON).decode('utf-8'))
        if response.status != 200 or value.get('isSuccess') is not True:
            raise RuntimeError('generated EPUB preview rejected')
        return value.get('data')


def execute():
    if RESULT.exists():
        raise ValueError('existing verification result is immutable; refuse re-registration')
    identity = verify_identity()
    secure_key = secret('reader-admin-key')
    opener = local_opener(http.cookiejar.CookieJar())
    # Register, reject wrong password, logout, and log in again using only the
    # fixed generated account.  Do not record access tokens/cookies.
    # A prior interrupted generated run may already have this fixed account;
    # first login makes the verifier resumable without probing any user account.
    current = request(opener, BASE + '/reader3/login', 'POST', {'username': ACCOUNT, 'password': PASSWORD, 'isLogin': True})
    if current[1].get('isSuccess') is not True:
        success(request(opener, BASE + '/reader3/login', 'POST', {'username': ACCOUNT, 'password': PASSWORD, 'isLogin': False}))
    wrong_status, wrong = request(opener, BASE + '/reader3/login', 'POST', {'username': ACCOUNT, 'password': PASSWORD + '-wrong', 'isLogin': True})
    if wrong_status != 200 or wrong.get('isSuccess') is not False or wrong.get('errorMsg') != '密码错误':
        raise RuntimeError('wrong generated password was accepted')
    logged = success(request(opener, BASE + '/reader3/login', 'POST', {'username': ACCOUNT, 'password': PASSWORD, 'isLogin': True}))
    if not isinstance(logged, dict) or not isinstance(logged.get('accessToken'), str):
        raise RuntimeError('generated login missing token')
    success(request(opener, BASE + '/reader3/logout?accessToken=' + urllib.parse.quote(logged['accessToken'], safe=''), 'POST', {}))
    success(request(opener, BASE + '/reader3/login', 'POST', {'username': ACCOUNT, 'password': PASSWORD, 'isLogin': True}))
    license_imported = issue_and_import(opener, secure_key)
    if not license_imported:
        raise RuntimeError('generated license not verified')
    DATA.mkdir(mode=0o700, parents=True, exist_ok=True)
    fixture = load_fixture(); epub = DATA / 'generated-small.epub'
    fixture_info = fixture.verify(epub, fixture.SMALL) if epub.exists() else fixture.create(epub, fixture.SMALL)
    preview = post_multipart(opener, '/reader3/importBookPreview', *multipart_epub(epub))
    books = preview if isinstance(preview, list) else []
    if not books or not isinstance(books[0], dict) or not isinstance(books[0].get('book'), dict):
        raise RuntimeError('generated EPUB preview lacks book')
    # saveBook moves the preview from its temporary URL to persistent storage.
    # Read the server's returned book, exactly as the real UI does.
    book = success(request(opener, BASE + '/reader3/saveBook', 'POST', books[0]['book']))
    if not isinstance(book, dict) or not str(book.get('bookUrl', '')).startswith('storage/data/' + ACCOUNT + '/'):
        raise RuntimeError('saved generated book namespace mismatch')
    escaped = urllib.parse.quote(str(book.get('bookUrl', '')), safe='')
    chapters = success(request(opener, BASE + '/reader3/getChapterList?url=' + escaped))
    if not isinstance(chapters, list) or len(chapters) != 3:
        raise RuntimeError('generated EPUB did not expose three chapters')
    for index in range(3):
        content = success(request(opener, BASE + '/reader3/getBookContent?url=' + escaped + '&index=' + str(index) + '&epubContent=1'))
        if (not isinstance(content, dict) or fixture.title(index) not in content.get('content', '')
                or '\ufffd' in content['content'] or chapters[index].get('title') != fixture.title(index)):
            raise RuntimeError('generated chapter title or UTF-8 mismatch')
    browser = embedded_browser_once(opener)
    browser['sourceCleaned'] = True
    report = {'schemaVersion': 1, 'scope': 'local-generated-initial-acceptance', 'identity': identity,
        'generatedOnly': True, 'privateBookInputRead': False, 'realCredentialsImported': False,
        'licenseImported': license_imported, 'authLifecycle': {'register': True, 'wrongPasswordRejected': True,
            'logout': True, 'relogin': True}, 'epub': {'chaptersVerified': 3,
            'fixtureSha256': fixture_info['sha256'], 'fixtureBytes': fixture_info['bytes']},
        'embeddedBrowser': browser, 'uiManualBookRetained': True,
        'automaticProductionAcceptance': False}
    with RESULT.open('x', encoding='utf-8') as stream:
        stream.write(json.dumps(report, sort_keys=True, indent=2) + '\n')
    print(json.dumps({'completed': True, 'chaptersVerified': 3, 'licenseImported': True}, separators=(',', ':')))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execute', action='store_true')
    args = parser.parse_args(argv)
    if not args.execute:
        print(json.dumps({'planned': True, 'mutating': False, 'readerPort': 18963, 'generatedOnly': True}, separators=(',', ':')))
        return 0
    execute(); return 0


if __name__ == '__main__':
    raise SystemExit(main())
