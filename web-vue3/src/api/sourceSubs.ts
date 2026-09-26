import { get, post } from './request'
import { onBackendReachable } from './backendFlag'
import type { BookSource, ReturnData, SourceSub } from '@/types'

/**
 * 书源订阅存储层 —— 服务端为准，localStorage 仅为离线只读镜像：
 * - 后端可用：读写走服务端（账号内多设备一致）
 * - 后端不可用：可读取上次缓存，但写操作明确失败，避免离线假成功
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/getSourceSubs      → ReturnData<SourceSub[]>
 * POST /reader3/saveSourceSub      body: { url, name } → ReturnData<{ count }>
 *                                   （服务端抓取校验远程书源 JSON → 订阅入库 + 批量导入书源表）
 * POST /reader3/refreshSourceSub   body: { url }       → ReturnData<{ count }>
 *                                   （重新拉取并覆盖导入书源；订阅需已存在）
 * POST /reader3/deleteSourceSub    body: { url }       → ReturnData<null>
 *                                   （仅删订阅行，不影响已导入书源）
 * POST /reader3/deleteSourceSubs   body: string[] | { urls: [] } → ReturnData<{ deleted }>
 * POST /reader3/setSourceSubEnabled body: { url, enabled } → ReturnData<{ enabled }>
 * ================================================================
 * localStorage key: reader_source_subs:<username>:<scope>（值为 SourceSub[] 的 JSON）。
 * 旧全局键不迁移：它无法证明属于哪个用户，读取会造成跨账号泄露。
 * 订阅只记录远程书源地址与名称；书源数据由后端 saveSourceSub/refreshSourceSub 导入，
 * 不在服务端业务拒绝或网络断开时切换到浏览器抓取。
 * 订阅支持「禁用」：禁用后停止自动刷新，保留订阅记录与已导入书源；删除则移除订阅。
 */

const STORAGE_KEY = 'reader_source_subs'

function scopedStorageKey(): string | null {
  try {
    const localToken = localStorage.getItem('reader_access_token')
    const sessionToken = sessionStorage.getItem('reader_access_token')
    const username = localToken
      ? localStorage.getItem('reader_username')
      : sessionToken
        ? sessionStorage.getItem('reader_username')
        : null
    if (!username) return null
    const defaultScope =
      localStorage.getItem('reader_default_config_mode') === '1' ||
      sessionStorage.getItem('reader_default_config_mode') === '1'
    return `${STORAGE_KEY}:${encodeURIComponent(username)}:${defaultScope ? 'default' : 'user'}`
  } catch {
    return null
  }
}

/** 同步读取（localStorage 异常时返回空数组） */
export function loadSourceSubs(): SourceSub[] {
  try {
    const key = scopedStorageKey()
    if (!key) return []
    const raw = localStorage.getItem(key)
    if (!raw) return []
    const arr = JSON.parse(raw) as unknown
    if (!Array.isArray(arr)) return []
    return (arr as SourceSub[]).filter((s) => s && typeof s === 'object' && typeof s.url === 'string')
  } catch {
    return []
  }
}

/** 同步持久化整表 */
export function persistSourceSubs(subs: SourceSub[]): void {
  try {
    const key = scopedStorageKey()
    if (key) localStorage.setItem(key, JSON.stringify(subs))
  } catch {
    /* localStorage 满/不可用：忽略 */
  }
}

/** 业务错误保留原文案；网络错误明确提示未写入服务器。 */
function errMsg(err: unknown, fallback: string): { msg: string; down: boolean } {
  if (err instanceof Error) {
    const e = err as Error & { data?: unknown; response?: { data?: { errorMsg?: string } }; code?: string }
    if ('data' in e || 'response' in e) {
      const timeout =
        e.code === 'ECONNABORTED' || (e.message || '').toLowerCase().includes('timeout')
      const msg = timeout
        ? '请求超时：订阅源较大或网络较慢，请稍后重试'
        : e.response?.data?.errorMsg || e.message || fallback
      return { msg, down: false }
    }
  }
  return { msg: '服务端暂不可用，订阅未更改', down: true }
}

/** GET /reader3/getSourceSubs（后端优先；失败降级 localStorage 并镜像缓存） */
export async function getSourceSubs(): Promise<ReturnData<SourceSub[]>> {
  try {
    const res = await get<SourceSub[]>('/getSourceSubs', undefined, { silent: true })
    if (res.isSuccess) persistSourceSubs(res.data ?? [])
    return res
  } catch (err) {
    const { msg } = errMsg(err, '获取订阅列表失败')
    return { isSuccess: false, errorMsg: msg, data: loadSourceSubs() }
  }
}

/**
 * POST /reader3/saveSourceSub（服务端抓取校验、持久化并导入书源）。
 * 失败时不写本地镜像，也不伪装成一个仅在当前浏览器存在的订阅。
 */
