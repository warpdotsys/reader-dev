import type { BookChapter } from '@/types'

/**
 * 目录预览条目构建（GAP 18/91）：前 max 章，卷标题行（isVolume）渲染为分隔行且不计入章数上限。
 *
 * P2 修复：章节数达到上限后立即停止——不再追加任何行（含后续分卷标题行，
 * 避免「分卷标题无限追加」）。index 为完整目录数组下标（与阅读页 ?chapter 语义一致）。
 * 纯函数，可单测。
 */

export interface TocEntry {
  kind: 'volume' | 'chapter'
  index: number
  title: string
}

export function buildTocEntries(
  chapters: BookChapter[],
  max: number,
  transform: (title: string) => string = (t) => t,
): TocEntry[] {
  const out: TocEntry[] = []
  let count = 0
  for (let i = 0; i < chapters.length; i++) {
    const c = chapters[i]
    if (count >= max) break // 截断后停止（分卷标题行也不再追加）
    if (c.isVolume) {
      out.push({ kind: 'volume', index: i, title: transform(c.title) })
      continue
    }
    count++
    out.push({ kind: 'chapter', index: i, title: transform(c.title) })
  }
  return out
}
