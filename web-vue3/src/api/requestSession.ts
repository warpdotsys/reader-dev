/** 短期内存身份：不写入持久化缓存、日志或服务端请求参数。 */
export interface RequestSessionState {
  accessToken: string
  username: string
  isAdmin: boolean
  defaultConfigMode: boolean
  sessionRevision: number
}

export type RequestSessionSnapshot = Readonly<Pick<RequestSessionState,
  'accessToken' | 'username' | 'sessionRevision'> & { namespace: string }>

function requestNamespace(store: RequestSessionState): string {
  return store.isAdmin && store.defaultConfigMode ? 'default' : store.username
}

export function captureRequestSession(store: RequestSessionState): RequestSessionSnapshot {
  return Object.freeze({
    accessToken: store.accessToken,
    username: store.username,
    namespace: requestNamespace(store),
    sessionRevision: store.sessionRevision,
  })
}

export function isRequestSessionCurrent(
  snapshot: RequestSessionSnapshot | undefined,
  store: RequestSessionState,
): boolean {
  return !!snapshot && snapshot.sessionRevision === store.sessionRevision &&
    snapshot.accessToken === store.accessToken && snapshot.username === store.username &&
    snapshot.namespace === requestNamespace(store)
}

/** 未登录本就没有会话可过期，不能因匿名 401 使并行登录响应失效。 */
export function canExpireRequestSession(
  snapshot: RequestSessionSnapshot | undefined,
  store: RequestSessionState,
): boolean {
  return !!store.accessToken && isRequestSessionCurrent(snapshot, store)
}

/** 旧请求仍然失败，但不清新会话、不跳登录、不弹旧账号的全局错误。 */
export class StaleSessionResponseError extends Error {
  readonly code = 'READER_STALE_SESSION'
  constructor() {
    super('请求所属登录会话已改变')
    this.name = 'StaleSessionResponseError'
  }
}
