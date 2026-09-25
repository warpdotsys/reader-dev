import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  clearBackendReachableHooks,
  notifyBackendReachable,
  onBackendReachable,
} from './backendFlag.ts'

test('P2 backendDown 复位：注册的回调在 notifyBackendReachable 时全部触发', () => {
  clearBackendReachableHooks()
  let a = 0
  let b = 0
  onBackendReachable(() => {
    a++
  })
  onBackendReachable(() => {
    b++
  })
  notifyBackendReachable()
  assert.equal(a, 1)
  assert.equal(b, 1)
  // 可重复通知（每次后端请求成功都会触发）
  notifyBackendReachable()
  assert.equal(a, 2)
})

test('clearBackendReachableHooks 后通知为空操作', () => {
  clearBackendReachableHooks()
  let called = 0
  onBackendReachable(() => {
    called++
  })
  clearBackendReachableHooks()
  notifyBackendReachable()
  assert.equal(called, 0)
})
