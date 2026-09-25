import { useUserStore } from '@/stores/user'
import { parseSSEBlock, consumeSSEStreamBlocks } from './sse'

/**
 * 书源调试 —— 后端契约（并行实现中）
 *
 * GET /reader3/bookSourceDebugSSE?bookSource=<书源URL>&action=search|explore|toc|content&key=<关键词>&chapterUrl=<章节URL>
 *   → text/event-stream，事件体（无名 data 帧，兼容 event: step/result 命名）：
 *       {type: 'start',  message: {action, bookSource}}        开始
 *       {type: 'step',   message: {ruleName,url,elapsedMs,resultLen,error,detail}}  逐步日志（每次一步）
 *       {type: 'result', data: …}                             最终结果（JSON 任意结构）
 *       {type: 'error',  message: '…'}                        失败
 *
 * 实现说明：
 * - 参数名以实际后端为准：bookSource（必填）+ action + key + chapterUrl；
 * - 原生 fetch 流式读取（GET 路由，accessToken 手动附加 query——无 axios 拦截器）；
 * - 后端未就绪（网络错误 / HTTP 非 200）reject，由调用方 silent 降级提示，不弹全局 toast。
 */

export type DebugAction = 'search' | 'explore' | 'toc' | 'content'

export interface DebugSSEParams {
  /** 被测书源 URL */
  bookSourceUrl: string
  /** 动作：search=搜索 / explore=探索 / toc=目录 / content=正文 */
  action: DebugAction
  /** action=search 时的关键词 */
  key?: string
  /** action=toc|content 时的章节 URL（toc 为书 URL） */
  chapterUrl?: string
}

export interface DebugSSECallbacks {
  /** 逐步日志（type=step） */
  onStep: (message: string) => void
  /** 最终结果（type=result；data 为任意 JSON） */
  onResult: (data: unknown) => void
  /** 流正常结束 */
  onEnd?: () => void
  /** 传输层失败（后端未就绪 / 连接断开，非用户关闭） */
  onStreamError: (msg: string) => void
}

export interface DebugSSEHandle {
  close: () => void
}

function tryJson(s: string): unknown {
  try {
    return JSON.parse(s) as unknown
  } catch {
    return null
  }
}

/** 日志消息渲染：字符串直出，对象/数组 JSON 缩进格式化 */
function formatMessage(m: unknown): string {
  if (typeof m === 'string') return m
  try {
    return JSON.stringify(m, null, 2)
  } catch {
    return String(m)
  }
}

function dispatchSSEBlock(block: string, cbs: DebugSSECallbacks) {
  // SSE 事件块解析统一走 api/sse.ts（P2：SSE 解析三处统一）
  const evt = parseSSEBlock(block)
  if (!evt || !evt.data) return
  // 后端当前输出无名 data 帧 + JSON.type；兼容 event: step/result 命名
  if (evt.event === 'step') {
    const p = tryJson(evt.data) as { message?: unknown } | null
    cbs.onStep(p && 'message' in p ? formatMessage(p.message) : evt.data)
  } else if (evt.event === 'result') {
    const p = tryJson(evt.data) as { data?: unknown } | null
    cbs.onResult(p && 'data' in p ? p.data : (p ?? evt.data))
  } else if (evt.event === 'message' || evt.event === '') {
    const p = tryJson(evt.data) as { type?: unknown; message?: unknown; data?: unknown } | null
    if (p && typeof p === 'object') {
      if (p.type === 'start' || p.type === 'step') {
        cbs.onStep(formatMessage(p.message))
      } else if (p.type === 'result') {
        cbs.onResult('data' in p ? p.data : p)
      } else if (p.type === 'error') {
        cbs.onStreamError(typeof p.message === 'string' ? p.message : '调试失败')
      }
    }
  }
}

async function consumeSSEStream(
  body: ReadableStream<Uint8Array>,
  cbs: DebugSSECallbacks,
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
  // 仅正常结束触发 onEnd（连接中断/用户关闭不触发）
  if (!streamFailed && !closed()) cbs.onEnd?.()
}

/**
 * 建立书源调试 SSE 连接（原生 fetch 流式读取）。
 * 传输层失败（网络错误 / 非 200 / 非 event-stream 响应）reject，调用方可 silent 降级提示。
 */
export function bookSourceDebugSSE(
  params: DebugSSEParams,
  cbs: DebugSSECallbacks,
): Promise<DebugSSEHandle> {
  const controller = new AbortController()
  const token = useUserStore().accessToken
  const query = new URLSearchParams({ bookSource: params.bookSourceUrl, action: params.action })
  if (params.key) query.set('key', params.key)
  if (params.chapterUrl) query.set('chapterUrl', params.chapterUrl)
  if (token) query.set('accessToken', token)

  return fetch(`/reader3/bookSourceDebugSSE?${query.toString()}`, {
    method: 'GET',
    headers: { Accept: 'text/event-stream' },
    signal: controller.signal,
  }).then(async (response) => {
    if (!response.ok) throw new Error(`调试服务异常（HTTP ${response.status}）`)
    if (!response.body) throw new Error('调试服务未返回数据流')
    void consumeSSEStream(response.body, cbs, () => controller.signal.aborted)
    return { close: () => controller.abort() } satisfies DebugSSEHandle
  })
}
