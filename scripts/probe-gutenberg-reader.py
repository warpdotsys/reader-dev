"""One fixed public-domain book through the bundled Reader, not site search.

No real credentials, private storage, original binary, raw body or browser
Cookie is exported. Reuse an independently tested image; never retry a failed
business call, accept an empty success, or disable TLS/private-network guards.
"""
import argparse
import hashlib
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import threading
import time
import urllib.error
import urllib.parse
import urllib.request

SOURCE = 'https://www.gutenberg.org'
BOOK = SOURCE + '/cache/epub/84/pg84-images.html'
OPTIONS = ', {"webView":true}'
BOOK_REQUEST = BOOK + OPTIONS
BASE = 'http://127.0.0.1:18893'
EXPECTED_JAR = 'f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b'
# Historical default keeps old frozen callers verifiable. Current native probes
# pass the exact digest from independently verified artifact metadata explicitly.
MAX_BYTES = 128 * 1024
AUTHOR_RULE = r"@CSS:h2:matches(^by\s)@text"
CHAPTER_URL_RULE = """@href@js:result + ', {"webView":true}'"""
TOC_FRAGMENTS = tuple('letter' + str(index) for index in range(1, 5)) + tuple(
    'chap' + str(index).zfill(2) for index in range(1, 25))
SOURCE_DEFINITION = {
    'bookSourceUrl': SOURCE, 'bookSourceName': 'Fixed Gutenberg 84 anonymous compatibility probe',
    'enabledCookieJar': False,
    'ruleBookInfo': {'name': 'h1@text', 'author': AUTHOR_RULE,
        'tocUrl': "@js:'" + BOOK_REQUEST + "'"},
    'ruleToc': {'chapterList': 'table a[href^="#"]', 'chapterName': '@text',
        'chapterUrl': CHAPTER_URL_RULE},
    'ruleContent': {'content': """@js:(function () {
        var url = String(chapter.url);
        var id = url.split('#')[1].split(',')[0];
        if (!/^(letter[1-4]|chap[0-9]{2})$/.test(id)) throw new Error('UnexpectedFixedChapter');
        var doc = Packages.org.jsoup.Jsoup.parse(String(result));
        return String(doc.select('div.chapter:has(#' + id + ') p').text());
    })()"""},
}
BOOK_BODY = {'url': BOOK_REQUEST, 'bookSourceUrl': SOURCE}
CONTENT_BODY = dict(BOOK_BODY, index=0, cache=0, refresh=0)

def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

class Rejected(ValueError):
    pass


def validated_jar_sha256(value):
    if not isinstance(value, str) or not re.fullmatch('[0-9a-f]{64}', value):
        raise Rejected('InvalidJarIdentity')
    return value

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise Rejected('UnexpectedReaderRedirect')

def allowed_request(path, body):
    if path in ('/getSystemInfo', '/getBookSourceCookie') and body is None:
        return
    if path == '/login' and isinstance(body, dict) and set(body) == {'username', 'password', 'isLogin'}:
        if (re.fullmatch(r'gutenberg[0-9a-f]{10}', body['username']) and
                re.fullmatch(r'Generated-[0-9a-f]{36}', body['password']) and body['isLogin'] is False):
            return
    expected = {'/saveBookSource': SOURCE_DEFINITION, '/getBookSource': {'bookSourceUrl': SOURCE},
        '/getBookInfo': BOOK_BODY, '/getChapterList': BOOK_BODY, '/getBookContent': CONTENT_BODY,
        '/setBookSourceCookie': {'bookSource': SOURCE, 'cookie': ''}, '/logout': {}}
    if path not in expected or json.dumps(body, sort_keys=True) != json.dumps(expected[path], sort_keys=True):
        raise Rejected('UnexpectedFixedProbeRequest')

