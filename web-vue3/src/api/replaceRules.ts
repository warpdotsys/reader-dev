import { get, post } from './request'
import { onBackendReachable } from './backendFlag'
import { fromLegacyReplaceRule, toLegacyReplaceRule } from './replaceRuleAdapter'
import type { ReplaceRule, ReturnData } from '@/types'

/**
 * 替换规则适配器。
 * Vue 3 用 find/replace/enabled；Java/Kotlin data class 使用 pattern/
 * replacement/isEnabled、Long id。所有服务端请求必须经过显式字段映射，
 * 不能把 Vue/Rust 形状直接发给 legacy CURD。
 *
 * GET  /reader3/getReplaceRules    → legacy ReplaceRule[]
 * POST /reader3/saveReplaceRule   → legacy ReplaceRule
 * POST /reader3/saveReplaceRules  → legacy ReplaceRule[]
 * POST /reader3/deleteReplaceRule → 完整 legacy ReplaceRule（控制器按 name 匹配）
 * POST /reader3/deleteReplaceRules → legacy ReplaceRule[]
 *
 * localStorage 只作为明确标注的离线缓存；HTTP/业务错误不会伪装成本地成功。
 */

const STORAGE_KEY = 'reader_replace_rules'
let backendDown = false

export function loadReplaceRules(): ReplaceRule[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const arr = JSON.parse(raw) as unknown
    if (!Array.isArray(arr)) return []
    return (arr as ReplaceRule[]).filter(
      (rule) => rule && typeof rule === 'object' && typeof rule.find === 'string',
    )
  } catch {
    return []
  }
}

export function persistReplaceRules(rules: ReplaceRule[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(rules))
  } catch {
    // Quota/private-mode errors must not turn a successful server write into a failure.
  }
}

function isOfflineTransportError(error: unknown): boolean {
  if (!(error instanceof Error)) return false
  const candidate = error as Error & {
    data?: unknown
    response?: unknown
    code?: string
  }
  if ('data' in candidate || 'response' in candidate) return false
  if (
    candidate.code === 'ECONNABORTED' ||
    candidate.code === 'ETIMEDOUT' ||
    /timeout/i.test(candidate.message)
  ) {
    return false
  }
  return true
}

function offlineResult<T>(data: T): ReturnData<T> {
  return { isSuccess: false, errorMsg: '服务端不可达，本次变更仅保存在本机，尚未同步', data }
}

function mergeLocalRules(rules: ReplaceRule[]): void {
  const current = loadReplaceRules()
  for (const rule of rules) {
    const index = current.findIndex((item) => item.name === rule.name || item.id === rule.id)
    if (index >= 0) current[index] = rule
    else current.push(rule)
  }
  persistReplaceRules(current)
}

function removeLocalRules(rules: ReplaceRule[]): void {
  const ids = new Set(rules.map((rule) => rule.id))
  const names = new Set(rules.map((rule) => rule.name))
  persistReplaceRules(loadReplaceRules().filter((rule) => !ids.has(rule.id) && !names.has(rule.name)))
}

export async function getReplaceRules(): Promise<ReturnData<ReplaceRule[]>> {
  if (backendDown) return offlineResult(loadReplaceRules())
  try {
    const response = await get<unknown>('/getReplaceRules')
    if (!Array.isArray(response.data)) {
      throw new Error('服务器返回的替换规则不是数组')
    }
    const rules = response.data.map(fromLegacyReplaceRule)
    persistReplaceRules(rules)
    return { ...response, data: rules }
  } catch (error) {
    if (!isOfflineTransportError(error)) throw error
    backendDown = true
    return offlineResult(loadReplaceRules())
  }
}

export async function saveReplaceRule(rule: ReplaceRule): Promise<ReturnData<null>> {
  if (backendDown) {
    mergeLocalRules([rule])
    return offlineResult(null)
  }
  try {
    const response = await post<unknown>('/saveReplaceRule', toLegacyReplaceRule(rule))
    mergeLocalRules([rule])
    return { ...response, data: null }
  } catch (error) {
    if (!isOfflineTransportError(error)) throw error
    backendDown = true
    mergeLocalRules([rule])
    return offlineResult(null)
  }
}

export async function saveReplaceRules(rules: ReplaceRule[]): Promise<ReturnData<{ count: number }>> {
  if (backendDown) {
    mergeLocalRules(rules)
    return offlineResult({ count: rules.length })
  }
  try {
    const response = await post<unknown>('/saveReplaceRules', rules.map(toLegacyReplaceRule))
    mergeLocalRules(rules)
    return { ...response, data: { count: rules.length } }
  } catch (error) {
    if (!isOfflineTransportError(error)) throw error
    backendDown = true
    mergeLocalRules(rules)
    return offlineResult({ count: rules.length })
  }
}

export async function deleteReplaceRule(rule: ReplaceRule): Promise<ReturnData<null>> {
  if (backendDown) {
    removeLocalRules([rule])
    return offlineResult(null)
  }
  try {
    const response = await post<unknown>('/deleteReplaceRule', toLegacyReplaceRule(rule))
    removeLocalRules([rule])
    return { ...response, data: null }
  } catch (error) {
    if (!isOfflineTransportError(error)) throw error
    backendDown = true
    removeLocalRules([rule])
    return offlineResult(null)
  }
}

export async function deleteReplaceRules(rules: ReplaceRule[]): Promise<ReturnData<{ count: number }>> {
  if (rules.length === 0) return { isSuccess: true, errorMsg: '', data: { count: 0 } }
  if (backendDown) {
    removeLocalRules(rules)
    return offlineResult({ count: rules.length })
  }
  try {
    // CURD.deleteMulti expects an array of entities, not { ids: [...] }.
    const response = await post<unknown>('/deleteReplaceRules', rules.map(toLegacyReplaceRule), { silent: true })
    removeLocalRules(rules)
    return { ...response, data: { count: rules.length } }
  } catch (error) {
    if (!isOfflineTransportError(error)) throw error
    backendDown = true
    removeLocalRules(rules)
    return offlineResult({ count: rules.length })
  }
}

/** Any successful API response makes a later retry use the server again. */
export function resetBackendFlag(): void {
  backendDown = false
}

onBackendReachable(resetBackendFlag)
