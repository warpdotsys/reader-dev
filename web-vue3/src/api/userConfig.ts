import { get, post } from './request'
import type { ReturnData } from '@/types'

/**
 * 用户配置同步 —— 后端契约（并行实现中，未就绪时 silent 降级为纯本地）
 *
 * GET  /reader3/getUserConfig → ReturnData<Record<string, unknown>>（当前用户配置 JSON）
 * POST /reader3/saveUserConfig → ReturnData<null>（body = 配置 JSON，整量覆盖）
 *
 * 用途：阅读偏好（简繁/宽度/字体/主题/行距等 reader_* 键）多端一致——
 * 设置页进入时 get 并与 localStorage 合并（服务器优先），保存时 post。
 */

/** GET /reader3/getUserConfig（silent：后端未实现时调用方降级，不弹全局提示） */
export function getUserConfig(): Promise<ReturnData<Record<string, unknown>>> {
  return get<Record<string, unknown>>('/getUserConfig', undefined, { silent: true })
}

/** POST /reader3/saveUserConfig（body = 配置 JSON；silent：后端未实现时调用方降级） */
export function saveUserConfig(config: Record<string, unknown>): Promise<ReturnData<null>> {
  return post<null>('/saveUserConfig', config, { silent: true })
}
