"""Synthetic provenance gates plus exact canonical recovery derivation; no network or credentials."""
import copy
import importlib.util
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[3]

def module(name):
    spec=importlib.util.spec_from_file_location(name.replace('-','_'),ROOT/'scripts'/name)
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

VERIFY=module('verify-release-resume-inputs.py')
RENDER=module('build-release-resume-workflow.py')
REVISION='a'*40
RUN=1234

def fixture():
    run=dict(id=RUN,event='push',head_branch='v6.1.0',head_sha=REVISION,status='completed',conclusion='failure',path='.github/workflows/release.yml')
    jobs=[dict(name=name,status='completed',conclusion='success',runner_name='GitHub Actions generated',labels=labels)
        for name,labels in VERIFY.REQUIRED.items()]
    jobs.append(dict(name=VERIFY.FAILED_PUBLISHER,status='completed',conclusion='failure'))
    jobs.extend(dict(name=name,status='completed',conclusion='skipped')
        for name in ('Publish GitHub Release','Deploy immutable image to production'))
    names=['reader-release-jar-','reader-publisher-import-','vue3-core-browser-reports-',
        'vue3-subdir-browser-reports-','vue3-manager-namespace-']
    for arch in ('amd64','arm64'):
        names.extend(['reader-tested-native-'+arch+'-','reader-native-transfer-'+arch+'-'])
        names.extend('reader-epub-pressure-'+phase+'-'+arch+'-' for phase in ('built','transferred'))
    artifacts=[dict(name=prefix+REVISION,id=index+1,size_in_bytes=100,expired=False,digest='sha256:'+'b'*64,
        workflow_run=dict(id=RUN,head_sha=REVISION)) for index,prefix in enumerate(names)]
    return run,jobs,artifacts

class ResumeInputTests(unittest.TestCase):
    def setUp(self):self.run,self.jobs,self.artifacts=fixture()
    def verify(self):return VERIFY.validate_source(self.run,self.jobs,self.artifacts,'v6.1.0',REVISION,RUN)
    def test_all_nine_hosted_source_jobs_and_thirteen_artifacts_required(self):self.assertEqual(len(self.verify()),13)
    def test_failed_skipped_or_self_hosted_required_gate_rejected(self):
        for key,value in (('conclusion','failure'),('conclusion','skipped'),('status','in_progress'),('runner_name','self-hosted'),('labels',['self-hosted'])):
            with self.subTest(key=key,value=value):
                self.run,self.jobs,self.artifacts=fixture();self.jobs[3][key]=value
                with self.assertRaises(ValueError):self.verify()
    def test_tag_source_event_and_workflow_must_match(self):
        for key,value in (('head_sha','c'*40),('head_branch','other'),('event','pull_request'),('path','other.yml'),('status','in_progress'),('conclusion','success'),('id',999)):
            with self.subTest(key=key):
                self.run,self.jobs,self.artifacts=fixture();self.run[key]=value
                with self.assertRaises(ValueError):self.verify()
    def test_already_started_release_or_production_cannot_resume(self):
        for name in ('Publish GitHub Release','Deploy immutable image to production'):
            self.run,self.jobs,self.artifacts=fixture()
            next(job for job in self.jobs if job['name']==name)['conclusion']='failure'
            with self.assertRaises(ValueError):self.verify()
    def test_does_not_resume_publisher_that_was_not_failed(self):
        next(job for job in self.jobs if job['name']==VERIFY.FAILED_PUBLISHER)['conclusion']='success'
        with self.assertRaises(ValueError):self.verify()
    def test_missing_or_ambiguous_jobs_rejected(self):
        self.jobs.pop()
        with self.assertRaises(ValueError):self.verify()
        self.run,self.jobs,self.artifacts=fixture();self.jobs[-1]['name']=self.jobs[0]['name']
        with self.assertRaises(ValueError):self.verify()
    def test_each_artifact_role_is_mandatory(self):
        original=copy.deepcopy(self.artifacts)
        for index in range(len(original)):
            self.artifacts=original[:index]+original[index+1:]
            with self.assertRaises(ValueError):self.verify()
    def test_artifacts_must_be_current_bounded_and_content_addressed(self):
        for key,value in (('expired',True),('size_in_bytes',0),('size_in_bytes',2**40),('digest','invalid'),
                ('workflow_run',dict(id=RUN+1,head_sha=REVISION)),('workflow_run',dict(id=RUN,head_sha='c'*40))):
            self.run,self.jobs,self.artifacts=fixture();self.artifacts[0][key]=value
            with self.assertRaises(ValueError):self.verify()
    def test_duplicate_artifact_names_rejected(self):
        self.artifacts.append(copy.deepcopy(self.artifacts[0]))
        with self.assertRaises(ValueError):self.verify()
    def test_registry_paths_cannot_contain_commands_or_tags(self):
        VERIFY.validate_registry_images('ghcr.io/generated/reader','generated/reader')
        for value in ('generated/reader:latest','generated/reader\nother','../repo','generated/reader;echo'):
            with self.assertRaises(ValueError):VERIFY.validate_registry_images('ghcr.io/generated/reader',value)
    def test_stable_semver_ordering_is_numeric(self):
        self.assertGreater(VERIFY.version_tuple('v6.1.0'),VERIFY.version_tuple('v6.0.45'))
        for value in ('v06.1.0','v6.1.0-rc1','v6.1','6.1.0'):
            with self.assertRaises(ValueError):VERIFY.version_tuple(value)

class ResumeWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.canonical=(ROOT/'.github/workflows/release.yml').read_text(encoding='utf-8')
        self.actual=(ROOT/'.github/workflows/release-resume.yml').read_text(encoding='utf-8')
    def test_committed_workflow_is_exact_derivation_of_canonical_jobs(self):self.assertEqual(RENDER.render(self.canonical),self.actual)
    def test_no_rebuild_and_no_tag_overwrite_added(self):
        for token in ('docker buildx build','./gradlew','npm ci','git tag -f','self-hosted'):
            self.assertNotIn(token,self.actual)
    def test_cross_run_downloads_are_bound_to_verified_source(self):
        self.assertEqual(self.actual.count('run-id: ${{ needs.verify-resume-inputs.outputs.source-run }}'),4)
        self.assertEqual(self.actual.count('ref: ${{ needs.verify-resume-inputs.outputs.tag }}'),3)
        self.assertNotIn('$GITHUB_SHA',self.actual)
        self.assertNotIn('${{ github.sha }}',self.actual)
    def test_canonical_snapshots_rescue_public_identity_and_immutable_images_retained(self):
        for token in ('snapshot_ready=false','trap rollback ERR','rescue-after-failure-',
                'storage-after-public-failure-','sha256sum -c','--no-build --force-recreate',
                'group: reader-pro-production','test ! -e "$rollback_dir/state-before-${RELEASE_TAG}.tar"'):
            self.assertIn(token,self.actual)
    def test_renderer_fails_if_canonical_shape_moves(self):
        with self.assertRaises(ValueError):RENDER.render(self.canonical.replace('    timeout-minutes: 80','    timeout-minutes: 81'))

if __name__=='__main__':unittest.main()
