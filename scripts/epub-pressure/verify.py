"""Fail closed on incomplete, wrong-identity, skipped or out-of-budget pressure evidence."""
import argparse
import json
import math
from pathlib import Path
import re
import hashlib

MIB = 1024 * 1024
SCENES = {'text805', 'text806', 'text806Refresh', 'html806', 'raw806', 'raw805', 'raw805Refresh', 'text805Return'}
FLAGS = ('wrongPasswordContractAndUiAccepted', 'existingGeneratedAccountLoginAccepted',
         'rememberedTokenStoredInBothAccepted', 'importPreviewViaUiSucceeded', 'importSaveViaUiSucceeded',
         'importDialogReopensEmptyAccepted', 'reopenedImportSurvivesOldAutoCloseAccepted',
         'logoutThreeFieldContractAccepted', 'logoutBrowserTokensClearedAccepted',
         'logoutReloadRemainsAnonymousAccepted', 'revokedTokenRejectedByRealApi',
         'freshContextReloginAccepted', 'existingShelfNamespaceConfirmedAccepted',
         'unrememberedTokenSessionOnlyAccepted', 'unrememberedRefreshKeepsSessionAccepted',
         'newTabDoesNotAdoptCookieOnlyIdentityAccepted', 'defaultPackagedUiStaticAccepted',
         'exactPackagedEntrypointExecuted', 'generatedRegistrationAccepted',
         'generatedRegistrationProcessStopped', 'ownedJavaStopped', 'passiveSamplerStopped',
         'readerEightScenesAccepted', 'passed')
SOURCE_NAMES = {'fixture.py', 'generated_account.py', 'main.py', 'ui.py', 'pressure.py',
                'storage_budget.py', 'worker_network.py', 'file_network.py', 'epub-frame-readiness.js',
                'verify-reader-default-ui.py'}


class PressureRejected(ValueError):
    pass


def require(value):
    if not value:
        raise PressureRejected('Generated large-EPUB evidence rejected')


def number(value, lower, upper):
    return type(value) in (int, float) and math.isfinite(value) and lower < value <= upper


