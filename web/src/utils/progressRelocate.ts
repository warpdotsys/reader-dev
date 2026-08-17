import type { BookChapter } from '@/types'

/**
 * GAP 6：换源后阅读位置重定位（纯函数——便于单测）
 *
 * 换源后新书源的章节目录与旧源不同：旧 durChapterIndex 可能越界或指向错误章节。
 * 注意 durChapterIndex 是「原始目录数组」下标（目录可能含卷标行 isVolume=true，
 * 阅读器内真实章节为过滤卷标后的平铺列表）。
 *
 * 策略（按优先级）：
 *   1. 旧索引在原始目录范围内且非卷标行 → 原样保留（多数换源章节数相近）；
 *   2. 旧章标题（durChapterTitle）在非卷标章中精确命中 → 用命中章原始下标（标题跨源稳定）；
 *   3. 兜底：就近钳制——从 min(旧索引, 末章) 向前找最近的非卷标章（不越界、不落卷标行）。
 *
 * @param oldIndex 换源前的 durChapterIndex（<0 视为无进度）
 * @param oldTitle 换源前的 durChapterTitle（可为空）
 * @param toc      新书源的章节目录（getBookToc 返回，原始数组含卷标行）
 * @returns 重定位后的原始目录下标；无进度或目录为空时返回 -1（调用方不动服务端进度）
 */
export function relocateChapterIndex(
  oldIndex: number,
  oldTitle: string | null | undefined,
  toc: BookChapter[],
): number {
  if (typeof oldIndex !== 'number' || !Number.isFinite(oldIndex) || oldIndex < 0) return -1
  if (toc.length === 0) return -1

  // 1) 旧索引直接可用（范围内且非卷标行）
  if (oldIndex < toc.length && !toc[oldIndex].isVolume) return oldIndex

  // 2) 标题精确匹配（跨源同名章；只匹配真实章节）
  const title = (oldTitle ?? '').trim()
  if (title) {
    const hit = toc.findIndex((c) => !c.isVolume && (c.title ?? '').trim() === title)
    if (hit >= 0) return hit
  }

  // 3) 就近钳制：从 min(旧索引, 末章) 向前找最近的非卷标章
  let i = Math.min(oldIndex, toc.length - 1)
  while (i >= 0 && toc[i].isVolume) i -= 1
  return i >= 0 ? i : -1
}
