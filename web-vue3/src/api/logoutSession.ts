import { captureRequestSession, isRequestSessionCurrent } from './requestSession.ts'
import type { RequestSessionSnapshot, RequestSessionState } from './requestSession.ts'
import type { ReturnData } from '../types/index.ts'

export type LogoutOutcome = 'revoked' | 'local-only' | 'superseded'

/** 捕获退出身份再发送；旧退出完成不得清除或跳转新登录会话。 */
export async function logoutSession(
  store: RequestSessionState & { clear(): void },
  revoke: (session: RequestSessionSnapshot) => Promise<ReturnData<unknown>>,
): Promise<LogoutOutcome> {
  const session = captureRequestSession(store)
  let outcome: LogoutOutcome = 'local-only'
  if (session.accessToken) {
    try {
      const result = await revoke(session)
      // 原 JAR 的正常退出契约不是普通 data=null，也不能用 HTTP 200 当撤销证明。
      if (result?.isSuccess === true && result.errorMsg === '' && result.data === 'NEED_LOGIN') outcome = 'revoked'
    } catch { /* 仍退出本机，但必须告知服务端撤销尚未确认 */ }
  }
  if (!isRequestSessionCurrent(session, store)) return 'superseded'
  store.clear()
  return outcome
}
