/** 先检查 ZIP 目录预算，再按小块核对实际输出；不信任条目声明的大小。 */
import { Unzip, unzipSync, type UnzipFileInfo } from 'fflate'
import { BoundedEpubInflate } from './epubInflate.ts'

export interface EpubArchiveLimits {
  compressedBytes: number
  expandedBytes: number
  entryBytes: number
  textBytes: number
  entries: number
}

export const EPUB_ARCHIVE_LIMITS: Readonly<EpubArchiveLimits> = Object.freeze({
  compressedBytes: 64 * 1024 * 1024,
  expandedBytes: 256 * 1024 * 1024,
  entryBytes: 32 * 1024 * 1024,
  textBytes: 4 * 1024 * 1024,
  entries: 8192,
})

function zipKey(name: string): string {
  let key = name.replace(/\\/g, '/')
  try { key = decodeURIComponent(key) } catch { /* 非编码文件名 */ }
  if (key.startsWith('/') || key.includes('\0') || /^[a-z][a-z0-9+.-]*:/i.test(key)) {
    throw new Error('EPUB ZIP 条目路径无效')
  }
  const parts: string[] = []
  for (const part of key.split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      if (!parts.length) throw new Error('EPUB ZIP 条目越过书内根目录')
      parts.pop()
    } else parts.push(part)
  }
  if (!parts.length) throw new Error('EPUB ZIP 条目路径为空')
  return parts.join('/') + (key.endsWith('/') ? '/' : '')
}

function entryLimit(key: string, limits: Readonly<EpubArchiveLimits>): number {
  return /\.(xml|opf|ncx|x?html?|css)$/i.test(key) ? Math.min(limits.entryBytes, limits.textBytes) : limits.entryBytes
}

export function expandEpubArchive(bytes: Uint8Array,
  limits: Readonly<EpubArchiveLimits> = EPUB_ARCHIVE_LIMITS): Map<string, Uint8Array> {
  if (bytes.length > limits.compressedBytes) throw new Error('EPUB 文件超过原版排版下载上限（64 MiB）')
  const declared = new Map<string, UnzipFileInfo>()
  let reserved = 0
  // filter=false 只读取中央目录，不按 originalSize 分配或解压任何资源。
  unzipSync(bytes, { filter: entry => {
    const key = zipKey(entry.name)
    if (declared.size >= limits.entries) throw new Error('EPUB ZIP 条目数量超过原版排版上限')
    if (declared.has(key)) throw new Error('EPUB ZIP 包含重复或冲突路径')
    if (![entry.size, entry.originalSize].every(size => Number.isSafeInteger(size) && size >= 0)
      || ![0, 8].includes(entry.compression)) throw new Error('EPUB ZIP 条目大小或压缩格式无效')
    if (entry.size > bytes.length || entry.originalSize > entryLimit(key, limits)) {
      throw new Error('EPUB 单个资源超过原版排版上限')
    }
    reserved += entry.originalSize
    if (reserved > limits.expandedBytes) throw new Error('EPUB 解压总量超过原版排版上限（256 MiB）')
    declared.set(key, entry)
    return false
  } })
  const files = new Map<string, Uint8Array>()
  const seen = new Set<string>()
  let expanded = 0
  let completed = 0
  const archive = new Unzip(file => {
    const key = zipKey(file.name)
    const expected = declared.get(key)
    if (!expected || seen.has(key) || file.compression !== expected.compression
      || (file.size !== undefined && file.size !== expected.size)
      || (file.originalSize !== undefined && file.originalSize !== expected.originalSize)) {
      throw new Error('EPUB ZIP 本地条目与中央目录不一致')
    }
    seen.add(key)
    let size = 0
    // The central directory and local entry have already passed the size and
    // aggregate budgets. Fill one owned destination instead of retaining all
    // emitted chunks and allocating/copying another complete resource at EOF.
    // DEFLATE emits at most 64 KiB per callback; no output-ratio-sized temporary.
    const result = key.endsWith('/') ? undefined : new Uint8Array(expected.originalSize)
    file.ondata = (error, chunk, final) => {
      if (error) throw error
      if (chunk) {
        const offset = size
        size += chunk.length
        expanded += chunk.length
        if (size > expected.originalSize || size > entryLimit(key, limits) || expanded > limits.expandedBytes) {
          throw new Error('EPUB 实际解压量超过声明或原版排版上限')
        }
        if (chunk.length && result) result.set(chunk, offset)
      }
      if (final) {
        if (size !== expected.originalSize) throw new Error('EPUB ZIP 实际资源大小与声明不一致')
        completed++
        if (result) files.set(key, result)
      }
    }
    file.start()
  })
  class ArchiveInflate extends BoundedEpubInflate {
    constructor(name: string) {
      const expected = declared.get(zipKey(name))
      if (!expected) throw new Error('EPUB ZIP 缺少声明条目')
      super(expected.size, expected.originalSize)
    }
  }
  archive.register(ArchiveInflate)
  // 单次输入小块，限制解码器在调用 ondata 之前的临时展开；生产另有 Worker 超时。
  for (let offset = 0; offset < bytes.length; offset += 4096) {
    const end = Math.min(offset + 4096, bytes.length)
    archive.push(bytes.subarray(offset, end), end === bytes.length)
  }
  if (completed !== declared.size || seen.size !== declared.size) throw new Error('EPUB ZIP 缺少完整本地条目')
  return files
}

