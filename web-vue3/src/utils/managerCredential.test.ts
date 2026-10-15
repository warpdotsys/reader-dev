import assert from 'node:assert/strict'
import { test } from 'node:test'
import { discardManagerCredential, managerCredentialScope, readManagerCredential, saveManagerCredential } from './managerCredential.ts'

function storage() {
  const values = new Map<string, string>()
  return { getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => { values.set(key, value) },
    removeItem: (key: string) => { values.delete(key) } }
}
const user = { accessToken: 'generated-token', username: 'generated-a' }

test('管理凭据绑定部署和账号，未登录不读，旧共享键不读取、不迁移、不删除', () => {
  const store = storage()
  const scope = managerCredentialScope(user, 'https://generated/')
  store.setItem('reader_secure_key', 'unknown-owner')
  assert.equal(readManagerCredential(scope, store), '')
  assert.ok(saveManagerCredential(scope, 'generated-secret', store))
  assert.equal(readManagerCredential(scope, store), 'generated-secret')
  assert.equal(readManagerCredential(managerCredentialScope({ ...user, username: 'generated-b' }, 'https://generated/'), store), '')
  assert.equal(readManagerCredential(managerCredentialScope(user, 'https://other/'), store), '')
  assert.equal(readManagerCredential(managerCredentialScope({ ...user, accessToken: '' }, 'https://generated/'), store), '')
  discardManagerCredential(store)
  assert.equal(readManagerCredential(scope, store), '')
  assert.equal(store.getItem('reader_secure_key'), 'unknown-owner')
})

test('格式损坏或浏览器存储不可用时关闭管理能力，不读取伪造持久角色', () => {
  const store = storage()
  const scope = managerCredentialScope(user, 'https://generated/')
  store.setItem('reader_manager_credential_v2', 'invalid-json')
  assert.equal(readManagerCredential(scope, store), '')
  const unavailable = { getItem: () => { throw new Error('blocked') }, setItem: () => { throw new Error('blocked') },
    removeItem: () => { throw new Error('blocked') } }
  assert.equal(readManagerCredential(scope, unavailable), '')
  assert.equal(saveManagerCredential(scope, 'generated-secret', unavailable), false)
  assert.doesNotThrow(() => discardManagerCredential(unavailable))
})
