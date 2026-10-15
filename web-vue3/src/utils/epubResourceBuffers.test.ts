import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import ts from 'typescript'
import { zipSync } from 'fflate'
import { expandEpubArchive, expandEpubForTransfer, EPUB_ARCHIVE_LIMITS } from './epubArchive.ts'
import { EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES } from './epubTransfer.ts'
import { packedEpubResourceAllocator } from './epubResourceBuffers.ts'

function assertCovered(files: Map<string, Uint8Array>) {
  const buffers = new Map<ArrayBufferLike, [number, number][]>()
  for (const data of files.values()) {
    const list = buffers.get(data.buffer) ?? []
    list.push([data.byteOffset, data.byteLength]); buffers.set(data.buffer, list)
  }
  for (const [buffer, ranges] of buffers) {
    ranges.sort(([a, sizeA], [b, sizeB]) => a - b || sizeA - sizeB)
    let offset = 0
    for (const [start, length] of ranges) { assert.equal(start, offset); offset += length }
    assert.equal(offset, buffer.byteLength)
    assert.ok(ranges.length <= EPUB_TRANSFER_BATCH_ENTRIES)
    assert.ok(buffer.byteLength <= EPUB_TRANSFER_BATCH_BYTES || ranges.length === 1)
  }
  return buffers
}

test('共享输出保持全部原字节和无填充覆盖，70个资源只需两个转移缓冲', () => {
  const entries = Object.fromEntries(Array.from({ length: 70 }, (_, n) => [`a/${n}.bin`, new Uint8Array([n, 1, 2])]))
  for (const level of [0, 6] as const) {
    const zip = zipSync(entries, { level })
    const files = expandEpubForTransfer(zip)
    assert.equal(files.size, 70)
    for (const [name, expected] of Object.entries(entries)) assert.deepEqual(files.get(name), expected)
    const buffers = assertCovered(files)
    assert.equal(buffers.size, 2)
    assert.equal([...buffers.keys()].reduce((n, b) => n + b.byteLength, 0), 210)
    assert.equal(files.get('a/0.bin')!.buffer, files.get('a/63.bin')!.buffer)
    assert.notEqual(files.get('a/63.bin')!.buffer, files.get('a/64.bin')!.buffer)
    const ordinary = expandEpubArchive(zip)
    assert.notEqual(ordinary.get('a/0.bin')!.buffer, ordinary.get('a/1.bin')!.buffer)
    assert.equal(ordinary.get('a/1.bin')!.byteOffset, 0)
  }
})

test('共享输出按实际分组字节受限，大资源完整独立，不按整书声明一次预分配', () => {
  const entries = { 'a.bin': new Uint8Array(1024 * 1024 + 1).fill(1),
    'b.bin': new Uint8Array(1024 * 1024).fill(2), 'c.bin': new Uint8Array(3 * 1024 * 1024).fill(3),
    'empty.bin': new Uint8Array(), 'd.bin': new Uint8Array([4]) }
  const files = expandEpubForTransfer(zipSync(entries))
  for (const [name, bytes] of Object.entries(entries)) assert.deepEqual(files.get(name), bytes)
  const buffers = assertCovered(files)
  assert.equal([...buffers.keys()].reduce((n, b) => n + b.byteLength, 0),
    Object.values(entries).reduce((n, b) => n + b.byteLength, 0))
  assert.equal(files.get('c.bin')!.buffer.byteLength, 3 * 1024 * 1024)
  assert.notEqual(files.get('c.bin')!.buffer, files.get('d.bin')!.buffer)
})

