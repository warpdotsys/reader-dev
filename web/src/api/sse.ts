import type { ReturnData, SearchBook } from '../types'

/**
 * SSE 事件流通用解析/分发（searchBookMultiSSE / searchBookSourceSSE / bookSourceDebugSSE 共用）。
 * 纯函数 + 无外部依赖——node:test 可直接单测。
 *
 * 后端事件契约（legacy 兼容）：
 *   event: book  + data {"lastIndex": n, "data": [SearchBook]}
 *   event: end   + data {"lastIndex": n, "isEnd": true}
 *   event: error + data ReturnData（{isSuccess, errorMsg, data}）
 */

export interface ParsedSSEEvent {
  event: string
  data: string
}

/** 解析一个 SSE 事件块（event: / data: 行，兼容 \r\n；多行 data 以 \n 拼接） */
export function parseSSEBlock(block: string): ParsedSSEEvent | null {
  let event = ''
  const dataLines: string[] = []
  for (const rawLine of block.split('\n')) {
    const line = rawLine.endsWith('\r') ? rawLine.slice(0, -1) : rawLine
    if (line.startsWith('event:')) {
      event = line.slice(6).trim()
    } else if (line.startsWith('data:')) {
      dataLines.push(line.slice(5).replace(/^ /, ''))
    }
  }
  if (!dataLines.length) return null
  return { event, data: dataLines.join('\n') }
}

export interface SSEBookEndCallbacks {
  /** 单个书源结果到达（data 可能为空数组） */
  onBooks: (lastIndex: number, books: SearchBook[]) => void
  /** 流正常结束（event: end） */
  onEnd: (lastIndex: number, isEnd: boolean) => void
  /** 服务端业务错误（event: error，data 为 ReturnData） */
  onErrorEvent: (ret: ReturnData) => void
}

/** 分发一个 SSE 事件块到对应回调（book/end/error；无法解析的数据块忽略） */
export function dispatchSSEBlock(block: string, cbs: SSEBookEndCallbacks) {
  const evt = parseSSEBlock(block)
  if (!evt || !evt.data) return
  if (evt.event === 'book') {
    try {
      const payload = JSON.parse(evt.data) as { lastIndex?: number; data?: SearchBook[] }
      cbs.onBooks(payload.lastIndex ?? -1, Array.isArray(payload.data) ? payload.data : [])
    } catch {
      // 忽略无法解析的数据块
    }
  } else if (evt.event === 'end') {
    try {
      const payload = JSON.parse(evt.data) as { lastIndex?: number; isEnd?: boolean }
      cbs.onEnd(payload.lastIndex ?? -1, payload.isEnd ?? false)
    } catch {
      cbs.onEnd(-1, false)
    }
  } else if (evt.event === 'error') {
    try {
      cbs.onErrorEvent(JSON.parse(evt.data) as ReturnData)
    } catch {
      cbs.onErrorEvent({ isSuccess: false, errorMsg: evt.data, data: null })
    }
  }
}

export interface SSEStreamCallbacks extends SSEBookEndCallbacks {
  /** 流中途中断（连接断开，非用户取消） */
  onStreamError?: (msg: string) => void
}

/** 通用块消费：按 \n\n 切分事件块并逐块回调（cacheBookSSE / bookSourceDebugSSE 等
 *  非 book/end/error 事件流共用——块内解析由 onBlock 自行调 parseSSEBlock/分发）。
 *  用户取消（isAborted）静默返回；连接中断回调 onStreamError（缺省文案「连接中断，请重试」）。 */
export async function consumeSSEStreamBlocks(
  body: ReadableStream<Uint8Array>,
  onBlock: (block: string) => void,
  isAborted: () => boolean,
  onStreamError?: (msg: string) => void,
): Promise<void> {
  const reader = body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  try {
    for (;;) {
      const { done, value } = await reader.read()
      if (done) break
      buffer += decoder.decode(value, { stream: true })
      buffer = buffer.replace(/\r\n?/g, '\n')
      let sep: number
      while ((sep = buffer.indexOf('\n\n')) !== -1) {
        const block = buffer.slice(0, sep)
        buffer = buffer.slice(sep + 2)
        onBlock(block)
      }
    }
    if (buffer.trim()) onBlock(buffer)
  } catch {
    if (isAborted()) return // 用户主动取消
    onStreamError?.('连接中断，请重试')
  }
}

/** 增量消费 ReadableStream，按 \n\n 切分事件块（book/end/error 事件流，见 dispatchSSEBlock） */
export async function consumeSSEStream(
  body: ReadableStream<Uint8Array>,
  cbs: SSEStreamCallbacks,
  isAborted: () => boolean,
): Promise<void> {
  await consumeSSEStreamBlocks(body, (block) => dispatchSSEBlock(block, cbs), isAborted, (msg) =>
    cbs.onStreamError?.(msg),
  )
}

/**
 * 发起 POST SSE 流式请求（原生 fetch，不走 axios——SSE 无 axios 拦截器）。
 * - accessToken 手动附加 query
 * - 传输层失败（网络错误 / 非 200 / 非 event-stream 响应）reject，调用方可降级普通接口
 * - resolve 返回 { abort } 句柄；流事件经 cbs 回调消费（后台消费，不 await 流结束）
 */
export function openSSEPost(
  path: string,
  body: Record<string, unknown>,
  cbs: SSEStreamCallbacks,
  accessToken: string | null,
): Promise<{ abort: () => void }> {
  const controller = new AbortController()
  const query = accessToken ? `?accessToken=${encodeURIComponent(accessToken)}` : ''
  return fetch(`${path}${query}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream' },
    body: JSON.stringify(body),
    signal: controller.signal,
  }).then(async (response) => {
    if (!response.ok) throw new Error(`服务异常（HTTP ${response.status}）`)
    const contentType = response.headers.get('content-type') ?? ''
    if (contentType && !contentType.includes('text/event-stream')) {
      throw new Error('当前服务不支持流式输出')
    }
    if (!response.body) throw new Error('当前服务不支持流式输出')
    let aborted = false
    void consumeSSEStream(response.body, cbs, () => aborted)
    return {
      abort: () => {
        aborted = true
        controller.abort()
      },
    }
  })
}
