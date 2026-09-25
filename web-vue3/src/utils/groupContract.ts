/**
 * Translation between the Rust Vue 3 group's UI model and the legacy
 * Java/Kotlin BookGroup + Book.group bit-mask contract.
 */

export interface GroupViewModel {
  id: number
  name: string
  cover?: string | null
  show?: boolean
  order?: number
  orderNum?: number
  bookCount?: number
  [key: string]: unknown
}

export interface LegacyBookGroupMembership {
  group?: unknown
  groupIds?: unknown
  origin?: unknown
  type?: unknown
  canUpdate?: unknown
  lastCheckError?: unknown
}

export function fromLegacyBookGroup(value: Record<string, unknown>): GroupViewModel {
  const rawId = value.groupId ?? value.id
  const id = typeof rawId === 'number' ? rawId : Number(rawId)
  if (!Number.isSafeInteger(id)) throw new Error('后端返回的分组 ID 不是安全整数')

  const rawName = value.groupName ?? value.name
  const orderValue = value.order ?? value.orderNum
  const order = typeof orderValue === 'number' ? orderValue : Number(orderValue ?? 0)
  const rawCount = value.bookCount
  const bookCount = typeof rawCount === 'number' ? rawCount : undefined

  return {
    ...value,
    id,
    name: typeof rawName === 'string' ? rawName : '',
    cover: typeof value.cover === 'string' ? value.cover : null,
    show: value.show !== false,
    order: Number.isFinite(order) ? order : 0,
    bookCount,
  }
}

export function toLegacyBookGroup(value: {
  id?: number
  name: string
  cover?: string | null
  show?: boolean
  order?: number
  orderNum?: number
}): Record<string, unknown> {
  const result: Record<string, unknown> = {
    groupId: value.id ?? 0,
    groupName: value.name,
  }
  const order = value.order ?? value.orderNum
  if (order !== undefined) result.order = order
  if (value.cover !== undefined) result.cover = value.cover
  if (value.show !== undefined) result.show = value.show
  return result
}

export function toLegacyBookGroupOrder(
  order: { id: number; orderNum: number }[],
): { groupId: number; order: number }[] {
  return order.map((item) => ({ groupId: item.id, order: item.orderNum }))
}

function parseGroupMask(value: unknown): bigint {
  if (typeof value === 'bigint') {
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
      throw new Error('书籍分组掩码超出浏览器可安全处理的整数范围')
    }
    return value
  }
  if (typeof value === 'number') {
    if (!Number.isSafeInteger(value)) throw new Error('书籍分组掩码超出浏览器可安全处理的整数范围')
    return BigInt(value)
  }
  if (typeof value === 'string' && /^-?\d+$/.test(value.trim())) {
    const mask = BigInt(value.trim())
    if (mask > BigInt(Number.MAX_SAFE_INTEGER)) {
      throw new Error('书籍分组掩码超出浏览器可安全处理的整数范围')
    }
    return mask
  }
  return 0n
}

/** Legacy Book.group is a Long bit mask; each user group ID is one bit. */
export function decodeGroupMask(value: unknown): number[] {
  let mask = parseGroupMask(value)
  if (mask <= 0n) return []

  const ids: number[] = []
  while (mask !== 0n) {
    const bit = mask & -mask
    const id = Number(bit)
    if (!Number.isFinite(id) || BigInt(id) !== bit) {
      throw new Error('分组 ID 超出浏览器可安全处理的整数范围')
    }
    ids.push(id)
    mask &= mask - 1n
  }
  return ids
}

export function encodeGroupMask(groupIds: number[]): number {
  let mask = 0n
  for (const id of new Set(groupIds)) {
    if (!Number.isSafeInteger(id) || id <= 0) throw new Error('无效的用户分组 ID')
    const bit = BigInt(id)
    if ((bit & (bit - 1n)) !== 0n) throw new Error('旧版书籍分组 ID 必须是单个二进制位')
    mask |= bit
  }

  const result = Number(mask)
  if (!Number.isFinite(result) || BigInt(result) !== mask) {
    throw new Error('组合后的分组掩码超出浏览器可安全处理的整数范围')
  }
  return result
}

/** Special negative group IDs are legacy shelf filters, not membership bits. */
export function isInLegacyBookGroup(book: LegacyBookGroupMembership, groupId: number): boolean {
  const ids = Array.isArray(book.groupIds)
    ? book.groupIds.filter((id): id is number => typeof id === 'number' && Number.isSafeInteger(id) && id > 0)
    : decodeGroupMask(book.group)
  switch (groupId) {
    case -1:
      return true
    case -2:
      return book.origin === 'loc_book'
    case -3:
      return book.type === 1
    case -4:
    case 0:
      return ids.length === 0
    case -5:
      return book.canUpdate === true && typeof book.lastCheckError === 'string' && book.lastCheckError.length > 0
    default:
      return ids.includes(groupId)
  }
}
