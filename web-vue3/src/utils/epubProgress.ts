import { epubFragment } from './epubLoader.ts'

/** 原版位置是 iframe 内的像素位置，不能与宿主页面/其他用户的滚动位置混用。 */
export function epubProgressKey(namespace: string, bookUrl: string): string {
  return `reader-epub-progress-${encodeURIComponent(JSON.stringify([namespace, bookUrl]))}`
}

/** null 表示没有可恢复的位置，应定位目录锚点；保存过的 0 是有效页顶位置。 */
export function epubScrollPosition(raw: string | null, chapterUrl: string,
  fragment = epubFragment(chapterUrl)): number | null {
  if (!raw) return null
  try {
    const value = JSON.parse(raw) as { chapterUrl?: unknown; scrollY?: unknown; fragment?: unknown }
    const savedFragment = typeof value?.fragment === 'string' ? value.fragment : epubFragment(chapterUrl)
    return value?.chapterUrl === chapterUrl && savedFragment === fragment && typeof value.scrollY === 'number'
      && Number.isFinite(value.scrollY) && value.scrollY >= 0 ? value.scrollY : null
  } catch {
    return null
  }
}
