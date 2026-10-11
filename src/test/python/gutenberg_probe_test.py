import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).parents[3]
spec = importlib.util.spec_from_file_location('gutenberg_probe', ROOT / 'scripts/probe-gutenberg-reader.py')
PROBE = importlib.util.module_from_spec(spec)
spec.loader.exec_module(PROBE)

class GutenbergProbeTests(unittest.TestCase):
    def test_probe_and_wrapper_pin_the_actual_new_product_instead_of_old_frontend(self):
        digest = 'f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b'
        self.assertEqual(digest, PROBE.EXPECTED_JAR)
        wrapper = (ROOT / 'scripts/probe-gutenberg-native-image.sh').read_text(encoding='utf-8')
        self.assertIn('= ' + digest, wrapper)
        self.assertNotIn('9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84', wrapper)

    def generated_toc(self, absolute=False):
        return [dict(url=(PROBE.BOOK if absolute else '') + '#' + fragment + PROBE.OPTIONS,
            baseUrl=PROBE.BOOK, title='GENERATED_TITLE_NOT_EXPORTED_' + str(index),
            index=index, isVolume=False) for index, fragment in enumerate(PROBE.TOC_FRAGMENTS)]

    def test_legacy_relative_toc_is_validated_without_rewriting_it(self):
        toc = self.generated_toc()
        before = json.dumps(toc)
        facts = PROBE.toc_facts(toc)
        self.assertIs(True, facts['fixedBookChapterUrlsValid'])
        self.assertEqual(28, facts['relativeChapterUrls'])
        self.assertEqual(before, json.dumps(toc))
        self.assertNotIn('GENERATED_TITLE_NOT_EXPORTED', json.dumps(facts))
        self.assertNotIn('#letter1', json.dumps(facts))
        self.assertEqual(64, len(facts['canonicalTocSha256']))

    def test_absolute_toc_still_requires_the_same_fixed_page_and_fragments(self):
        facts = PROBE.toc_facts(self.generated_toc(absolute=True))
        self.assertIs(True, facts['fixedBookChapterUrlsValid'])
        self.assertEqual(0, facts['relativeChapterUrls'])

    def test_toc_foreign_target_or_base_is_rejected(self):
        for key, value in (('url', 'https://other.invalid/#letter1' + PROBE.OPTIONS),
                ('url', '/cache/epub/99/other.html#letter1' + PROBE.OPTIONS),
                ('baseUrl', 'https://other.invalid/'), ('url', '//other.invalid/#letter1' + PROBE.OPTIONS)):
            toc = self.generated_toc()
            toc[0][key] = value
            self.assertIs(False, PROBE.toc_facts(toc)['fixedBookChapterUrlsValid'])

    def test_toc_options_order_duplicates_and_wrong_anchor_are_rejected(self):
        for url in ('#letter1', '#letter1, {"webView":false}', '#wrong' + PROBE.OPTIONS,
                '#letter2' + PROBE.OPTIONS, '\n#letter1' + PROBE.OPTIONS):
            toc = self.generated_toc()
            toc[0]['url'] = url
            self.assertIs(False, PROBE.toc_facts(toc)['fixedBookChapterUrlsValid'])
        toc = self.generated_toc()
        toc.reverse()
        self.assertIs(False, PROBE.toc_facts(toc)['fixedBookChapterUrlsValid'])

    def test_toc_missing_wrong_types_and_default_fields_are_rejected(self):
        for value in (None, {}, [], self.generated_toc()[:-1], [None] * 28):
            self.assertIs(False, PROBE.toc_facts(value)['fixedBookChapterUrlsValid'])
        for key, value in (('url', {}), ('title', ''), ('title', None), ('index', True),
                ('index', 1), ('isVolume', True), ('isVolume', None)):
            toc = self.generated_toc()
            toc[0][key] = value
            self.assertIs(False, PROBE.toc_facts(toc)['fixedBookChapterUrlsValid'])

    def test_raw_chapter_rule_keeps_relative_href_and_original_options(self):
        self.assertEqual("@href@js:result + ', {\"webView\":true}'", PROBE.CHAPTER_URL_RULE)
        self.assertEqual(PROBE.CHAPTER_URL_RULE, PROBE.SOURCE_DEFINITION['ruleToc']['chapterUrl'])

    def test_source_is_one_fixed_https_book_no_search_or_login(self):
        source = PROBE.SOURCE_DEFINITION
        self.assertEqual('https://www.gutenberg.org', source['bookSourceUrl'])
        self.assertIs(False, source['enabledCookieJar'])
        self.assertFalse({'searchUrl', 'loginUrl', 'header', 'loginInfo'} & source.keys())
        self.assertEqual(PROBE.BOOK, 'https://www.gutenberg.org/cache/epub/84/pg84-images.html')
        self.assertIn('webView', source['ruleToc']['chapterUrl'])
        self.assertNotIn('/ebooks/search', json.dumps(source))

    def test_only_fixed_body_requests_are_allowed(self):
        for path in ('/getBookInfo', '/getChapterList'):
            PROBE.allowed_request(path, dict(PROBE.BOOK_BODY))
        PROBE.allowed_request('/getBookContent', dict(PROBE.CONTENT_BODY))
        for path, body in (('/getBookContent', dict(PROBE.CONTENT_BODY, index=1)),
                ('/getBookInfo', dict(PROBE.BOOK_BODY, url='https://private.invalid/')),
                ('/searchBook', {}), ('/setSourceLogin', {}), ('/getBookContent', {}),
                ('/getBookInfo', dict(PROBE.BOOK_BODY, cookie='PRIVATE_COOKIE'))):
            with self.assertRaises(PROBE.Rejected):
                PROBE.allowed_request(path, body)

    def test_author_rule_selects_by_heading_not_subtitle(self):
        self.assertEqual(r'@CSS:h2:matches(^by\s)@text', PROBE.AUTHOR_RULE)
        self.assertEqual(PROBE.AUTHOR_RULE, PROBE.SOURCE_DEFINITION['ruleBookInfo']['author'])
        self.assertNotIn('h3', PROBE.SOURCE_DEFINITION['ruleBookInfo']['author'])

    def test_only_generated_accounts_can_be_used(self):
        good = {'username': 'gutenberg0123456789', 'password': 'Generated-' + 'a' * 36, 'isLogin': False}
        PROBE.allowed_request('/login', good)
        for bad in (dict(good, username='realaccount'), dict(good, password='realpassword'),
                dict(good, isLogin=True), dict(good, accessToken='PRIVATE_TOKEN')):
            with self.assertRaises(PROBE.Rejected):
                PROBE.allowed_request('/login', bad)

    def test_empty_success_and_boolean_alias_are_rejected(self):
        for value in ({'isSuccess': False, 'errorMsg': 'PRIVATE_SERVER_MESSAGE'},
                {'isSuccess': 1, 'errorMsg': ''}, {'isSuccess': True, 'errorMsg': 'bad'}):
            with self.assertRaisesRegex(PROBE.Rejected, '^ReaderRejected$'):
                PROBE.successful((200, value))

    def test_body_output_is_hashes_and_finite_flags_only(self):
        private = 'To Mrs. Saville, England. ' + 'BODY_MUST_NOT_BE_EXPORTED ' * 100
        facts = PROBE.content_facts(private)
        self.assertEqual({'characters', 'sha256', 'replacementCharacters',
            'firstLetterMarkerPresent', 'noScriptTag', 'isNotResourceUrl'}, set(facts))
        self.assertNotIn('BODY_MUST_NOT_BE_EXPORTED', json.dumps(facts))
        self.assertEqual(64, len(facts['sha256']))
        self.assertTrue(facts['firstLetterMarkerPresent'])

    def test_missing_and_wrong_type_body_are_rejected(self):
        for value in (None, {}, [], '', 'https://example.org/chapter'):
            with self.assertRaises(PROBE.Rejected):
                PROBE.content_facts(value)

    def test_bad_encoding_and_script_have_explicit_facts(self):
        facts = PROBE.content_facts('To Mrs. Saville, England. ' + 'a' * 1000 + '\ufffd<script>')
        self.assertEqual(1, facts['replacementCharacters'])
        self.assertIs(False, facts['noScriptTag'])

    def test_wrapper_keeps_resource_tls_and_private_network_guards(self):
        wrapper = (ROOT / 'scripts/probe-gutenberg-native-image.sh').read_text(encoding='utf-8')
        for guard in ('--memory=2g --memory-swap=2g --pids-limit=256 --cpus=2',
                '--security-opt no-new-privileges:true --cap-drop ALL',
                'READER_BROWSER_ALLOW_PRIVATE_NETWORKS=false', 'READER_APP_SECURE=true',
                'node scripts/release-native-artifacts.mjs check',
                'node scripts/release-native-artifacts.mjs loaded', 'sudo chown 10001:10001',
                'publishedPorts: 0', 'docker rm -f "$container_id"'):
            self.assertIn(guard, wrapper)
        self.assertNotIn('ignoreHTTPSErrors', wrapper)
        self.assertNotIn('--privileged', wrapper)
        self.assertNotIn('docker logs >', wrapper)

    def test_workflow_is_hosted_readonly_and_retains_failure_reports(self):
        workflow = (ROOT / '.github/workflows/browser-image.yml').read_text(encoding='utf-8')
        workflow = workflow.split('\n  public-metadata:\n', 1)[1].split('\n  compare-public-webview:', 1)[0]
        for guard in ('ubuntu-24.04-arm', 'ubuntu-24.04', 'contents: read', 'actions: read',
                'if: always()', 'digest-mismatch: error', 'timeout-minutes: 25'):
            self.assertIn(guard, workflow)
        for forbidden in ('self-hosted', 'secrets.', 'docker push', 'deploy', 'ssh '):
            self.assertNotIn(forbidden, workflow)

if __name__ == '__main__':
    unittest.main()