def call(opener, path, body=None):
    allowed_request(path, body)
    raw = json.dumps(body, ensure_ascii=False).encode('utf-8') if body is not None else None
    request = urllib.request.Request(BASE + '/reader3' + path, data=raw,
        headers={'Content-Type': 'application/json; charset=utf-8'} if raw is not None else {},
        method='POST' if raw is not None else 'GET')
    try:
        response = opener.open(request, timeout=35)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        data = response.read(MAX_BYTES + 1)
        if len(data) > MAX_BYTES:
            raise Rejected('ReaderResponseTooLarge')
        value = json.loads(data.decode('utf-8'))
        if not isinstance(value, dict):
            raise Rejected('UnexpectedReaderJsonType')
        return int(response.status), value

def envelope(result):
    status, value = result
    return {'status': status, 'isSuccess': value.get('isSuccess') if type(value.get('isSuccess')) is bool else None,
        'errorMsgEmpty': value.get('errorMsg') == '', 'fields': sorted(value)}

def successful(result):
    status, value = result
    if status != 200 or value.get('isSuccess') is not True or value.get('errorMsg') != '':
        raise Rejected('ReaderRejected')
    return value.get('data')

def content_facts(value):
    if not isinstance(value, str) or not 1000 <= len(value) <= MAX_BYTES:
        raise Rejected('EmptyOrUnexpectedChapter')
    return {'characters': len(value), 'sha256': hashlib.sha256(value.encode('utf-8')).hexdigest(),
        'replacementCharacters': value.count('\ufffd'),
        'firstLetterMarkerPresent': 'To Mrs. Saville, England.' in value,
        'noScriptTag': '<script' not in value.lower(),
        'isNotResourceUrl': not value.strip().startswith(('http://', 'https://'))}

