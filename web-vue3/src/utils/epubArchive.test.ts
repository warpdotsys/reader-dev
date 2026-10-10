import { test } from 'node:test'
import assert from 'node:assert/strict'
import { zipSync, strToU8 } from 'fflate'
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
