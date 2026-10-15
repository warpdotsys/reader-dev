import { get, post } from './request'
import type { BookSource, ExploreCategory, ReturnData, SearchBook } from '@/types'

/**
 * Java/Kotlin 的实际书源接口。旧 Vue 客户端也是取得完整书源后，在浏览器解析
 * exploreUrl；服务端从未注册 getExploreSources / getExploreUrls。
 */
export function getExploreSources(): Promise<ReturnData<BookSource[]>> {
  return get<BookSource[]>('/getBookSources')
}

function unsupportedExploreUrl(source: BookSource): Error {
  return new Error(`书源“${source.bookSourceName}”的探索分类使用动态 JavaScript，当前 Web 端不能安全执行；请在书源管理中改为 JSON 或“标题::地址”格式。`)
}

/**
 * 与旧版 Explore.vue 一致地在客户端读取静态 exploreUrl。
 *
 * 这里只接受 JSON 数组或换行的“标题::地址”。不使用 new Function，也不把
 * <js> / @js: 动态规则伪装成空分类；它们需要由后续的受控规则执行器实现。
 */
export function getExploreUrls(source: BookSource): ExploreCategory[] {
  const raw = source.exploreUrl?.trim()
  if (!raw) return []
  if (raw.startsWith('@js:') || raw.includes('<js>')) throw unsupportedExploreUrl(source)

  try {
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) {
      throw new Error('探索分类 JSON 必须是数组')
    }
    const categories = parsed.flatMap((entry): ExploreCategory[] => {
      if (!entry || typeof entry !== 'object') return []
      const value = entry as Record<string, unknown>
      const title = typeof value.title === 'string' ? value.title.trim() : ''
      const url = typeof value.url === 'string' ? value.url.trim() : ''
      return title && url ? [{ title, url, type: typeof value.type === 'string' ? value.type : undefined }] : []
    })
    if (categories.length === 0 && parsed.length > 0) {
      throw new Error('探索分类缺少 title 或 url')
    }
    return categories
  } catch (error) {
    // JSON 失败时才采用 JAR 内旧 Web 的纯文本格式；绝不执行书源提供的 JS。
    if (!(error instanceof SyntaxError)) throw error
    const categories = raw
      .replace(/\r\n/g, '\n')
      .split('\n')
      .map((line) => {
        const separator = line.indexOf('::')
        if (separator < 1) return null
        const title = line.slice(0, separator).trim()
        const url = line.slice(separator + 2).trim()
        return title && url ? { title, url } : null
      })
      .filter((entry): entry is ExploreCategory => entry !== null)
    if (categories.length === 0) {
      throw new Error(`书源“${source.bookSourceName}”的探索分类不是受支持的 JSON 或“标题::地址”格式。`)
    }
    return categories
  }
}

/**
 * Java/Kotlin 真实契约：POST body {ruleFindUrl, bookSourceUrl, page}。
 * 原先的 GET {url, bookSource} 会使服务端拿不到书源及 ruleFindUrl。
 */
export function exploreBook(
  ruleFindUrl: string,
  bookSourceUrl: string,
  page = 1,
): Promise<ReturnData<SearchBook[]>> {
  return post<SearchBook[]>('/exploreBook', { ruleFindUrl, bookSourceUrl, page })
}
