import type { Book } from '@/types'

/**
 * GAP 78：本地书判定（书架「重新扫描」入口可用性——纯函数，便于单测）。
 * 后端 refreshLocalBook 支持三类本地书：local:// 双轨书、loc_book 文件书（legacy）、storage/ 路径文件书。
 */

/** 是否 local:// 双轨仓书 */
export function isLocalUrl(url: string): boolean {
  return url.startsWith('local://')
}

/** 是否 loc_book 文件书（legacy 文件引用模式） */
export function isLocBook(origin: string | null | undefined): boolean {
  return origin === 'loc_book'
}

/** 是否 storage/ 路径文件书（老导入路径形态） */
export function isStoragePath(url: string): boolean {
  return url.startsWith('storage/')
}

const FILE_EXTENSIONS = ['txt', 'epub', 'md', 'fb2', 'azw3', 'mobi']

/** 是否文件扩展名书（含大写——后端大小写不敏感） */
export function isFileExtension(url: string): boolean {
  const lower = url.toLowerCase()
  return FILE_EXTENSIONS.some((e) => lower.endsWith(`.${e}`))
}

/** 书架书可否「重新扫描」（refreshLocalBook 支持范围） */
export function canRescanBook(book: Pick<Book, 'bookUrl' | 'origin'>): boolean {
  return isLocalUrl(book.bookUrl) || isLocBook(book.origin) || isStoragePath(book.bookUrl) || isFileExtension(book.bookUrl)
}
