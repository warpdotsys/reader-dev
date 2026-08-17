import { get, post } from './request'
import type { Bookmark, ReturnData } from '@/types'

/**
 * 书签（/reader3/bookmarks 系列）：
 * - GET  /reader3/getBookmarks?bookUrl=…  → ReturnData<Bookmark[]>
 * - POST /reader3/deleteBookmark          body {bookUrl, title}
 * - POST /reader3/saveBookmark            body = Bookmark
 * - POST /reader3/saveBookmarks           body = Bookmark[]（批量导入）
 * - POST /reader3/deleteBookmarks         body {bookUrl, ids: title[]}
 */

/** GET /reader3/getBookmarks：单书书签列表（bookUrl 参数） */
export function getBookmarks(bookUrl: string, opts?: { silent?: boolean }): Promise<ReturnData<Bookmark[]>> {
  return get<Bookmark[]>('/getBookmarks', { bookUrl }, opts)
}

/** POST /reader3/saveBookmark：新增 / 编辑书签（body = Bookmark；createdAt 为空时后端补齐） */
export function saveBookmark(bookmark: Bookmark): Promise<ReturnData<null>> {
  return post<null>('/saveBookmark', bookmark)
}

/** POST /reader3/saveBookmarks：批量保存书签（JSON 导入；body = Bookmark[]） */
export function saveBookmarks(bookmarks: Bookmark[]): Promise<ReturnData<{ count: number }>> {
  return post<{ count: number }>('/saveBookmarks', bookmarks)
}

/** POST /reader3/deleteBookmark：删除书签（body：bookUrl + title） */
export function deleteBookmark(bookUrl: string, title: string): Promise<ReturnData<null>> {
  return post<null>('/deleteBookmark', { bookUrl, title })
}

/** POST /reader3/deleteBookmarks：批量删除书签（body：{bookUrl, ids}；ids 为书签标题） */
export function deleteBookmarks(bookUrl: string, titles: string[]): Promise<ReturnData<{ count: number }>> {
  return post<{ count: number }>('/deleteBookmarks', { bookUrl, ids: titles })
}

/** 从任意 JSON 文本解析书签数组（对象/数组兼容；bookUrl 缺失时用给定书 URL 兜底） */
export function parseBookmarksJson(raw: string, fallbackBookUrl: string): Bookmark[] {
  const parsed = JSON.parse(raw) as unknown
  const arr = Array.isArray(parsed) ? parsed : [parsed]
  const out: Bookmark[] = []
  for (const item of arr) {
    if (!item || typeof item !== 'object') continue
    const b = item as Record<string, unknown>
    const title = String(b.title ?? b.bookTitle ?? '').trim()
    if (!title) continue
    const chapterIndex =
      typeof b.chapterIndex === 'number'
        ? b.chapterIndex
        : typeof b.chapterPos === 'number'
          ? b.chapterPos
          : 0
    const paragraphIndex =
      typeof b.paragraphIndex === 'number'
        ? b.paragraphIndex
        : typeof b.chapterPos === 'number'
          ? b.chapterPos
          : 0
    out.push({
      bookUrl: String(b.bookUrl ?? b.bookName ?? fallbackBookUrl),
      title,
      bookName: String(b.bookName ?? ''),
      bookAuthor: String(b.bookAuthor ?? ''),
      chapterName: String(b.chapterName ?? ''),
      bookText: String(b.bookText ?? ''),
      content: String(b.content ?? ''),
      paragraphIndex,
      chapterIndex,
      createdAt:
        typeof b.createdAt === 'number'
          ? b.createdAt
          : typeof b.time === 'number'
            ? b.time
            : Date.now(),
    })
  }
  return out
}
