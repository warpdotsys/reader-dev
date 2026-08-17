/**
 * 搜索历史（localStorage，最近 10 条；ExploreView 下拉提示与 SearchView 历史共用）
 */
const HISTORY_KEY = 'reader_search_history'
const HISTORY_MAX = 10

/** 读取最近 10 条（解析失败/不可用 → 空数组） */
export function loadSearchHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (!raw) return []
    const arr = JSON.parse(raw)
    return Array.isArray(arr) ? arr.filter((x): x is string => typeof x === 'string') : []
  } catch {
    return []
  }
}

/** 压入一条（去重置顶，截断 10 条）并写回；返回新列表 */
export function pushSearchHistory(word: string): string[] {
  const next = [word, ...loadSearchHistory().filter((h) => h !== word)].slice(0, HISTORY_MAX)
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(next))
  } catch {
    // localStorage 不可用时静默降级
  }
  return next
}

/** 清空历史 */
export function clearSearchHistory(): void {
  try {
    localStorage.removeItem(HISTORY_KEY)
  } catch {
    // 忽略
  }
}
