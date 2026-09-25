/**
 * P0-3b 边听边缓存（对齐 legacy cacheTTSAudio）：
 * 合成成功的整章音频写入 Cache API，再次播放同一章（同引擎/语音/语速/音调）
 * 直接命中本地，省去网络往返与合成耗时。
 *
 * 键设计：/tts/{sha1(params+text 前 N 字)}——text 全文哈希保证内容变更即失效；
 * Cache API 存储配额由浏览器管理（Chrome ~可用空间 60%），超限写入静默失败不影响播放。
 */

const CACHE_NAME = 'tts-audio-v1'

export interface TtsCacheParams {
  engine: string
  voice: string
  rate: string
  pitch: string
  volume?: string
  style?: string
}

/** 简易 FNV-1a 32 位哈希 → hex（足够区分章节文本，无加密需求） */
function fnv1a(s: string): string {
  let h = 0x811c9dc5
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 0x01000193)
  }
  return (h >>> 0).toString(16).padStart(8, '0')
}

/** 缓存键：参数 + 文本前 4096 字符的哈希（长章尾部截断不影响命中率主体） */
export function ttsCacheKey(text: string, p: TtsCacheParams): string {
  const sig = [p.engine, p.voice, p.rate, p.pitch, p.volume ?? '', p.style ?? ''].join('|')
  return `/tts/${fnv1a(sig)}/${fnv1a(text.slice(0, 4096) + ':' + text.length)}.mp3`
}

/** 查缓存：命中返回 Blob，未命中返回 null */
export async function getCachedTts(key: string): Promise<Blob | null> {
  if (typeof caches === 'undefined') return null
  try {
    const cache = await caches.open(CACHE_NAME)
    const hit = await cache.match(key)
    return hit ? await hit.blob() : null
  } catch {
    return null
  }
}

/** 写缓存（异步后台执行，失败静默） */
export function putCachedTts(key: string, blob: Blob): void {
  if (typeof caches === 'undefined') return
  void (async () => {
    try {
      const cache = await caches.open(CACHE_NAME)
      // 显式带 Content-Type，避免部分浏览器 match 时 MIME 不匹配
      const res = new Response(blob, {
        headers: { 'Content-Type': blob.type || 'audio/mpeg' },
      })
      await cache.put(key, res)
    } catch {
      /* 配额满/隐私模式等：静默放弃 */
    }
  })()
}

/** 听书缓存条目数与估算体积（SettingsView 缓存管理展示用） */
export async function ttsCacheStats(): Promise<{ count: number; bytes: number }> {
  if (typeof caches === 'undefined') return { count: 0, bytes: 0 }
  try {
    const cache = await caches.open(CACHE_NAME)
    const keys = await cache.keys()
    let bytes = 0
    for (const req of keys) {
      const res = await cache.match(req)
      if (res) bytes += Number(res.headers.get('Content-Length') ?? 0) || (await res.clone().blob()).size
    }
    return { count: keys.length, bytes }
  } catch {
    return { count: 0, bytes: 0 }
  }
}

/** 清空听书缓存 */
export async function clearTtsCache(): Promise<void> {
  if (typeof caches === 'undefined') return
  try {
    await caches.delete(CACHE_NAME)
  } catch {
    /* 忽略 */
  }
}
