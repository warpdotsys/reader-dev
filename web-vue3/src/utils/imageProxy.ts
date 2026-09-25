/**
 * legacy imageProxy：设置页开关 reader_image_proxy（默认关）。
 * 开启后远端 http(s) 图片经后端 /assets/proxy 回源（复用书源登录态/UA/Referer 与
 * WebP 转换，防盗链与私网拦截由服务端统一处理）。
 */

const PROXY_KEY = 'reader_image_proxy'

export function imageProxyEnabled(): boolean {
  try {
    return localStorage.getItem(PROXY_KEY) === '1'
  } catch {
    return false
  }
}

export function setImageProxyEnabled(on: boolean): void {
  try {
    localStorage.setItem(PROXY_KEY, on ? '1' : '0')
  } catch {
    /* ignore */
  }
}

export function proxyImageUrl(url: string | null | undefined): string | null | undefined {
  if (!url || !/^https?:\/\//i.test(url)) return url
  if (!imageProxyEnabled()) return url
  return `/assets/proxy?url=${encodeURIComponent(url)}`
}
