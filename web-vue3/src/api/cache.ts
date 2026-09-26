import { get, post } from './request'
import type { ContentSearchHit, ReturnData } from '@/types'

/**
 * 单书缓存 + 全书内容搜索 —— 已验证的后端契约
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/getShelfBookWithCacheInfo → ReturnData<Book + cacheChapterCount + cacheSize>
 * POST /reader3/deleteBookCache body: { bookUrl } → ReturnData<"">
 * GET  /reader3/searchBookContent → params { key, bookUrl } → ReturnData<ContentSearchHit[]>
 *                                   hit: { chapterIndex, title, snippet }
 *                                   （全书内容搜索，本地书正文逐章匹配；书源书返回「仅支持本地书内容搜索」）
 * ================================================================
 *
 * 不存在全局 getCacheInfo/clearCache 路由；设置页必须明确提示该限制，不能探测后假定可用。
 */

/**
 * POST /reader3/deleteBookCache：删除单书缓存（body { bookUrl }；
 * 书需在本人书架——后端校验归属；legacy 对齐：成功 data=""）。
 * 后端未实现（404）时 silent 降级——调用方提示。
 */
export function deleteBookCache(bookUrl: string): Promise<ReturnData<string | null>> {
  return post<string | null>('/deleteBookCache', { bookUrl }, { silent: true })
}

/** GET /reader3/searchBookContent（params key + bookUrl → 章节命中列表；失败由调用方在搜索弹层内提示） */
export function searchBookContent(key: string, bookUrl: string): Promise<ReturnData<ContentSearchHit[]>> {
  return get<ContentSearchHit[]>('/searchBookContent', { key, bookUrl }, { silent: true })
}

/**
 * GAP 82：GET /reader3/getShelfBookWithCacheInfo：书架单书 + 缓存信息（后端已有）。
 * 返回书架书全字段 + cacheChapterCount（已缓存章数）+ cacheSize（缓存正文大小，字节）。
 * 以 silent 调用：接口未实现/书不在书架时由调用方降级（隐藏状态区）。
 */
export interface ShelfBookCacheInfo {
  bookUrl?: string
  name?: string
  /** 已缓存章节数（后端 book_cache_info） */
  cacheChapterCount?: number
  /** 缓存正文大小（字节） */
  cacheSize?: number
  [key: string]: unknown
}

export function getShelfBookWithCacheInfo(url: string): Promise<ReturnData<ShelfBookCacheInfo>> {
  return get<ShelfBookCacheInfo>('/getShelfBookWithCacheInfo', { url }, { silent: true })
}
