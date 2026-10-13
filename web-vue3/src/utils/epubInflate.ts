/** ZIP 分帧仍由 fflate 负责；DEFLATE 输出用固定上限缓冲，不随压缩比扩容。 */
import { Inflate } from 'pako'

export const EPUB_INFLATE_CHUNK_BYTES = 64 * 1024

export class BoundedEpubInflate {
  static readonly compression = 8
  ondata!: (error: Error | null, data: Uint8Array | null, final: boolean) => void
  private readonly decoder: Inflate & { ended: boolean, strm: { avail_in: number } }
  private readonly compressedBytes: number
  private received = 0
  private finished = false

  constructor(compressedBytes: number, expandedBytes: number) {
    if (![compressedBytes, expandedBytes].every(size => Number.isSafeInteger(size) && size >= 0)) {
      throw new Error('EPUB ZIP 解码预算无效')
    }
    this.compressedBytes = compressedBytes
    this.decoder = new Inflate({ raw: true,
      chunkSize: Math.max(1, Math.min(EPUB_INFLATE_CHUNK_BYTES, expandedBytes + 1)) }) as typeof this.decoder
    // 不使用默认的 chunks/result 聚合：调用方立即核对实际计数并写入独占目的缓冲。
    this.decoder.onData = chunk => {
      if (!(chunk instanceof Uint8Array)) throw new Error('EPUB ZIP 解码器输出类型无效')
      this.ondata(null, chunk, false)
    }
  }

  push(chunk: Uint8Array, final: boolean): void {
    if (this.finished) throw new Error('EPUB ZIP 解码器重复结束')
    this.received += chunk.length
    if (this.received > this.compressedBytes) throw new Error('EPUB ZIP 实际压缩量与声明不一致')
    if (this.decoder.ended) {
      if (chunk.length) throw new Error('EPUB ZIP DEFLATE 结束后存在多余数据')
    } else {
      if (!this.decoder.push(chunk, final) || this.decoder.err) {
        throw new Error('EPUB ZIP DEFLATE 无效或无法解压')
      }
      // 固定 2.1.0 的输入计数；禁止成功流后剩余压缩字节被静默忽略。
      const remaining = this.decoder.strm.avail_in
      if (remaining !== 0) throw new Error('EPUB ZIP DEFLATE 结束后存在多余数据')
    }
    if (final) {
      if (!this.decoder.ended || this.received !== this.compressedBytes) {
        throw new Error('EPUB ZIP DEFLATE 截断或压缩量与声明不一致')
      }
      this.finished = true
      this.ondata(null, new Uint8Array(), true)
    }
  }
}
