import { test, beforeEach } from 'node:test'
import assert from 'node:assert/strict'

/** node 环境无 localStorage：最小内存 stub（hanMode/chinese 仅用 getItem/setItem） */
const mem = new Map<string, string>()
;(globalThis as { localStorage?: unknown }).localStorage = {
  getItem: (k: string) => mem.get(k) ?? null,
  setItem: (k: string, v: string) => void mem.set(k, v),
  removeItem: (k: string) => void mem.delete(k),
}

// 必须在设置 localStorage stub 之后导入（模块初始化即读取 reader_han_mode）
const { hanMode, useHanMode, setGlobalHanMode, hanText, syncHanMode } = await import('./hanMode.ts')

beforeEach(() => mem.clear())

test('默认模式为 auto，且转换简体', () => {
  mem.clear()
  syncHanMode()
  assert.equal(hanMode.value, 'auto')
  assert.equal(hanText('繁體中文'), '繁体中文')
})

test('setGlobalHanMode 更新响应式状态并写入 localStorage（全站响应）', () => {
  setGlobalHanMode('trad')
  assert.equal(hanMode.value, 'trad')
  assert.equal(mem.get('reader_han_mode'), 'trad')
  assert.equal(hanText('简体字'), '簡體字')
  assert.equal(useHanMode().value, 'trad')
})

test('setGlobalHanMode 切回简体后转换恢复', () => {
  setGlobalHanMode('trad')
  assert.equal(hanText('历史'), '歷史')
  setGlobalHanMode('simp')
  assert.equal(hanText('歷史'), '历史')
})

test('syncHanMode 从 localStorage 重新同步（同标签页直写场景）', () => {
  setGlobalHanMode('simp')
  mem.set('reader_han_mode', 'trad') // 模拟其他写入方直改 localStorage
  assert.equal(hanMode.value, 'simp') // 响应式状态尚未感知
  syncHanMode()
  assert.equal(hanMode.value, 'trad')
  assert.equal(hanText('阅读'), '閱讀')
})

test('无效模式值回退 auto', () => {
  mem.set('reader_han_mode', 'bogus')
  syncHanMode()
  assert.equal(hanMode.value, 'auto')
})
