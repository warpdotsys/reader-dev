import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'
import ts from 'typescript'
import { zipSync } from 'fflate'
import { expandEpubArchive } from './epubArchive.ts'
import { expandEpubInWorker, type EpubArchiveWorker } from './epubWorker.ts'
import { EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES } from './epubTransfer.ts'

const source = readFileSync(new URL('./epubArchive.worker.ts', import.meta.url), 'utf8')
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText

function producer(decode: (input: Uint8Array) => Map<string, Uint8Array>, actualTransfer = false) {
  const messages: { value: any; transfer: Transferable[] }[] = []
  let closed = 0
  const scope = {
    onmessage: null as ((event: MessageEvent<unknown>) => void) | null,
    postMessage(value: unknown, transfer: Transferable[]) {
      messages.push({ value: actualTransfer ? structuredClone(value, { transfer }) : value, transfer })
    },
    close() { closed++ },
    Uint8Array, ArrayBuffer, Error, exports: {},
    require(name: string) {
      if (name === './epubArchive') return { expandEpubArchive: decode }
      assert.equal(name, './epubTransfer')
      return { EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES }
    },
  }
  runInNewContext(compiled, scope)
  return { scope, messages, get closed() { return closed } }
}

function send(fixture: ReturnType<typeof producer>, data: unknown) {
  fixture.scope.onmessage!({ data } as MessageEvent<unknown>)
}

test('实际 Worker 等匹配确认后才传下一批和关闭，固定阶段不含资源', () => {
  const files = new Map([['generated/a.xhtml', new Uint8Array([1, 2])], ['generated/b.css', new Uint8Array([3])]])
  const fixture = producer(() => files)
  send(fixture, new ArrayBuffer(2))
  assert.deepEqual(fixture.messages.slice(0, 3).map(message => message.value.phase), ['ready', 'expanding', 'transferring'])
  for (const message of fixture.messages.slice(0, 3)) {
    assert.deepEqual(Object.keys(message.value), ['phase'])
    assert.equal(message.transfer.length, 0)
  }
  assert.equal(fixture.closed, 0)
  assert.equal(fixture.messages.length, 4)
  const result = fixture.messages[3]
  assert.equal(result.value.batch, 0)
  assert.deepEqual([...result.value.files].map(([name, bytes]: [string, Uint8Array]) => [name, [...bytes]]),
    [['generated/a.xhtml', [1, 2]], ['generated/b.css', [3]]])
  assert.equal(result.transfer.length, 2)
  assert.equal(files.size, 0)
  send(fixture, { batchAccepted: 0 })
  assert.equal(JSON.stringify(fixture.messages[4].value), JSON.stringify({ complete: true, entries: 2, bytes: 3 }))
  assert.equal(fixture.closed, 1)
  assert.equal(fixture.scope.onmessage, null)
})

test('实际 Worker 每次至多64条且只有一批在途，所有资源恰好转移一次', () => {
  const files = new Map(Array.from({ length: 130 }, (_, n) => [`generated/${n}.bin`, new Uint8Array([n % 256])] as const))
  const fixture = producer(() => files, true)
  send(fixture, new ArrayBuffer(2))
  const sizes: number[] = []
  const seen = new Map<string, Uint8Array>()
  for (let n = 0; n < 3; n++) {
    const batch = fixture.messages.at(-1)!.value
    assert.equal(batch.batch, n)
    sizes.push(batch.files.length)
    for (const [name, bytes] of batch.files) {
      assert.equal(seen.has(name), false)
      seen.set(name, bytes)
    }
    const before = fixture.messages.length
    assert.equal(fixture.closed, 0)
    send(fixture, { batchAccepted: n })
    assert.equal(fixture.messages.length, before + 1)
  }
  assert.deepEqual(sizes, [64, 64, 2])
  assert.equal(seen.size, 130)
  for (let n = 0; n < 130; n++) assert.deepEqual(seen.get(`generated/${n}.bin`), new Uint8Array([n % 256]))
  assert.equal(JSON.stringify(fixture.messages.at(-1)!.value), JSON.stringify({ complete: true, entries: 130, bytes: 130 }))
  assert.equal(fixture.closed, 1)
})

test('实际 Worker 按2MiB分批，大于批预算但合法的单项不截断，真实转移分离发送方字节', () => {
  const original = [new Uint8Array(1024 * 1024 + 1).fill(1), new Uint8Array(1024 * 1024).fill(2),
    new Uint8Array(3 * 1024 * 1024).fill(3), new Uint8Array([4])]
  const expected = original.map(bytes => bytes.slice())
  const fixture = producer(() => new Map(original.map((bytes, n) => [`generated/${n}.bin`, bytes])), true)
  send(fixture, new ArrayBuffer(2))
  for (let n = 0; n < 4; n++) {
    const batch = fixture.messages.at(-1)!.value
    assert.equal(batch.batch, n)
    assert.equal(batch.files.length, 1)
    assert.deepEqual(batch.files[0][1], expected[n])
    assert.equal(original[n]!.byteLength, 0)
    send(fixture, { batchAccepted: n })
  }
  assert.equal(fixture.messages.at(-1)!.value.bytes, expected.reduce((n, bytes) => n + bytes.byteLength, 0))
  assert.equal(fixture.closed, 1)
})

