/**
 * 分组拖拽重排（GAP 13）：把 fromId 分组移到 toId 分组所在位置（目标行之前）。
 *
 * P2 修复：目标索引必须在移除源分组后重新定位——直接使用移除前的 toIdx 在
 * 「向下拖拽」场景会偏移一位（落点跑到目标行之后）。纯函数，可单测。
 */

export interface GroupLike {
  id: number
}

/** 返回重排后的新数组（不修改入参）；from/to 任一不存在时返回原数组副本 */
export function moveGroupTo<T extends GroupLike>(list: T[], fromId: number, toId: number): T[] {
  const out = list.slice()
  const fromIdx = out.findIndex((x) => x.id === fromId)
  const toIdx = out.findIndex((x) => x.id === toId)
  if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) return out
  const [moved] = out.splice(fromIdx, 1)
  // 目标索引在移除后重新定位（直接使用移除前的 toIdx 在向下拖拽时会偏移一位）
  const targetIdx = out.findIndex((x) => x.id === toId)
  out.splice(targetIdx, 0, moved)
  return out
}
