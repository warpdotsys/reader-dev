import { test, beforeEach } from 'node:test'
import assert from 'node:assert/strict'

/**
 * wakeLock 模块惰性读取 navigator（每次调用现查），测试通过替换全局 navigator 模拟：
 * - 不支持（无 wakeLock 属性）→ 静默返回 false；
 * - 支持 → request 成功持有 / release 释放；
 * - request 抛错（如浏览器拒绝）→ 静默返回 false；
 * - 浏览器后台自动释放（release 事件）→ 内部置空。
 */

let requested: string[] = []
let autoReleaseOnRequest = false
let autoReleaseCb: (() => void) | null = null
const released: Array<{ id: number }> = []
let failNext = false
let seq = 0

const fakeWakeLock = {
  request: async (type: string) => {
    requested.push(type)
    if (failNext) {
      failNext = false
      throw new Error('NotAllowedError')
    }
    const handle = { id: ++seq, released: false }
    const release = async () => {
      handle.released = true
      released.push(handle)
    }
    return {
      release,
      // 模拟浏览器后台自动释放：request 完成后异步触发 release 事件
      addEventListener: (type: string, cb: () => void) => {
        if (type === 'release' && autoReleaseOnRequest) {
          autoReleaseCb = cb
          setTimeout(cb, 0)
        }
      },
    } as unknown as { release: () => Promise<void>; addEventListener?: (t: string, cb: () => void) => void }
  },
}

function setNavigator(hasWakeLock: boolean) {
  const nav = hasWakeLock ? { wakeLock: fakeWakeLock } : {}
  Object.defineProperty(globalThis, 'navigator', { value: nav, configurable: true })
}

const { requestWakeLock, releaseWakeLock, isWakeLockHeld } = await import('./wakeLock.ts')

beforeEach(() => {
  requested = []
  released.length = 0
  autoReleaseOnRequest = false
  autoReleaseCb = null
  failNext = false
  seq = 0
})

test('不支持 Wake Lock 的环境：静默返回 false，不持有', async () => {
  setNavigator(false)
  assert.equal(await requestWakeLock(), false)
  assert.equal(isWakeLockHeld(), false)
  // release 空操作不抛错
  await releaseWakeLock()
  assert.equal(isWakeLockHeld(), false)
})

test('支持环境：request 持有，release 释放', async () => {
  setNavigator(true)
  assert.equal(await requestWakeLock(), true)
  assert.deepEqual(requested, ['screen'])
  assert.equal(isWakeLockHeld(), true)
  await releaseWakeLock()
  assert.equal(isWakeLockHeld(), false)
  assert.equal(released.length, 1)
})

test('重复 request：先释放旧锁再请求新锁（仅持有一个）', async () => {
  setNavigator(true)
  assert.equal(await requestWakeLock(), true)
  assert.equal(await requestWakeLock(), true)
  assert.equal(isWakeLockHeld(), true)
  assert.equal(released.length, 1)
  assert.equal(requested.length, 2)
})

test('request 抛错（浏览器拒绝）：静默返回 false，不持有', async () => {
  setNavigator(true)
  failNext = true
  assert.equal(await requestWakeLock(), false)
  assert.equal(isWakeLockHeld(), false)
})

test('浏览器后台自动释放（release 事件）：内部置空，可再次请求', async () => {
  setNavigator(true)
  autoReleaseOnRequest = true
  assert.equal(await requestWakeLock(), true)
  // request 后异步触发 release 事件 → 内部置空
  await new Promise((r) => setTimeout(r, 10))
  assert.equal(isWakeLockHeld(), false)
  assert.equal(autoReleaseCb !== null, true)
  autoReleaseOnRequest = false
  assert.equal(await requestWakeLock(), true)
  assert.equal(isWakeLockHeld(), true)
})
