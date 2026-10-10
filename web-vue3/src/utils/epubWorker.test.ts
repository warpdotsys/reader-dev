import { test } from 'node:test'
import assert from 'node:assert/strict'
import { expandEpubInWorker, type EpubArchiveWorker } from './epubWorker.ts'

function fixtureWorker() {
  let terminated = 0
  let sent: ArrayBuffer | null = null
  let transferred: Transferable[] = []
  const worker: EpubArchiveWorker = {
    onmessage: null, onerror: null,
    postMessage(value, transfer) { sent = value; transferred = transfer },
    terminate() { terminated++ },
  }
  return { worker, get terminated() { return terminated }, get sent() { return sent }, get transferred() { return transferred } }
}

test('EPUB Worker：输入/输出保持完整，成功后终止并清除回调', async () => {
  const fixture = fixtureWorker()
  const input = new Uint8Array([80, 75])
  const result = expandEpubInWorker(input, undefined, () => fixture.worker)
  assert.equal(fixture.sent, input.buffer)
  assert.deepEqual(fixture.transferred, [input.buffer])
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
  assert.deepEqual(new Uint8Array(fixture.sent!), new Uint8Array([80, 75]))
  assert.notEqual(fixture.sent, input.buffer)
  fixture.worker.onmessage!({ data: { error: 'EPUB generated' } } as MessageEvent)
  await assert.rejects(result)
  const failed = fixtureWorker()
  failed.worker.postMessage = () => { throw new Error('Generated send failure') }
  await assert.rejects(expandEpubInWorker(new Uint8Array(), undefined, () => failed.worker), /send failure/)
  assert.equal(failed.terminated, 1)
})
