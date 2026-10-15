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

    def packaged_double(self):
        """Adapt generated observations for envelope unit tests; NEVER fresh execution evidence."""
        old=self.report()
        value={'schemaVersion':2,'mode':'reader-api','architecture':'amd64','revision':REVISION,
            'jarSha256':JAR,'workerSha256':guard.RECORDED_WORKER_SHA,
            'identity':old['identity'],'resources':old['finalResources'],'observation':old['observation'],
            'generatedOnly':True,'fixtureOnlyPrivateDistributionPolicy':True,
            'fixtureOnlyPrivateFontconfigTmpfs':True,'fixtureOnlyPrivateAppDataTmpfs':True,
            'fixtureOnlyPrivateStorageTmpfs':True,'httpsTested':True,'readerJarStarted':True,
            'realCredentialsImported':False,'privateBookBodyRead':False,'hostTrustStoreChanged':False,
            'ignoreHttpsErrorsUsed':False,'workerLaunchOverridden':False,'productionChanged':False,'fullGoalComplete':False}
        value['resources']['memoryHighBytes']='max'
        value['identity']['groups']=[10001]
        return value

    def reject_packaged(self,mutate):
        value=self.packaged_double()
        mutate(value)
        with self.assertRaises((ValueError,SystemExit,KeyError)):
            guard.validate_packaged(value,JAR,guard.RECORDED_WORKER_SHA,REVISION,'amd64')

    def test_packaged_generated_double_preserves_business_checks(self):
        value=self.packaged_double()
        result=guard.validate_packaged(value,JAR,guard.RECORDED_WORKER_SHA,REVISION,'amd64')
        self.assertEqual(result['actualConnectTunnels'],14)
        self.assertFalse(result['fullGoalComplete'])

    def test_packaged_mode_must_be_actual_reader_api(self):
        self.reject_packaged(lambda r:r.update(mode='worker'))

    def test_packaged_build_worker_identity_must_match(self):
        self.reject_packaged(lambda r:r.update(workerSha256='0'*64))

    def test_packaged_private_storage_is_required(self):
        self.reject_packaged(lambda r:r.update(fixtureOnlyPrivateStorageTmpfs=False))

    def test_packaged_foreign_group_is_rejected(self):
        self.reject_packaged(lambda r:r['identity'].update(groups=[10001,0]))

    def test_packaged_arm_claim_requires_actual_architecture(self):
        self.reject_packaged(lambda r:r.update(architecture='arm64'))

    def test_packaged_no_swap_still_required(self):
        self.reject_packaged(lambda r:r['resources'].update(swapCurrentBytes=1))

    def test_packaged_jvm_cleanup_still_required(self):
        self.reject_packaged(lambda r:r['observation']['readerCleanup'].update(javaExitedBeforeOuterCleanup=False))

    def test_packaged_live_socket_ownership_still_required(self):
        self.reject_packaged(lambda r:r['observation']['results'][0]['tunnels'][0]['actualClientOwnership'].update(clientIsOwnedCurrentReaderJvm=False))

    def test_packaged_cross_origin_header_check_not_relaxed(self):
        self.reject_packaged(lambda r:r['observation']['results'][1]['targetRequests'][1]['headers'].append(['Authorization',guard.AUTH]))

    def test_packaged_second_namespace_cookie_check_not_relaxed(self):
        self.reject_packaged(lambda r:r['observation']['results'][4]['targetRequests'][0]['headers'].append(['Cookie',guard.COOKIE]))

    def test_packaged_negative_tls_connection_check_not_relaxed(self):
        self.reject_packaged(lambda r:r['observation']['results'][-1]['negativeTlsFailures'][0].update(sourcePort=1))

    def certificate_hints_double(self):
        """Generated protocol mutations, not a fresh TLS/browser execution."""
        value=self.packaged_double()
        for row in value['observation']['results'][-2:]:
            row['readerApiResult']['returnData']['errorMsg']=(
                'java.lang.IllegalStateException: Camoufox HTTPS 证书域名不匹配 (SSL_ERROR_BAD_CERT_DOMAIN)' if row['case']=='wrong-host'
                else 'java.lang.IllegalStateException: Camoufox HTTPS 证书签发机构不受信任 (SEC_ERROR_UNKNOWN_ISSUER)')
        return value

    def check_certificate_hints(self,value):
        return guard.validate_packaged(value,JAR,guard.RECORDED_WORKER_SHA,REVISION,'amd64',True)

    def test_current_certificate_hint_gate_accepts_only_fixed_messages(self):
        self.assertTrue(self.check_certificate_hints(self.certificate_hints_double())['acceptedCertificateHintMessages'])

    def test_missing_reader_exception_prefix_is_not_the_actual_wire_contract(self):
        value=self.certificate_hints_double()
        for row in value['observation']['results'][-2:]:
            row['readerApiResult']['returnData']['errorMsg']=row['readerApiResult']['returnData']['errorMsg'].removeprefix('java.lang.IllegalStateException: ')
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)

    def test_arbitrary_exception_prefix_is_not_accepted(self):
        value=self.certificate_hints_double()
        value['observation']['results'][-1]['readerApiResult']['returnData']['errorMsg']='generated.OtherError: '+value['observation']['results'][-1]['readerApiResult']['returnData']['errorMsg']
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)

    def check_recorded_failed_hosted_job(self,arch,digest):
        """Read captured log payloads; GitHub masks headers, so NOT full wire acceptance."""
        path=ROOT / ('docs/evidence/certificate-hints-hosted-native-'+arch+'-failed-raw-fc0860f1-2026-10-10.json')
        self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),digest)
        report=json.loads(path.read_bytes())
        self.assertEqual(report['architecture'],arch)
        self.assertEqual(report['jarSha256'],'0882fcf42692723381fc20dd746f269df96035c0c2b1c336d0fed027782bdf56')
        self.assertEqual(report['workerSha256'],'d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5')
        self.assertEqual(report['revision'],'65d7cd8144f85cb1316d102d26ca6b0604cfe632')
        rows=report['observation']['results'][-2:]
        self.assertEqual([row['case'] for row in rows],['wrong-host','untrusted'])
        self.assertEqual([row['readerApiResult']['returnData'] for row in rows],[
            {'isSuccess':False,'errorMsg':'java.lang.IllegalStateException: Camoufox HTTPS 证书域名不匹配 (SSL_ERROR_BAD_CERT_DOMAIN)'},
            {'isSuccess':False,'errorMsg':'java.lang.IllegalStateException: Camoufox HTTPS 证书签发机构不受信任 (SEC_ERROR_UNKNOWN_ISSUER)'}])
        self.assertEqual([row['negativeHttpRequests'] for row in rows],[0,0])
        self.assertFalse(report['fullGoalComplete'])

    def test_recorded_actual_amd64_api_includes_reader_exception_prefix(self):
        self.check_recorded_failed_hosted_job('amd64','ac08ee86d873160963250ca8cb094ef1c9640cfc8ab7cecec983aac9dabbff9d')

    def test_recorded_actual_arm64_api_includes_reader_exception_prefix(self):
        self.check_recorded_failed_hosted_job('arm64','0abb147cb993777001aa6cba643a908a0326688025705055395d2e979e7dff8e')

    def test_recorded_unmasked_full_image_actual_reader_tls_accepts_the_complete_hint_contract(self):
        """Successful c158 artifact, unlike masked fc job logs; not a new browser run."""
        path=ROOT / 'docs/evidence/certificate-hints-hosted-full-raw-c1581a9e-2026-10-10.json'
        self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),
            'f060d3a95346256925a42130293a4c79c8f269164846f6a5265ecfb55bfddaff')
        result=guard.validate_packaged(json.loads(path.read_bytes()),
            '0882fcf42692723381fc20dd746f269df96035c0c2b1c336d0fed027782bdf56',
            'd8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5',
            '0c5f744cdd0d730b33cc3cf64b5d85c5826c2405','amd64',True)
        self.assertTrue(result['acceptedCertificateHintMessages'])
        self.assertEqual((result['generatedScenarios'],result['actualTargetRequests'],result['actualConnectTunnels'],
            result['parsedHeaderFields']),(8,12,14,178))
        self.assertFalse(result['fullGoalComplete'])

    def test_old_generic_errors_remain_historical_but_fail_new_hint_gate(self):
        value=self.packaged_double()
        self.assertTrue(guard.validate_packaged(value,JAR,guard.RECORDED_WORKER_SHA,REVISION,'amd64')['acceptedCurrentReaderHttpsBusinessSubset'])
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)

    def test_swapped_certificate_messages_fail_new_hint_gate(self):
        value=self.certificate_hints_double()
        rows=value['observation']['results'][-2:]
        rows[0]['readerApiResult']['returnData']['errorMsg'], rows[1]['readerApiResult']['returnData']['errorMsg']=(
            rows[1]['readerApiResult']['returnData']['errorMsg'],rows[0]['readerApiResult']['returnData']['errorMsg'])
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)

    def test_certificate_message_cannot_echo_a_url_or_private_text(self):
        value=self.certificate_hints_double()
        value['observation']['results'][-1]['readerApiResult']['returnData']['errorMsg']+=' https://generated.invalid/?token=PRIVATE'
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)

    def test_certificate_hint_gate_does_not_add_a_fake_data_field(self):
        value=self.certificate_hints_double()
        value['observation']['results'][-1]['readerApiResult']['returnData']['data']=None
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)

    def test_certificate_hint_does_not_replace_actual_http_rejection(self):
        value=self.certificate_hints_double()
        value['observation']['results'][-1]['negativeHttpRequests']=1
        with self.assertRaises(ValueError):
            self.check_certificate_hints(value)


if __name__ == '__main__':
    unittest.main()
