import assert from 'node:assert/strict'
import { test } from 'node:test'
import { assertLegacyNamespace, legacyNamespaceContext, NamespaceProofError } from './legacyNamespace.ts'
import { localChapterCacheScope } from './readerLocalCache.ts'

const user = { accessToken: 'generated-token', username: 'generated-a', isAdmin: true, defaultConfigMode: true }

test('管理空间只发送 legacy userNS 和密钥请求头，不占用配置 ns 或把密钥写入 URL', () => {
  const context = legacyNamespaceContext(user, 'generated-secret')
  assert.deepEqual(context.params, { accessToken: 'generated-token', readerAuth: 'access-token', userNS: 'default' })
  assert.deepEqual(context.headers, { 'X-Reader-Secure-Key': 'generated-secret' })
  assert.equal(new URLSearchParams(context.params).toString().includes('generated-secret'), false)
  assert.throws(() => legacyNamespaceContext(user, ''), NamespaceProofError)
  assert.deepEqual(legacyNamespaceContext({ ...user, isAdmin: false }, 'generated-secret').headers, {})
})

test('系统空间拒绝缺失/错误的后端空间确认；全局用户管理和本人空间不要求此头', () => {
  for (const path of ['/getBookshelf', '/reader3/file/download', '/getUserConfig?ns=reader', '/searchBookMultiSSE']) {
    for (const proof of [undefined, null, '', 'generated-a']) {
      assert.throws(() => assertLegacyNamespace({ systemNamespace: true }, path, proof), NamespaceProofError)
    }
    assert.doesNotThrow(() => assertLegacyNamespace({ systemNamespace: true }, path, 'default'))
  }
  assert.doesNotThrow(() => assertLegacyNamespace({ systemNamespace: true }, '/reader3/getUserList', null))
  assert.doesNotThrow(() => assertLegacyNamespace({ systemNamespace: false }, '/getBookshelf', null))
})

test('旧 Cookie 归属及错误 ns 缓存均不复用，新缓存明确记录 token 鉴权版本', () => {
  assert.equal(localChapterCacheScope(user, 'https://generated/'),
    JSON.stringify(['https://generated/', 'generated-a', 'default', 'legacy-userNS-v1', 'token-auth-v1']))
  assert.equal(localChapterCacheScope({ ...user, defaultConfigMode: false }, 'https://generated/'),
    JSON.stringify(['https://generated/', 'generated-a', 'generated-a', 'token-auth-v1']))
})
