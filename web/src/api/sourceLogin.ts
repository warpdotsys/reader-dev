import { get, post, type RequestOptions } from './request'
import type { CookieRow, ReturnData } from '@/types'

/** loginBookSource / submitCaptcha 软性结果（data 内；成功/验证码/手动 Cookie/失败 四态） */
export interface BookSourceLoginResult {
  /** loginBookSource 成功标记 */
  success?: boolean
  /** submitCaptcha 成功标记 */
  isLogin?: boolean
  /** 需要图片验证码（captchaUrl 为图片 URL 或 data URI，captchaId 提交时回传） */
  needCaptcha?: boolean
  /** 需要手动 Cookie（点击类验证码无法自动处理） */
  needManualCaptcha?: boolean
  captchaUrl?: string
  captchaId?: string
  message?: string
  cookie?: string
}

/** POST /reader3/getCaptcha 探测结果 */
export interface CaptchaProbe {
  /** image | slider | click | none */
  captchaType?: string
  captchaUrl?: string
  captchaId?: string
  pageUrl?: string
  message?: string
}

/** POST/GET /reader3/loginBookSource：书源登录（body/query：bookSource、username、password、captcha 可选、mode=browser 可选） */
export function loginBookSource(params: {
  bookSource: string
  username?: string
  password?: string
  captcha?: string
  mode?: string
}): Promise<ReturnData<BookSourceLoginResult>> {
  return post<BookSourceLoginResult>('/loginBookSource', params)
}

/** POST /reader3/setBookSourceCookie：手动设置书源 cookie（cookie 为空 = 清除） */
export function setBookSourceCookie(
  bookSource: string,
  cookie: string,
): Promise<ReturnData<{ success: boolean; cleared?: boolean }>> {
  return post<{ success: boolean; cleared?: boolean }>('/setBookSourceCookie', { bookSource, cookie })
}

/** GET/POST /reader3/getBookSourceCookie：读取当前用户全部书源登录态（Cookie 管理） */
export function getBookSourceCookie(): Promise<ReturnData<CookieRow[]>> {
  return get<CookieRow[]>('/getBookSourceCookie', undefined, { silent: true })
}

/** POST /reader3/getCaptcha：探测验证码（image → captchaUrl 为 data URI，可直接显示；探测失败静默，调用方降级） */
export function getCaptcha(bookSource: string, opts?: RequestOptions): Promise<ReturnData<CaptchaProbe>> {
  return post<CaptchaProbe>('/getCaptcha', { bookSource }, opts)
}

/** POST /reader3/submitCaptcha：图片验证码文本提交（浏览器流，带 username/password 覆盖会话值）→ {isLogin} */
export function submitCaptcha(params: {
  bookSource: string
  captchaId: string
  captchaText: string
  username?: string
  password?: string
}): Promise<ReturnData<BookSourceLoginResult>> {
  return post<BookSourceLoginResult>('/submitCaptcha', params)
}
