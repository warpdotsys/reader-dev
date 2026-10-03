/** 原版位置是 iframe 内的像素位置，不能与宿主页面/其他用户的滚动位置混用。 */
export function epubProgressKey(namespace: string, bookUrl: string): string {
  return `reader-epub-progress-${encodeURIComponent(JSON.stringify([namespace, bookUrl]))}`
}

export function epubScrollPosition(raw: string | null, chapterUrl: string): number {
  if (!raw) return 0
  try {
    const value = JSON.parse(raw) as { chapterUrl?: unknown; scrollY?: unknown }
    return value?.chapterUrl === chapterUrl && typeof value.scrollY === 'number'
      && Number.isFinite(value.scrollY) && value.scrollY >= 0 ? value.scrollY : 0
  } catch {
    return 0
  }
}
