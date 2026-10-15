/** 仅生成文本/音频和内存 Cache API；不读取浏览器真实缓存或正文。 */
import { test, type TestContext } from 'node:test'
import assert from 'node:assert/strict'
import { ttsCacheKey, getCachedTts, putCachedTts, clearTtsCache, ttsCacheStats } from './ttsCache.ts'
import { localChapterCacheScope } from './readerLocalCache.ts'

const P = { engine: 'edge', voice: 'generated', rate: '+0%', pitch: '+0Hz' }
const user = { username: 'generated-a', accessToken: 'generated-token', isAdmin: false, defaultConfigMode: false }
const deployment = 'https://generated.invalid/reader/'
const SCOPE = localChapterCacheScope(user, deployment)
const B_SCOPE = localChapterCacheScope({ ...user, username: 'generated-b' }, deployment)

function setGlobal(t: TestContext, name: string, value: unknown) {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, name)
  Object.defineProperty(globalThis, name, { configurable: true, value })
  t.after(() => {
    if (descriptor) Object.defineProperty(globalThis, name, descriptor)
    else Reflect.deleteProperty(globalThis, name)
  })
}
function memoryCaches(t: TestContext) {
  const maps = new Map<string, Map<string, Response>>()
  const opened: string[] = []
  const deleted: string[] = []
  const path = (key: RequestInfo | URL) => key instanceof Request ? new URL(key.url).pathname : String(key)
  const storage = {
    async open(name: string) {
      opened.push(name)
      let entries = maps.get(name)
      if (!entries) { entries = new Map(); maps.set(name, entries) }
      const values = entries
      return {
        async put(key: RequestInfo | URL, value: Response) { values.set(path(key), value.clone()) },
        async match(key: RequestInfo | URL) { return values.get(path(key))?.clone() },
        async keys() { return [...values.keys()].map(key => new Request('https://generated.invalid' + key)) },
      }
    },
    async delete(name: string) { deleted.push(name); return maps.delete(name) },
  }
  setGlobal(t, 'caches', storage)
  return { storage, maps, opened, deleted }
}

test('全文与参数相同时摘要稳定；键不含正文或令牌', async () => {
  const key = await ttsCacheKey(SCOPE, '生成正文', P)
  assert.deepEqual(key, await ttsCacheKey(SCOPE, '生成正文', P))
  assert.ok(key)
  assert.match(key.cacheName, /^tts-audio-v2:[0-9a-f]{64}$/)
  assert.match(key.url, /^\/tts\/v2\/[0-9a-f]{64}\.mp3$/)
  assert.ok(Object.isFrozen(key))
  assert.doesNotMatch(JSON.stringify(key), /生成正文|generated-token|generated-a/)
})

test('先前已失败的生成负例：相同长度、相同前 4096 字符、不同尾部不再复用音频', async () => {
  const prefix = '生成'.repeat(2500)
  assert.notDeepEqual(await ttsCacheKey(SCOPE, prefix + '甲', P), await ttsCacheKey(SCOPE, prefix + '乙', P))
})

test('任何合成参数变化都重新寻址，参数内分隔符不产生串接歧义', async () => {
  const base = await ttsCacheKey(SCOPE, '生成正文', P)
  for (const changed of [{ voice: 'other' }, { rate: '+10%' }, { pitch: '-2Hz' }, { engine: 'http' },
      { style: 'cheerful' }, { volume: '+20%' }]) {
    assert.notDeepEqual(base, await ttsCacheKey(SCOPE, '生成正文', { ...P, ...changed }))
  }
  assert.notDeepEqual(
    await ttsCacheKey(SCOPE, '生成正文', { ...P, voice: 'a|b', rate: 'c' }),
    await ttsCacheKey(SCOPE, '生成正文', { ...P, voice: 'a', rate: 'b|c' }),
  )
})

test('部署、账号、管理员实际配置层互相隔离；令牌更换仍仅复用同账号数据', async () => {
  const base = await ttsCacheKey(SCOPE, '生成正文', P)
  for (const scope of [B_SCOPE, localChapterCacheScope(user, deployment + 'other/'),
      localChapterCacheScope({ ...user, isAdmin: true, defaultConfigMode: true }, deployment)]) {
    const other = await ttsCacheKey(scope, '生成正文', P)
    assert.ok(base && other)
    assert.notEqual(base.cacheName, other.cacheName)
  }
  assert.deepEqual(base, await ttsCacheKey(localChapterCacheScope({ ...user, accessToken: 'new-generated-token' }, deployment), '生成正文', P))
})

test('无认证作用域时不建立共享键，也不打开/删除任何缓存', async t => {
  const cache = memoryCaches(t)
  assert.equal(await ttsCacheKey(null, '生成正文', P), null)
  assert.equal(await getCachedTts(null), null)
  await putCachedTts(null, new Blob(['generated']))
  assert.deepEqual(await ttsCacheStats(null), { count: 0, bytes: 0 })
  assert.equal(await clearTtsCache(null), false)
  assert.deepEqual(cache.opened, [])
  assert.deepEqual(cache.deleted, [])
})

