import { test, beforeEach } from 'node:test'
import assert from 'node:assert/strict'
import {
  SOURCE_GROUP_ORDER_KEY,
  loadGroupOrder,
  persistGroupOrder,
  clearGroupOrder,
  mergeGroupOrder,
  reorderGroup,
} from './sourceGroupOrder.ts'

/** node 环境无 localStorage：最小内存 stub */
const mem = new Map<string, string>()
const storage = {
  getItem: (k: string) => mem.get(k) ?? null,
  setItem: (k: string, v: string) => void mem.set(k, v),
  removeItem: (k: string) => void mem.delete(k),
}

beforeEach(() => mem.clear())

test('load/persist 往返：reader_source_group_order 存 string[]', () => {
  assert.deepEqual(loadGroupOrder(storage), [])
  persistGroupOrder(['都市', '玄幻', '言情'], storage)
  assert.deepEqual(loadGroupOrder(storage), ['都市', '玄幻', '言情'])
  clearGroupOrder(storage)
  assert.deepEqual(loadGroupOrder(storage), [])
})

test('load 容错：非法 JSON / 非数组 / 混入非字符串 → 空或过滤', () => {
  mem.set(SOURCE_GROUP_ORDER_KEY, 'not-json{')
  assert.deepEqual(loadGroupOrder(storage), [])
  mem.set(SOURCE_GROUP_ORDER_KEY, '{"a":1}')
  assert.deepEqual(loadGroupOrder(storage), [])
  mem.set(SOURCE_GROUP_ORDER_KEY, JSON.stringify(['甲', 42, '', '乙']))
  assert.deepEqual(loadGroupOrder(storage), ['甲', '乙'])
})

test('mergeGroupOrder：已保存顺序过滤失效组 + 新组追加；无保存顺序返回 null', () => {
  // 无保存顺序 → null（调用方回退默认排序）
  assert.equal(mergeGroupOrder([], ['甲', '乙']), null)
  // 正常合并
  assert.deepEqual(mergeGroupOrder(['乙', '甲'], ['甲', '乙', '丙']), ['乙', '甲', '丙'])
  // 已保存含失效组 → 过滤
  assert.deepEqual(mergeGroupOrder(['乙', '已删', '甲'], ['甲', '乙']), ['乙', '甲'])
})

test('reorderGroup：把 from 移到 to 位置，其余相对顺序不变', () => {
  const list = ['全部之外', '都市', '玄幻', '言情']
  assert.deepEqual(reorderGroup(list, '玄幻', '都市'), ['全部之外', '玄幻', '都市', '言情'])
  assert.deepEqual(reorderGroup(list, '都市', '言情'), ['全部之外', '玄幻', '言情', '都市'])
  // 源/目标不在列表或相同 → 原样
  assert.deepEqual(reorderGroup(list, '不存在', '都市'), list)
  assert.deepEqual(reorderGroup(list, '都市', '都市'), list)
  // 不修改入参
  assert.deepEqual(list, ['全部之外', '都市', '玄幻', '言情'])
})
