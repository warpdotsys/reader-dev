import { get, post } from './request'
import { onBackendReachable } from './backendFlag'
import type { ReplaceRule, ReturnData } from '@/types'

/**
 * 替换规则存储层 —— 后端为主（GET/POST /reader3/*ReplaceRule*），localStorage 为降级缓存：
 * - 后端可用：读写走服务端（账号内多设备一致），成功后镜像写 localStorage（阅读页渲染
 *   走同步的 loadReplaceRules，无需等待网络）
 * - 后端失败（未启动/接口异常）：读写降级到 localStorage，功能不中断
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/getReplaceRules    → ReturnData<ReplaceRule[]>
 * POST /reader3/saveReplaceRule    body: ReplaceRule        → ReturnData<null>
 * POST /reader3/saveReplaceRules   body: ReplaceRule[]      → ReturnData<{ count: number }>
 * POST /reader3/deleteReplaceRule  body: { id: string }     → ReturnData<null>
 * POST /reader3/deleteReplaceRules body: { ids: string[] } | { all: true } | 规则对象数组 → ReturnData<{ count }>
 * ================================================================
 * localStorage key: reader_replace_rules（值为 ReplaceRule[] 的 JSON）
 */

const STORAGE_KEY = 'reader_replace_rules'

/** 同步读取（阅读页渲染时直接使用；localStorage 异常时返回空数组） */
export function loadReplaceRules(): ReplaceRule[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const arr = JSON.parse(raw) as unknown
    if (!Array.isArray(arr)) return []
    return (arr as ReplaceRule[]).filter((r) => r && typeof r === 'object' && typeof r.find === 'string')
  } catch {
    return []
  }
}

/** 同步持久化整表（后端成功后的本地镜像 / 后端失败时的降级存储） */
export function persistReplaceRules(rules: ReplaceRule[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(rules))
  } catch {
    /* localStorage 满/不可用：忽略 */
  }
}

/** 后端不可达标志（本模块内短路，避免每次操作都等 15s 超时） */
let backendDown = false

/** 业务错误（拦截器 reject 携带 data / HTTP 响应）——后端可达，展示真实错误；纯网络错误才置 backendDown */
function errMsg(err: unknown, fallback: string): { msg: string; down: boolean } {
  if (err instanceof Error) {
    const e = err as Error & { data?: unknown; response?: { data?: { errorMsg?: string } }; code?: string }
    if ('data' in e || 'response' in e) {
      const timeout =
        e.code === 'ECONNABORTED' || (e.message || '').toLowerCase().includes('timeout')
      const msg = timeout
        ? '请求超时，请稍后重试'
        : e.response?.data?.errorMsg || e.message || fallback
      return { msg, down: false }
    }
  }
  backendDown = true
  return { msg: '服务端暂不可用，已降级本地数据', down: true }
}

/** GET /reader3/getReplaceRules（后端优先；失败降级 localStorage 并镜像缓存） */
export async function getReplaceRules(): Promise<ReturnData<ReplaceRule[]>> {
  if (backendDown) {
    return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: loadReplaceRules() }
  }
  try {
    const res = await get<ReplaceRule[]>('/getReplaceRules')
    persistReplaceRules(res.data ?? []) // 镜像到本地（阅读页同步渲染）
    return res
  } catch {
    backendDown = true
    return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: loadReplaceRules() }
  }
}

/** POST /reader3/saveReplaceRule（后端优先；失败降级 localStorage）
 *  P1-C2：响应 data.id = 后端生效 id（归属冲突时后端改插新 id——前端据此同步本地列表） */
export async function saveReplaceRule(rule: ReplaceRule): Promise<ReturnData<{ id?: string } | null>> {
  if (!backendDown) {
    try {
      const res = await post<{ id?: string } | null>('/saveReplaceRule', rule)
      // 镜像更新本地缓存
      const list = loadReplaceRules()
      const i = list.findIndex((r) => r.id === rule.id)
      if (i >= 0) list[i] = rule
      else list.push(rule)
      persistReplaceRules(list)
      return res
    } catch {
      backendDown = true
    }
  }
  // 降级：本地增改
  const list = loadReplaceRules()
  const i = list.findIndex((r) => r.id === rule.id)
  if (i >= 0) list[i] = rule
  else list.push(rule)
  persistReplaceRules(list)
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: null }
}

/** POST /reader3/saveReplaceRules（批量；后端失败降级为整表本地覆盖） */
export async function saveReplaceRules(rules: ReplaceRule[]): Promise<ReturnData<{ count: number }>> {
  if (!backendDown) {
    try {
      const res = await post<{ count: number }>('/saveReplaceRules', rules)
      persistReplaceRules(rules)
      return res
    } catch {
      backendDown = true
    }
  }
  persistReplaceRules(rules)
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: { count: rules.length } }
}

/** POST /reader3/deleteReplaceRule（后端优先；失败降级 localStorage） */
export async function deleteReplaceRule(id: string): Promise<ReturnData<null>> {
  if (!backendDown) {
    try {
      const res = await post<null>('/deleteReplaceRule', { id })
      persistReplaceRules(loadReplaceRules().filter((r) => r.id !== id))
      return res
    } catch {
      backendDown = true
    }
  }
  persistReplaceRules(loadReplaceRules().filter((r) => r.id !== id))
  return { isSuccess: false, errorMsg: '服务端暂不可用，已降级本地数据', data: null }
}

/** POST /reader3/deleteReplaceRules（批量；后端失败降级为逐条 deleteReplaceRule） */
export async function deleteReplaceRules(ids: string[]): Promise<ReturnData<{ count: number }>> {
  if (ids.length === 0) return { isSuccess: false, errorMsg: '参数错误', data: { count: 0 } }
  if (!backendDown) {
    try {
      const res = await post<{ count: number }>('/deleteReplaceRules', { ids }, { silent: true })
      const removed = new Set(ids)
      persistReplaceRules(loadReplaceRules().filter((r) => !removed.has(r.id)))
      return res
    } catch (err) {
      const { msg, down } = errMsg(err, '批量删除规则失败')
      if (!down) return { isSuccess: false, errorMsg: msg, data: { count: 0 } }
    }
  }
  let count = 0
  for (const id of ids) {
    const res = await deleteReplaceRule(id)
    if (res.isSuccess) count += 1
  }
  return { isSuccess: true, errorMsg: '', data: { count } }
}

/** 恢复后端调用（登录态变化/网络恢复时由上层调用） */
export function resetBackendFlag(): void {
  backendDown = false
}

// P2：任一后端请求成功（request.ts 拦截器）即复位短路标志——网络恢复后自动回到后端优先
onBackendReachable(resetBackendFlag)
