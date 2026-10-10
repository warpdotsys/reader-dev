import { TOKEN_AUTH_CACHE_SCOPE } from './tokenAuthentication.ts'

/**
 * 本机章节缓存（IndexedDB）——阅读页离线/快速回读用。
 *
 * 读取顺序：本机缓存 → 服务器缓存/书源（getBookContent 已自动写服务器缓存）。
 * 批量「拉取到本机」由 ChapterCacheDialog 调用 getBookContent 逐章写入本机，
 * 服务器没有缓存时会自动先经书源补齐到服务器，再回写本机。
 */

export interface LocalCachedChapter {
  key: string
  bookUrl: string
  chapterUrl: string
  title: string
  /** 目录实章序号（0 基，过滤卷标题后） */
  index: number
  content: string
  updatedAt: number
}

/** 旧共享数据库的正文无法确定归属：不读取、不迁移、不删除。 */
const DB_PREFIX = 'reader-local-cache-v2:'
const DB_VERSION = 1
const STORE = 'chapters'
const KEY_PREFIX = 'ch:'

function cacheKey(bookUrl: string, chapterUrl: string): string {
  return `${KEY_PREFIX}${bookUrl}\u0000${chapterUrl}`
}

export interface LocalCacheUser {
  accessToken: string
  username: string
  isAdmin: boolean
  defaultConfigMode: boolean
}

/** 身份与配置命名空间均隔离；凭据仅用于判断登录态，绝不写入缓存名称。 */
export function localChapterCacheScope(user: LocalCacheUser, deployment: string): string | null {
  if (!user.accessToken || !user.username || !deployment) return null
  const namespace = user.isAdmin && user.defaultConfigMode ? 'default' : user.username
  // 旧 default 缓存曾使用错误的 ns 参数，不能认定来自真实系统空间。
  return JSON.stringify(namespace === 'default'
    ? [deployment, user.username, namespace, 'legacy-userNS-v1', TOKEN_AUTH_CACHE_SCOPE]
    : [deployment, user.username, namespace, TOKEN_AUTH_CACHE_SCOPE])
}

const dbPromises = new Map<string, Promise<IDBDatabase>>()

function openDb(scope: string): Promise<IDBDatabase> {
  const existing = dbPromises.get(scope)
  if (existing) return existing
  const pending = new Promise<IDBDatabase>((resolve, reject) => {
    if (typeof indexedDB === 'undefined') {
      reject(new Error('IndexedDB 不可用'))
      return
    }
    const req = indexedDB.open(DB_PREFIX + scope, DB_VERSION)
    req.onupgradeneeded = () => {
      const db = req.result
      if (!db.objectStoreNames.contains(STORE)) {
        const store = db.createObjectStore(STORE, { keyPath: 'key' })
        store.createIndex('bookUrl', 'bookUrl', { unique: false })
      }
    }
    req.onsuccess = () => {
      const db = req.result
      db.onversionchange = () => { db.close(); dbPromises.delete(scope) }
      resolve(db)
    }
    req.onerror = () => reject(req.error ?? new Error('IndexedDB 打开失败'))
  })
  dbPromises.set(scope, pending)
  void pending.catch(() => { if (dbPromises.get(scope) === pending) dbPromises.delete(scope) })
  return pending
}

/** 读取单章本机缓存；不可用时返回 null（调用方继续走服务器/书源） */
export async function getLocalChapter(
  scope: string | null,
  bookUrl: string,
  chapterUrl: string,
): Promise<LocalCachedChapter | null> {
  if (!scope) return null
  try {
    const db = await openDb(scope)
    const key = cacheKey(bookUrl, chapterUrl)
    return await new Promise<LocalCachedChapter | null>((resolve, reject) => {
      const tx = db.transaction(STORE, 'readonly')
      const req = tx.objectStore(STORE).get(key)
      req.onsuccess = () => resolve((req.result as LocalCachedChapter | undefined) ?? null)
      req.onerror = () => reject(req.error)
    })
  } catch {
    return null
  }
}

