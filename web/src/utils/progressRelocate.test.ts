import { test } from 'node:test'
import assert from 'node:assert/strict'
import { relocateChapterIndex } from './progressRelocate.ts'
import type { BookChapter } from '@/types'

function toc(titles: string[], volumeAt: number[] = []): BookChapter[] {
  return titles.map((title, i) => ({
    title,
    url: `http://src/${i}`,
    index: i,
    isVolume: volumeAt.includes(i),
  }))
}

test('GAP 6：旧索引范围内且非卷标 → 原样保留', () => {
  const t = toc(['第一卷', '第一章', '第二章', '第三章'], [0])
  assert.equal(relocateChapterIndex(2, '第二章', t), 2)
})

test('GAP 6：旧索引指向新目录的卷标行 → 标题匹配非卷标章', () => {
  const t = toc(['第一卷', '第一章', '第二章'], [0])
  // 旧索引 0 在新源是卷标行 → 标题「第一章」命中下标 1
  assert.equal(relocateChapterIndex(0, '第一章', t), 1)
})

test('GAP 6：旧索引越界 + 标题命中 → 用命中下标', () => {
  const t = toc(['楔子', '第一章', '第二章'])
  assert.equal(relocateChapterIndex(99, '第一章', t), 1)
})

test('GAP 6：旧索引越界 + 标题不匹配 → 钳制到末章', () => {
  const t = toc(['第一章', '第二章'])
  assert.equal(relocateChapterIndex(99, '不存在的章', t), 1)
})

test('GAP 6：旧索引越界 + 无标题 → 钳制到末章', () => {
  const t = toc(['第一章', '第二章', '第三章'])
  assert.equal(relocateChapterIndex(10, null, t), 2)
})

test('GAP 6：钳制不落卷标行（末行是卷标 → 前移）', () => {
  const t = toc(['第一章', '第二章', '第三卷'], [2])
  assert.equal(relocateChapterIndex(99, '', t), 1)
})

test('GAP 6：标题匹配跳过卷标行（同名卷标不误命中）', () => {
  const t = toc(['第三卷', '第一章', '第二章'], [0])
  // 标题「第三卷」是卷标行 → 不命中，走钳制
  assert.equal(relocateChapterIndex(99, '第三卷', t), 2)
})

test('GAP 6：无进度（-1）/ 空目录 / 非法索引 → -1（不动服务端进度）', () => {
  assert.equal(relocateChapterIndex(-1, 'x', toc(['a'])), -1)
  assert.equal(relocateChapterIndex(0, 'x', []), -1)
  assert.equal(relocateChapterIndex(Number.NaN, 'x', toc(['a'])), -1)
  // 全卷标目录 → -1
  assert.equal(relocateChapterIndex(0, 'x', toc(['卷'], [0])), -1)
})
