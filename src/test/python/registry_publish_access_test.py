"""Only synthetic token metadata; never authenticates or reads a personal credential."""
import base64
import importlib.util
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('registry_access',ROOT/'scripts/check-registry-publish-access.py')
ACCESS=importlib.util.module_from_spec(spec);spec.loader.exec_module(ACCESS)

def token(actions,name='generated/repo',**updates):
    value=dict(iss='auth.docker.io',aud='registry.docker.io',exp=2000,
        access=[dict(type='repository',name=name,actions=actions)])
    value.update(updates)
    raw=base64.urlsafe_b64encode(json.dumps(value).encode()).decode().rstrip('=')
    return 'synthetic.'+raw+'.synthetic-signature-not-verified'

class RegistryAccessTests(unittest.TestCase):
    def test_reports_exact_requested_repository_grants(self):
        self.assertEqual({'pull':True,'push':True},ACCESS.reported_actions(token(['pull','push']),'generated/repo',1000))

    def test_successful_token_exchange_does_not_imply_push(self):
        self.assertEqual({'pull':True,'push':False},ACCESS.reported_actions(token(['pull']),'generated/repo',1000))

    def test_empty_grant_is_not_permission(self):
        self.assertEqual({'pull':False,'push':False},ACCESS.reported_actions(token([]),'generated/repo',1000))

    def test_different_namespace_cannot_grant_requested_target(self):
        self.assertFalse(ACCESS.reported_actions(token(['push'],name='other/repo'),'generated/repo',1000)['push'])

    def test_other_issuer_audience_or_expiry_is_rejected(self):
        for change in ({'iss':'other.invalid'},{'aud':'other.registry'},{'exp':999},{'exp':True},{'exp':1000000}):
            with self.assertRaises(ValueError):ACCESS.reported_actions(token(['push'],**change),'generated/repo',1000)

    def test_opaque_oversized_or_malformed_token_not_assumed_to_grant_access(self):
        for raw in ('opaque','a..c','x'*65537,'a.!!!!.c'):
            with self.assertRaises((ValueError,TypeError)):ACCESS.reported_actions(raw,'generated/repo',1000)

    def test_credential_redirect_is_not_followed(self):
        with self.assertRaises(ValueError):ACCESS.NoRedirect().redirect_request(None,None,None,None,None,None)

    def test_no_registry_write_request_or_token_file_in_source(self):
        text=(ROOT/'scripts/check-registry-publish-access.py').read_text(encoding='utf-8')
        self.assertNotIn('method="POST"',text)
        self.assertNotIn('offline_token',text)
        self.assertNotIn('json.dump(value,',text)

if __name__=='__main__':unittest.main()
