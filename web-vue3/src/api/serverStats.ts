import request from './request'
import type { ReturnData, ServerStats } from '@/types'

/**
 * 服务监控
 *
 * GET /reader3/getServerStats → ReturnData<ServerStats>
 * （内存/CPU 短采样/请求计数/在线会话/书源成功率/uptime 聚合）
 */

/** GET /reader3/getServerStats */
export function getServerStats(): Promise<ReturnData<ServerStats>> {
  return request.get('/getServerStats').then((r) => r.data as ReturnData<ServerStats>)
}
