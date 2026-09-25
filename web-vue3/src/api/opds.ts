import { get, post } from './request'
import type { ReturnData } from '@/types'

/**
 * OPDS 独立账号（/reader3/getOpdsSettings | saveOpdsSettings）：
 * - GET  /reader3/getOpdsSettings  → {enabled, username, passwordSet, namespace}（不回传密码）
 * - POST /reader3/saveOpdsSettings body {username, password}；username 空 = 禁用独立账号
 */

export interface OpdsSettings {
  enabled: boolean
  username: string
  passwordSet: boolean
  namespace?: string
}

/** GET /reader3/getOpdsSettings：读取 OPDS 账号配置（不回传密码） */
export function getOpdsSettings(): Promise<ReturnData<OpdsSettings>> {
  return get<OpdsSettings>('/getOpdsSettings')
}

/** POST /reader3/saveOpdsSettings：配置/禁用 OPDS 独立账号（username 空 = 禁用） */
export function saveOpdsSettings(username: string, password: string): Promise<ReturnData<{ enabled: boolean; username?: string }>> {
  return post<{ enabled: boolean; username?: string }>('/saveOpdsSettings', { username, password })
}
