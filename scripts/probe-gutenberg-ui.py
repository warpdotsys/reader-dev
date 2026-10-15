"""Actual default Vue UI and public book; bounded facts, never exported body/auth.

Runs as a disposable TEST CLIENT in the same pinned Reader image, not a second
WebView server or a deployment dependency. Server and client budgets are separate.
"""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import sys
import time
from urllib.parse import quote, urlencode, urlsplit

import importlib.util


def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


PROBE = helper('probe-gutenberg-reader')
BASE = PROBE.BASE
OUTPUT = Path('/verification-output')
BUSINESS = ('login', 'logout', 'getBookInfo', 'getChapterList', 'getBookContent', 'saveBook', 'saveBookProgress')
PAINT_READY = """selector => {
    const element = document.querySelector(selector);
    if (!element) return false;
    for (let node = element; node instanceof HTMLElement; node = node.parentElement) {
        const style = getComputedStyle(node);
        if (Number(style.opacity) < 0.99 || style.visibility !== 'visible' || style.display === 'none') return false;
        const animations = node.getAnimations({subtree: node === element});
        if (!animations.every(animation => animation.effect?.getComputedTiming().iterations === Infinity
            || ['finished', 'idle'].includes(animation.playState))) return false;
    }
    return true;
}"""


class Rejected(ValueError):
    pass


def check(value, reason):
    if not value:
        raise Rejected(reason)


def origin_allowed(url):
    try:
        target = urlsplit(url)
        return (target.scheme == 'http' and target.hostname == '127.0.0.1' and target.port == 18893
                and target.username is None and target.password is None)
    except ValueError:
        return False


def bounded_json(response):
    check(response.status == 200, 'UnexpectedHttpStatus')
    raw = response.body()
    check(len(raw) <= PROBE.MAX_BYTES, 'ResponseTooLarge')
    value = json.loads(raw.decode('utf-8'))
    check(isinstance(value, dict), 'UnexpectedEnvelope')
    return value


def success(response):
    value = bounded_json(response)
    check(value.get('isSuccess') is True and value.get('errorMsg') == '', 'ReaderRejected')
    return value.get('data')


def body_facts(text):
    check(isinstance(text, str) and 1000 <= len(text) <= PROBE.MAX_BYTES, 'MissingPaintedContent')
    return {'characters': len(text), 'sha256': hashlib.sha256(text.encode('utf-8')).hexdigest(),
            'replacementCharacters': text.count('\ufffd')}


def normalized(text):
    return re.sub(r'\s+', '', text)


