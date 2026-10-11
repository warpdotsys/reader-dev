"""Generated-only timing/safety checks; no real book, browser or internet."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest

spec = importlib.util.spec_from_file_location('worker_observer', Path(__file__).resolve().parents[3] / 'scripts/epub_worker_lifecycle_probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class WorkerObserverTests(unittest.TestCase):
    def value(self):
        return {'supported': True, 'overflow': False, 'publishFailed': False, 'constructorStillInstalled': True, 'records': [
            {'kind': 'created', 'workerId': 1, 'epochMs': 1000},
            {'kind': 'posted', 'workerId': 1, 'epochMs': 1001, 'inputIsArrayBuffer': True, 'inputBytes': 123},
            {'kind': 'message', 'workerId': 1, 'epochMs': 1002, 'filesReceived': True,
                'filesCount': 2, 'outputBytes': 999, 'errorReply': False},
            {'kind': 'terminated', 'workerId': 1, 'epochMs': 1003}]}

    def test_complete_finite_snapshot_detaches_input(self):
        value = self.value()
        result = probe.validate_snapshot(value)
        self.assertEqual(result, value)
        self.assertIsNot(result['records'], value['records'])

    def test_no_worker_use_is_not_invented_into_execution(self):
        value = self.value()
        value['records'] = []
        self.assertEqual(probe.validate_snapshot(value)['records'], [])

    def test_body_or_filename_field_is_rejected(self):
        for key in ('body', 'text', 'url', 'filename', 'cookie', 'password'):
            value = self.value()
            value['records'][2][key] = 'Generated secret must not be exported'
            with self.subTest(key=key), self.assertRaises(ValueError):
                probe.validate_snapshot(value)

    def test_unknown_top_level_field_is_rejected(self):
        value = self.value()
        value['token'] = 'Generated secret'
        with self.assertRaises(ValueError):
            probe.validate_snapshot(value)

    def test_overflow_is_rejected(self):
        value = self.value()
        value['overflow'] = True
        with self.assertRaises(ValueError):
            probe.validate_snapshot(value)

    def test_invalid_boolean_is_rejected(self):
        value = self.value()
        value['records'][1]['inputIsArrayBuffer'] = 1
        with self.assertRaises(ValueError):
            probe.validate_snapshot(value)

    def test_oversized_counts_are_rejected(self):
        for row, key, value in ((1, 'inputBytes', 64 * 1024**2 + 1), (2, 'outputBytes', 256 * 1024**2 + 1), (2, 'filesCount', 8193)):
            observed = self.value()
            observed['records'][row][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                probe.validate_snapshot(observed)

    def test_boolean_does_not_become_count(self):
        observed = self.value()
        observed['records'][2]['filesCount'] = True
        with self.assertRaises(ValueError):
            probe.validate_snapshot(observed)

    def test_backwards_time_is_rejected(self):
        value = self.value()
        value['records'][-1]['epochMs'] = 999
        with self.assertRaises(ValueError):
            probe.validate_snapshot(value)

    def test_native_behavior_and_error_propagation_are_preserved_in_js(self):
        code = '''
          let received, terminated = 0;
          class Native {
            constructor(url) { if (url === 'bad') throw new TypeError('generated constructor failure'); this.listeners = {}; }
            addEventListener(name, callback) { this.listeners[name] = callback; }
            postMessage(...args) { received = args; return 42; }
            terminate() { terminated++; return 24; }
          }
          globalThis.Worker = Native;
        ''' + probe.EPUB_WORKER_OBSERVER_JS + '''
          const buffer = new ArrayBuffer(123), transfer = [buffer];
          const worker = new Worker('generated-url');
          if (!(worker instanceof Native) || worker.postMessage(buffer, transfer) !== 42 || received[0] !== buffer || received[1] !== transfer) throw new Error('Delegation changed');
          worker.listeners.message({data:{files:[['DO NOT EXPORT THIS NAME',new Uint8Array(77)]]}});
          worker.listeners.error({message:'DO NOT EXPORT THIS ERROR'});
          if (worker.terminate() !== 24 || terminated !== 1) throw new Error('Termination changed');
          let rejected = false;
          try { new Worker('bad'); } catch (error) { rejected = error instanceof TypeError; }
          if (!rejected) throw new Error('Constructor failure hidden');
          process.stdout.write(JSON.stringify(__readerEpubWorkerProbe.snapshot()));
        '''
        result = subprocess.run(['node', '--input-type=module', '-e', code], capture_output=True, encoding='utf-8', timeout=10, check=True)
        value = probe.validate_snapshot(json.loads(result.stdout))
        self.assertEqual([row['kind'] for row in value['records']], ['created', 'posted', 'message', 'error', 'terminated', 'constructorError'])
        self.assertEqual(value['records'][2]['outputBytes'], 77)
        self.assertNotIn('DO NOT EXPORT', result.stdout)

    def test_page_window_is_patched_instead_of_sandbox_shadow(self):
        code = '''
          class Native {
            constructor() { this.listeners = {}; }
            addEventListener(name, callback) { this.listeners[name] = callback; }
            postMessage() { return 77; }
            terminate() {}
          }
          globalThis.Worker = Native;
          const page = { Worker:Native, ArrayBuffer, Uint8Array };
          globalThis.window = page;
        ''' + probe.EPUB_WORKER_OBSERVER_JS + '''
          if (globalThis.Worker !== Native || page.Worker === Native) throw new Error('Wrong global patched');
          const actual = new page.Worker();
          if (actual.postMessage(new ArrayBuffer(99)) !== 77) throw new Error('Page delegation changed');
          actual.terminate();
          process.stdout.write(JSON.stringify(page.__readerEpubWorkerProbe.snapshot()));
        '''
        result = subprocess.run(['node', '--input-type=module', '-e', code], capture_output=True, encoding='utf-8', timeout=10, check=True)
        value = probe.validate_snapshot(json.loads(result.stdout))
        self.assertEqual([row['kind'] for row in value['records']], ['created', 'posted', 'terminated'])
        self.assertTrue(value['constructorStillInstalled'])

    def test_test_only_html_inserts_before_product_scripts(self):
        original = b'<!doctype html><html><head><script src="/static/generated.js"></script></head></html>'
        modified = probe.inject_test_observer(original)
        self.assertLess(modified.index(b'const Native'), modified.index(b'/static/generated.js'))
        self.assertEqual(modified.count(b'<head>'), 1)
        self.assertTrue(modified.endswith(b'</head></html>'))
        self.assertEqual(original.count(b'const Native'), 0)

    def test_invalid_or_previously_instrumented_html_is_rejected(self):
        for html in (b'<body>generated</body>', b'<head>reader-epub-worker-observations',
                     b'<head>' + bytes(2 * 1024**2), '<head>not bytes'):
            with self.subTest(kind=type(html).__name__), self.assertRaises(ValueError):
                probe.inject_test_observer(html)

    def test_generated_browser_selfcheck_uses_no_real_inputs(self):
        html = probe.generated_worker_selfcheck_html()
        self.assertIn(b'new Worker(url, {type:', html)
        self.assertIn(b'new ArrayBuffer(123)', html)
        self.assertIn(b'new Uint8Array(77)', html)
        self.assertNotIn(b'authorized.epub', html)

    def test_finite_dom_marker_contains_no_message_names_or_bytes(self):
        code = '''
          const nodes = new Map();
          globalThis.document = {
            head:{appendChild(node) {nodes.set(node.id,node);}},
            getElementById(id) {return nodes.get(id);},
            createElement() {return {setAttribute(key,value) {this[key]=value;}};}
          };
          class Native {
            constructor() {this.listeners={};}
            addEventListener(key,callback) {this.listeners[key]=callback;}
            postMessage() {}
            terminate() {}
          }
          globalThis.Worker=Native;
        ''' + probe.EPUB_WORKER_OBSERVER_JS + '''
          const worker=new Worker('DO NOT EXPORT URL');
          worker.postMessage(new ArrayBuffer(7));
          worker.listeners.message({data:{files:[['DO NOT EXPORT NAME',new Uint8Array(13)]]}});
          worker.terminate();
          process.stdout.write(document.getElementById('reader-epub-worker-observations').content);
        '''
        result = subprocess.run(['node', '--input-type=module', '-e', code], capture_output=True,
            encoding='utf-8', timeout=10, check=True)
        value = probe.validate_snapshot(json.loads(result.stdout))
        self.assertEqual(value['records'][2]['outputBytes'], 13)
        self.assertNotIn('DO NOT EXPORT', result.stdout)

    def test_dom_observation_failure_does_not_replace_native_behavior(self):
        code = '''
          globalThis.document = {head:{}, getElementById(){throw new Error('Generated DOM failure');}};
          let sent=0, stopped=0;
          class Native {
            addEventListener() {}
            postMessage() {sent++; return 81;}
            terminate() {stopped++; return 82;}
          }
          globalThis.Worker=Native;
        ''' + probe.EPUB_WORKER_OBSERVER_JS + '''
          const worker=new Worker();
          if(worker.postMessage(new ArrayBuffer(7))!==81 || worker.terminate()!==82 || sent!==1 || stopped!==1) throw new Error('Native behavior replaced');
          process.stdout.write(JSON.stringify(__readerEpubWorkerProbe.snapshot()));
        '''
        result = subprocess.run(['node', '--input-type=module', '-e', code], capture_output=True,
            encoding='utf-8', timeout=10, check=True)
        value = json.loads(result.stdout)
        self.assertTrue(value['publishFailed'])
        self.assertEqual([row['kind'] for row in value['records']], ['created', 'posted', 'terminated'])
        with self.assertRaises(ValueError):
            probe.validate_snapshot(value)

    def test_failed_or_non_boolean_dom_publish_flag_is_rejected(self):
        for flag in (True, 0, None, 'false'):
            value = self.value()
            value['publishFailed'] = flag
            with self.subTest(flag=flag), self.assertRaises(ValueError):
                probe.validate_snapshot(value)


if __name__ == '__main__':
    unittest.main()
