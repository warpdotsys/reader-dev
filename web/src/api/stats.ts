import { get } from './request'
import type { ReturnData } from '@/types'

/**
 * 阅读统计 —— 后端契约（并行实现中）
 *
 * GET /reader3/getReadingStats → ReturnData<ReadingStats>
 *   后端实现形态（storage::ReadingStats，camelCase）：
 *     {
 *       today:  <秒>,                       // 今日阅读秒数
 *       week:   <秒>,                       // 近 7 天阅读秒数
 *       total:  <秒>,                       // 累计阅读秒数
 *       books:  [{ bookUrl, name, seconds, chars }]   // 单书汇总（按秒数降序）
 *     }
 *   契约描述形态（兼容读取）：today/week/total 为 {count,minutes,books} 对象，TOP 列表字段 topBooks。
 *
 * 说明：接口未实现（404）时调用方 silent 降级——用本地阅读进度（reader-progress-*）
 * 计算近似统计并标注「本地统计」。
 */

/** 单个时间窗统计（契约形态；秒数形态由调用方归一化） */
export interface ReadingStatsItem {
  count: number
  minutes: number
  books: number
  [key: string]: unknown
}

/** 单书统计项（后端 books[] / 契约 topBooks[] 兼容） */
export interface ReadingTopBook {
  name: string
  bookUrl?: string
  /** 累计阅读秒数 */
  seconds?: number
  /** 累计阅读字数 */
  chars?: number
  count?: number
  minutes?: number
  [key: string]: unknown
}

export interface ReadingStats {
  /** 秒数（后端）或 {count,minutes,books}（契约） */
  today: ReadingStatsItem | number
  week: ReadingStatsItem | number
  total: ReadingStatsItem | number
  books?: ReadingTopBook[]
  topBooks?: ReadingTopBook[]
  [key: string]: unknown
}

/** GET /reader3/getReadingStats（silent：未实现时调用方降级本地统计） */
export function getReadingStats(): Promise<ReturnData<ReadingStats>> {
  return get<ReadingStats>('/getReadingStats', undefined, { silent: true })
}
