import { test } from 'node:test'
import assert from 'node:assert/strict'
import { buildTocEntries } from './tocPreview.ts'
import type { BookChapter } from '@/types'

const ch = (title: string, isVolume = false): BookChapter => ({
  title,
  url: `u-${title}`,
  isVolume,
  index: 0,
})

test('P2 分卷标题无限追加：前 50 章截断后不再追加任何行（含后续分卷标题行）', () => {
  const chapters: BookChapter[] = []
  for (let i = 1; i <= 50; i++) chapters.push(ch(`第 ${i} 章`))
  // 50 章之后还有分卷标题 + 更多章（旧实现会把分卷标题无限追加到预览里）
  chapters.push(ch('第二卷', true))
  chapters.push(ch('第 51 章'))
  chapters.push(ch('第三卷', true))

  const entries = buildTocEntries(chapters, 50)
  assert.equal(entries.length, 50)
  assert.ok(entries.every((e) => e.kind === 'chapter'))
  assert.equal(entries[0].title, '第 1 章')
  assert.equal(entries[49].title, '第 50 章')
})

test('卷标题行渲染为分隔行且不计入章数上限', () => {
  const chapters = [
    ch('第一卷', true),
    ch('第 1 章'),
    ch('第 2 章'),
    ch('第二卷', true),
    ch('第 3 章'),
  ]
  const entries = buildTocEntries(chapters, 50)
  assert.deepEqual(
    entries.map((e) => `${e.kind}:${e.title}`),
    ['volume:第一卷', 'chapter:第 1 章', 'chapter:第 2 章', 'volume:第二卷', 'chapter:第 3 章'],
  )
})

test('卷标题在截断边界：达到上限后出现的分卷行不再追加', () => {
  const chapters: BookChapter[] = []
  for (let i = 1; i <= 50; i++) chapters.push(ch(`第 ${i} 章`))
  chapters.push(ch('尾部卷', true))
  const entries = buildTocEntries(chapters, 50)
  assert.equal(entries.length, 50)
  assert.ok(entries.every((e) => e.kind === 'chapter'))
})

test('max=0 或空目录：返回空数组；index 为完整目录下标', () => {
  assert.deepEqual(buildTocEntries([ch('第 1 章')], 0), [])
  assert.deepEqual(buildTocEntries([], 50), [])
  const chapters = [ch('第一卷', true), ch('第 1 章'), ch('第 2 章')]
  const entries = buildTocEntries(chapters, 50)
  // 卷标题行占下标 0，章节从 1 开始（与阅读页 ?chapter 语义一致）
  assert.deepEqual(
    entries.map((e) => e.index),
    [0, 1, 2],
  )
})

test('transform 应用于标题（简繁转换场景）', () => {
  const entries = buildTocEntries([ch('简体章')], 50, (t) => `[${t}]`)
  assert.equal(entries[0].title, '[简体章]')
})
