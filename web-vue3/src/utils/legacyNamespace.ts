import type { LocalCacheUser } from './readerLocalCache'

export class NamespaceProofError extends Error {
  readonly code = 'READER_NAMESPACE_UNVERIFIED'
  readonly data = 'NEED_SECURE_KEY'
  constructor() { super('未确认系统配置空间，请重新验证管理密码'); this.name = 'NamespaceProofError' }
}

/** 全局管理/状态接口没有账号数据空间，不能要求它们返回空间凭据。 */
const GLOBAL_APIS = new Set(['/login', '/logout', '/getUserInfo', '/getUserList', '/addUser', '/updateUser',
  '/deleteUsers', '/resetPassword', '/clearInactiveUsers', '/getSystemInfo', '/getLicense',
  '/importLicense', '/decryptLicense', '/supplyLicense'])

export interface LegacyNamespaceContext {
  params: Record<string, string>
  headers: Record<string, string>
  systemNamespace: boolean
}

/** ns 是部分配置接口的配置键；实际管理空间必须走 legacy userNS + 请求头密钥。 */
export function legacyNamespaceContext(user: LocalCacheUser, key: string): LegacyNamespaceContext {
  const params: Record<string, string> = user.accessToken ? { accessToken: user.accessToken } : {}
  const headers: Record<string, string> = {}
  const systemNamespace = user.isAdmin && user.defaultConfigMode
  if (systemNamespace) {
    if (!key) throw new NamespaceProofError()
    params.userNS = 'default'
    headers['X-Reader-Secure-Key'] = key
  }
  return { params, headers, systemNamespace }
}

/** legacy 错误管理密钥会静默回退本人空间；只接受后端实际空间确认，不能缓存错归属数据。 */
export function assertLegacyNamespace(context: Pick<LegacyNamespaceContext, 'systemNamespace'>, path: string, proof: string | null | undefined): void {
  const api = path.split('?')[0].replace(/^\/reader3/, '')
  if (context.systemNamespace && !GLOBAL_APIS.has(api) && proof !== 'default') throw new NamespaceProofError()
}
