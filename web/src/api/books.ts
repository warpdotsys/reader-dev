import { get } from './request'
import { useUserStore } from '@/stores/user'
import { openSSEPost } from './sse'
import type { BookChapter, BookContent, BookInfo, ReturnData, SearchBook } from '@/types'

/** GET /reader3/getBookInfo：书籍详情（参数 url + bookSource=book.origin） */
export function getBookInfo(url: string, bookSource: string, opts?: { silent?: boolean }): Promise<ReturnData<BookInfo>> {
  return get<BookInfo>('/getBookInfo', { url, bookSource }, opts)
}

/**
 * GET /reader3/searchBookSource：换源搜索——按 url（当前书 bookUrl）+ bookSource（当前源）
 * 搜索同书的其他书源，返回 SearchBook[]（每项含新源 origin/originName/tocUrl）。
 * 后端并行实现中（可能 404）：调用方传 { silent: true } 自行降级提示。
 */
export function searchBookSource(
  url: string,
  bookSource: string,
  opts?: { silent?: boolean },
): Promise<ReturnData<SearchBook[]>> {
  return get<SearchBook[]>('/searchBookSource', { url, bookSource }, opts)
}

/** GET /reader3/getBookToc：章节目录（tocUrl=info.tocUrl + bookSource） */
export function getBookToc(
  tocUrl: string,
  bookSource: string,
  opts?: { timeout?: number },
): Promise<ReturnData<BookChapter[]>> {
  return get<BookChapter[]>('/getBookToc', { tocUrl, bookSource }, opts)
}

/* ================= GAP 81：换源 SSE 流式（/reader3/searchBookSourceSSE） ================= */

export interface SourceSSECallbacks {
  /** 单个书源结果到达（data 可能为空数组；lastIndex 为该源序号） */
  onBooks: (lastIndex: number, books: SearchBook[]) => void
  /** 流正常结束（event: end） */
  onEnd: (lastIndex: number, isEnd: boolean) => void
  /** 服务端业务错误（event: error，data 为 ReturnData） */
  onErrorEvent: (ret: ReturnData) => void
  /** 流中途中断（连接断开，非用户取消） */
  onStreamError?: (msg: string) => void
}

/**
 * POST /reader3/searchBookSourceSSE：流式换源（后端逐书源 event: book → event: end；
 * 与普通 searchBookSource 同契约 SearchBook[]，增量推送）。
 * 传输层失败 reject → 调用方降级普通 searchBookSource。
 */
export function searchBookSourceSSE(
  url: string,
  bookSource: string,
  cbs: SourceSSECallbacks,
): Promise<{ abort: () => void }> {
  const token = useUserStore().accessToken
  return openSSEPost(
    '/reader3/searchBookSourceSSE',
    { url, bookSource },
    { onBooks: cbs.onBooks, onEnd: cbs.onEnd, onErrorEvent: cbs.onErrorEvent, onStreamError: cbs.onStreamError },
    token,
  )
}

/** GET /reader3/getBookContent：章节正文（chapterUrl + bookSource，正文在 data.content） */
export function getBookContent(
  chapterUrl: string,
  bookSource: string,
  opts?: { timeout?: number },
): Promise<ReturnData<BookContent>> {
  return get<BookContent>('/getBookContent', { chapterUrl, bookSource }, opts)
}
