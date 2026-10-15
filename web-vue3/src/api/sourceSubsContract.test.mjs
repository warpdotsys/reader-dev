import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { createContext, runInContext } from 'node:vm'
import test from 'node:test'
import ts from 'typescript'
import * as requestSession from './requestSession.ts'

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
  const store = { accessToken: '', username: '', isAdmin: false, defaultConfigMode: false, sessionRevision: 0 }
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
      if (name === '@/stores/user') return { useUserStore: () => store }
      if (name === './requestSession') return requestSession
      throw new Error(`Unexpected import: ${name}`)
    },
  })
  runInContext(compiled, context, { filename: 'sourceSubs.ts' })
  return { api: module.exports, localStorage, sessionStorage, transport, store }
}

function session(storage, username, store) {
  storage.setItem('reader_access_token', `token-for-${username}`)
  storage.setItem('reader_username', username)
  Object.assign(store, { accessToken: `token-for-${username}`, username, isAdmin: false, defaultConfigMode: false,
    sessionRevision: store.sessionRevision + 1 })
}

test('订阅离线镜像按账号及 default 命名空间隔离，旧全局键不可读取', () => {
  const { api, localStorage, sessionStorage, store } = contract()
  localStorage.setItem('reader_source_subs', JSON.stringify([{ url: 'https://old-global.example' }]))
  const oldOwn = JSON.stringify([{ url: 'https://unverified-old-cookie-owner.example' }])
  localStorage.setItem('reader_source_subs:alice:user', oldOwn)
  session(localStorage, 'alice', store)
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
  api.persistSourceSubs([{ url: 'https://alice.example', name: 'Alice' }])
  assert.equal(localStorage.getItem('reader_source_subs:alice:user'), oldOwn)

  localStorage.removeItem('reader_access_token')
  localStorage.removeItem('reader_username')
  session(sessionStorage, 'bob', store)
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
  api.persistSourceSubs([{ url: 'https://bob.example', name: 'Bob' }])

  sessionStorage.removeItem('reader_access_token')
  sessionStorage.removeItem('reader_username')
  session(localStorage, 'alice', store)
  assert.equal(api.loadSourceSubs()[0].url, 'https://alice.example')
  localStorage.setItem('reader_default_config_mode', '1')
  store.isAdmin = true
  store.defaultConfigMode = true
  localStorage.setItem('reader_source_subs:alice:default', JSON.stringify([{ url: 'https://unverified-old-default.example' }]))
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
  api.persistSourceSubs([{ url: 'https://new-system-mirror-must-not-be-faked.example' }])
  assert.equal(JSON.parse(localStorage.getItem('reader_source_subs:alice:default'))[0].url, 'https://unverified-old-default.example')
  localStorage.removeItem('reader_default_config_mode')
  store.defaultConfigMode = false
  assert.equal(api.loadSourceSubs()[0].url, 'https://alice.example')
})

test('服务器不可用或业务拒绝时，不会在本地伪造订阅变更', async () => {
  const { api, localStorage, transport, store } = contract()
  session(localStorage, 'alice', store)
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

test('订阅镜像跟随当前标签页身份，不跟随另一页共享存储；迟到响应不写新账号', async () => {
  const { api, localStorage, sessionStorage, transport, store } = contract()
  session(sessionStorage, 'alice', store)
  localStorage.setItem('reader_access_token', 'token-for-bob')
  localStorage.setItem('reader_username', 'bob')
  api.persistSourceSubs([{ url: 'https://alice.example' }])
  assert.equal(localStorage.getItem('reader_source_subs_v2:bob:user'), null)
  assert.equal(api.loadSourceSubs()[0].url, 'https://alice.example')
  let finish
  transport.get = () => new Promise(resolve => { finish = resolve })
  const pending = api.getSourceSubs()
  session(sessionStorage, 'bob', store)
  finish({ isSuccess: true, data: [{ url: 'https://late-alice.example' }] })
  await pending
  assert.equal(localStorage.getItem('reader_source_subs_v2:bob:user'), null)
  assert.deepEqual(Array.from(api.loadSourceSubs()), [])
})
