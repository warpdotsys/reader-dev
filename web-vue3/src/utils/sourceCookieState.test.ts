import { test } from 'node:test'
import assert from 'node:assert/strict'
import { redactSourceCookiePreview, savedSourceCookieRows } from './sourceCookieState.ts'

test('Cookie 摘要隐藏完整值和前缀，保持带等号的值不可见', () => {
  assert.equal(redactSourceCookiePreview('session=private==; token=another-private'), 'session=***; token=***')
  assert.equal(redactSourceCookiePreview('session=***'), 'session=***')
  assert.equal(redactSourceCookiePreview('不是 Cookie'), '内容已隐藏')
  assert.equal(redactSourceCookiePreview(null), '内容已隐藏')
  assert.equal(redactSourceCookiePreview('opaque'), '内容已隐藏')
})

test('只接受服务端确认的 Cookie 行，不将任意标记当登录成功', () => {
  assert.deepEqual(savedSourceCookieRows([null, {}, { sourceUrl: 'https://example.com', hasCookie: false },
    { sourceUrl: 'https://example.com', cookie: 'session=private' }]), [])
  assert.throws(() => savedSourceCookieRows(true), /不是数组/)
})

test('移除认证头和 Cookie 原值，不保留未知字段或重复状态', () => {
  const input = { sourceUrl: 'https://example.com', hasCookie: true, cookie: 'session=private==',
    loginHeader: 'Authorization: Bearer private', password: 'private', updatedAt: 123 }
  const rows = savedSourceCookieRows([input, input])
  assert.equal(rows.length, 1)
  assert.equal(rows[0].cookiePreview, 'session=***')
  assert.equal(rows[0].updatedAt, 123)
  assert.equal(JSON.stringify(rows).includes('private'), false)
  assert.equal(input.cookie, 'session=private==')
})
