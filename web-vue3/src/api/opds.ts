import { get, post } from './request'
import type { ReturnData } from '@/types'

export interface OpdsSettings {
  enabled: boolean
  username: string
  passwordSet: boolean
}

/** 绑定当前登录用户；独立密码不回传，也不写浏览器持久缓存。 */
export function getOpdsSettings(): Promise<ReturnData<OpdsSettings>> {
  return get<OpdsSettings>('/getOpdsSettings', undefined, { silent: true })
}

/** 空用户名禁用；相同用户名的空密码保留服务端已有密码。 */
export function saveOpdsSettings(username: string, password: string): Promise<ReturnData<OpdsSettings>> {
  return post<OpdsSettings>('/saveOpdsSettings', { username, password }, { silent: true })
}
