import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  accumulateDaily,
  last7Days,
  localDateStr,
  parseDailyStats,
} from './dailyStats.ts'

test('GAP 110：localDateStr 本地时区 YYYY-MM-DD', () => {
  assert.equal(localDateStr(new Date(2025, 0, 5)), '2025-01-05')
  assert.equal(localDateStr(new Date(2025, 11, 31)), '2025-12-31')
})

test('GAP 110：parseDailyStats 容错（null/坏 JSON/非对象/非法键/非正数）', () => {
  assert.deepEqual(parseDailyStats(null), {})
  assert.deepEqual(parseDailyStats('not-json'), {})
  assert.deepEqual(parseDailyStats('[1,2]'), {})
  assert.deepEqual(parseDailyStats('{"bad": 3}'), {})
  assert.deepEqual(parseDailyStats('{"2025-01-05": -3}'), {})
  assert.deepEqual(parseDailyStats('{"2025-01-05": 120, "junk": 9}'), { '2025-01-05': 120 })
})

test('GAP 110：accumulateDaily 同日累计 + 过期清理（>31 天删除，31 天保留）', () => {
  const now = new Date(2025, 0, 5, 12, 0, 0) // 2025-01-05
  const m1 = accumulateDaily({ '2025-01-05': 30 }, 90, now)
  assert.deepEqual(m1, { '2025-01-05': 120 })

  // 32 天前（2024-12-04）删除；31 天前（2024-12-05）保留
  const m2 = accumulateDaily(
    { '2024-12-04': 10, '2024-12-05': 20, '2025-01-04': 40 },
    10,
    now,
  )
  assert.deepEqual(m2, { '2024-12-05': 20, '2025-01-04': 40, '2025-01-05': 10 })

  // 非正秒数忽略（原表引用不变）
  const m3 = accumulateDaily({ '2025-01-05': 30 }, 0, now)
  assert.deepEqual(m3, { '2025-01-05': 30 })
})

test('GAP 110：last7Days 含今天、从 6 天前起、缺省日补 0', () => {
  const now = new Date(2025, 0, 10, 9, 0, 0) // 2025-01-10（周五）
  const days = last7Days({ '2025-01-08': 60, '2025-01-10': 300 }, now)
  assert.equal(days.length, 7)
  assert.equal(days[0].date, '2025-01-04')
  assert.equal(days[0].seconds, 0)
  assert.equal(days[4].date, '2025-01-08')
  assert.equal(days[4].seconds, 60)
  assert.equal(days[6].date, '2025-01-10')
  assert.equal(days[6].seconds, 300)
  // 负数/小数钳制
  assert.deepEqual(last7Days({ '2025-01-10': -5 }, now)[6], { date: '2025-01-10', seconds: 0 })
})
