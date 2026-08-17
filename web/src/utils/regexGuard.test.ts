import { test } from 'node:test'
import assert from 'node:assert/strict'
import { checkTestRegex, MAX_TEST_REGEX_LEN } from './regexGuard.ts'

/* ================= P1-5：ReplaceRuleView 测试正则长度限制 ================= */

test('P1-5：200 字符以内正则放行', () => {
  assert.equal(checkTestRegex('第(.+?)章'), null)
  assert.equal(checkTestRegex('a'.repeat(200)), null)
})

test('P1-5：超过 200 字符拒绝并提示', () => {
  const err = checkTestRegex('a'.repeat(201))
  assert.ok(err !== null)
  assert.ok(err.includes(`${MAX_TEST_REGEX_LEN}`))
  assert.ok(err.includes('上限'))
  // 典型恶意模式（超长嵌套量词）应被拒
  const evil = `(a+)+${'b'.repeat(220)}`
  assert.ok(checkTestRegex(evil) !== null)
})

test('P1-5：空串与边界值', () => {
  assert.equal(checkTestRegex(''), null)
  assert.equal(checkTestRegex(''.padEnd(200, 'x')), null)
})
