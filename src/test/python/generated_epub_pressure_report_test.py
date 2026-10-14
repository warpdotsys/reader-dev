"""Synthetic receipt doubles exercise fail-closed parsing, never prove actual UI runs."""
import copy
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).parents[3]
SPEC = importlib.util.spec_from_file_location('pressure_report_guard', ROOT / 'scripts/epub-pressure/verify.py')
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)
REVISION, JAR = 'a' * 40, 'b' * 64
HASHES = dict((name, 'c' * 64) for name in GUARD.SOURCE_NAMES)


def receipt_double():
    fixture = dict(version='reader-generated-epub-pressure-v1', profile='pressure', entries=1690,
                   chapters=850, bytes=38626835, expandedBytes=93511402, sha256='d' * 64,
                   generatedFromPublicConstantsOnly=True, originalOrPrivateInputRead=False)
    cases = {name: dict(characters=6000, paragraphs=64, replacementCharacters=0, scripts=0,
                       resourceUrlOnly=False, frameBusy=False, fontStatus='loaded',
                       headingMatchesRequestedChapterTitle=True, chapterQueryIndex=806 if name == 'raw806' else 805)
             for name in GUARD.SCENES}
    observation = dict(schemaVersion=1, scope='default-image-generated-large-epub-pressure',
                       revision=REVISION, jarSha256=JAR, architecture='amd64',
                       privateBookInputRead=False, realCredentialsImported=False, defaultJvmUnchanged=True,
                       downloadDeadlineSeconds=60, workerDeadlineSeconds=15, uiObservationSeconds=30,
                       fixtureGeneratedInsideAggregateBudget=True, browserClientIncludedInAggregateBudget=True,
                       webUiOrRendererOverriddenByTest=False, workerInstrumentationInstalled=False,
                       testOnlyPageHtmlInstrumented=False, contextCloseFailureCode=None, passiveSamplerErrors=[],
                       seconds=45.5, fixture=fixture, cases=cases,
                       rawRefreshComparison=dict(innerTextEqual=True, textContentEqual=True,
                            nonWhitespaceInnerTextEqual=True, headingEqual=True,
                            beforeHeadingEqualsPreviousRawChapter=False,
                            before=dict(chapterQueryIndex=805, headingMatchesRequestedChapterTitle=True),
                            after=dict(chapterQueryIndex=805, headingMatchesRequestedChapterTitle=True)),
                       generatedDiskStorageBudget=dict(monitorStopped=True, metadataReadErrors=[],
                            budgetExceeded=False, budgetBytes=512 * GUARD.MIB, maximumBytesObserved=180 * GUARD.MIB))
    observation.update((name, True) for name in GUARD.FLAGS)
    rounds = []
    for i in range(3):
        p = dict(token='readergen' + f'{i:012x}', imageId='sha256:' + 'e' * 64, revision=REVISION,
                 jarSha256=JAR, architecture='amd64', privateBookInputRead=False, realCredentialsImported=False,
                 generatedLargeEpubAccepted=True, controllerIncludedInAggregateBudget=True,
                 generatedDataCleared=True, ownedContainerRemoved=True, freshProcessAndStorage=True,
                 hostPageCacheColdProven=False, containerOomKilled=False, containerExitCode=0, workerExitCode=0,
                 sourceHashes=HASHES, aggregateBudget=dict(cpuQuota=200000, cpuPeriod=100000,
                    memoryMaxBytes=2147483648, memoryHighBytes=1610612736, swapMaxBytes=0, pidsMax=512,
                    memoryPeakBytes=1500 * GUARD.MIB, memoryEvents=dict(max=0, oom=0, oom_kill=0), pidsEvents=dict(max=0)))
        rounds.append(dict(round=i + 1, accepted=True, freshProcessAndStorage=True, parentEmptyAndStopped=True,
                           executionReturnCode=0, fallbackOwnedContainersRemoved=0,
                           provenance=p, observation=copy.deepcopy(observation)))
    return dict(schemaVersion=1, scope='three-fresh-default-image-generated-large-epub-rounds',
                accepted=True, revision=REVISION, jarSha256=JAR, architecture='amd64', imageId='sha256:' + 'e' * 64,
                privateBookInputRead=False, realAccountAuthenticationTested=False,
                originalJarAccountCompatibilityTested=False, hostGlobalCachesChanged=False,
                hostPageCacheColdProven=False, productionOrLongRunProven=False, rounds=rounds)


