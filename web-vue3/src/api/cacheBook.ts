import { useUserStore } from '@/stores/user'
import { get, post } from './request'
import { parseSSEBlock, consumeSSEStreamBlocks } from './sse'
import type { ReturnData } from '@/types'

/**
 * 服务端缓存本书 —— 后端契约（后端并行实现中，未就绪时 silent 降级提示）
 *
 * POST /reader3/cacheBookOnServer?url=<bookUrl>   → 启动后台整书缓存任务，立即返回
 * POST /reader3/cacheBookRangeOnServer           → 启动后台章节范围缓存，返回 taskId
 *      ReturnData<{ started, url, cached, total, title }>
 * GET  /reader3/cacheBookSSE?url=<bookUrl>        → SSE 进度流（约每 300ms 一帧；taskId 精确订阅）
 *      data: { cached, total, title, finished, cancelled, error }
 * GET  /reader3/cancelCacheBook?url=<bookUrl>     → 取消任务（内存任务表移除）
 *
 * 说明：任务契约描述为「POST 即 SSE 进度」，后端实现为「POST 启动 + cacheBookSSE 轮询推送」，
 * 此处按后端实现对接（两者进度事件结构一致：{cached,total,title}）。
 * 传输层失败（网络错误 / 非 200）reject，由调用方 silent 降级提示。
 */

/** POST /reader3/cacheBookOnServer 启动结果 */
export interface CacheStartResult {
  started: boolean
  url?: string
  cached: number
  total: number
  title?: string
}

/** POST /reader3/cacheBookRangeOnServer 启动结果（taskId 供 SSE/取消精确订阅） */
export interface CacheRangeStartResult extends CacheStartResult {
  taskId?: string
}

/** 缓存进度帧（SSE data） */
export interface CacheSSEProgress {
  cached: number
  total: number
  title?: string
  finished?: boolean
  cancelled?: boolean
  error?: string | null
}

export interface CacheProgressCallbacks {
  /** 进度帧到达 */
  onProgress: (p: CacheSSEProgress) => void
  /** 流正常结束（含 finished 帧后结束） */
  onEnd?: () => void
  /** 传输层失败（非用户关闭） */
  onStreamError: (msg: string) => void
}

export interface CacheProgressHandle {
  close: () => void
}

/** POST /reader3/cacheBookOnServer：启动后台缓存（silent——未就绪时调用方降级） */
export function cacheBookOnServer(url: string): Promise<ReturnData<CacheStartResult>> {
  return post<CacheStartResult>('/cacheBookOnServer', { url }, { silent: true })
}

/** POST /reader3/cacheBookRangeOnServer：启动目录实章 0 基闭区间缓存任务 */
export function cacheBookRangeOnServer(
  url: string,
  from: number,
  to: number,
): Promise<ReturnData<CacheRangeStartResult>> {
  return post<CacheRangeStartResult>('/cacheBookRangeOnServer', { url, from, to }, { silent: true })
}

/** GET /reader3/getBookCacheChapters：拉取服务器已缓存章节（目录缓存标记用） */
export interface ServerCachedChapter {
  index: number
  title: string
  content: string
}

export interface ServerCachedChapters {
  url: string
  chapters: ServerCachedChapter[]
  hasMore: boolean
}

export function getBookCacheChapters(
  url: string,
): Promise<ReturnData<ServerCachedChapters>> {
  return get<ServerCachedChapters>('/getBookCacheChapters', { url }, { silent: true })
}

function tryJson(s: string): unknown {
  try {
    return JSON.parse(s) as unknown
  } catch {
    return null
  }
}

function dispatchSSEBlock(block: string, cbs: CacheProgressCallbacks) {
  // SSE 事件块解析统一走 api/sse.ts（P2：SSE 解析三处统一）
  const evt = parseSSEBlock(block)
  if (!evt || !evt.data) return
  const p = tryJson(evt.data) as (CacheSSEProgress & { type?: unknown; message?: unknown }) | null
  if (!p || typeof p !== 'object') return
  // 兼容两种命名：`event: progress` 与无名事件 + JSON.type=progress（后端当前为无名 data 帧）
  const isProgress =
    evt.event === 'progress' || evt.event === '' || evt.event === 'message' || p.type === 'progress'
  if (isProgress && typeof p.cached === 'number' && typeof p.total === 'number') {
    cbs.onProgress({
      cached: Math.max(0, p.cached),
      total: Math.max(0, p.total),
      title: typeof p.title === 'string' ? p.title : undefined,
      finished: p.finished === true,
      cancelled: p.cancelled === true,
      error: typeof p.error === 'string' ? p.error : null,
    })
    if (p.finished === true) cbs.onEnd?.()
  } else if (evt.event === 'error' || p.type === 'error') {
    cbs.onStreamError(typeof p.message === 'string' ? p.message : '缓存任务失败')
  }
}

async function consumeSSEStream(
  body: ReadableStream<Uint8Array>,
  cbs: CacheProgressCallbacks,
  closed: () => boolean,
): Promise<void> {
  // 块切分/增量消费统一走 api/sse.ts（P2：SSE 解析三处统一）
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
  // 仅正常结束触发 onEnd（连接中断/用户关闭不触发——调用方据此判断「缓存完成」）
  if (!streamFailed && !closed()) cbs.onEnd?.()
}

/** GET /reader3/cacheBookSSE：订阅缓存进度流（原生 fetch 流式读取，accessToken 手动附加） */
export function cacheBookSSE(
  key: string,
  cbs: CacheProgressCallbacks,
  useTaskId = false,
): Promise<CacheProgressHandle> {
  const controller = new AbortController()
  const token = useUserStore().accessToken
  const idParam = useTaskId ? 'taskId' : 'url'
  const query = token
    ? `?${idParam}=${encodeURIComponent(key)}&accessToken=${encodeURIComponent(token)}`
    : `?${idParam}=${encodeURIComponent(key)}`
  return fetch(`/reader3/cacheBookSSE${query}`, {
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

/** GET /reader3/cancelCacheBook：取消缓存任务（taskId 精确取消；silent——未就绪时调用方降级） */
export function cancelCacheBook(
  key: string,
  useTaskId = false,
): Promise<ReturnData<{ cancelled: boolean }>> {
  return get<{ cancelled: boolean }>(
    '/cancelCacheBook',
    useTaskId ? { taskId: key } : { url: key },
    { silent: true },
  )
}
