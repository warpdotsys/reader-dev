import { test } from 'node:test'
import assert from 'node:assert/strict'
import { zipSync, strToU8, Zip, ZipDeflate, Unzip } from 'fflate'
import { expandEpubArchive, readEpubResponse, EPUB_ARCHIVE_LIMITS, EpubDownloadBuffer } from './epubArchive.ts'

const small = { ...EPUB_ARCHIVE_LIMITS, entryBytes: 1000, textBytes: 1000, expandedBytes: 2000 }

test('EPUB ZIP：正常 deflate/stored 和目录条目逐字节恢复', () => {
  for (const level of [0, 6] as const) {
    const source = strToU8('生成中文正文 '.repeat(40))
    const bytes = zipSync({ 'OEBPS/': new Uint8Array(), 'OEBPS/ch.xhtml': source }, { level })
    const files = expandEpubArchive(bytes)
    assert.deepEqual(files.get('OEBPS/ch.xhtml'), source)
    assert.equal(files.size, 1)
  }
})

function generatedBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length)
  let state = 0x13579bdf
  for (let index = 0; index < length; index++) {
    state ^= state << 13
    state ^= state >>> 17
    state ^= state << 5
    bytes[index] = state & 255
  }
  return bytes
}

test('EPUB ZIP：多块与多资源填入独占缓冲区，逐字节且没有零填充尾部', () => {
  const binary = generatedBytes(256 * 1024 + 7)
  const text = strToU8('生成正文，跨分块且不能损坏中文。'.repeat(3000))
  for (const level of [0, 6] as const) {
    const bytes = zipSync({ 'assets/a.bin': binary, 'OEBPS/ch.xhtml': text }, { level })
    assert.ok(bytes.length > 4096 * 2)
    const files = expandEpubArchive(bytes)
    for (const [name, expected] of [['assets/a.bin', binary], ['OEBPS/ch.xhtml', text]] as const) {
      const result = files.get(name)!
      assert.deepEqual(result, expected)
      assert.equal(result.byteOffset, 0)
      assert.equal(result.buffer.byteLength, expected.length)
      assert.notEqual(result.buffer, expected.buffer)
      assert.notEqual(result.buffer, bytes.buffer)
    }
    assert.notEqual(files.get('assets/a.bin')!.buffer, files.get('OEBPS/ch.xhtml')!.buffer)
  }
})

test('EPUB ZIP：未知本地大小的数据描述符仍由中央预算和实际输出核验', () => {
  const expected = generatedBytes(192 * 1024 + 11)
  const chunks: Uint8Array[] = []
  let completed = false
  const zip = new Zip((error, chunk, final) => {
    assert.equal(error, null)
    chunks.push(chunk)
    completed ||= final
  })
  const entry = new ZipDeflate('assets/streamed.bin', { level: 6 })
  zip.add(entry)
  entry.push(expected.subarray(0, 70000), false)
  entry.push(expected.subarray(70000), true)
  zip.end()
  assert.equal(completed, true)
  const bytes = new Uint8Array(chunks.reduce((size, chunk) => size + chunk.length, 0))
  let offset = 0
  for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length }
  assert.notEqual(new DataView(bytes.buffer).getUint16(6, true) & 8, 0)
  const actual = expandEpubArchive(bytes).get('assets/streamed.bin')!
  assert.deepEqual(actual, expected)
  assert.equal(actual.buffer.byteLength, expected.length)
})

test('EPUB ZIP：高报大小必须拒绝，空资源保留而目录不物化', () => {
  const expected = strToU8('只生成的小样本')
  const bytes = zipSync({ 'a.xhtml': expected })
  const view = new DataView(bytes.buffer)
  for (let index = 0; index + 46 <= bytes.length; index++) {
    const signature = view.getUint32(index, true)
    if (signature === 0x04034b50) view.setUint32(index + 22, expected.length + 9, true)
    if (signature === 0x02014b50) view.setUint32(index + 24, expected.length + 9, true)
  }
  assert.throws(() => expandEpubArchive(bytes), /实际资源大小与声明不一致/)
  const files = expandEpubArchive(zipSync({ 'OEBPS/': new Uint8Array(), 'OEBPS/empty.xhtml': new Uint8Array() }))
  assert.deepEqual([...files.keys()], ['OEBPS/empty.xhtml'])
  assert.equal(files.get('OEBPS/empty.xhtml')!.buffer.byteLength, 0)
})

