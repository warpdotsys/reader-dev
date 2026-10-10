"""Validate actual generated Reader API/Java-proxy HTTPS observations, not flags alone."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re

PLAN = (('a','same-get'),('a','cross-post'),('a','same-post-307'),
        ('a','same-get'),('b','same-get'),('a','cross-post-308'),('a','wrong-host'),('a','untrusted'))
AUTH = 'Bearer GENERATED_NOT_A_REAL_CREDENTIAL'
TRACE = 'GENERATED_HEADER_TRACE_ONLY'
UA = 'ReaderGeneratedHeaderProbe/1.0'
COOKIE = 'generated_session=GENERATED_SESSION_ONLY'
POST = b'q=generated-header-only'
UTF8_POST = 'q=生成中文&value=保持原始字节'.encode('utf-8')
RECORDED_WORKER_SHA = 'c3ef2486a4b6ed37e0f3a898518dd4173c42e270c62d49d87e881562c50f4c24'

def require(value, reason):
    if not value:
        raise ValueError(reason)

def header(row,name):
    return [value for key,value in row['headers'] if key.lower()==name.lower()]

def validate(report,jar_sha,revision,architecture='amd64'):
    require(re.fullmatch('[0-9a-f]{64}',jar_sha) and re.fullmatch('[0-9a-f]{40}',revision), 'Invalid expected identity')
    require(report.get('actualJarSha256')==jar_sha and report.get('sourceRevision')==revision, 'Current JAR/source mismatch')
    require(report.get('passed') is True and report.get('readerStarted') is True, 'Reader did not complete its expected outcomes')
    for key in ('realCredentialsImported','productionChanged','privateBookBodyRead','fullGoalComplete'):
        require(report.get(key) is False,'Invalid scope: '+key)
    require(report.get('browserBinaryBytesChanged') is False and
            report.get('dockerLikeReadonlyGeneratedHome') is True and
            report.get('fixtureOnlyPrivateDistributionTmpfsVerified') is True,
            'Readonly browser/home or private CA isolation evidence missing')
    require(report.get('hostIndependentlyVerifiedIdentityAndReadOnlyInputs') is True and
            report.get('independentlyStoppedPidZeroAndDeleted') is True and
            report.get('ownedContainersRemaining')==0 and report.get('originalCachedPolicyUnchanged') is True,
            'Owned runtime/input/cleanup evidence missing')
    identity=report['identity']
    require(identity['uid']==identity['gid']==10001 and identity['groups']==[] and
            identity['interfaces']==['lo'] and identity['capsZero'] is True and identity['noNewPrivileges'] is True,
            'Business test must be offline and unprivileged')
    budget=report['finalResources']
    require(budget['architecture']=={'amd64':'x86_64','arm64':'aarch64'}[architecture], 'Wrong observed architecture')
    spec=importlib.util.spec_from_file_location('reader_business_budget',Path(__file__).with_name('report-browser-cgroup.py'))
    resources=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(resources)
    resources.verify_report(budget,require_no_swap=True,expected_memory_high=1610612736)
    return validate_observation(report['observation'],jar_sha,RECORDED_WORKER_SHA)

def validate_packaged(report,jar_sha,worker_sha,revision,architecture,require_certificate_hints=False):
    """Hosted Docker scope differs from the historical rootless envelope, not the business checks."""
    require(re.fullmatch('[0-9a-f]{64}',jar_sha) and re.fullmatch('[0-9a-f]{64}',worker_sha) and
            re.fullmatch('[0-9a-f]{40}',revision),'Invalid expected packaged identity')
    require(report.get('schemaVersion')==2 and report.get('mode')=='reader-api' and
            report.get('jarSha256')==jar_sha and report.get('workerSha256')==worker_sha and
            report.get('revision')==revision and report.get('architecture')==architecture,
            'Packaged Reader business identity mismatch')
    for key in ('generatedOnly','fixtureOnlyPrivateDistributionPolicy','fixtureOnlyPrivateFontconfigTmpfs',
                'fixtureOnlyPrivateAppDataTmpfs','fixtureOnlyPrivateStorageTmpfs','httpsTested','readerJarStarted'):
        require(report.get(key) is True,'Missing packaged business isolation: '+key)
    for key in ('realCredentialsImported','privateBookBodyRead','hostTrustStoreChanged','ignoreHttpsErrorsUsed',
                'workerLaunchOverridden','productionChanged','fullGoalComplete'):
        require(report.get(key) is False,'Invalid packaged business scope: '+key)
    identity=report['identity']
    require(identity['uid']==identity['gid']==10001 and isinstance(identity['groups'],list) and
            set(identity['groups'])<={10001} and identity['interfaces']==['lo'] and
            identity['capsZero'] is True and identity['noNewPrivileges'] is True,'Invalid packaged business runtime')
    budget=report['resources']
    require(budget['architecture']=={'amd64':'x86_64','arm64':'aarch64'}[architecture],'Wrong packaged runtime architecture')
    spec=importlib.util.spec_from_file_location('packaged_business_budget',Path(__file__).with_name('report-browser-cgroup.py'))
    resources=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(resources)
    resources.verify_report(budget,require_no_swap=True)
    result=validate_observation(report['observation'],jar_sha,worker_sha)
    if require_certificate_hints:
        # Reader search preserves Throwable.toString(), not just renderer.message.
        # Require the exact public wire value; do not strip arbitrary prefixes.
        expected={'wrong-host':'java.lang.IllegalStateException: Camoufox HTTPS 证书域名不匹配 (SSL_ERROR_BAD_CERT_DOMAIN)',
                  'untrusted':'java.lang.IllegalStateException: Camoufox HTTPS 证书签发机构不受信任 (SEC_ERROR_UNKNOWN_ISSUER)'}
        for row in report['observation']['results']:
            if row['case'] in expected:
                require(row['readerApiResult']['returnData']['errorMsg']==expected[row['case']],
                        'Actual Reader certificate hint is missing, swapped, or contains untrusted text')
        result=dict(result,acceptedCertificateHintMessages=True)
    return result

def validate_observation(value,jar_sha,worker_sha):
    """Shared strict wire/header/socket/TLS checks for both actual execution envelopes."""
    require(value['readerJarStarted'] is True and value['actualJarSha256']==jar_sha and
            value['workerSha256']==worker_sha,
            'Wrong actual packaged worker/JAR')
    for key in ('hostTrustStoreChanged','ignoreHttpsErrorsUsed','workerLaunchOverridden','realCredentialsImported','fullGoalComplete'):
        require(value.get(key) is False,'Invalid observation scope: '+key)
    require(value.get('fixtureOnlyPrivateDistributionPolicy') is True,'CA fixture was not private')
    require(value['readerCleanup']['javaExitedBeforeOuterCleanup'] is True,'Java did not exit before owned container cleanup')
    auth=value['authObservationsWithoutCredentialsOrTokens']
    require([(x['actor'],x['operation']) for x in auth]==[('a','register'),('a','login'),('b','register'),('b','login')],
            'Missing actual generated login/register calls')
    require(all(x['status']==200 and x['isSuccess'] is True and x['cookieCount']>0 for x in auth), 'Authentication did not create cookie state')
    require(all(x['returnDataFieldNames']==['data','errorMsg','isSuccess'] for x in auth),'Actual auth ReturnData shape changed')
    require(all(set(x)=={'actor','operation','status','isSuccess','returnDataFieldNames','cookieCount'} for x in auth),
            'Do not retain authentication token/credential values')
    rows=value['results']
    require(value.get('generatedOnly') is True and value.get('httpsTested') is True,'Not a generated HTTPS observation')
    require([(x['account'],x['case']) for x in rows]==list(PLAN), 'Missing or reordered business cases')
    java_identity=set()
    target_count=0
    parsed_fields=0
    for index,row in enumerate(rows):
        require(row['failureType'] is None and row['workerResult'] is None,'Do not replace Reader ReturnData with worker-shaped output')
        api=row['readerApiResult']
        require(api['actor']==row['account'] and api['status']==200 and api['sourceRoundtripMatched'] is True and
                api['actualJarSha256']==jar_sha and re.fullmatch(r'http://127\.0\.0\.1:[0-9]{1,5}',api['configuredProxy']),
                'Source persistence/actual API identity mismatch')
        wire=api['returnData']
        # Retain actual wire shape: this JAR omits data on these failures.
        require(set(wire)==({'isSuccess','errorMsg'} if row['case'] in ('wrong-host','untrusted')
                            else {'isSuccess','errorMsg','data'}),'ReturnData wire fields changed')
        tunnels=row['tunnels']
        require(tunnels and len(tunnels)<=8,'No actual selected upstream proxy tunnel')
        for tunnel in tunnels:
            owner=tunnel['actualClientOwnership']
            require(owner['clientIsOwnedCurrentReaderJvm'] is True and type(owner['javaPid']) is int and owner['javaPid']>0 and
                    re.fullmatch('[0-9]+',owner['javaPidStartTick']) and re.fullmatch('[0-9]+',owner['javaSocketInode']) and
                    owner['clientHost']=='127.0.0.1' and 0<owner['clientPort']<65536 and
                    api['configuredProxy']=='http://127.0.0.1:'+str(owner['proxyPort']), 'Upstream client is not the owned current JVM')
            java_identity.add((owner['javaPid'],owner['javaPidStartTick']))
            require(tunnel['method']=='CONNECT' and tunnel['host'] in ('127.0.0.1','127.0.0.2') and
                    0<tunnel['port']<65536 and 0<tunnel['targetSourcePort']<65536 and
                    0<tunnel['clientToServerBytes']<=4194304 and 0<tunnel['serverToClientBytes']<=4194304,'Invalid actual TLS tunnel')
        negative=row['case'] in ('wrong-host','untrusted')
        if negative:
            expected='SSLV3_ALERT_BAD_CERTIFICATE' if row['case']=='wrong-host' else 'TLSV1_ALERT_UNKNOWN_CA'
            require(wire['isSuccess'] is False and isinstance(wire['errorMsg'],str) and wire['errorMsg'] and
                    row['negativeHttpRequests']==0 and row['targetRequests']==[] and len(tunnels)==1,
                    'Invalid certificate did not fail before HTTP fields')
            alerts=row['negativeTlsFailures']
            require(len(alerts)==1 and alerts[0]['reason']==expected and alerts[0]['sourcePort']==tunnels[0]['targetSourcePort'],
                    'TLS rejection reason is missing or belongs to another connection')
            continue
        require(wire['isSuccess'] is True and wire['errorMsg']=='' and isinstance(wire['data'],list) and len(wire['data'])==1 and
                wire['data'][0]['name']=='认证头差分书-'+row['case'] and wire['data'][0]['author']=='生成作者','Actual script/search parsing failed')
        book=wire['data'][0]
        defaults={'type':0,'intro':'','wordCount':'','latestChapterTitle':'','tocUrl':'',
                  'time':0,'originOrder':0,'infoHtml':'','tocHtml':''}
        require(set(book)==set(defaults)|{'bookUrl','origin','originName','name','author'} and
                all(type(book[key]) is type(default) and book[key]==default for key,default in defaults.items()) and
                book['originName']=='Generated header '+row['case'], 'Actual parsed fields/defaults changed')
        origin=book['origin'].removesuffix('/source/'+row['case'])
        require(re.fullmatch(r'https://127\.0\.0\.1:[0-9]{1,5}',origin) and
                book['origin']==origin+'/source/'+row['case'] and
                book['bookUrl']==origin+'/book/'+row['case'],'Parsed book URL/source changed')
        targets=row['targetRequests']
        require(len(targets)==len(tunnels)==2 and row['everyActualTargetConnectionUsedProxy'] is True,
                'Unexpected actual positive target requests')
        require(sorted(row['observedTargetSourcePorts'])==sorted(t['targetSourcePort'] for t in tunnels),'Unobserved target connection')
        for position,target in enumerate(targets):
            actor='secondary' if row['case'].startswith('cross-') and position==1 else 'primary'
            require(target['case']==row['case'] and target['actor']==actor and
                    target['role']==('start' if position==0 else 'end'), 'Wrong actual redirect target')
            matching=[t for t in tunnels if t['targetSourcePort']==target['targetSourcePort']]
            require(len(matching)==1 and header(target,'Host')==[matching[0]['host']+':'+str(matching[0]['port'])], 'HTTPS Host/tunnel mismatch')
            require(header(target,'Authorization')==([AUTH] if actor=='primary' else []) and
                    header(target,'X-Generated-Trace')==([TRACE] if actor=='primary' else []) and
                    header(target,'User-Agent')==[UA] and header(target,'proxy')==[],'Rule headers crossed origins or proxy leaked as a header')
            body=UTF8_POST if row['case'].endswith(('307','308')) else (POST if row['case']=='cross-post' and position==0 else b'')
            method='POST' if body else 'GET'
            require(target['method']==method and target['bodyByteCount']==len(body) and
                    target['bodySha256']==hashlib.sha256(body).hexdigest(),'Actual method/body bytes changed')
            if row['case'].endswith(('307','308')):
                require(header(target,'Content-Type')==['application/json; charset=UTF-8'],'Declared UTF-8 media type changed')
            if row['case']=='cross-post' and position==0:
                require(header(target,'Content-Type')==['application/x-www-form-urlencoded; charset=UTF-8'],'Form media type changed')
            if row['case']=='cross-post' and position==1:
                require(header(target,'Content-Type')==[],'303 GET retained POST media type')
            expected_cookie=COOKIE if ((index==2 and position==1) or index==3 or (index==5 and position==0)) else None
            require(header(target,'Cookie')==([expected_cookie] if expected_cookie else []),'Browser Cookie state crossed namespace/origin or disappeared')
            require(len(target['headers'])<=64 and sum(len(k)+len(v) for k,v in target['headers'])<=8192,'Header capture exceeded budget')
            target_count+=1
            parsed_fields+=len(target['headers'])
        if row['account']=='b':
            require(api['bookSourcesWereEmptyForSecondNamespace'] is True,'Book source namespace was shared')
    require(len(java_identity)==1 and target_count==12,'Business requests did not share one verified actual JVM')
    return {'acceptedCurrentReaderHttpsBusinessSubset':True,'generatedScenarios':8,'actualTargetRequests':12,
        'actualConnectTunnels':14,'parsedHeaderFields':parsed_fields,'negativeCertificateControls':2,
        'generatedUserNamespaces':2,'proxySocketOwnershipVerifiedByLiveProducer':True,
        'originalJarThreeWayAccepted':False,'fullNativeImageLocallyAccepted':False,'realSiteLoginAccepted':False,
        'productionChanged':False,'fullGoalComplete':False}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report',type=Path)
    parser.add_argument('--jar-sha',required=True)
    parser.add_argument('--revision',required=True)
    parser.add_argument('--packaged',action='store_true',help='Validate the actual Docker Reader API report, not a rootless envelope')
    parser.add_argument('--worker-sha',help='Mandatory build worker digest for packaged reports')
    parser.add_argument('--require-certificate-hints',action='store_true',
                        help='New-build gate only; preserve historical generic-error observations by default')
    parser.add_argument('--architecture',choices=('amd64','arm64'),default='amd64')
    args=parser.parse_args()
    require(args.report.is_file() and not args.report.is_symlink() and args.report.stat().st_size<524288,'Expected bounded regular report')
    report=json.loads(args.report.read_bytes())
    if args.packaged:
        require(args.worker_sha is not None,'Packaged business report requires worker digest')
        result=validate_packaged(report,args.jar_sha,args.worker_sha,args.revision,args.architecture,args.require_certificate_hints)
    else:
        require(args.worker_sha is None,'Rootless historical report uses its recorded worker identity')
        require(not args.require_certificate_hints,'Certificate message gate requires a current packaged report')
        result=validate(report,args.jar_sha,args.revision,args.architecture)
    print(json.dumps(result))

if __name__=='__main__':
    main()
