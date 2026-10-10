"""Generated-only finite API phase diagnostics; no Java, credentials or external traffic."""
import importlib.util
import json
from pathlib import Path
import contextlib
import io
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

p = Path(__file__).resolve().parents[3] / 'scripts/compare-webview-cookie.py'
s = importlib.util.spec_from_file_location('generated_phase_probe', p)
m = importlib.util.module_from_spec(s)
s.loader.exec_module(m)

class GeneratedApiPhaseTest(unittest.TestCase):
    def test_phase_does_not_forge_a_failed_api_as_completed(self):
        phase = m.GeneratedApiPhase()
        self.assertEqual('generated', phase.call('sourceSave', lambda: 'generated'))
        def timeout(): raise TimeoutError('PRIVATE_generated_exception')
        with self.assertRaises(TimeoutError): phase.call('sourceRead', timeout)
        self.assertEqual({'activePhase':'sourceRead','lastCompletedPhase':'sourceSave',
            'startedCalls':2,'completedCalls':1}, phase.failure_context())
        self.assertNotIn('PRIVATE', json.dumps(phase.failure_context()))

    def test_rejected_phase_cannot_invoke_the_callback_or_serialize_untrusted_values(self):
        phase = m.GeneratedApiPhase()
        calls = []
        for value in ('PRIVATE_URL?token=generated', '', None, [], True, 1):
            with self.assertRaises(ValueError): phase.call(value, lambda: calls.append(1))
        self.assertEqual([], calls)
        self.assertEqual({'activePhase':'betweenCalls','lastCompletedPhase':None,
            'startedCalls':0,'completedCalls':0}, phase.failure_context())

    def test_all_finite_operations_preserve_args_kwargs_return_value_and_execute_once(self):
        for stage in m.GeneratedApiPhase.ALLOWED:
            phase = m.GeneratedApiPhase()
            calls = []
            def operation(*args, **kwargs):
                calls.append((args, kwargs))
                return {'generated':True}
            self.assertEqual({'generated':True}, phase.call(stage, operation, 'PRIVATE_generated', token='PRIVATE_generated'))
            self.assertEqual([(('PRIVATE_generated',), {'token':'PRIVATE_generated'})], calls)
            self.assertEqual(stage, phase.failure_context()['lastCompletedPhase'])
            self.assertEqual('betweenCalls', phase.failure_context()['activePhase'])
            self.assertNotIn('PRIVATE', json.dumps(phase.failure_context()))

    def test_startup_retry_context_records_attempts_without_adding_its_own_retry(self):
        phase = m.GeneratedApiPhase()
        calls = []
        def failing():
            calls.append(1)
            raise OSError('generated')
        with self.assertRaises(OSError): phase.call('startup', failing)
        phase.call('startup', lambda: None)
        phase.call('register', lambda: True)
        self.assertEqual([1], calls)
        self.assertEqual({'activePhase':'betweenCalls','lastCompletedPhase':'register',
            'startedCalls':3,'completedCalls':2}, phase.failure_context())

    def test_post_and_metadata_timeout_never_replay_an_operation(self):
        for stage in ('postSearch','metadataGetBookInfo'):
            phase = m.GeneratedApiPhase()
            calls = []
            def timeout():
                calls.append(1)
                raise TimeoutError('generated')
            with self.assertRaises(TimeoutError): phase.call(stage, timeout)
            self.assertEqual([1], calls)
            self.assertEqual(stage, phase.failure_context()['activePhase'])
            self.assertEqual(0, phase.failure_context()['completedCalls'])

    def test_real_probe_control_flow_preserves_source_read_timeout_phase(self):
        fixture = SimpleNamespace(snapshot=lambda: [], request_snapshot=lambda: [], observe_target_headers=False)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = mock.Mock()
            process.poll.return_value = None
            with mock.patch.object(m.subprocess, 'Popen', return_value=process), \
                    mock.patch.object(m, 'require_success', side_effect=[True, True, True, True,
                        TimeoutError('PRIVATE_generated_error')]), contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(TimeoutError):
                    m.run_jar('generated-java', Path('generated.jar'), root/'data', 9,
                        'http://127.0.0.1:1', fixture, failure_report=root/'failure.json')
            result = json.loads((root/'failure.json').read_text(encoding='utf-8'))
            self.assertEqual({'activePhase':'sourceRead','lastCompletedPhase':'sourceSave',
                'startedCalls':5,'completedCalls':4}, result['apiFailureContext'])
            self.assertEqual([], result['completedSearches'])
            self.assertNotIn('PRIVATE', json.dumps(result))
            process.terminate.assert_called_once()

    def test_actual_metadata_branch_records_attempt_but_not_return_or_completion(self):
        helper = m.metadata_helper()
        fixture = SimpleNamespace(metadata=helper, snapshot=lambda: [], request_snapshot=lambda: [],
                                  observe_target_headers=False)
        saved = {}
        def success(opener, base, path, body=None, include_return_data=False):
            if path.endswith('saveBookSource'): saved.clear(); saved.update(body)
            data = dict(saved) if path.endswith('getBookSource') else [{'name':'WebView差分书'}]
            value = {'status':200,'isSuccess':True,'errorMsg':'','data':data}
            if include_return_data: value['returnData'] = {'isSuccess':True,'errorMsg':'','data':data}
            return value
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = mock.Mock()
            process.poll.return_value = None
            with mock.patch.object(m.subprocess, 'Popen', return_value=process), \
                    mock.patch.object(m, 'require_verified_private_loopback'), \
                    mock.patch.object(m, 'require_success', side_effect=success), \
                    mock.patch.object(m, 'request', side_effect=TimeoutError('PRIVATE_generated_error')) as request, \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(TimeoutError):
                    m.run_jar('generated-java', Path('generated.jar'), root/'data', 9,
                        'http://127.0.0.1:1', fixture, exercise_script=True, exercise_post=True,
                        renderer_base='http://127.0.0.1:2', include_data=True, exercise_metadata=True,
                        metadata_clock_contract=True, failure_report=root/'failure.json')
            result = json.loads((root/'failure.json').read_text(encoding='utf-8'))
            self.assertEqual({'activePhase':'metadataGetBookInfo','lastCompletedPhase':'metadataSourceRead',
                'startedCalls':15,'completedCalls':14}, result['apiFailureContext'])
            self.assertEqual(5, len(result['completedSearches']))
            self.assertEqual(1, result['metadataBookInfoApiCalls'])
            self.assertIsNone(result['lastObservedMetadata'])
            self.assertNotIn('PRIVATE', json.dumps(result))
            request.assert_called_once()
            process.terminate.assert_called_once()

if __name__ == '__main__': unittest.main()
