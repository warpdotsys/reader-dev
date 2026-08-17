import { test } from 'node:test'
import assert from 'node:assert/strict'
import { moveGroupTo } from './groupOrder.ts'

const groups = [{ id: 1, name: 'A' }, { id: 2, name: 'B' }, { id: 3, name: 'C' }, { id: 4, name: 'D' }]

test('P2 分组拖拽 off-by-one：向下拖拽（A 拖到 C）落点应取 C 的位置而非其后', () => {
  // 旧实现：移除 A 后仍按移除前的 toIdx 插入 → [B, C, A]（偏移一位）
  const out = moveGroupTo(groups, 1, 3)
  assert.deepEqual(
    out.map((g) => g.id),
    [2, 1, 3, 4],
  )
})

test('向上拖拽（C 拖到 A）：移除后目标索引不变，落点在 A 位置', () => {
  const out = moveGroupTo(groups, 3, 1)
  assert.deepEqual(
    out.map((g) => g.id),
    [3, 1, 2, 4],
  )
})

test('相邻向下拖拽（A 拖到 B）：A 已在 B 正上方，插入 B 位置 = 原位（无变化）', () => {
  // 语义：拖拽项落在目标行位置（目标行之前）；A 本就在 B 正上方 → 不变
  const out = moveGroupTo(groups, 1, 2)
  assert.deepEqual(
    out.map((g) => g.id),
    [1, 2, 3, 4],
  )
})

test('相邻向上拖拽（B 拖到 A）：B 落在 A 位置', () => {
  const out = moveGroupTo(groups, 2, 1)
  assert.deepEqual(
    out.map((g) => g.id),
    [2, 1, 3, 4],
  )
})

test('源/目标不存在：返回原数组副本，不修改', () => {
  assert.deepEqual(moveGroupTo(groups, 99, 1).map((g) => g.id), [1, 2, 3, 4])
  assert.deepEqual(moveGroupTo(groups, 1, 99).map((g) => g.id), [1, 2, 3, 4])
  assert.deepEqual(moveGroupTo(groups, 1, 1).map((g) => g.id), [1, 2, 3, 4])
  // 不修改入参
  assert.deepEqual(
    groups.map((g) => g.id),
    [1, 2, 3, 4],
  )
})
