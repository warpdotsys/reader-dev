import assert from 'node:assert/strict'
import { test } from 'node:test'
import { h } from 'vue'
import { readerMessageOptions } from './messageOptions.ts'

test('普通文本仍是文本，不变成 HTML 或丢失业务错误', () => {
  const result = readerMessageOptions('生成业务错误 <b>原样文本</b>')
  assert.equal(result.message, '生成业务错误 <b>原样文本</b>')
  assert.notEqual(result.dangerouslyUseHTMLString, true)
  assert.equal(result.placement, 'bottom-right')
  assert.equal(result.showClose, true)
})

test('不改网络重复合并、警告类型、计时和回调，也不修改输入', () => {
  const onClose = () => {}
  const original = Object.freeze({ message: '生成通知', type: 'warning' as const,
    grouping: true, duration: 0, repeatNum: 2, onClose })
  const result = readerMessageOptions(original)
  for (const key of ['message', 'type', 'grouping', 'duration', 'repeatNum', 'onClose'] as const) {
    assert.equal(result[key], original[key])
  }
  assert.deepEqual(original, { message: '生成通知', type: 'warning', grouping: true,
    duration: 0, repeatNum: 2, onClose })
})

test('不同警告不强制合并，也没有丢弃额外消息的数量上限', () => {
  const warning = readerMessageOptions({ message: '另一条生成警告', grouping: false })
  assert.equal(warning.grouping, false)
  assert.equal('max' in warning, false)
  assert.equal(readerMessageOptions('普通警告').grouping, undefined)
})

test('VNode 和工厂的身份保留，不把它们误当选项对象', () => {
  const vnode = h('span', '生成节点')
  const factory = () => vnode
  assert.equal(readerMessageOptions(vnode).message, vnode)
  assert.equal(readerMessageOptions(factory).message, factory)
})

test('调用方样式和显示选项保留，但提示统一进自己的区域', () => {
  const original = Object.freeze({ message: '生成提示', customClass: 'generated-class',
    showClose: false, plain: true, offset: 32, placement: 'top' as const,
    appendTo: '#generated-old-target' })
  const result = readerMessageOptions(original)
  assert.equal(result.customClass, 'reader-message generated-class')
  assert.equal(result.showClose, false)
  assert.equal(result.plain, true)
  assert.equal(result.offset, 32)
  assert.equal(result.placement, 'bottom-right')
  assert.equal(result.appendTo, undefined)
  assert.equal(original.appendTo, '#generated-old-target')
})

test('没有消息的调用仍可由原库处理，不读取凭据配置', () => {
  assert.equal(readerMessageOptions(undefined).message, undefined)
  assert.equal(readerMessageOptions(undefined).customClass, 'reader-message')
})
