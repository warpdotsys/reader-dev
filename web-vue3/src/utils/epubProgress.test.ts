import { test } from 'node:test'
import assert from 'node:assert/strict'
import { epubProgressKey, epubScrollPosition } from './epubProgress.ts'

test('raw EPUB positions are scoped to namespace and book, without key collisions', () => {
  assert.notEqual(epubProgressKey('alice', 'book'), epubProgressKey('bob', 'book'))
  assert.notEqual(epubProgressKey('alice', 'one'), epubProgressKey('alice', 'two'))
  assert.notEqual(epubProgressKey('a-b', 'c'), epubProgressKey('a', 'b-c'))
})

test('raw position only restores its exact chapter, including a TOC fragment', () => {
  const raw = JSON.stringify({ chapterUrl: 'chapter.xhtml#part2', scrollY: 640 })
  assert.equal(epubScrollPosition(raw, 'chapter.xhtml#part2'), 640)
  assert.equal(epubScrollPosition(raw, 'chapter.xhtml#part1'), null)
  assert.equal(epubScrollPosition(raw, 'other.xhtml#part2'), null)
})

test('missing or invalid positions use the chapter anchor, while a saved zero remains valid', () => {
  for (const raw of [null, '', '{', 'null', '[]', '{}',
    '{"chapterUrl":"chapter","scrollY":-1}', '{"chapterUrl":"chapter","scrollY":"500"}',
    '{"chapterUrl":"chapter","scrollY":1e999}']) {
    assert.equal(epubScrollPosition(raw, 'chapter'), null)
  }
  assert.equal(epubScrollPosition('{"chapterUrl":"chapter","scrollY":0}', 'chapter'), 0)
})

test('separate raw anchors may share a legacy chapter without restoring each other positions', () => {
  const raw = JSON.stringify({ chapterUrl: 'shared.xhtml', fragment: '第二节', scrollY: 2400 })
  assert.equal(epubScrollPosition(raw, 'shared.xhtml', '第二节'), 2400)
  assert.equal(epubScrollPosition(raw, 'shared.xhtml', '首节'), null)
  assert.equal(epubScrollPosition(raw, 'shared.xhtml'), null)
  assert.equal(epubScrollPosition('{"chapterUrl":"shared.xhtml","scrollY":640}', 'shared.xhtml'), 640)
})
