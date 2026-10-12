"""Reject an incomplete actual-network-book UI journey; no content is read."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import zlib


def helper(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def png_visual_facts(raw):
    """Bounded 8-bit RGB/RGBA screenshots, no Pillow/system package dependency."""
    assert raw[:8] == b'\x89PNG\r\n\x1a\n' and len(raw) <= 2 * 1024 * 1024
    offset, data, header, ended = 8, bytearray(), None, False
    while offset + 12 <= len(raw):
        size = struct.unpack('>I', raw[offset:offset + 4])[0]
        kind = raw[offset + 4:offset + 8]
        assert offset + size + 12 <= len(raw)
        chunk = raw[offset + 8:offset + 8 + size]
        assert zlib.crc32(kind + chunk) & 0xffffffff == struct.unpack('>I', raw[offset + size + 8:offset + size + 12])[0]
        if kind == b'IHDR':
            assert header is None and size == 13
            header = struct.unpack('>IIBBBBB', chunk)
        elif kind == b'IDAT':
            data.extend(chunk)
        offset += size + 12
        if kind == b'IEND':
            assert size == 0 and offset == len(raw)
            ended = True
            break
    assert ended and header is not None and offset == len(raw)
    width, height, bits, color, compression, filtering, interlace = header
    assert 1 <= width <= 1280 and 1 <= height <= 900
    assert bits == 8 and color in (2, 6) and (compression, filtering, interlace) == (0, 0, 0)
    channels = 3 if color == 2 else 4
    stride = width * channels
    expected = height * (stride + 1)
    inflater = zlib.decompressobj()
    pixels = inflater.decompress(data, expected + 1)
    assert inflater.eof and not inflater.unconsumed_tail and not inflater.unused_data and len(pixels) == expected
    prior, counts = bytearray(stride), {}
    for y in range(height):
        start = y * (stride + 1)
        method = pixels[start]
        assert 0 <= method <= 4
        row = bytearray(pixels[start + 1:start + 1 + stride])
        for x in range(stride):
            left = row[x - channels] if x >= channels else 0
            above = prior[x]
            upper_left = prior[x - channels] if x >= channels else 0
            if method == 1:
                prediction = left
            elif method == 2:
                prediction = above
            elif method == 3:
                prediction = (left + above) // 2
            elif method == 4:
                p = left + above - upper_left
                distances = (abs(p - left), abs(p - above), abs(p - upper_left))
                prediction = (left, above, upper_left)[distances.index(min(distances))]
            else:
                prediction = 0
            row[x] = (row[x] + prediction) & 255
        for x in range(0, stride, channels):
            pixel = bytes(row[x:x + channels])
            counts[pixel] = counts.get(pixel, 0) + 1
        prior = row
    return {'width': width, 'height': height, 'distinctPixelValues': len(counts),
            'nonDominantPixels': width * height - max(counts.values())}


def verify_report(report, revision, expected_jar_sha256=None):
    probe = helper('probe-gutenberg-reader')
    expected_jar = probe.validated_jar_sha256(probe.EXPECTED_JAR if expected_jar_sha256 is None else expected_jar_sha256)
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
    assert report['jarSha256'] == expected_jar
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
                            'matchesActualApiAfterWhitespaceNormalization', 'postAnimationFramesObserved',
                            'computedOpacityAndFiniteAnimationsSettled'}
        assert 1000 <= row['characters'] <= 128 * 1024 and row['replacementCharacters'] == 0
        assert row['matchesActualApiAfterWhitespaceNormalization'] is True
        assert row['postAnimationFramesObserved'] is True
        assert row['computedOpacityAndFiniteAnimationsSettled'] is True
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
        assert set(row) == {'sha256', 'bytes', 'persisted', 'computedOpacityAndFiniteAnimationsSettled'}
        assert row['computedOpacityAndFiniteAnimationsSettled'] is True
        assert row['persisted'] is (name != 'toolbar')
        assert 1000 <= row['bytes'] <= 2 * 1024 * 1024
        assert len(row['sha256']) == 64 and int(row['sha256'], 16) >= 0
    assert 'failureClass' not in report
    return counts


def verify_running_identity(identity, revision, expected_jar_sha256):
    expected_jar = helper('probe-gutenberg-reader').validated_jar_sha256(expected_jar_sha256)
    assert identity == {'revision': revision, 'jarSha256': expected_jar, 'network': 'bridge', 'publishedPorts': 0}
    assert type(identity['publishedPorts']) is int


def verify_directory(directory, revision, expected_jar_sha256=None):
    probe = helper('probe-gutenberg-reader')
    expected_jar = probe.validated_jar_sha256(probe.EXPECTED_JAR if expected_jar_sha256 is None else expected_jar_sha256)
    read = lambda name: json.loads((directory / name).read_text(encoding='utf-8'))
    report = read('GUTENBERG_UI_REPORT.json')
    counts = verify_report(report, revision, expected_jar)
    for name in ('UI_BROWSER_RESOURCE_BUDGET.json', 'UI_SERVER_RESOURCE_BUDGET.json'):
        helper('report-browser-cgroup').verify_report(read(name), require_no_swap=True)
    assert read('UI_CLIENT_CLEANUP.json') == {'ownedTestClientRemoved': True,
        'testClientNotDeploymentDependency': True, 'clientOutsideReaderServerBudget': True}
    assert read('CLEANUP.json') == {'ownedContainerRemoved': True, 'probeExit': 0}
    verify_running_identity(read('RUNNING_JAR_IDENTITY.json'), revision, expected_jar)
    assert not (directory / 'ui-toolbar.png').exists()
    for name in ('login', 'logout'):
        raw = (directory / ('ui-' + name + '.png')).read_bytes()
        assert len(raw) == report['screenshots'][name]['bytes']
        assert hashlib.sha256(raw).hexdigest() == report['screenshots'][name]['sha256']
        pixels = png_visual_facts(raw)
        assert 500 <= pixels['width'] <= 1280 and 500 <= pixels['height'] <= 900
        assert pixels['distinctPixelValues'] >= 16 and pixels['nonDominantPixels'] >= 1000
    return {'accepted': True, 'expectedImageRevision': revision, 'jarSha256': report['jarSha256'],
            'paintedIndices': [row['index'] for row in report['paintedChapters']], 'actualUiApiResponseCounts': counts,
            'combinedTwoGiBBudgetProven': False, 'productionChanged': False, 'fullGoalComplete': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('directory', type=Path)
    parser.add_argument('revision')
    parser.add_argument('--expected-jar-sha256')
    args = parser.parse_args()
    print(json.dumps(verify_directory(args.directory, args.revision, args.expected_jar_sha256), sort_keys=True))
