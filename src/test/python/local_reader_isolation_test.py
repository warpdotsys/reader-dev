import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[3] / 'scripts/local-reader'
sys.path.insert(0, str(SOURCE))
spec = importlib.util.spec_from_file_location('local_isolation', SOURCE / 'verify-isolation.py')
isolation = importlib.util.module_from_spec(spec); spec.loader.exec_module(isolation)


class IsolationTests(unittest.TestCase):
    def test_each_security_flag_is_required(self):
        value = dict(uid=10001, gid=10001, capabilitiesEmpty=True, noNewPrivileges=True,
            foreignSecretsAbsent=True, ownSecretFilePresent=True, foreignDataNotMounted=True)
        self.assertTrue(isolation.accepted_probe(value))
        for key in value:
            with self.subTest(key=key):
                self.assertFalse(isolation.accepted_probe({**value, key: False}))

    def test_default_does_not_execute(self):
        with patch.object(isolation, 'execute') as execute:
            self.assertEqual(0, isolation.main([]))
            execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
