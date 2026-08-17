import { test } from 'node:test'
import assert from 'node:assert/strict'

import { parsePageMode, PAGE_MODES, PAGE_MODE_LABELS } from './readerPageMode.ts'

test('parsePageMode：合法值原样返回（含新增 hslide/flip）', () => {
  assert.equal(parsePageMode('scroll'), 'scroll')
  assert.equal(parsePageMode('slide'), 'slide')
  assert.equal(parsePageMode('hslide'), 'hslide')
  assert.equal(parsePageMode('flip'), 'flip')
})

test('parsePageMode：null/undefined/非法值回退默认（默认 scroll）', () => {
  assert.equal(parsePageMode(null), 'scroll')
  assert.equal(parsePageMode(undefined), 'scroll')
  assert.equal(parsePageMode(''), 'scroll')
  assert.equal(parsePageMode('curl'), 'scroll')
  assert.equal(parsePageMode('SCROLL'), 'scroll')
  assert.equal(parsePageMode('bad', 'slide'), 'slide')
})

test('PAGE_MODES 覆盖全部模式且含默认', () => {
  assert.deepEqual(PAGE_MODES, ['scroll', 'slide', 'hslide', 'flip'])
  for (const m of PAGE_MODES) {
    assert.equal(parsePageMode(m), m)
    assert.equal(typeof PAGE_MODE_LABELS[m], 'string')
    assert.ok(PAGE_MODE_LABELS[m].length > 0)
  }
})
