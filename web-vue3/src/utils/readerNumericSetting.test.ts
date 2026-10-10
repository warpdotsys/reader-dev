import { test } from 'node:test'
import assert from 'node:assert/strict'
import { parseReaderNumericSetting } from './readerNumericSetting.ts'

test('未保存的听书音量使用 100%，不能将 null 当成显式静音', () => {
  assert.equal(parseReaderNumericSetting(null, 0, 200, 100), 100)
})
test('用户明确保存的 0 音量保留，不能在升级时擅自取消静音', () => {
  assert.equal(parseReaderNumericSetting('0', 0, 200, 100), 0)
})
test('空值、空白、非数字、无穷设置使用默认值', () => {
  for (const raw of ['', ' ', '\n\t', 'bad', 'NaN', 'Infinity', '-Infinity']) {
    assert.equal(parseReaderNumericSetting(raw, 0, 200, 100), 100)
  }
})
test('越界不夹成新用户偏好，使用原默认值', () => {
  for (const raw of ['-1', '201']) assert.equal(parseReaderNumericSetting(raw, 0, 200, 100), 100)
})
test('边界及已有正常设置原值保留', () => {
  for (const raw of ['0', '50', '100', '200']) assert.equal(parseReaderNumericSetting(raw, 0, 200, 100), Number(raw))
})
test('步长舍入保持原语义', () => {
  assert.equal(parseReaderNumericSetting('273', 0, 1000, 260, 10), 270)
  assert.equal(parseReaderNumericSetting('1.24', 0.5, 2, 1, 0.1), 1.2000000000000002)
})
