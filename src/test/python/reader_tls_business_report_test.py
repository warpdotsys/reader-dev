"""Mutation guards over a recorded GENERATED fixture; not a fresh live acceptance."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).parents[3]
spec = importlib.util.spec_from_file_location('reader_business_guard', ROOT / 'scripts/verify-reader-tls-business.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
RECEIPT = ROOT / 'docs/evidence/reader-business-https-22f949cc-2026-10-10.json'
JAR = 'fcd28a8d974666f12de6671570f2435a9161789d29f20314c7e1f28bf8f2285f'
REVISION = '22f949ccb3befff4d66af793598889b5879b0c45'


class ReaderTlsBusinessReportTest(unittest.TestCase):
    def report(self):
        return json.loads(RECEIPT.read_bytes())

    def reject(self, mutate):
        report = self.report()
        mutate(report)
        with self.assertRaises((ValueError, SystemExit, KeyError)):
            guard.validate(report, JAR, REVISION)

    def first_target(self, report):
        return report['observation']['results'][0]['targetRequests'][0]

    def test_recorded_generated_receipt_digest(self):
        self.assertEqual(hashlib.sha256(RECEIPT.read_bytes()).hexdigest(),
                         '2427acdaaf7e48835bdb73a0f29689017c186a6f242f528c13c4379c0e4b3a28')

    def test_recorded_hosted_receipt_bytes_preserved_across_checkouts(self):
        expected = {
            'java':'22ac58ac1db010134419238b7021e904ff5e61767a0dea626863ff74eacf4b40',
            'ui-java':'5beb148d0e08822b776e0cb8e26c019fe5297f9dafa464ea74e7fa16784184af',
            'full':'266813238bcc19938f9642c27cb6095a5e18da6341855c893d635d3b6e2d0abb',
            'native':'5cdbdd5ca079512e92f78810d528f9a58ae0b54efd556f5892c590866b535524',
        }
        for kind,digest in expected.items():
            path=ROOT / ('docs/evidence/proxy-appdir-hosted-'+kind+'-22f949cc-2026-10-10.json')
            with self.subTest(kind=kind):
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),digest)

    def test_recorded_generated_receipt_is_finite_subset(self):
        result = guard.validate(self.report(), JAR, REVISION)
        self.assertEqual((result['actualTargetRequests'], result['actualConnectTunnels'],
                          result['parsedHeaderFields']), (12, 14, 178))
        self.assertFalse(result['originalJarThreeWayAccepted'])
        self.assertFalse(result['fullNativeImageLocallyAccepted'])
        self.assertFalse(result['realSiteLoginAccepted'])
        self.assertFalse(result['fullGoalComplete'])

    def test_other_source_rejected(self):
        self.reject(lambda r: r.update(sourceRevision='0'*40))

    def test_other_jar_rejected(self):
        self.reject(lambda r: r.update(actualJarSha256='0'*64))

    def test_other_architecture_rejected(self):
        self.reject(lambda r: r['finalResources'].update(architecture='aarch64'))

    def test_root_rejected(self):
        self.reject(lambda r: r['identity'].update(uid=0))

    def test_external_network_rejected(self):
        self.reject(lambda r: r['identity'].update(interfaces=['lo','eth0']))

    def test_readonly_browser_missing_rejected(self):
        self.reject(lambda r: r.update(browserBinaryBytesChanged=True))

    def test_private_ca_missing_rejected(self):
        self.reject(lambda r: r.update(fixtureOnlyPrivateDistributionTmpfsVerified=False))

    def test_no_cleanup_rejected(self):
        self.reject(lambda r: r.update(ownedContainersRemaining=1))

    def test_soft_pressure_preserved_not_falsely_rejected(self):
        report = self.report()
        self.assertEqual(report['finalResources']['memoryEvents']['high'], 2468)
        self.assertTrue(guard.validate(report, JAR, REVISION)['acceptedCurrentReaderHttpsBusinessSubset'])

    def test_oom_rejected(self):
        self.reject(lambda r: r['finalResources']['memoryEvents'].update(oom=1))

    def test_swap_rejected(self):
        self.reject(lambda r: r['finalResources'].update(swapMaxBytes=1073741824))

    def test_generated_auth_failure_rejected(self):
        self.reject(lambda r: r['observation']['authObservationsWithoutCredentialsOrTokens'][0].update(isSuccess=False))

    def test_auth_token_field_rejected(self):
        self.reject(lambda r: r['observation']['authObservationsWithoutCredentialsOrTokens'][0].update(token='GENERATED_ONLY'))

    def test_worker_substitute_rejected(self):
        self.reject(lambda r: r['observation']['results'][0].update(workerResult={'body':'GENERATED_ONLY'}))

    def test_source_roundtrip_missing_rejected(self):
        self.reject(lambda r: r['observation']['results'][0]['readerApiResult'].update(sourceRoundtripMatched=False))

    def test_source_proxy_mismatch_rejected(self):
        self.reject(lambda r: r['observation']['results'][0]['readerApiResult'].update(configuredProxy='http://127.0.0.1:1'))

    def test_jvm_socket_ownership_missing_rejected(self):
        self.reject(lambda r: r['observation']['results'][0]['tunnels'][0]['actualClientOwnership'].update(clientIsOwnedCurrentReaderJvm=False))

    def test_other_jvm_rejected(self):
        self.reject(lambda r: r['observation']['results'][0]['tunnels'][0]['actualClientOwnership'].update(javaPid=99999))

    def test_wrong_host_rejected(self):
        self.reject(lambda r: self.first_target(r)['headers'].append(['Host','other.invalid']))

    def test_cross_origin_authorization_rejected(self):
        self.reject(lambda r: r['observation']['results'][1]['targetRequests'][1]['headers'].append(['Authorization',guard.AUTH]))

    def test_utf8_body_changed_rejected(self):
        self.reject(lambda r: r['observation']['results'][2]['targetRequests'][0].update(bodyByteCount=40))

    def test_second_namespace_cookie_rejected(self):
        self.reject(lambda r: r['observation']['results'][4]['targetRequests'][0]['headers'].append(['Cookie',guard.COOKIE]))

    def test_second_namespace_sources_shared_rejected(self):
        self.reject(lambda r: r['observation']['results'][4]['readerApiResult'].update(bookSourcesWereEmptyForSecondNamespace=False))

    def test_search_default_changed_rejected(self):
        self.reject(lambda r: r['observation']['results'][0]['readerApiResult']['returnData']['data'][0].update(time=1))

    def test_negative_omitted_data_preserved(self):
        report = self.report()
        for row in report['observation']['results'][-2:]:
            self.assertNotIn('data', row['readerApiResult']['returnData'])
        self.assertTrue(guard.validate(report, JAR, REVISION)['acceptedCurrentReaderHttpsBusinessSubset'])

    def test_negative_added_fake_data_rejected(self):
        self.reject(lambda r: r['observation']['results'][-1]['readerApiResult']['returnData'].update(data=None))

    def test_invalid_cert_http_request_rejected(self):
        self.reject(lambda r: r['observation']['results'][-1].update(negativeHttpRequests=1))

    def test_wrong_cert_reason_rejected(self):
        self.reject(lambda r: r['observation']['results'][-1]['negativeTlsFailures'][0].update(reason='OTHER'))

    def test_unrelated_tls_connection_rejected(self):
        self.reject(lambda r: r['observation']['results'][-1]['negativeTlsFailures'][0].update(sourcePort=1))

    def test_claimed_whole_goal_rejected(self):
        self.reject(lambda r: r.update(fullGoalComplete=True))


if __name__ == '__main__':
    unittest.main()
