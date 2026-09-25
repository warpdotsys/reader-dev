import { get, post } from './request'
import type { Book, BookGroup, ReturnData } from '@/types'

/** GET /reader3/getBookshelf（refresh=1 时先回写 can_update=1 的书最新章/总数） */
export function getBookshelf(refresh = false): Promise<ReturnData<Book[]>> {
  return get<Book[]>('/getBookshelf', refresh ? { refresh: 1 } : undefined)
}

/** POST /reader3/saveBook：入架/编辑（body = 完整 Book JSON，upsert） */
export function saveBook(book: Book): Promise<ReturnData<null>> {
  return post<null>('/saveBook', book)
}

/**
 * GAP 78：POST /reader3/refreshLocalBook：重扫本地书（local:// 重解析原文件；
 * loc_book/storage 文件书重解析）——书架长按菜单「重新扫描」入口。
 */
export function refreshLocalBook(
  url: string,
): Promise<ReturnData<{ bookUrl?: string; name?: string; chapterCount?: number; totalChapterNum?: number } | null>> {
  return post<{ bookUrl?: string; name?: string; chapterCount?: number; totalChapterNum?: number } | null>(
    '/refreshLocalBook',
    { url },
  )
}

/** POST /reader3/deleteBook：移出书架（bookUrl） */
export function deleteBook(bookUrl: string): Promise<ReturnData<null>> {
  return post<null>('/deleteBook', { bookUrl })
}

/**
 * POST /reader3/deleteBooks：批量移出书架（body { bookUrls: string[] }）。
 * 后端并行实现中（可能 404）：调用方传 { silent: true } 并降级逐本 deleteBook。
 */
export function deleteBooks(bookUrls: string[], opts?: { silent?: boolean }): Promise<ReturnData<{ count?: number } | null>> {
  return post<{ count?: number } | null>('/deleteBooks', { bookUrls }, opts)
}

/** GET /reader3/getBookGroups：书架分组列表（契约：data [{id,name,orderNum,bookCount}]；后端当前输出 order） */
export function getBookGroups(): Promise<ReturnData<BookGroup[]>> {
  return get<BookGroup[]>('/getBookGroups')
}

/**
 * POST /reader3/saveBookGroup：新建 / 重命名分组
 * body {id?, name, order?}：id 缺省或 <=0 自动新建；id>0 按 id 覆盖（重命名）。
 */
export function saveBookGroup(group: {
  id?: number
  name: string
  order?: number
  /** 分组封面 URL（legacy BookGroup.cover） */
  cover?: string | null
  /** 分组是否显示（legacy BookGroup.show；false=隐藏，不出现在分组栏） */
  show?: boolean
}): Promise<ReturnData<BookGroup>> {
  return post<BookGroup>('/saveBookGroup', group)
}

/**
 * POST /reader3/deleteBookGroup：删除分组（body {id}，组内书置未分组）。
 * 后端并行实现中（可能 404）：调用方传 { silent: true } 自行降级提示。
 */
export function deleteBookGroup(id: number, opts?: { silent?: boolean }): Promise<ReturnData<null>> {
  return post<null>('/deleteBookGroup', { id }, opts)
}

/**
 * POST /reader3/updateBookGroupId：书设分组
 * 注意：后端实现读 body/query 的 group（数值分组 id）写入 books.group_name，
 * 并非字面上的 groupId 字段。
 */
export function updateBookGroupId(bookUrl: string, group: number): Promise<ReturnData<null>> {
  return post<null>('/updateBookGroupId', { bookUrl, group })
}

/**
 * POST /reader3/setBookGroups：多分组设置（body {bookUrl, groupIds:[...]}；
 * groupIds 空数组 = 移入未分组；与 legacy saveBookGroupId 位掩码语义一致）
 */
export function setBookGroups(bookUrl: string, groupIds: number[]): Promise<ReturnData<{ groupIds: number[] }>> {
  return post<{ groupIds: number[] }>('/setBookGroups', { bookUrl, groupIds })
}

/** POST /reader3/addBookGroup：往书籍追加一个分组（body {bookUrl, groupId}） */
export function addBookGroup(bookUrl: string, groupId: number): Promise<ReturnData<null>> {
  return post<null>('/addBookGroup', { bookUrl, groupId })
}

/** POST /reader3/removeBookGroup：从书籍移除一个分组（body {bookUrl, groupId}） */
export function removeBookGroup(bookUrl: string, groupId: number): Promise<ReturnData<null>> {
  return post<null>('/removeBookGroup', { bookUrl, groupId })
}

/**
 * POST /reader3/addBookGroupMulti：批量追加分组（body {bookUrls, groupId}；
 * 后端未实现时调用方降级逐本 addBookGroup）
 */
export function addBookGroupMulti(
  bookUrls: string[],
  groupId: number,
  opts?: { silent?: boolean },
): Promise<ReturnData<{ count?: number } | null>> {
  return post<{ count?: number } | null>('/addBookGroupMulti', { bookUrls, groupId }, opts)
}

/**
 * POST /reader3/removeBookGroupMulti：批量移出分组（body {bookUrls, groupId?}；
 * groupId 缺省 = 清空全部多分组；后端未实现时调用方降级逐本 removeBookGroup）
 */
export function removeBookGroupMulti(
  bookUrls: string[],
  groupId?: number,
  opts?: { silent?: boolean },
): Promise<ReturnData<{ count?: number } | null>> {
  return post<{ count?: number } | null>('/removeBookGroupMulti', { bookUrls, groupId }, opts)
}

/**
 * POST /reader3/saveBookGroupOrder：分组排序（body {order:[{id,orderNum}]}，orderNum 为新的序号）
 * GAP 13：分组管理弹窗拖拽排序后保存。
 */
export function saveBookGroupOrder(order: { id: number; orderNum: number }[]): Promise<ReturnData<{ count: number }>> {
  return post<{ count: number }>('/saveBookGroupOrder', { order })
}
