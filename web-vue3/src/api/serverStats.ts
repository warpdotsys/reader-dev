import type { ReturnData, ServerStats } from '@/types'

/**
 * 当前 Java/Kotlin 服务端仅提供 getSystemInfo，缺少请求计数、在线会话、
 * CPU 采样与书源成功率等 ServerStats 的必要字段。不得将不完整的系统信息
 * 伪装成监控数据。
 */

/** 当前后端未实现完整服务监控数据。 */
export function getServerStats(): Promise<ReturnData<ServerStats>> {
  return Promise.reject(new Error('当前 Java/Kotlin 服务端未实现服务监控接口'))
}