class GeneratedPressureReportGuardTest(unittest.TestCase):
    def setUp(self):
        self.value = receipt_double()

    def verify(self):
        return GUARD.verify(self.value, REVISION, JAR, 'amd64', HASHES)

    def test_receipt_double_only_shows_parser_acceptance_with_scope_limits(self):
        value = self.verify()
        self.assertEqual(3, value['rounds'])
        self.assertEqual(24, value['readingScenes'])
        self.assertFalse(value['privateBookOrRealAccountTested'])
        self.assertFalse(value['hostPageCacheColdProven'])
        self.assertFalse(value['productionOrLongRunProven'])

    def test_missing_or_extra_round_is_rejected(self):
        for count in (0, 1, 2, 4):
            self.value = receipt_double()
            self.value['rounds'] = (self.value['rounds'] * 2)[:count]
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_wrong_image_build_revision_jar_or_architecture_rejected(self):
        for field, value in (('revision', 'f' * 40), ('jarSha256', 'f' * 64), ('architecture', 'arm64')):
            self.value = receipt_double()
            self.value['rounds'][1]['observation'][field] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_failed_or_incomplete_scene_cannot_be_hidden_by_series_success_flag(self):
        self.value['rounds'][0]['observation']['cases'].pop('raw805Refresh')
        with self.assertRaises(GUARD.PressureRejected):
            self.verify()

    def test_client_outside_budget_or_changed_deadlines_rejected(self):
        for field, value in (('browserClientIncludedInAggregateBudget', False), ('defaultJvmUnchanged', False),
                             ('downloadDeadlineSeconds', 90), ('workerDeadlineSeconds', 30), ('uiObservationSeconds', 60)):
            self.value = receipt_double()
            self.value['rounds'][0]['observation'][field] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_wider_memory_cpu_swap_or_hard_limit_events_rejected(self):
        for field, value in (('memoryMaxBytes', 3 * 1024 * GUARD.MIB), ('memoryHighBytes', 2 * 1024 * GUARD.MIB),
                             ('cpuQuota', 300000), ('swapMaxBytes', GUARD.MIB), ('pidsMax', 1024)):
            self.value = receipt_double()
            self.value['rounds'][0]['provenance']['aggregateBudget'][field] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()
        for event in ('max', 'oom', 'oom_kill'):
            self.value = receipt_double()
            self.value['rounds'][0]['provenance']['aggregateBudget']['memoryEvents'][event] = 1
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_auth_or_private_data_claims_cannot_be_upgraded(self):
        for field in ('privateBookInputRead', 'realAccountAuthenticationTested', 'originalJarAccountCompatibilityTested',
                      'hostPageCacheColdProven', 'productionOrLongRunProven', 'hostGlobalCachesChanged'):
            self.value = receipt_double()
            self.value[field] = True
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_cleanup_fallback_and_reused_round_token_rejected(self):
        self.value['rounds'][1]['provenance']['token'] = self.value['rounds'][0]['provenance']['token']
        with self.assertRaises(GUARD.PressureRejected):
            self.verify()
        for field, value in (('ownedContainerRemoved', False), ('generatedDataCleared', False)):
            self.value = receipt_double()
            self.value['rounds'][0]['provenance'][field] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()
        self.value = receipt_double()
        self.value['rounds'][0]['fallbackOwnedContainersRemoved'] = 1
        with self.assertRaises(GUARD.PressureRejected):
            self.verify()

    def test_small_fixture_bad_text_or_wrong_previous_chapter_cannot_pass(self):
        for field, value in (('entries', 10), ('profile', 'small-unit-only'), ('bytes', 50000)):
            self.value = receipt_double()
            self.value['rounds'][0]['observation']['fixture'][field] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()
        self.value = receipt_double()
        self.value['rounds'][1]['observation']['cases']['raw805Refresh']['chapterQueryIndex'] = 806
        with self.assertRaises(GUARD.PressureRejected):
            self.verify()

    def test_changed_frozen_source_and_nonfinite_duration_rejected(self):
        self.value['rounds'][0]['provenance']['sourceHashes'] = dict(HASHES, **{'ui.py': 'f' * 64})
        with self.assertRaises(GUARD.PressureRejected):
            self.verify()
        for value in (0, -1, float('nan'), float('inf'), True, 221):
            self.value = receipt_double()
            self.value['rounds'][0]['observation']['seconds'] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_boolean_success_flags_require_exact_true(self):
        for value in (1, 'true', None, False):
            self.value = receipt_double()
            self.value['rounds'][0]['observation']['wrongPasswordContractAndUiAccepted'] = value
            with self.assertRaises(GUARD.PressureRejected):
                self.verify()

    def test_duplicate_json_fields_rejected(self):
        with self.assertRaises(GUARD.PressureRejected):
            GUARD.strict_object([('accepted', False), ('accepted', True)])


if __name__ == '__main__':
    unittest.main()
