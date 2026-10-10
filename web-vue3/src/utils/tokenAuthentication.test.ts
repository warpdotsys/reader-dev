import assert from 'node:assert/strict'
import { test } from 'node:test'
import { authenticatedReaderUrl, tokenAuthenticationParams } from './tokenAuthentication.ts'

test('token 身份必须显式 opt-in，未登录不伪造鉴权参数', () => {
  assert.deepEqual(tokenAuthenticationParams('generated-token'), { accessToken: 'generated-token', readerAuth: 'access-token' })
  for (const token of ['', null, undefined]) assert.deepEqual(tokenAuthenticationParams(token), {})
})

test('文件直出 URL 使用当前 token 与显式模式，保留路径/流/片段，不把 token 给外站', () => {
  const path = '/reader3/file/download?path=generated.txt&stream=1&accessToken=old-generated-token#page=2'
  const url = new URL(authenticatedReaderUrl(path, 'new-generated-token'), 'https://reader-api.invalid')
  assert.equal(url.searchParams.get('readerAuth'), 'access-token')
  assert.equal(url.searchParams.get('accessToken'), 'new-generated-token')
  assert.equal(url.searchParams.get('path'), 'generated.txt')
  assert.equal(url.searchParams.get('stream'), '1')
  assert.equal(url.hash, '#page=2')
  for (const external of ['https://generated.example/reader3/file/download', '//generated.example/reader3/file/download', '/assets/proxy?url=generated', '/reader3/../../assets/proxy', '/reader3/%2e%2e/assets/proxy', '/reader3/..\\assets/proxy']) {
    assert.equal(authenticatedReaderUrl(external, 'generated-secret'), external)
  }
  const loggedOut = new URL(authenticatedReaderUrl(path, ''), 'https://reader-api.invalid')
  assert.equal(loggedOut.searchParams.has('accessToken'), false)
  assert.equal(loggedOut.searchParams.get('readerAuth'), 'access-token')
})
