"""Capture one authorized fixed public page for isolated byte-identical replay.

Body stays in ignored local build data, not GitHub. No search, credentials,
Cookie, redirect or insecure TLS. The capture is NOT a Reader comparison.
"""
import argparse
import hashlib
import json
from pathlib import Path
import ssl
import urllib.request

URL = 'https://www.gutenberg.org/cache/epub/84/pg84-images.html'
MAX_BYTES = 2 * 1024 * 1024
ROOT = Path(__file__).resolve().parents[1]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, url):
        raise ValueError('UnexpectedFixedPageRedirect')


def capture(directory):
    directory = directory.resolve()
    if (not directory.is_relative_to(ROOT / 'build') or directory == ROOT / 'build' or directory.exists()):
        raise ValueError('ExpectedFreshIgnoredBuildDirectory')
    opener = urllib.request.build_opener(NoRedirect(), urllib.request.HTTPSHandler(context=ssl.create_default_context()))
    request = urllib.request.Request(URL, headers={'User-Agent': 'ReaderCompatibilityProbe/1.0'})
    with opener.open(request, timeout=25) as response:
        if response.status != 200 or response.geturl() != URL or response.headers.get_content_type() != 'text/html':
            raise ValueError('UnexpectedFixedPageResponse')
        body = response.read(MAX_BYTES + 1)
    if not 100000 <= len(body) <= MAX_BYTES:
        raise ValueError('UnexpectedFixedPageSize')
    text = body.decode('utf-8')
    if '\ufffd' in text or not all(marker in text for marker in ('Frankenstein', 'Shelley', 'id="letter1"', 'id="letter2"')):
        raise ValueError('MissingRealFixedPageMarkers')
    report = {'schemaVersion': 1, 'sourceUrl': URL, 'status': 200, 'bytes': len(body),
              'sha256': hashlib.sha256(body).hexdigest(), 'utf8DecodedStrictly': True,
              'tlsErrorsIgnored': False, 'redirectFollowed': False, 'credentialsOrCookieSent': False,
              'siteSearchRequested': False, 'captureOnlyNotReaderParity': True,
              'rawBodyUploadedToGitHub': False, 'privateLocalBooksRead': 0}
    directory.mkdir()
    target = directory / 'public-page.html'
    with target.open('xb') as stream:
        stream.write(body)
    target.chmod(0o444)
    with (directory / 'capture.json').open('x', encoding='utf-8') as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps(report, sort_keys=True))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('directory', type=Path)
    args = parser.parse_args()
    capture(args.directory)
