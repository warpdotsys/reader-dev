/**
 * GAP 110：每日阅读时长累计（本机）。
 *
 * 阅读页每 30s（可见且正在阅读）向当日桶累计秒数，存 localStorage（reader_daily_stats）
 * 的 {"YYYY-MM-DD": seconds} 映射；统计弹窗据此渲染「近 7 天每日时长」纯 CSS 柱状图。
 * 后端 getReadingStats 无每日维度，故每日数据以本机累计为准（后端就绪后可扩展）。
 */

export const DAILY_STATS_KEY = 'reader_daily_stats'

/** 本地时区日期键（YYYY-MM-DD） */
export function localDateStr(d: Date): string {
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`
}

/** 解析存储中的每日时长表（损坏/缺省/非正数 → 丢弃，返回干净表） */
export function parseDailyStats(raw: string | null): Record<string, number> {
  if (!raw) return {}
  try {
    const o = JSON.parse(raw) as unknown
    if (!o || typeof o !== 'object' || Array.isArray(o)) return {}
    const out: Record<string, number> = {}
    for (const [k, v] of Object.entries(o as Record<string, unknown>)) {
      const n = Number(v)
      if (/^\d{4}-\d{2}-\d{2}$/.test(k) && Number.isFinite(n) && n > 0) out[k] = n
    }
    return out
  } catch {
    return {}
  }
}

/** 累计当日时长（返回新表；非正秒数忽略；顺带清理 31 天前的过期条目防无限增长） */
export function accumulateDaily(
  map: Record<string, number>,
  seconds: number,
  now: Date = new Date(),
): Record<string, number> {
  if (!Number.isFinite(seconds) || seconds <= 0) return map
  const out = { ...map }
  const k = localDateStr(now)
  out[k] = (out[k] ?? 0) + Math.round(seconds)
  // 清理 31 天前的过期条目（YYYY-MM-DD 字典序比较等价于日期比较，规避时区边界）
  const cutoffDate = localDateStr(new Date(now.getTime() - 31 * 86400000))
  for (const key of Object.keys(out)) {
    if (key < cutoffDate) delete out[key]
  }
  return out
}

/** 近 7 天每日秒数（从 6 天前到今天，缺省日 = 0） */
export function last7Days(
  map: Record<string, number>,
  now: Date = new Date(),
): { date: string; seconds: number }[] {
  const out: { date: string; seconds: number }[] = []
  for (let i = 6; i >= 0; i--) {
    const d = new Date(now)
    d.setDate(d.getDate() - i)
    const k = localDateStr(d)
    out.push({ date: k, seconds: Math.max(0, Math.round(map[k] ?? 0)) })
  }
  return out
}
