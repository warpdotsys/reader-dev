"""Exact original/remote/Camoufox replay of one captured public HTML page.

No WAN, real credentials or private books. This is HTTP loopback snapshot
compatibility, NOT real-site HTTPS, real login or current whole-image proof.
Raw chapters remain in memory; reports contain only types/counts/hashes.
"""
import argparse
import copy
import hashlib
import http.cookiejar
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import importlib.util
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request

ORIGINAL_SHA = 'b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c'
RESTORED_SHA = 'f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b'
CAPTURE_SHA = '23403a6a3b1cace42a06d6c378a6f1acf8ec97c940c652d12fe3c0d30275e58f'
PAGE_PATH = '/cache/epub/84/pg84-images.html'
PUBLIC_URL = 'https://www.gutenberg.org' + PAGE_PATH
MAX_RESPONSE = 128 * 1024
CLOCK_FIELDS = frozenset(('latestChapterTime', 'lastCheckTime', 'durChapterTime'))
FRAGMENTS = tuple('letter' + str(i) for i in range(1, 5)) + tuple('chap' + str(i).zfill(2) for i in range(1, 25))
OPTIONS = ', {"webView":true}'


class Rejected(ValueError):
    pass


def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'), allow_nan=False).encode('utf-8')


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def file_sha(path):
    # The packaged browser runtime is Python 3.10; file_digest needs 3.11.
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def source_definition(base):
    parsed = urllib.parse.urlsplit(base)
    if parsed.scheme != 'http' or parsed.hostname != '127.0.0.1' or parsed.path or not parsed.port or parsed.username or parsed.password or parsed.query or parsed.fragment:
        raise Rejected('NotPrivateLoopbackAlias')
    source = copy.deepcopy(helper('probe-gutenberg-reader').SOURCE_DEFINITION)
    source['bookSourceUrl'] = base
    source['bookSourceName'] = 'Captured Gutenberg 84 loopback replay'
    source['ruleBookInfo']['tocUrl'] = "@js:'" + base + PAGE_PATH + OPTIONS + "'"
    return source


def validate_metadata(value, window):
    if not isinstance(value, dict) or value.get('isSuccess') is not True or value.get('errorMsg') != '':
        raise Rejected('MetadataEnvelopeRejected')
    book = value.get('data')
    if not isinstance(book, dict) or 'Frankenstein' not in str(book.get('name', '')) or 'Shelley' not in str(book.get('author', '')):
        raise Rejected('PublicMetadataMissing')
    if set(window) != {'started', 'completed'} or any(type(v) is not int for v in window.values()) or not 0 < window['started'] <= window['completed'] <= window['started'] + 35000:
        raise Rejected('ActualRequestWindowMissing')
    if any(type(book.get(k)) is not int or not window['started'] <= book[k] <= window['completed'] for k in CLOCK_FIELDS):
        raise Rejected('DefaultClockNotFresh')
    result = dict(value)
    result['data'] = {k: v for k, v in book.items() if k not in CLOCK_FIELDS}
    return result


def validate_toc(value, book):
    if not isinstance(value, list) or len(value) != 28:
        raise Rejected('PublicTocNot28')
    for i, row in enumerate(value):
        if not isinstance(row, dict) or type(row.get('index')) is not int or row['index'] != i or row.get('isVolume') is not False:
            raise Rejected('TocIndexOrDefaultsDiffer')
        url = row.get('url')
        if not isinstance(url, str) or url != '#' + FRAGMENTS[i] + OPTIONS or row.get('baseUrl') != book:
            raise Rejected('TocRawRelativeUrlOrOptionsDiffer')
        if not isinstance(row.get('title'), str) or not row['title'].strip():
            raise Rejected('TocTitleMissing')


def validate_content(value):
    if not isinstance(value, str) or not 1000 <= len(value) <= MAX_RESPONSE or '\ufffd' in value or '<script' in value.lower():
        raise Rejected('EmptyOrInvalidPublicChapter')
    if 'To Mrs. Saville, England.' not in value or value.strip().startswith(('http://', 'https://')):
        raise Rejected('ExpectedLetterBodyMissing')


