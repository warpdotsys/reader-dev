import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import vm from 'node:vm'
import ts from 'typescript'

const valid = { host: '*', type: 'trial', userMaxLimit: 15, expiredAt: 0, simpleWebExpiredAt: 1793000000000 }
const source = readFileSync(new URL('./license.ts', import.meta.url), 'utf8')
function api(response) {
  const calls = [], exports = {}
  const request = { get: async (...args) => { calls.push(['get', ...args]); return response },
    post: async (...args) => { calls.push(['post', ...args]); return response } }
  const output = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText
  vm.runInNewContext(output, { exports, require: name => { assert.equal(name, './request'); return request } })
  return { ...exports, calls }
}
const signal = new AbortController().signal

test('license status requires the real ReturnData shape and both expiry fields', async () => {
  assert.equal((await api({ isSuccess: true, data: { license: valid } }).loadLicense(signal)).expiredAt, 0)
  for (const response of [{ data: { license: valid } }, { isSuccess: false, data: { license: valid } },
    { isSuccess: true, data: { license: { ...valid, expiredAt: '0' } } },
    { isSuccess: true, data: { license: { ...valid, simpleWebExpiredAt: undefined } } }]) {
    await assert.rejects(api(response).loadLicense(signal))
  }
})

test('trial email never sends malformed address or assumes an HTTP success is a sent code', async () => {
  const a = api({ isSuccess: true, data: '' })
  for (const value of ['', 'bad', 'x@y\n.example', 'a'.repeat(255) + '@example.test']) await assert.rejects(a.sendTrialCode(value, signal))
  assert.equal(a.calls.length, 0)
  await a.sendTrialCode(' trial@example.test ', signal)
  assert.equal(a.calls[0][1], '/sendCodeToEmail')
  assert.equal(a.calls[0][2].email, 'trial@example.test')
  assert.equal(a.calls[0][3].signal, signal)
  await assert.rejects(api({ isSuccess: true, data: null }).sendTrialCode('trial@example.test', signal))
})

test('trial claim validates eight digits and a bounded returned key; it never imports implicitly', async () => {
  const a = api({ isSuccess: true, data: { key: 'generated-key' } })
  await assert.rejects(a.claimTrialLicense('trial@example.test', '1234', signal))
  assert.equal(a.calls.length, 0)
  assert.equal(await a.claimTrialLicense('trial@example.test', '12345678', signal), 'generated-key')
  assert.equal(a.calls.length, 1); assert.equal(a.calls[0][1], '/supplyLicense')
  await assert.rejects(api({ isSuccess: true, data: { key: 'x'.repeat(32769) } }).claimTrialLicense('trial@example.test', '12345678', signal))
})

test('license import sends one-use manager credential only in header and validates server reply', async () => {
  const a = api({ isSuccess: true, data: { license: valid } })
  await a.installLicense(' generated-key ', 'generated-manager-secret', signal)
  const [, path, body, options] = a.calls[0]
  assert.equal(path, '/importLicense'); assert.deepEqual(JSON.parse(JSON.stringify(body)), { content: 'generated-key' })
  assert.equal(options.headers['X-Reader-Secure-Key'], 'generated-manager-secret')
  assert.equal(options.signal, signal); assert.equal(options.params, undefined)
  await assert.rejects(a.installLicense('generated-key', 'bad\nheader', signal))
  assert.equal(a.calls.length, 1)
})

function panel(overrides = {}) {
  const source = readFileSync(new URL('../components/LicensePanel.vue', import.meta.url), 'utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1]
  let watched, unmounted, mounted
  const store = { sessionRevision: 0 }
  const methods = { loadLicense: async () => valid, sendTrialCode: async () => {}, claimTrialLicense: async () => 'generated-key', installLicense: async () => valid, ...overrides }
  const context = { AbortController, Error, window: { setTimeout, clearTimeout }, require(name) {
    if (name === 'vue') return { ref: value => ({ value }), watch: (_get, fn) => { watched = fn }, onMounted: fn => { mounted = fn }, onBeforeUnmount: fn => { unmounted = fn } }
    if (name === '@/stores/user') return { useUserStore: () => store }
    if (name === '@/api/license') return methods
    throw new Error(name)
  } }
  const code = ts.transpileModule(source + '\nglobalThis.fixture = { run, email, code, content, managerKey, message, busy, failed, license };', {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText
  vm.runInNewContext(code, { ...context, exports: {}, get fixture() { return context.fixture }, set fixture(value) { context.fixture = value } })
  return { ...context.fixture, switchSession() { store.sessionRevision++; watched() }, unmount() { unmounted() }, mount() { mounted() } }
}

test('license panel displays server failures, clears temporary manager password and does not fake installed state', async () => {
  const f = panel({ installLicense: async () => { throw new Error('请输入管理密码') } })
  f.managerKey.value = 'wrong-generated'; f.content.value = 'generated-key'
  await f.run('install')
  assert.equal(f.failed.value, true); assert.equal(f.message.value, '请输入管理密码')
  assert.equal(f.managerKey.value, ''); assert.equal(f.license.value, null); assert.equal(f.content.value, 'generated-key')
  f.unmount()
})

test('late trial claim cannot leak a key across sessions and aborts its old request', async () => {
  let resolve, seenSignal
  const f = panel({ claimTrialLicense: (_email, _code, signal) => { seenSignal = signal; return new Promise(done => { resolve = done }) } })
  f.email.value = 'trial@example.test'; f.code.value = '12345678'
  const pending = f.run('claim')
  f.switchSession(); resolve('generated-late-key'); await pending
  assert.equal(seenSignal.aborted, true); assert.equal(f.content.value, ''); assert.equal(f.email.value, '')
  assert.doesNotMatch(f.message.value, /已领取/); f.unmount()
})

test('license panel serializes clicks and clears transient secrets on unmount', async () => {
  let resolve, calls = 0, seenSignal
  const f = panel({ sendTrialCode: (_email, signal) => { calls++; seenSignal = signal; return new Promise(done => { resolve = done }) } })
  f.email.value = 'trial@example.test'; f.managerKey.value = 'generated-password'
  const pending = f.run('send'); await f.run('send')
  assert.equal(calls, 1); f.unmount(); resolve(); await pending
  assert.equal(seenSignal.aborted, true); assert.equal(f.email.value, ''); assert.equal(f.managerKey.value, '')
  assert.equal(f.message.value, '')
})
