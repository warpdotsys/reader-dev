import { get, post } from './request'
import type { CacheClearResult, CacheClearType, CacheInfo, ContentSearchHit, ReturnData } from '@/types'

/**
 * 缓存管理 + 全书内容搜索 —— 后端契约
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/getCacheInfo      → ReturnData<CacheInfo>
 *                                   （缓存统计：tocCacheCount 目录缓存数 / tocCacheSize 目录缓存大小 /
 *                                     chapterCount 章节缓存数 / chapterSize 章节缓存大小 / totalSize 总大小(字节)）
 * POST /reader3/clearCache        body: { type: 'toc' | 'chapters' | 'all' } → ReturnData<{ deletedToc, deletedChapters }>
 *                                   （清理目录缓存 / 章节缓存 / 全部）
 * GET  /reader3/searchBookContent → params { key, bookUrl } → ReturnData<ContentSearchHit[]>
 *                                   hit: { chapterIndex, title, snippet }
 *                                   （全书内容搜索，本地书正文逐章匹配；书源书返回「仅支持本地书内容搜索」）
 * ================================================================
 *
 * 说明：接口以 silent 模式调用（后端未实现/不可用时返回 404，静默失败由调用方降级展示，
 * 不弹全局错误提示）；后端实现后无需改调用方即可自动生效。
 */

/** GET /reader3/getCacheInfo（silent 探测；失败时调用方显示「后端待实现」） */
export function getCacheInfo(): Promise<ReturnData<CacheInfo>> {
  return get<CacheInfo>('/getCacheInfo', undefined, { silent: true })
}

/** POST /reader3/clearCache（body { type }；失败时调用方提示「后端待实现」） */
export function clearCache(type: CacheClearType): Promise<ReturnData<CacheClearResult>> {
  return post<CacheClearResult>('/clearCache', { type }, { silent: true })
}

/**
 * GAP 79：GET/POST /reader3/deleteBookCache：删除单书缓存（body { bookUrl }；
 * 书需在本人书架——后端校验归属；返回 { deleted } 删除行数）。
 * 后端未实现（404）时 silent 降级——调用方提示。
 */
export function deleteBookCache(bookUrl: string): Promise<ReturnData<{ deleted?: number } | null>> {
  return post<{ deleted?: number } | null>('/deleteBookCache', { bookUrl }, { silent: true })
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
