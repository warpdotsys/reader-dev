import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import vm from 'node:vm'
import ts from 'typescript'

const code = ts.transpileModule(readFileSync(new URL('./opdsConnection.ts', import.meta.url), 'utf8'), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
}).outputText
const ns = 'http://www.w3.org/2005/Atom'
// This is a generated DOM contract double. Actual browser XML parsing is a separate acceptance gate.
function fixture({ response = new Response('<feed/>', { headers: { 'content-type': 'application/atom+xml' } }),
  rootName = 'feed', namespace = ns, fieldNames = ['id', 'title', 'updated'], parseError = false } = {}) {
  const calls = []
  const exports = {}
  class Parser {
    parseFromString() { return { documentElement: { localName: rootName, namespaceURI: namespace,
      children: fieldNames.map(localName => ({ localName, namespaceURI: ns, textContent: 'generated' })) },
    getElementsByTagName: () => parseError ? [{}] : [] } }
  }
  vm.runInNewContext(code, { exports, URL, TextEncoder, TextDecoder, DOMParser: Parser, btoa,
    fetch: async (...args) => { calls.push(args); return response } })
  const signal = new AbortController().signal
  return { ...exports, calls, probe: (origin = 'http://127.0.0.1:18999', user = 'generated', pass = 'GeneratedOnly-2026') =>
    exports.probeOpdsCatalog(origin, user, pass, signal), signal }
}

test('OPDS catalog URL never retains existing token query, fragment or path', () => {
  const f = fixture()
  assert.equal(f.opdsCatalogUrl('https://reader.invalid/reader3?a=generated#secret'), 'https://reader.invalid/reader3/opds/catalog')
  assert.throws(() => f.opdsCatalogUrl('https://user:pass@reader.invalid'), /地址无效/)
})

test('OPDS displayed and probed catalog points to the registered backend handler', () => {
  const backend = readFileSync(new URL('../../../src/main/java/com/htmake/reader/api/YueduApi.kt', import.meta.url), 'utf8')
  const path = new URL(fixture().opdsCatalogUrl('http://127.0.0.1:18999')).pathname
  assert.ok(backend.includes(`router.get("${path}").coroutineHandlerWithoutRes { opdsController.catalog(it) }`))
})

test('OPDS password preservation requires the same previously enabled account', () => {
  const f = fixture(), previous = { enabled: true, username: 'generated', passwordSet: true }
  assert.equal(f.validateOpdsAccount('generated', '', previous), '')
  assert.notEqual(f.validateOpdsAccount('changed', '', previous), '')
  assert.notEqual(f.validateOpdsAccount('generated', '', { ...previous, passwordSet: false }), '')
  assert.equal(f.validateOpdsAccount('', '', previous), '')
  for (const value of ['short', 'x'.repeat(129), 'long-generated\nsecret', 'long-generated\u0085secret',
    'long-generated\u009fsecret']) assert.notEqual(f.validateOpdsAccount('generated', value, previous), '')
  for (const value of ['a:b', '../', '中文', 'has space']) assert.notEqual(f.validateOpdsAccount(value, 'GeneratedOnly-2026', previous), '')
})

test('OPDS probe sends UTF-8 Basic only, no cookies, redirects or token URL', async () => {
  const f = fixture()
  await f.probe(undefined, 'generated', '生成密码-GeneratedOnly')
  const [url, init] = f.calls[0]
  assert.equal(url, 'http://127.0.0.1:18999/reader3/opds/catalog')
  assert.equal(Buffer.from(init.headers.Authorization.slice(6), 'base64').toString('utf8'), 'generated:生成密码-GeneratedOnly')
  assert.equal(init.credentials, 'omit'); assert.equal(init.redirect, 'error'); assert.equal(init.cache, 'no-store')
  assert.equal(init.signal, f.signal)
})

test('OPDS probe refuses non-loopback HTTP or invalid credentials before sending', async () => {
  const f = fixture()
  await assert.rejects(f.probe('http://reader.invalid'), /HTTPS/)
  await assert.rejects(f.probe(undefined, '', 'GeneratedOnly-2026'), /账号/)
  assert.equal(f.calls.length, 0)
})

test('OPDS successful HTML login response is not a valid catalog', async () => {
  const f = fixture({ response: new Response('<html>login</html>', { headers: { 'content-type': 'text/html' } }) })
  await assert.rejects(f.probe(), /未返回 OPDS/)
})

test('OPDS authorization errors cannot fall back to Reader authentication', async () => {
  const f = fixture({ response: new Response('denied', { status: 401 }) })
  await assert.rejects(f.probe(), /认证未通过/)
  assert.equal(f.calls.length, 1)
})

test('OPDS rejects malformed DOM, wrong XML namespace and missing required feed metadata', async () => {
  for (const changes of [{ parseError: true }, { rootName: 'html' }, { namespace: 'urn:wrong' }, { fieldNames: ['title'] }]) {
    await assert.rejects(fixture(changes).probe(), /有效的 OPDS/)
  }
})

test('OPDS rejects XML declarations, excessive bodies and invalid UTF-8', async () => {
  for (const [body, message] of [['<!DOCTYPE feed><feed/>', /XML 声明/],
    ['x'.repeat(1024 * 1024 + 1), /大小上限/], [new Uint8Array([0xff]), /encoded data|encoding/]]) {
    const f = fixture({ response: new Response(body, { headers: { 'content-type': 'application/atom+xml' } }) })
    await assert.rejects(f.probe(), message)
  }
})
