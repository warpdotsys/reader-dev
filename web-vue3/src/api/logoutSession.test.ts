import assert from 'node:assert/strict'
import { test } from 'node:test'
import { logoutSession } from './logoutSession.ts'
import type { RequestSessionState } from './requestSession.ts'

function state(): RequestSessionState & { clear(): void } {
  return { accessToken: 'generated-a', username: 'alice', isAdmin: false, defaultConfigMode: false,
    sessionRevision: 1, clear() { this.sessionRevision++; this.accessToken = ''; this.username = '' } }
}

test('退出捕获实际身份；仅原 JAR 三字段退出契约确认服务端撤销', async () => {
  const store = state()
  assert.equal(await logoutSession(store, async session => {
    assert.equal(session.accessToken, 'generated-a')
    assert.equal(session.username, 'alice')
    assert.ok(Object.isFrozen(session))
    return { isSuccess: true, errorMsg: '', data: 'NEED_LOGIN' }
  }), 'revoked')
  assert.equal(store.accessToken, '')
})

test('网络/业务/畸形失败只清本机，不伪造远端撤销成功', async () => {
  for (const reply of [null, { isSuccess: false, errorMsg: '拒绝', data: 'NEED_LOGIN' },
    { isSuccess: true, errorMsg: '', data: null }, { isSuccess: true, errorMsg: '异常', data: 'NEED_LOGIN' }]) {
    const store = state()
    assert.equal(await logoutSession(store, async () => {
      if (!reply) throw new Error('generated network failure')
      return reply
    }), 'local-only')
    assert.equal(store.accessToken, '')
  }
})

test('旧退出成功或失败晚到，不能清除新账号/同账号新会话', async () => {
  for (const sameAccount of [false, true]) {
    for (const failed of [false, true]) {
      const store = state()
      let release!: () => void
      const pending = new Promise<void>(resolve => { release = resolve })
      const logout = logoutSession(store, async captured => {
        await pending
        assert.equal(captured.accessToken, 'generated-a')
        if (failed) throw new Error('generated delayed failure')
        return { isSuccess: true, errorMsg: '', data: 'NEED_LOGIN' }
      })
      store.sessionRevision++
      store.accessToken = 'generated-new'
      store.username = sameAccount ? 'alice' : 'bob'
      release()
      assert.equal(await logout, 'superseded')
      assert.equal(store.accessToken, 'generated-new')
      assert.equal(store.username, sameAccount ? 'alice' : 'bob')
    }
  }
})

test('匿名退出不发送共享 Cookie 鉴权请求', async () => {
  const store = state()
  store.accessToken = ''
  assert.equal(await logoutSession(store, async () => { throw new Error('must not send') }), 'local-only')
})
