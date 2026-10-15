import { EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES } from './epubTransfer.ts'

/** 只计划分组，遇到该组的首个本地条目才分配；总容量不含填充或额外副本。 */
export function packedEpubResourceAllocator(sizes: ReadonlyMap<string, number>): (name: string) => Uint8Array {
  type Group = { size: number; count: number; bytes?: Uint8Array }
  const locations = new Map<string, { group: Group; offset: number; size: number }>()
  let group: Group = { size: 0, count: 0 }
  for (const [name, size] of sizes) {
    if (!Number.isSafeInteger(size) || size < 0) throw new Error('EPUB ZIP 资源缓冲预算无效')
    if (group.count && (group.count >= EPUB_TRANSFER_BATCH_ENTRIES
      || group.size + size > EPUB_TRANSFER_BATCH_BYTES)) group = { size: 0, count: 0 }
    locations.set(name, { group, offset: group.size, size })
    group.size += size
    group.count++
  }
  return name => {
    const location = locations.get(name)
    if (!location) throw new Error('EPUB ZIP 缺少资源缓冲声明')
    const { group, offset, size } = location
    group.bytes ??= new Uint8Array(group.size)
    return group.bytes.subarray(offset, offset + size)
  }
}
