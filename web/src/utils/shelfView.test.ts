import { test } from 'node:test'
import assert from 'node:assert/strict'
import { parseShelfView, shelfViewMetrics, SHELF_VIEW_KEY } from './shelfView.ts'

test('parseShelfView：' + SHELF_VIEW_KEY + " 支持 'wall'，非法值回落 grid", () => {
  assert.equal(parseShelfView('grid'), 'grid')
  assert.equal(parseShelfView('list'), 'list')
  assert.equal(parseShelfView('wall'), 'wall')
  assert.equal(parseShelfView(null), 'grid')
  assert.equal(parseShelfView(undefined), 'grid')
  assert.equal(parseShelfView('bogus'), 'grid')
  assert.equal(parseShelfView(''), 'grid')
})

test('墙模式尺寸：大卡片 + 宽间距（桌面 232px / 窄屏 158px，间距 40/48）', () => {
  const desk = shelfViewMetrics('wall', false)
  assert.equal(desk.cardMinW, 232)
  assert.equal(desk.colGap, 40)
  assert.equal(desk.rowGap, 48)
  assert.equal(desk.metaH, 96)

  const narrow = shelfViewMetrics('wall', true)
  assert.equal(narrow.cardMinW, 158)
  // 墙模式不受密度影响
  assert.equal(shelfViewMetrics('wall', false, 's').cardMinW, 232)
})

test('网格模式：密度影响卡片宽（桌面 128/160/204，窄屏 96/120/150）', () => {
  assert.equal(shelfViewMetrics('grid', false, 's').cardMinW, 128)
  assert.equal(shelfViewMetrics('grid', false, 'm').cardMinW, 160)
  assert.equal(shelfViewMetrics('grid', false, 'l').cardMinW, 204)
  assert.equal(shelfViewMetrics('grid', true, 's').cardMinW, 96)
  assert.equal(shelfViewMetrics('grid', true, 'm').cardMinW, 120)
  assert.equal(shelfViewMetrics('grid', true, 'l').cardMinW, 150)
})

test('列表模式：单列占位宽 1px，行高兜底 76', () => {
  const m = shelfViewMetrics('list', false)
  assert.equal(m.cardMinW, 1)
  assert.equal(m.metaH, 76)
})

test('虚拟滚动行高按墙尺寸计算：封面 4:3 + 元信息区', () => {
  const m = shelfViewMetrics('wall', false)
  // 列数 = floor((容器宽 + 列距) / (卡片最小宽 + 列距))
  const wrapW = 1200
  const cols = Math.max(1, Math.floor((wrapW + m.colGap) / (m.cardMinW + m.colGap)))
  const cw = (wrapW - (cols - 1) * m.colGap) / cols
  const rowH = Math.round((cw * 4) / 3 + m.metaH)
  // 与网格模式同容器对比：墙行高显著更高
  const gm = shelfViewMetrics('grid', false, 'm')
  const gCols = Math.max(1, Math.floor((wrapW + gm.colGap) / (gm.cardMinW + gm.colGap)))
  const gcw = (wrapW - (gCols - 1) * gm.colGap) / gCols
  const gRowH = Math.round((gcw * 4) / 3 + gm.metaH)
  assert.ok(rowH > gRowH, `墙行高 ${rowH} 应大于网格行高 ${gRowH}`)
  assert.equal(cols, 4) // 1200px 下墙模式 4 列（232px 起）
})