test('EPUB ZIP：压缩输入、单资源、文本、累计量和条目数分别受限', () => {
  const bytes = zipSync({ 'a.bin': new Uint8Array(1100), 'b.bin': new Uint8Array(1100) })
  assert.throws(() => expandEpubArchive(bytes, { ...small, compressedBytes: 1 }), /下载上限/)
  assert.throws(() => expandEpubArchive(bytes, small), /单个资源/)
  assert.throws(() => expandEpubArchive(bytes, { ...small, entryBytes: 1200 }), /解压总量/)
  assert.throws(() => expandEpubArchive(bytes, { ...small, entries: 1, entryBytes: 1200, expandedBytes: 2400 }), /条目数量/)
  assert.throws(() => expandEpubArchive(zipSync({ 'a.xhtml': new Uint8Array(1100) }),
    { ...small, entryBytes: 2000 }), /单个资源/)
})

test('EPUB ZIP：不信任低报的中央/本地大小，不能静默截断正文', () => {
  const bytes = zipSync({ 'a.xhtml': strToU8('x'.repeat(4096)) })
  const view = new DataView(bytes.buffer)
  for (let index = 0; index + 46 <= bytes.length; index++) {
    const sig = view.getUint32(index, true)
    if (sig === 0x04034b50) view.setUint32(index + 22, 16, true)
    if (sig === 0x02014b50) view.setUint32(index + 24, 16, true)
  }
  assert.throws(() => expandEpubArchive(bytes), /实际解压量超过声明/)
})

test('EPUB ZIP：跨多个输入块的32MiB高压缩率条目低报为16字节，在首个有界输出拒绝', () => {
  const bytes = zipSync({ 'generated.bin': new Uint8Array(32 * 1024 * 1024).fill(0x55) }, { level: 6 })
  assert.ok(bytes.length > 8192 * 2 && bytes.length < 65536)
  const view = new DataView(bytes.buffer)
  let locals = 0, centrals = 0
  for (let index = 0; index + 46 <= bytes.length; index++) {
    const signature = view.getUint32(index, true)
    if (signature === 0x04034b50) { view.setUint32(index + 22, 16, true); locals++ }
    if (signature === 0x02014b50) { view.setUint32(index + 24, 16, true); centrals++ }
  }
  assert.equal(locals, 1)
  assert.equal(centrals, 1)
  const register = Unzip.prototype.register
  const chunks: number[] = []
  Unzip.prototype.register = function (Decoder) {
    class Observed extends Decoder {
      constructor(...args: ConstructorParameters<typeof Decoder>) {
        super(...args)
        let downstream: InstanceType<typeof Decoder>['ondata']
        const ondata: typeof downstream = (error, data, final) => {
          if (data?.length) chunks.push(data.length)
          downstream(error, data, final)
        }
        Object.defineProperty(this, 'ondata', { get: () => ondata, set: value => { downstream = value } })
      }
    }
    return register.call(this, Observed)
  }
  try {
    assert.throws(() => expandEpubArchive(bytes), /实际解压量超过声明/)
    assert.equal(chunks.length, 1)
    // 中央目录低报16字节，固定缓冲仅允许多解出1字节就立即拒绝。
    assert.equal(chunks[0], 17)
  } finally { Unzip.prototype.register = register }
})

