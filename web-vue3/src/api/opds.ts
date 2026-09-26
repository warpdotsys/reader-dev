import type { ReturnData } from '@/types'

/**
 * Java/Kotlin 兼容边界：当前恢复的服务端没有 OPDS 路由，也没有独立 OPDS
 * 账号配置的持久化模型。不能把普通登录凭据伪装成独立 OPDS 设置。
 *
 * 视图仍可展示地址，但调用这两个配置接口会得到明确错误；待服务端实现完整
 * OPDS 认证和命名空间隔离后再接入真实路由。
 */

export interface OpdsSettings {
  enabled: boolean
  username: string
  passwordSet: boolean
  namespace?: string
}

const OPDS_SETTINGS_UNAVAILABLE = '当前 Java/Kotlin 服务端未实现 OPDS 独立账号配置'

function unavailable<T>(): Promise<ReturnData<T>> {
  return Promise.reject(new Error(OPDS_SETTINGS_UNAVAILABLE))
}

/** 当前后端未提供安全的 OPDS 配置读取能力。 */
export function getOpdsSettings(): Promise<ReturnData<OpdsSettings>> {
  return unavailable<OpdsSettings>()
}

/** 当前后端未提供安全的 OPDS 配置写入能力。 */
export function saveOpdsSettings(username: string, password: string): Promise<ReturnData<{ enabled: boolean; username?: string }>> {
  // 保留形参以维持视图调用签名；禁止将密码写入任意替代存储。
  void username
  void password
  return unavailable<{ enabled: boolean; username?: string }>()
}
