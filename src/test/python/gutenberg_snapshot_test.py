"""Pure snapshot parity contract tests: no Reader, internet or real body."""
import copy
import importlib.util
import json
import subprocess
import sys
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('snapshot_probe', Path(__file__).resolve().parents[3] / 'scripts/compare-gutenberg-snapshot.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
BOOK = 'http://127.0.0.1:18896' + probe.PAGE_PATH


def side(clock=1005):
    def envelope(value):
        return {'isSuccess': True, 'errorMsg': '', 'data': value}
    result = {'metadata': envelope({'name': 'Frankenstein', 'author': 'Mary Shelley',
        'bookUrl': BOOK + probe.OPTIONS, 'type': 0, 'canUpdate': True,
        **{key: clock for key in probe.CLOCK_FIELDS}}),
        'metadataWindowMs': {'started': 1000, 'completed': 1010},
        'toc': envelope([{'index': i, 'isVolume': False, 'title': 'Generated title ' + str(i),
            'baseUrl': BOOK, 'url': '#' + fragment + probe.OPTIONS} for i, fragment in enumerate(probe.FRAGMENTS)]),
        'chapter0': envelope('To Mrs. Saville, England. ' + 'Generated alpha. ' * 100),
        'chapter1': envelope('To Mrs. Saville, England. ' + 'Generated beta. ' * 100),
        'sourceRulesRetained': True, 'readerStopped': True, 'logoutSucceeded': True,
        'businessApiCalls': {key: 1 for key in ('metadata', 'toc', 'chapter0', 'chapter1')},
        'httpStatuses': {key: 200 for key in ('metadata', 'toc', 'chapter0', 'chapter1')}}
    return result


def report(sides):
    return {'passed': True, 'originalExecuted': True, 'sides': {k: probe.facts(v) for k, v in sides.items()},
        'originalJarSha256': probe.ORIGINAL_SHA, 'restoredJarSha256': probe.RESTORED_SHA,
        'captureSha256': probe.CAPTURE_SHA, 'captureBytes': 468370, 'capturedPublicUrl': probe.PUBLIC_URL,
        'realSiteHttpsThreeWayProven': False, 'realAuthenticationProven': False,
        'rawBodyOrCredentialsExported': False, 'fullGoalComplete': False, 'privateBooksRead': 0,
        'comparison': probe.compare(sides)}


class SnapshotContracts(unittest.TestCase):
    def setUp(self):
        self.sides = {'original': side(), 'restored': side(1006), 'camoufox': side(1007)}

    def rejected(self):
        with self.assertRaises(probe.Rejected):
            probe.compare(self.sides)

    def test_complete_nonclock_parity_preserves_inputs(self):
        before = copy.deepcopy(self.sides)
        facts = probe.compare(self.sides)
        self.assertTrue(facts['nonClockMetadataFullJsonEqual'])
        self.assertFalse(facts['literalFullMetadataEqual'])
        self.assertEqual(before, self.sides)

    def test_identical_clocks_are_not_forced_to_differ(self):
        for value in self.sides.values():
            for key in probe.CLOCK_FIELDS:
                value['metadata']['data'][key] = 1005
        self.assertTrue(probe.compare(self.sides)['literalFullMetadataEqual'])

    def test_missing_side_rejected(self):
        del self.sides['camoufox']
        self.rejected()

    def test_metadata_extra_field_not_ignored(self):
        self.sides['camoufox']['metadata']['data']['extra'] = ''
        self.rejected()

    def test_nested_field_not_ignored(self):
        self.sides['camoufox']['metadata']['data']['nested'] = {'flag': True}
        self.rejected()

    def test_boolean_does_not_equal_integer(self):
        self.sides['camoufox']['metadata']['data']['type'] = False
        self.rejected()

    def test_stale_clock_rejected(self):
        self.sides['original']['metadata']['data']['durChapterTime'] = 999
        self.rejected()

    def test_future_clock_rejected(self):
        self.sides['restored']['metadata']['data']['lastCheckTime'] = 1011
        self.rejected()

    def test_clock_boolean_rejected(self):
        self.sides['camoufox']['metadata']['data']['latestChapterTime'] = True
        self.rejected()

    def test_missing_clock_rejected(self):
        del self.sides['original']['metadata']['data']['durChapterTime']
        self.rejected()

    def test_overlong_actual_window_rejected(self):
        self.sides['camoufox']['metadataWindowMs']['completed'] = 36001
        self.rejected()

    def test_missing_toc_row_rejected(self):
        self.sides['camoufox']['toc']['data'].pop()
        self.rejected()

    def test_missing_webview_option_rejected(self):
        self.sides['camoufox']['toc']['data'][0]['url'] = '#letter1'
        self.rejected()

    def test_cross_book_url_rejected(self):
        self.sides['camoufox']['toc']['data'][0]['url'] = 'https://example.test/#letter1' + probe.OPTIONS
        self.rejected()

    def test_default_toc_field_not_ignored(self):
        self.sides['camoufox']['toc']['data'][0]['newDefault'] = ''
        self.rejected()

    def test_empty_chapter_rejected(self):
        self.sides['camoufox']['chapter0']['data'] = ''
        self.rejected()

    def test_full_chapter_difference_rejected(self):
        self.sides['camoufox']['chapter1']['data'] += ' changed'
        self.rejected()

    def test_nonempty_error_message_rejected(self):
        for value in self.sides.values():
            value['chapter1']['errorMsg'] = 'unexpected'
        self.rejected()

    def test_200_only_is_not_success(self):
        for value in self.sides.values():
            value['chapter0']['isSuccess'] = False
        self.rejected()

    def test_wrong_http_status_rejected(self):
        self.sides['camoufox']['httpStatuses']['chapter0'] = 201
        self.rejected()

    def test_extra_business_retry_rejected(self):
        self.sides['camoufox']['businessApiCalls']['chapter0'] = 2
        self.rejected()

    def test_boolean_call_count_rejected(self):
        self.sides['camoufox']['businessApiCalls']['chapter0'] = True
        self.rejected()

    def test_facts_export_no_body_or_generated_credentials(self):
        observed = probe.facts(self.sides['original'])
        raw = json.dumps(observed)
        self.assertNotIn('Generated alpha', raw)
        self.assertNotIn('Generated beta', raw)
        self.assertNotIn('password', raw)
        self.assertEqual(observed['chapterCount'], 28)
        self.assertEqual(len(observed['operations']['chapter0']['bodySha256']), 64)

    def test_source_uses_same_rules_and_only_changes_alias(self):
        value = probe.source_definition('http://127.0.0.1:18896')
        original = probe.helper('probe-gutenberg-reader').SOURCE_DEFINITION
        for key in ('ruleToc', 'ruleContent', 'enabledCookieJar'):
            self.assertEqual(value[key], original[key])
        self.assertIn(BOOK, value['ruleBookInfo']['tocUrl'])

    def test_source_rejects_external_target(self):
        with self.assertRaises(probe.Rejected):
            probe.source_definition('https://www.gutenberg.org')

    def test_source_rejects_userinfo(self):
        with self.assertRaises(probe.Rejected):
            probe.source_definition('http://secret@127.0.0.1:18896')

    def test_source_rejects_query(self):
        with self.assertRaises(probe.Rejected):
            probe.source_definition('http://127.0.0.1:18896?unexpected=1')

    def test_normal_host_refuses_before_reading_inputs_or_creating_report(self):
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / 'report.json'
            command = [sys.executable, '-B', str(Path(probe.__file__))]
            for name in ('java', 'original', 'restored', 'snapshot', 'phase-handoff-dir'):
                command.extend(('--' + name, str(Path(directory) / ('must-not-exist-' + name))))
            command.extend(('--report', str(report)))
            result = subprocess.run(command, capture_output=True, timeout=10, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'root-verified private loopback-only namespace', result.stderr)
            self.assertFalse(report.exists())

    def test_finite_report_contract_accepts_complete_generated_observation(self):
        self.assertTrue(probe.validate_report(report(self.sides)))

    def test_report_requires_explicit_current_restored_sha_and_keeps_original_and_capture_fixed(self):
        value = report(self.sides)
        value['restoredJarSha256'] = 'b' * 64
        self.assertTrue(probe.validate_report(value, 'b' * 64))
        for wrong in (probe.RESTORED_SHA, 'c' * 64):
            with self.assertRaises(probe.Rejected):
                probe.validate_report(value, wrong)
        for key in ('originalJarSha256', 'captureSha256'):
            bad = copy.deepcopy(value); bad[key] = 'b' * 64
            with self.assertRaises(probe.Rejected):
                probe.validate_report(bad, 'b' * 64)

    def test_current_restored_identity_does_not_admit_malformed_or_nonstring_hash(self):
        for bad in (None, 1, True, [], '', 'g' * 64, 'A' * 64, 'a' * 63, 'a' * 64 + '\n'):
            with self.assertRaisesRegex(probe.Rejected, '^InvalidRestoredJarIdentity$'):
                probe.validated_restored_sha256(bad)

    def test_finite_report_rejects_incomplete_execution(self):
        value = report(self.sides)
        value['passed'] = False
        with self.assertRaises(probe.Rejected):
            probe.validate_report(value)

    def test_finite_report_rejects_wrong_original_sha(self):
        value = report(self.sides)
        value['originalJarSha256'] = '0' * 64
        with self.assertRaises(probe.Rejected):
            probe.validate_report(value)

    def test_finite_report_cannot_claim_real_authentication(self):
        value = report(self.sides)
        value['realAuthenticationProven'] = True
        with self.assertRaises(probe.Rejected):
            probe.validate_report(value)

    def test_finite_report_rejects_differing_toc_hash(self):
        value = report(self.sides)
        value['sides']['camoufox']['operations']['toc']['fullReturnDataSha256'] = '0' * 64
        with self.assertRaises(probe.Rejected):
            probe.validate_report(value)

    def test_finite_report_rejects_stale_clock(self):
        value = report(self.sides)
        value['sides']['original']['clocks']['durChapterTime'] = 999
        with self.assertRaises(probe.Rejected):
            probe.validate_report(value)


if __name__ == '__main__':
    unittest.main()
