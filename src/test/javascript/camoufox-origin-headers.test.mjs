// Runs the production listener's exact JS, not an actual browser or TLS test.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import vm from 'node:vm';

const worker = readFileSync(new URL('../../main/resources/camoufox/worker.py', import.meta.url), 'utf8');
const marker = 'ORIGIN_HEADER_LISTENER = """';
assert.equal(worker.split(marker).length, 2);
const start = worker.indexOf(marker) + marker.length;
const source = worker.slice(start, worker.indexOf('"""', start));
const origin = 'https://generated.invalid:443/book';
function listener(url = origin, policyOverrides = {}) {
  let rewrite, ready;
  const policy = { url, bootstrapUrl: 'http://reader-header-generated.invalid/ready', nonce: 'GENERATED_NONCE',
    headers: { authorization: 'Bearer GENERATED', 'x-trace': 'GENERATED_TRACE', accept: 'GENERATED_ACCEPT',
      'content-type': 'application/json; charset=UTF-8' } };
  Object.assign(policy, policyOverrides);
  const browser = { webRequest: { onBeforeSendHeaders: { addListener(fn, filter, options) {
    assert.deepEqual(Array.from(filter.urls), ['http://*/*', 'https://*/*']);
    assert.deepEqual(Array.from(options), ['blocking', 'requestHeaders']);
    rewrite = fn;
  } } }, runtime: { id: 'GENERATED_ADDON', onMessage: { addListener(fn) { ready = fn; } } } };
  vm.runInNewContext(source, { policy, browser, URL });
  return { rewrite: (url, headers = [], method = 'GET') => JSON.parse(JSON.stringify(rewrite({ url, requestHeaders: headers, method }))), ready, policy };
}

test('native form default changes only initial POST type without freezing transport headers', () => {
  const subject = listener(origin + '#generated-fragment', { headers: {}, defaultFormPost: true });
  const actual = [{ name: 'Host', value: 'generated.invalid' }, { name: 'Cookie', value: 'generated=ONLY' },
    { name: 'Content-Type', value: 'application/octet-stream' }];
  assert.deepEqual(subject.rewrite(origin, actual, 'POST').requestHeaders, [
    actual[0], actual[1], { name: 'Content-Type', value: 'application/x-www-form-urlencoded; charset=UTF-8' }]);
  assert.equal(actual[2].value, 'application/octet-stream');
});

test('GET does not consume the pending initial form default', () => {
  const subject = listener(origin, { headers: {}, defaultFormPost: true });
  assert.deepEqual(subject.rewrite(origin, [], 'GET').requestHeaders, []);
  assert.equal(subject.rewrite(origin, [], 'POST').requestHeaders[0].value,
    'application/x-www-form-urlencoded; charset=UTF-8');
});

test('unrelated or cross-origin POST does not consume the initial form default', () => {
  const subject = listener(origin, { headers: {}, defaultFormPost: true });
  assert.deepEqual(subject.rewrite('https://generated.invalid/other', [], 'POST').requestHeaders, []);
  assert.deepEqual(subject.rewrite('https://other.invalid/book', [], 'POST').requestHeaders, []);
  assert.equal(subject.rewrite(origin, [], 'POST').requestHeaders.length, 1);
});

test('later website POST keeps its own type rather than reapplying form defaults', () => {
  const subject = listener(origin, { headers: {}, defaultFormPost: true });
  subject.rewrite(origin, [], 'POST');
  const website = [{ name: 'Content-Type', value: 'application/json' }];
  assert.deepEqual(subject.rewrite(origin, website, 'POST').requestHeaders, website);
});

test('native redirects keep POST metadata and never add it to redirected GET', () => {
  const subject = listener(origin, { headers: {}, defaultFormPost: true });
  const initial = subject.rewrite(origin, [], 'POST').requestHeaders;
  assert.deepEqual(subject.rewrite('https://other.invalid/end', initial, 'POST').requestHeaders, initial);
  assert.deepEqual(subject.rewrite('https://other.invalid/end', [], 'GET').requestHeaders, []);
});