def toc_facts(value):
    """Validate legacy raw relative URLs by their actual fixed-page resolution.

    Do not rewrite chapter.url, invent anchors, emit raw titles/URLs, accept a
    cross-book target, or delete the WebView-options check to obtain a pass.
    """
    rows = value if isinstance(value, list) else []
    urls, fragments = [], []
    options, bases, titles, indices, volumes = [], [], [], [], []
    for index, item in enumerate(rows):
        row = item if isinstance(item, dict) else {}
        url = row.get('url')
        has_options = isinstance(url, str) and url.endswith(OPTIONS)
        raw = url[:-len(OPTIONS)] if has_options else ''
        try:
            resolved = urllib.parse.urljoin(BOOK, raw)
            fragment = urllib.parse.urlsplit(resolved).fragment
            urls.append(resolved == BOOK + '#' + fragment and raw == raw.strip()
                and not any(ord(char) < 32 for char in raw) and fragment in TOC_FRAGMENTS)
        except ValueError:
            fragment = ''
            urls.append(False)
        fragments.append(fragment)
        options.append(has_options)
        bases.append(row.get('baseUrl') == BOOK)
        titles.append(isinstance(row.get('title'), str) and bool(row['title'].strip()))
        indices.append(type(row.get('index')) is int and row['index'] == index)
        volumes.append(row.get('isVolume') is False)
    expected_count = len(rows) == len(TOC_FRAGMENTS)
    facts = {'chapterCount': len(rows) if isinstance(value, list) else None,
        'webViewOptionsRetained': expected_count and all(options),
        'baseUrlsMatchFixedBook': expected_count and all(bases),
        'chaptersRemainInFixedBook': expected_count and all(urls),
        'fragmentsMatchExpected': tuple(fragments) == TOC_FRAGMENTS,
        'chapterTitlesPresent': expected_count and all(titles),
        'indicesSequential': expected_count and all(indices),
        'noVolumeEntries': expected_count and all(volumes),
        'relativeChapterUrls': sum(isinstance(row, dict) and isinstance(row.get('url'), str)
            and row['url'].startswith('#') for row in rows),
        'canonicalTocSha256': hashlib.sha256(json.dumps(value, ensure_ascii=False,
            sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest()}
    facts['fixedBookChapterUrlsValid'] = expected_count and all(facts[key] for key in (
        'webViewOptionsRetained', 'baseUrlsMatchFixedBook', 'chaptersRemainInFixedBook',
        'fragmentsMatchExpected', 'chapterTitlesPresent', 'indicesSequential', 'noVolumeEntries'))
    return facts

def require_environment(revision, output):
    if not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise Rejected('InvalidRevision')
    if (not hasattr(os, 'getuid') or os.getuid() != 10001 or
            os.environ.get('READER_BUILD_REVISION') != revision or
            os.environ.get('READER_APP_WEBVIEWRENDERER') != 'camoufox' or
            os.environ.get('READER_BROWSER_ALLOW_PRIVATE_NETWORKS') != 'false' or
            os.environ.get('READER_SERVER_BINDADDRESS') != '127.0.0.1' or
            os.environ.get('READER_APP_SECURE') != 'true'):
        raise Rejected('UnexpectedRuntimeIdentityOrGuard')
    if output.resolve() != Path('/verification-output') or not output.is_dir():
        raise Rejected('UnexpectedOutput')
    if any((output / name).exists() for name in ('GUTENBERG_REPORT.json', 'RUNTIME_RESOURCE_BUDGET.json')):
        raise Rejected('PreserveExistingReport')

def write_new(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2, sort_keys=True)
        stream.write('\n')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--expected-revision', required=True)
    parser.add_argument('--expected-jar-sha256', default=EXPECTED_JAR)
    parser.add_argument('--output', type=Path, default=Path('/verification-output'))
    args = parser.parse_args()
    expected_jar = validated_jar_sha256(args.expected_jar_sha256)
    require_environment(args.expected_revision, args.output)
    cgroup, processes, auth = helper('report-browser-cgroup'), helper('soak-bundled-browser'), helper('probe-public-metadata')
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    report = {'schemaVersion': 1, 'scope': 'fixed Gutenberg 84 anonymous bundled Reader; not original or archived parity',
        'source': BOOK, 'expectedImageRevision': args.expected_revision, 'jarSha256': expected_jar,
        'renderer': 'camoufox', 'runtimeUid': os.getuid(), 'privateNetworkGuardEnabled': True,
        'tlsErrorsIgnored': False, 'realCredentialsImported': False, 'privateLocalBooksRead': 0,
        'siteSearchRequested': False, 'captchaInteractedWith': False, 'rawBodyCookieOrErrorPersisted': False,
        'originalOrRemoteParityProven': False, 'realAuthenticationProven': False,
        'businessApiCalls': {}, 'passed': False}
    stop = threading.Event()
    maximum = {'worker': 0, 'browser': 0, 'driver': 0}
    monitor_failed = []
    logged_in = False
    def monitor():
        try:
            while not stop.is_set():
                found = processes.browser_processes()
                for kind in maximum:
                    maximum[kind] = max(maximum[kind], len(found[kind]))
                stop.wait(0.05)
        except Exception:
            monitor_failed.append(True)
    watcher = threading.Thread(target=monitor, daemon=True)
    started = time.monotonic()
    try:
        cgroup.verify_report(cgroup.collect_report(), require_no_swap=True)
        with Path('/app/reader.jar').open('rb') as stream:
            digest = hashlib.sha256()
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        if digest.hexdigest() != expected_jar:
            raise Rejected('UnexpectedJarBytes')
        successful(call(opener, '/getSystemInfo'))
        successful(call(opener, '/login', {'username': 'gutenberg' + secrets.token_hex(5),
            'password': 'Generated-' + secrets.token_hex(18), 'isLogin': False}))
        logged_in = True
        report['cookieRowsBefore'] = auth.cookie_count(call(opener, '/getBookSourceCookie'))
        if report['cookieRowsBefore'] != 0:
            raise Rejected('NonemptyFreshCookieIndex')
        successful(call(opener, '/saveBookSource', SOURCE_DEFINITION))
        saved = successful(call(opener, '/getBookSource', {'bookSourceUrl': SOURCE}))
        report['sourceRulesRetained'] = isinstance(saved, dict) and all(saved.get(key) == value for key, value in SOURCE_DEFINITION.items())
        if not report['sourceRulesRetained']:
            raise Rejected('SourceRulesNotRetained')
        watcher.start()
        for path, body, key in (('/getBookInfo', BOOK_BODY, 'metadata'),
                ('/getChapterList', BOOK_BODY, 'toc'), ('/getBookContent', CONTENT_BODY, 'chapter')):
            report['businessApiCalls'][path] = 1
            result = call(opener, path, body)
            report[key] = envelope(result)
            value = successful(result)
            if key == 'metadata':
                report[key].update({'dataIsObject': isinstance(value, dict),
                    'nameMatches': isinstance(value, dict) and 'Frankenstein' in str(value.get('name', '')),
                    'authorMatches': isinstance(value, dict) and 'Shelley' in str(value.get('author', ''))})
                if not all(report[key][item] for item in ('dataIsObject', 'nameMatches', 'authorMatches')):
                    raise Rejected('MissingRealMetadata')
            elif key == 'toc':
                report[key].update(toc_facts(value))
                if not report[key]['fixedBookChapterUrlsValid']:
                    raise Rejected('UnexpectedRealToc')
            else:
                report[key].update(content_facts(value))
                if report[key]['replacementCharacters'] or not all(report[key][item] for item in
                        ('firstLetterMarkerPresent', 'noScriptTag', 'isNotResourceUrl')):
                    raise Rejected('UnexpectedRealChapter')
            value = result = None
    except Rejected as failure:
        report['failureCategory'] = str(failure)
    except (Exception, SystemExit):
        report['failureCategory'] = 'ProbeTransportOrRuntimeFailure'
    finally:
        stop.set()
        if watcher.ident is not None:
            watcher.join(timeout=3)
        report['maximumObservedProcesses'] = maximum
        report['processMonitorPassed'] = not monitor_failed and not watcher.is_alive()
        report['bundledBrowserObserved'] = maximum['worker'] > 0 and maximum['browser'] > 0
        if logged_in:
            try:
                successful(call(opener, '/setBookSourceCookie', {'bookSource': SOURCE, 'cookie': ''}))
                report['cookieRowsAfter'] = auth.cookie_count(call(opener, '/getBookSourceCookie'))
                auth.verify_cookie_session_logged_out(call(opener, '/logout', {}), call(opener, '/getBookSourceCookie'))
                report['generatedSessionRevoked'] = True
            except Exception:
                report['cleanupCategory'] = 'GeneratedSessionCleanupFailed'
        try:
            budget = cgroup.collect_report('fixed Gutenberg anonymous bundled Reader; three finite business calls')
            write_new(args.output / 'RUNTIME_RESOURCE_BUDGET.json', budget)
            cgroup.verify_report(budget, require_no_swap=True)
            report['resourceGuardPassed'] = True
        except (Exception, SystemExit):
            report['resourceGuardPassed'] = False
        report['seconds'] = round(time.monotonic() - started, 3)
        report['passed'] = bool(not report.get('failureCategory') and not report.get('cleanupCategory')
            and set(report['businessApiCalls']) == {'/getBookInfo', '/getChapterList', '/getBookContent'}
            and report['sourceRulesRetained'] and report['resourceGuardPassed'] and report['processMonitorPassed']
            and report['bundledBrowserObserved'] and report.get('generatedSessionRevoked')
            and report.get('cookieRowsAfter') == 0)
        write_new(args.output / 'GUTENBERG_REPORT.json', report)
    print(json.dumps({'passed': report['passed'], 'businessApiCalls': report['businessApiCalls'],
        'failureCategory': report.get('failureCategory'), 'resourceGuardPassed': report['resourceGuardPassed']}))
    return 0 if report['passed'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
