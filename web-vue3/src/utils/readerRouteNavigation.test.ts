import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readerRouteNavigation } from './readerRouteNavigation.ts'

const chapters = [{ isVolume: false }, { isVolume: false }, { isVolume: true }, { isVolume: false }]

test('EPUB 路由：同章 UTF-8/命名/空锚点与前进后退保持真实 backend index', () => {
  for (const [next, previous, current, expected] of [
    [{ chapter: '0', anchor: 'intro' }, { chapter: '0', anchor: '中文目标' }, 0,
      { kind: 'anchor', index: 0, fragment: 'intro' }],
    [{ chapter: '0', anchor: 'note' }, { chapter: '0', anchor: 'intro' }, 0,
      { kind: 'anchor', index: 0, fragment: 'note' }],
    [{ chapter: '0', anchor: '' }, { chapter: '0', anchor: 'intro' }, 0,
      { kind: 'anchor', index: 0, fragment: '' }],
    [{ chapter: '0', anchor: undefined }, { chapter: '0', anchor: 'intro' }, 0,
      { kind: 'anchor', index: 0, fragment: null }],
    [{ chapter: undefined, anchor: '中文目标' }, { chapter: undefined, anchor: undefined }, 1,
      { kind: 'anchor', index: 1, fragment: '中文目标' }],
    [{ chapter: '0', anchor: 'intro' }, { chapter: undefined, anchor: undefined }, 0,
      { kind: 'anchor', index: 0, fragment: 'intro' }],
    [{ chapter: '1', anchor: 'finale' }, { chapter: '0', anchor: 'intro' }, 0,
      { kind: 'chapter', index: 1, fragment: 'finale' }],
    [{ chapter: '0', anchor: '中文目标' }, { chapter: '1', anchor: 'finale' }, 1,
      { kind: 'chapter', index: 0, fragment: '中文目标' }],
  ] as const) {
    assert.deepEqual(readerRouteNavigation(next, previous, current, chapters, true), expected)
  }
})

test('EPUB 路由：拒绝无效章节/卷标/多值锚点，不覆盖自己的章同步或普通文本模式', () => {
  for (const invalid of [null, true, 1, [], {}, '', '-1', '1.5', '9007199254740992', '999']) {
    assert.equal(readerRouteNavigation({ chapter: invalid, anchor: 'intro' },
      { chapter: '0', anchor: 'note' }, 0, chapters, true), null)
  }
  assert.equal(readerRouteNavigation({ chapter: '2', anchor: 'intro' },
    { chapter: '0', anchor: 'note' }, 0, chapters, true), null)
  assert.equal(readerRouteNavigation({ chapter: '1', anchor: 'intro' },
    { chapter: '0', anchor: 'note' }, 0, new Array(2), true), null)
  assert.equal(readerRouteNavigation({ chapter: '0', anchor: ['intro', 'note'] },
    { chapter: '0', anchor: 'note' }, 0, chapters, true), null)
  assert.equal(readerRouteNavigation({ chapter: '1', anchor: undefined },
    { chapter: '0', anchor: 'intro' }, 1, chapters, true), null)
  assert.equal(readerRouteNavigation({ chapter: '0', anchor: 'intro' },
    { chapter: '0', anchor: 'intro' }, 0, chapters, true), null)
  assert.equal(readerRouteNavigation({ chapter: '0', anchor: 'intro' },
    { chapter: '0', anchor: 'note' }, 0, chapters, false), null)
  assert.deepEqual(readerRouteNavigation({ chapter: '1', anchor: 'ignored-in-text' },
    { chapter: '0', anchor: 'intro' }, 0, chapters, false), { kind: 'chapter', index: 1, fragment: null })
})
