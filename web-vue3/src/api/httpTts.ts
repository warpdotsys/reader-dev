import { get, post } from './request'
import { onBackendReachable } from './backendFlag'
import type { HttpTts, ReturnData } from '@/types'

/**
 * HttpTTS 听书源存储层 —— 后端为主（/reader3/getHttpTTSList 等），localStorage 为降级缓存：
 * - 后端可用：读写走服务端（账号内多设备一致）
 * - 后端失败：降级 localStorage，功能不中断
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/getHttpTTSList    → ReturnData<HttpTts[]>
 * POST /reader3/saveHttpTTS       body: HttpTts         → ReturnData<null>
 * POST /reader3/httpTTS/saveMulti body: HttpTts[]       → ReturnData<{ count }>
 * POST /reader3/deleteHttpTTS     body: { id: string }  → ReturnData<null>
 * POST /reader3/deleteHttpTTSs    body: { ids: string[] } → ReturnData<{ count }>
 * ================================================================
 * localStorage key: reader_http_tts_list（值为 HttpTts[] 的 JSON）
 * type 参考 legado HttpTTS：0=在线合成（http 请求音频），1=本地引擎（预留）
 */

const STORAGE_KEY = 'reader_http_tts_list'

/** 同步读取（localStorage 异常时返回空数组） */
export function loadHttpTtsList(): HttpTts[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const arr = JSON.parse(raw) as unknown
    if (!Array.isArray(arr)) return []
    return (arr as HttpTts[]).filter((t) => t && typeof t === 'object' && typeof t.url === 'string')
  } catch {
    return []
  }
}

function persistHttpTtsList(list: HttpTts[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(list))
  } catch {
    /* localStorage 满/不可用：忽略 */
  }
}

/** 后端不可达标志（本模块内短路，避免每次操作都等超时） */
let backendDown = false

/** GET /reader3/getHttpTTSList（后端优先；失败降级 localStorage 并镜像缓存） */
export async function getHttpTtsList(): Promise<ReturnData<HttpTts[]>> {
  if (backendDown) {
    return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: loadHttpTtsList() }
  }
  try {
    const res = await get<HttpTts[]>('/getHttpTTSList')
    persistHttpTtsList(res.data ?? [])
    return res
  } catch {
    backendDown = true
    return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: loadHttpTtsList() }
  }
}

/** POST /reader3/saveHttpTTS（后端优先；失败降级 localStorage，id 相同则覆盖） */
export async function saveHttpTts(tts: HttpTts): Promise<ReturnData<null>> {
  if (!backendDown) {
    try {
      const res = await post<null>('/saveHttpTTS', tts)
      const list = loadHttpTtsList()
      const i = list.findIndex((t) => t.id === tts.id)
      if (i >= 0) list[i] = tts
      else list.push(tts)
      persistHttpTtsList(list)
      return res
    } catch {
      backendDown = true
    }
  }
  const list = loadHttpTtsList()
  const i = list.findIndex((t) => t.id === tts.id)
  if (i >= 0) list[i] = tts
  else list.push(tts)
  persistHttpTtsList(list)
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: null }
}

/** POST /reader3/httpTTS/saveMulti（批量导入；后端失败降级为逐条 saveHttpTts） */
export async function saveHttpTtsMulti(list: HttpTts[]): Promise<ReturnData<{ count: number }>> {
  if (list.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: { count: 0 } }
  if (!backendDown) {
    try {
      const res = await post<{ count: number }>('/httpTTS/saveMulti', list)
      const merged = [...loadHttpTtsList()]
      for (const t of list) {
        const i = merged.findIndex((x) => x.id === t.id || x.url === t.url)
        if (i >= 0) merged[i] = t
        else merged.push(t)
      }
      persistHttpTtsList(merged)
      return res
    } catch {
      backendDown = true
    }
  }
  let count = 0
  for (const t of list) {
    const res = await saveHttpTts(t)
    if (res.isSuccess) count += 1
  }
  return { isSuccess: true, errorMsg: '', data: { count } }
}

/** POST /reader3/deleteHttpTTS（后端优先；失败降级 localStorage） */
export async function deleteHttpTts(id: string): Promise<ReturnData<null>> {
  if (!backendDown) {
    try {
      const res = await post<null>('/deleteHttpTTS', { id })
      persistHttpTtsList(loadHttpTtsList().filter((t) => t.id !== id))
      return res
    } catch {
      backendDown = true
    }
  }
  persistHttpTtsList(loadHttpTtsList().filter((t) => t.id !== id))
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: null }
}

/** POST /reader3/deleteHttpTTSs（批量；后端失败降级为逐条 deleteHttpTts） */
export async function deleteHttpTtsMany(ids: string[]): Promise<ReturnData<{ count: number }>> {
  if (ids.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: { count: 0 } }
  if (!backendDown) {
    try {
      const res = await post<{ count: number }>('/deleteHttpTTSs', { ids }, { silent: true })
      const removed = new Set(ids)
      persistHttpTtsList(loadHttpTtsList().filter((t) => !removed.has(t.id) && !removed.has(t.url)))
      return res
    } catch {
      backendDown = true
    }
  }
  let count = 0
  for (const id of ids) {
    const res = await deleteHttpTts(id)
    if (res.isSuccess) count += 1
  }
  return { isSuccess: true, errorMsg: '', data: { count } }
}

/** 从任意 JSON 文本解析 HttpTTS 数组（对象/数组兼容；id 缺失时生成） */
export function parseHttpTtsJson(raw: string): HttpTts[] {
  const parsed = JSON.parse(raw) as unknown
  const arr = Array.isArray(parsed) ? parsed : [parsed]
  const out: HttpTts[] = []
  for (const item of arr) {
    if (!item || typeof item !== 'object') continue
    const o = item as Record<string, unknown>
    const url = String(o.url ?? o.id ?? '').trim()
    if (!url) continue
    const name = String(o.name ?? url)
    out.push({
      id: String(o.id ?? url),
      name,
      url,
      type: typeof o.type === 'number' ? o.type : 0,
      contentType: o.contentType ? String(o.contentType) : undefined,
      concurrentRate: o.concurrentRate ? String(o.concurrentRate) : undefined,
      loginUrl: o.loginUrl ? String(o.loginUrl) : undefined,
      loginUi: o.loginUi ? JSON.stringify(o.loginUi) : undefined,
      header: o.header ? (typeof o.header === 'string' ? o.header : JSON.stringify(o.header)) : undefined,
      jsLib: o.jsLib ? String(o.jsLib) : undefined,
      enabledCookieJar: o.enabledCookieJar ? !!o.enabledCookieJar : undefined,
      loginCheckJs: o.loginCheckJs ? String(o.loginCheckJs) : undefined,
    })
  }
  return out
}

/** 恢复后端调用（登录态变化/网络恢复时由上层调用） */
export function resetBackendFlag(): void {
  backendDown = false
}

// P2：任一后端请求成功（request.ts 拦截器）即复位短路标志——网络恢复后自动回到后端优先
onBackendReachable(resetBackendFlag)
