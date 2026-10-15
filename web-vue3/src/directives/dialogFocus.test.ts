import { test } from 'node:test'
import assert from 'node:assert/strict'
import { dialogFocus, dialogTabBoundary } from './dialogFocus.ts'

// 不替换全局 document：每个场景使用独立文档，测试结束必须拆掉监听器。
class FakeDocument {
  activeElement: FakeElement | null = null
  listeners = new Map<string, Set<(event: never) => void>>()
  defaultView = { getComputedStyle: (item: FakeElement) => ({ visibility: item.visibility }) }
  addEventListener(name: string, handler: (event: never) => void) {
    const handlers = this.listeners.get(name) ?? new Set()
    handlers.add(handler)
    this.listeners.set(name, handlers)
  }
  removeEventListener(name: string, handler: (event: never) => void) { this.listeners.get(name)?.delete(handler) }
  emit(name: string, event: unknown) {
    for (const handler of this.listeners.get(name) ?? []) handler(event as never)
  }
}

class FakeElement {
  ownerDocument: FakeDocument
  parent: FakeElement | null = null
  children: FakeElement[] = []
  tabIndex = 0
  isConnected = true
  disabled = false
  hidden = false
  visibility = 'visible'
  modal = false
  constructor(doc: FakeDocument, parent?: FakeElement) {
    this.ownerDocument = doc
    if (parent) { this.parent = parent; parent.children.push(this) }
  }
  getClientRects() { return this.hidden ? [] : [{}] }
  matches(selector: string) { return selector === ':disabled' && this.disabled }
  closest(selector: string): FakeElement | null {
    if (selector.includes('aria-modal')) return this.modal ? this : this.parent?.closest(selector) ?? null
    if (selector.includes('[hidden]')) return this.hidden ? this : this.parent?.closest(selector) ?? null
    return null
  }
  contains(item: FakeElement | null): boolean {
    return !!item && (item === this || this.children.some(child => child.contains(item)))
  }
  querySelectorAll(): FakeElement[] {
    return this.children.flatMap(child => [child, ...child.querySelectorAll()])
  }
  focus() {
    this.ownerDocument.activeElement = this
    this.ownerDocument.emit('focusin', { target: this })
  }
}

const directive = dialogFocus as unknown as {
  mounted(element: FakeElement, binding?: { value?: FakeElement }): void
  updated(element: FakeElement): void
  beforeUnmount(element: FakeElement): void
}

function fixture() {
  const doc = new FakeDocument()
  const opener = new FakeElement(doc)
  opener.focus()
  const dialog = new FakeElement(doc)
  dialog.modal = true
  dialog.tabIndex = -1
  const first = new FakeElement(doc, dialog)
  const last = new FakeElement(doc, dialog)
  return { doc, opener, dialog, first, last }
}

function key(doc: FakeDocument, name = 'Tab', shiftKey = false) {
  const event = { key: name, shiftKey, defaultPrevented: false, altKey: false, ctrlKey: false, metaKey: false,
    preventDefault() { this.defaultPrevented = true } }
  doc.emit('keydown', event)
  return event
}

test('Tab 仅接管首尾边界、无控件和外部焦点', () => {
  assert.equal(dialogTabBoundary(3, 1, false), null)
  assert.equal(dialogTabBoundary(3, 1, true), null)
  assert.equal(dialogTabBoundary(3, 2, false), 0)
  assert.equal(dialogTabBoundary(3, 0, true), 2)
  assert.equal(dialogTabBoundary(3, -1, false), 0)
  assert.equal(dialogTabBoundary(3, -1, true), 2)
  assert.equal(dialogTabBoundary(0, -1, true), -1)
})

