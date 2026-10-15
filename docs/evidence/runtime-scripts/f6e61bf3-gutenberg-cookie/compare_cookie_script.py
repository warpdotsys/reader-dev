"""Only fixed public Gutenberg HTTPS: actual DOM script, generated cookie, two synthetic users."""
import argparse
import copy
import hashlib
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import subprocess
import tempfile
import time
import urllib.request
import urllib.error

import allowlist_transport as transport
from compare_live import upstream_snapshot

COOKIE_NAME = 'reader_public_probe'
GENERATED_COOKIE = COOKIE_NAME + '=generated-scope-only'
SCRIPT = ("(function(){var h=document.querySelector('h1');if(!h)throw new Error('FixedHeadingMissing');"
    "h.setAttribute('data-reader-probe-script','executed');"
    "h.setAttribute('data-reader-probe-cookie',document.cookie.split(/;\\s*/).indexOf("
    + json.dumps(GENERATED_COOKIE) + ")>=0?'probe-present':'probe-absent');"
    "return document.documentElement.outerHTML;})()")
PHASES = ('baseline', 'seeded', 'other-user', 'cleared')


def original_cookie_endpoint_observation(opener, base, source):
    """Observe the extension's actual absence; never parse HTML as ReturnData."""
    snapshot = helper('compare-gutenberg-snapshot')
    request = urllib.request.Request(base + '/reader3/setBookSourceCookie',
        data=snapshot.canonical(dict(bookSource=source['bookSourceUrl'], cookie='')),
        headers={'Content-Type': 'application/json; charset=utf-8'}, method='POST')
    try:
        response = opener.open(request, timeout=10)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read(16385)
        if len(raw) > 16384 or response.status != 404:
            raise ValueError('Exact original endpoint absence must be observed, not assumed')
        return dict(httpStatus=int(response.status), contentType=response.headers.get('Content-Type'),
            bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(), available=False,
            inputTransportNotParityClaimed=True)


def original_source_cookie_rule(source, phase):
    """Use B26's real source-JVM cookie/cache API, never modify the original JAR.

    Original BaseSource$DefaultImpls bytecode binds `cookie` to its namespace's
    CookieStore. Its public getCacheInstance exposes the legacy cookieJar key.
    The current HTTP cookie setter is an extension absent from B26.
    """
    if phase not in ('seeded', 'cleared'):
        raise ValueError('Only explicit generated cookie input mutations permitted')
    result = copy.deepcopy(source)
    value = json.dumps(GENERATED_COOKIE)
    key = json.dumps('gutenberg.org_cookieJar')
    if phase == 'seeded':
        operation = ('cookie.setCookie(source.getKey(),' + value + ');'
            'var c=cookie.getCacheInstance();c.put(' + key + ',' + value + ');'
            'if(String(cookie.getCookie(source.getKey()))!==' + value + '||String(c.getAsString(' + key + '))!==' + value + ')'
            "throw new Error('GeneratedLegacyCookieNotStored');")
    else:
        operation = ('cookie.removeCookie(source.getKey());var c=cookie.getCacheInstance();c.remove(' + key + ');'
            'var v=c.getAsString(' + key + ');if(String(cookie.getCookie(source.getKey()))!==""||'
            '(v!==null&&String(v)!==""))throw new Error("GeneratedLegacyCookieNotCleared");')
    result['header'] = '@js:(function(){' + operation + 'return "{}";})()'
    return result


