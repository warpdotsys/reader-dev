import { expandEpubArchive } from './epubArchive'
import { EPUB_TRANSFER_BATCH_BYTES, EPUB_TRANSFER_BATCH_ENTRIES } from './epubTransfer'

const scope = globalThis as unknown as {
  onmessage: ((event: MessageEvent<unknown>) => void) | null
  postMessage: (value: unknown, transfer: Transferable[]) => void
  close: () => void
}

let files: Map<string, Uint8Array> | null = null
let started = false
let closed = false
let sequence = 0
let entries = 0
let bytes = 0

function closeTransfer(): void {
  closed = true
  files?.clear()
  files = null
  scope.onmessage = null
  scope.close()
}

function fail(error: unknown): void {
  try {
    const detail = error instanceof Error ? error.message : ''
    scope.postMessage({ error: detail.startsWith('EPUB ') ? detail : 'EPUB ZIP 无效或无法解压' }, [])
  } finally { closeTransfer() }
}

function sendNextBatch(): void {
  if (!files?.size) {
    // Only the last matching ACK can reach this point after a nonempty result.
    scope.postMessage({ complete: true, entries, bytes }, [])
    closeTransfer()
    return
  }
  const batch: [string, Uint8Array][] = []
  let batchBytes = 0
  for (const [name, data] of files) {
    if (batch.length && (batch.length >= EPUB_TRANSFER_BATCH_ENTRIES
      || batchBytes + data.byteLength > EPUB_TRANSFER_BATCH_BYTES)) break
    // An already-budgeted resource larger than 2 MiB travels alone, intact.
    batch.push([name, data])
    batchBytes += data.byteLength
    files.delete(name)
  }
  entries += batch.length
  bytes += batchBytes
  scope.postMessage({ batch: sequence, files: batch }, batch.map(([, data]) => data.buffer as ArrayBuffer))
}

scope.onmessage = event => {
  if (closed) return
  try {
    if (!started) {
      if (!(event.data instanceof ArrayBuffer)) throw new Error('EPUB Worker 输入协议无效')
      started = true
      scope.postMessage({ phase: 'expanding' }, [])
      files = expandEpubArchive(new Uint8Array(event.data))
      scope.postMessage({ phase: 'transferring' }, [])
      sendNextBatch()
      return
    }
    const ack = event.data as { batchAccepted?: unknown }
    if (!ack || typeof ack !== 'object' || Object.keys(ack).length !== 1
      || ack.batchAccepted !== sequence || !files) throw new Error('EPUB Worker 结果确认协议无效')
    sequence++
    // Keep at most one batch queued; do not close while its delivery is pending.
    sendNextBatch()
  } catch (error) { fail(error) }
}

// 只报告固定阶段，不包含文件名、正文或输入；主线程仍使用同一条总超时。
scope.postMessage({ phase: 'ready' }, [])