test('EPUB ZIP：本地与中央不一致、重复解码路径和越界路径拒绝', () => {
  const bytes = zipSync({ 'a.xhtml': strToU8('generated') })
  new DataView(bytes.buffer).setUint32(22, 1, true)
  assert.throws(() => expandEpubArchive(bytes), /本地条目与中央目录/)
  assert.throws(() => expandEpubArchive(zipSync({ 'a.xhtml': strToU8('a'), '%61.xhtml': strToU8('b') })), /重复或冲突/)
  for (const name of ['../x', '/x', 'a/../../x', 'https:book', 'x\0y']) {
    assert.throws(() => expandEpubArchive(zipSync({ [name]: strToU8('generated') })), /条目/)
  }
})

test('EPUB ZIP：破损、缺少本地条目不会返回伪造空资源', () => {
  assert.throws(() => expandEpubArchive(new Uint8Array([80, 75, 1, 2])))
  const bytes = zipSync({ 'a.xhtml': strToU8('generated') })
  bytes[0] = 0
  assert.throws(() => expandEpubArchive(bytes), /缺少完整本地条目/)
})

test('EPUB 下载：声明超限时不读正文，取消流', async () => {
  let cancelled = false
  const response = new Response(new ReadableStream({ cancel() { cancelled = true } }),
    { headers: { 'content-length': '999' } })
  await assert.rejects(readEpubResponse(response, 10), /下载上限/)
  assert.equal(cancelled, true)
  assert.equal(response.body?.locked, false)
})

test('EPUB 下载：缺失/低报 Content-Length 按实际字节拒绝并取消', async () => {
  for (const declared of ['', '1']) {
    let cancelled = false
    let count = 0
    const stream = new ReadableStream<Uint8Array>({
      pull(controller) { count++; controller.enqueue(new Uint8Array(6)) },
      cancel() { cancelled = true },
    })
    await assert.rejects(readEpubResponse(new Response(stream, {
      headers: declared ? { 'content-length': declared } : {},
    }), 10), /下载上限/)
    assert.equal(cancelled, true)
    assert.ok(count <= 3)
    assert.equal(stream.locked, false)
  }
})

test('EPUB 下载：流失败保留错误并释放锁，正常字节不改动', async () => {
  const broken = new ReadableStream<Uint8Array>({ start(controller) { controller.error(new Error('generated failure')) } })
  await assert.rejects(readEpubResponse(new Response(broken)), /generated failure/)
  assert.equal(broken.locked, false)
  assert.deepEqual(await readEpubResponse(new Response(new Uint8Array([80, 75, 9]))), new Uint8Array([80, 75, 9]))
})

test('EPUB 下载：精确有界提示复用一份目的缓冲，完成后不再保留块引用', () => {
  const collector = new EpubDownloadBuffer(20, 8)
  collector.push(new Uint8Array([1, 2, 3, 4]))
  assert.equal(collector.retainedBytes, 4)
  collector.push(new Uint8Array([5, 6, 7, 8]))
  assert.equal(collector.retainedBytes, 8)
  assert.deepEqual(collector.finish(), new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))
  assert.equal(collector.retainedBytes, 0)
})

test('EPUB 下载：大提示必须先收到真实数据，未知/低报/高报均保留完整实际字节', () => {
  const tiny = new EpubDownloadBuffer(64 * 1024 * 1024, 64 * 1024 * 1024)
  tiny.push(new Uint8Array([1, 2]))
  assert.equal(tiny.retainedBytes, 2)
  assert.deepEqual(tiny.finish(), new Uint8Array([1, 2]))
  for (const hint of [null, 0, 1, 4, 10, 21, NaN]) {
    const collector = new EpubDownloadBuffer(20, hint)
    collector.push(new Uint8Array([1, 2, 3, 4]))
    collector.push(new Uint8Array([5, 6]))
    assert.deepEqual(collector.finish(), new Uint8Array([1, 2, 3, 4, 5, 6]))
    assert.equal(collector.retainedBytes, 0)
  }
})

