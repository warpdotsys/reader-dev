"""Reject an incomplete actual-network-book UI journey; no content is read."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct


def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_report(report, revision):
    expected_keys = set('schemaVersion scope expectedImageRevision jarSha256 clientRuntimeUid '
        'testClientNotDeploymentDependency clientOutsideReaderServerBudget combinedTwoGiBBudgetProven '
        'realCredentialsImported privateLocalBooksRead responseMocked siteSearchRequested tlsErrorsIgnored '
        'rawBodyCookieOrErrorPersisted bodyScreenshotsExported originalOrRemoteParityProven passed stage '
        'apiObservations paintedChapters screenshots defaultUiReleaseIdentityMatches uiRegistrationSucceeded '
        'uiLoginSucceeded generatedNamespaceVerified sourceSeededViaApiNotUi sourceRulesRetained addedViaUi '
        'generatedSessionRevoked testBrowserClosed elapsedSeconds blockedClientRequests pageErrorCount '
        'apiObservationErrorCount budgetAccepted toc'.split())
    assert set(report) == expected_keys
    assert report['schemaVersion'] == 1 and report['expectedImageRevision'] == revision
    assert report['jarSha256'] == helper('probe-gutenberg-reader').EXPECTED_JAR
    assert report['clientRuntimeUid'] == 10001 and report['stage'] == 'complete'
    for key in ('passed', 'budgetAccepted', 'testClientNotDeploymentDependency', 'clientOutsideReaderServerBudget',
                'defaultUiReleaseIdentityMatches', 'uiRegistrationSucceeded', 'uiLoginSucceeded',
                'generatedNamespaceVerified', 'sourceSeededViaApiNotUi', 'sourceRulesRetained',
                'addedViaUi', 'generatedSessionRevoked', 'testBrowserClosed'):
        assert report[key] is True, key
    for key in ('combinedTwoGiBBudgetProven', 'realCredentialsImported', 'responseMocked', 'siteSearchRequested',
                'tlsErrorsIgnored', 'rawBodyCookieOrErrorPersisted', 'bodyScreenshotsExported', 'originalOrRemoteParityProven'):
        assert report[key] is False, key
    for key in ('blockedClientRequests', 'pageErrorCount', 'apiObservationErrorCount', 'privateLocalBooksRead'):
        assert type(report[key]) is int and report[key] == 0, key
    assert 0 < report['elapsedSeconds'] <= 210
    rows = report['paintedChapters']
    assert len(rows) == 4 and [row['index'] for row in rows] == [0, 1, 1, 0]
    for row in rows:
        assert set(row) == {'index', 'characters', 'sha256', 'replacementCharacters',
                            'matchesActualApiAfterWhitespaceNormalization', 'postAnimationFramesObserved'}
        assert 1000 <= row['characters'] <= 128 * 1024 and row['replacementCharacters'] == 0
        assert row['matchesActualApiAfterWhitespaceNormalization'] is True
        assert row['postAnimationFramesObserved'] is True
        assert len(row['sha256']) == 64 and int(row['sha256'], 16) >= 0
    assert rows[0]['sha256'] == rows[3]['sha256']
    assert rows[1]['sha256'] == rows[2]['sha256'] != rows[0]['sha256']
    toc = report['toc']
    assert set(toc) == {'chapterCount', 'webViewOptionsRetained', 'baseUrlsMatchFixedBook',
        'chaptersRemainInFixedBook', 'fragmentsMatchExpected', 'chapterTitlesPresent', 'indicesSequential',
        'noVolumeEntries', 'relativeChapterUrls', 'canonicalTocSha256', 'fixedBookChapterUrlsValid'}
    assert toc['chapterCount'] == 28 and toc['relativeChapterUrls'] == 28
    for key in ('webViewOptionsRetained', 'baseUrlsMatchFixedBook', 'chaptersRemainInFixedBook',
                'fragmentsMatchExpected', 'chapterTitlesPresent', 'indicesSequential',
                'noVolumeEntries', 'fixedBookChapterUrlsValid'):
        assert toc[key] is True
    assert len(toc['canonicalTocSha256']) == 64 and int(toc['canonicalTocSha256'], 16) >= 0
    observations = report['apiObservations']
    assert 1 <= len(observations) <= 64
    counts = {}
    for row in observations:
        assert set(row) == {'endpoint', 'status', 'isSuccess', 'errorMsgEmpty', 'fields'}
        assert row['status'] == 200 and row['isSuccess'] is True and row['errorMsgEmpty'] is True
        assert row['fields'] == ['data', 'errorMsg', 'isSuccess']
        assert row['endpoint'] in helper('probe-gutenberg-ui').BUSINESS
        counts[row['endpoint']] = counts.get(row['endpoint'], 0) + 1
    for endpoint, minimum in {'login': 2, 'logout': 2, 'getBookInfo': 1, 'getChapterList': 1,
                              'getBookContent': 2, 'saveBook': 1, 'saveBookProgress': 1}.items():
        assert counts.get(endpoint, 0) >= minimum, endpoint
    assert set(report['screenshots']) == {'login', 'toolbar', 'logout'}
    for name, row in report['screenshots'].items():
        assert set(row) == {'sha256', 'bytes', 'persisted'}
        assert row['persisted'] is (name != 'toolbar')
        assert 1000 <= row['bytes'] <= 2 * 1024 * 1024
        assert len(row['sha256']) == 64 and int(row['sha256'], 16) >= 0
    assert 'failureClass' not in report
    return counts


def verify_directory(directory, revision):
    read = lambda name: json.loads((directory / name).read_text(encoding='utf-8'))
    report = read('GUTENBERG_UI_REPORT.json')
    counts = verify_report(report, revision)
    for name in ('UI_BROWSER_RESOURCE_BUDGET.json', 'UI_SERVER_RESOURCE_BUDGET.json'):
        helper('report-browser-cgroup').verify_report(read(name), require_no_swap=True)
    assert read('UI_CLIENT_CLEANUP.json') == {'ownedTestClientRemoved': True,
        'testClientNotDeploymentDependency': True, 'clientOutsideReaderServerBudget': True}
    assert read('CLEANUP.json') == {'ownedContainerRemoved': True, 'probeExit': 0}
    assert read('RUNNING_JAR_IDENTITY.json')['revision'] == revision
    assert not (directory / 'ui-toolbar.png').exists()
    for name in ('login', 'logout'):
        raw = (directory / ('ui-' + name + '.png')).read_bytes()
        assert len(raw) == report['screenshots'][name]['bytes']
        assert hashlib.sha256(raw).hexdigest() == report['screenshots'][name]['sha256']
        assert raw[:8] == b'\x89PNG\r\n\x1a\n'
        width, height = struct.unpack('>II', raw[16:24])
        assert 500 <= width <= 1280 and 500 <= height <= 900
    return {'accepted': True, 'expectedImageRevision': revision, 'jarSha256': report['jarSha256'],
            'paintedIndices': [row['index'] for row in report['paintedChapters']], 'actualUiApiResponseCounts': counts,
            'combinedTwoGiBBudgetProven': False, 'productionChanged': False, 'fullGoalComplete': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('directory', type=Path)
    parser.add_argument('revision')
    args = parser.parse_args()
    print(json.dumps(verify_directory(args.directory, args.revision), sort_keys=True))
