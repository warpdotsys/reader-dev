import { useUserStore } from '@/stores/user'
import { managerCredentialScope, readManagerCredential, readerDeployment } from '@/utils/managerCredential'
import { assertLegacyNamespace, legacyNamespaceContext } from '@/utils/legacyNamespace'
import { captureRequestSession, isRequestSessionCurrent, StaleSessionResponseError } from './requestSession'

/** 同步捕获请求空间与短期身份，原生 fetch 与 axios 使用相同规则。 */
export function readerRequestContext() {
  const store = useUserStore()
  const context = legacyNamespaceContext(store, readManagerCredential(managerCredentialScope(store, readerDeployment())))
  const session = captureRequestSession(store)
  return {
    ...context,
    isCurrent: () => isRequestSessionCurrent(session, useUserStore()),
    assertResponse(response: Response, path: string) {
      try {
        if (!isRequestSessionCurrent(session, useUserStore())) throw new StaleSessionResponseError()
        assertLegacyNamespace(context, path, response.headers.get('X-Reader-Namespace'))
      } catch (error) {
        // 拒绝响应时同时停止尚未读取的原生流，不留下无句柄的后台下载。
        void response.body?.cancel().catch(() => {})
        throw error
      }
    },
  }
}
