import { test } from 'node:test'
import assert from 'node:assert/strict'
import { zipSync, strToU8, Zip, ZipDeflate } from 'fflate'
import { expandEpubArchive, readEpubResponse, EPUB_ARCHIVE_LIMITS } from './epubArchive.ts'

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
