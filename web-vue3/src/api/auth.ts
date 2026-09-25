import { post } from './request'
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
