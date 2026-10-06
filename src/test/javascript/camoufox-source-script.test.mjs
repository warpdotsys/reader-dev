// Real JavaScript execution of the worker's exact helpers, not a browser test.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import vm from 'node:vm';
import { setTimeout as delay } from 'node:timers/promises';

const worker = readFileSync(new URL('../../main/resources/camoufox/worker.py', import.meta.url), 'utf8');
function helper(name) {
  const marker = `${name} = """`;
  assert.equal(worker.split(marker).length, 2, 'Exactly one actual worker helper is required');
  const start = worker.indexOf(marker) + marker.length;
  const end = worker.indexOf('"""', start);
  assert.ok(end > start);
  return worker.slice(start, end);
}
const scripts = Object.fromEntries(['START', 'READ', 'CLEAR', 'DOCUMENT'].map(operation =>
  [operation, helper(`SOURCE_SCRIPT_${operation}`)]));

function page(source, key = '__generated_test_state') {
  const context = vm.createContext({ setTimeout, clearTimeout, document: { generated: true } });
  const functions = Object.fromEntries(Object.entries(scripts).map(([operation, script]) =>
    [operation, vm.runInContext(`(${script})`, context)]));
  const holder = functions.START({ source, key });
  assert.equal(Object.getPrototypeOf(holder), null, 'Opaque holder must not adopt a page thenable');
  assert.equal(holder.then, undefined, 'Start must not return a pending Promise');
  return {
    context,
    read: () => JSON.parse(JSON.stringify(functions.READ(key))),
    clear: () => functions.CLEAR(key),
    documentObservation: () => functions.DOCUMENT(holder),
  };
}

async function settled(subject) {
  // This bounds the language fixture, not the production Python/parent watchdog.
  const deadline = Date.now() + 1000;
  let result;
  while ((result = subject.read()).status === 'pending') {
    assert.ok(Date.now() < deadline, 'Generated result did not settle');
    await delay(5);
  }
  return result;
}

const values = [
  ["'generated-string'", 'generated-string'],
  ["''", ''],
  ['({answer:42,ready:true})', '{"answer":42,"ready":true}'],
  ["['alpha',7]", '["alpha",7]'],
  ['42', '42'],
  ['false', 'false'],
  ['null', ''],
  ['undefined', ''],
];
for (const [expression, expected] of values) {
  test(`synchronous generated value: ${expression}`, async () => {
    assert.deepEqual(await settled(page(expression)), { status: 'done', body: expected });
  });
}
for (const [expression, expected] of values.slice(0, 5)) {
  test(`resolved generated Promise: ${expression}`, async () => {
    assert.deepEqual(await settled(page(`Promise.resolve(${expression})`)),
      { status: 'done', body: expected });
  });
}

test('delayed Promise serializes its final value', async () => {
  const subject = page("new Promise(resolve => setTimeout(() => resolve({ready:true}), 20))");
  assert.deepEqual(subject.read(), { status: 'pending' });
  assert.deepEqual(await settled(subject), { status: 'done', body: '{"ready":true}' });
});
test('thenable is adopted exactly once', async () => {
  const subject = page("({then(resolve){ globalThis.generatedCalls = (globalThis.generatedCalls || 0) + 1; resolve('adopted'); }})");
  assert.deepEqual(await settled(subject), { status: 'done', body: 'adopted' });
  assert.equal(subject.context.generatedCalls, 1);
});
test('rejected value is never copied into state or protocol', async () => {
  const subject = page("Promise.reject(new Error('generated-secret-do-not-copy'))");
  assert.deepEqual(await settled(subject), { status: 'rejected' });
  assert.equal(JSON.stringify(subject.context).includes('generated-secret-do-not-copy'), false);
});
test('synchronous exception is a sanitized rejection', () => {
  assert.deepEqual(page("throw new Error('generated-secret-do-not-copy')").read(), { status: 'rejected' });
});
test('circular result is rejected instead of returning partial JSON', async () => {
  assert.deepEqual(await settled(page('(()=>{ const a = {}; a.self = a; return a; })()')), { status: 'rejected' });
});
test('never-settling Promise returns control to the Python budget owner', () => {
  const subject = page('new Promise(()=>{})');
  assert.deepEqual(subject.read(), { status: 'pending' });
  subject.clear();
  assert.equal(subject.read(), null);
});
test('polling does not replay mutating source and transient state is removable', async () => {
  const subject = page("(()=>{ globalThis.generatedRuns = (globalThis.generatedRuns || 0) + 1; return new Promise(resolve => setTimeout(() => resolve(globalThis.generatedRuns), 10)); })()");
  assert.deepEqual(subject.read(), { status: 'pending' });
  assert.deepEqual(await settled(subject), { status: 'done', body: '1' });
  assert.equal(subject.context.generatedRuns, 1);
  assert.equal(vm.runInContext("Object.keys(globalThis).some(key => key.startsWith('__generated_test_state'))", subject.context), false);
  subject.clear();
  assert.equal(subject.read(), null);
});
test('separate generated contexts do not share result state', async () => {
  assert.equal((await settled(page("'first'"))).body, 'first');
  assert.equal((await settled(page("'second'"))).body, 'second');
});
test('JSON-undefined result preserves the existing empty-string format', async () => {
  assert.deepEqual(await settled(page('(()=>{})')), { status: 'done', body: '' });
});
test('non-JSON BigInt is rejected without copying its value', async () => {
  assert.deepEqual(await settled(page('42n')), { status: 'rejected' });
});
test('nested Promise resolution retains null semantics', async () => {
  assert.deepEqual(await settled(page('Promise.resolve(Promise.resolve(null))')),
    { status: 'done', body: '' });
});
test('throwing then getter is a sanitized rejection', async () => {
  assert.deepEqual(await settled(page("({get then(){throw new Error('generated-secret-do-not-copy');}})")),
    { status: 'rejected' });
});
test('native adoption reads a mutating then getter only once', async () => {
  const subject = page("({get then(){globalThis.generatedGetterReads = (globalThis.generatedGetterReads || 0) + 1; return resolve => resolve('once');}})");
  assert.deepEqual(await settled(subject), { status: 'done', body: 'once' });
  assert.equal(subject.context.generatedGetterReads, 1);
});

test('document holder captures the reference before source execution', async () => {
  // A VM global swap is not browser navigation; this checks helper semantics only.
  const subject = page("globalThis.document = {generated:'replacement'}; 'generated-result'");
  assert.equal(subject.documentObservation(), false);
  assert.deepEqual(await settled(subject), { status: 'done', body: 'generated-result' });
});
test('state deletion leaves document identity without another page-global marker', () => {
  const subject = page("delete globalThis.__generated_test_state; new Promise(()=>{})");
  assert.equal(subject.read(), null);
  assert.equal(subject.documentObservation(), true);
  assert.deepEqual(Object.keys(subject.context).sort(), ['clearTimeout', 'document', 'setTimeout']);
});
test('hostile prototype then does not make the document holder await a rule', async () => {
  const subject = page("Object.prototype.then = () => {}; 'generated-result'");
  assert.equal(subject.documentObservation(), true);
  assert.deepEqual(await settled(subject), { status: 'done', body: 'generated-result' });
});