test('中央目录顺序不同于本地顺序仍逐字节恢复，并完整覆盖每个共享缓冲', () => {
  const source = Object.fromEntries(Array.from({ length: 70 }, (_, n) => [`${n}.bin`, new Uint8Array([n])]))
  const zip = zipSync(source)
  const view = new DataView(zip.buffer)
  const eocd = zip.byteLength - 22
  assert.equal(view.getUint32(eocd, true), 0x06054b50)
  const start = view.getUint32(eocd + 16, true)
  const records: Uint8Array[] = []
  let offset = start
  while (offset < eocd) {
    assert.equal(view.getUint32(offset, true), 0x02014b50)
    const size = 46 + view.getUint16(offset + 28, true) + view.getUint16(offset + 30, true) + view.getUint16(offset + 32, true)
    records.push(zip.slice(offset, offset + size)); offset += size
  }
  offset = start
  for (const record of records.reverse()) { zip.set(record, offset); offset += record.length }
  const files = expandEpubForTransfer(zip)
  assert.equal(files.size, 70)
  for (const [name, bytes] of Object.entries(source)) assert.deepEqual(files.get(name), bytes)
  assert.equal(assertCovered(files).size, 2)
})

test('共享输出仍完整检查无关资源、声明与实际大小、重复路径及全部既有限额', () => {
  const zip = zipSync({ 'a.bin': new Uint8Array(20), 'unused.bin': new Uint8Array(20) })
  assert.throws(() => expandEpubForTransfer(zip, { ...EPUB_ARCHIVE_LIMITS, expandedBytes: 30 }), /解压总量/)
  assert.throws(() => expandEpubForTransfer(zip, { ...EPUB_ARCHIVE_LIMITS, compressedBytes: 1 }), /下载上限/)
  assert.throws(() => expandEpubForTransfer(zip, { ...EPUB_ARCHIVE_LIMITS, entryBytes: 10 }), /单个资源/)
  assert.throws(() => expandEpubForTransfer(zip, { ...EPUB_ARCHIVE_LIMITS, entries: 1 }), /条目数量/)
  const damaged = zip.slice()
  const view = new DataView(damaged.buffer)
  for (let p = 0; p + 46 <= damaged.length; p++) {
    const sig = view.getUint32(p, true)
    if (sig === 0x04034b50) view.setUint32(p + 22, 21, true)
    if (sig === 0x02014b50) view.setUint32(p + 24, 21, true)
  }
  assert.throws(() => expandEpubForTransfer(damaged), /实际资源大小与声明不一致/)
  assert.throws(() => expandEpubForTransfer(zipSync({ 'a.bin': new Uint8Array(), './a.bin': new Uint8Array() })), /重复或冲突/)
  const empty = expandEpubForTransfer(zipSync({ 'dir/': new Uint8Array(), 'dir/empty.bin': new Uint8Array() }))
  assert.deepEqual([...empty.keys()], ['dir/empty.bin'])
  assert.equal(empty.get('dir/empty.bin')!.byteLength, 0)
  assertCovered(empty)
})

test('实际分配器先只计划；只有请求该组首项才分配，不提前保留整书容量', () => {
  const source = readFileSync(new URL('./epubResourceBuffers.ts', import.meta.url), 'utf8')
  const code = ts.transpileModule(source, { compilerOptions: {
    module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText
  const allocations: number[] = []
  class ObservedBytes extends Uint8Array {
    constructor(...args: any[]) {
      super(...args as [number])
      if (typeof args[0] === 'number') allocations.push(args[0])
    }
  }
  const scope = { exports: {} as any, Uint8Array: ObservedBytes,
    require: () => ({ EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES }) }
  runInNewContext(code, scope)
  const sizes = new Map(Array.from({ length: 70 }, (_, n) => [String(n), 1024 * 1024]))
  const allocate = scope.exports.packedEpubResourceAllocator(sizes)
  assert.deepEqual(allocations, [])
  const first = allocate('0')
  assert.deepEqual(allocations, [2 * 1024 * 1024])
  assert.equal(allocate('1').buffer, first.buffer)
  assert.deepEqual(allocations, [2 * 1024 * 1024])
  assert.notEqual(allocate('2').buffer, first.buffer)
  assert.deepEqual(allocations, [2 * 1024 * 1024, 2 * 1024 * 1024])
})

test('共享分配器拒绝无效长度和未声明资源，不产生隐式空实现', () => {
  for (const value of [-1, NaN, Infinity, 0.5, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(() => packedEpubResourceAllocator(new Map([['a', value]])), /预算无效/)
  }
  assert.throws(() => packedEpubResourceAllocator(new Map())('missing'), /缺少资源缓冲声明/)
})
