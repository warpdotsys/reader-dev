/**
 * 夜读 Reader · Service Worker（PWA 离线壳）——ES Module（main.ts 以 { type: 'module' } 注册）
 *
 * 缓存策略（v2，M5）：
 * - 核心壳（导航请求 / 页面 HTML）→ 网络优先 + 缓存回退（保证每次访问尽量拿到最新版本；
 *   离线时回退缓存壳，SPA 前端路由照常工作）
 * - 静态资源（图片 / 字体 / manifest 等 hash 文件名）→ 缓存优先（hash 文件名天然免疫陈旧内容），
 *   写入前按条目数上限裁剪（防无界膨胀）
 * - 动态 API（/reader3/*，含 file/download、probeSecureMode 的 _t 防缓存参数）→ 一律网络直连，
 *   不缓存、不拦截（M5：cache-first 会把瞬态错误/旧封面永久缓存）
 * - 图片代理 /assets/proxy → 网络优先；仅缓存合法 image/* 成功响应（回源失败返回的
 *   200 + JSON 错误体不缓存——避免一次瞬时上游故障 = 封面永久损坏）
 * - 跨域请求（书源封面 / 正文等）一律不缓存、不拦截
 *
 * 版本号：发版时递增 CACHE_VERSION 即可让旧缓存整体失效（activate 清理）。
 *
 * 纯函数部分（isApiPath/isProxyPath/isDynamicPath/isImageResponse 等）导出供 node 单测
 * （web-ui/src/utils/sw.test.ts）直接导入；浏览器中 self 分支注册 SW 逻辑。
 */

export const CACHE_VERSION = 'reader-shell-v3'
export const SHELL_CACHE = `${CACHE_VERSION}-shell`
export const STATIC_CACHE = `${CACHE_VERSION}-static`

/** 动态 API 路径前缀：一律网络直连（不缓存） */
export const API_PREFIX = '/reader3/'
/** 图片代理路径：网络优先，仅缓存合法图片响应 */
export const PROXY_PATH = '/assets/proxy'
/** 静态缓存条目上限（防无界膨胀；超出时删除最旧条目） */
export const STATIC_CACHE_MAX_ENTRIES = 200

/** 是否动态 API 路径（/reader3/*——含 file/download、SSE、probeSecureMode 等） */
export function isApiPath(pathname) {
  return pathname.startsWith(API_PREFIX)
}

/** 是否图片代理路径（/assets/proxy） */
export function isProxyPath(pathname) {
  return pathname === PROXY_PATH
}

/** 是否动态路径（API / 图片代理 / 下载）：M5——不得 cache-first */
export function isDynamicPath(pathname) {
  return isApiPath(pathname) || isProxyPath(pathname)
}

/** 响应是否为合法图片（Content-Type image/*）——仅此类响应可入图片代理缓存 */
export function isImageResponse(response) {
  const ct = (response.headers.get('content-type') || '').split(';')[0].trim().toLowerCase()
  return ct.startsWith('image/')
}

/** 预缓存的离线壳（install 时写入，供离线首屏） */
const PRECACHE_URLS = ['/', '/index.html', '/manifest.webmanifest']

// ==================== 以下为浏览器 SW 注册逻辑（node 单测环境无 self，自动跳过） ====================

if (typeof self !== 'undefined' && typeof self.addEventListener === 'function') {
  self.addEventListener('install', (event) => {
    event.waitUntil(
      caches
        .open(SHELL_CACHE)
        .then((cache) => cache.addAll(PRECACHE_URLS))
        .then(() => self.skipWaiting()),
    )
  })

  self.addEventListener('activate', (event) => {
    event.waitUntil(
      caches
        .keys()
        .then((keys) =>
          Promise.all(
            keys
              .filter((k) => k !== SHELL_CACHE && k !== STATIC_CACHE)
              .map((k) => caches.delete(k)),
          ),
        )
        .then(() => self.clients.claim()),
    )
  })

  self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'SKIP_WAITING') {
      self.skipWaiting()
    }
  })

  /** 是否同源 GET 请求 */
  function isSameOriginGet(request) {
    if (request.method !== 'GET') return false
    const url = new URL(request.url)
    return url.origin === self.location.origin
  }

  /** 网络优先 + 缓存回退（核心壳导航 / 图片代理）；写入受 putPolicy 约束 */
  async function networkFirst(request, options = {}) {
    const cacheName = options.cacheName || SHELL_CACHE
    const cacheKey = options.cacheKey || request
    const imageOnly = !!options.imageOnly
    const cache = await caches.open(cacheName)
    try {
      const response = await fetch(request)
      if (response && response.ok && (!imageOnly || isImageResponse(response))) {
        await cache.put(cacheKey, response.clone())
      }
      return response
    } catch {
      const cached = await cache.match(cacheKey)
      if (cached) return cached
      throw new Error(`offline: ${request.url}`)
    }
  }

  /** 静态缓存条目数上限：超出时删除最旧条目（cache.keys 顺序 ≈ 写入序） */
  async function trimStaticCache(cache) {
    const keys = await cache.keys()
    const excess = keys.length - STATIC_CACHE_MAX_ENTRIES + 1
    if (excess <= 0) return
    await Promise.all(keys.slice(0, excess).map((k) => cache.delete(k)))
  }

  /** 缓存优先 + 网络回填（静态资源：hash 文件名 / 字体 / 图片 / manifest） */
  async function cacheFirst(request) {
    const cache = await caches.open(STATIC_CACHE)
    const cached = await cache.match(request)
    if (cached) return cached
    try {
      const response = await fetch(request)
      if (response && response.ok) {
        await trimStaticCache(cache)
        await cache.put(request, response.clone())
      }
      return response
    } catch {
      if (cached) return cached
      throw new Error(`offline: ${request.url}`)
    }
  }

  self.addEventListener('fetch', (event) => {
    const { request } = event
    if (!isSameOriginGet(request)) return
    const pathname = new URL(request.url).pathname

    // 动态 API（/reader3/*）：网络直连，不缓存、不拦截
    // （M5：cache-first 会把 200+错误体 / 换图后的旧封面永久缓存；probeSecureMode 的
    //   _t 防缓存参数也不再为每次探测新建缓存条目）
    if (isApiPath(pathname)) return

    // 导航请求（页面 / 前端路由）→ 网络优先 + 缓存回退（SPA：统一以 /index.html 为键）
    if (request.mode === 'navigate') {
      event.respondWith(
        networkFirst(request, { cacheKey: '/index.html' }).catch(() => caches.match('/index.html')),
      )
      return
    }

    // 图片代理 /assets/proxy → 网络优先；仅缓存合法图片成功响应
    if (isProxyPath(pathname)) {
      event.respondWith(networkFirst(request, { cacheName: STATIC_CACHE, imageOnly: true }))
      return
    }

    // 静态资源 → 缓存优先（构建产物带 hash，安全）；失败不拿 HTML 冒充
    event.respondWith(cacheFirst(request))
  })
}
