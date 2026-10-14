import { test } from 'node:test'
import assert from 'node:assert/strict'
import { expandEpubInWorker, type EpubArchiveWorker } from './epubWorker.ts'

function fixtureWorker() {
  let terminated = 0
  let sent: ArrayBuffer | null = null
  let sentCount = 0
  let transferred: Transferable[] = []
  const worker: EpubArchiveWorker = {
    onmessage: null, onerror: null,
    postMessage(value, transfer) { sent = value; transferred = transfer; sentCount++ },
    terminate() { terminated++ },
  }
  return { worker, get terminated() { return terminated }, get sent() { return sent },
    get sentCount() { return sentCount }, get transferred() { return transferred } }
}

test('EPUB Worker：输入/输出保持完整，成功后终止并清除回调', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([80, 75])
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  assert.equal(fixture.sent, null)
  fixture.worker.onmessage!({ data: { phase: 'ready' } } as MessageEvent)
  assert.equal(fixture.sent, input.buffer)
  assert.deepEqual(fixture.transferred, [input.buffer])
  fixture.worker.onmessage!({ data: { phase: 'expanding' } } as MessageEvent)
  fixture.worker.onmessage!({ data: { phase: 'transferring' } } as MessageEvent)
  fixture.worker.onmessage!({ data: { files: [['a', new Uint8Array([7])]] } } as MessageEvent)
  assert.deepEqual((await result).get('a'), new Uint8Array([7]))
  assert.equal(fixture.terminated, 1)
  assert.equal(fixture.worker.onmessage, null)
})

test('EPUB Worker：切换/卸载取消，迟到结果不能恢复已终止任务', async () => {
  const fixture = fixtureWorker()
  const controller = new AbortController()
  const result = expandEpubInWorker(new Uint8Array([80, 75]), controller.signal, () => fixture.worker)
  const observed = assert.rejects(result, { name: 'AbortError' })
  controller.abort()
  await observed
  assert.equal(fixture.terminated, 1)
  assert.equal(fixture.worker.onmessage, null)
  await assert.rejects(expandEpubInWorker(new Uint8Array(), controller.signal,
    () => { throw new Error('Must not start') }), { name: 'AbortError' })
})

test('EPUB Worker：超时终止、解压错误和无效协议均拒绝，不伪报空成功', async () => {
  const timeout = fixtureWorker()
  await assert.rejects(expandEpubInWorker(new Uint8Array(), undefined, () => timeout.worker, 5), /解压超时/)
  assert.equal(timeout.terminated, 1)
  for (const data of [{ error: 'EPUB 生成超限' }, { files: [['a', 'not bytes']] }, null]) {
    const fixture = fixtureWorker()
    const result = expandEpubInWorker(new Uint8Array(), undefined, () => fixture.worker)
    fixture.worker.onmessage!({ data } as MessageEvent)
    await assert.rejects(result, /EPUB/)
    assert.equal(fixture.terminated, 1)
  }
})

test('EPUB Worker：发送失败及子视图不转移无关字节', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([99, 80, 75, 99])
  const result = expandEpubInWorker(input.subarray(1, 3), undefined, () => fixture.worker)
  assert.equal(fixture.sent, null)
  fixture.worker.onmessage!({ data: { phase: 'ready' } } as MessageEvent)
  assert.deepEqual(new Uint8Array(fixture.sent!), new Uint8Array([80, 75]))
  assert.notEqual(fixture.sent, input.buffer)
  fixture.worker.onmessage!({ data: { error: 'EPUB generated' } } as MessageEvent)
  await assert.rejects(result)
  const failed = fixtureWorker()
  failed.worker.postMessage = () => { throw new Error('Generated send failure') }
  const failure = expandEpubInWorker(new Uint8Array(), undefined, () => failed.worker)
  const observed = assert.rejects(failure, /send failure/)
  failed.worker.onmessage!({ data: { phase: 'ready' } } as MessageEvent)
  await observed
  assert.equal(failed.terminated, 1)
})

