import { get, post } from './request'
import type { BookSource, ReturnData } from '@/types'

/** GET /reader3/getBookSources：当前用户书源列表 */
export function getBookSources(): Promise<ReturnData<BookSource[]>> {
  return get<BookSource[]>('/getBookSources')
}

/** POST /reader3/saveBookSource：保存单个书源（body = 完整书源 JSON） */
export function saveBookSource(source: BookSource): Promise<ReturnData<null>> {
  return post<null>('/saveBookSource', source)
}

/** POST /reader3/saveBookSources：批量保存（body = 书源数组） */
export function saveBookSources(sources: BookSource[]): Promise<ReturnData<{ count: number }>> {
  return post<{ count: number }>('/saveBookSources', sources)
}

/** POST /reader3/previewRemoteBookSources：只读预览；确认后另行 saveBookSources。 */
export function previewRemoteSource(
  url: string,
): Promise<ReturnData<{ sources: BookSource[]; existing: string[] }>> {
  return post<{ sources: BookSource[]; existing: string[] }>(
    '/previewRemoteBookSources',
    { url },
  )
}

/** POST /reader3/deleteBookSource：删除单个书源（body bookSourceUrl） */
export function deleteBookSource(bookSourceUrl: string): Promise<ReturnData<null>> {
  return post<null>('/deleteBookSource', { bookSourceUrl })
}

/**
 * POST /reader3/deleteBookSources：旧后端要求对象数组，而非 URL 字符串数组。
 */
export function deleteBookSources(
  urls: string[],
  opts?: { silent?: boolean },
): Promise<ReturnData<{ deleted?: number } | null>> {
  return post<{ deleted?: number } | null>(
    '/deleteBookSources', urls.map((bookSourceUrl) => ({ bookSourceUrl })), opts,
  )
}

/**
 * GET /reader3/getInvalidBookSources：检测失效书源，返回失效书源列表。
 * 后端返回含 bookSourceUrl / bookSourceName / error 的对象数组（兼容旧版 string[]）。
 * 后端 96 并发 + 8s/源，6900+ 书源可能耗时 10+ 分钟——必须放宽 axios 默认 15s 超时，
 * 否则检测必然以 timeout of 15000ms exceeded 失败。
 */
export interface InvalidBookSource {
  bookSourceUrl: string
  bookSourceName?: string
  error?: string
}

export function getInvalidBookSources(): Promise<ReturnData<Array<string | InvalidBookSource>>> {
  return get<Array<string | InvalidBookSource>>('/getInvalidBookSources', undefined, { silent: true, timeout: 900000 })
}

/**
 * POST /reader3/setAsDefaultBookSources：设置默认书源（body { bookSources: string[] }）。
 * 后端并行实现中（可能 404）：调用方传 { silent: true } 自行降级提示。
 */
export function setAsDefaultBookSources(bookSources: string[]): Promise<ReturnData<null>> {
  return post<null>('/setAsDefaultBookSources', { bookSources }, { silent: true })
}