for (const [name, url, same] of [
  ['exact origin', 'https://generated.invalid/next', true],
  ['uppercase host and default port', 'https://GENERATED.invalid:443/next', true],
  ['path is not an origin boundary', 'https://generated.invalid/other/next', true],
  ['another port', 'https://generated.invalid:444/next', false],
  ['another scheme', 'http://generated.invalid/next', false],
  ['subdomain', 'https://child.generated.invalid/next', false],
  ['host suffix is not origin', 'https://generated.invalid.attacker.invalid/next', false],
  ['unrelated host', 'https://other.invalid/next', false],
]) test(name, () => {
  const result = listener().rewrite(url, [{ name: 'Cookie', value: 'GENERATED_NATIVE_COOKIE' }]);
  assert.equal(result.requestHeaders.some(header => header.name === 'authorization'), same);
  assert.equal(result.requestHeaders.find(header => header.name === 'Cookie').value, 'GENERATED_NATIVE_COOKIE');
});

test('same-origin rule values replace case variants once without mutating browser input', () => {
  const input = [{ name: 'Authorization', value: 'old' }, { name: 'AUTHORIZATION', value: 'old2' }];
  assert.deepEqual(listener().rewrite(origin, input).requestHeaders.filter(h => h.name.toLowerCase() === 'authorization'),
    [{ name: 'authorization', value: 'Bearer GENERATED' }]);
  assert.deepEqual(input, [{ name: 'Authorization', value: 'old' }, { name: 'AUTHORIZATION', value: 'old2' }]);
});
test('cross-origin inherited rule values are removed including surrounding whitespace', () => {
  assert.deepEqual(listener().rewrite('https://other.invalid/end', [
    { name: 'AUTHORIZATION', value: ' Bearer GENERATED ' }, { name: 'x-trace', value: 'GENERATED_TRACE' },
  ]).requestHeaders, []);
});
test('other-origin script credentials and native Accept are not falsely removed', () => {
  const input = [{ name: 'Authorization', value: 'Bearer OTHER_GENERATED' }, { name: 'Accept', value: '*/*' }];
  assert.deepEqual(listener().rewrite('https://other.invalid/end', input).requestHeaders, input);
});
test('same origin is normalized for IDN', () => {
  assert.equal(listener('https://书.invalid/a').rewrite('https://xn--1jq.invalid/b').requestHeaders.length, 4);
});
test('IPv6 default ports are normalized but other ports remain separate', () => {
  assert.equal(listener('http://[::1]:80/a').rewrite('http://[::1]/b').requestHeaders.length, 4);
  assert.equal(listener('http://[::1]:80/a').rewrite('http://[::1]:81/b').requestHeaders.length, 0);
});
test('unrelated sender cannot announce policy readiness', () => {
  const subject = listener();
  assert.equal(subject.ready(subject.policy.nonce, { id: 'OTHER', url: subject.policy.bootstrapUrl }), undefined);
});
test('a page outside the private bootstrap cannot announce readiness', () => {
  const subject = listener();
  assert.equal(subject.ready(subject.policy.nonce, { id: 'GENERATED_ADDON', url: origin }), undefined);
});
test('wrong nonce cannot announce readiness', () => {
  const subject = listener();
  assert.equal(subject.ready('WRONG', { id: 'GENERATED_ADDON', url: subject.policy.bootstrapUrl }), undefined);
});
test('private content-script handshake proves listener registration without exposing headers', async () => {
  const subject = listener();
  assert.equal(await subject.ready(subject.policy.nonce, { id: 'GENERATED_ADDON', url: subject.policy.bootstrapUrl }),
    subject.policy.nonce);
});
test('cross-origin preserved POST retains native body media type but no origin credential', () => {
  const input = [{ name: 'Content-Type', value: 'application/json; charset=UTF-8' },
    { name: 'Authorization', value: 'Bearer GENERATED' }, { name: 'X-Trace', value: 'GENERATED_TRACE' }];
  assert.deepEqual(listener().rewrite('https://other.invalid/end', input, 'POST').requestHeaders,
    [{ name: 'Content-Type', value: 'application/json; charset=UTF-8' }]);
});
test('cross-origin GET does not reintroduce rule body media type after a 303', () => {
  assert.deepEqual(listener().rewrite('https://other.invalid/end',
    [{ name: 'Content-Type', value: 'application/json; charset=UTF-8' }], 'GET').requestHeaders, []);
});
