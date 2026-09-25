/**
 * C2: rssMedia unit tests (HLS detection; enhancement requires DOM, covered by e2e).
 */
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { isHlsSrc } from './rssMedia.ts'

test('C2: isHlsSrc matches .m3u8 with query/hash suffixes', () => {
  assert.equal(isHlsSrc('https://x.com/live/index.m3u8'), true)
  assert.equal(isHlsSrc('https://x.com/live.m3u8?token=abc'), true)
  assert.equal(isHlsSrc('https://x.com/live.M3U8#frag'), true)
})

test('C2: isHlsSrc rejects native media formats and non-media paths', () => {
  assert.equal(isHlsSrc('https://x.com/episode.mp3'), false)
  assert.equal(isHlsSrc('https://x.com/video.mp4'), false)
  assert.equal(isHlsSrc('https://x.com/audio.ogg'), false)
  assert.equal(isHlsSrc('https://x.com/stream.flv'), false)
  assert.equal(isHlsSrc(''), false)
})
