import { useUserStore } from '@/stores/user'
import { post } from './request'
import { parseSSEBlock, consumeSSEStreamBlocks } from './sse'
import type { ReturnData } from '@/types'

/**
 * 已验证的 Java/Kotlin 服务端缓存契约。
 *
 * POST /reader3/cacheBookOnServer body: { bookUrlList: string[] }
 *   异步整书预缓存；不提供任务号或进度。
 * GET /reader3/cacheBookSSE?url=<bookUrl>&refresh=0&concurrentCount=24
 *   当前请求直接执行整书缓存。SSE data 为
 *   { cachedCount, successCount, failedCount }，以 event: end 结束。
 *
 * 服务端没有范围缓存、任务取消或已缓存章节列表 API。关闭 SSE 连接是唯一可用的停止方式；
 * 调用方不得把它包装成精确取消或范围缓存。
 */

export interface CacheSSEProgress {
  /** 缓存目录中已有章节的总数（含本次已写入） */
  cachedCount: number
  /** 本次 SSE 成功写入的章节数 */
  successCount: number
  /** 本次 SSE 拉取失败的章节数 */
  failedCount: number
}

export interface CacheProgressCallbacks {
  onProgress: (p: CacheSSEProgress) => void
  onEnd?: () => void
  onStreamError: (msg: string) => void
}

export interface CacheProgressHandle {
  close: () => void
}

/** 无进度的整书后台预缓存。需要进度时使用 cacheBookSSE。 */
export function cacheBookOnServer(bookUrlList: string[]): Promise<ReturnData<string>> {
  return post<string>('/cacheBookOnServer', { bookUrlList }, { silent: true })
}

function tryJson(s: string): unknown {
  try {
    return JSON.parse(s) as unknown
  } catch {
    return null
  }
}

function dispatchSSEBlock(block: string, cbs: CacheProgressCallbacks) {
  const evt = parseSSEBlock(block)
  if (!evt || !evt.data) return
  const p = tryJson(evt.data) as (CacheSSEProgress & { message?: unknown }) | null
  if (!p || typeof p !== 'object') return
  if (typeof p.cachedCount === 'number') {
    cbs.onProgress({
      cachedCount: Math.max(0, p.cachedCount),
      successCount: typeof p.successCount === 'number' ? Math.max(0, p.successCount) : 0,
      failedCount: typeof p.failedCount === 'number' ? Math.max(0, p.failedCount) : 0,
    })
  } else if (evt.event === 'error') {
    cbs.onStreamError(typeof p.message === 'string' ? p.message : '缓存任务失败')
  }
}

async function consumeSSEStream(
  body: ReadableStream<Uint8Array>,
  cbs: CacheProgressCallbacks,
  closed: () => boolean,
): Promise<void> {
  let streamFailed = false
  await consumeSSEStreamBlocks(
    body,
    (block) => dispatchSSEBlock(block, cbs),
    closed,
    (msg) => {
      streamFailed = true
      cbs.onStreamError(msg)
    },
  )
  if (!streamFailed && !closed()) cbs.onEnd?.()
}

/** 订阅并执行整书缓存；refresh=1 会无视已有正文缓存重新拉取。 */
export function cacheBookSSE(
  bookUrl: string,
  cbs: CacheProgressCallbacks,
  options: { refresh?: number; concurrentCount?: number } = {},
): Promise<CacheProgressHandle> {
  const controller = new AbortController()
  const token = useUserStore().accessToken
  const params = new URLSearchParams({ url: bookUrl })
  if (typeof options.refresh === 'number') params.set('refresh', String(options.refresh))
  if (typeof options.concurrentCount === 'number') params.set('concurrentCount', String(options.concurrentCount))
  if (token) params.set('accessToken', token)
  return fetch(`/reader3/cacheBookSSE?${params.toString()}`, {
    method: 'GET',
    headers: { Accept: 'text/event-stream' },
    signal: controller.signal,
  }).then(async (response) => {
    if (!response.ok) throw new Error(`缓存进度服务异常（HTTP ${response.status}）`)
    if (!response.body) throw new Error('缓存进度服务未返回数据流')
    void consumeSSEStream(response.body, cbs, () => controller.signal.aborted)
    return { close: () => controller.abort() } satisfies CacheProgressHandle
  })
}
