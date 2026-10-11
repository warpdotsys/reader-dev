import importlib.util
import json
from pathlib import Path
import struct
import subprocess
import unittest
import zlib

ROOT = Path(__file__).parents[3]
SCRIPT = ROOT / 'scripts/probe-gutenberg-ui.py'
spec = importlib.util.spec_from_file_location('gutenberg_ui_probe', SCRIPT)
UI = importlib.util.module_from_spec(spec)
spec.loader.exec_module(UI)
spec = importlib.util.spec_from_file_location('gutenberg_ui_verify', ROOT / 'scripts/verify-gutenberg-ui-report.py')
VERIFY = importlib.util.module_from_spec(spec)
spec.loader.exec_module(VERIFY)


class Response:
    def __init__(self, value, status=200):
        self.status = status
        self.raw = json.dumps(value).encode()

    def body(self):
        return self.raw


class GutenbergUiProbeTests(unittest.TestCase):
    def test_client_only_permits_the_exact_isolated_reader_origin(self):
        self.assertTrue(UI.origin_allowed(UI.BASE + '/reader3/getBookInfo?generated=1'))
        for url in ('https://www.gutenberg.org/', 'http://127.0.0.1:18931/',
                    'http://localhost:18893/', 'https://127.0.0.1:18893/',
                    'http://generated:credential@127.0.0.1:18893/', 'http://127.0.0.1:bad/'):
            self.assertFalse(UI.origin_allowed(url))

    def test_body_facts_never_export_body(self):
        text = 'GENERATED_BODY_MUST_NOT_BE_EXPORTED' * 100
        facts = UI.body_facts(text)
        self.assertEqual({'characters', 'sha256', 'replacementCharacters'}, set(facts))
        self.assertNotIn('GENERATED_BODY', json.dumps(facts))
        self.assertEqual(64, len(facts['sha256']))

    def test_missing_oversized_or_nontext_body_is_not_success(self):
        for text in ('', 'x' * (UI.PROBE.MAX_BYTES + 1), None, {}, []):
            with self.assertRaisesRegex(UI.Rejected, '^MissingPaintedContent$'):
                UI.body_facts(text)

    def test_encoding_error_remains_observable(self):
        self.assertEqual(1, UI.body_facts('x' * 1100 + '\ufffd')['replacementCharacters'])

    def test_only_whitespace_is_normalized_for_painted_api_comparison(self):
        self.assertEqual(UI.normalized('A \n B\tC'), 'ABC')
        self.assertNotEqual(UI.normalized('AB\ufffdC'), 'ABC')

    def test_http_or_business_failure_does_not_export_raw_error(self):
        for response, reason in ((Response({}, 500), 'UnexpectedHttpStatus'),
                (Response({'isSuccess': False, 'errorMsg': 'PRIVATE_ERROR'}), 'ReaderRejected'),
                (Response({'isSuccess': 1, 'errorMsg': ''}), 'ReaderRejected'),
                (Response([]), 'UnexpectedEnvelope')):
            with self.assertRaisesRegex(UI.Rejected, '^' + reason + '$'):
                UI.success(response)

    def test_good_envelope_preserves_actual_body_for_in_memory_comparison(self):
        self.assertEqual('GENERATED', UI.success(Response({'isSuccess': True, 'errorMsg': '', 'data': 'GENERATED'})))

    def test_ui_flow_uses_actual_controls_without_fake_responses_or_body_upload(self):
        text = SCRIPT.read_text(encoding='utf-8')
        for item in ("page.locator('.submit-btn').click()", "page.locator('.add-btn').click()",
                "page.locator('.chapter-nav button').last.click()", "page.reload()",
                "page.locator('.chapter-nav button').first.click()", "page.locator('.logout-btn').click()",
                "normalized(text) == normalized(content_by_index[index])", "'testClientNotDeploymentDependency': True",
                "persisted = name in ('login', 'logout')", "'budgetAccepted'", "'SessionNotRevoked'"):
            self.assertIn(item, text)
        for forbidden in ('route.fulfill', 'ignore_https_errors=True', 'storage_state=',
                          'trace.start', 'record_video', 'PRIVATE_TOKEN'):
            self.assertNotIn(forbidden, text)

    def test_wrapper_makes_ui_mandatory_and_removes_only_owned_client(self):
        wrapper = (ROOT / 'scripts/probe-gutenberg-native-image.sh').read_text(encoding='utf-8')
        for item in ('--network "container:$container_id"', '--entrypoint python "$image"',
                'probe-gutenberg-ui.py --expected-revision "$revision"',
                'UI_SERVER_RESOURCE_BUDGET.json', 'UI_CLIENT_CLEANUP.json', 'docker rm -f "$client_id"',
                'timeout --signal=TERM 210s docker start -a "$client_id"'):
            self.assertIn(item, wrapper)
        self.assertEqual(2, wrapper.count('--memory=2g --memory-swap=2g --pids-limit=256 --cpus=2'))
        for forbidden in ('--privileged', '--network host', '--ignore', '|| true', 'docker system prune'):
            self.assertNotIn(forbidden, wrapper)

    def complete_report(self):
        report = {'schemaVersion': 1, 'expectedImageRevision': 'a' * 40, 'jarSha256': UI.PROBE.EXPECTED_JAR,
                  'scope': 'GENERATED_TEST_ONLY', 'clientRuntimeUid': 10001, 'stage': 'complete', 'elapsedSeconds': 50}
        for key in ('passed', 'budgetAccepted', 'testClientNotDeploymentDependency', 'clientOutsideReaderServerBudget',
                    'defaultUiReleaseIdentityMatches', 'uiRegistrationSucceeded', 'uiLoginSucceeded',
                    'generatedNamespaceVerified', 'sourceSeededViaApiNotUi', 'sourceRulesRetained',
                    'addedViaUi', 'generatedSessionRevoked', 'testBrowserClosed'):
            report[key] = True
        for key in ('combinedTwoGiBBudgetProven', 'realCredentialsImported', 'responseMocked', 'siteSearchRequested',
                    'tlsErrorsIgnored', 'rawBodyCookieOrErrorPersisted', 'bodyScreenshotsExported', 'originalOrRemoteParityProven'):
            report[key] = False
        for key in ('blockedClientRequests', 'pageErrorCount', 'apiObservationErrorCount', 'privateLocalBooksRead'):
            report[key] = 0
        report['paintedChapters'] = [dict(index=index, characters=1600, sha256=str(index) * 64,
            replacementCharacters=0, matchesActualApiAfterWhitespaceNormalization=True,
            postAnimationFramesObserved=True, computedOpacityAndFiniteAnimationsSettled=True) for index in (0, 1, 1, 0)]
        report['toc'] = UI.PROBE.toc_facts([dict(url='#' + fragment + UI.PROBE.OPTIONS,
            baseUrl=UI.PROBE.BOOK, title='GENERATED_TITLE_' + str(index), index=index, isVolume=False)
            for index, fragment in enumerate(UI.PROBE.TOC_FRAGMENTS)])
        report['apiObservations'] = [dict(endpoint=endpoint, status=200, isSuccess=True, errorMsgEmpty=True,
            fields=['data', 'errorMsg', 'isSuccess']) for endpoint in
            ('login', 'logout', 'login', 'getBookInfo', 'saveBook', 'getChapterList', 'getBookContent',
             'getBookContent', 'saveBookProgress', 'logout')]
        report['screenshots'] = {name: dict(sha256='a' * 64, bytes=20000, persisted=name != 'toolbar',
                                          computedOpacityAndFiniteAnimationsSettled=True)
                                 for name in ('login', 'toolbar', 'logout')}
        return report

    def test_verifier_accepts_only_complete_generated_fixture(self):
        counts = VERIFY.verify_report(self.complete_report(), 'a' * 40)
        self.assertEqual(2, counts['getBookContent'])

    def test_verifier_rejects_different_identity_skip_or_unbounded_raw_fields(self):
        for key, value in (('expectedImageRevision', 'b' * 40), ('passed', False), ('stage', 'next'),
                           ('rawBody', 'PRIVATE_BODY'), ('combinedTwoGiBBudgetProven', True),
                           ('generatedSessionRevoked', False)):
            report = self.complete_report()
            report[key] = value
            with self.assertRaises(AssertionError):
                VERIFY.verify_report(report, 'a' * 40)

    def test_verifier_rejects_unpainted_or_different_cached_chapter(self):
        for key, value in (('index', 2), ('sha256', 'f' * 64), ('replacementCharacters', 1),
                           ('matchesActualApiAfterWhitespaceNormalization', False)):
            report = self.complete_report()
            report['paintedChapters'][2][key] = value
            with self.assertRaises(AssertionError):
                VERIFY.verify_report(report, 'a' * 40)

    def test_verifier_rejects_missing_actual_login_add_or_progress(self):
        for endpoint in ('login', 'saveBook', 'saveBookProgress', 'getBookContent'):
            report = self.complete_report()
            report['apiObservations'] = [row for row in report['apiObservations'] if row['endpoint'] != endpoint]
            with self.assertRaises(AssertionError):
                VERIFY.verify_report(report, 'a' * 40)

    def test_verifier_rejects_failed_business_or_private_error_field(self):
        for key, value in (('status', 500), ('isSuccess', False), ('errorMsgEmpty', False),
                           ('errorMsg', 'PRIVATE_ERROR')):
            report = self.complete_report()
            report['apiObservations'][0][key] = value
            with self.assertRaises(AssertionError):
                VERIFY.verify_report(report, 'a' * 40)

    def test_verifier_rejects_exported_toolbar_or_body_screenshot(self):
        for key, value in (('persisted', True), ('body', 'PRIVATE_BODY')):
            report = self.complete_report()
            report['screenshots']['toolbar'][key] = value
            with self.assertRaises(AssertionError):
                VERIFY.verify_report(report, 'a' * 40)

    def test_workflow_cannot_accept_ui_as_optional_after_api_green(self):
        workflow = (ROOT / '.github/workflows/browser-image.yml').read_text(encoding='utf-8')
        segment = workflow.split('\n  public-metadata:\n', 1)[1].split('\n  compare-public-webview:', 1)[0]
        self.assertIn('python3 scripts/verify-gutenberg-ui-report.py', segment)
        self.assertNotIn('continue-on-error', segment)

    def generated_png(self, channels=3, method=0, extra=b''):
        def chunk(kind, data):
            return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data) & 0xffffffff)
        header = struct.pack('>IIBBBBB', 4, 4, 8, 2 if channels == 3 else 6, 0, 0, 0)
        rows = bytes([method]) + b'\0' * (4 * channels)
        return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', header) +
                chunk(b'IDAT', zlib.compress(rows * 4 + extra)) + chunk(b'IEND', b''))

    def test_png_uniform_picture_remains_a_measurable_failure_not_visual_pass(self):
        for channels in (3, 4):
            facts = VERIFY.png_visual_facts(self.generated_png(channels))
            self.assertEqual(1, facts['distinctPixelValues'])
            self.assertEqual(0, facts['nonDominantPixels'])

    def test_png_all_five_filter_methods_decode_generated_uniform_samples(self):
        for method in range(5):
            self.assertEqual(1, VERIFY.png_visual_facts(self.generated_png(method=method))['distinctPixelValues'])

    def test_png_nonuniform_rgb_roundtrip_for_every_filter_method(self):
        def chunk(kind, data):
            return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data) & 0xffffffff)
        for method in range(5):
            prior, filtered = [0] * 12, bytearray()
            for y in range(4):
                row = [x * 20 + y * 7 + channel for x in range(4) for channel in range(3)]
                filtered.append(method)
                for i, value in enumerate(row):
                    left = row[i - 3] if i >= 3 else 0
                    above = prior[i]
                    corner = prior[i - 3] if i >= 3 else 0
                    predictions = (0, left, above, (left + above) // 2)
                    if method == 4:
                        p = left + above - corner
                        candidates = (left, above, corner)
                        predicted = min(enumerate(candidates), key=lambda pair: (abs(p - pair[1]), pair[0]))[1]
                    else:
                        predicted = predictions[method]
                    filtered.append((value - predicted) % 256)
                prior = row
            header = struct.pack('>IIBBBBB', 4, 4, 8, 2, 0, 0, 0)
            raw = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', header) + chunk(b'IDAT', zlib.compress(filtered)) + chunk(b'IEND', b'')
            self.assertEqual({'width': 4, 'height': 4, 'distinctPixelValues': 16, 'nonDominantPixels': 15},
                             VERIFY.png_visual_facts(raw))

    def test_png_bad_filter_crc_truncation_or_inflated_stream_is_rejected(self):
        for raw in (self.generated_png(method=5), self.generated_png()[:-1], self.generated_png(extra=b'\0'),
                    self.generated_png()[:-8] + b'corrupt!'):
            with self.assertRaises(AssertionError):
                VERIFY.png_visual_facts(raw)

    def test_verifier_no_longer_accepts_prepaint_row_or_picture(self):
        for field in ('paintedChapters', 'screenshots'):
            report = self.complete_report()
            row = report[field][0] if field == 'paintedChapters' else report[field]['logout']
            row['computedOpacityAndFiniteAnimationsSettled'] = False
            with self.assertRaises(AssertionError):
                VERIFY.verify_report(report, 'a' * 40)

    def test_actual_javascript_gate_rejects_hidden_opacity_or_running_finite_animation(self):
        code = 'const gate=(' + UI.PAINT_READY + ');' + r'''
            global.HTMLElement = class Element {};
            global.getComputedStyle = node => node.style;
            const make = (opacity='1', state='finished', iterations=1) => {
                const node = new HTMLElement(); node.style={opacity,visibility:'visible',display:'block'};
                node.parentElement=null;node.getAnimations=()=>[{playState:state,effect:{getComputedTiming:()=>({iterations})}}];
                return node;
            };
            let current=make();global.document={querySelector:()=>current};
            if(!gate('.test'))throw Error('finished rejected');
            for(const node of [make('0'),make('0.5'),make('1','running')]){
                current=node;if(gate('.test'))throw Error('unfinished accepted');
            }
            current=make();current.parentElement=make('0');
            if(gate('.test'))throw Error('hidden ancestor accepted');
            current=make('1','running',Infinity);if(!gate('.test'))throw Error('infinite spinner blocks ready forever');
            current=null;if(gate('.test'))throw Error('missing accepted');
        '''
        result = subprocess.run(['node', '-e', code], capture_output=True, timeout=10)
        self.assertEqual(0, result.returncode, result.stderr.decode('utf-8', errors='replace'))


if __name__ == '__main__':
    unittest.main()