def write_once(name, value):
    with (OUTPUT / name).open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--expected-revision', required=True)
    parser.add_argument('--expected-jar-sha256', default=PROBE.EXPECTED_JAR)
    args = parser.parse_args()
    expected_jar = PROBE.validated_jar_sha256(args.expected_jar_sha256)
    check(re.fullmatch('[0-9a-f]{40}', args.expected_revision), 'InvalidRevision')
    check(os.getuid() == 10001, 'UnexpectedRuntimeUid')
    for name in ('GUTENBERG_UI_REPORT.json', 'UI_BROWSER_RESOURCE_BUDGET.json', 'ui-login.png', 'ui-toolbar.png', 'ui-logout.png'):
        check(not (OUTPUT / name).exists(), 'ExistingEvidence')
    cgroup = helper('report-browser-cgroup')
    report = {'schemaVersion': 1, 'scope': 'actual bundled default Vue UI; fixed anonymous Gutenberg 84',
              'expectedImageRevision': args.expected_revision, 'jarSha256': expected_jar,
              'clientRuntimeUid': os.getuid(), 'testClientNotDeploymentDependency': True,
              'clientOutsideReaderServerBudget': True, 'combinedTwoGiBBudgetProven': False,
              'realCredentialsImported': False, 'privateLocalBooksRead': 0,
              'responseMocked': False, 'siteSearchRequested': False, 'tlsErrorsIgnored': False,
              'rawBodyCookieOrErrorPersisted': False, 'bodyScreenshotsExported': False,
              'originalOrRemoteParityProven': False, 'passed': False, 'stage': 'launch',
              'apiObservations': [], 'paintedChapters': [], 'screenshots': {}}
    blocked, page_errors, observation_errors = [], [], []
    content_by_index = {}
    started = time.monotonic()
    os.environ['OPENBLAS_NUM_THREADS'] = '1'
    os.environ['OMP_NUM_THREADS'] = '1'
    os.environ['MKL_NUM_THREADS'] = '1'
    try:
        cgroup.verify_report(cgroup.collect_report(), require_no_swap=True)
        digest = hashlib.sha256()
        with Path('/app/reader.jar').open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        check(digest.hexdigest() == expected_jar, 'UnexpectedJarBytes')
        # Do not export third-party diagnostics or a Playwright exception which
        # may contain a request URL/token. No real profile or cookies are loaded.
        with contextlib.redirect_stdout(sys.stderr):
            from camoufox import Camoufox, DefaultAddons, NewContext
            with Camoufox(headless=True, browser='152.0.4-beta.30', geoip=False,
                          block_webrtc=True, exclude_addons=[DefaultAddons.UBO],
                          i_know_what_im_doing=True,
                          firefox_user_prefs={'network.dns.disablePrefetch': True,
                                              'network.prefetch-next': False,
                                              'network.predictor.enabled': False,
                                              'network.http.speculative-parallel-limit': 0}) as browser:
                context = NewContext(browser, viewport={'width': 1280, 'height': 900},
                                     accept_downloads=False, service_workers='block')
                try:
                    context.set_default_timeout(35000)
                    context.set_default_navigation_timeout(35000)

                    def route(request_route):
                        if origin_allowed(request_route.request.url):
                            request_route.continue_()
                        else:
                            blocked.append(True)
                            request_route.abort()
                    context.route('**/*', route)
                    page = context.new_page()
                    page.on('pageerror', lambda error: page_errors.append(True))

                    def observe(response):
                        endpoint = urlsplit(response.url).path.removeprefix('/reader3/')
                        if endpoint not in BUSINESS:
                            return
                        try:
                            value = bounded_json(response)
                            facts = PROBE.envelope((response.status, value))
                            facts['endpoint'] = endpoint
                            check(len(report['apiObservations']) < 64, 'TooManyBusinessResponses')
                            report['apiObservations'].append(facts)
                            if endpoint == 'getChapterList' and value.get('isSuccess') is True:
                                report['toc'] = PROBE.toc_facts(value.get('data'))
                            if endpoint == 'getBookContent' and value.get('isSuccess') is True:
                                from urllib.parse import parse_qs
                                index = int(parse_qs(urlsplit(response.url).query).get('index', ['-1'])[0])
                                text = value.get('data')
                                check(index in (0, 1, 2) and isinstance(text, str), 'UnexpectedContentRequest')
                                content_by_index[index] = text  # Only in memory, never serialized.
                        except Exception:
                            observation_errors.append(True)
                    page.on('response', observe)

                    def screenshot(locator, name):
                        selector = '.topbar' if name == 'toolbar' else '.login-page'
                        page.wait_for_function(PAINT_READY, arg=selector)
                        page.evaluate('() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))')
                        raw = locator.screenshot()
                        # A transparent/sticky toolbar could reveal body behind
                        # it. Its image stays in memory; only empty login pages
                        # can be exported. This is not manual visual approval.
                        persisted = name in ('login', 'logout')
                        if persisted:
                            path = OUTPUT / ('ui-' + name + '.png')
                            with path.open('xb') as stream:
                                stream.write(raw)
                        report['screenshots'][name] = {'sha256': hashlib.sha256(raw).hexdigest(),
                                                       'bytes': len(raw), 'persisted': persisted,
                                                       'computedOpacityAndFiniteAnimationsSettled': True}

                    report['stage'] = 'register'
                    page.goto(BASE + '/login')
                    page.locator('.login-page').wait_for()
                    release = context.request.get(BASE + '/assets/reader-release.json')
                    check(release.status == 200, 'ReleaseIdentityUnavailable')
                    identity = release.json()
                    check(identity.get('version') == '4.0.7' and identity.get('buildRevision') == args.expected_revision,
                          'UnexpectedUiReleaseIdentity')
                    report['defaultUiReleaseIdentityMatches'] = True
                    screenshot(page.locator('.login-page'), 'login')
                    username = 'gutenberg' + secrets.token_hex(5)
                    password = 'Generated-' + secrets.token_hex(18)
                    page.locator('.mode-switch button').nth(1).click()
                    page.locator('input[autocomplete=username]').fill(username)
                    page.locator('input[autocomplete=current-password]').fill(password)
                    page.locator('.submit-btn').click()
                    page.locator('.bookshelf-page').wait_for()
                    report['uiRegistrationSucceeded'] = True
                    page.locator('.logout-btn').click()
                    page.locator('.login-page').wait_for()
                    report['stage'] = 'login'
                    page.locator('input[autocomplete=username]').fill(username)
                    page.locator('input[autocomplete=current-password]').fill(password)
                    page.locator('input[type=checkbox]').uncheck()
                    page.locator('.submit-btn').click()
                    page.locator('.bookshelf-page').wait_for()
                    token = page.evaluate("sessionStorage.getItem('reader_access_token')")
                    check(isinstance(token, str) and token.startswith(username + ':'), 'UnexpectedTokenNamespace')
                    report['uiLoginSucceeded'] = report['generatedNamespaceVerified'] = True
                    query = urlencode({'accessToken': token, 'readerAuth': 'access-token'})
                    success(context.request.post(BASE + '/reader3/saveBookSource?' + query,
                                                 data=PROBE.SOURCE_DEFINITION))
                    saved = success(context.request.post(BASE + '/reader3/getBookSource?' + query,
                        data={'bookSourceUrl': PROBE.SOURCE}))
                    check(isinstance(saved, dict) and all(saved.get(key) == value for key, value in PROBE.SOURCE_DEFINITION.items()),
                          'SourceRulesNotRetained')
                    report['sourceSeededViaApiNotUi'] = report['sourceRulesRetained'] = True
                    report['stage'] = 'detail-and-add'
                    page.goto(BASE + '/book/' + quote(PROBE.BOOK_REQUEST, safe='') + '?' + urlencode({'origin': PROBE.SOURCE}))
                    page.locator('.add-btn').wait_for()
                    check('Frankenstein' in page.locator('.book-name').inner_text(), 'MissingRealUiMetadata')
                    page.locator('.add-btn').click()
                    page.locator('.read-btn:not(.ghost)').wait_for()
                    report['addedViaUi'] = True
                    report['stage'] = 'read-first'
                    page.locator('.read-btn:not(.ghost)').click()

                    def painted(index):
                        page.wait_for_function("arg => { const title=document.querySelector('.chapter-title');"
                            "const el=document.querySelector('.reader-content');return title && title.textContent.includes('Letter ' + arg)"
                            "&& el && el.innerText.length>1000 && !document.querySelector('.loading-text'); }", arg=index + 1)
                        page.wait_for_function(PAINT_READY, arg='.reader-content')
                        page.evaluate('() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))')
                        text = page.locator('.reader-content').inner_text()
                        facts = body_facts(text)
                        check(facts['replacementCharacters'] == 0, 'PaintedEncodingError')
                        check(index in content_by_index and normalized(text) == normalized(content_by_index[index]),
                              'PaintedContentNotMatchingActualApi')
                        if index == 0:
                            check('To Mrs. Saville, England.' in text, 'MissingRealFirstLetter')
                        facts.update({'index': index, 'matchesActualApiAfterWhitespaceNormalization': True,
                                      'postAnimationFramesObserved': True,
                                      'computedOpacityAndFiniteAnimationsSettled': True})
                        report['paintedChapters'].append(facts)
                    painted(0)
                    screenshot(page.locator('.topbar'), 'toolbar')
                    report['stage'] = 'next'
                    page.locator('.chapter-nav button').last.click()
                    painted(1)
                    report['stage'] = 'reload'
                    page.reload()
                    painted(1)
                    report['stage'] = 'previous'
                    page.locator('.chapter-nav button').first.click()
                    painted(0)
                    report['stage'] = 'return-and-logout'
                    page.goto(BASE + '/')
                    page.locator('.bookshelf-page').wait_for()
                    check(page.locator('.book-card').count() == 1, 'UnexpectedShelfCount')
                    page.locator('.logout-btn').click()
                    page.locator('.login-page').wait_for()
                    check(page.evaluate("() => ['localStorage','sessionStorage'].every(s => !window[s].getItem('reader_access_token'))"),
                          'FrontendTokenNotCleared')
                    denied = bounded_json(context.request.get(BASE + '/reader3/getBookshelf?' + query))
                    check(denied.get('isSuccess') is False and denied.get('data') == 'NEED_LOGIN', 'SessionNotRevoked')
                    report['generatedSessionRevoked'] = True
                    page.goto(BASE + '/')
                    page.locator('.login-page').wait_for()
                    check(not page.locator('input[autocomplete=current-password]').input_value(), 'NonemptyScreenshotPassword')
                    screenshot(page.locator('.login-page'), 'logout')
                    check(not blocked and not page_errors and not observation_errors, 'ClientObservationFailure')
                    check(report.get('toc', {}).get('fixedBookChapterUrlsValid') is True, 'UnexpectedUiToc')
                    check(all(row['isSuccess'] is True and row['errorMsgEmpty'] is True for row in report['apiObservations']),
                          'UiBusinessApiRejected')
                    report['stage'] = 'complete'
                    report['passed'] = True
                finally:
                    context.close()
            report['testBrowserClosed'] = True
    except Exception as error:
        report['failureClass'] = str(error) if isinstance(error, Rejected) else type(error).__name__
    finally:
        report.update({'elapsedSeconds': round(time.monotonic() - started, 3),
                       'blockedClientRequests': len(blocked), 'pageErrorCount': len(page_errors),
                       'apiObservationErrorCount': len(observation_errors)})
        budget = cgroup.collect_report('disposable packaged-browser UI test CLIENT only; Reader server excluded')
        try:
            cgroup.verify_report(budget, require_no_swap=True)
        except SystemExit:
            report['passed'] = False
            report['budgetAccepted'] = False
        else:
            report['budgetAccepted'] = True
        write_once('UI_BROWSER_RESOURCE_BUDGET.json', budget)
        write_once('GUTENBERG_UI_REPORT.json', report)
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
