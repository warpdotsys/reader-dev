"""Generated-only transient download fixtures; no network/browser/runtime installation."""
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import types
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('owned_pinned_installer', ROOT / 'deploy/reader-pro/install_camoufox_pinned.py')
installer = importlib.util.module_from_spec(spec)
try:
    import requests as request_types
except ModuleNotFoundError:
    # CI runs pure tests before installing the pinned runtime. These generated
    # exception/response doubles keep that suite dependency-free, never accept
    # a real download and are replaced by the actual package in runtime gates.
    class RequestException(Exception):
        def __init__(self, message='', response=None):
            super().__init__(message)
            self.response = response
    class ConnectionError(RequestException): pass
    class Timeout(RequestException): pass
    class HTTPError(RequestException): pass
    class SSLError(ConnectionError): pass
    class ReadTimeout(Timeout): pass
    class ConnectTimeout(ConnectionError, Timeout): pass
    class ChunkedEncodingError(RequestException): pass
    class GeneratedResponse: pass
    request_types = types.ModuleType('requests')
    request_types.exceptions = types.SimpleNamespace(RequestException=RequestException, ConnectionError=ConnectionError,
        Timeout=Timeout, HTTPError=HTTPError, SSLError=SSLError, ReadTimeout=ReadTimeout,
        ConnectTimeout=ConnectTimeout, ChunkedEncodingError=ChunkedEncodingError)
    request_types.Response = GeneratedResponse
    def forbidden_network(*_, **__): raise AssertionError('Generated fixture must not access the network')
    request_types.get = forbidden_network
with patch.dict(sys.modules, {'requests': request_types}):
    spec.loader.exec_module(installer)

DATA = b'generated-browser-archive'
SHA = hashlib.sha256(DATA).hexdigest()
URL = 'https://example.invalid/locked.zip'

class Response:
    def __init__(self, status=200, chunks=None):
        self.status = status
        self.chunks = [DATA] if chunks is None else chunks
        self.closed = False
    def __enter__(self): return self
    def __exit__(self, *_): self.closed = True
    def raise_for_status(self):
        if self.status != 200:
            response = installer.requests.Response()
            response.status_code = self.status
            raise installer.requests.exceptions.HTTPError('generated HTTP failure', response=response)
    def iter_content(self, chunk_size):
        for chunk in self.chunks:
            if isinstance(chunk, BaseException): raise chunk
            yield chunk

class PinnedInstallerTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='reader-generated-installer-')
        self.addCleanup(self.directory.cleanup)
        self.target = Path(self.directory.name) / 'generated.zip'

    def test_504_then_complete_same_asset_is_verified(self):
        first, second = Response(504), Response()
        with patch.object(installer.requests, 'get', side_effect=[first, second]) as get, patch.object(installer.time, 'sleep') as sleep:
            self.assertEqual(installer.download_verified(URL, self.target, len(DATA), SHA), (len(DATA), SHA))
        self.assertEqual(get.call_count, 2)
        self.assertTrue(all(call.args == (URL,) and call.kwargs == {'stream': True, 'timeout': (15, 120)} for call in get.call_args_list))
        sleep.assert_called_once_with(1)
        self.assertTrue(first.closed and second.closed)
        self.assertEqual(self.target.read_bytes(), DATA)

    def test_transient_statuses_stop_after_three_attempts(self):
        for status in (429, 500, 502, 503, 504):
            with self.subTest(status=status), patch.object(installer.requests, 'get', side_effect=[Response(status) for _ in range(3)]) as get, patch.object(installer.time, 'sleep') as sleep:
                with self.assertRaises(installer.requests.exceptions.HTTPError):
                    installer.download_verified(URL, self.target, len(DATA), SHA)
                self.assertEqual(get.call_count, 3)
                self.assertEqual([call.args[0] for call in sleep.call_args_list], [1, 2])
                self.assertFalse(self.target.exists())

    def test_404_and_certificate_failures_are_not_retried(self):
        for outcome in (Response(404), installer.requests.exceptions.SSLError('generated certificate rejection')):
            with self.subTest(outcome=type(outcome).__name__), patch.object(installer.requests, 'get', side_effect=[outcome]) as get, patch.object(installer.time, 'sleep') as sleep:
                with self.assertRaises(installer.requests.exceptions.RequestException):
                    installer.download_verified(URL, self.target, len(DATA), SHA)
                self.assertEqual(get.call_count, 1)
                sleep.assert_not_called()
                self.assertFalse(self.target.exists())

    def test_partial_timeout_is_removed_before_a_fresh_verified_attempt(self):
        first = Response(chunks=[DATA[:3], installer.requests.exceptions.ReadTimeout('generated partial')])
        with patch.object(installer.requests, 'get', side_effect=[first, Response()]) as get, patch.object(installer.time, 'sleep'):
            self.assertEqual(installer.download_verified(URL, self.target, len(DATA), SHA), (len(DATA), SHA))
        self.assertEqual(get.call_count, 2)
        self.assertEqual(self.target.read_bytes(), DATA)

    def test_connect_timeout_can_retry_but_preserves_final_error(self):
        final = installer.requests.exceptions.ConnectTimeout('generated final timeout')
        with patch.object(installer.requests, 'get', side_effect=[installer.requests.exceptions.ConnectTimeout('first'), installer.requests.exceptions.ConnectionError('second'), final]) as get, patch.object(installer.time, 'sleep'):
            with self.assertRaises(installer.requests.exceptions.ConnectTimeout) as observed:
                installer.download_verified(URL, self.target, len(DATA), SHA)
            self.assertIs(observed.exception, final)
            self.assertEqual(get.call_count, 3)

    def test_integrity_and_size_failures_never_retry_or_leave_bad_assets(self):
        for data, size, checksum in ((DATA, len(DATA), '0' * 64), (DATA, len(DATA) - 1, SHA), (DATA[:3], len(DATA), SHA)):
            with self.subTest(size=size), patch.object(installer.requests, 'get', return_value=Response(chunks=[data])) as get, patch.object(installer.time, 'sleep') as sleep:
                with self.assertRaises(RuntimeError): installer.download_verified(URL, self.target, size, checksum)
                self.assertEqual(get.call_count, 1)
                sleep.assert_not_called()
                self.assertFalse(self.target.exists())

    def test_existing_target_is_preserved_without_network_access(self):
        self.target.write_bytes(b'generated-retained-owner-data')
        with patch.object(installer.requests, 'get') as get:
            with self.assertRaisesRegex(RuntimeError, 'overwrite'): installer.download_verified(URL, self.target, len(DATA), SHA)
            get.assert_not_called()
        self.assertEqual(self.target.read_bytes(), b'generated-retained-owner-data')

if __name__ == '__main__': unittest.main()