def compare(sides):
    if set(sides) != {'original', 'restored', 'camoufox'}:
        raise Rejected('MissingActualSide')
    statics = [validate_metadata(side['metadata'], side['metadataWindowMs']) for side in sides.values()]
    for side in sides.values():
        if any(side.get(key) is not True for key in ('sourceRulesRetained', 'readerStopped', 'logoutSucceeded')):
            raise Rejected('SideLifecycleNotAccepted')
        if set(side['httpStatuses']) != {'metadata', 'toc', 'chapter0', 'chapter1'} or any(type(v) is not int or v != 200 for v in side['httpStatuses'].values()):
            raise Rejected('SideHttpStatusDiffers')
        if side['businessApiCalls'] != {'metadata': 1, 'toc': 1, 'chapter0': 1, 'chapter1': 1} or any(type(v) is not int for v in side['businessApiCalls'].values()):
            raise Rejected('UnexpectedBusinessCalls')
        url = side['metadata']['data'].get('bookUrl')
        if not isinstance(url, str) or not url.endswith(OPTIONS):
            raise Rejected('ActualBookUrlMissing')
        for key in ('toc', 'chapter0', 'chapter1'):
            value = side[key]
            if not isinstance(value, dict) or value.get('isSuccess') is not True or value.get('errorMsg') != '':
                raise Rejected('OperationEnvelopeRejected')
            if key == 'toc':
                validate_toc(value['data'], url[:-len(OPTIONS)])
            else:
                validate_content(value['data'])
    for operation in ('toc', 'chapter0', 'chapter1'):
        rows = [side[operation] for side in sides.values()]
        if len({canonical(value) for value in rows}) != 1:
            raise Rejected('CompleteReturnDataDiffers-' + operation)
    if len({canonical(value) for value in statics}) != 1:
        raise Rejected('CompleteNonClockMetadataDiffers')
    return {'nonClockMetadataFullJsonEqual': True,
        'literalFullMetadataEqual': len({canonical(side['metadata']) for side in sides.values()}) == 1,
        'defaultClocksFreshInOwnActualWindows': True,
        'tocFullReturnDataEqual': True, 'firstTwoChaptersFullReturnDataEqual': True,
        'jsonTypesAndAllNonClockFieldsCompared': True}


def facts(side):
    static = validate_metadata(side['metadata'], side['metadataWindowMs'])
    result = {'metadataNonClockReturnDataSha256': digest(static),
        'metadataRawReturnDataSha256': digest(side['metadata']),
        'metadataDataFields': sorted(side['metadata']['data']),
        'metadataWindowMs': side['metadataWindowMs'],
        'clocks': {key: side['metadata']['data'][key] for key in sorted(CLOCK_FIELDS)},
        'sourceRulesRetained': side['sourceRulesRetained'],
        'httpStatuses': side['httpStatuses'], 'businessApiCalls': side['businessApiCalls'],
        'readerStopped': side['readerStopped'], 'logoutSucceeded': side['logoutSucceeded'],
        'chapterCount': len(side['toc']['data']), 'operations': {}}
    for key in ('metadata', 'toc', 'chapter0', 'chapter1'):
        value = side[key]
        result['operations'][key] = {'isSuccess': value['isSuccess'], 'errorMsgEmpty': value['errorMsg'] == '',
            'fields': sorted(value), 'fullReturnDataSha256': digest(value)}
        if key.startswith('chapter'):
            result['operations'][key]['characters'] = len(value['data'])
            result['operations'][key]['bodySha256'] = hashlib.sha256(value['data'].encode()).hexdigest()
            result['operations'][key]['replacementCharacters'] = value['data'].count('\ufffd')
    return result


