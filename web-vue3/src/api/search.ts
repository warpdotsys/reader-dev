import { post } from './request'
import { useUserStore } from '@/stores/user'
import { openSSEPost } from './sse'
import type { ReturnData, SearchBook } from '@/types'

export interface SearchPage {
  lastIndex: number
  list: SearchBook[]
  /** 单源精确筛选前的数量，用来判断书源页是否已经耗尽。 */
  rawCount?: number
}

export interface SearchRequest {
  key: string
  lastIndex: number
  searchSize?: number
  bookSourceGroup?: string
  sourceUrl?: string
  page?: number
  exact?: boolean
}

/** legacy 多源搜索用 lastIndex 游标；单源搜索改用 /searchBook 的页码。 */
export async function searchBookMulti(
  params: SearchRequest,
  signal?: AbortSignal,
): Promise<ReturnData<SearchPage>> {
  if (params.sourceUrl) {
    const result = await post<SearchBook[]>('/searchBook', {
      key: params.key,
      page: params.page ?? 1,
      bookSourceUrl: params.sourceUrl,
    }, { signal })
    const normalizedKey = params.key.normalize('NFKC').toLocaleLowerCase()
    const list = params.exact
      ? result.data.filter((book) =>
          book.name?.normalize('NFKC').toLocaleLowerCase() === normalizedKey ||
          book.author?.normalize('NFKC').toLocaleLowerCase() === normalizedKey)
      : result.data
    return { ...result, data: { lastIndex: params.page ?? 1, list, rawCount: result.data.length } }
  }
  return post<SearchPage>('/searchBookMulti', {
    key: params.exact ? `=${params.key}` : params.key,
    lastIndex: params.lastIndex,
    searchSize: params.searchSize ?? 50,
    bookSourceGroup: params.bookSourceGroup ?? '',
  }, { signal })
}

/* ================= SSE 流式搜索（/reader3/searchBookMultiSSE） ================= */

export interface SearchSSEParams {
  key: string
  /** 书源分组过滤（空串 = 全部） */
  bookSourceGroup?: string
  /** P1-4 单源指定：精确 bookSourceUrl（非空时后端只搜该源） */
  bookSourceUrl?: string
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
  if (params.exact) body.key = `=${params.key}`

  return openSSEPost('/reader3/searchBookMultiSSE', body, cbs, token)
}
