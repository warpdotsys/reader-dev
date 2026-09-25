import { get, post } from './request'
import type { Book, BookGroup, ReturnData } from '@/types'
import {
  decodeGroupMask,
  encodeGroupMask,
  fromLegacyBookGroup,
  toLegacyBookGroup,
  toLegacyBookGroupOrder,
} from '@/utils/groupContract'

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

/** GET /reader3/getBookGroups：把 legacy groupId/groupName/order 映射为 Vue 3 的 id/name/order。 */
export async function getBookGroups(): Promise<ReturnData<BookGroup[]>> {
  const result = await get<Record<string, unknown>[]>('/getBookGroups')
  const data = (result.data ?? []).map(fromLegacyBookGroup).sort((a, b) => (a.order ?? 0) - (b.order ?? 0))
  return { ...result, data }
}

/**
 * POST /reader3/saveBookGroup：新建 / 重命名分组
 * body {id?, name, order?}：id 缺省或 <=0 自动新建；id>0 按 id 覆盖（重命名）。
 */
export async function saveBookGroup(group: {
  id?: number
  name: string
  order?: number
  orderNum?: number
  /** 分组封面 URL（legacy BookGroup.cover） */
  cover?: string | null
  /** 分组是否显示（legacy BookGroup.show；false=隐藏，不出现在分组栏） */
  show?: boolean
}): Promise<ReturnData<BookGroup>> {
  const existingGroups = await getBookGroups()
  const existing = group.id === undefined
    ? undefined
    : existingGroups.data.find((item) => item.id === group.id)
  const beforeIds = new Set(existingGroups.data.map((item) => item.id))
  const draft = {
    ...group,
    cover: group.cover !== undefined ? group.cover : existing?.cover,
    show: group.show !== undefined ? group.show : existing?.show,
    order: group.order ?? group.orderNum ?? existing?.order ?? existing?.orderNum,
  }
  const result = await post<unknown>('/saveBookGroup', toLegacyBookGroup(draft))
  const after = await getBookGroups()
  const savedGroup = group.id === undefined
    ? after.data.find((item) => item.name === group.name && !beforeIds.has(item.id))
    : after.data.find((item) => item.id === group.id)
  if (!savedGroup) throw new Error('分组请求已成功，但重新读取后未找到保存结果')
  return { ...result, data: savedGroup }
}

/**
 * POST /reader3/deleteBookGroup：legacy CURD 接收 {groupId}。
 */
export function deleteBookGroup(id: number, opts?: { silent?: boolean }): Promise<ReturnData<null>> {
  return post<null>('/deleteBookGroup', { groupId: id }, opts)
}

/**
 * POST /reader3/saveBookGroupId：legacy 单组接口使用 groupId 字段，值写入 Book.group。
 */
export function updateBookGroupId(bookUrl: string, group: number): Promise<ReturnData<null>> {
  return post<null>('/saveBookGroupId', { bookUrl, groupId: group })
}

async function findBooks(bookUrls: string[]): Promise<Book[]> {
  const urls = Array.from(new Set(bookUrls))
  if (urls.length === 0) return []
  const result = await getBookshelf()
  const byUrl = new Map((result.data ?? []).map((book) => [book.bookUrl, book]))
  const missing = urls.filter((url) => !byUrl.has(url))
  if (missing.length) throw new Error(`这些书已不在书架中：${missing.length} 本，请刷新后重试`)
  return urls.map((url) => byUrl.get(url)!)
}

function currentGroupIds(book: Book): number[] {
  if (Array.isArray(book.groupIds)) {
    return Array.from(new Set(book.groupIds.filter((id) => Number.isSafeInteger(id) && id > 0)))
  }
  return decodeGroupMask(book.group)
}

type GroupMutationResult = ReturnData<{ count: number }>

