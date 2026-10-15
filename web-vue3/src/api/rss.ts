import { get, post, type RequestOptions } from './request'
import type { RssArticle, RssSource, ReturnData } from '@/types'

/** GET /reader3/getRssSources：当前用户 RSS 订阅源列表 */
export function getRssSources(): Promise<ReturnData<RssSource[]>> {
  return get<RssSource[]>('/getRssSources')
}

/** POST /reader3/saveRssSource：新增/更新订阅源（body = 完整订阅源 JSON） */
export function saveRssSource(source: RssSource): Promise<ReturnData<null>> {
  return post<null>('/saveRssSource', source)
}

/** POST /reader3/deleteRssSource：legacy 按完整 RssSource 实体反序列化后以 sourceUrl 匹配。 */
export function deleteRssSource(source: RssSource): Promise<ReturnData<string>> {
  return post<string>('/deleteRssSource', source)
}

/**
 * GET /reader3/getRssArticles：订阅源文章列表（params sourceUrl + sortName + sortUrl + page）。
 * sortUrl 为 legacy sortUrl 多段 `名称::地址` 中的分类 URL——传该段后后端抓对应分类 feed。
 * 后端每次调用会重新抓取 feed——「刷新全部」即逐源循环调此接口（silent 模式不弹全局提示）。
 */
export function getRssArticles(
  sourceUrl: string,
  page = 1,
  sortUrl?: string,
  sortName?: string,
  opts?: RequestOptions,
): Promise<ReturnData<RssArticle[]>> {
  return get<RssArticle[]>('/getRssArticles', { sourceUrl, sortName, sortUrl, page }, opts)
}

/** GET /reader3/getRssContent：正文直接作为 data 字符串返回。 */
export function getRssContent(
  sourceUrl: string,
  link: string,
  origin: string,
): Promise<ReturnData<string>> {
  return get<string>('/getRssContent', { sourceUrl, link, origin })
}
