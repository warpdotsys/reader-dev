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

/** POST /reader3/deleteRssSource：删除订阅源（body { rssSourceUrl }） */
export function deleteRssSource(rssSourceUrl: string): Promise<ReturnData<null>> {
  return post<null>('/deleteRssSource', { rssSourceUrl })
}

/**
 * GET /reader3/getRssArticles：订阅源文章列表（params rssSourceUrl + page，分页；hasRead 已读标记）。
 * sortUrl 为 legacy sortUrl 多段 `名称::地址` 中的分类 URL——传该段后后端抓对应分类 feed。
 * 后端每次调用会重新抓取 feed——「刷新全部」即逐源循环调此接口（silent 模式不弹全局提示）。
 */
export function getRssArticles(
  rssSourceUrl: string,
  page = 1,
  sortUrl?: string,
  opts?: RequestOptions,
): Promise<ReturnData<RssArticle[]>> {
  return get<RssArticle[]>('/getRssArticles', { rssSourceUrl, page, sortUrl }, opts)
}

/** POST /reader3/markRssArticleRead：标记文章已读/未读（body { articleUrl, read }） */
export function markRssArticleRead(articleUrl: string, read: boolean): Promise<ReturnData<null>> {
  return post<null>('/markRssArticleRead', { articleUrl, read })
}

/** GET /reader3/getRssArticle：文章正文（data: { content }，content 为 HTML） */
export function getRssArticle(url: string): Promise<ReturnData<{ content: string }>> {
  return get<{ content: string }>('/getRssArticle', { url })
}
