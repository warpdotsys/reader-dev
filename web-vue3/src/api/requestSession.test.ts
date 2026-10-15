import assert from 'node:assert/strict'
import { test } from 'node:test'
import { canExpireRequestSession, captureRequestSession, isRequestSessionCurrent, StaleSessionResponseError } from './requestSession.ts'

const session = { accessToken: 'generated-token', username: 'generated-user', isAdmin: false,
  defaultConfigMode: false, sessionRevision: 1 }

test('请求快照不随响应前的 store 变更而改变', () => {
  const store = { ...session }
  const snapshot = captureRequestSession(store)
  assert.ok(Object.isFrozen(snapshot))
  assert.ok(isRequestSessionCurrent(snapshot, store))
  store.username = 'generated-other'
  assert.equal(snapshot.username, session.username)
  assert.equal(isRequestSessionCurrent(snapshot, store), false)
})

test('换账号、换 token 或实际配置命名空间后旧响应均失效', () => {
  const snapshot = captureRequestSession(session)
  for (const change of [{ username: 'other' }, { accessToken: 'new-token' },
    { isAdmin: true, defaultConfigMode: true }, { sessionRevision: 2 }]) {
    assert.equal(isRequestSessionCurrent(snapshot, { ...session, ...change }), false)
  }
  assert.equal(isRequestSessionCurrent(undefined, session), false)
  assert.ok(isRequestSessionCurrent(snapshot, { ...session, isAdmin: true }))
  assert.ok(isRequestSessionCurrent(snapshot, { ...session, defaultConfigMode: true }))
})

test('同账号同 token 的重新登录仍由内存代数区分，错误不泄露凭据', () => {
  const snapshot = captureRequestSession(session)
  assert.equal(isRequestSessionCurrent(snapshot, { ...session, sessionRevision: 3 }), false)
  const error = new StaleSessionResponseError()
  assert.equal(error.code, 'READER_STALE_SESSION')
  assert.doesNotMatch(error.message + error.stack, /generated-token|generated-user/)
})

test('只有当前已认证会话能够过期，匿名失败不能改变并行登录的代数', () => {
  const snapshot = captureRequestSession(session)
  assert.ok(canExpireRequestSession(snapshot, session))
  assert.equal(canExpireRequestSession(snapshot, { ...session, sessionRevision: 2 }), false)
  const anonymous = { ...session, accessToken: '', username: '', sessionRevision: 0 }
  assert.ok(isRequestSessionCurrent(captureRequestSession(anonymous), anonymous))
  assert.equal(canExpireRequestSession(captureRequestSession(anonymous), anonymous), false)
})
