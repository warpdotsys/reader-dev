import { get } from './request'
import type { ReturnData, SystemInfo } from '@/types'

/**
 * 系统信息 + 书源导出
 *
 * GET /reader3/getSystemInfo      → ReturnData<SystemInfo>（版本/端口/用户数/书数/书源数）
 * 书源导出使用 Java/Kotlin 已注册的 getBookSources 等价实现，在浏览器内组装
 * JSON Blob；服务端没有 exportBookSources 下载端点。
 */

/** GET /reader3/getSystemInfo */
export function getSystemInfo(): Promise<ReturnData<SystemInfo>> {
  return get<SystemInfo>('/getSystemInfo')
}

/**
 * 当前命名空间书源 JSON。getBookSources 的鉴权、命名空间和权限检查仍由后端
 * 执行；仅下载封装在浏览器完成，避免调用不存在的 exportBookSources 路由。
 */
export async function exportBookSources(): Promise<Blob> {
  const response = await get<unknown[]>('/getBookSources')
  return new Blob([JSON.stringify(response.data ?? [], null, 2)], { type: 'application/json;charset=utf-8' })
}
