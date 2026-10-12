/** 路由只能选择已有 backend 章节；EPUB 小节锚点不伪造另一个 index。 */
export interface ReaderRouteLocation {
  chapter: unknown
  anchor: unknown
}

export interface ReaderRouteNavigation {
  kind: 'chapter' | 'anchor'
  index: number
  fragment: string | null
}

export function readerRouteNavigation(next: ReaderRouteLocation, previous: ReaderRouteLocation,
  currentIndex: number, chapters: ReadonlyArray<{ isVolume?: boolean }>, rawEpub: boolean): ReaderRouteNavigation | null {
  let index = currentIndex
  if (next.chapter !== undefined) {
    if (typeof next.chapter !== 'string' || !/^\d+$/.test(next.chapter)) return null
    index = Number(next.chapter)
  }
  if (!Number.isSafeInteger(index) || index < 0 || index >= chapters.length || !chapters[index] || chapters[index]?.isVolume) return null
  if (index !== currentIndex) {
    return { kind: 'chapter', index, fragment: rawEpub && typeof next.anchor === 'string' ? next.anchor : null }
  }
  if (!rawEpub || next.anchor === previous.anchor) return null
  if (next.anchor !== undefined && next.anchor !== null && typeof next.anchor !== 'string') return null
  // goToChapter 已改变 index，随后自己同步 ?chapter 并删除旧锚点；
  // 这不是一次外部同章锚点请求，不能再覆盖 pending fragment/滚动位置。
  if (next.chapter !== previous.chapter && (next.anchor === undefined || next.anchor === null)) return null
  return { kind: 'anchor', index, fragment: typeof next.anchor === 'string' ? next.anchor : null }
}
