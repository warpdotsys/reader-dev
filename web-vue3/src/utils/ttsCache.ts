/** 听书音频仅按已认证的部署/账号/命名空间读取。旧共享 v1 不读、不迁移、不删。 */
const CACHE_PREFIX = 'tts-audio-v2:'
const KEY_PREFIX = '/tts/v2/'
const DIGEST_PATTERN = '[0-9a-f]{64}'

export interface TtsCacheParams {
  engine: string
  voice: string
  rate: string
  pitch: string
  volume?: string
  style?: string
}

/** 缓存名称和 URL 都只含摘要；正文、访问令牌不进入持久化键。 */
export interface TtsCacheKey {
  readonly cacheName: string
  readonly url: string
}

async function digest(value: string): Promise<string | null> {
  if (!globalThis.crypto?.subtle) return null
  try {
    const result = await globalThis.crypto.subtle.digest('SHA-256', new TextEncoder().encode(value))
    return Array.from(new Uint8Array(result), (byte) => byte.toString(16).padStart(2, '0')).join('')
  } catch {
    // 摘要不可用时不缓存，不退回截断正文或共享/弱哈希键。
    return null
  }
}

async function cacheName(scope: string | null): Promise<string | null> {
  if (!scope) return null
  const value = await digest(scope)
  return value ? CACHE_PREFIX + value : null
}

/** 全文摘要 + 无歧义参数编码；同长度不同尾部也必须重新合成。 */
export async function ttsCacheKey(scope: string | null, text: string, p: TtsCacheParams): Promise<TtsCacheKey | null> {
  const name = await cacheName(scope)
  if (!name) return null
  const value = await digest(JSON.stringify([p.engine, p.voice, p.rate, p.pitch, p.volume ?? '', p.style ?? '', text]))
  return value ? Object.freeze({ cacheName: name, url: KEY_PREFIX + value + '.mp3' }) : null
}

function validKey(key: TtsCacheKey | null): key is TtsCacheKey {
  return !!key && new RegExp('^' + CACHE_PREFIX + DIGEST_PATTERN + '$').test(key.cacheName)
    && new RegExp('^' + KEY_PREFIX + DIGEST_PATTERN + '\\.mp3$').test(key.url)
}

/** 每次异步返回再核对短期会话；晚到缓存命中不交给新账号播放。 */
export async function getCachedTts(key: TtsCacheKey | null, isCurrent: () => boolean = () => true): Promise<Blob | null> {
  if (!validKey(key) || typeof caches === 'undefined' || !isCurrent()) return null
  try {
    const cache = await caches.open(key.cacheName)
    if (!isCurrent()) return null
    const hit = await cache.match(key.url)
    if (!hit || !isCurrent()) return null
    const blob = await hit.blob()
    return isCurrent() ? blob : null
  } catch {
    return null
  }
}

/** 失败不影响播放。已发出的 Cache API put 不能取消，但只会写捕获的旧作用域。 */
export async function putCachedTts(key: TtsCacheKey | null, blob: Blob, isCurrent: () => boolean = () => true): Promise<void> {
  if (!validKey(key) || typeof caches === 'undefined' || !isCurrent()) return
  try {
    const cache = await caches.open(key.cacheName)
    if (!isCurrent()) return
    await cache.put(key.url, new Response(blob, { headers: { 'Content-Type': blob.type || 'audio/mpeg' } }))
  } catch {
    // 配额/隐私模式等失败只放弃音频缓存。
  }
}

/** 只统计当前作用域；不探测旧共享缓存或其他账号。 */
export async function ttsCacheStats(scope: string | null, isCurrent: () => boolean = () => true): Promise<{ count: number; bytes: number }> {
  const empty = { count: 0, bytes: 0 }
  if (typeof caches === 'undefined' || !isCurrent()) return empty
  const name = await cacheName(scope)
  if (!name || !isCurrent()) return empty
  try {
    const cache = await caches.open(name)
    if (!isCurrent()) return empty
    const keys = await cache.keys()
    let bytes = 0
    for (const req of keys) {
      if (!isCurrent()) return empty
      const response = await cache.match(req)
      if (response) bytes += (await response.blob()).size
    }
    return isCurrent() ? { count: keys.length, bytes } : empty
  } catch {
    return empty
  }
}

/** 仅删除明确作用域；API 不可用或身份晚到失效不谎报清理成功。 */
export async function clearTtsCache(scope: string | null, isCurrent: () => boolean = () => true): Promise<boolean> {
  if (typeof caches === 'undefined' || !isCurrent()) return false
  const name = await cacheName(scope)
  if (!name || !isCurrent()) return false
  try {
    // 不存在的当前缓存本就为空；成功返回后仍须核对显示提示所属会话。
    await caches.delete(name)
    return isCurrent()
  } catch {
    return false
  }
}