def observed_cookie_request(request, phase, original):
    """Observe B26's actual JVM binding/path, not an inferred namespace.

    Book.putVariable and the public CookieStore getters avoid exposing File
    objects through the source scripting engine. Files are checked independently
    by the bounded outside observer. D observed original's actual `default`
    namespace. Preserve that historical discrepancy; require actual generated
    authenticated namespaces on both restored sides. Network URL/webJs unchanged.
    """
    if phase not in PHASES:
        raise ValueError('Unexpected generated phase')
    body = ''
    if original and phase in ('seeded', 'cleared'):
        source_rule = original_source_cookie_rule({}, phase)['header']
        body = source_rule.removeprefix('@js:(function(){').removesuffix('return "{}";})()')
    diagnostic = (
        'var pc=cookie.getCacheInstance();'
        'book.putVariable("_readerGeneratedCookieProbe",JSON.stringify({'
        'phase:' + json.dumps(phase) + ',namespace:String(cookie.getUserNameSpace()),'
        'manualPresent:String(cookie.getCookie(source.getKey()))===' + json.dumps(GENERATED_COOKIE) + ','
        'jarPresent:String(pc.getAsString("gutenberg.org_cookieJar"))===' + json.dumps(GENERATED_COOKIE) + '}));')
    option = dict(webView=True, webJs=SCRIPT, js=body + diagnostic + 'result;')
    current = copy.deepcopy(request)
    current['url'] = helper('probe-gutenberg-reader').BOOK + ', ' + json.dumps(option, separators=(',', ':'))
    return current, hashlib.sha256(option['js'].encode()).hexdigest()


def generated_cache_observation(directory, username, phase, value, observation, original):
    """Strictly prove current isolation and retain the observed original defect.

    Only original may use `default`, actually observed in D. Original's second
    account will observe its shared generated cache: document, never mark that
    as successful historical cookie-store isolation. Current must use username.
    """
    if not re.fullmatch(r'gutenberg[0-9a-f]{10}', username):
        raise ValueError('Only this generated namespace may be inspected')
    cookie_root = directory / 'storage/cache/cookie'
    if cookie_root.exists() and (cookie_root.is_symlink() or not cookie_root.is_dir()):
        raise ValueError('Unexpected generated cookie root')
    # Metadata only, bounded to generated scratch data; never read login tables.
    metadata = []
    if cookie_root.is_dir():
        for path in sorted(cookie_root.rglob('*')):
            if len(metadata) >= 32 or path.is_symlink():
                raise ValueError('Unexpected generated cookie metadata scope')
            metadata.append(dict(relativePath=str(path.relative_to(cookie_root)),
                directory=path.is_dir(), bytes=path.stat().st_size if path.is_file() else 0))
    observation['generatedCookieCacheMetadata'] = metadata
    variables = json.loads(value['data'].get('variable') or '{}')
    marker = variables.get('_readerGeneratedCookieProbe')
    if not isinstance(marker, str):
        raise ValueError('Original URL-option probe did not return execution marker')
    diagnostic = json.loads(marker)
    if (set(diagnostic) != {'phase', 'namespace', 'manualPresent', 'jarPresent'}
            or diagnostic['phase'] != phase
            or type(diagnostic['manualPresent']) is not bool or type(diagnostic['jarPresent']) is not bool
            or not isinstance(diagnostic['namespace'], str)
            or not re.fullmatch(r'gutenberg[0-9a-f]{10}|default|unknow|unknown|', diagnostic['namespace'])):
        raise ValueError('Unexpected bounded original execution marker')
    observation['actualJvmCookieBinding'] = diagnostic
    observation['actualCookieNamespaceMatchesGeneratedUser'] = diagnostic['namespace'] == username
    target = cookie_root / diagnostic['namespace']
    if not target.resolve().is_relative_to(directory.resolve()) or (target.exists() and (target.is_symlink() or not target.is_dir())):
        raise ValueError('Unexpected generated cookie directory')
    rows = {}
    for key in ('gutenberg.org', 'gutenberg.org_cookieJar'):
        code = 0
        for char in key:
            code = (31 * code + ord(char)) & 0xffffffff
        if code >= 0x80000000:
            code -= 0x100000000
        file = target / str(code)
        if file.exists():
            if file.is_symlink() or not file.is_file() or file.stat().st_size > 16384:
                raise ValueError('Unexpected bounded generated cache object')
            raw = file.read_bytes()
        else:
            raw = b''
        present = raw == GENERATED_COOKIE.encode('ascii')
        rows[key] = dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(), generatedLiteralPresent=present,
            relativePath=str(file.relative_to(directory)))
    observation['actualGeneratedCookieStoreInputs'] = rows
    expected_namespace = 'default' if original else username
    if diagnostic['namespace'] != expected_namespace:
        raise ValueError('Actual JVM cookie namespace differs from explicit observed contract')
    expected_present = phase == 'seeded' or (original and phase == 'other-user')
    for key, row in rows.items():
        present = row['generatedLiteralPresent']
        if expected_present and not present:
            raise ValueError('Actual generated cookie input was not stored')
        if not expected_present and row['bytes']:
            raise ValueError('Generated cookie remained in fresh/other/cleared input')
    if diagnostic['manualPresent'] != expected_present or diagnostic['jarPresent'] != expected_present:
        raise ValueError('Actual JVM getter and filesystem input observations differ')
    return rows


