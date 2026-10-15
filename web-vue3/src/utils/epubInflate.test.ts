import { test } from 'node:test'
import assert from 'node:assert/strict'
import { deflateSync } from 'fflate'
import { BoundedEpubInflate, EPUB_INFLATE_CHUNK_BYTES } from './epubInflate.ts'

function collect(compressed: number, expanded: number) {
  const decoder = new BoundedEpubInflate(compressed, expanded)
  const chunks: Uint8Array[] = []
  let finals = 0
  decoder.ondata = (error, data, final) => {
    assert.equal(error, null)
    if (data?.length) chunks.push(data.slice())
    if (final) finals++
  }
  return { decoder, chunks, get finals() { return finals } }
}

test('EPUB DEFLATE：解码预算必须是非负安全整数', () => {
  for (const value of [-1, NaN, Infinity, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(() => new BoundedEpubInflate(value, 10), /预算无效/)
    assert.throws(() => new BoundedEpubInflate(10, value), /预算无效/)
  }
})

test('EPUB DEFLATE：64KiB 输出上限与跨输入块的完整字节', () => {
  const expected = new Uint8Array(1024 * 1024 + 13)
  for (let index = 0; index < expected.length; index++) expected[index] = (index * 17) & 255
  const compressed = deflateSync(expected)
  const result = collect(compressed.length, expected.length)
  for (let offset = 0; offset < compressed.length; offset += 29) {
    const end = Math.min(offset + 29, compressed.length)
    result.decoder.push(compressed.subarray(offset, end), end === compressed.length)
  }
  assert.equal(result.finals, 1)
  assert.ok(result.chunks.length > 10)
  assert.ok(result.chunks.every(chunk => chunk.length <= EPUB_INFLATE_CHUNK_BYTES))
  assert.deepEqual(Buffer.concat(result.chunks), Buffer.from(expected))
  assert.throws(() => result.decoder.push(new Uint8Array(), true), /重复结束/)
})

test('EPUB DEFLATE：结束早于 ZIP 描述符的空 final 只完成一次', () => {
  const expected = new TextEncoder().encode('生成的中文正文')
  const compressed = deflateSync(expected)
  const result = collect(compressed.length, expected.length)
  result.decoder.push(compressed, false)
  assert.equal(result.finals, 0)
  result.decoder.push(new Uint8Array(), true)
  assert.equal(result.finals, 1)
  assert.deepEqual(Buffer.concat(result.chunks), Buffer.from(expected))
})

test('EPUB DEFLATE：末块耗尽输入时仍排空64KiB边界后的有效输出，不伪报截断', () => {
  for (const length of [65536, 65537, 65538, 1048577, 1048578]) {
    const expected = new Uint8Array(length).fill(1)
    const compressed = deflateSync(expected)
    const result = collect(compressed.length, length)
    result.decoder.push(compressed, true)
    assert.equal(result.finals, 1)
    assert.ok(result.chunks.every(chunk => chunk.length <= EPUB_INFLATE_CHUNK_BYTES))
    assert.deepEqual(Buffer.concat(result.chunks), Buffer.from(expected))
    const truncated = compressed.subarray(0, compressed.length - 1)
    assert.throws(() => collect(truncated.length, length).decoder.push(truncated, true), /截断|无效/)
    const trailing = new Uint8Array(compressed.length + 1)
    trailing.set(compressed)
    assert.throws(() => collect(trailing.length, length).decoder.push(trailing, true), /多余数据/)
  }
})

test('EPUB DEFLATE：截断、破损以及相同/后续块的尾随垃圾均拒绝', () => {
  const compressed = deflateSync(new TextEncoder().encode('generated '.repeat(100)))
  const truncated = compressed.subarray(0, compressed.length - 1)
  assert.throws(() => collect(truncated.length, 1000).decoder.push(truncated, true), /截断|无效/)
  const bad = new Uint8Array([0xff, 0xff, 0xff])
  assert.throws(() => collect(bad.length, 1000).decoder.push(bad, true), /无效/)
  const trailing = new Uint8Array(compressed.length + 1)
  trailing.set(compressed)
  assert.throws(() => collect(trailing.length, 1000).decoder.push(trailing, true), /多余数据/)
  const split = collect(trailing.length, 1000)
  split.decoder.push(compressed, false)
  assert.throws(() => split.decoder.push(new Uint8Array([1]), true), /多余数据/)
})

test('EPUB DEFLATE：压缩量高报或低报不允许静默成功', () => {
  const compressed = deflateSync(new Uint8Array(100))
  assert.throws(() => collect(compressed.length - 1, 100).decoder.push(compressed, true), /实际压缩量/)
  assert.throws(() => collect(compressed.length + 1, 100).decoder.push(compressed, true), /压缩量与声明/)
})

test('EPUB DEFLATE：低报16字节的32MiB样本在17字节回调立即中止', () => {
  const compressed = deflateSync(new Uint8Array(32 * 1024 * 1024).fill(85))
  const decoder = new BoundedEpubInflate(compressed.length, 16)
  let calls = 0
  const primary = new Error('generated output over declared budget')
  decoder.ondata = (_error, data) => {
    calls++
    assert.equal(data?.length, 17)
    throw primary
  }
  assert.throws(() => decoder.push(compressed, true), error => error === primary)
  assert.equal(calls, 1)
})