def verify(value, revision, jar_sha, architecture, expected_source_hashes=None):
    require(type(value) is dict and re.fullmatch('[0-9a-f]{40}', revision or '') is not None)
    require(re.fullmatch('[0-9a-f]{64}', jar_sha or '') is not None and architecture in ('amd64', 'arm64'))
    require(value.get('schemaVersion') == 1 and type(value.get('schemaVersion')) is int)
    require(value.get('scope') == 'three-fresh-default-image-generated-large-epub-rounds')
    require(value.get('accepted') is True and value.get('revision') == revision and value.get('jarSha256') == jar_sha)
    require(value.get('architecture') == architecture and re.fullmatch('sha256:[0-9a-f]{64}', value.get('imageId', '')) is not None)
    for key in ('privateBookInputRead', 'realAccountAuthenticationTested', 'originalJarAccountCompatibilityTested',
                'hostGlobalCachesChanged', 'hostPageCacheColdProven', 'productionOrLongRunProven'):
        require(value.get(key) is False)
    rounds = value.get('rounds')
    require(type(rounds) is list and len(rounds) == 3)
    observed_tokens, source_hashes, maximum_peak = set(), None, 0
    for index, row in enumerate(rounds):
        require(type(row) is dict and type(row.get('round')) is int and row['round'] == index + 1)
        require(row.get('accepted') is True and row.get('freshProcessAndStorage') is True and row.get('parentEmptyAndStopped') is True)
        require(type(row.get('executionReturnCode')) is int and row['executionReturnCode'] == 0)
        require(row.get('fallbackOwnedContainersRemoved') == 0 and type(row.get('fallbackOwnedContainersRemoved')) is int)
        p, observation = row.get('provenance'), row.get('observation')
        require(type(p) is dict and type(observation) is dict)
        for item in (p, observation):
            require(item.get('revision') == revision and item.get('jarSha256') == jar_sha and item.get('architecture') == architecture)
            require(item.get('privateBookInputRead') is False and item.get('realCredentialsImported') is False)
        require(p.get('imageId') == value['imageId'] and p.get('generatedLargeEpubAccepted') is True)
        require(p.get('controllerIncludedInAggregateBudget') is True and p.get('generatedDataCleared') is True and p.get('ownedContainerRemoved') is True)
        require(p.get('freshProcessAndStorage') is True and p.get('hostPageCacheColdProven') is False)
        require(p.get('containerOomKilled') is False and type(p.get('containerExitCode')) is int and p['containerExitCode'] == 0)
        require(type(p.get('workerExitCode')) is int and p['workerExitCode'] == 0)
        token = p.get('token', '')
        require(re.fullmatch('readergen[0-9a-f]{12}', token) is not None and token not in observed_tokens)
        observed_tokens.add(token)
        hashes = p.get('sourceHashes')
        require(type(hashes) is dict and set(hashes) == SOURCE_NAMES and all(type(v) is str and re.fullmatch('[0-9a-f]{64}', v) for v in hashes.values()))
        if expected_source_hashes is not None:
            require(hashes == expected_source_hashes)
        if source_hashes is None:
            source_hashes = hashes
        require(hashes == source_hashes)
        b = p.get('aggregateBudget')
        require(type(b) is dict and type(b.get('cpuQuota')) is int and type(b.get('cpuPeriod')) is int and b['cpuPeriod'] > 0)
        require(b['cpuQuota'] == b['cpuPeriod'] * 2 and b.get('memoryMaxBytes') == 2 * 1024 * MIB and b.get('memoryHighBytes') == 1536 * MIB)
        require(b.get('swapMaxBytes') == 0 and b.get('pidsMax') == 512 and number(b.get('memoryPeakBytes'), 0, 2 * 1024 * MIB))
        require(type(b.get('memoryEvents')) is dict and type(b.get('pidsEvents')) is dict)
        require(all(type(b['memoryEvents'].get(k)) is int and b['memoryEvents'][k] == 0 for k in ('max', 'oom', 'oom_kill')))
        require(type(b['pidsEvents'].get('max')) is int and b['pidsEvents']['max'] == 0)
        maximum_peak = max(maximum_peak, b['memoryPeakBytes'])
        require(observation.get('scope') == 'default-image-generated-large-epub-pressure' and all(observation.get(k) is True for k in FLAGS))
        require(observation.get('defaultJvmUnchanged') is True and observation.get('downloadDeadlineSeconds') == 60)
        require(observation.get('workerDeadlineSeconds') == 15 and observation.get('uiObservationSeconds') == 30)
        require(observation.get('fixtureGeneratedInsideAggregateBudget') is True and observation.get('browserClientIncludedInAggregateBudget') is True)
        require(observation.get('webUiOrRendererOverriddenByTest') is False and observation.get('workerInstrumentationInstalled') is False)
        require(observation.get('testOnlyPageHtmlInstrumented') is False and observation.get('contextCloseFailureCode') is None and observation.get('passiveSamplerErrors') == [])
        require(number(observation.get('seconds'), 0, 220))
        fixture = observation.get('fixture')
        require(type(fixture) is dict and fixture.get('version') == 'reader-generated-epub-pressure-v1' and fixture.get('profile') == 'pressure')
        require(fixture.get('entries') == 1690 and fixture.get('chapters') == 850 and number(fixture.get('bytes'), 36 * MIB - 1, 40 * MIB))
        require(number(fixture.get('expandedBytes'), 88 * MIB - 1, 92 * MIB) and re.fullmatch('[0-9a-f]{64}', fixture.get('sha256', '')) is not None)
        require(fixture.get('generatedFromPublicConstantsOnly') is True and fixture.get('originalOrPrivateInputRead') is False)
        cases = observation.get('cases')
        require(type(cases) is dict and set(cases) == SCENES)
        for name, case in cases.items():
            require(type(case) is dict and number(case.get('characters'), 1000, 4 * MIB) and number(case.get('paragraphs'), 20, 8192))
            require(case.get('replacementCharacters') == 0 and case.get('scripts') == 0 and case.get('resourceUrlOnly') is False)
            if name.startswith('raw'):
                require(case.get('frameBusy') is False and case.get('fontStatus') == 'loaded' and case.get('headingMatchesRequestedChapterTitle') is True)
                require(case.get('chapterQueryIndex') == (806 if name == 'raw806' else 805))
        comparison = observation.get('rawRefreshComparison')
        require(type(comparison) is dict and all(comparison.get(k) is True for k in ('innerTextEqual', 'textContentEqual', 'nonWhitespaceInnerTextEqual', 'headingEqual')))
        require(comparison.get('beforeHeadingEqualsPreviousRawChapter') is False)
        for key in ('before', 'after'):
            require(type(comparison.get(key)) is dict and comparison[key].get('chapterQueryIndex') == 805 and comparison[key].get('headingMatchesRequestedChapterTitle') is True)
        disk = observation.get('generatedDiskStorageBudget')
        require(type(disk) is dict and disk.get('monitorStopped') is True and disk.get('metadataReadErrors') == [] and disk.get('budgetExceeded') is False)
        require(disk.get('budgetBytes') == 512 * MIB and number(disk.get('maximumBytesObserved'), 0, 512 * MIB))
    return dict(accepted=True, scope=value['scope'], revision=revision, jarSha256=jar_sha,
                architecture=architecture, rounds=3, readingScenes=24, maximumAggregatePeakBytes=maximum_peak,
                hostPageCacheColdProven=False, privateBookOrRealAccountTested=False, productionOrLongRunProven=False)


def strict_object(pairs):
    value = {}
    for key, item in pairs:
        require(key not in value)
        value[key] = item
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--revision', required=True)
    parser.add_argument('--jar-sha', required=True)
    parser.add_argument('--architecture', choices=('amd64', 'arm64'), required=True)
    args = parser.parse_args()
    try:
        require(args.report.is_file() and not args.report.is_symlink() and 0 < args.report.stat().st_size <= 4 * MIB)
        value = json.loads(args.report.read_bytes(), object_pairs_hook=strict_object,
                           parse_constant=lambda _: (_ for _ in ()).throw(PressureRejected('Non-finite value')))
        source = Path(__file__).resolve().parent
        expected = {}
        for name in SOURCE_NAMES:
            path = source.parent / name if name == 'verify-reader-default-ui.py' else source / name
            require(path.is_file() and not path.is_symlink() and path.stat().st_size < 65536)
            expected[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        result = verify(value, args.revision, args.jar_sha, args.architecture, expected)
    except (OSError, ValueError, TypeError, KeyError):
        parser.exit(1, 'Generated large-EPUB pressure evidence rejected; no body, credential or private value echoed.\n')
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    main()
