"""Read-only content-addressed image identity; never pull, load, tag or rebuild.

Classic Docker image IDs address config bytes. A containerd-backed Docker may
address a manifest instead. Only explicit hash-verified manifest -> config ->
layer evidence establishes their relationship; similar environment is not proof.
"""
import argparse
import hashlib
import json
import re
import subprocess
import threading

MAX_BLOB=1024*1024
DIGEST=re.compile(r'sha256:[0-9a-f]{64}\Z')
CONFIG_TYPES={'application/vnd.oci.image.config.v1+json','application/vnd.docker.container.image.v1+json'}
MANIFEST_TYPES={'application/vnd.oci.image.manifest.v1+json','application/vnd.docker.distribution.manifest.v2+json'}

def bounded_command(*args):
    child=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
    expired=threading.Event()
    def deadline():
        expired.set()
        if child.poll() is None:child.kill()
    timer=threading.Timer(20,deadline)
    timer.daemon=True
    timer.start()
    try:
        raw=child.stdout.read(MAX_BLOB+1)
        if len(raw)>MAX_BLOB:
            raise ValueError('Identity command output exceeds byte bound')
        code=child.wait(timeout=2)
        if expired.is_set():raise TimeoutError('Read-only identity command exceeded deadline')
        if code!=0:raise RuntimeError('Read-only identity command failed')
        return raw
    finally:
        timer.cancel()
        if child.poll() is None:
            child.kill();child.wait(timeout=2)
        child.stdout.close()

def check(value,reason):
    if not value: raise ValueError(reason)

def object_from(raw):
    check(type(raw) is bytes and 0<len(raw)<=MAX_BLOB,'Invalid bounded identity blob')
    def pairs(values):
        result={}
        for key,value in values:
            check(key not in result,'Duplicate identity field')
            result[key]=value
        return result
    value=json.loads(raw.decode('utf-8'),object_pairs_hook=pairs,
        parse_constant=lambda _: (_ for _ in ()).throw(ValueError('Non-finite identity field')))
    check(type(value) is dict,'Identity blob must be an object')
    return value

def hashed_object(raw,expected):
    check(type(expected) is str and DIGEST.fullmatch(expected),'Invalid expected identity digest')
    value=object_from(raw)
    check('sha256:'+hashlib.sha256(raw).hexdigest()==expected,'Identity blob byte checksum mismatch')
    return value