test('打开聚焦、双向循环、Escape 留给业务、关闭归还焦点并移除监听', () => {
  const { doc, opener, dialog, first, last } = fixture()
  directive.mounted(dialog)
  assert.equal(doc.activeElement, first)
  assert.equal(key(doc, 'Escape').defaultPrevented, false)
  assert.equal(key(doc, 'Tab', true).defaultPrevented, true)
  assert.equal(doc.activeElement, last)
  assert.equal(key(doc).defaultPrevented, true)
  assert.equal(doc.activeElement, first)
  assert.equal(key(doc).defaultPrevented, false) // 内部正常 Tab 不拦截
  directive.beforeUnmount(dialog)
  assert.equal(doc.activeElement, opener)
  assert.equal(doc.listeners.get('keydown')?.size, 0)
  assert.equal(doc.listeners.get('focusin')?.size, 0)
})

test('每次查询排除隐藏/禁用控件，更新后不会留在失效按钮', () => {
  const { doc, dialog, first, last } = fixture()
  first.hidden = true
  directive.mounted(dialog)
  assert.equal(doc.activeElement, last)
  first.hidden = false
  last.disabled = true
  directive.updated(dialog)
  assert.equal(doc.activeElement, first)
  assert.equal(key(doc).defaultPrevented, true)
  assert.equal(doc.activeElement, first)
  directive.beforeUnmount(dialog)
})

test('CSS 不可见与 tabindex=-1 不成为 Tab 目标，空窗聚焦容器', () => {
  const { doc, dialog, first, last } = fixture()
  first.visibility = 'hidden'
  last.tabIndex = -1
  directive.mounted(dialog)
  assert.equal(doc.activeElement, dialog)
  assert.equal(key(doc).defaultPrevented, true)
  assert.equal(doc.activeElement, dialog)
  directive.beforeUnmount(dialog)
})

test('外部普通元素无法夺走活动弹窗的焦点', () => {
  const { doc, opener, dialog, first } = fixture()
  directive.mounted(dialog)
  opener.focus()
  assert.equal(doc.activeElement, first)
  directive.beforeUnmount(dialog)
})

test('文件选择器的隐藏输入不成为恢复目标，使用显式的原始按钮', () => {
  const { doc, opener, dialog } = fixture()
  const fileInput = new FakeElement(doc)
  fileInput.hidden = true
  fileInput.focus()
  directive.mounted(dialog, { value: opener })
  directive.beforeUnmount(dialog)
  assert.equal(doc.activeElement, opener)
})

test('嵌套自定义弹窗只由最上层接管，关闭恢复下层触发按钮', () => {
  const { doc, dialog, first } = fixture()
  directive.mounted(dialog)
  const nested = new FakeElement(doc)
  nested.modal = true
  nested.tabIndex = -1
  const nestedButton = new FakeElement(doc, nested)
  directive.mounted(nested)
  assert.equal(doc.activeElement, nestedButton)
  assert.equal(key(doc).defaultPrevented, true)
  assert.equal(doc.activeElement, nestedButton)
  directive.beforeUnmount(nested)
  assert.equal(doc.activeElement, first)
  directive.beforeUnmount(dialog)
})

test('外部组件模态窗有自己的焦点管理，不被旧弹窗抢回', () => {
  const { doc, dialog } = fixture()
  directive.mounted(dialog)
  const external = new FakeElement(doc)
  external.modal = true
  const externalInput = new FakeElement(doc, external)
  externalInput.focus()
  assert.equal(doc.activeElement, externalInput)
  assert.equal(key(doc).defaultPrevented, false)
  directive.updated(dialog)
  assert.equal(doc.activeElement, externalInput)
  directive.beforeUnmount(dialog)
  assert.equal(doc.activeElement, externalInput)
  assert.equal(doc.listeners.get('keydown')?.size, 0)
})

test('销毁下层不会抢走上层焦点，失效触发按钮不会被聚焦', () => {
  const { doc, dialog, first } = fixture()
  directive.mounted(dialog)
  const nested = new FakeElement(doc)
  nested.modal = true
  const button = new FakeElement(doc, nested)
  directive.mounted(nested)
  directive.beforeUnmount(dialog)
  assert.equal(doc.activeElement, button)
  first.isConnected = false
  dialog.isConnected = false
  directive.beforeUnmount(nested)
  assert.equal(doc.listeners.get('keydown')?.size, 0)
  assert.equal(doc.listeners.get('focusin')?.size, 0)
})
