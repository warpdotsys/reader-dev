/** 专用单 Worker：转移输入/输出，不在主线程同步解压整本书；终止和超时统一收尾。 */
import { EPUB_ARCHIVE_LIMITS } from './epubArchive.ts'
import { EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES, type EpubTransferAck } from './epubTransfer.ts'

export interface EpubArchiveWorker {
  onmessage: ((event: MessageEvent) => void) | null
  onerror: ((event: ErrorEvent) => void) | null
  postMessage: (value: ArrayBuffer | EpubTransferAck, transfer: Transferable[]) => void
  terminate: () => void
}

export function expandEpubInWorker(bytes: Uint8Array, signal?: AbortSignal,
  createWorker: () => EpubArchiveWorker = () => {
    if (typeof Worker === 'undefined') throw new Error('当前浏览器不支持 EPUB 原版排版解压 Worker')
    return new Worker(new URL('./epubArchive.worker.ts', import.meta.url), { type: 'module' })
  },
  timeoutMs = 15000): Promise<Map<string, Uint8Array>> {
  if (signal?.aborted) return Promise.reject(new DOMException('EPUB 加载已取消', 'AbortError'))
  return new Promise((resolve, reject) => {
    const worker = createWorker()
    let finished = false
    const received = new Map<string, Uint8Array>()
    let sequence = 0
    let receivedBytes = 0
    let phase: 'starting' | 'ready' | 'expanding' | 'transferring' = 'starting'
    const phaseText = { starting: '等待 Worker 启动', ready: '等待接收文件', expanding: '解压文件', transferring: '等待解压结果传递' }
    const timer = setTimeout(() => finish(new Error(
      `EPUB 解压超时（15 秒；阶段：${phaseText[phase]}），已终止原版排版加载`)), timeoutMs)
    const abort = () => finish(new DOMException('EPUB 加载已取消', 'AbortError'))
    function finish(error?: Error, files?: Map<string, Uint8Array>) {
      if (finished) return
      finished = true
      clearTimeout(timer)
      signal?.removeEventListener('abort', abort)
      worker.onmessage = worker.onerror = null
      worker.terminate()
      if (error) { received.clear(); reject(error) }
      else resolve(files!)
    }
    function sendInput() {
      if (finished) return
      if (signal?.aborted) { abort(); return }
      try {
        // 在固定 ready 确认后才转移整本输入，不将大文件排入尚未启动的 Worker。
        // 仍使用创建时的同一条总期限；子视图只复制有效字节，不转移无关数据。
        const input = bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength
          && bytes.buffer instanceof ArrayBuffer ? bytes.buffer : bytes.slice().buffer as ArrayBuffer
        worker.postMessage(input, [input])
      } catch (error) { finish(error instanceof Error ? error : new Error('EPUB Worker 启动失败')) }
    }
    worker.onmessage = event => {
      if (finished) return
      const reply = event.data as { phase?: unknown; batch?: unknown; files?: unknown; error?: string;
        complete?: unknown; entries?: unknown; bytes?: unknown }
      if (reply && typeof reply === 'object' && 'phase' in reply) {
        // 阶段确认不是结果，也不能重启/延长计时器；拒绝混合、重复或逆序协议。
        if (Object.keys(reply).length !== 1
          || !((phase === 'starting' && reply.phase === 'ready')
            || (phase === 'ready' && reply.phase === 'expanding')
            || (phase === 'expanding' && reply.phase === 'transferring'))) {
          finish(new Error('EPUB 解压 Worker 返回无效阶段')); return
        }
        phase = reply.phase as 'ready' | 'expanding' | 'transferring'
        if (phase === 'ready') sendInput()
        return
      }
      if (typeof reply?.error === 'string') { finish(new Error(reply.error)); return }
      if (phase !== 'transferring' || !reply || typeof reply !== 'object') {
        finish(new Error('EPUB 解压 Worker 返回无效数据')); return
      }
      if ('complete' in reply) {
        if (Object.keys(reply).length !== 3 || reply.complete !== true
          || reply.entries !== received.size || reply.bytes !== receivedBytes) {
          finish(new Error('EPUB 解压 Worker 返回无效数据')); return
        }
        finish(undefined, received)
        return
      }
      if (Object.keys(reply).length !== 2 || reply.batch !== sequence || !Array.isArray(reply.files)
        || !reply.files.length || reply.files.length > EPUB_TRANSFER_BATCH_ENTRIES) {
        finish(new Error('EPUB 解压 Worker 返回无效数据')); return
      }
      const names = new Set<string>()
      let batchBytes = 0
      for (const entry of reply.files) {
        if (!Array.isArray(entry) || entry.length !== 2 || typeof entry[0] !== 'string' || !entry[0]
          || !(entry[1] instanceof Uint8Array) || !(entry[1].buffer instanceof ArrayBuffer)
          || entry[1].byteOffset !== 0 || entry[1].byteLength !== entry[1].buffer.byteLength
          || received.has(entry[0]) || names.has(entry[0])) {
          finish(new Error('EPUB 解压 Worker 返回无效数据')); return
        }
        const limit = /\.(xml|opf|ncx|x?html?|css)$/i.test(entry[0])
          ? EPUB_ARCHIVE_LIMITS.textBytes : EPUB_ARCHIVE_LIMITS.entryBytes
        if (entry[1].byteLength > limit) { finish(new Error('EPUB 解压 Worker 资源超过原版排版上限')); return }
        names.add(entry[0])
        batchBytes += entry[1].byteLength
      }
      if ((reply.files.length > 1 && batchBytes > EPUB_TRANSFER_BATCH_BYTES)
        || received.size + names.size > EPUB_ARCHIVE_LIMITS.entries
        || receivedBytes + batchBytes > EPUB_ARCHIVE_LIMITS.expandedBytes) {
        finish(new Error('EPUB 解压 Worker 资源超过原版排版上限')); return
      }
      for (const [name, data] of reply.files as [string, Uint8Array][]) received.set(name, data)
      receivedBytes += batchBytes
      sequence++
      try { worker.postMessage({ batchAccepted: sequence - 1 }, []) }
      catch (error) { finish(error instanceof Error ? error : new Error('EPUB Worker 结果确认失败')) }
    }
    worker.onerror = event => { event.preventDefault(); finish(new Error('EPUB 解压 Worker 运行失败')) }
    signal?.addEventListener('abort', abort, { once: true })
    if (signal?.aborted) { abort(); return }
  })
}
