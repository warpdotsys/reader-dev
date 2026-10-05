import type { LocalCacheUser } from './readerLocalCache'

/** 旧共享 reader_secure_key 无法确定账号归属：不读取、不迁移。 */
const KEY = 'reader_manager_credential_v2'
type ManagerStorage = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>

export function readerDeployment(): string {
  return typeof window === 'undefined' ? '' : window.location.origin + (import.meta.env?.BASE_URL ?? '/')
}

export function managerCredentialScope(user: Pick<LocalCacheUser, 'accessToken' | 'username'>, deployment: string): string | null {
  return user.accessToken && user.username && deployment ? JSON.stringify([deployment, user.username]) : null
}

export function readManagerCredential(scope: string | null, storage?: ManagerStorage): string {
  if (!scope) return ''
  try {
    const value = JSON.parse((storage ?? globalThis.sessionStorage).getItem(KEY) ?? '')
    return value?.version === 2 && value.scope === scope && typeof value.key === 'string' ? value.key : ''
  } catch { return '' }
}

/** 保存待验证密钥，不代表获得管理能力；内存调用方需等真实成功响应。 */
export function saveManagerCredential(scope: string | null, key: string, storage?: ManagerStorage): boolean {
  if (!scope) return false
  try {
    (storage ?? globalThis.sessionStorage).setItem(KEY, JSON.stringify({ version: 2, scope, key }))
    return true
  } catch { return false }
}

/** 登录/退出使本标签页新版本凭据失效，不动归属未知的旧共享键。 */
export function discardManagerCredential(storage?: ManagerStorage): void {
  try { (storage ?? globalThis.sessionStorage).removeItem(KEY) } catch { /* unavailable */ }
}
