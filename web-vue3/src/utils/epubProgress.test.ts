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
  assert.equal(epubScrollPosition(raw, 'chapter.xhtml#part1'), 0)
  assert.equal(epubScrollPosition(raw, 'other.xhtml#part2'), 0)
})

test('invalid, negative, non-finite and non-numeric positions start at the top', () => {
  for (const raw of [null, '', '{', 'null', '[]', '{}',
    '{"chapterUrl":"chapter","scrollY":-1}', '{"chapterUrl":"chapter","scrollY":"500"}',
    '{"chapterUrl":"chapter","scrollY":1e999}']) {
    assert.equal(epubScrollPosition(raw, 'chapter'), 0)
  }
  assert.equal(epubScrollPosition('{"chapterUrl":"chapter","scrollY":0}', 'chapter'), 0)
})
