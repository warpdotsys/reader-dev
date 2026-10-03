import axios from 'axios'
import { get, post } from './request'
import { onBackendReachable } from './backendFlag'
import type { HttpTts, ReturnData } from '@/types'

/**
 * HttpTTS 听书源存储层 —— 后端为主（/reader3/httpTTS/*），localStorage 为降级缓存：
 * - 后端可用：读写走服务端（账号内多设备一致）
 * - 后端失败：降级 localStorage，功能不中断
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/httpTTS/list       → ReturnData<HttpTts[]>
 * POST /reader3/httpTTS/save       body: HttpTTS entity  → ReturnData<string>（data 通常为 ""）
 * POST /reader3/httpTTS/saveMulti  body: HttpTTS[]       → ReturnData<string>（data 通常为 ""）
 * POST /reader3/httpTTS/delete     body: HttpTTS entity  → ReturnData<string>（data 通常为 ""）
 * POST /reader3/httpTTS/deleteMulti body: HttpTTS[]      → ReturnData<string>（data 通常为 ""）
 * ================================================================
 * localStorage key: reader_http_tts_list（值为 HttpTts[] 的 JSON）
 * type 参考 legado HttpTTS：0=在线合成（http 请求音频），1=本地引擎（预留）
 */

const STORAGE_KEY = 'reader_http_tts_list'

/**
 * legacy HttpTTS.id 是 Kotlin Long，而 Vue 3 预览期曾生成 base36 字符串 id。
 * 将非数值 id 稳定映射到正安全整数，使同一条缓存记录后续保存/删除命中同一后端实体。
 */
function toLegacyId(id: string): number {
  const numeric = Number(id)
  if (Number.isSafeInteger(numeric) && numeric >= 0) return numeric

  let hash = 2166136261
  for (let index = 0; index < id.length; index += 1) {
    hash ^= id.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  return 1_000_000_000_000 + (hash >>> 0)
}

/** 仅发送 legacy HttpTTS 识别的字段，避免把 Vue 侧展示字段写入持久化数据。 */
function toLegacyHttpTts(tts: HttpTts): Record<string, unknown> {
  return {
    id: toLegacyId(tts.id),
    name: tts.name,
    url: tts.url,
    contentType: tts.contentType,
    concurrentRate: tts.concurrentRate,
    loginUrl: tts.loginUrl,
    loginUi: tts.loginUi,
    header: tts.header,
    jsLib: tts.jsLib,
    enabledCookieJar: tts.enabledCookieJar,
    loginCheckJs: tts.loginCheckJs,
  }
}

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

/** 业务和 HTTP 错误由调用方显示；只有没有响应的传输故障才允许降级到本地缓存。 */
function isTransportFailure(error: unknown): boolean {
  // 业务失败由 request.ts 包装为普通 Error（带 data），不能伪装成断网成功写入本地。
  // 只有 Axios 未收到 HTTP 响应，才允许保留离线缓存并等待重连。
  return axios.isAxiosError(error) && !error.response
}

/** 后端不可达标志（本模块内短路，避免每次操作都等超时） */
let backendDown = false

/** GET /reader3/httpTTS/list（后端优先；失败降级 localStorage 并镜像缓存） */
export async function getHttpTtsList(): Promise<ReturnData<HttpTts[]>> {
  if (backendDown) {
    return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: loadHttpTtsList() }
  }
  try {
    const res = await get<HttpTts[]>('/httpTTS/list')
    persistHttpTtsList(res.data ?? [])
    return res
  } catch (error) {
    if (!isTransportFailure(error)) throw error
    backendDown = true
    return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: loadHttpTtsList() }
  }
}

/** POST /reader3/httpTTS/save（后端优先；失败降级 localStorage，id 相同则覆盖） */
export async function saveHttpTts(tts: HttpTts): Promise<ReturnData<string>> {
  if (!backendDown) {
    try {
      const res = await post<string>('/httpTTS/save', toLegacyHttpTts(tts))
      const list = loadHttpTtsList()
      const i = list.findIndex((t) => t.id === tts.id)
      if (i >= 0) list[i] = tts
      else list.push(tts)
      persistHttpTtsList(list)
      return res
    } catch (error) {
      if (!isTransportFailure(error)) throw error
      backendDown = true
    }
  }
  const list = loadHttpTtsList()
  const i = list.findIndex((t) => t.id === tts.id)
  if (i >= 0) list[i] = tts
  else list.push(tts)
  persistHttpTtsList(list)
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: '' }
}

/** POST /reader3/httpTTS/saveMulti（后端 data 为 ""，导入数量由调用方输入列表确定） */
export async function saveHttpTtsMulti(list: HttpTts[]): Promise<ReturnData<string>> {
  if (list.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: '' }
  if (!backendDown) {
    try {
      const res = await post<string>('/httpTTS/saveMulti', list.map(toLegacyHttpTts))
      const merged = [...loadHttpTtsList()]
      for (const t of list) {
        const i = merged.findIndex((x) => x.id === t.id || x.url === t.url)
        if (i >= 0) merged[i] = t
        else merged.push(t)
      }
      persistHttpTtsList(merged)
      return res
    } catch (error) {
      if (!isTransportFailure(error)) throw error
      backendDown = true
    }
  }
  for (const t of list) {
    await saveHttpTts(t)
  }
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: '' }
}

/** POST /reader3/httpTTS/delete（删除需要完整实体，legacy 以 name 为键匹配） */
export async function deleteHttpTts(tts: HttpTts): Promise<ReturnData<string>> {
  if (!backendDown) {
    try {
      const res = await post<string>('/httpTTS/delete', toLegacyHttpTts(tts))
      persistHttpTtsList(loadHttpTtsList().filter((t) => t.id !== tts.id))
      return res
    } catch (error) {
      if (!isTransportFailure(error)) throw error
      backendDown = true
    }
  }
  persistHttpTtsList(loadHttpTtsList().filter((t) => t.id !== tts.id))
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: '' }
}

/** POST /reader3/httpTTS/deleteMulti（后端 data 为 ""，删除数量由调用方输入列表确定） */
export async function deleteHttpTtsMany(list: HttpTts[]): Promise<ReturnData<string>> {
  if (list.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: '' }
  if (!backendDown) {
    try {
      const res = await post<string>('/httpTTS/deleteMulti', list.map(toLegacyHttpTts), { silent: true })
      const removed = new Set(list.map((t) => t.id))
      persistHttpTtsList(loadHttpTtsList().filter((t) => !removed.has(t.id)))
      return res
    } catch (error) {
      if (!isTransportFailure(error)) throw error
      backendDown = true
    }
  }
  for (const tts of list) {
    await deleteHttpTts(tts)
  }
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: '' }
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
