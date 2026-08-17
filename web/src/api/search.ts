import { post } from './request'
import { useUserStore } from '@/stores/user'
import { openSSEPost } from './sse'
import type { ReturnData, SearchBook } from '@/types'

/**
 * POST /reader3/searchBookMulti：多书源并发搜索（body {key, maxSources, page, exact}；signal 可中止请求）
 * page 从 1 开始——普通（非 SSE）搜索分页场景（GAP 100：批量模式「加载更多」逐页累加）。
 * exact=true 时后端按书名/作者等值过滤（大小写/全半角忽略）。
 */
export function searchBookMulti(
  key: string,
  maxSources = 50,
  signal?: AbortSignal,
  page = 1,
  exact = false,
  bookSourceGroup = '',
): Promise<ReturnData<SearchBook[]>> {
  return post<SearchBook[]>(
    '/searchBookMulti',
    { key, maxSources, page, exact: exact ? 1 : 0, bookSourceGroup },
    { signal },
  )
}

/* ================= SSE 流式搜索（/reader3/searchBookMultiSSE） ================= */

export interface SearchSSEParams {
  key: string
  /** 书源分组过滤（空串 = 全部） */
  bookSourceGroup?: string
  /** 起始索引（-1 = 从头搜索） */
  lastIndex?: number
  /** 本次搜索覆盖的书源数量 */
  searchSize?: number
  /** 并发数 */
  concurrentCount?: number
  /** 精确匹配（exact=1：书名/作者等值，忽略大小写/全半角；缺省模糊 contains） */
  exact?: boolean
}

export interface SearchSSECallbacks {
  /** 单个书源结果到达（data 可能为空数组） */
  onBooks: (lastIndex: number, books: SearchBook[]) => void
  /** 流正常结束（event: end） */
  onEnd: (lastIndex: number, isEnd: boolean) => void
  /** 服务端业务错误（event: error，data 为 ReturnData） */
  onErrorEvent: (ret: ReturnData) => void
  /** 流中途中断（连接断开，非用户取消） */
  onStreamError?: (msg: string) => void
}

export interface SearchSSEHandle {
  abort: () => void
}

/**
 * POST /reader3/searchBookMultiSSE：多书源流式搜索（原生 fetch，不走 axios）
 * - accessToken 手动附加 query（SSE 无 axios 拦截器）
 * - SSE 解析/分发/消费逻辑见 api/sse.ts（searchBookSourceSSE 等共用）
 * - 传输层失败（网络错误 / 非 200 / 非 event-stream 响应）reject，调用方可降级 searchBookMulti
 */
export function searchBookMultiSSE(
  params: SearchSSEParams,
  cbs: SearchSSECallbacks,
): Promise<SearchSSEHandle> {
  const token = useUserStore().accessToken
  const body: Record<string, unknown> = { key: params.key }
  if (params.bookSourceGroup !== undefined) body.bookSourceGroup = params.bookSourceGroup
  if (params.lastIndex !== undefined) body.lastIndex = params.lastIndex
  if (params.searchSize !== undefined) body.searchSize = params.searchSize
  if (params.concurrentCount !== undefined) body.concurrentCount = params.concurrentCount
  if (params.exact !== undefined) body.exact = params.exact ? 1 : 0

  return openSSEPost('/reader3/searchBookMultiSSE', body, cbs, token)
}
