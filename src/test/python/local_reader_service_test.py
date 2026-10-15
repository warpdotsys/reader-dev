"""Generated metadata tests; run under WSL Python (fcntl is Linux-specific)."""
import importlib.util
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[3] / 'scripts/local-reader'
sys.path.insert(0, str(SOURCE))
import service


class ServiceTests(unittest.TestCase):
    def test_runtime_env_loopback_and_checks(self):
        result = dict(x.split('=', 1) for x in service.runtime_env([
            'PATH=/opt/java/openjdk/bin:/usr/bin', 'READER_APP_LICENSECHECKENABLED=false',
            'LICENSE_ADMIN_TOKEN=do-not-copy', 'READER_APP_SECUREKEY=do-not-copy',
            'READER_APP_LICENSEPUBLICKEY=do-not-copy']))
        self.assertEqual(result['READER_SERVER_BINDADDRESS'], '127.0.0.1')
        self.assertEqual(result['READER_SERVER_PORT'], '18963')
        self.assertEqual(result['READER_APP_LICENSECHECKENABLED'], 'true')
        self.assertEqual(result['READER_APP_SECURE'], 'true')
        self.assertEqual(result['READER_BROWSER_ALLOW_PRIVATE_NETWORKS'], 'false')
        self.assertFalse({'LICENSE_ADMIN_TOKEN', 'READER_APP_SECUREKEY', 'READER_APP_LICENSEPUBLICKEY'} & result.keys())

    def test_generated_loopback_requires_explicit_test_switch(self):
        result = dict(x.split('=', 1) for x in service.runtime_env([], True))
        self.assertEqual(result['READER_BROWSER_ALLOW_PRIVATE_NETWORKS'], 'true')

    def test_oci_budget_user_and_persistence(self):
        value = service.config(Path('/generated/root'), Path('/generated/frozen'), ['PATH=/usr/bin'], 'reader', 'generated-reader')
        self.assertEqual(value['process']['user']['uid'], 10001)
        self.assertEqual(['/usr/bin/python3', '/local-license/init.py', '--role', 'reader'], value['process']['args'])
        self.assertTrue(value['process']['noNewPrivileges'])
        self.assertTrue(all(not x for x in value['process']['capabilities'].values()))
        self.assertEqual(value['linux']['resources']['memory']['limit'], 2147483648)
        self.assertEqual(value['linux']['resources']['cpu']['quota'], 200000)
        self.assertNotIn('network', [x['type'] for x in value['linux']['namespaces']])
        mounts = {x['destination']: x for x in value['mounts']}
        for destination in ('/storage', '/logs'):
            self.assertIn('rw', mounts[destination]['options'])
            self.assertEqual(mounts[destination]['source'], str(service.MOUNT / 'data' / destination[1:]))
        self.assertIn('ro', mounts['/local-secrets/reader-admin-key']['options'])
        self.assertIn('ro', mounts['/local-license']['options'])

    def test_license_and_reader_mounts_do_not_cross_expose_private_material(self):
        reader = {x['destination']: x for x in service.config(Path('/root'), Path('/frozen'), [], 'reader', 'reader')['mounts']}
        license_config = service.config(Path('/root'), Path('/frozen'), [], 'license', 'license')
        license = {x['destination']: x for x in license_config['mounts']}
        self.assertEqual(['/usr/bin/python3', '/local-license/init.py', '--role', 'license'], license_config['process']['args'])
        self.assertIn('/local-secrets/reader-admin-key', reader)
        self.assertNotIn('/local-secrets/license-admin-token', reader)
        self.assertNotIn('/license-data', reader)
        self.assertIn('/local-secrets/license-admin-token', license)
        self.assertNotIn('/local-secrets/reader-admin-key', license)
        self.assertIn('/license-data', license)
        self.assertNotIn('/storage', license)
        self.assertEqual(license['/logs']['source'], str(service.MOUNT / 'data' / 'license-logs'))

    def test_receipt_rejects_wrong_root_before_using_disk(self):
        value = {'accepted': True, 'executed': True, 'lock': service.prepare.LOCK,
                 'imageDisk': '/mnt/d/unrelated-user-data'}
        with patch.object(service, 'metadata', return_value=value):
            with self.assertRaisesRegex(ValueError, 'path mismatch'):
                service.verified_receipt()

    def test_receipt_rejects_unaccepted(self):
        with patch.object(service, 'metadata', return_value={'accepted': False}):
            with self.assertRaisesRegex(ValueError, 'identity mismatch'):
                service.verified_receipt()

    def test_no_format_or_broad_cleanup_entrypoint(self):
        source = (SOURCE / 'service.py').read_text()
        self.assertNotIn("command('mkfs", source)
        self.assertNotIn('shutil.rmtree(data', source)
        self.assertIn("runroot.parent != MOUNT / 'image/service-runs'", source)
        self.assertIn("command('mountpoint', '-q', target", source)


if __name__ == '__main__':
    unittest.main()
