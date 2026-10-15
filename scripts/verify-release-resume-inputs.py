"""Fail closed before resuming only a failed registry publisher; no build, registry, or production writes."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

SEMVER=re.compile(r'v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z')
SHA=re.compile(r'[0-9a-f]{40}\Z')
NATIVE_PREFIX='Require both native image builds and fresh-runner round trips / '
VUE_PREFIX='Require complete Vue 3 browser journeys for this release commit / '
REQUIRED={
    'Verify stable release inputs':['ubuntu-24.04'],
    VUE_PREFIX+'build':['ubuntu-24.04'],
    VUE_PREFIX+'Isolated Vue 3 browser journeys in Chromium':['ubuntu-24.04'],
    NATIVE_PREFIX+'Build one shared Java Kotlin JAR':['ubuntu-24.04'],
    NATIVE_PREFIX+'Build and test native amd64 image':['ubuntu-24.04'],
    NATIVE_PREFIX+'Build and test native arm64 image':['ubuntu-24.04-arm'],
    NATIVE_PREFIX+'Reload and run tested amd64 image':['ubuntu-24.04'],
    NATIVE_PREFIX+'Reload and run tested arm64 image':['ubuntu-24.04-arm'],
    NATIVE_PREFIX+'Exercise both publisher imports without registry credentials':['ubuntu-24.04'],
}
FAILED_PUBLISHER='Publish only the two verified native images and their manifest'

def require(value,message):
    if not value:raise ValueError(message)

def validate_source(run,jobs,artifacts,tag,revision,run_id):
    require(SEMVER.fullmatch(tag) and SHA.fullmatch(revision),'Invalid immutable tag identity')
    require(run['id']==run_id and run['event']=='push' and run['head_branch']==tag
        and run['head_sha']==revision and run['status']=='completed' and run['conclusion']=='failure'
        and run['path']=='.github/workflows/release.yml','Not a failed stable-tag release run')
    optional_registry='Verify registry publish access without image writes'
    registry_present=any(job['name']==optional_registry for job in jobs)
    require(len(jobs)==12+int(registry_present) and len({job['name'] for job in jobs})==len(jobs),'Unexpected source job set')
    indexed={job['name']:job for job in jobs}
    required=dict(REQUIRED)
    if registry_present:required[optional_registry]=['ubuntu-24.04']
    for name,labels in required.items():
        job=indexed.get(name,{})
        require(job.get('status')=='completed' and job.get('conclusion')=='success'
            and job.get('labels')==labels and job.get('runner_name','').startswith('GitHub Actions '),
            'A required source build, native pressure, reload, or browser gate did not pass on a hosted runner')
    require(indexed.get(FAILED_PUBLISHER,{}).get('conclusion')=='failure','Publisher is not the failed boundary')
    for name in ('Publish GitHub Release','Deploy immutable image to production'):
        require(indexed.get(name,{}).get('status')=='completed' and indexed[name]['conclusion']=='skipped',
            'Refuse a run whose release or production activation already started')
    required={'reader-release-jar-'+revision:536870912,'reader-publisher-import-'+revision:1048576,
        'vue3-core-browser-reports-'+revision:1048576,'vue3-subdir-browser-reports-'+revision:1048576,
        'vue3-manager-namespace-'+revision:1048576}
    for arch in ('amd64','arm64'):
        required['reader-tested-native-'+arch+'-'+revision]=3221225472
        required['reader-native-transfer-'+arch+'-'+revision]=1048576
        for phase in ('built','transferred'):required['reader-epub-pressure-'+phase+'-'+arch+'-'+revision]=1048576
    require(len({item['name'] for item in artifacts})==len(artifacts),'Ambiguous source artifacts')
    selected={}
    for name,limit in required.items():
        found=[item for item in artifacts if item['name']==name]
        require(len(found)==1,'Required verified-source artifact is missing')
        item=found[0]
        require(item.get('expired') is False and 0<item['size_in_bytes']<=limit
            and item['workflow_run']['head_sha']==revision and item['workflow_run']['id']==run_id
            and re.fullmatch(r'sha256:[0-9a-f]{64}',item.get('digest','')),
            'Wrong source, expired, oversized, or unbound artifact')
        selected[name]={key:item[key] for key in ('id','size_in_bytes','digest')}
    return selected

def validate_registry_images(ghcr,dockerhub):
    require(re.fullmatch(r'ghcr\.io/[a-z0-9][a-z0-9._-]*/[a-z0-9][a-z0-9._-]*',ghcr),'Invalid GHCR repository')
    require(re.fullmatch(r'[a-z0-9][a-z0-9._-]*/[a-z0-9][a-z0-9._-]*',dockerhub),'Invalid Docker Hub repository')

def version_tuple(tag):
    match=SEMVER.fullmatch(tag)
    require(match is not None,'Invalid stable SemVer')
    return tuple(map(int,match.groups()))

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag',required=True)
    parser.add_argument('--source-run',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    require(SEMVER.fullmatch(args.tag) and re.fullmatch('[1-9][0-9]{0,17}',args.source_run),'Invalid recovery inputs')
    require(not args.output.exists(),'Do not overwrite a prior input receipt')
    repository=os.environ['GITHUB_REPOSITORY']
    require(re.fullmatch('[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository),'Invalid repository')
    def api(path):
        value=subprocess.run(['gh','api','repos/'+repository+'/'+path],capture_output=True,check=True,timeout=30)
        require(len(value.stdout)<=8388608,'GitHub response exceeds bound')
        return json.loads(value.stdout)
    revision=subprocess.run(['git','rev-parse',args.tag+'^{commit}'],capture_output=True,text=True,check=True).stdout.strip()
    require(SHA.fullmatch(revision),'Cannot resolve exact annotated tag commit')
    subprocess.run(['git','merge-base','--is-ancestor',revision,'origin/legacy'],check=True,capture_output=True)
    source_run=int(args.source_run)
    run=api('actions/runs/'+args.source_run)
    require(run['repository']['full_name'].lower()==repository.lower(),'Foreign source repository')
    jobs_response=api('actions/runs/'+args.source_run+'/jobs?per_page=100')
    require(jobs_response['total_count']==len(jobs_response['jobs']),'Incomplete source job inventory')
    artifact_response=api('actions/runs/'+args.source_run+'/artifacts?per_page=100')
    require(artifact_response['total_count']==len(artifact_response['artifacts']),'Incomplete source artifact inventory')
    selected=validate_source(run,jobs_response['jobs'],artifact_response['artifacts'],args.tag,revision,source_run)
    stable=[]
    complete=False
    for page in range(1,11):
        releases=api('releases?per_page=100&page='+str(page))
        require(not any(item['tag_name']==args.tag for item in releases),'Release already exists; do not overwrite it')
        stable.extend(version_tuple(item['tag_name']) for item in releases
            if not item['draft'] and not item['prerelease'] and SEMVER.fullmatch(item['tag_name']))
        if len(releases)<100:complete=True;break
    require(complete and (not stable or version_tuple(args.tag)>max(stable)),'Release version must be above all existing stable releases')
    def source(path):
        return subprocess.run(['git','show',revision+':'+path],capture_output=True,text=True,encoding='utf-8',check=True).stdout
    version=args.tag[1:]
    require(re.search(r'^version = "'+re.escape(version)+r'"$',source('build.gradle.kts'),re.M),'Backend version differs')
    for path in ('web/package.json','web-vue3/package.json'):
        require(json.loads(source(path))['version']==version,'Frontend version differs')
    require(re.search(r'^##\s+已知问题',source('docs/releases/'+args.tag+'.md'),re.M),'Release notes lack known limitations')
    ghcr=os.environ.get('GHCR_IMAGE','');dockerhub=os.environ.get('DOCKERHUB_IMAGE','')
    validate_registry_images(ghcr,dockerhub)
    values={'tag':args.tag,'version':version,'revision':revision,'source-run':args.source_run,
        'ghcr-image':ghcr,'dockerhub-image':dockerhub}
    with Path(os.environ['GITHUB_OUTPUT']).open('a',encoding='utf-8') as output:
        for key,value in values.items():output.write(key+'='+value+'\n')
    report=dict(schemaVersion=1,sourceRun=source_run,sourceRunAttempt=run['run_attempt'],
        sourceRevision=revision,tag=args.tag,requiredSourceJobs=sorted([*REQUIRED]+([
            'Verify registry publish access without image writes'] if any(job['name']=='Verify registry publish access without image writes'
                for job in jobs_response['jobs']) else [])),
        artifactMetadata=selected,images={'ghcr':ghcr,'dockerhub':dockerhub},
        sourcePublisherFailedButAllBuildsAndPressurePassed=True,
        sourceReleaseAndProductionSkipped=True,applicationRebuilt=False,
        tagOrSourceMoved=False,registryOrProductionWrittenByPreflight=False)
    with args.output.open('x',encoding='utf-8') as output:json.dump(report,output,indent=2)
    print(json.dumps(dict(accepted=True,sourceRun=source_run,sourceRevision=revision,tag=args.tag,
        selectedSourceArtifacts=len(selected),applicationRebuilt=False)))

if __name__=='__main__':main()