export async function saveSourceSub(
  url: string,
  name: string,
  selectedUrls?: string[],
): Promise<ReturnData<{ count: number; name?: string } | null>> {
  try {
    const res = await post<{ count: number; name?: string }>(
      '/saveSourceSub',
      { url, name, ...(selectedUrls ? { selectedUrls } : {}) },
      { silent: true, timeout: 60000 },
    )
    // A reachable backend can reject SSRF, invalid payloads, limits, or permissions.
    // That is a real business failure, not an offline condition: do not create a
    // local-only subscription or make the UI appear to have saved it.
    if (!res.isSuccess) return res
    const list = loadSourceSubs()
    const existing = list.find((s) => s.url === url)
    if (existing) {
      existing.name = name
    } else {
      list.push({ url, name })
    }
    persistSourceSubs(list)
    return res
  } catch (err) {
    const { msg } = errMsg(err, '订阅失败')
    return { isSuccess: false, errorMsg: msg, data: null }
  }
}

/**
 * POST /reader3/previewSourceSub：拉取订阅 URL 并返回书源列表 + 库内已存在 URL
 * （不写订阅、不导入书源），供前端选择/排序后确认。
 */
export async function previewSourceSub(
  url: string,
): Promise<ReturnData<{ sources: BookSource[]; existing: string[] } | null>> {
  try {
    return await post<{ sources: BookSource[]; existing: string[] }>(
      '/previewSourceSub',
      { url },
      { silent: true, timeout: 60000 },
    )
  } catch (err) {
    const { msg } = errMsg(err, '订阅预览失败')
    return { isSuccess: false, errorMsg: msg, data: null }
  }
}

/** POST /reader3/deleteSourceSub（服务端确认成功后才更新本地镜像） */
export async function deleteSourceSub(url: string): Promise<ReturnData<null>> {
  try {
    const res = await post<null>('/deleteSourceSub', { url }, { silent: true })
    if (res.isSuccess) persistSourceSubs(loadSourceSubs().filter((s) => s.url !== url))
    return res
  } catch (err) {
    const { msg } = errMsg(err, '删除订阅失败')
    return { isSuccess: false, errorMsg: msg, data: null }
  }
}

/**
 * POST /reader3/deleteSourceSubs（批量；服务端失败时保留本地镜像，避免伪报已删除）。
 */
export async function deleteSourceSubs(urls: string[]): Promise<ReturnData<{ deleted: number }>> {
  if (urls.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: { deleted: 0 } }
  try {
    const res = await post<{ deleted: number }>(
      '/deleteSourceSubs',
      { urls },
      { silent: true },
    )
    if (res.isSuccess) {
      const keep = new Set(urls)
      persistSourceSubs(loadSourceSubs().filter((s) => !keep.has(s.url)))
    }
    return res
  } catch (err) {
    const { msg } = errMsg(err, '批量删除订阅失败')
    return { isSuccess: false, errorMsg: msg, data: { deleted: 0 } }
  }
}

/**
 * POST /reader3/setSourceSubEnabled（启停订阅：禁用后定时任务跳过自动刷新，
 * 订阅记录与已导入书源保留）。服务端失败时本地镜像也保持原状。
 */
export async function setSourceSubEnabled(
  url: string,
  enabled: boolean,
): Promise<ReturnData<{ enabled: boolean }>> {
  try {
    const res = await post<{ enabled: boolean }>(
      '/setSourceSubEnabled',
      { url, enabled },
      { silent: true },
    )
    if (!res.isSuccess) return res
    const list = loadSourceSubs()
    const sub = list.find((s) => s.url === url)
    if (sub) sub.enabled = enabled
    persistSourceSubs(list)
    return res
  } catch (err) {
    const { msg } = errMsg(err, '操作失败')
    return { isSuccess: false, errorMsg: msg, data: { enabled } }
  }
}

/**
 * POST /reader3/refreshSourceSub（后端优先：重新拉取远程书源 JSON 并覆盖导入书源表，返回导入数；
 * 订阅不存在返回业务失败）。失败返回 isSuccess=false，不进行浏览器直连回退。
 */
export async function refreshSourceSub(url: string): Promise<ReturnData<{ count: number }>> {
  try {
    return await post<{ count: number }>(
      '/refreshSourceSub',
      { url },
      { silent: true, timeout: 60000 },
    )
  } catch (err) {
    const { msg } = errMsg(err, '刷新订阅失败')
    return { isSuccess: false, errorMsg: msg, data: { count: 0 } }
  }
}

/** 恢复后端调用（登录态变化/网络恢复时由上层调用） */
export function resetBackendFlag(): void {
  /* no-op：不再使用全局短路标志，网络恢复由 request.ts 自动复位 */
}

// P2：任一后端请求成功（request.ts 拦截器）即复位短路标志——网络恢复后自动回到后端优先
onBackendReachable(resetBackendFlag)
