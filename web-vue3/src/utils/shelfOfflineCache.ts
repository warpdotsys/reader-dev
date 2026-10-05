import { isAxiosError } from 'axios'
import type { Book, BookGroup } from '../types'

/** 身份 scope 由 localChapterCacheScope 统一生成；不读取/迁移/删除旧共享键。 */
const PREFIX = 'reader_shelf_offline_v2:'
type ShelfStorage = Pick<Storage, 'getItem' | 'setItem'>

export interface ShelfOfflineCache {
  books: Book[]
  groups: BookGroup[]
  ts: number
}

function isShelf(value: unknown): value is ShelfOfflineCache {
  if (!value || typeof value !== 'object') return false
  const shelf = value as Partial<ShelfOfflineCache>
  return Number.isFinite(shelf.ts) && typeof shelf.ts === 'number' && shelf.ts >= 0 &&
    Array.isArray(shelf.books) && shelf.books.every(book => book &&
      ['bookUrl', 'name', 'author', 'tocUrl', 'origin', 'originName'].every(field => typeof book[field] === 'string')) &&
    Array.isArray(shelf.groups) && shelf.groups.every(group => group &&
      Number.isFinite(group.id) && typeof group.name === 'string')
}

/** 仅保存同一身份完整成功的书架/分组快照，不把失败伪装成已保存。 */
export function saveOfflineShelf(scope: string | null, shelf: ShelfOfflineCache, storage?: ShelfStorage): boolean {
  if (!scope || !isShelf(shelf)) return false
  try {
    (storage ?? globalThis.localStorage).setItem(PREFIX + scope, JSON.stringify({ version: 2, scope, ...shelf }))
    return true
  } catch {
    return false
  }
}

export function loadOfflineShelf(scope: string | null, storage?: ShelfStorage): ShelfOfflineCache | null {
  if (!scope) return null
  try {
    const raw: unknown = JSON.parse((storage ?? globalThis.localStorage).getItem(PREFIX + scope) ?? '')
    if (!isShelf(raw)) return null
    const envelope = raw as ShelfOfflineCache & { version?: unknown; scope?: unknown }
    if (envelope.version !== 2 || envelope.scope !== scope) return null
    return { books: raw.books, groups: raw.groups, ts: raw.ts }
  } catch {
    return null
  }
}

/** 认证、业务、过时会话、取消及配置错误都不能伪装成“服务端暂不可用”。 */
export function canUseOfflineShelf(error: unknown): boolean {
  if (!isAxiosError(error) || error.code === 'ERR_CANCELED') return false
  const data = error.response?.data as { data?: unknown; errorMsg?: unknown } | undefined
  if (data?.data === 'NEED_LOGIN' || data?.data === 'NEED_SECURE_KEY' ||
      (typeof data?.errorMsg === 'string' && data.errorMsg.includes('请登录'))) return false
  if (error.response) return [502, 503, 504].includes(error.response.status)
  return ['ERR_NETWORK', 'ECONNABORTED', 'ETIMEDOUT'].includes(error.code ?? '')
}
