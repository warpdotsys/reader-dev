import { test } from 'node:test'
import assert from 'node:assert/strict'
// @ts-ignore public/sw.js 为无类型声明的 ES module（node 直接导入；浏览器中为 SW 脚本）
import * as sw from '../../public/sw.js'

test('M5：/reader3/* 动态 API 判定为动态路径（不得 cache-first）', () => {
  assert.equal(sw.isApiPath('/reader3/getBookshelf'), true)
  assert.equal(sw.isApiPath('/reader3/file/download/cover/1'), true)
  assert.equal(sw.isApiPath('/reader3/probeSecureMode?_t=123'), true)
  assert.equal(sw.isDynamicPath('/reader3/getSystemInfo'), true)
  // 非 API 路径不误判
  assert.equal(sw.isApiPath('/assets/logo.png'), false)
  assert.equal(sw.isApiPath('/index.html'), false)
})

test('M5：/assets/proxy 为动态路径（网络优先，不得 cache-first）', () => {
  assert.equal(sw.isProxyPath('/assets/proxy'), true)
  assert.equal(sw.isProxyPath('/assets/proxy?url=http%3A%2F%2Fx%2F1.png'), false) // 仅取 pathname
  assert.equal(sw.isProxyPath('/assets/proxy2'), false)
  assert.equal(sw.isDynamicPath('/assets/proxy'), true)
})

test('M5：静态资源路径不误判为动态', () => {
  assert.equal(sw.isDynamicPath('/assets/logo.png'), false)
  assert.equal(sw.isDynamicPath('/index.html'), false)
  assert.equal(sw.isDynamicPath('/manifest.webmanifest'), false)
  assert.equal(sw.isDynamicPath('/assets/reader-abc123.js'), false)
})

test('M5：仅合法图片响应可入代理缓存（错误 JSON 体不缓存）', () => {
  const img = new Response('x', { headers: { 'content-type': 'image/png' } })
  assert.equal(sw.isImageResponse(img), true)
  const webp = new Response('x', { headers: { 'content-type': 'image/webp; charset=binary' } })
  assert.equal(sw.isImageResponse(webp), true)
  const json = new Response('{"isSuccess":false}', { headers: { 'content-type': 'application/json' } })
  assert.equal(sw.isImageResponse(json), false)
  const text = new Response('err', { headers: { 'content-type': 'text/plain' } })
  assert.equal(sw.isImageResponse(text), false)
})

test('M5：缓存版本已升级（旧 cache-first v1 缓存整体失效）+ 静态缓存条目上限已配置', () => {
  assert.equal(sw.CACHE_VERSION, 'reader-shell-v2')
  assert.ok(
    sw.STATIC_CACHE_MAX_ENTRIES > 0 && sw.STATIC_CACHE_MAX_ENTRIES <= 500,
    `条目上限应合理: ${sw.STATIC_CACHE_MAX_ENTRIES}`,
  )
})