test('实际 ZIP 解码后分批转移保留未在manifest列出的资源，不用伪造解码数据证明兼容', () => {
  const entries: Record<string, Uint8Array> = {}
  for (let n = 0; n < 70; n++) entries[`generated/${n}.bin`] = new Uint8Array([n])
  const zip = zipSync(entries)
  const fixture = producer(() => expandEpubArchive(zip), true)
  send(fixture, new ArrayBuffer(2))
  const result = new Map<string, Uint8Array>()
  while (!fixture.closed) {
    const batch = fixture.messages.at(-1)!.value
    for (const [name, bytes] of batch.files) result.set(name, bytes)
    send(fixture, { batchAccepted: batch.batch })
  }
  assert.equal(result.size, 70)
  for (const [name, bytes] of Object.entries(entries)) assert.deepEqual(result.get(name), bytes)
})

test('实际 Worker 拒绝未知、逆序、混合和重复确认，错误后清余项且关闭', () => {
  for (const ack of [{ batchAccepted: 1 }, { batchAccepted: -1 }, { batchAccepted: '0' },
    { batchAccepted: 0, extra: true }, {}, new ArrayBuffer(2), null]) {
    const files = new Map(Array.from({ length: 65 }, (_, n) => [`${n}.bin`, new Uint8Array([n])] as const))
    const fixture = producer(() => files)
    send(fixture, new ArrayBuffer(2))
    send(fixture, ack)
    assert.equal(fixture.messages.at(-1)!.value.error, 'EPUB Worker 结果确认协议无效')
    assert.equal(files.size, 0)
    assert.equal(fixture.closed, 1)
  }
  const fixture = producer(() => new Map(Array.from({ length: 129 }, (_, n) => [`${n}.bin`, new Uint8Array([n])] as const)))
  send(fixture, new ArrayBuffer(2))
  send(fixture, { batchAccepted: 0 })
  send(fixture, { batchAccepted: 0 })
  assert.equal(fixture.messages.at(-1)!.value.error, 'EPUB Worker 结果确认协议无效')
  assert.equal(fixture.closed, 1)
})

test('实际 Worker 解码失败不宣告完成或传递，仍关闭且过滤非 EPUB 私有错误', () => {
  for (const error of [new Error('EPUB generated limit'), new Error('PRIVATE_PATH_OR_CONTENT')]) {
    const fixture = producer(() => { throw error })
    send(fixture, new ArrayBuffer(2))
    assert.deepEqual(fixture.messages.slice(0, 2).map(message => message.value.phase), ['ready', 'expanding'])
    assert.equal(fixture.messages.length, 3)
    assert.equal(fixture.messages[2].value.error,
      error.message.startsWith('EPUB ') ? error.message : 'EPUB ZIP 无效或无法解压')
    assert.equal(JSON.stringify(fixture.messages).includes('PRIVATE_'), false)
    assert.equal(fixture.closed, 1)
  }
})

test('实际 Worker 无效首消息不能调用解码，真实空ZIP显式完成但不伪造文件', () => {
  const invalid = producer(() => { throw new Error('Must not decode') })
  send(invalid, { batchAccepted: 0 })
  assert.equal(invalid.messages.at(-1)!.value.error, 'EPUB Worker 输入协议无效')
  assert.equal(invalid.closed, 1)
  const empty = producer(() => expandEpubArchive(zipSync({})))
  send(empty, new ArrayBuffer(2))
  assert.equal(JSON.stringify(empty.messages.at(-1)!.value), JSON.stringify({ complete: true, entries: 0, bytes: 0 }))
  assert.equal(empty.closed, 1)
})

test('实际生产者与消费者连通：真实ZIP/ArrayBuffer转移/多批确认保持全部字节', async () => {
  const entries: Record<string, Uint8Array> = {}
  for (let n = 0; n < 130; n++) entries[`generated/${n}.bin`] = new Uint8Array([n, 1, 2])
  entries['generated/large.bin'] = new Uint8Array(3 * 1024 * 1024).fill(17)
  const input = zipSync(entries)
  const transport = producer(bytes => expandEpubArchive(bytes), true)
  let cursor = 0
  let terminated = 0
  const worker: EpubArchiveWorker = {
    onmessage: null, onerror: null,
    postMessage(value, transfer) {
      const data = structuredClone(value, { transfer })
      setImmediate(() => { transport.scope.onmessage?.({ data } as MessageEvent); pump() })
    },
    terminate() { terminated++ },
  }
  function pump() {
    while (cursor < transport.messages.length) worker.onmessage?.({ data: transport.messages[cursor++]!.value } as MessageEvent)
  }
  const promise = expandEpubInWorker(input, undefined, () => worker)
  pump()
  const result = await promise
  assert.equal(input.byteLength, 0)
  assert.equal(result.size, Object.keys(entries).length)
  for (const [name, bytes] of Object.entries(entries)) assert.deepEqual(result.get(name), bytes)
  assert.equal(transport.closed, 1)
  assert.equal(terminated, 1)
  assert.equal(worker.onmessage, null)
})