def business_observation(opener, base, request, observation):
    """Keep the complete bounded real ReturnData BEFORE a validator rejects it."""
    snapshot = helper('compare-gutenberg-snapshot')
    req = urllib.request.Request(base + '/reader3/getBookInfo', data=snapshot.canonical(request),
        headers={'Content-Type':'application/json; charset=utf-8'}, method='POST')
    try:
        response = opener.open(req, timeout=35)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read(snapshot.MAX_RESPONSE + 1)
        observation.update(httpStatus=int(response.status), responseBytes=len(raw),
            responseSha256=hashlib.sha256(raw).hexdigest())
        if len(raw) > snapshot.MAX_RESPONSE:
            raise ValueError('Oversized actual API response')
        value = json.loads(raw.decode('utf-8'))
        observation['returnData'] = value
        if not isinstance(value, dict) or response.status != 200 or value.get('isSuccess') is not True or value.get('errorMsg') != '':
            raise ValueError('Actual API ReturnData rejected')
        return int(response.status), value


def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path('/probe') / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source_and_request():
    public = helper('probe-gutenberg-reader')
    source = copy.deepcopy(public.SOURCE_DEFINITION)
    source['enabledCookieJar'] = True
    source['ruleBookInfo']['intro'] = 'h1@data-reader-probe-script'
    source['ruleBookInfo']['kind'] = 'h1@data-reader-probe-cookie'
    options = json.dumps(dict(webView=True, webJs=SCRIPT), separators=(',', ':'))
    return source, dict(url=public.BOOK + ', ' + options, bookSourceUrl=public.SOURCE)


def validate_phase(phase, value, window, current):
    snapshot = helper('compare-gutenberg-snapshot')
    snapshot.validate_metadata(value, window)
    data = value['data']
    if data.get('intro') != 'executed' or data.get('kind') not in ('probe-present', 'probe-absent'):
        raise ValueError('Actual fixed-page browser DOM script did not produce both markers')
    present = data['kind'] == 'probe-present'
    if phase != 'seeded' and present:
        raise ValueError('Generated cookie leaked into fresh/other/cleared namespace')
    if phase == 'seeded' and current and not present:
        raise ValueError('Current browser did not replay generated manual cookie')
    return dict(scriptExecuted=True, generatedCookieVisible=present, defaultClocksFreshInActualWindow=True,
        fullReturnDataSha256=snapshot.digest(value), metadataFields=sorted(data),
        nameAndAuthorValid=True, httpStatus=200, isSuccess=True, errorMsgEmpty=True)