def validated_restored_sha256(value):
    try:
        return helper('probe-gutenberg-reader').validated_jar_sha256(value)
    except ValueError:
        raise Rejected('InvalidRestoredJarIdentity') from None


def validate_report(report, expected_restored_sha256=RESTORED_SHA):
    """Independently reject partial/identity-mismatched exported observations."""
    expected_restored = validated_restored_sha256(expected_restored_sha256)
    if report.get('passed') is not True or report.get('originalExecuted') is not True or set(report.get('sides', {})) != {'original', 'restored', 'camoufox'}:
        raise Rejected('IncompleteThreeWayReport')
    if any(report.get(k) != v for k, v in {'originalJarSha256': ORIGINAL_SHA, 'restoredJarSha256': expected_restored,
            'captureSha256': CAPTURE_SHA, 'captureBytes': 468370, 'capturedPublicUrl': PUBLIC_URL}.items()):
        raise Rejected('ReportInputIdentityDiffers')
    if any(report.get(k) is not False for k in ('realSiteHttpsThreeWayProven', 'realAuthenticationProven', 'rawBodyOrCredentialsExported', 'fullGoalComplete')) or report.get('privateBooksRead') != 0:
        raise Rejected('ReportEvidenceScopeChanged')
    expected = {'nonClockMetadataFullJsonEqual', 'defaultClocksFreshInOwnActualWindows',
        'tocFullReturnDataEqual', 'firstTwoChaptersFullReturnDataEqual', 'jsonTypesAndAllNonClockFieldsCompared'}
    if any(report.get('comparison', {}).get(k) is not True for k in expected):
        raise Rejected('MissingFullJsonComparison')
    sides = list(report['sides'].values())
    if len({row['metadataNonClockReturnDataSha256'] for row in sides}) != 1:
        raise Rejected('StaticMetadataHashesDiffer')
    for key in ('toc', 'chapter0', 'chapter1'):
        if len({row['operations'][key]['fullReturnDataSha256'] for row in sides}) != 1:
            raise Rejected('OperationHashesDiffer')
    for row in sides:
        if row['chapterCount'] != 28 or any(row.get(k) is not True for k in ('sourceRulesRetained', 'readerStopped', 'logoutSucceeded')):
            raise Rejected('IncompleteSideFacts')
        if row['httpStatuses'] != {k: 200 for k in ('metadata', 'toc', 'chapter0', 'chapter1')} or row['businessApiCalls'] != {k: 1 for k in ('metadata', 'toc', 'chapter0', 'chapter1')}:
            raise Rejected('ApiFactsDiffer')
        window = row['metadataWindowMs']
        if not 0 < window['started'] <= window['completed'] <= window['started'] + 35000 or set(row['clocks']) != CLOCK_FIELDS or any(type(v) is not int or not window['started'] <= v <= window['completed'] for v in row['clocks'].values()):
            raise Rejected('ReportedClocksNotFresh')
        for key, value in row['operations'].items():
            if value['isSuccess'] is not True or value['errorMsgEmpty'] is not True:
                raise Rejected('ReportedEnvelopeDiffers')
            if key.startswith('chapter') and (not 1000 <= value['characters'] <= MAX_RESPONSE or value['replacementCharacters'] != 0):
                raise Rejected('ReportedChapterMissing')
    return True


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise Rejected('UnexpectedApiRedirect')


def call(opener, base, path, body=None):
    request = urllib.request.Request(base + '/reader3' + path,
        data=None if body is None else canonical(body),
        headers={} if body is None else {'Content-Type': 'application/json; charset=utf-8'},
        method='GET' if body is None else 'POST')
    try:
        response = opener.open(request, timeout=35)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read(MAX_RESPONSE + 1)
        if len(raw) > MAX_RESPONSE:
            raise Rejected('OversizedResponse')
        value = json.loads(raw.decode('utf-8'))
        if response.status != 200 or not isinstance(value, dict) or value.get('isSuccess') is not True or value.get('errorMsg') != '':
            raise Rejected('ReaderReturnDataRejected')
        return int(response.status), value


