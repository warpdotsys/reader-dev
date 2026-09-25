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

/** Java/Kotlin GET /reader3/getChapterList：必须传书籍 bookUrl，不是 tocUrl。 */
export function getBookToc(
  bookUrl: string,
  bookSource: string,
  opts?: { timeout?: number },
): Promise<ReturnData<BookChapter[]>> {
  return get<BookChapter[]>('/getChapterList', { url: bookUrl, bookSource }, opts)
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
 * POST /reader3/searchBookSourceSSE：流式换源（后端逐书源无名 data → event: end，legacy 对齐；
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

/**
 * GET /reader3/getBookContent：旧后端仅在 chapterUrl 为空且 index >= 0 时
 * 从目录中解析 BookChapter；直接传 chapterUrl 无法获取正文。
 * epubContent=1 且为 EPUB 本地书 → 返回 HTML 结构化正文（legacy 参数对齐；缺省/0 = 纯文本不变）
 */
export async function getBookContent(
  bookUrl: string,
  chapterUrl: string,
  bookSource: string,
  opts?: { timeout?: number; index?: number; cache?: boolean },
  epubContent?: number,
): Promise<ReturnData<BookContent>> {
  let index = opts?.index
  if (index === undefined || index < 0) {
    const toc = await getBookToc(bookUrl, bookSource, opts)
    index = toc.data.findIndex((chapter) => chapter.url === chapterUrl)
  }
  if (index < 0) throw new Error(`目录中未找到章节：${chapterUrl}`)
  const response = await get<BookContent | string>(
    '/getBookContent',
    { url: bookUrl, index, bookSource, ...(opts?.cache ? { cache: 1 } : {}), ...(epubContent === 1 ? { epubContent } : {}) },
    opts,
  )
  return {
    ...response,
    // The Java/Kotlin endpoint returns a plain string for ordinary chapters;
    // the Rust UI expected { content }. EPUB HTML mode already returns a map.
    data: typeof response.data === 'string' ? { content: response.data } : response.data,
  }
}