def compare(sides):
    if set(sides) != {'original', 'restored', 'camoufox'}:
        raise ValueError('All actual sides required')
    for name, side in sides.items():
        if set(side['phases']) != set(PHASES) or not side['readerStopped'] or not side['bothGeneratedSessionsLoggedOut']:
            raise ValueError('Complete four-phase side and lifecycle required')
        for phase in PHASES:
            item = side['phases'][phase]
            validate_phase(phase, item['returnData'], item['windowMs'], name == 'camoufox')
            if type(item['httpStatus']) is not int or type(item['actualCalls']) is not int or item['httpStatus'] != 200 or item['actualCalls'] != 1:
                raise ValueError('Actual single API observation per phase required')
    values = {name: side['phases']['seeded']['returnData']['data']['kind'] == 'probe-present' for name, side in sides.items()}
    originals = sides['original']['phases']
    if any(row['actualJvmCookieBinding']['namespace'] != 'default' for row in originals.values()):
        raise ValueError('Actual original namespace changed from directly observed default')
    for name in ('restored', 'camoufox'):
        phases = sides[name]['phases']
        if any(row['actualCookieNamespaceMatchesGeneratedUser'] is not True for row in phases.values()):
            raise ValueError('Current authenticated cookie namespace isolation not proven')
        if phases['baseline']['actualJvmCookieBinding']['namespace'] == phases['other-user']['actualJvmCookieBinding']['namespace']:
            raise ValueError('Current generated accounts share cookie namespace')
    return dict(allThreeExecuteActualBrowserScriptOnFixedRealHttpsPage=True,
        currentGeneratedManualCookieReplay=True, allThreeBrowserDomFreshOtherUserAndClearedCookieAbsent=True,
        currentBothRenderersCookieStoreUserIsolationProven=True,
        originalBookRequestCookieNamespace='default', originalCookieStoreUserIsolationProven=False,
        originalSharedCookieInputObservedInSecondUser=True, originalNamespaceDefectNotReintroduced=True,
        seededCookieVisibility=values, threeWaySeededCookieVisibilityEqual=len(set(values.values())) == 1,
        allRawMetadataReturnDataAndDefaultClocksPreserved=True, fullLiteralJsonParityClaimed=False,
        trueWebsiteAuthenticationClaimed=False)