def run_side(args, side, jar, root, alias, progress):
    guard = helper('compare-webview-cookie')
    guard.require_verified_private_loopback()
    work = root / side
    work.mkdir()
    port = guard.free_port()
    base = 'http://127.0.0.1:' + str(port)
    command = [str(args.java), '-Xms128m', '-Xmx768m', '-jar', str(jar),
        '--reader.app.workDir=' + str(work), '--reader.server.port=' + str(port),
        '--reader.server.bindAddress=127.0.0.1', '--reader.app.secure=true',
        '--reader.app.licenseCheckEnabled=false', '--spring.profiles.active=prod',
        '--reader.app.webviewRenderer=' + ('camoufox' if side == 'camoufox' else 'remote'),
        '--reader.app.remote-webview-api=http://127.0.0.1:8050']
    env = os.environ.copy()
    env.pop('READER_BROWSER_ALLOW_PRIVATE_NETWORKS', None)
    if side == 'camoufox':
        command.append('--reader.app.camoufoxPythonExecutable=/usr/bin/python3')
        # Explicit fixture-only exception after verified no-WAN namespace.
        env['READER_BROWSER_ALLOW_PRIVATE_NETWORKS'] = 'true'
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    result = {'businessApiCalls': {}, 'httpStatuses': {}, 'readerStopped': False, 'logoutSucceeded': False}
    progress['activeSide'] = side
    with (work / 'private-reader.log').open('xb') as log:
        process = subprocess.Popen(command, cwd=work, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        try:
            progress['stage'] = 'startup'
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise Rejected('ReaderExitedAtStartup')
                try:
                    call(opener, base, '/getSystemInfo')
                    break
                except (OSError, ValueError):
                    time.sleep(0.25)
            else:
                raise TimeoutError('ReaderStartupTimeout')
            username, password = 'snapshot' + secrets.token_hex(5), 'Generated-' + secrets.token_hex(18)
            for login in (False, True):
                progress['stage'] = 'login' if login else 'register'
                call(opener, base, '/login', {'username': username, 'password': password, 'isLogin': login})
            definition = source_definition(alias)
            progress['stage'] = 'sourceSave'
            call(opener, base, '/saveBookSource', definition)
            saved = call(opener, base, '/getBookSource', {'bookSourceUrl': alias})[1]['data']
            result['sourceRulesRetained'] = isinstance(saved, dict) and all(canonical(saved.get(k)) == canonical(v) for k, v in definition.items())
            if not result['sourceRulesRetained']:
                raise Rejected('SourceRulesNotRetained')
            body = {'url': alias + PAGE_PATH + OPTIONS, 'bookSourceUrl': alias}
            for key, path, payload in (('metadata', '/getBookInfo', body), ('toc', '/getChapterList', body),
                    ('chapter0', '/getBookContent', dict(body, index=0, cache=0, refresh=0)),
                    ('chapter1', '/getBookContent', dict(body, index=1, cache=0, refresh=0))):
                progress['stage'] = key
                result['businessApiCalls'][key] = 1
                started = time.time_ns() // 1000000
                status, value = call(opener, base, path, payload)
                completed = time.time_ns() // 1000000
                result[key], result['httpStatuses'][key] = value, status
                if key == 'metadata':
                    result['metadataWindowMs'] = {'started': started, 'completed': completed}
                    validate_metadata(value, result['metadataWindowMs'])
                elif key == 'toc':
                    validate_toc(value['data'], alias + PAGE_PATH)
                else:
                    validate_content(value['data'])
            progress['stage'] = 'logout'
            call(opener, base, '/logout', {})
            result['logoutSucceeded'] = True
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            result['readerStopped'] = process.poll() is not None
            if (work / 'private-reader.log').stat().st_size > 4 * 1024 * 1024:
                raise Rejected('OversizedPrivateReaderLog')
    return result


def write_once(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('java', 'original', 'restored', 'snapshot', 'report', 'phase-handoff-dir'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--expected-restored-sha256', default=RESTORED_SHA)
    args = parser.parse_args()
    expected_restored = validated_restored_sha256(args.expected_restored_sha256)
    helper('compare-webview-cookie').require_verified_private_loopback()
    if os.environ.get('READER_ORIGINAL_JAR_NETWORK_ISOLATED') != 'confirmed' or os.getuid() != 10001:
        raise Rejected('HostVerifiedIsolationRequired')
    if args.report.exists() or args.phase_handoff_dir.exists():
        raise Rejected('PreserveExistingOutput')
    if file_sha(args.original) != ORIGINAL_SHA or file_sha(args.restored) != expected_restored or file_sha(args.snapshot) != CAPTURE_SHA:
        raise Rejected('ExactInputShaMismatch')
    raw_html = args.snapshot.read_bytes()
    if len(raw_html) != 468370:
        raise Rejected('CapturedPageSizeMismatch')
    requests = {'fixedPage': 0, 'otherPaths': 0, 'cookiePresent': 0}
    lock = threading.Lock()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            fixed = self.path == PAGE_PATH
            with lock:
                requests['fixedPage' if fixed else 'otherPaths'] += 1
                requests['cookiePresent'] += bool(self.headers.get('Cookie'))
            body = raw_html if fixed else b''
            self.send_response(200 if fixed else 404)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    server.daemon_threads = True
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    alias = 'http://127.0.0.1:' + str(server.server_port)
    sides, progress = {}, {}
    report = {'schemaVersion': 1, 'passed': False, 'originalExecuted': False,
        'scope': 'captured public HTML; same loopback HTTP alias; no WAN three-way replay',
        'originalJarSha256': ORIGINAL_SHA, 'restoredJarSha256': expected_restored,
        'captureSha256': CAPTURE_SHA, 'captureBytes': len(raw_html), 'capturedPublicUrl': PUBLIC_URL,
        'realSiteHttpsThreeWayProven': False, 'realAuthenticationProven': False,
        'notLatestWholeNativeImageAcceptance': True, 'privateBooksRead': 0,
        'rawBodyOrCredentialsExported': False, 'fixtureOnlyPrivateNetworkException': True,
        'interfacesLoopbackOnlyVerified': True, 'fullGoalComplete': False, 'sides': {}}
    try:
        with tempfile.TemporaryDirectory(prefix='reader-public-snapshot-') as temp:
            root = Path(temp)
            for name, jar in (('original', args.original), ('restored', args.restored)):
                sides[name] = run_side(args, name, jar, root, alias, progress)
                report['sides'][name] = facts(sides[name])
            report['originalExecuted'] = True
            # Preserve bounded pair facts before host-owned reference shutdown.
            write_once(args.report.with_name(args.report.stem + '.historical-observation.json'), report)
            args.phase_handoff_dir.mkdir()
            (args.phase_handoff_dir / 'remote-complete').touch(exist_ok=False)
            deadline = time.monotonic() + 60
            marker = args.phase_handoff_dir / 'camoufox-permitted'
            while not marker.exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            if not marker.is_file() or marker.is_symlink() or marker.stat().st_size != 0:
                raise Rejected('IndependentlyStoppedHistoricalRequired')
            sides['camoufox'] = run_side(args, 'camoufox', args.restored, root, alias, progress)
            report['sides']['camoufox'] = facts(sides['camoufox'])
            report['comparison'] = compare(sides)
            report['passed'] = True
    except Exception as error:
        # Finite stage/class only, never exception text/URL/body/session values.
        report['failureType'] = type(error).__name__
        report['failureContext'] = dict(progress)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
        report['fixtureRequests'] = dict(requests)
        write_once(args.report, report)
    print(json.dumps({'passed': report['passed'], 'completedSides': sorted(report['sides'])}))
    return 0 if report['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
