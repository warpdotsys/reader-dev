// Execute the exact probe script with generated DOM/clock doubles, not a browser.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import vm from 'node:vm';

const source = readFileSync(new URL('../../../scripts/probe-public-metadata.py', import.meta.url), 'utf8');
const marker = 'METADATA_DOM_SCRIPT = """';
assert.equal(source.split(marker).length, 2);
const start = source.indexOf(marker) + marker.length;
const end = source.indexOf('"""', start);
assert.ok(end > start);
const script = source.slice(start, end);
const selectors = ['#bookName', '.book-info-top .book-meta .author', '#bookImg img'];

async function execute({ readyAt = 0, missing, blank } = {}) {
  let now = 0;
  let polls = 0;
  const timers = [];
  const html = '<html><body>Generated original page; no inserted metadata.</body></html>';
  const document = {
    get cookie() { throw new Error('Cookie access is forbidden in this probe'); },
    set cookie(_) { throw new Error('Cookie writes are forbidden in this probe'); },
    documentElement: Object.freeze({ outerHTML: html }),
    querySelector(selector) {
      assert.ok(selectors.includes(selector), 'Only fixed public metadata selectors are allowed');
      if (now < readyAt || selector === missing) return null;
      return Object.freeze({
        textContent: selector === blank ? ' \n ' : 'Generated field',
        getAttribute(name) {
          assert.equal(name, 'src');
          return selector === blank ? '' : 'https://bookcover.yuewen.com/generated';
        },
      });
    },
  };
  const context = vm.createContext({
    document,
    performance: { now: () => now },
    setTimeout(callback, milliseconds) {
      assert.equal(milliseconds, 100);
      timers.push(callback);
    },
  });
  const result = vm.runInContext(script, context, { timeout: 1000 });
  assert.equal(typeof result.then, 'function', 'The exact script must return a Promise');
  while (timers.length) {
    now += 100;
    polls++;
    assert.ok(polls <= 80, 'Generated clock must stop at the 8-second budget');
    timers.shift()();
  }
  assert.equal(await result, html, 'The script must return the original HTML without synthesizing values');
  return { now, polls };
}

test('already-present fields return original HTML without polling', async () => {
  assert.deepEqual(await execute(), { now: 0, polls: 0 });
});

test('delayed fields are awaited rather than replacing metadata', async () => {
  assert.deepEqual(await execute({ readyAt: 300 }), { now: 300, polls: 3 });
});

test('an empty generated page returns its unchanged HTML at the fixed deadline', async () => {
  assert.deepEqual(await execute({ readyAt: Infinity }), { now: 8000, polls: 80 });
});

for (const missing of selectors) {
  test(`missing required selector remains bounded: ${missing}`, async () => {
    assert.deepEqual(await execute({ missing }), { now: 8000, polls: 80 });
  });
}

test('blank text does not count as metadata readiness', async () => {
  assert.deepEqual(await execute({ blank: selectors[0] }), { now: 8000, polls: 80 });
});

test('an empty cover attribute does not count as metadata readiness', async () => {
  assert.deepEqual(await execute({ blank: selectors[2] }), { now: 8000, polls: 80 });
});
