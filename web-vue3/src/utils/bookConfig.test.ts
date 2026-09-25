import { test, beforeEach } from 'node:test'
import assert from 'node:assert/strict'
import {
  bookConfigKey,
  loadBookConfig,
  saveBookConfig,
  clearBookConfig,
  hasBookConfig,
} from './bookConfig.ts'

/** node 环境无 localStorage：最小内存 stub（bookConfig 仅用 getItem/setItem/removeItem） */
const mem = new Map<string, string>()
;(globalThis as { localStorage?: unknown }).localStorage = {
  getItem: (k: string) => mem.get(k) ?? null,
  setItem: (k: string, v: string) => void mem.set(k, v),
  removeItem: (k: string) => void mem.delete(k),
}

beforeEach(() => mem.clear())

test('GAP 6：bookConfigKey 按 bookUrl 唯一命名', () => {
  assert.equal(bookConfigKey('https://a/b.txt'), 'reader_book_config_https://a/b.txt')
  assert.equal(bookConfigKey('a') !== bookConfigKey('b'), true)
})

test('GAP 6：save/load 往返（reader_* 键 + 字符串值）', () => {
  saveBookConfig('b1', { reader_font_size: '20', reader_theme: 'paper' })
  assert.deepEqual(loadBookConfig('b1'), { reader_font_size: '20', reader_theme: 'paper' })
  // 不同书互不影响
  assert.deepEqual(loadBookConfig('b2'), {})
})

test('GAP 6：非法 JSON / 非对象 / 非字符串值 → 空对象', () => {
  mem.set('reader_book_config_bad', 'not-json{')
  assert.deepEqual(loadBookConfig('bad'), {})
  mem.set('reader_book_config_arr', JSON.stringify([1, 2]))
  assert.deepEqual(loadBookConfig('arr'), {})
  mem.set('reader_book_config_mix', JSON.stringify({ reader_font_size: 20, reader_theme: 'dark', n: null }))
  // 仅字符串值保留
  assert.deepEqual(loadBookConfig('mix'), { reader_theme: 'dark' })
})

test('GAP 6：hasBookConfig / clearBookConfig', () => {
  assert.equal(hasBookConfig('b1'), false)
  saveBookConfig('b1', { reader_font_size: '16' })
  assert.equal(hasBookConfig('b1'), true)
  assert.equal(clearBookConfig('b1'), true)
  assert.equal(hasBookConfig('b1'), false)
  assert.deepEqual(loadBookConfig('b1'), {})
})

test('GAP 6：空对象写入后视为无覆盖', () => {
  saveBookConfig('b1', {})
  assert.equal(hasBookConfig('b1'), false)
})