async function mutateBookGroup(
  route: '/addBookGroupMulti' | '/removeBookGroupMulti',
  books: Book[],
  groupId: number,
  opts?: { silent?: boolean },
): Promise<GroupMutationResult> {
  encodeGroupMask([groupId]) // Validate the legacy group bit before sending it.
  if (!books.length) return { isSuccess: true, errorMsg: '', data: { count: 0 } }
  const result = await post<unknown>(
    route,
    { groupId, bookList: books.map((book) => ({ bookUrl: book.bookUrl })) },
    opts,
  )
  return { ...result, data: { count: books.length } }
}

/**
 * Legacy stores memberships as a Long bit mask. Reconcile via its actual
 * add/remove routes; there is no /setBookGroups endpoint in YueduApi.
 */
export async function setBookGroups(
  bookUrl: string,
  groupIds: number[],
): Promise<ReturnData<{ groupIds: number[]; group: number }>> {
  const [book] = await findBooks([bookUrl])
  const nextIds = Array.from(new Set(groupIds)).sort((a, b) => a - b)
  const nextMask = encodeGroupMask(nextIds)
  const previousIds = currentGroupIds(book)
  for (const groupId of previousIds.filter((id) => !nextIds.includes(id))) {
    await mutateBookGroup('/removeBookGroupMulti', [book], groupId)
  }
  for (const groupId of nextIds.filter((id) => !previousIds.includes(id))) {
    await mutateBookGroup('/addBookGroupMulti', [book], groupId)
  }
  return { isSuccess: true, errorMsg: '', data: { groupIds: nextIds, group: nextMask } }
}

/**
 * POST /reader3/addBookGroupMulti：批量追加分组（body {bookUrls, groupId}；
 * 后端未实现时调用方降级逐本 addBookGroup）
 */
export function addBookGroupMulti(
  bookUrls: string[],
  groupId: number,
  opts?: { silent?: boolean },
): Promise<GroupMutationResult> {
  return findBooks(bookUrls).then((books) =>
    mutateBookGroup(
      '/addBookGroupMulti',
      books.filter((book) => !currentGroupIds(book).includes(groupId)),
      groupId,
      opts,
    ),
  )
}

/**
 * POST /reader3/removeBookGroupMulti：body {bookList:[{bookUrl}], groupId}。
 * Legacy remove uses XOR, so only books currently carrying this bit are sent.
 * If groupId is omitted, clear each actual bit separately (the backend rejects 0).
 */
export function removeBookGroupMulti(
  bookUrls: string[],
  groupId?: number,
  opts?: { silent?: boolean },
): Promise<GroupMutationResult> {
  return findBooks(bookUrls).then(async (books) => {
    if (groupId !== undefined) {
      return mutateBookGroup(
        '/removeBookGroupMulti',
        books.filter((book) => currentGroupIds(book).includes(groupId)),
        groupId,
        opts,
      )
    }

    const affected = books.filter((book) => currentGroupIds(book).length > 0)
    const groupIds = Array.from(new Set(affected.flatMap(currentGroupIds)))
    for (const id of groupIds) {
      const members = affected.filter((book) => currentGroupIds(book).includes(id))
      await mutateBookGroup('/removeBookGroupMulti', members, id, opts)
    }
    return { isSuccess: true, errorMsg: '', data: { count: affected.length } }
  })
}

/** Single-book helpers share the legacy multi-book contract. */
export function addBookGroup(bookUrl: string, groupId: number): Promise<GroupMutationResult> {
  return addBookGroupMulti([bookUrl], groupId)
}

export function removeBookGroup(bookUrl: string, groupId: number): Promise<GroupMutationResult> {
  return removeBookGroupMulti([bookUrl], groupId)
}

/**
 * POST /reader3/saveBookGroupOrder：legacy body {order:[{groupId,order}]}
 * GAP 13：分组管理弹窗拖拽排序后保存。
 */
export function saveBookGroupOrder(order: { id: number; orderNum: number }[]): Promise<ReturnData<string>> {
  return post<string>('/saveBookGroupOrder', { order: toLegacyBookGroupOrder(order) })
}
