import type { ReturnData } from '@/types'

/**
 * 阅读统计没有可等价映射的 Java/Kotlin 路由。当前服务端只持久化章节进度，
 * 未记录时长/字数聚合；因此不能编造 getReadingStats 响应。
 * 调用方会捕获此明确错误并显示标注过的本地近似统计。
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

/** 当前后端没有阅读时长统计模型。 */
export function getReadingStats(): Promise<ReturnData<ReadingStats>> {
  return Promise.reject(new Error('当前 Java/Kotlin 服务端未实现阅读统计接口'))
}
