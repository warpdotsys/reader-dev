import { get, post } from './request'
import { useUserStore } from '@/stores/user'
import type { ReaderUser, ReturnData, UserUpdatePayload } from '@/types'

/**
 * 用户管理 API（后端并行实现中，接口可能 404——调用方需容错）
 *
 * GET  /reader3/getUsers         → ReturnData<ReaderUser[]>（secure 模式需 secure+secureKey query）
 * POST /reader3/updateUser       → body：username + enableWebdav/enableLocalStore/enableBookSource/
 *                                  enableRssSource/bookSourceLimit/bookLimit（缺省字段不修改）
 * POST /reader3/deleteUser       → body：username（不能删除自己）
 * POST /reader3/resetUserPassword → body：username + newPassword
 *
 * secure 模式约定（对齐后端 checkManagerAuth）：缺/错 secureKey 返回
 * { isSuccess:false, errorMsg:'请输入管理密码', data:'NEED_SECURE_KEY' }。
 */

const SECURE_KEY_STORAGE = 'reader_secure_key'

/** 读取已保存的 secureKey（sessionStorage：刷新页面仍可用，关闭标签页失效） */
export function getStoredSecureKey(): string {
  return sessionStorage.getItem(SECURE_KEY_STORAGE) || ''
}

/** 保存 secureKey 到 sessionStorage */
export function storeSecureKey(key: string): void {
  sessionStorage.setItem(SECURE_KEY_STORAGE, key)
}

/** 管理参数：已有 secureKey 时携带 secure+secureKey（非 secure 模式不携带，后端视为通过） */
function managerParams(): Record<string, unknown> | undefined {
  const key = getStoredSecureKey()
  if (!key) return undefined
  return { secure: 1, secureKey: key }
}

/** GET /reader3/getUsers：用户列表（secure 模式缺 secureKey 时 reject，错误 data = 'NEED_SECURE_KEY'） */
export function getUsers(): Promise<ReturnData<ReaderUser[]>> {
  return get<ReaderUser[]>('/getUsers', managerParams())
}

/** POST /reader3/updateUser：更新用户权限/上限（payload 缺省字段不修改） */
export function updateUser(payload: UserUpdatePayload): Promise<ReturnData<unknown>> {
  return post('/updateUser', payload, managerParams())
}

/** POST /reader3/deleteUser：删除用户（不能删除自己） */
export function deleteUser(username: string): Promise<ReturnData<unknown>> {
  return post('/deleteUser', { username }, managerParams())
}

/** POST /reader3/deleteUsers：批量删除用户（返回剩余用户列表；不能删除自己） */
export function deleteUsers(usernames: string[]): Promise<ReturnData<ReaderUser[]>> {
  return post<ReaderUser[]>('/deleteUsers', { usernames }, managerParams())
}

/** POST /reader3/clearInactiveUsers：清理 inactiveDay 天内未登录用户（返回删除列表） */
export function clearInactiveUsers(
  inactiveDay: number,
): Promise<ReturnData<{ deleted: string[]; count: number }>> {
  return post<{ deleted: string[]; count: number }>(
    '/clearInactiveUsers',
    { inactiveDay },
    managerParams(),
  )
}

/** POST /reader3/resetUserPassword：重置密码（body username + newPassword） */
export function resetUserPassword(username: string, newPassword: string): Promise<ReturnData<unknown>> {
  return post('/resetUserPassword', { username, newPassword }, managerParams())
}

/** 新增用户请求体（POST /reader3/addUser；后端并行实现中——404 时调用方降级 register） */
export interface AddUserPayload {
  username: string
  password: string
  enableWebdav?: boolean
  enableLocalStore?: boolean
  enableBookSource?: boolean
  enableRssSource?: boolean
  bookSourceLimit?: number
  bookLimit?: number
  isAdmin?: boolean
}

/**
 * POST /reader3/addUser：新增用户（silent——未实现时降级 register，业务错误由调用方提示）
 * secure 模式同样需 secureKey（缺/错返回 NEED_SECURE_KEY，由调用方引导输入）。
 */
export function addUser(payload: AddUserPayload): Promise<ReturnData<unknown>> {
  return post('/addUser', payload, { silent: true, params: managerParams() })
}

/** 判断接口是否未实现（404/501/网络失败）——P3-A：收敛至 utils/errors（重导出保持兼容） */
export { isNotImplemented } from '@/utils/errors'

/**
 * 探测后端是否处于 secure 模式（决定书架导航「用户」入口是否显示）。
 * getUsers 无 secureKey 返回 NEED_SECURE_KEY ⇒ secure；其余（成功/404/网络错误）视为非 secure。
 * 已保存 secureKey 时顺带刷新当前用户的 isAdmin（管理员才显示入口）。
 * 走 fetch 而非 axios 实例，避免 404/业务错误触发全局 toast。
 */
export async function probeSecureMode(): Promise<boolean> {
  const store = useUserStore()
  try {
    const params = new URLSearchParams()
    if (store.accessToken) params.set('accessToken', store.accessToken)
    const key = getStoredSecureKey()
    if (key) {
      params.set('secure', '1')
      params.set('secureKey', key)
    }
    params.set('_t', String(Date.now())) // 防 GET 缓存
    const res = await fetch(`/reader3/getUsers?${params.toString()}`, { method: 'GET' })
    if (!res.ok) return false
    const json = (await res.json()) as { isSuccess?: boolean; data?: unknown }
    if (json.data === 'NEED_SECURE_KEY') return true
    if (Array.isArray(json.data) && store.username) {
      const me = json.data.find((u) => (u as { username?: string })?.username === store.username)
      if (me) {
        store.setSession(
          store.accessToken,
          store.username,
          true,
          (me as { isAdmin?: boolean }).isAdmin === true,
        )
      }
    }
    return false
  } catch {
    return false
  }
}

/** 判断请求错误是否为 NEED_SECURE_KEY（secure 模式缺/错 secureKey） */
export function isNeedSecureKey(err: unknown): boolean {
  return (err as { data?: unknown } | null)?.data === 'NEED_SECURE_KEY'
}
