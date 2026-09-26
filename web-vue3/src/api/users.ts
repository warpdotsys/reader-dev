import { get, post } from './request'
import type { RequestOptions } from './request'
import { useUserStore } from '@/stores/user'
import type { ReaderUser, ReturnData, UserUpdatePayload } from '@/types'

/**
 * 用户管理 API（与 Java/Kotlin legacy 的 UserController 对齐）
 *
 * GET  /reader3/getUserList      → ReturnData<ReaderUser[]>（secure 模式需管理密钥）
 * POST /reader3/updateUser       → body：username + enableWebdav/enableLocalStore/enableBookSource/
 *                                  enableRssSource/bookSourceLimit/bookLimit（缺省字段不修改）
 * POST /reader3/deleteUsers      → body：string[]（返回剩余用户列表）
 * POST /reader3/resetPassword    → body：username + password
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

/** 管理密钥走请求头；legacy query 参数只供旧客户端兼容，避免密钥进入 URL/访问日志。 */
function managerOptions(): RequestOptions | undefined {
  const key = getStoredSecureKey()
  if (!key) return undefined
  return { headers: { 'X-Reader-Secure-Key': key } }
}

/** GET /reader3/getUserList：用户列表（secure 模式缺 secureKey 时 reject，错误 data = 'NEED_SECURE_KEY'） */
export function getUsers(): Promise<ReturnData<ReaderUser[]>> {
  return get<ReaderUser[]>('/getUserList', undefined, managerOptions())
}

/** POST /reader3/updateUser：更新用户权限/上限，返回完整用户列表。 */
export function updateUser(payload: UserUpdatePayload): Promise<ReturnData<ReaderUser[]>> {
  return post<ReaderUser[]>('/updateUser', payload, managerOptions())
}

/** 单删复用 legacy 的 deleteUsers，后端只提供字符串数组请求体。 */
export function deleteUser(username: string): Promise<ReturnData<ReaderUser[]>> {
  return deleteUsers([username])
}

/** POST /reader3/deleteUsers：批量删除用户（返回剩余用户列表；不能删除自己） */
export function deleteUsers(usernames: string[]): Promise<ReturnData<ReaderUser[]>> {
  return post<ReaderUser[]>('/deleteUsers', usernames, managerOptions())
}

/** POST /reader3/clearInactiveUsers：清理后返回完整剩余用户列表。 */
export function clearInactiveUsers(
  inactiveDay: number,
): Promise<ReturnData<ReaderUser[]>> {
  return post<ReaderUser[]>(
    '/clearInactiveUsers',
    { inactiveDay },
    managerOptions(),
  )
}

/** POST /reader3/resetPassword：管理员重置密码（body username + password，data 为 ""）。 */
export function resetUserPassword(username: string, newPassword: string): Promise<ReturnData<unknown>> {
  return post('/resetPassword', { username, password: newPassword }, managerOptions())
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
 * POST /reader3/addUser：新增用户，返回完整用户列表；业务错误由调用方提示。
 * secure 模式同样需 secureKey（缺/错返回 NEED_SECURE_KEY，由调用方引导输入）。
 */
export function addUser(payload: AddUserPayload): Promise<ReturnData<ReaderUser[]>> {
  return post<ReaderUser[]>('/addUser', payload, { silent: true, ...managerOptions() })
}

/** 判断接口是否未实现（404/501/网络失败）——P3-A：收敛至 utils/errors（重导出保持兼容） */
export { isNotImplemented } from '@/utils/errors'

/**
 * 探测后端是否处于 secure 模式（决定书架导航「用户」入口是否显示）。
 * getUserList 无 secureKey 返回 NEED_SECURE_KEY ⇒ secure；其余（成功/404/网络错误）视为非 secure。
 * 已保存 secureKey 时顺带刷新当前用户的 isAdmin（管理员才显示入口）。
 * 走 fetch 而非 axios 实例，避免 404/业务错误触发全局 toast。
 */
export async function probeSecureMode(): Promise<boolean> {
  const store = useUserStore()
  try {
    const params = new URLSearchParams()
    if (store.accessToken) params.set('accessToken', store.accessToken)
    const key = getStoredSecureKey()
    params.set('_t', String(Date.now())) // 防 GET 缓存
    const res = await fetch(`/reader3/getUserList?${params.toString()}`, {
      method: 'GET',
      headers: key ? { 'X-Reader-Secure-Key': key } : undefined,
    })
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
