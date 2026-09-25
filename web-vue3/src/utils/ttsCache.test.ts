/**
 * C2: ttsCache unit tests (pure key logic + Cache API absent fallbacks).
 */
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ttsCacheKey, getCachedTts, putCachedTts, clearTtsCache, ttsCacheStats } from './ttsCache.ts'

const P = {
  engine: 'edge',
  voice: 'zh-CN-XiaoxiaoNeural',
  rate: '+0%',
  pitch: '+0Hz',
}

test('C2: ttsCacheKey is deterministic for identical inputs', () => {
  assert.equal(ttsCacheKey('same text', P), ttsCacheKey('same text', P))
})

test('C2: ttsCacheKey changes when any synthesis param changes', () => {
  const base = ttsCacheKey('text', P)
  assert.notEqual(base, ttsCacheKey('text', { ...P, voice: 'other' }))
  assert.notEqual(base, ttsCacheKey('text', { ...P, rate: '+10%' }))
  assert.notEqual(base, ttsCacheKey('text', { ...P, pitch: '-2Hz' }))
  assert.notEqual(base, ttsCacheKey('text', { ...P, engine: 'http' }))
  assert.notEqual(base, ttsCacheKey('text', { ...P, style: 'cheerful' }))
})

test('C2: ttsCacheKey distinguishes text content and length', () => {
  const a = ttsCacheKey('abc'.repeat(100), P)
  const b = ttsCacheKey('abd'.repeat(100), P)
  assert.notEqual(a, b)
  // length suffix makes prefix-collisions unlikely even with same first 4096 chars
  const c = ttsCacheKey('x'.repeat(5000), P)
  const d = ttsCacheKey(`${'x'.repeat(5000)}tail`, P)
  assert.notEqual(c, d)
})

test('C2: key format is /tts/<sig>/<body>.mp3 path segments', () => {
  const k = ttsCacheKey('hello', P)
  assert.ok(k.startsWith('/tts/'), `unexpected key: ${k}`)
  assert.ok(k.endsWith('.mp3'))
  assert.equal(k.split('/').length, 4)
})

test('C2: graceful no-op when Cache API unavailable (node)', async () => {
  assert.equal(await getCachedTts('/tts/x/y.mp3'), null)
  await ttsCacheStats()
  putCachedTts('/tts/x/y.mp3', new Blob([new Uint8Array([1])]))
  await clearTtsCache()
})
