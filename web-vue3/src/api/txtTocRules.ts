import { get, post } from './request'
import type { ReturnData, TxtTocRule } from '@/types'

/**
 * 自定义 TXT 目录规则（对齐 legado TxtTocRule）
 *
 * ============================ 后端契约 ============================
 * GET  /reader3/getTxtTocRules             → ReturnData<TxtTocRule[]>（内置默认规则 + 用户自定义）
 * POST /reader3/saveTxtTocRule             body: TxtTocRule       → ReturnData<null>
 * POST /reader3/deleteTxtTocRule           body: { id: string }   → ReturnData<null>
 * POST /reader3/importDefaultTxtTocRules   → ReturnData<{ count: number }>
 * ================================================================
 * 上传 TXT 本地书时，后端会用启用的用户规则分章（无用户规则回退内置默认规则）。
 */

/** GET /reader3/getTxtTocRules：内置默认规则 + 用户自定义规则 */
export function getTxtTocRules(): Promise<ReturnData<TxtTocRule[]>> {
  return get<TxtTocRule[]>('/getTxtTocRules')
}

/** POST /reader3/saveTxtTocRule：保存（id 相同覆盖，缺 id 后端自动补；
 *  P1-C2：响应 data.id = 后端生效 id——归属冲突时后端改插新 id） */
export function saveTxtTocRule(rule: TxtTocRule): Promise<ReturnData<{ id?: string } | null>> {
  return post<{ id?: string } | null>('/saveTxtTocRule', rule)
}

/** POST /reader3/deleteTxtTocRule：按 id 删除 */
export function deleteTxtTocRule(id: string): Promise<ReturnData<null>> {
  return post<null>('/deleteTxtTocRule', { id })
}

/** POST /reader3/importDefaultTxtTocRules：导入内置默认规则为用户规则（幂等） */
export function importDefaultTxtTocRules(): Promise<ReturnData<{ count: number }>> {
  return post<{ count: number }>('/importDefaultTxtTocRules')
}