test('EPUB Worker：固定阶段确认不完成任务，最终文件与转移协议不变', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([80, 75])
  let completed = false
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  void result.then(() => { completed = true })
  assert.equal(fixture.sentCount, 0)
  for (const phase of ['ready', 'expanding', 'transferring']) {
    fixture.worker.onmessage!({ data: { phase } } as MessageEvent)
    await Promise.resolve()
    assert.equal(completed, false)
    assert.equal(fixture.terminated, 0)
  }
  assert.equal(fixture.sent, input.buffer)
  assert.equal(fixture.sentCount, 1)
  assert.deepEqual(fixture.transferred, [input.buffer])
  fixture.worker.onmessage!({ data: { files: [['a', new Uint8Array([7])]] } } as MessageEvent)
  assert.deepEqual((await result).get('a'), new Uint8Array([7]))
  assert.equal(fixture.terminated, 1)
})

for (const targetPhase of ['starting', 'ready', 'expanding', 'transferring'] as const) {
  test(`EPUB Worker：${targetPhase} 阶段仍在原总期限结束，不重置超时`, async t => {
    t.mock.timers.enable({ apis: ['setTimeout'] })
    const fixture = fixtureWorker()
    const result = expandEpubInWorker(new Uint8Array(), undefined, () => fixture.worker)
    const expected = { starting: '等待 Worker 启动', ready: '等待接收文件', expanding: '解压文件', transferring: '等待解压结果传递' }
    const observed = assert.rejects(result, new RegExp(`解压超时.*阶段：${expected[targetPhase]}`))
    t.mock.timers.tick(14998)
    assert.equal(fixture.sentCount, 0)
    if (targetPhase !== 'starting') fixture.worker.onmessage!({ data: { phase: 'ready' } } as MessageEvent)
    t.mock.timers.tick(1)
    if (targetPhase === 'expanding' || targetPhase === 'transferring') fixture.worker.onmessage!({ data: { phase: 'expanding' } } as MessageEvent)
    if (targetPhase === 'transferring') fixture.worker.onmessage!({ data: { phase: 'transferring' } } as MessageEvent)
    assert.equal(fixture.terminated, 0)
    t.mock.timers.tick(1)
    await observed
    assert.equal(fixture.sentCount, targetPhase === 'starting' ? 0 : 1)
    assert.equal(fixture.terminated, 1)
    assert.equal(fixture.worker.onmessage, null)
    assert.equal(fixture.worker.onerror, null)
  })
}

test('EPUB Worker：未知、混合、逆序和重复阶段拒绝，不能伪报成功', async () => {
  for (const phases of [
    [{ phase: 'unknown' }], [{ phase: undefined }], [{ phase: 'expanding' }],
    [{ phase: 'ready', files: [] }], [{ phase: 'ready', error: 'generated' }],
    [{ phase: 'ready' }, { phase: 'ready' }],
    [{ phase: 'ready' }, { phase: 'expanding' }, { phase: 'ready' }],
    [{ phase: 'transferring' }], [{ phase: 'ready' }, { phase: 'transferring' }],
    [{ phase: 'ready' }, { phase: 'expanding' }, { phase: 'transferring', files: [] }],
    [{ phase: 'ready' }, { phase: 'expanding' }, { phase: 'transferring' }, { phase: 'transferring' }],
    [{ phase: 'ready' }, { phase: 'expanding' }, { phase: 'transferring' }, { phase: 'expanding' }],
  ]) {
    const fixture = fixtureWorker()
    const result = expandEpubInWorker(new Uint8Array(), undefined, () => fixture.worker)
    const observed = assert.rejects(result, /无效阶段/)
    for (const data of phases) fixture.worker.onmessage!({ data } as MessageEvent)
    await observed
    assert.equal(fixture.terminated, 1)
  }
})

test('EPUB Worker：未 ready 前既不排入整本输入，也不复制子视图', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([99, 80, 75, 99]).subarray(1, 3)
  let copies = 0
  const originalSlice = input.slice.bind(input)
  input.slice = (...args) => { copies++; return originalSlice(...args) }
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  assert.equal(fixture.sentCount, 0)
  assert.equal(copies, 0)
  fixture.worker.onmessage!({ data: { phase: 'ready' } } as MessageEvent)
  assert.equal(fixture.sentCount, 1)
  assert.equal(copies, 1)
  assert.deepEqual(new Uint8Array(fixture.sent!), new Uint8Array([80, 75]))
  fixture.worker.onmessage!({ data: { phase: 'expanding' } } as MessageEvent)
  fixture.worker.onmessage!({ data: { phase: 'transferring' } } as MessageEvent)
  fixture.worker.onmessage!({ data: { files: [] } } as MessageEvent)
  assert.deepEqual(await result, new Map())
})

