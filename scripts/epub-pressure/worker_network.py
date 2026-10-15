"""Passive exact same-origin Worker-asset events; never capture bodies, cookies, URLs or headers."""
import time
from urllib.parse import urlsplit

import zipfile
with zipfile.ZipFile('/app/reader.jar') as _archive:
    _workers = [item.filename for item in _archive.infolist() if item.filename.startswith('BOOT-INF/classes/web-vue3/static/epubArchive.worker-') and item.filename.endswith('.js')]
assert len(_workers) == 1
ASSET = '/' + _workers[0].removeprefix('BOOT-INF/classes/web-vue3/')
del _archive, _workers
MAX_EVENTS = 16

class WorkerNetworkObservation:
    def __init__(self, base, observation):
        parsed = urlsplit(base)
        assert parsed.scheme == 'http' and parsed.hostname == '127.0.0.1'
        self.origin = (parsed.scheme, parsed.netloc)
        self.start = time.monotonic()
        self.observation = observation
        self.events = []
        self.total = 0
        observation['passiveWorkerAssetEvents'] = self.events
        observation['passiveWorkerAssetObserverInstalled'] = True

    def matches(self, url):
        parsed = urlsplit(url)
        return (parsed.scheme, parsed.netloc) == self.origin and parsed.path == ASSET

    def record(self, kind, **fields):
        self.total += 1
        self.observation['passiveWorkerAssetEventCount'] = self.total
        if len(self.events) < MAX_EVENTS:
            self.events.append({'kind': kind, 'milliseconds': round((time.monotonic() - self.start) * 1000),
                'phase': self.observation.get('activePhase'), **fields})
        self.observation['passiveWorkerAssetEventsTruncated'] = self.total > MAX_EVENTS

    def request(self, request):
        if self.matches(request.url):
            self.record('request', get=request.method == 'GET', script=request.resource_type == 'script')

    def response(self, response):
        if self.matches(response.url):
            media = response.headers.get('content-type', '').split(';', 1)[0].lower()
            self.record('response', status=response.status, javascriptMedia=media in ('application/javascript', 'text/javascript'))

    def failed(self, request):
        if self.matches(request.url):
            self.record('requestfailed')

    def worker(self, worker):
        if self.matches(worker.url):
            self.record('worker-created')

    def attach(self, page):
        page.on('request', self.request)
        page.on('response', self.response)
        page.on('requestfailed', self.failed)
        page.on('worker', self.worker)