/** 写入单章本机缓存；返回是否成功（失败静默，不影响阅读） */
export async function saveLocalChapter(scope: string | null, input: {
  bookUrl: string
  chapterUrl: string
  title: string
  index: number
  content: string
}): Promise<boolean> {
  if (!scope) return false
  try {
    const db = await openDb(scope)
    const rec: LocalCachedChapter = {
      key: cacheKey(input.bookUrl, input.chapterUrl),
      bookUrl: input.bookUrl,
      chapterUrl: input.chapterUrl,
      title: input.title,
      index: input.index,
      content: input.content,
      updatedAt: Date.now(),
    }
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE, 'readwrite')
      tx.objectStore(STORE).put(rec)
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
      tx.onabort = () => reject(tx.error ?? new Error('缓存写入已取消'))
    })
    return true
  } catch {
    return false
  }
}

/** 批量写入本机缓存（逐条 put，单事务）；返回成功条数 */
export async function saveLocalChapters(
  scope: string | null,
  items: Array<{
    bookUrl: string
    chapterUrl: string
    title: string
    index: number
    content: string
  }>,
): Promise<number> {
  if (!scope || items.length === 0) return 0
  try {
    const db = await openDb(scope)
    const now = Date.now()
    const recs: LocalCachedChapter[] = items.map((item) => ({
      key: cacheKey(item.bookUrl, item.chapterUrl),
      bookUrl: item.bookUrl,
      chapterUrl: item.chapterUrl,
      title: item.title,
      index: item.index,
      content: item.content,
      updatedAt: now,
    }))
    let saved = 0
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE, 'readwrite')
      const store = tx.objectStore(STORE)
      for (const rec of recs) {
        store.put(rec)
        saved++
      }
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
      tx.onabort = () => reject(tx.error ?? new Error('缓存写入已取消'))
    })
    return saved
  } catch {
    return 0
  }
}

/** 清空某书本机缓存；返回删除条数 */
export async function clearLocalBook(scope: string | null, bookUrl: string): Promise<number> {
  if (!scope) return 0
  try {
    const db = await openDb(scope)
    return await new Promise<number>((resolve, reject) => {
      const tx = db.transaction(STORE, 'readwrite')
      const store = tx.objectStore(STORE)
      const req = store.index('bookUrl').openKeyCursor(IDBKeyRange.only(bookUrl))
      let deleted = 0
      req.onsuccess = () => {
        const cursor = req.result
        if (cursor) {
          store.delete(cursor.primaryKey)
          deleted++
          cursor.continue()
        }
      }
      req.onerror = () => reject(req.error)
      tx.oncomplete = () => resolve(deleted)
      tx.onerror = () => reject(tx.error)
      tx.onabort = () => reject(tx.error ?? new Error('缓存清除已取消'))
    })
  } catch {
    return 0
  }
}

/** 列出某书本机已缓存章节 URL（目录缓存标记用；失败返回空数组） */
export async function listLocalChapterUrls(scope: string | null, bookUrl: string): Promise<string[]> {
  if (!scope) return []
  try {
    const db = await openDb(scope)
    return await new Promise<string[]>((resolve, reject) => {
      const tx = db.transaction(STORE, 'readonly')
      const store = tx.objectStore(STORE)
      const req = store.index('bookUrl').openCursor(IDBKeyRange.only(bookUrl))
      const urls: string[] = []
      req.onsuccess = () => {
        const cursor = req.result
        if (cursor) {
          urls.push((cursor.value as LocalCachedChapter).chapterUrl)
          cursor.continue()
        }
      }
      req.onerror = () => reject(req.error)
      tx.oncomplete = () => resolve(urls)
      tx.onerror = () => reject(tx.error)
    })
  } catch {
    return []
  }
}
