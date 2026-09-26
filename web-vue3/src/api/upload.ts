import request from './request'
import type { Book, BookChapter, ReturnData } from '@/types'

/**
 * Java/Kotlin 的本地书导入预览结果。
 *
 * `/importBookPreview` 不是纯预览接口：它先把 multipart 文件保存到当前用户的
 * `storage/assets/<namespace>/book`，再返回可直接提交给 `/saveBook` 的完整 Book。
 * 因此前端必须保留这个 Book，确认时调用 saveBook；不能再请求不存在的
 * `/uploadLocalBook`。
 */
export interface LocalBookImportPreview {
  book: Book
  chapters: BookChapter[]
}

/**
 * POST /reader3/importBookPreview（multipart/form-data，字段名可为 file、file0 等）：
 * 保存并解析本地书，返回 `[{ book, chapters }]`。调用方确认后将 `book` POST 到
 * `/saveBook` 才会入书架。
 */
export function importBookPreview(file: File): Promise<ReturnData<LocalBookImportPreview[]>> {
  const form = new FormData()
  form.append('file', file)
  return request
    .post('/importBookPreview', form, { timeout: 120_000, silent: true })
    .then((r) => r.data as ReturnData<LocalBookImportPreview[]>)
}

/**
 * 丢弃尚未入书架的本地书预览文件。
 * 这是 legacy `/deleteFile` 的受限用户资产清理接口；已成功 saveBook 的文件不能调用。
 */
export function discardImportPreview(preview: LocalBookImportPreview): Promise<ReturnData<unknown> | null> {
  const url = preview.book.bookUrl
  if (!url.startsWith('/assets/')) return Promise.resolve(null)
  return request
    .post('/deleteFile', { url }, { timeout: 30_000, silent: true })
    .then((r) => r.data as ReturnData<unknown>)
}
