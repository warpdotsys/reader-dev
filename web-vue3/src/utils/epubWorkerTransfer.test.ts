import assert from 'node:assert/strict'
import { test } from 'node:test'
import { expandEpubInWorker, type EpubArchiveWorker } from './epubWorker.ts'

function fixture() {
  const sent: unknown[] = []
  let terminated = 0
  const worker: EpubArchiveWorker = { onmessage: null, onerror: null,
    postMessage(value) { sent.push(value) }, terminate() { terminated++ } }
  const result = expandEpubInWorker(new Uint8Array([80, 75]), undefined, () => worker)
  const reply = (data: unknown) => worker.onmessage!({ data } as MessageEvent)
  for (const phase of ['ready', 'expanding', 'transferring']) reply({ phase })
  return { worker, result, reply, sent, get terminated() { return terminated } }
}

test('消费者只逐批确认，显式计数完成前不成功；完整结果保留每项原字节', async () => {
  const f = fixture()
  let resolved = false
  void f.result.then(() => { resolved = true })
  f.reply({ batch: 0, files: [['a.bin', new Uint8Array([1, 2])]] })
  await Promise.resolve()
  assert.equal(resolved, false)
  assert.deepEqual(f.sent.at(-1), { batchAccepted: 0 })
  f.reply({ batch: 1, files: [['b.bin', new Uint8Array([3])], ['empty.bin', new Uint8Array()]] })
  assert.deepEqual(f.sent.at(-1), { batchAccepted: 1 })
  f.reply({ complete: true, entries: 3, bytes: 3 })
  assert.deepEqual([...(await f.result)].map(([name, bytes]) => [name, [...bytes]]),
    [['a.bin', [1, 2]], ['b.bin', [3]], ['empty.bin', []]])
  assert.equal(f.terminated, 1)
})

test('消费者拒绝逆序、重复、混合及伪造完整计数，不接受旧的整批成功协议', async () => {
  for (const data of [{ batch: 2, files: [['b', new Uint8Array()]] },
    { batch: '1', files: [['b', new Uint8Array()]] }, { batch: 0, files: [['b', new Uint8Array()]] },
    { batch: 1, files: [['a', new Uint8Array()]] }, { batch: 1, files: [] },
    { batch: 1, files: [['b', new Uint8Array()]], extra: true },
    { files: [['b', new Uint8Array()]] }, { complete: true, entries: 0, bytes: 1 },
    { complete: true, entries: 1, bytes: 0 }, { complete: true, entries: 1, bytes: 1, extra: true },
    { complete: false, entries: 1, bytes: 1 }]) {
    const f = fixture()
    f.reply({ batch: 0, files: [['a', new Uint8Array([1])]] })
    const rejected = assert.rejects(f.result, /无效数据/)
    f.reply(data)
    await rejected
    assert.equal(f.terminated, 1)
    assert.equal(f.sent.length, 2)
  }
})

test('消费者核对每批条数、实际字节、子视图及重复路径；合法大项单独通过', async () => {
  for (const files of [Array.from({ length: 65 }, (_, n) => [String(n), new Uint8Array()]),
    [['a', new Uint8Array()], ['a', new Uint8Array()]], [['a', new Uint8Array([1, 2]).subarray(0, 1)]],
    [['a', new Uint8Array(1024 * 1024 + 1)], ['b', new Uint8Array(1024 * 1024)]],
    [['a.xhtml', new Uint8Array(4 * 1024 * 1024 + 1)]]]) {
    const f = fixture()
    const rejected = assert.rejects(f.result, /无效数据|超过原版排版上限/)
    f.reply({ batch: 0, files })
    await rejected
    assert.equal(f.sent.length, 1)
    assert.equal(f.terminated, 1)
  }
  const f = fixture()
  const data = new Uint8Array(3 * 1024 * 1024).fill(7)
  f.reply({ batch: 0, files: [['large.bin', data]] })
  f.reply({ complete: true, entries: 1, bytes: data.byteLength })
  assert.equal((await f.result).get('large.bin'), data)
})

test('消费者批次确认仍在创建时的15秒总期限内，任何中间结果都不延长', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] })
  const f = fixture()
  const rejected = assert.rejects(f.result, /解压超时.*等待解压结果传递/)
  t.mock.timers.tick(14999)
  f.reply({ batch: 0, files: [['a.bin', new Uint8Array([1])]] })
  assert.equal(f.terminated, 0)
  t.mock.timers.tick(1)
  await rejected
  assert.equal(f.terminated, 1)
  assert.equal(f.worker.onmessage, null)
})

test('消费者收到一批后取消，迟到结果/完成不能恢复文件或再确认；确认发送失败也拒绝', async () => {
  const controller = new AbortController()
  let posts = 0
  const worker: EpubArchiveWorker = { onmessage: null, onerror: null,
    postMessage() { posts++ }, terminate() {} }
  const result = expandEpubInWorker(new Uint8Array([80, 75]), controller.signal, () => worker)
  for (const phase of ['ready', 'expanding', 'transferring']) worker.onmessage!({ data: { phase } } as MessageEvent)
  worker.onmessage!({ data: { batch: 0, files: [['a', new Uint8Array([1])]] } } as MessageEvent)
  const late = worker.onmessage!
  const rejected = assert.rejects(result, { name: 'AbortError' })
  controller.abort()
  late({ data: { batch: 1, files: [['b', new Uint8Array([2])]] } } as MessageEvent)
  late({ data: { complete: true, entries: 2, bytes: 2 } } as MessageEvent)
  await rejected
  assert.equal(posts, 2)
  const f = fixture()
  f.worker.postMessage = () => { throw new Error('Generated ACK send failure') }
  const ackRejected = assert.rejects(f.result, /ACK send failure/)
  f.reply({ batch: 0, files: [['a', new Uint8Array([1])]] })
  await ackRejected
  assert.equal(f.terminated, 1)
})
