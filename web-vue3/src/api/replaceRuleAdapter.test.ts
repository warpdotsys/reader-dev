import { test } from 'node:test'
import assert from 'node:assert/strict'
import type { ReplaceRule } from '@/types'
import {
  fromLegacyReplaceRule,
  legacyReplaceRuleId,
  newReplaceRuleId,
  toLegacyReplaceRule,
} from './replaceRuleAdapter.ts'

test('legacy numeric fields map into the Vue 3 view model and preserve rule options', () => {
  const rule = fromLegacyReplaceRule({
    id: 1725000000123,
    name: '删除广告',
    group: '阅读',
    pattern: '广告词',
    replacement: '',
    scope: 'source.example',
    scopeTitle: false,
    scopeContent: true,
    isEnabled: false,
    isRegex: true,
    timeoutMillisecond: 4500,
    order: 7,
  })

  assert.equal(rule.id, '1725000000123')
  assert.equal(rule.find, '广告词')
  assert.equal(rule.replace, '')
  assert.equal(rule.enabled, false)
  assert.equal(rule.isRegex, true)
  assert.equal(rule.timeoutMillisecond, 4500)
  assert.equal(rule.order, 7)
})

test('Vue 3 rules serialize using the Java/Kotlin data class contract and defaults', () => {
  const rule: ReplaceRule = {
    id: '1725000000123', name: '过滤', find: '推广', replace: '', enabled: true, order: 2,
  }
  assert.deepEqual(toLegacyReplaceRule(rule), {
    id: 1725000000123,
    name: '过滤',
    group: null,
    pattern: '推广',
    replacement: '',
    scope: null,
    scopeTitle: false,
    scopeContent: true,
    isEnabled: true,
    isRegex: false,
    timeoutMillisecond: 3000,
    order: 2,
  })
})

test('preview-only string IDs become stable safe Long-compatible numbers', () => {
  const first = legacyReplaceRuleId('filter-1725000', '过滤规则\u0000正文\u0000')
  const second = legacyReplaceRuleId('filter-1725000', '过滤规则\u0000正文\u0000')
  assert.equal(first, second)
  assert.ok(Number.isSafeInteger(first) && first > 0)
  assert.match(newReplaceRuleId(1725000000000, 0.123), /^1725000000000123$/)
})
