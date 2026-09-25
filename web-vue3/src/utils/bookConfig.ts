/**
 * GAP 6：每本书独立阅读配置（per-book override）。
 * localStorage：reader_book_config_{bookUrl} —— JSON 对象，键 = reader_* 键名（与全局键一致），
 * 值为字符串（形态与全局 localStorage 一致）。本书设置优先于全局；
 * 「恢复全局默认」= 删除该书配置键。纯函数便于 node --test 单测（localStorage 需 stub）。
 */

const PREFIX = 'reader_book_config_'

export function bookConfigKey(bookUrl: string): string {
  return `${PREFIX}${bookUrl}`
}

/** 读取本书配置（非法 JSON / 非对象 / 非字符串值一律忽略） */
export function loadBookConfig(bookUrl: string): Record<string, string> {
  try {
    const raw = localStorage.getItem(bookConfigKey(bookUrl))
    if (!raw) return {}
    const o: unknown = JSON.parse(raw)
    if (!o || typeof o !== 'object' || Array.isArray(o)) return {}
    const out: Record<string, string> = {}
    for (const [k, v] of Object.entries(o as Record<string, unknown>)) {
      if (typeof v === 'string') out[k] = v
    }
    return out
  } catch {
    return {}
  }
}

/** 整份覆盖写入本书配置 */
export function saveBookConfig(bookUrl: string, cfg: Record<string, string>) {
  try {
    localStorage.setItem(bookConfigKey(bookUrl), JSON.stringify(cfg))
  } catch {
    /* ignore */
  }
}

/** 删除本书配置；返回是否删除成功 */
export function clearBookConfig(bookUrl: string): boolean {
  try {
    localStorage.removeItem(bookConfigKey(bookUrl))
    return true
  } catch {
    return false
  }
}

/** 本书是否有任一覆盖项 */
export function hasBookConfig(bookUrl: string): boolean {
  return Object.keys(loadBookConfig(bookUrl)).length > 0
}