test('摘要 API 不可用时禁用缓存，不能退回 v1/FNV 或裸正文', async t => {
  const cache = memoryCaches(t)
  setGlobal(t, 'crypto', undefined)
  assert.equal(await ttsCacheKey(SCOPE, '生成正文', P), null)
  assert.deepEqual(await ttsCacheStats(SCOPE), { count: 0, bytes: 0 })
  assert.equal(await clearTtsCache(SCOPE), false)
  assert.deepEqual(cache.opened, [])
  assert.deepEqual(cache.deleted, [])
})

test('Cache API 不可用时读/统计/清理安全退出', async t => {
  setGlobal(t, 'caches', undefined)
  const key = await ttsCacheKey(SCOPE, '生成正文', P)
  assert.equal(await getCachedTts(key), null)
  await putCachedTts(key, new Blob(['generated']))
  assert.deepEqual(await ttsCacheStats(SCOPE), { count: 0, bytes: 0 })
  assert.equal(await clearTtsCache(SCOPE), false)
})

test('A 音频不被 B 命中；统计与清理仅属于所选账号，旧 v1 保留而不读取', async t => {
  const cache = memoryCaches(t)
  cache.maps.set('tts-audio-v1', new Map([['/tts/legacy.mp3', new Response('generated legacy')]]))
  const a = await ttsCacheKey(SCOPE, '同样的生成正文', P)
  const b = await ttsCacheKey(B_SCOPE, '同样的生成正文', P)
  await putCachedTts(a, new Blob(['generated audio A'], { type: 'audio/mpeg' }))
  assert.equal(await getCachedTts(b), null)
  await putCachedTts(b, new Blob(['generated B']))
  assert.equal(await (await getCachedTts(a))?.text(), 'generated audio A')
  assert.equal(await (await getCachedTts(b))?.text(), 'generated B')
  assert.deepEqual(await ttsCacheStats(SCOPE), { count: 1, bytes: 17 })
  assert.deepEqual(await ttsCacheStats(B_SCOPE), { count: 1, bytes: 11 })
  assert.equal(await clearTtsCache(SCOPE), true)
  assert.equal(await getCachedTts(a), null)
  assert.equal(await (await getCachedTts(b))?.text(), 'generated B')
  assert.equal(cache.maps.has('tts-audio-v1'), true)
  assert.equal(cache.opened.includes('tts-audio-v1'), false)
  assert.equal(cache.deleted.includes('tts-audio-v1'), false)
})

test('晚到 cache.open 后身份失效，不调用 match/put', async t => {
  const key = await ttsCacheKey(SCOPE, '生成正文', P)
  let current = true, calls = 0
  setGlobal(t, 'caches', { open: async () => {
    current = false
    return { match: async () => { calls++; return new Response('generated') }, put: async () => { calls++ } }
  } })
  assert.equal(await getCachedTts(key, () => current), null)
  assert.equal(calls, 0)
  current = true
  await putCachedTts(key, new Blob(['generated']), () => current)
  assert.equal(calls, 0)
})

test('晚到 match 或 blob 不交给新会话播放', async t => {
  const key = await ttsCacheKey(SCOPE, '生成正文', P)
  let current = true
  let phase = 'match'
  setGlobal(t, 'caches', { open: async () => ({ match: async () => {
    if (phase === 'match') {
      current = false
      return new Response('generated old')
    }
    return { blob: async () => { current = false; return new Blob(['generated old']) } }
  } }) })
  assert.equal(await getCachedTts(key, () => current), null)
  current = true
  phase = 'blob'
  assert.equal(await getCachedTts(key, () => current), null)
})

test('读取与清理开始前已过期，不探测其他账号或旧缓存', async t => {
  const cache = memoryCaches(t)
  const key = await ttsCacheKey(SCOPE, '生成正文', P)
  assert.equal(await getCachedTts(key, () => false), null)
  await putCachedTts(key, new Blob(['generated']), () => false)
  assert.deepEqual(await ttsCacheStats(SCOPE, () => false), { count: 0, bytes: 0 })
  assert.equal(await clearTtsCache(SCOPE, () => false), false)
  assert.deepEqual(cache.opened, [])
  assert.deepEqual(cache.deleted, [])
})

test('统计过程中会话变化时不向新账号展示旧字节数', async t => {
  let current = true
  setGlobal(t, 'caches', { open: async () => ({
    keys: async () => ['generated key'],
    match: async () => ({ blob: async () => { current = false; return new Blob(['generated private']) } }),
  }) })
  assert.deepEqual(await ttsCacheStats(SCOPE, () => current), { count: 0, bytes: 0 })
})

test('不合法旧键拒绝，不打开任何缓存', async t => {
  const cache = memoryCaches(t)
  const bad = { cacheName: 'tts-audio-v1', url: '/tts/old/old.mp3' }
  assert.equal(await getCachedTts(bad), null)
  await putCachedTts(bad, new Blob(['generated']))
  assert.deepEqual(cache.opened, [])
})

test('缓存 API 拒绝不会谎报清理成功或阻断合成调用方', async t => {
  setGlobal(t, 'caches', { open: async () => { throw new Error('generated denied') },
    delete: async () => { throw new Error('generated denied') } })
  const key = await ttsCacheKey(SCOPE, '生成正文', P)
  assert.equal(await getCachedTts(key), null)
  await putCachedTts(key, new Blob(['generated']))
  assert.deepEqual(await ttsCacheStats(SCOPE), { count: 0, bytes: 0 })
  assert.equal(await clearTtsCache(SCOPE), false)
})
