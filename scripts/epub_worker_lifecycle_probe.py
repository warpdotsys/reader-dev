"""Finite Worker timing observations for an authorized isolated UI test.

No constructor URL, message bytes, filenames, text, Cookie or credentials are
exported. A test-only HTML prefix installs native-delegating wrappers in the
page world and publishes numeric observations in a generated meta element.
This is not product code or a production-response transformation.
"""
import json

EPUB_WORKER_OBSERVER_JS = r"""(() => {
  // This bootstrap must be delivered as a real page script. Merely assigning
  // either globalThis or window inside Camoufox's isolated evaluator did not
  // intercept the product Worker in the recorded H and I diagnostics.
  const target = typeof window === 'object' ? window : globalThis;
  const Native = target.Worker;
  const BufferType = target.ArrayBuffer || ArrayBuffer;
  const BytesType = target.Uint8Array || Uint8Array;
  let installed = null;
  const records = [];
  let sequence = 0, overflow = false, publishFailed = false;
  const identities = new WeakMap();
  function snapshot() {
    return { supported: typeof Native === 'function', overflow, publishFailed,
      constructorStillInstalled: target.Worker === installed,
      records: records.map(row => ({ ...row })) };
  }
  function publish() {
    if (typeof document !== 'object' || !document.head) return;
    try {
      let node = document.getElementById('reader-epub-worker-observations');
      if (!node) {
        node = document.createElement('meta');
        node.id = 'reader-epub-worker-observations';
        document.head.appendChild(node);
      }
      node.setAttribute('content', JSON.stringify(snapshot()));
    } catch {
      // Instrumentation must not replace the native constructor/send/terminate
      // result. A later usable snapshot is rejected, not accepted as complete.
      publishFailed = true;
    }
  }
  function emit(kind, workerId, details = {}) {
    if (records.length >= 64) { overflow = true; publish(); return; }
    records.push({ kind, workerId, epochMs: Date.now(), ...details });
    publish();
  }
  Object.defineProperty(target, '__readerEpubWorkerProbe', { configurable: false,
    value: { snapshot } });
  if (typeof Native !== 'function') { publish(); return; }
  class ObservedWorker extends Native {
    constructor(...args) {
      const id = ++sequence;
      try { super(...args); }
      catch (error) { emit('constructorError', id); throw error; }
      identities.set(this, id);
      emit('created', id);
      this.addEventListener('message', event => {
        const files = event.data?.files;
        const valid = Array.isArray(files) && files.length <= 8192 &&
          files.every(entry => Array.isArray(entry) && entry[1] instanceof BytesType);
        // Only byte lengths are observed; no entry name or byte is inspected.
        emit('message', id, { filesReceived: valid,
          filesCount: valid ? files.length : 0,
          outputBytes: valid ? files.reduce((sum, entry) => sum + entry[1].byteLength, 0) : 0,
          errorReply: typeof event.data?.error === 'string' });
      });
      this.addEventListener('error', () => emit('error', id));
      this.addEventListener('messageerror', () => emit('messageerror', id));
    }
    postMessage(value) {
      emit('posted', identities.get(this), { inputIsArrayBuffer: value instanceof BufferType,
        inputBytes: value instanceof BufferType ? value.byteLength : 0 });
      return Reflect.apply(Native.prototype.postMessage, this, arguments);
    }
    terminate() {
      emit('terminated', identities.get(this));
      return Reflect.apply(Native.prototype.terminate, this, arguments);
    }
  }
  target.Worker = ObservedWorker;
  installed = ObservedWorker;
  publish();
})()"""


def inject_test_observer(html):
    """Insert only into the isolated fixture's bounded, public UI HTML."""
    if not isinstance(html, bytes) or len(html) > 2 * 1024**2:
        raise ValueError('InvalidFixtureHtmlSize')
    if b'<head>' not in html or b'reader-epub-worker-observations' in html:
        raise ValueError('UnexpectedFixtureHtmlHead')
    script = EPUB_WORKER_OBSERVER_JS.encode('utf-8')
    if b'</script' in script.lower():
        raise ValueError('UnsafeTestBootstrapDelimiter')
    return html.replace(b'<head>', b'<head><script>' + script + b'</script>', 1)


def generated_worker_selfcheck_html():
    """Exercise a real page module Worker, without a book or credentials."""
    return inject_test_observer(b'''<!doctype html><html><head></head><body>
      <script>
        const source = "onmessage=()=>postMessage({files:[['GENERATED',new Uint8Array(77)]]})";
        const url = URL.createObjectURL(new Blob([source], {type:'text/javascript'}));
        const worker = new Worker(url, {type:'module'});
        worker.onmessage = () => {
          worker.terminate(); URL.revokeObjectURL(url);
          const marker=document.createElement('meta');
          marker.id='reader-epub-worker-selfcheck'; marker.setAttribute('content','complete');
          document.head.appendChild(marker);
        };
        const input=new ArrayBuffer(123);
        worker.postMessage(input,[input]);
      </script></body></html>''')

EVENT_FIELDS = {'created': set(), 'constructorError': set(), 'error': set(),
    'messageerror': set(), 'terminated': set(),
    'posted': {'inputIsArrayBuffer', 'inputBytes'},
    'message': {'filesReceived', 'filesCount', 'outputBytes', 'errorReply'}}


def validate_snapshot(value):
    if not isinstance(value, dict) or set(value) != {'supported', 'overflow', 'publishFailed', 'constructorStillInstalled', 'records'}:
        raise ValueError('UnexpectedWorkerSnapshotFields')
    if type(value['supported']) is not bool or value['overflow'] is not False or value['publishFailed'] is not False or type(value['constructorStillInstalled']) is not bool:
        raise ValueError('WorkerObservationUnsupportedOrOverflowed')
    records = value['records']
    if not isinstance(records, list) or len(records) > 64:
        raise ValueError('WorkerObservationOversized')
    last_time = 0
    for row in records:
        if not isinstance(row, dict) or row.get('kind') not in EVENT_FIELDS or set(row) != {'kind', 'workerId', 'epochMs'} | EVENT_FIELDS[row['kind']]:
            raise ValueError('UnexpectedWorkerEventFields')
        if type(row['workerId']) is not int or not 1 <= row['workerId'] <= 64 or type(row['epochMs']) is not int or not last_time <= row['epochMs']:
            raise ValueError('InvalidWorkerIdentityOrTime')
        last_time = row['epochMs']
        for key in ('inputIsArrayBuffer', 'filesReceived', 'errorReply'):
            if key in row and type(row[key]) is not bool:
                raise ValueError('InvalidWorkerBoolean')
        for key, limit in (('inputBytes', 64 * 1024**2), ('outputBytes', 256 * 1024**2), ('filesCount', 8192)):
            if key in row and (type(row[key]) is not int or not 0 <= row[key] <= limit):
                raise ValueError('InvalidWorkerCount')
    # Make a detached finite JSON-only copy; no live MessageEvent or buffers.
    return json.loads(json.dumps(value, allow_nan=False))