def run_side(args, name, jar, directory, report, progress):
    snapshot, guard = helper('compare-gutenberg-snapshot'), helper('compare-webview-cookie')
    guard.require_verified_private_loopback()
    directory.mkdir()
    port = guard.free_port()
    base = 'http://127.0.0.1:' + str(port)
    source, request = source_and_request()
    command = [str(args.java), '-Xms128m', '-Xmx768m', '-jar', str(jar),
        '--reader.app.workDir=' + str(directory), '--reader.server.port=' + str(port),
        '--reader.server.bindAddress=127.0.0.1', '--reader.app.secure=true',
        '--reader.app.licenseCheckEnabled=false', '--spring.profiles.active=prod',
        '--reader.app.webviewRenderer=' + ('camoufox' if name == 'camoufox' else 'remote'),
        '--reader.app.remote-webview-api=http://127.0.0.1:8050']
    if name == 'camoufox':
        command.append('--reader.app.camoufoxPythonExecutable=/usr/bin/python3')
    openers = [urllib.request.build_opener(urllib.request.ProxyHandler({}), snapshot.NoRedirect(),
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar())) for _ in range(2)]
    result = dict(phases={}, readerStopped=False, bothGeneratedSessionsLoggedOut=False, sourceRulesRetained=False)
    generated_usernames = []
    report['sides'][name] = result
    progress['activeSide'] = name
    with (directory / 'generated-reader.log').open('xb') as logfile:
        process = subprocess.Popen(command, cwd=directory, env=dict(os.environ, READER_BROWSER_ALLOW_PRIVATE_NETWORKS='false'),
            stdin=subprocess.DEVNULL, stdout=logfile, stderr=logfile)
        try:
            progress['stage'] = 'startup'
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise ValueError('Owned Reader exited at startup')
                try:
                    snapshot.call(openers[0], base, '/getSystemInfo')
                    break
                except (OSError, ValueError):
                    time.sleep(0.25)
            else:
                raise TimeoutError('Owned Reader startup timeout')
            for index, opener in enumerate(openers):
                progress['stage'] = 'generated-account-' + str(index)
                username, password = 'gutenberg' + secrets.token_hex(5), 'Generated-' + secrets.token_hex(18)
                generated_usernames.append(username)
                for login in (False, True):
                    snapshot.call(opener, base, '/login', dict(username=username, password=password, isLogin=login))
                snapshot.call(opener, base, '/saveBookSource', source)
                saved = snapshot.call(opener, base, '/getBookSource', dict(bookSourceUrl=source['bookSourceUrl']))[1]['data']
                if not isinstance(saved, dict) or not all(snapshot.canonical(saved.get(k)) == snapshot.canonical(v) for k, v in source.items()):
                    raise ValueError('Actual source readback differs')
            result['sourceRulesRetained'] = True
            if name == 'original':
                result['cookieSetterExtension'] = original_cookie_endpoint_observation(openers[0], base, source)
                result['cookieInputMechanism'] = 'original Book/request namespace CookieStore via supported AnalyzeUrl JVM option js'
                result['legacyRequestCookieRulesSha256'] = {}
            else:
                result['cookieInputMechanism'] = 'current setBookSourceCookie HTTP extension'
            for phase in PHASES:
                progress['stage'] = phase
                phase_request = request
                phase_request, script_sha = observed_cookie_request(request, phase, name == 'original')
                result.setdefault('requestCookieProbeRulesSha256', {})[phase] = script_sha
                if phase in ('seeded', 'cleared'):
                    if name != 'original':
                        status, mutation = snapshot.call(openers[0], base, '/setBookSourceCookie', dict(bookSource=source['bookSourceUrl'],
                            cookie=GENERATED_COOKIE if phase == 'seeded' else ''))
                        result.setdefault('cookieSetterExtensionMutations', {})[phase] = dict(httpStatus=status, returnData=mutation)
                opener = openers[1] if phase == 'other-user' else openers[0]
                observation = dict(actualCalls=1, transportFacts={})
                result['phases'][phase] = observation
                start = time.time_ns() // 1000000
                status, value = business_observation(opener, base, phase_request, observation)
                observation.update(httpStatus=status, returnData=value,
                    windowMs=dict(started=start, completed=time.time_ns() // 1000000))
                observation['validatedFacts'] = validate_phase(phase, value, observation['windowMs'], name == 'camoufox')
                user = generated_usernames[1] if phase == 'other-user' else generated_usernames[0]
                generated_cache_observation(directory, user, phase, value, observation, name == 'original')
            for opener in openers:
                snapshot.call(opener, base, '/logout', {})
            result['bothGeneratedSessionsLoggedOut'] = True
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            result['readerStopped'] = process.poll() is not None
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('java', 'original', 'restored', 'report', 'phase-handoff-dir'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--expected-restored-sha256', required=True)
    parser.add_argument('--public-bridge-ip', required=True)
    args = parser.parse_args()
    snapshot = helper('compare-gutenberg-snapshot')
    helper('compare-webview-cookie').require_verified_private_loopback()
    if os.getuid() != 10001 or os.environ.get('READER_ORIGINAL_JAR_NETWORK_ISOLATED') != 'confirmed':
        raise ValueError('Verified isolated non-root runtime required')
    if args.report.exists() or args.phase_handoff_dir.exists():
        raise ValueError('Keep existing observations')
    if snapshot.file_sha(args.original) != snapshot.ORIGINAL_SHA or snapshot.file_sha(args.restored) != args.expected_restored_sha256:
        raise ValueError('Require exact original and candidate JAR hashes')
    report = dict(schemaVersion=1, passed=False, scope='real fixed Gutenberg HTTPS DOM script and generated cookie/two-user scope only',
        originalJarSha256=snapshot.ORIGINAL_SHA, restoredJarSha256=args.expected_restored_sha256,
        fixedPublicUrl=snapshot.PUBLIC_URL, browserScriptSha256=hashlib.sha256(SCRIPT.encode()).hexdigest(),
        realSiteScriptCookieThreeWayProven=False, realWebsiteAuthenticationProven=False,
        realCredentialsImported=False, localBooksRead=0, rawChapterBodiesExported=False,
        syntheticCookieLiteralIncludedInOwnedTestCode=True, currentPrivateNetworkGuardDisabled=False,
        originalCookieInputUsesVerifiedLegacyRequestJvmApi=True,
        originalDefaultNamespaceDirectlyObservedInRejectedD=True,
        originalNamespaceIsolationNotClaimed=True,
        currentHttpCookieSetterNotClaimedOriginalApi=True,
        currentTlsErrorsIgnored=False, historicalTlsErrorsIgnored=True, wholeGoalComplete=False, sides={})
    relay = transport.Relay('/egress/egress.sock', args.public_bridge_ip)
    proxy, progress = 'http://127.0.0.1:' + str(relay.port), {}
    start = time.monotonic()
    try:
        report['whiteListNegativeChecks'] = relay.negative_checks()
        report['upstreamBefore'] = upstream_snapshot(proxy)
        with tempfile.TemporaryDirectory(prefix='reader-public-cookie-script-') as temporary:
            directory = Path(temporary)
            for name, jar in (('original', args.original), ('restored', args.restored)):
                run_side(args, name, jar, directory / name, report, progress)
            snapshot.write_once(args.report.with_name('historical-observation.json'), report)
            args.phase_handoff_dir.mkdir()
            (args.phase_handoff_dir / 'remote-complete').touch(exist_ok=False)
            marker = args.phase_handoff_dir / 'camoufox-permitted'
            deadline = time.monotonic() + 60
            while not marker.exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            if not marker.is_file() or marker.is_symlink() or marker.stat().st_size != 0:
                raise ValueError('Reference must be independently stopped before current browser')
            run_side(args, 'camoufox', args.restored, directory / 'camoufox', report, progress)
            report['upstreamAfter'] = upstream_snapshot(proxy)
            report['upstreamBytesStable'] = report['upstreamBefore'] == report['upstreamAfter']
            report['comparison'] = compare(report['sides'])
            if not report['upstreamBytesStable']:
                raise ValueError('Fixed public page changed; parity not accepted')
            report['realSiteScriptCookieThreeWayProven'] = report['passed'] = True
    except Exception as error:
        report['failureType'] = type(error).__name__
        if isinstance(error, ValueError) and str(error) in (
                'Actual generated cookie input was not stored',
                'Original URL-option probe did not return execution marker',
                'Actual JVM cookie namespace differs from explicit observed contract',
                'Actual JVM getter and filesystem input observations differ',
                'Actual API ReturnData rejected',
                'Generated cookie remained in fresh/other/cleared input',
                'Actual fixed-page browser DOM script did not produce both markers',
                'Current browser did not replay generated manual cookie',
                'Generated cookie leaked into fresh/other/cleared namespace'):
            report['failureReason'] = str(error)
        report['failureContext'] = dict(progress)
        if isinstance(error, urllib.error.HTTPError):
            report['failurePublicHttpStatus'] = int(error.code)
            retry = error.headers.get('Retry-After', '')
            report['failureRetryAfterSeconds'] = int(retry) if retry.isdigit() and int(retry) <= 3600 else None
            report['failureBeforeAnyReaderStarted'] = not report['sides']
    finally:
        relay.close()
        report['seconds'] = round(time.monotonic() - start, 3)
        snapshot.write_once(args.report, report)
    print(json.dumps(dict(passed=report['passed'], completedSides=sorted(report['sides']), failureContext=report.get('failureContext'))), flush=True)
    return 0 if report['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())


