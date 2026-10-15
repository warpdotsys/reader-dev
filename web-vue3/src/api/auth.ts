import { post } from './request'
import { logoutSession } from './logoutSession'
import { useUserStore } from '@/stores/user'
import { tokenAuthenticationParams } from '@/utils/tokenAuthentication'
import { notifyBackendReachable } from './backendFlag'
import type { ReturnData, UserInfo } from '@/types'

export interface LoginParams {
  username: string
  password: string
  /** true=登录，false=注册（自动注册） */
  isLogin: boolean
  code?: string
}

/** POST /reader3/login */
export function login(params: LoginParams): Promise<ReturnData<UserInfo>> {
  return post<UserInfo>('/login', params)
}

/** 退出使用已捕获身份，不经过会在异步拦截器中读取新账号的公共请求实例。 */
export function logout() {
  return logoutSession(useUserStore(), async (session) => {
    const controller = new AbortController()
    const timeout = setTimeout(() => controller.abort(), 8000)
    try {
      const query = new URLSearchParams(tokenAuthenticationParams(session.accessToken))
      const response = await fetch('/reader3/logout?' + query, {
        method: 'POST', credentials: 'same-origin', cache: 'no-store',
        headers: { Accept: 'application/json' }, signal: controller.signal,
      })
      notifyBackendReachable()
      if (!response.ok) throw new Error('服务端退出未确认')
      return await response.json() as ReturnData<unknown>
    } finally { clearTimeout(timeout) }
  })
}