/** Bounded owned buffer: the header is a capacity hint, never proof of actual bytes. */
export class EpubDownloadBuffer {
  private chunks: Uint8Array[] = []
  private owned: Uint8Array | null = null
  private size = 0
  private hint: number | null
  private readonly limit: number

  constructor(limit: number, declared: number | null) {
    this.limit = limit
    this.hint = declared !== null && Number.isSafeInteger(declared) && declared > 0 && declared <= limit
      ? declared : null
  }

  get retainedBytes(): number {
    return (this.owned?.byteLength ?? 0) + this.chunks.reduce((total, chunk) => total + chunk.byteLength, 0)
  }

  push(chunk: Uint8Array): void {
    const next = this.size + chunk.byteLength
    if (next > this.limit) throw new Error('EPUB 文件超过原版排版下载上限（64 MiB）')
    if (this.hint !== null && next > this.hint) {
      // An understated header must not truncate data or exempt it from the cap.
      if (this.owned) this.chunks.push(this.owned.subarray(0, this.size))
      this.owned = null
      this.hint = null
    }
    if (this.owned) this.owned.set(chunk, this.size)
    else {
      this.chunks.push(chunk)
      // Do not amplify an empty/small body to a large untrusted header hint.
      // Large hints need at least half their bytes (and a MiB) before reserving;
      // real exact-length files then stop retaining chunks plus a second whole ZIP.
      const threshold = this.hint === null ? Infinity : this.hint <= 1024 * 1024
        ? this.hint : Math.max(1024 * 1024, Math.ceil(this.hint / 2))
      if (this.hint !== null && next >= threshold) {
        this.owned = new Uint8Array(this.hint)
        let offset = 0
        for (const value of this.chunks) { this.owned.set(value, offset); offset += value.byteLength }
        this.chunks.length = 0
      }
    }
    this.size = next
  }

  finish(): Uint8Array {
    try {
      if (this.owned) return this.size === this.owned.byteLength ? this.owned : this.owned.slice(0, this.size)
      const result = new Uint8Array(this.size)
      let offset = 0
      for (const chunk of this.chunks) { result.set(chunk, offset); offset += chunk.byteLength }
      return result
    } finally { this.clear() }
  }

  clear(): void {
    this.chunks.length = 0
    this.owned = null
    this.size = 0
  }
}

/** 按实际字节限流，Content-Length仅作有界容量提示；出错/超限必须取消响应流。 */
export async function readEpubResponse(response: Response,
  limit = EPUB_ARCHIVE_LIMITS.compressedBytes): Promise<Uint8Array> {
  const declared = response.headers.get('content-length')
  if (declared && /^\d+$/.test(declared) && Number(declared) > limit) {
    await response.body?.cancel()
    throw new Error('EPUB 文件超过原版排版下载上限（64 MiB）')
  }
  if (!response.body) throw new Error('EPUB 下载响应缺少正文流')
  const reader = response.body.getReader()
  const bytes = new EpubDownloadBuffer(limit, declared && /^\d+$/.test(declared) ? Number(declared) : null)
  try {
    for (;;) {
      const { done, value } = await reader.read()
      if (done) break
      bytes.push(value)
    }
    return bytes.finish()
  } catch (error) {
    try { await reader.cancel() } catch { /* 原下载错误优先 */ }
    throw error
  } finally { bytes.clear(); reader.releaseLock() }
}
