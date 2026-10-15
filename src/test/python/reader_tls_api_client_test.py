"""Generated unit guards, not a JVM/browser/real-site acceptance."""
import hashlib
import importlib.util
import io
from pathlib import Path
from unittest.mock import Mock,patch
import unittest

ROOT=Path(__file__).parents[3]
spec=importlib.util.spec_from_file_location('generated_tls_api_client',ROOT/'scripts/reader_tls_api_client.py')
client=importlib.util.module_from_spec(spec)
spec.loader.exec_module(client)


class ReaderTlsApiClientTest(unittest.TestCase):
    def api(self):
        return client.ReaderApi(None,Path('/GENERATED_ONLY.jar'),'a'*64,'b'*64,'152.0.4-beta.30')

    def test_decode_owned_ipv4_tcp_address(self):
        self.assertEqual(client.address('0100007F:ABCD'),('127.0.0.1',43981))

    def test_decode_jvm_ipv4_mapped_tcp6_address(self):
        self.assertEqual(client.address('0000000000000000FFFF00000100007F:ABCD'),('127.0.0.1',43981))

    def test_decode_ipv6_loopback(self):
        self.assertEqual(client.address('00000000000000000000000001000000:0001'),('::1',1))

    def test_invalid_tcp_address_rejected(self):
        with self.assertRaises(ValueError):
            client.address('INVALID:ABCD')

    def test_bad_jar_digest_rejected_before_process_start(self):
        with self.assertRaises(AssertionError):
            client.ReaderApi(None,'/GENERATED_ONLY.jar','INVALID','b'*64,'152.0.4-beta.30')

    def test_bad_worker_digest_rejected_before_process_start(self):
        with self.assertRaises(AssertionError):
            client.ReaderApi(None,'/GENERATED_ONLY.jar','a'*64,'INVALID','152.0.4-beta.30')

    def test_browser_version_argument_injection_rejected(self):
        with self.assertRaises(AssertionError):
            client.ReaderApi(None,'/GENERATED_ONLY.jar','a'*64,'b'*64,'152.0.4 --GENERATED_ONLY')

    def test_stopping_unstarted_fixture_does_not_kill_anything(self):
        with patch.object(client.subprocess,'Popen') as process:
            result=self.api().stop()
        process.assert_not_called()
        self.assertFalse(result['javaExitedBeforeOuterCleanup'])
        self.assertIsNone(result['javaExitCode'])

    def test_start_as_root_rejected_before_process_start(self):
        with patch.object(client.os,'getuid',return_value=0,create=True), \
             patch.object(client.os,'getgid',return_value=0,create=True), \
             patch.object(client.subprocess,'Popen') as process:
            with self.assertRaises(AssertionError):
                self.api().start()
        process.assert_not_called()

    def test_external_interface_rejected_before_process_start(self):
        with patch.object(client.os,'getuid',return_value=10001,create=True), \
             patch.object(client.os,'getgid',return_value=10001,create=True), \
             patch.object(client.os,'getgroups',return_value=[],create=True), \
             patch.object(client.socket,'if_nameindex',return_value=[(1,'lo'),(2,'eth0')]), \
             patch.object(client.subprocess,'Popen') as process:
            with self.assertRaises(AssertionError):
                self.api().start()
        process.assert_not_called()

    def test_streaming_hash_reads_bounded_chunks(self):
        body=b'GENERATED_ONLY'*100000
        stream=io.BytesIO(body)
        reads=[]
        read=stream.read
        def bounded(size):
            reads.append(size)
            return read(size)
        stream.read=bounded
        path=Mock()
        path.open.return_value=stream
        self.assertEqual(client.hash_file(path),hashlib.sha256(body).hexdigest())
        self.assertTrue(reads and all(size==1048576 for size in reads))


if __name__=='__main__':
    unittest.main()
