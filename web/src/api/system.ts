import request from './request'
import type { ReturnData, SystemInfo } from '@/types'

/**
 * 系统信息 + 书源导出
 *
 * GET /reader3/getSystemInfo      → ReturnData<SystemInfo>（版本/端口/用户数/书数/书源数）
 * GET /reader3/exportBookSources  → 当前命名空间书源 JSON 附件下载
 */

/** GET /reader3/getSystemInfo */
export function getSystemInfo(): Promise<ReturnData<SystemInfo>> {
  return request.get('/getSystemInfo').then((r) => r.data as ReturnData<SystemInfo>)
}

/**
 * GET /reader3/exportBookSources：当前命名空间书源 JSON，返回 Blob。
 * 成功：application/json 的书源数组（文件名 bookSource.json）；
 * 失败：legacy ReturnData JSON 错误体（由调用方经 utils/download.ts downloadBlob 识别提示）。
 */
export function exportBookSources(): Promise<Blob> {
  return request
    .get('/exportBookSources', { responseType: 'blob', timeout: 60_000 })
    .then((r) => r.data as Blob)
}
