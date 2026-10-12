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
    let phase: 'starting' | 'ready' | 'expanding' = 'starting'
    const phaseText = { starting: '等待 Worker 启动', ready: '等待接收文件', expanding: '解压文件' }
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
      if (error) reject(error)
      else resolve(files!)
    }
    worker.onmessage = event => {
      const reply = event.data as { phase?: unknown; files?: [string, Uint8Array][]; error?: string }
      if (reply && typeof reply === 'object' && 'phase' in reply) {
        // 阶段确认不是结果，也不能重启/延长计时器；拒绝混合、重复或逆序协议。
        if (Object.keys(reply).length !== 1
          || !((phase === 'starting' && reply.phase === 'ready')
            || (phase === 'ready' && reply.phase === 'expanding'))) {
          finish(new Error('EPUB 解压 Worker 返回无效阶段')); return
        }
        phase = reply.phase as 'ready' | 'expanding'
        return
      }
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
