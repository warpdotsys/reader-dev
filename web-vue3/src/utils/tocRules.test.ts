import test from 'node:test'
import assert from 'node:assert/strict'
import { isDefaultTxtTocRuleId, txtTocRuleKey } from './tocRules.ts'

test('recognizes Java/Kotlin negative numeric ids as immutable built-in TOC rules', () => {
  assert.equal(isDefaultTxtTocRuleId(-1), true)
  assert.equal(isDefaultTxtTocRuleId(-12), true)
  assert.equal(isDefaultTxtTocRuleId('-12'), true)
})

test('continues recognizing the Rust snapshot default id format', () => {
  assert.equal(isDefaultTxtTocRuleId('default-1'), true)
})

test('keeps nonnegative numeric and custom string ids editable', () => {
  assert.equal(isDefaultTxtTocRuleId(0), false)
  assert.equal(isDefaultTxtTocRuleId(1740000000000), false)
  assert.equal(isDefaultTxtTocRuleId('custom-rule'), false)
  assert.equal(isDefaultTxtTocRuleId(undefined), false)
})

test('normalizes numeric and string ids for selection state', () => {
  assert.equal(txtTocRuleKey(-3), '-3')
  assert.equal(txtTocRuleKey('custom-rule'), 'custom-rule')
  assert.equal(txtTocRuleKey(null), '')
})