def validate_loaded_config(image,expected_config,manifest_raw,config_raw,architecture,revision,jar_sha):
    check(type(image) is dict and image.get('Os')=='linux' and image.get('Architecture')==architecture,
          'Loaded platform mismatch')
    check(architecture in ('amd64','arm64'),'Unsupported identity architecture')
    check(type(revision) is str and re.fullmatch('[0-9a-f]{40}',revision),'Invalid expected revision')
    check(type(jar_sha) is str and re.fullmatch('[0-9a-f]{64}',jar_sha),'Invalid expected JAR digest')
    loaded_id=image.get('Id')
    check(type(loaded_id) is str and DIGEST.fullmatch(loaded_id),'Invalid immutable loaded ID')
    config=hashed_object(config_raw,expected_config)
    check(config.get('os')=='linux' and config.get('architecture')==architecture,'Config platform mismatch')
    rootfs=config.get('rootfs')
    check(type(rootfs) is dict and rootfs.get('type')=='layers','Invalid config rootfs')
    layers=rootfs.get('diff_ids')
    check(type(layers) is list and 1<=len(layers)<=128 and all(type(v) is str and DIGEST.fullmatch(v) for v in layers),
          'Invalid config layer identities')
    check(image.get('RootFS')=={'Type':'layers','Layers':layers},'Loaded filesystem differs from config')
    runtime=config.get('config')
    actual=image.get('Config')
    check(type(runtime) is dict and type(actual) is dict,'Missing runtime config')
    # Every executable runtime field present in either representation must agree.
    fields=('User','Env','Entrypoint','Cmd','WorkingDir','ExposedPorts','Volumes','Labels',
            'StopSignal','StopTimeout','Healthcheck','Shell','OnBuild','ArgsEscaped')
    for field in fields:
        check(runtime.get(field)==actual.get(field),'Loaded runtime field mismatch: '+field)
    check(runtime.get('User')=='10001:10001' and runtime.get('WorkingDir')=='/app' and
          runtime.get('Entrypoint')==['/usr/local/bin/reader-entrypoint'],'Unsafe packaged runtime')
    env=runtime.get('Env')
    check(type(env) is list and all(type(v) is str for v in env),'Invalid runtime environment')
    for key,value in {'READER_BUILD_REVISION':revision,'READER_APP_WEBUI':'vue3',
                      'READER_APP_WEBVIEWRENDERER':'camoufox'}.items():
        check([v for v in env if v.startswith(key+'=')]==[key+'='+value],'Runtime environment mismatch: '+key)
    kind='config'
    if loaded_id==expected_config:
        check(manifest_raw is None,'Unexpected manifest for classic config-addressed image')
    else:
        kind='manifest'
        manifest=hashed_object(manifest_raw,loaded_id)
        check(manifest.get('schemaVersion')==2 and type(manifest.get('schemaVersion')) is int and
              manifest.get('mediaType') in MANIFEST_TYPES,'Unsupported loaded manifest')
        descriptor=manifest.get('config')
        check(type(descriptor) is dict and descriptor.get('digest')==expected_config and
              descriptor.get('mediaType') in CONFIG_TYPES and type(descriptor.get('size')) is int and
              descriptor['size']==len(config_raw),'Manifest does not reference exact original config bytes')
        check(image.get('Descriptor',{}).get('digest')==loaded_id and
              image['Descriptor'].get('mediaType')==manifest['mediaType'],'Inspected descriptor mismatch')
        descriptors=manifest.get('layers')
        check(type(descriptors) is list and len(descriptors)==len(layers),'Loaded manifest layer count mismatch')
        for row,diff_id in zip(descriptors,layers):
            check(type(row) is dict and row.get('mediaType')=='application/vnd.oci.image.layer.v1.tar' and
                  row.get('digest')==diff_id and type(row.get('size')) is int and row['size']>0,
                  'Require exact uncompressed layer evidence; compressed/index mappings unsupported')
    return {'schemaVersion':1,'loadedImageId':loaded_id,'loadedIdObjectKind':kind,
            'originalConfigDigest':expected_config,'configBytesSha256Verified':True,
            'manifestBytesSha256Verified':kind=='manifest','allOriginalLayerDiffIdsMatch':True,
            'layerCount':len(layers),'architecture':architecture,'expectedRevision':revision,
            'expectedJarSha256':jar_sha,'packagedJarBytesIndependentlyRead':False,
            'readOnlyIdentityCheck':True,'runtimeExecuted':False}

def inspect_verified(image_id,expected_config,architecture,revision,jar_sha):
    for value in (image_id,expected_config):
        check(type(value) is str and DIGEST.fullmatch(value),'Identity requires immutable sha256 ID')
    run=bounded_command
    images=json.loads(run('docker','image','inspect',image_id))
    check(type(images) is list and len(images)==1 and images[0].get('Id')==image_id,'Unexpected inspected identity')
    def blob(value):
        raw=run('ctr','--namespace','moby','content','get',value)
        check(len(raw)<=MAX_BLOB,'Identity content exceeds bound')
        return raw
    config=blob(expected_config)
    manifest=None if image_id==expected_config else blob(image_id)
    return validate_loaded_config(images[0],expected_config,manifest,config,architecture,revision,jar_sha)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image',required=True)
    parser.add_argument('--config',required=True)
    parser.add_argument('--architecture',choices=('amd64','arm64'),required=True)
    parser.add_argument('--revision',required=True)
    parser.add_argument('--jar-sha',required=True)
    args=parser.parse_args()
    print(json.dumps(inspect_verified(args.image,args.config,args.architecture,args.revision,args.jar_sha)))

if __name__=='__main__':main()
