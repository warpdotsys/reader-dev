import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { createContext, runInContext } from 'node:vm'
import test from 'node:test'
import ts from 'typescript'

function storage() {
  const values = new Map()
  return {
    values,
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, String(value)),
    removeItem: (key) => values.delete(key),
  }
}

function contract() {
  const localStorage = storage()
  const sessionStorage = storage()
  const transport = {
    get: async () => { throw new Error('offline') },
    post: async () => { throw new Error('offline') },
  }
  const source = readFileSync(new URL('./sourceSubs.ts', import.meta.url), 'utf8')
  const compiled = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText
  const module = { exports: {} }
  const context = createContext({
    module,
    exports: module.exports,
    localStorage,
    sessionStorage,
    encodeURIComponent,
    require(name) {
      if (name === './request') {
        return { get: (...args) => transport.get(...args), post: (...args) => transport.post(...args) }
      }
      if (name === './backendFlag') return { onBackendReachable() {} }
      throw new Error(`Unexpected import: ${name}`)
    },
  })
  runInContext(compiled, context, { filename: 'sourceSubs.ts' })
  return { api: module.exports, localStorage, sessionStorage, transport }
}

function session(store, username) {
  store.setItem('reader_access_token', `token-for-${username}`)
  store.setItem('reader_username', username)
}

test('订阅离线镜像按账号及 default 命名空间隔离，旧全局键不可读取', () => {
  const { api, localStorage, sessionStorage } = contract()
  localStorage.setItem('reader_source_subs', JSON.stringify([{ url: 'https://old-global.example' }]))
  session(localStorage, 'alice')
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
  api.persistSourceSubs([{ url: 'https://alice.example', name: 'Alice' }])

  localStorage.removeItem('reader_access_token')
  localStorage.removeItem('reader_username')
  session(sessionStorage, 'bob')
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
  api.persistSourceSubs([{ url: 'https://bob.example', name: 'Bob' }])

  sessionStorage.removeItem('reader_access_token')
  sessionStorage.removeItem('reader_username')
  session(localStorage, 'alice')
  assert.equal(api.loadSourceSubs()[0].url, 'https://alice.example')
  localStorage.setItem('reader_default_config_mode', '1')
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
  localStorage.removeItem('reader_default_config_mode')
  assert.equal(api.loadSourceSubs()[0].url, 'https://alice.example')
})

test('服务器不可用或业务拒绝时，不会在本地伪造订阅变更', async () => {
  const { api, localStorage, transport } = contract()
  session(localStorage, 'alice')
  const original = [{ url: 'https://one.example', name: 'One', enabled: true }]
  api.persistSourceSubs(original)

  assert.equal((await api.saveSourceSub('https://two.example', 'Two')).isSuccess, false)
  assert.equal((await api.deleteSourceSub(original[0].url)).isSuccess, false)
  assert.equal((await api.deleteSourceSubs([original[0].url])).isSuccess, false)
  assert.equal((await api.setSourceSubEnabled(original[0].url, false)).isSuccess, false)
  assert.equal(api.loadSourceSubs().length, 1)
  assert.equal(api.loadSourceSubs()[0].enabled, true)

  transport.post = async () => ({ isSuccess: false, errorMsg: '安全策略拒绝', data: null })
  assert.equal((await api.saveSourceSub('https://blocked.example', 'Blocked')).isSuccess, false)
  assert.equal((await api.setSourceSubEnabled(original[0].url, false)).isSuccess, false)
  assert.equal(api.loadSourceSubs().length, 1)
  assert.equal(api.loadSourceSubs()[0].enabled, true)
})
