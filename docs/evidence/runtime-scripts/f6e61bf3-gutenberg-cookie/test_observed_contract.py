"""Generated-only guard fixtures, not real website or browser evidence."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import types
import unittest

sys.modules['allowlist_transport'] = types.ModuleType('allowlist_transport')
live = types.ModuleType('compare_live')
live.upstream_snapshot = lambda *a, **k: (_ for _ in ()).throw(AssertionError('No external calls'))
sys.modules['compare_live'] = live
spec = importlib.util.spec_from_file_location('probe', Path(__file__).with_name('compare_cookie_script.py'))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
USERNAME = 'gutenberg0123456789'


def filename(key):
    code = 0
    for char in key:
        code = (31 * code + ord(char)) & 0xffffffff
    return str(code if code < 0x80000000 else code - 0x100000000)


class ContractTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def value(self, phase='seeded', namespace=USERNAME, stored=True, getters=None):
        for key in ('gutenberg.org', 'gutenberg.org_cookieJar'):
            path = self.root / 'storage/cache/cookie' / namespace / filename(key)
            if stored:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(probe.GENERATED_COOKIE.encode('ascii'))
        marker = dict(phase=phase, namespace=namespace, manualPresent=stored if getters is None else getters,
            jarPresent=stored if getters is None else getters)
        return dict(data=dict(variable=json.dumps({'_readerGeneratedCookieProbe':json.dumps(marker)})))

    def check(self, value, phase='seeded', original=False):
        self.observation = {}
        return probe.generated_cache_observation(self.root, USERNAME, phase, value, self.observation, original)

    def test_current_seed_requires_real_files_and_user_namespace(self):
        self.assertTrue(all(row['generatedLiteralPresent'] for row in self.check(self.value()).values()))
        self.assertTrue(self.observation['actualCookieNamespaceMatchesGeneratedUser'])

    def test_absent_current_seed_fails_and_preserves_records(self):
        with self.assertRaisesRegex(ValueError, 'was not stored'):
            self.check(self.value(stored=False))
        self.assertIn('actualGeneratedCookieStoreInputs', self.observation)

    def test_current_default_namespace_cannot_be_accepted(self):
        with self.assertRaisesRegex(ValueError, 'namespace differs'):
            self.check(self.value(namespace='default'))

    def test_original_default_is_explicit_observed_difference(self):
        self.check(self.value(namespace='default'), original=True)
        self.assertFalse(self.observation['actualCookieNamespaceMatchesGeneratedUser'])

    def test_original_second_user_shared_input_is_recorded_not_isolation(self):
        rows = self.check(self.value(phase='other-user',namespace='default'), 'other-user', True)
        self.assertTrue(all(row['generatedLiteralPresent'] for row in rows.values()))
        self.assertFalse(self.observation['actualCookieNamespaceMatchesGeneratedUser'])

    def test_current_other_user_shared_input_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'remained'):
            self.check(self.value(phase='other-user'), 'other-user')

    def test_current_cleared_input_is_rejected_if_present(self):
        with self.assertRaisesRegex(ValueError, 'remained'):
            self.check(self.value(phase='cleared'), 'cleared')

    def test_absent_baseline_accepted(self):
        rows = self.check(self.value(phase='baseline',stored=False), 'baseline')
        self.assertTrue(all(row['bytes']==0 for row in rows.values()))

    def test_getter_and_disk_disagreement_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'getters|getter'):
            self.check(self.value(getters=False))

    def test_missing_marker_rejected(self):
        with self.assertRaisesRegex(ValueError, 'execution marker'):
            self.check(dict(data=dict(variable=None)))

    def test_all_phase_options_preserve_url_and_do_not_expose_file_objects(self):
        public = types.SimpleNamespace(BOOK='https://www.gutenberg.org/cache/epub/84/pg84-images.html')
        probe.helper = lambda _: public
        for original in (False, True):
            for phase in probe.PHASES:
                request, digest = probe.observed_cookie_request({}, phase, original)
                url, text = request['url'].split(', ', 1)
                options = json.loads(text)
                self.assertEqual(url, public.BOOK)
                self.assertEqual(options['webJs'], probe.SCRIPT)
                self.assertNotIn('.file(', options['js'])
                self.assertNotIn('getCanonicalPath', options['js'])
                self.assertEqual('cookie.setCookie' in options['js'], original and phase=='seeded')
                self.assertEqual('cookie.removeCookie' in options['js'], original and phase=='cleared')
                self.assertEqual(len(digest), 64)


if __name__ == '__main__':
    unittest.main()
