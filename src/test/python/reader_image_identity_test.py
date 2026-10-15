"""Generated identity objects only; no image, private data, daemon or network required."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest import mock

ROOT=Path(__file__).resolve().parents[3]
SPEC=importlib.util.spec_from_file_location('reader_image_identity',ROOT/'scripts/reader_image_identity.py')
VERIFY=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(VERIFY)
REVISION='a'*40
JAR='b'*64

def raw(value):return json.dumps(value,separators=(',',':')).encode()
def sha(value):return 'sha256:'+hashlib.sha256(value).hexdigest()
def fixture():
    runtime=dict(User='10001:10001',Entrypoint=['/usr/local/bin/reader-entrypoint'],WorkingDir='/app',
        Env=['READER_BUILD_REVISION='+REVISION,'READER_APP_WEBUI=vue3','READER_APP_WEBVIEWRENDERER=camoufox'])
    config=dict(os='linux',architecture='amd64',config=runtime,rootfs={'type':'layers','diff_ids':['sha256:'+'c'*64]})
    config_raw=raw(config);config_id=sha(config_raw)
    manifest=dict(schemaVersion=2,mediaType='application/vnd.oci.image.manifest.v1+json',
        config=dict(mediaType='application/vnd.oci.image.config.v1+json',digest=config_id,size=len(config_raw)),
        layers=[dict(mediaType='application/vnd.oci.image.layer.v1.tar',digest='sha256:'+'c'*64,size=1024)])
    manifest_raw=raw(manifest);manifest_id=sha(manifest_raw)
    image=dict(Id=manifest_id,Os='linux',Architecture='amd64',Config=copy.deepcopy(runtime),
        RootFS={'Type':'layers','Layers':config['rootfs']['diff_ids'][:]},
        Descriptor={'digest':manifest_id,'mediaType':manifest['mediaType']})
    return image,config_id,manifest_raw,config_raw

class ReaderImageIdentityTest(unittest.TestCase):
    def verify(self,values):return VERIFY.validate_loaded_config(*values,'amd64',REVISION,JAR)
    def test_manifest_config_and_all_layer_relationship_is_verified_without_normalization(self):
        values=fixture();proof=self.verify(values)
        self.assertNotEqual(proof['loadedImageId'],proof['originalConfigDigest'])
        self.assertEqual(values[0]['Id'],proof['loadedImageId'])
        self.assertEqual(values[1],proof['originalConfigDigest'])
        self.assertTrue(proof['manifestBytesSha256Verified'])
        self.assertFalse(proof['packagedJarBytesIndependentlyRead'])
        self.assertFalse(proof['runtimeExecuted'])
    def test_classic_config_id_is_distinct_supported_case_not_manifest_alias(self):
        image,cid,_,config=fixture();image['Id']=cid
        proof=self.verify((image,cid,None,config))
        self.assertEqual('config',proof['loadedIdObjectKind'])
        self.assertFalse(proof['manifestBytesSha256Verified'])
    def test_same_environment_never_substitutes_different_config_bytes(self):
        image,cid,manifest,config=fixture()
        with self.assertRaisesRegex(ValueError,'byte checksum'):
            self.verify((image,cid,manifest,config+b'\n'))
    def test_manifest_exact_bytes_are_required_even_when_json_semantics_same(self):
        image,cid,manifest,config=fixture()
        with self.assertRaisesRegex(ValueError,'byte checksum'):
            self.verify((image,cid,manifest+b'\n',config))
    def test_manifest_different_config_size_digest_or_media_type_rejected(self):
        for field,value in [('digest','sha256:'+'d'*64),('size',True),('size',1),('mediaType','application/octet-stream')]:
            image,cid,manifest,config=fixture();doc=json.loads(manifest);doc['config'][field]=value
            blob=raw(doc);image['Id']=image['Descriptor']['digest']=sha(blob)
            with self.subTest(field=field,value=value),self.assertRaises(ValueError):self.verify((image,cid,blob,config))
    def test_wrong_missing_duplicate_or_compressed_layers_rejected(self):
        for mutate in (lambda v:v['layers'].clear(),lambda v:v['layers'].append(v['layers'][0]),
            lambda v:v['layers'][0].update(digest='sha256:'+'d'*64),
            lambda v:v['layers'][0].update(size=False),
            lambda v:v['layers'][0].update(mediaType='application/vnd.oci.image.layer.v1.tar+gzip')):
            image,cid,manifest,config=fixture();doc=json.loads(manifest);mutate(doc)
            blob=raw(doc);image['Id']=image['Descriptor']['digest']=sha(blob)
            with self.assertRaises(ValueError):self.verify((image,cid,blob,config))
    def test_mismatched_inspected_descriptor_and_rootfs_fail(self):
        for mutate in (lambda v:v['Descriptor'].update(digest='sha256:'+'e'*64),
                       lambda v:v['RootFS']['Layers'].clear()):
            values=list(fixture());mutate(values[0])
            with self.assertRaises(ValueError):self.verify(values)
    def test_all_executable_runtime_fields_are_compared(self):
        for field in ('User','Env','Entrypoint','Cmd','WorkingDir','Healthcheck','Volumes','StopSignal','OnBuild'):
            values=list(fixture());values[0]['Config'][field]='different'
            with self.subTest(field=field),self.assertRaises(ValueError):self.verify(values)
    def test_duplicate_expected_env_or_old_ui_requires_rehash_but_still_rejected(self):
        for mutation in (lambda v:v.append('READER_APP_WEBUI=vue3'),
                         lambda v:v.__setitem__(1,'READER_APP_WEBUI=vue2')):
            image,_,manifest,config=fixture();doc=json.loads(config);mutation(doc['config']['Env'])
            blob=raw(doc);cid=sha(blob);image['Config']=doc['config']
            md=json.loads(manifest);md['config'].update(digest=cid,size=len(blob));mb=raw(md)
            image['Id']=image['Descriptor']['digest']=sha(mb)
            with self.assertRaises(ValueError):self.verify((image,cid,mb,blob))
    def test_unsafe_id_revision_sha_and_architecture_fail(self):
        values=fixture()
        for architecture,revision,jar in [('x86_64',REVISION,JAR),('amd64','legacy',JAR),('amd64',REVISION,'bad')]:
            with self.assertRaises(ValueError):VERIFY.validate_loaded_config(*values,architecture,revision,jar)
        with self.assertRaises(ValueError):VERIFY.hashed_object(values[3],'reader:latest')
    def test_duplicate_keys_nonfinite_and_oversize_identity_json_rejected(self):
        for value in (b'{"a":1,"a":2}',b'{"a":NaN}',b'[]',b' '* (VERIFY.MAX_BLOB+1)):
            with self.assertRaises(ValueError):VERIFY.object_from(value)
    def test_read_only_inspection_uses_only_pinned_digests_and_returns_proof(self):
        image,cid,manifest,config=fixture()
        def execute(*args):
            content=json.dumps([image]).encode() if args[0]=='docker' else config if args[-1]==cid else manifest
            return content
        with mock.patch.object(VERIFY,'bounded_command',side_effect=execute) as commands:
            proof=VERIFY.inspect_verified(image['Id'],cid,'amd64',REVISION,JAR)
        self.assertEqual(3,commands.call_count)
        self.assertTrue(all(call.args[1] in ('image','--namespace') for call in commands.call_args_list))
        self.assertTrue(proof['readOnlyIdentityCheck'])
    def test_invalid_immutable_id_refuses_before_any_command(self):
        with mock.patch.object(VERIFY,'bounded_command') as command:
            with self.assertRaises(ValueError):VERIFY.inspect_verified('reader:latest','sha256:'+'b'*64,'amd64',REVISION,JAR)
            command.assert_not_called()
    def test_streaming_command_rejects_large_output_and_stops_owned_child(self):
        with self.assertRaisesRegex(ValueError,'byte bound'):
            VERIFY.bounded_command(sys.executable,'-c','import sys;sys.stdout.buffer.write(b"x"*1048578)')
    def test_command_failure_does_not_publish_stderr(self):
        with self.assertRaisesRegex(RuntimeError,'identity command failed'):
            VERIFY.bounded_command(sys.executable,'-c','import sys;print("generated-private-log",file=sys.stderr);sys.exit(3)')

if __name__=='__main__':unittest.main()