test('EPUB Worker：ready 前取消及保存的迟到 ready 回调均不能发送或复制', async () => {
  const fixture = fixtureWorker()
  const controller = new AbortController()
  const input = new Uint8Array([99, 80, 75]).subarray(1)
  input.slice = () => { throw new Error('Must not copy cancelled input') }
  const result = expandEpubInWorker(input, controller.signal, () => fixture.worker)
  const lateReady = fixture.worker.onmessage!
  const observed = assert.rejects(result, { name: 'AbortError' })
  controller.abort()
  lateReady({ data: { phase: 'ready' } } as MessageEvent)
  await observed
  assert.equal(fixture.sentCount, 0)
  assert.equal(fixture.terminated, 1)
})

test('EPUB Worker：ready 前超时后迟到确认不能发送，原输入未被分离', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] })
  const fixture = fixtureWorker()
  const input = new Uint8Array([80, 75])
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  const lateReady = fixture.worker.onmessage!
  const observed = assert.rejects(result, /解压超时.*等待 Worker 启动/)
  t.mock.timers.tick(15000)
  lateReady({ data: { phase: 'ready' } } as MessageEvent)
  await observed
  assert.equal(fixture.sentCount, 0)
  assert.equal(input.byteLength, 2)
  assert.equal(fixture.terminated, 1)
})

test('EPUB Worker：实际转移只在 ready 后分离独占输入，成功不保留回调', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([80, 75])
  let received: ArrayBuffer | null = null
  fixture.worker.postMessage = (value, transfer) => { received = structuredClone(value, { transfer }) }
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  assert.equal(input.byteLength, 2)
  assert.equal(received, null)
  fixture.worker.onmessage!({ data: { phase: 'ready' } } as MessageEvent)
  assert.equal(input.byteLength, 0)
  assert.deepEqual(new Uint8Array(received!), new Uint8Array([80, 75]))
  fixture.worker.onmessage!({ data: { phase: 'expanding' } } as MessageEvent)
  fixture.worker.onmessage!({ data: { phase: 'transferring' } } as MessageEvent)
  fixture.worker.onmessage!({ data: { files: [] } } as MessageEvent)
  await result
  assert.equal(fixture.terminated, 1)
  assert.equal(fixture.worker.onmessage, null)
})

test('EPUB Worker：未确认解压完成的结果不能伪报成功', async () => {
  for (const phases of [[], ['ready'], ['ready', 'expanding']]) {
    const fixture = fixtureWorker()
    const result = expandEpubInWorker(new Uint8Array([80, 75]), undefined, () => fixture.worker)
    const observed = assert.rejects(result, /无效数据/)
    for (const phase of phases) fixture.worker.onmessage!({ data: { phase } } as MessageEvent)
    fixture.worker.onmessage!({ data: { files: [] } } as MessageEvent)
    await observed
    assert.equal(fixture.sentCount, phases.length ? 1 : 0)
    assert.equal(fixture.terminated, 1)
  }
})

test('EPUB Worker：重复 ready 拒绝且不二次转移已分离的输入', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([80, 75])
  const send = fixture.worker.postMessage
  fixture.worker.postMessage = (value, transfer) => { send(value, transfer); structuredClone(value, { transfer }) }
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  const observed = assert.rejects(result, /无效阶段/)
  const reply = fixture.worker.onmessage!
  reply({ data: { phase: 'ready' } } as MessageEvent)
  assert.equal(input.byteLength, 0)
  reply({ data: { phase: 'ready' } } as MessageEvent)
  await observed
  assert.equal(fixture.sentCount, 1)
  assert.equal(fixture.terminated, 1)
})

test('EPUB Worker：结果传递阶段取消后迟到结果不能重建旧文档', async () => {
  const fixture = fixtureWorker()
  const controller = new AbortController()
  const result = expandEpubInWorker(new Uint8Array([80, 75]), controller.signal, () => fixture.worker)
  for (const phase of ['ready', 'expanding', 'transferring']) {
    fixture.worker.onmessage!({ data: { phase } } as MessageEvent)
  }
  const lateReply = fixture.worker.onmessage!
  const observed = assert.rejects(result, { name: 'AbortError' })
  controller.abort()
  lateReply({ data: { files: [['generated', new Uint8Array([1])]] } } as MessageEvent)
  await observed
  assert.equal(fixture.sentCount, 1)
  assert.equal(fixture.terminated, 1)
  assert.equal(fixture.worker.onmessage, null)
})
