import { after, test } from 'node:test'
import assert from 'node:assert/strict'
import { lazy } from './lazy.ts'
import type { DirectiveBinding } from 'vue'

/**
 * Node 无 DOM：用最小桩替代 document / IntersectionObserver。
 * node --test 每个测试文件独立进程，全局桩不影响其他文件。
 */
class FakeElement {
  src = ''
  classList = {
    set: new Set<string>(),
    add(c: string) {
      this.set.add(c)
    },
    contains(c: string) {
      return this.set.has(c)
    },
  }
}

class FakeIntersectionObserver {
  static instances: FakeIntersectionObserver[] = []
  els = new Set<Element>()
  cb: IntersectionObserverCallback
  disconnected = false
  constructor(cb: IntersectionObserverCallback) {
    this.cb = cb
    FakeIntersectionObserver.instances.push(this)
  }
  observe(el: Element) {
    this.els.add(el)
  }
  disconnect() {
    this.disconnected = true
    this.els.clear()
  }
  fire(intersecting = true) {
    const entries = Array.from(this.els).map((el) => ({
      isIntersecting: intersecting,
      target: el,
    })) as unknown as IntersectionObserverEntry[]
    this.cb(entries, this as unknown as IntersectionObserver)
  }
}

const prevObserver = globalThis.IntersectionObserver
const prevDoc = globalThis.document
globalThis.IntersectionObserver = FakeIntersectionObserver as unknown as typeof IntersectionObserver
globalThis.document = {
  createElement: () => new FakeElement(),
} as unknown as Document

after(() => {
  globalThis.IntersectionObserver = prevObserver
  globalThis.document = prevDoc
})

/** 断言对象型指令（mounted/updated/unmounted 钩子齐全） */
const dir = lazy as unknown as {
  mounted(el: HTMLImageElement, binding: DirectiveBinding<string>): void
  updated(el: HTMLImageElement, binding: DirectiveBinding<string>): void
  unmounted(el: HTMLImageElement): void
}
const binding = (value: string, oldValue?: string) =>
  ({ value, oldValue } as unknown as DirectiveBinding<string>)
const img = () => document.createElement('img') as unknown as HTMLImageElement

test('P2 v-lazy 闭包过期：绑定值在 intersect 前变化 → 回调加载最新值（非旧闭包值）', () => {
  FakeIntersectionObserver.instances = []
  const el = img()
  // 旧实现：mounted 闭包捕获 src=A；updated 换绑定值后回调仍写 A
  dir.mounted(el, binding('cover-a.jpg'))
  dir.updated(el, binding('cover-b.jpg', 'cover-a.jpg'))
  const obs = FakeIntersectionObserver.instances[0]
  assert.ok(obs)
  obs.fire(true) // 进入视口
  assert.equal(el.src.endsWith('cover-b.jpg'), true)
  assert.equal(el.classList.contains('is-loaded'), true)
  assert.equal(obs.disconnected, true) // 加载后断开 observer
})

test('P2 v-lazy：intersect 后已加载，updated 换图直接生效（保留 is-loaded）', () => {
  FakeIntersectionObserver.instances = []
  const el = img()
  dir.mounted(el, binding('cover-a.jpg'))
  FakeIntersectionObserver.instances[0].fire(true)
  assert.equal(el.src.endsWith('cover-a.jpg'), true)
  assert.equal(el.classList.contains('is-loaded'), true)
  dir.updated(el, binding('cover-b.jpg', 'cover-a.jpg'))
  assert.equal(el.src.endsWith('cover-b.jpg'), true)
  assert.equal(el.classList.contains('is-loaded'), true) // 不闪烁
})

test('P2 v-lazy：mounted 未加载时 updated 仅记录最新值，不提前设 src', () => {
  FakeIntersectionObserver.instances = []
  const el = img()
  dir.mounted(el, binding('cover-a.jpg'))
  dir.updated(el, binding('cover-b.jpg', 'cover-a.jpg'))
  // 未进入视口：src 保持初始空（懒加载语义），仅记录最新值
  assert.equal(el.src, '')
  FakeIntersectionObserver.instances[0].fire(true)
  assert.equal(el.src.endsWith('cover-b.jpg'), true)
})

test('v-lazy：unmounted 断开 observer；空 src 不创建 observer', () => {
  FakeIntersectionObserver.instances = []
  const el = img()
  dir.mounted(el, binding(''))
  assert.equal(FakeIntersectionObserver.instances.length, 0)

  const el2 = img()
  dir.mounted(el2, binding('cover-a.jpg'))
  dir.unmounted(el2)
  assert.equal(FakeIntersectionObserver.instances[0].disconnected, true)
})
