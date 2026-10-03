/** 专用单 Worker：转移输入/输出，不在主线程同步解压整本书；终止和超时统一收尾。 */
export interface EpubArchiveWorker {
  onmessage: ((event: MessageEvent) => void) | null
  onerror: ((event: ErrorEvent) => void) | null
  postMessage: (value: ArrayBuffer, transfer: Transferable[]) => void
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
    const timer = setTimeout(() => finish(new Error('EPUB 解压超时（15 秒），已终止原版排版加载')), timeoutMs)
    const abort = () => finish(new DOMException('EPUB 加载已取消', 'AbortError'))
    function finish(error?: Error, files?: Map<string, Uint8Array>) {
      if (finished) return
      finished = true
      clearTimeout(timer)
      signal?.removeEventListener('abort', abort)
      worker.onmessage = worker.onerror = null
      worker.terminate()
      if (error) reject(error)
      else resolve(files!)
    }
    worker.onmessage = event => {
      const reply = event.data as { files?: [string, Uint8Array][]; error?: string }
      if (typeof reply?.error === 'string') { finish(new Error(reply.error)); return }
      if (!Array.isArray(reply?.files) || !reply.files.every(entry => Array.isArray(entry)
        && typeof entry[0] === 'string' && entry[1] instanceof Uint8Array)) {
        finish(new Error('EPUB 解压 Worker 返回无效数据')); return
      }
      finish(undefined, new Map(reply.files))
    }
    worker.onerror = event => { event.preventDefault(); finish(new Error('EPUB 解压 Worker 运行失败')) }
    signal?.addEventListener('abort', abort, { once: true })
    if (signal?.aborted) { abort(); return }
    try {
      // 网络读取返回独占的连续 ArrayBuffer；测试用视图不能意外转移无关数据。
      const input = bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength
        && bytes.buffer instanceof ArrayBuffer ? bytes.buffer : bytes.slice().buffer as ArrayBuffer
      worker.postMessage(input, [input])
    } catch (error) { finish(error instanceof Error ? error : new Error('EPUB Worker 启动失败')) }
  })
}
