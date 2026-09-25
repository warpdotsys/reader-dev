import { get, post } from './request'
import { onBackendReachable } from './backendFlag'
import type { BookSource, ReturnData, SourceSub } from '@/types'

/**
 * 书源订阅存储层 —— 后端为主（/reader3/getSourceSubs 等），localStorage 为降级缓存：
 * - 后端可用：读写走服务端（账号内多设备一致）
 * - 后端失败：降级 localStorage（reader_source_subs），功能不中断
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
 * localStorage key: reader_source_subs（值为 SourceSub[] 的 JSON）
 * 订阅只记录远程书源地址与名称；书源数据由后端 saveSourceSub/refreshSourceSub 导入，
 * 降级模式下由调用方前端 fetch + saveBookSources 导入。
 * 订阅支持「禁用」：禁用后停止自动刷新，保留订阅记录与已导入书源；删除则移除订阅。
 */

const STORAGE_KEY = 'reader_source_subs'

/** 同步读取（localStorage 异常时返回空数组） */
export function loadSourceSubs(): SourceSub[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
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
    localStorage.setItem(STORAGE_KEY, JSON.stringify(subs))
  } catch {
    /* localStorage 满/不可用：忽略 */
  }
}

/** 业务错误（拦截器 reject 携带 data / HTTP 响应）——后端可达，展示真实错误；纯网络错误才置 backendDown */
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
  // 网络错误不再做永久短路：request.ts 在任一后端成功/HTTP 响应时复位；
  // 这里保留 down 标记仅用于调用方决定是否降级本地存储，不缓存全局状态。
  return { msg: '服务端暂不可用，已降级本地数据', down: true }
}

/** GET /reader3/getSourceSubs（后端优先；失败降级 localStorage 并镜像缓存） */
export async function getSourceSubs(): Promise<ReturnData<SourceSub[]>> {
  try {
    const res = await get<SourceSub[]>('/getSourceSubs', undefined, { silent: true })
    persistSourceSubs(res.data ?? [])
    return res
  } catch (err) {
    const { msg } = errMsg(err, '获取订阅列表失败')
    return { isSuccess: false, errorMsg: msg, data: loadSourceSubs() }
  }
}

/**
 * POST /reader3/saveSourceSub（后端优先：抓取校验 + 订阅入库 + 批量导入书源，返回导入数）。
 * 降级（后端不可达）：仅写入 localStorage，data=null —— 调用方需自行导入书源（fetch + saveBookSources）。
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
    const { msg, down } = errMsg(err, '订阅失败')
    if (!down) return { isSuccess: false, errorMsg: msg, data: null }
  }
  const list = loadSourceSubs()
  const existing = list.find((s) => s.url === url)
  if (existing) {
    existing.name = name
  } else {
    list.push({ url, name })
  }
  persistSourceSubs(list)
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: null }
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

/** POST /reader3/deleteSourceSub（后端优先；失败降级 localStorage） */
export async function deleteSourceSub(url: string): Promise<ReturnData<null>> {
  try {
    const res = await post<null>('/deleteSourceSub', { url }, { silent: true })
    persistSourceSubs(loadSourceSubs().filter((s) => s.url !== url))
    return res
  } catch (err) {
    const { msg, down } = errMsg(err, '删除订阅失败')
    if (!down) return { isSuccess: false, errorMsg: msg, data: null }
  }
  persistSourceSubs(loadSourceSubs().filter((s) => s.url !== url))
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: null }
}

/**
 * POST /reader3/deleteSourceSubs（批量；后端失败降级为逐条 deleteSourceSub）。
 */
export async function deleteSourceSubs(urls: string[]): Promise<ReturnData<{ deleted: number }>> {
  if (urls.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: { deleted: 0 } }
  try {
    const res = await post<{ deleted: number }>(
      '/deleteSourceSubs',
      { urls },
      { silent: true },
    )
    const keep = new Set(urls)
    persistSourceSubs(loadSourceSubs().filter((s) => !keep.has(s.url)))
    return res
  } catch (err) {
    const { msg, down } = errMsg(err, '批量删除订阅失败')
    if (!down) return { isSuccess: false, errorMsg: msg, data: { deleted: 0 } }
  }
  let deleted = 0
  for (const url of urls) {
    const res = await deleteSourceSub(url)
    if (res.isSuccess) deleted += 1
  }
  return { isSuccess: true, errorMsg: '', data: { deleted } }
}

/**
 * POST /reader3/setSourceSubEnabled（启停订阅：禁用后定时任务跳过自动刷新，
 * 订阅记录与已导入书源保留）。后端失败降级 localStorage。
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
    const list = loadSourceSubs()
    const sub = list.find((s) => s.url === url)
    if (sub) sub.enabled = enabled
    persistSourceSubs(list)
    return res
  } catch (err) {
    const { msg, down } = errMsg(err, '操作失败')
    if (!down) return { isSuccess: false, errorMsg: msg, data: { enabled } }
  }
  const list = loadSourceSubs()
  const sub = list.find((s) => s.url === url)
  if (sub) sub.enabled = enabled
  persistSourceSubs(list)
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: { enabled } }
}

/**
 * POST /reader3/refreshSourceSub（后端优先：重新拉取远程书源 JSON 并覆盖导入书源表，返回导入数；
 * 订阅不存在返回业务失败）。失败返回 isSuccess=false（不抛异常），由调用方降级为前端 fetch + saveBookSources 导入。
 */
export async function refreshSourceSub(url: string): Promise<ReturnData<{ count: number }>> {
  try {
    return await post<{ count: number }>(
      '/refreshSourceSub',
      { url },
      { silent: true, timeout: 60000 },
    )
  } catch (err) {
    const { msg, down } = errMsg(err, '刷新订阅失败')
    return { isSuccess: false, errorMsg: down ? '' : msg, data: { count: 0 } }
  }
}

/** 恢复后端调用（登录态变化/网络恢复时由上层调用） */
export function resetBackendFlag(): void {
  /* no-op：不再使用全局短路标志，网络恢复由 request.ts 自动复位 */
}

// P2：任一后端请求成功（request.ts 拦截器）即复位短路标志——网络恢复后自动回到后端优先
onBackendReachable(resetBackendFlag)
