"""Actual fixed Gutenberg HTTPS Reader three-way; full JSON compared only in memory."""
import argparse
import copy
import hashlib
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

import allowlist_transport as transport


def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path('/probe') / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def business_call(opener, base, path, payload, observed):
    snapshot = helper('compare-gutenberg-snapshot')
    request = urllib.request.Request(base + '/reader3' + path, data=snapshot.canonical(payload),
        headers={'Content-Type': 'application/json; charset=utf-8'}, method='POST')
    try:
        response = opener.open(request, timeout=35)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read(snapshot.MAX_RESPONSE + 1)
        observed['status'] = int(response.status)
        observed['bytes'] = len(raw)
        observed['sha256'] = hashlib.sha256(raw).hexdigest()
        if len(raw) > snapshot.MAX_RESPONSE:
            raise ValueError('Oversized actual API response')
        value = json.loads(raw.decode('utf-8'))
        if not isinstance(value, dict):
            raise ValueError('Actual API response is not object')
        observed['fields'] = sorted(value)
        observed['isSuccess'] = value.get('isSuccess') if type(value.get('isSuccess')) is bool else None
        observed['errorMsgEmpty'] = value.get('errorMsg') == ''
        observed['errorMsgSha256'] = hashlib.sha256(str(value.get('errorMsg')).encode('utf-8')).hexdigest()
        observed['dataType'] = type(value.get('data')).__name__
        if response.status != 200 or value.get('isSuccess') is not True or value.get('errorMsg') != '':
            raise ValueError('Actual API ReturnData rejected')
        return int(response.status), value


