/**
 * 书源分组胶囊排序（SourceManageView）——顺序持久化辅助（纯函数，可单测）。
 *
 * 后端无书源分组排序接口（BookSource 模型只有 bookSourceGroup 文本 token 与
 * 单个书源的 weight/customOrder，无分组级顺序字段）——按契约降级为本地持久：
 * localStorage 键 reader_source_group_order（string[]，分组名有序数组）。
 */

export const SOURCE_GROUP_ORDER_KEY = 'reader_source_group_order'

export interface StorageLike {
  getItem(key: string): string | null
  setItem(key: string, value: string): void
  removeItem?(key: string): void
}

/** 读取已保存的分组顺序（非法/缺失返回空数组） */
export function loadGroupOrder(storage: StorageLike): string[] {
  try {
    const raw = storage.getItem(SOURCE_GROUP_ORDER_KEY)
    if (!raw) return []
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed.filter((x): x is string => typeof x === 'string' && x.length > 0)
  } catch {
    return []
  }
}

/** 持久化分组顺序 */
export function persistGroupOrder(order: string[], storage: StorageLike): void {
  try {
    storage.setItem(SOURCE_GROUP_ORDER_KEY, JSON.stringify(order))
  } catch {
    /* 存储不可用时仅内存顺序 */
  }
}

/** 清空已保存的顺序（回退默认排序） */
export function clearGroupOrder(storage: StorageLike): void {
  try {
    storage.removeItem?.(SOURCE_GROUP_ORDER_KEY)
  } catch {
    /* ignore */
  }
}

/**
 * 合并：已保存顺序（过滤掉已不存在的分组）+ 未收录的新分组按出现顺序追加。
 * 无已保存顺序时返回 null（调用方回退默认排序，如按名称）。
 */
export function mergeGroupOrder(saved: string[], existing: string[]): string[] | null {
  if (saved.length === 0) return null
  const set = new Set(existing)
  const out = saved.filter((g) => set.has(g))
  for (const g of existing) {
    if (!out.includes(g)) out.push(g)
  }
  return out
}

/** 拖拽重排：把 from 移到 to 的位置（其余相对顺序不变；任一不在列表返回原列表） */
export function reorderGroup(list: string[], from: string, to: string): string[] {
  const fromIdx = list.indexOf(from)
  const toIdx = list.indexOf(to)
  if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) return list
  const out = list.slice()
  const [moved] = out.splice(fromIdx, 1)
  out.splice(toIdx, 0, moved)
  return out
}
