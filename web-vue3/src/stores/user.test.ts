import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createPinia, setActivePinia } from 'pinia'
import { useUserStore } from './user.ts'
import { captureRequestSession, isRequestSessionCurrent } from '../api/requestSession.ts'

function freshStore() {
  const storage = () => {
    const values = new Map<string, string>()
    return { getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => { values.set(key, value) },
      removeItem: (key: string) => { values.delete(key) } }
  }
  Object.defineProperty(globalThis, 'localStorage', { value: storage(), configurable: true })
  Object.defineProperty(globalThis, 'sessionStorage', { value: storage(), configurable: true })
  setActivePinia(createPinia())
  return useUserStore()
}

test('实际用户 store 的退出/同 token 重登会使旧快照失效，并清除管理员状态', () => {
  const store = freshStore()
  store.setSession('generated-token', 'generated-user', true)
  store.updateAdminStatus(true)
  const snapshot = captureRequestSession(store)
  store.clear()
  assert.equal(store.isAdmin, false)
  assert.equal(store.defaultConfigMode, false)
  store.setSession('generated-token', 'generated-user', true)
  assert.equal(isRequestSessionCurrent(snapshot, store), false)
})

test('管理员探测更新保留标签页登录和命名空间，不无故更新会话代数', () => {
  const store = freshStore()
  store.setSession('generated-token', 'generated-user', false)
  const snapshot = captureRequestSession(store)
  store.updateAdminStatus(false)
  assert.ok(isRequestSessionCurrent(snapshot, store))
  store.updateAdminStatus(true)
  assert.ok(isRequestSessionCurrent(snapshot, store))
  store.toggleDefaultConfigMode()
  const configured = captureRequestSession(store)
  store.updateAdminStatus(true)
  assert.ok(isRequestSessionCurrent(configured, store))
  assert.equal(store.defaultConfigMode, true)
  assert.equal(localStorage.getItem('reader_access_token'), null)
  assert.equal(sessionStorage.getItem('reader_access_token'), 'generated-token')
  assert.equal(sessionStorage.getItem('reader_is_admin'), null)
  assert.equal(localStorage.getItem('reader_remember'), '0')
  assert.equal(sessionStorage.getItem('reader_default_config_mode'), '1')
  store.updateAdminStatus(false)
  assert.equal(store.defaultConfigMode, false)
  assert.equal(isRequestSessionCurrent(configured, store), false)
})

test('旧持久角色标志不能授权系统配置；退出与重新登录清除新管理凭据', () => {
  freshStore()
  localStorage.setItem('reader_access_token', 'generated-token')
  localStorage.setItem('reader_username', 'generated-user')
  localStorage.setItem('reader_is_admin', '1')
  localStorage.setItem('reader_default_config_mode', '1')
  setActivePinia(createPinia())
  const store = useUserStore()
  assert.equal(store.isAdmin, false)
  assert.equal(store.defaultConfigMode, false)
  store.toggleDefaultConfigMode()
  assert.equal(store.defaultConfigMode, false)
  sessionStorage.setItem('reader_manager_credential_v2', 'generated-private-fixture')
  store.clear()
  assert.equal(sessionStorage.getItem('reader_manager_credential_v2'), null)
  sessionStorage.setItem('reader_manager_credential_v2', 'generated-private-fixture')
  store.setSession('generated-token', 'generated-user', false)
  assert.equal(sessionStorage.getItem('reader_manager_credential_v2'), null)
  assert.equal(store.isAdmin, false)
})

test('本标签临时身份优先完整读取，另一标签的记住我不被登录/退出删除', () => {
  const store = freshStore()
  store.setSession('token-a', 'alice', false)
  localStorage.setItem('reader_access_token', 'token-b')
  localStorage.setItem('reader_username', 'bob')
  localStorage.setItem('reader_default_config_mode', '1')
  setActivePinia(createPinia())
  const restored = useUserStore()
  assert.equal(restored.username, 'alice')
  assert.equal(restored.accessToken, 'token-a')
  restored.updateAdminStatus(false)
  assert.equal(localStorage.getItem('reader_default_config_mode'), '1')
  restored.setSession('token-a-new', 'alice', false)
  assert.equal(localStorage.getItem('reader_access_token'), 'token-b')
  restored.clear()
  assert.equal(localStorage.getItem('reader_access_token'), 'token-b')
  assert.equal(localStorage.getItem('reader_username'), 'bob')
  setActivePinia(createPinia())
  assert.equal(useUserStore().accessToken, '')
  assert.equal(useUserStore().username, '')
})

test('记住我的标签也固定完整身份，不随另一标签的共享保存值换号', () => {
  const store = freshStore()
  store.setSession('token-a', 'alice', true)
  assert.equal(sessionStorage.getItem('reader_access_token'), 'token-a')
  localStorage.setItem('reader_access_token', 'token-b')
  localStorage.setItem('reader_username', 'bob')
  setActivePinia(createPinia())
  const restored = useUserStore()
  assert.equal(restored.username, 'alice')
  assert.equal(restored.accessToken, 'token-a')
  restored.clear()
  assert.equal(localStorage.getItem('reader_access_token'), 'token-b')
})

test('旧共享记住我仅初始化新标签，完整配对后刷新固定；残缺配对不拼接', () => {
  freshStore()
  localStorage.setItem('reader_access_token', 'token-a')
  localStorage.setItem('reader_username', 'alice')
  setActivePinia(createPinia())
  assert.equal(useUserStore().username, 'alice')
  assert.equal(sessionStorage.getItem('reader_access_token'), 'token-a')
  localStorage.setItem('reader_access_token', 'token-b')
  localStorage.setItem('reader_username', 'bob')
  setActivePinia(createPinia())
  assert.equal(useUserStore().username, 'alice')
  sessionStorage.removeItem('reader_username')
  setActivePinia(createPinia())
  assert.equal(useUserStore().accessToken, '')
  assert.equal(useUserStore().username, '')
})

test('退出旧 token 不能删除同账号另一标签新 token 的记住我', () => {
  const store = freshStore()
  store.setSession('token-old', 'alice', true)
  localStorage.setItem('reader_access_token', 'token-new')
  store.clear()
  assert.equal(localStorage.getItem('reader_access_token'), 'token-new')
  assert.equal(localStorage.getItem('reader_username'), 'alice')
})
