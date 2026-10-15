import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ProgressWriteBarrier } from './progressBarrier.ts'

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: Error) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

test('进度屏障：无在途写入时不阻止新章读取', async () => {
  await new ProgressWriteBarrier().settle()
})

test('进度屏障：未完成的旧写入必须先于新章读取', async () => {
  const barrier = new ProgressWriteBarrier()
  const old = deferred<string>()
  const write = barrier.track(old.promise)
  let ready = false
  const read = barrier.settle().then(() => { ready = true })
  await Promise.resolve()
  assert.equal(ready, false)
  old.resolve('saved')
  assert.equal(await write, 'saved')
  await read
  assert.equal(ready, true)
  await barrier.settle()
})

test('进度屏障：多条已发送写入全部完成才能读取', async () => {
  const barrier = new ProgressWriteBarrier()
  const first = deferred<void>(), second = deferred<void>()
  barrier.track(first.promise)
  barrier.track(second.promise)
  let ready = false
  const read = barrier.settle().then(() => { ready = true })
  second.resolve()
  await Promise.resolve()
  assert.equal(ready, false)
  first.resolve()
  await read
  assert.equal(ready, true)
})

test('进度屏障：网络失败不伪报成功，也不无限阻塞下一章', async () => {
  const barrier = new ProgressWriteBarrier()
  const old = deferred<void>()
  const write = barrier.track(old.promise)
  const observedFailure = assert.rejects(write, /failed/)
  const read = barrier.settle()
  old.reject(new Error('failed'))
  await observedFailure
  await read
  await barrier.settle()
})

test('进度屏障：读取开始后新增的写入不改变既有快照', async () => {
  const barrier = new ProgressWriteBarrier()
  const old = deferred<void>(), fresh = deferred<void>()
  barrier.track(old.promise)
  const read = barrier.settle()
  const freshWrite = barrier.track(fresh.promise)
  old.resolve()
  await read
  fresh.resolve()
  await freshWrite
  await barrier.settle()
})