def run_side(args, name, jar, directory, proxy, progress, partial):
    guard, snapshot, public = helper('compare-webview-cookie'), helper('compare-gutenberg-snapshot'), helper('probe-gutenberg-reader')
    guard.require_verified_private_loopback()
    directory.mkdir()
    base = 'http://127.0.0.1:' + str(guard.free_port())
    command = [str(args.java), '-Xms128m', '-Xmx768m', '-jar', str(jar),
        '--reader.app.workDir=' + str(directory), '--reader.server.port=' + base.rsplit(':', 1)[1],
        '--reader.server.bindAddress=127.0.0.1', '--reader.app.secure=true',
        '--reader.app.licenseCheckEnabled=false', '--spring.profiles.active=prod',
        '--reader.app.webviewRenderer=' + ('camoufox' if name == 'camoufox' else 'remote'),
        '--reader.app.remote-webview-api=http://127.0.0.1:8050']
    environment = dict(os.environ, READER_BROWSER_ALLOW_PRIVATE_NETWORKS='false')
    if name == 'camoufox':
        command.append('--reader.app.camoufoxPythonExecutable=/usr/bin/python3')
    source = copy.deepcopy(public.SOURCE_DEFINITION)
    # All three retain the exact existing public source. Network routing is
    # provided by the sole-SNI transparent bridge, not by relaxing production
    # rejection of a loopback/private upstream proxy.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), snapshot.NoRedirect(),
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    result = dict(businessApiCalls={}, httpStatuses={}, readerStopped=False, logoutSucceeded=False)
    partial[name] = dict(actualApiObservations={}, readerStopped=False)
    progress['activeSide'] = name
    with (directory / 'generated-reader.log').open('xb') as logfile:
        process = subprocess.Popen(command, cwd=directory, env=environment, stdin=subprocess.DEVNULL, stdout=logfile, stderr=logfile)
        try:
            progress['stage'] = 'startup'
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError('Owned Reader exited at startup')
                try:
                    snapshot.call(opener, base, '/getSystemInfo')
                    break
                except (OSError, ValueError):
                    time.sleep(0.25)
            else:
                raise TimeoutError('Owned Reader startup timeout')
            username, password = 'gutenberg' + secrets.token_hex(5), 'Generated-' + secrets.token_hex(18)
            for login in (False, True):
                progress['stage'] = 'register' if not login else 'login'
                snapshot.call(opener, base, '/login', dict(username=username, password=password, isLogin=login))
            progress['stage'] = 'source-save-readback'
            snapshot.call(opener, base, '/saveBookSource', source)
            saved = snapshot.call(opener, base, '/getBookSource', dict(bookSourceUrl=public.SOURCE))[1]['data']
            result['sourceRulesRetained'] = isinstance(saved, dict) and all(snapshot.canonical(saved.get(key)) == snapshot.canonical(value) for key, value in source.items())
            if not result['sourceRulesRetained']:
                raise RuntimeError('Actual source readback differs')
            body = dict(public.BOOK_BODY)
            for key, route, request in (('metadata', '/getBookInfo', body), ('toc', '/getChapterList', body),
                    ('chapter0', '/getBookContent', dict(body, index=0, cache=0, refresh=0)),
                    ('chapter1', '/getBookContent', dict(body, index=1, cache=0, refresh=0))):
                progress['stage'] = key
                result['businessApiCalls'][key] = 1
                start = time.time_ns() // 1000000
                observation = {}
                partial[name]['actualApiObservations'][key] = observation
                status, value = business_call(opener, base, route, request, observation)
                result[key], result['httpStatuses'][key] = value, status
                if key == 'metadata':
                    result['metadataWindowMs'] = dict(started=start, completed=time.time_ns() // 1000000)
                    snapshot.validate_metadata(value, result['metadataWindowMs'])
                elif key == 'toc':
                    snapshot.validate_toc(value['data'], public.BOOK)
                else:
                    snapshot.validate_content(value['data'])
            progress['stage'] = 'logout'
            snapshot.call(opener, base, '/logout', {})
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
            partial[name]['readerStopped'] = result['readerStopped']
    return result


def upstream_snapshot(proxy):
    snapshot = helper('compare-gutenberg-snapshot')
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({'https': proxy}), snapshot.NoRedirect())
    with opener.open(snapshot.PUBLIC_URL, timeout=20) as response:
        raw = response.read(transport.LIMIT + 1)
        if response.status != 200 or len(raw) > transport.LIMIT:
            raise ValueError('Public page preflight not successful and bounded')
        import hashlib
        return dict(status=response.status, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(), tlsCertificateValidationEnabled=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('java', 'original', 'restored', 'report', 'phase-handoff-dir'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--expected-restored-sha256', required=True)
    parser.add_argument('--public-bridge-ip', required=True)
    args = parser.parse_args()
    snapshot = helper('compare-gutenberg-snapshot')
    helper('compare-webview-cookie').require_verified_private_loopback()
    if os.getuid() != 10001 or os.environ.get('READER_ORIGINAL_JAR_NETWORK_ISOLATED') != 'confirmed':
        raise RuntimeError('Verified isolated non-root runtime required')
    if args.report.exists() or args.phase_handoff_dir.exists():
        raise RuntimeError('Preserve old output')
    if snapshot.file_sha(args.original) != snapshot.ORIGINAL_SHA or snapshot.file_sha(args.restored) != args.expected_restored_sha256:
        raise RuntimeError('Original or candidate SHA mismatch')
    report = dict(schemaVersion=1, passed=False, scope='fixed public Gutenberg 84 actual HTTPS through sole-domain proxy',
        originalJarSha256=snapshot.ORIGINAL_SHA, restoredJarSha256=args.expected_restored_sha256,
        publicUrl=snapshot.PUBLIC_URL, originalExecuted=False, realSiteHttpsThreeWayProven=False,
        currentRendererTlsErrorsIgnored=False, historicalRendererIgnoresTlsErrors=True,
        allThreeRenderersCertificateValidationProven=False, privateBooksRead=0, realCredentialsImported=False,
        realAuthenticationProven=False, rawBodyOrCredentialsExported=False, fullGoalComplete=False,
        fixtureOnlyPrivateNetworkException=False, sides={}, partialSideFacts={})
    sides, progress = {}, {}
    relay = transport.Relay('/egress/egress.sock', args.public_bridge_ip)
    proxy = 'http://127.0.0.1:' + str(relay.port)
    started = time.monotonic()
    try:
        report['whiteListNegativeChecks'] = relay.negative_checks()
        report['upstreamBefore'] = upstream_snapshot(proxy)
        with tempfile.TemporaryDirectory(prefix='reader-live-gutenberg-') as temporary:
            directory = Path(temporary)
            for name, jar in (('original', args.original), ('restored', args.restored)):
                sides[name] = run_side(args, name, jar, directory / name, proxy, progress, report['partialSideFacts'])
                report['sides'][name] = snapshot.facts(sides[name])
            report['originalExecuted'] = True
            snapshot.write_once(args.report.with_name('historical-observation.json'), report)
            args.phase_handoff_dir.mkdir()
            (args.phase_handoff_dir / 'remote-complete').touch(exist_ok=False)
            marker = args.phase_handoff_dir / 'camoufox-permitted'
            deadline = time.monotonic() + 60
            while not marker.exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            if not marker.is_file() or marker.is_symlink() or marker.stat().st_size != 0:
                raise RuntimeError('Historical renderer shutdown not independently acknowledged')
            sides['camoufox'] = run_side(args, 'camoufox', args.restored, directory / 'camoufox', proxy, progress, report['partialSideFacts'])
            report['sides']['camoufox'] = snapshot.facts(sides['camoufox'])
            report['upstreamAfter'] = upstream_snapshot(proxy)
            report['upstreamBytesStable'] = report['upstreamBefore'] == report['upstreamAfter']
            report['comparison'] = snapshot.compare(sides)
            if not report['upstreamBytesStable']:
                raise ValueError('Live public page changed; differential is inconclusive')
            report['realSiteHttpsThreeWayProven'] = report['passed'] = True
    except Exception as error:
        report['failureType'] = type(error).__name__
        report['failureContext'] = dict(progress)
    finally:
        relay.close()
        report['seconds'] = round(time.monotonic() - started, 3)
        snapshot.write_once(args.report, report)
    print(json.dumps(dict(passed=report['passed'], completedSides=sorted(report['sides']), failureContext=report.get('failureContext'))), flush=True)
    return 0 if report['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