test('EPUB 下载：高报64MiB不能把仅1MiB正文提前放大成64MiB缓冲', () => {
  const collector = new EpubDownloadBuffer(64 * 1024 * 1024, 64 * 1024 * 1024)
  collector.push(new Uint8Array(1024 * 1024).fill(3))
  assert.equal(collector.retainedBytes, 1024 * 1024)
  const bytes = collector.finish()
  assert.equal(bytes.byteLength, 1024 * 1024)
  assert.equal(collector.retainedBytes, 0)
})

test('EPUB 下载：提示缓冲溢出后回到实际计数，仍拒绝越过硬限', () => {
  const collector = new EpubDownloadBuffer(8, 4)
  collector.push(new Uint8Array([1, 2, 3, 4]))
  collector.push(new Uint8Array([5, 6]))
  assert.equal(collector.retainedBytes, 6)
  assert.throws(() => collector.push(new Uint8Array(3)), /下载上限/)
  collector.clear()
  assert.equal(collector.retainedBytes, 0)
})

test('EPUB 下载：真实 MiB 阈值后不同时保留整份块与目的缓冲', () => {
  const data = new Uint8Array(1024 * 1024).fill(7)
  const collector = new EpubDownloadBuffer(data.length + 4, data.length + 4)
  collector.push(data)
  assert.equal(collector.retainedBytes, data.length + 4)
  collector.push(new Uint8Array([1, 2, 3, 4]))
  const result = collector.finish()
  assert.equal(result.byteLength, data.length + 4)
  assert.deepEqual(result.subarray(data.length), new Uint8Array([1, 2, 3, 4]))
  assert.ok(result.subarray(0, data.length).every(value => value === 7))
  assert.equal(collector.retainedBytes, 0)
})

test('EPUB 下载：已启用提示缓冲的过报也只返回真实字节，清理不擦掉返回值', () => {
  const size = 1024 * 1024
  const collector = new EpubDownloadBuffer(size * 2, size * 2)
  collector.push(new Uint8Array(size).fill(9))
  assert.equal(collector.retainedBytes, size * 2)
  const result = collector.finish()
  assert.equal(result.byteLength, size)
  assert.equal(result.buffer.byteLength, size)
  assert.ok(result.every(value => value === 9))
  collector.clear()
  assert.equal(collector.retainedBytes, 0)
  assert.equal(result[0], 9)
})

test('EPUB 下载：提示缓冲后流失败/取消失败仍保留首错并释放读取锁', async () => {
  let pulls = 0
  const primary = new Error('generated streamed primary')
  const stream = new ReadableStream<Uint8Array>({
    pull(controller) {
      if (++pulls === 1) controller.enqueue(new Uint8Array(1024 * 1024))
      else controller.error(primary)
    },
    cancel() { throw new Error('generated secondary cancellation') },
  })
  await assert.rejects(readEpubResponse(new Response(stream, {
    headers: { 'content-length': String(2 * 1024 * 1024) },
  })), error => error === primary)
  assert.equal(stream.locked, false)
})

test('EPUB 下载：连续块、过报和低报 Response 返回相同独占字节，不修改来源', async () => {
  for (const hint of ['', '2', '6', '10']) {
    const chunks = [new Uint8Array([80, 75, 1]), new Uint8Array([2, 3, 4])]
    const stream = new ReadableStream<Uint8Array>({ start(controller) {
      for (const chunk of chunks) controller.enqueue(chunk)
      controller.close()
    } })
    const result = await readEpubResponse(new Response(stream, { headers: hint ? { 'content-length': hint } : {} }), 20)
    assert.deepEqual(result, new Uint8Array([80, 75, 1, 2, 3, 4]))
    assert.equal(result.byteOffset, 0)
    assert.equal(result.byteLength, result.buffer.byteLength)
    assert.notEqual(result.buffer, chunks[0].buffer)
    result[0] = 9
    assert.equal(chunks[0][0], 80)
    assert.equal(stream.locked, false)
  }
})
