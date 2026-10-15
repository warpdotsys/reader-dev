/** 已有 token 时显式选择身份；匿名初始化保留配置探测契约，不伪造 token。 */
export function tokenAuthenticationParams(accessToken: string | null | undefined): Record<string, string> {
  return accessToken ? { accessToken, readerAuth: 'access-token' } : {}
}

/** 旧按账号命名的缓存可能由共享 Cookie 错归属，保留但不能直接复用。 */
export const TOKEN_AUTH_CACHE_SCOPE = 'token-auth-v1'

/** 仅为本站 Reader API 路径加凭据；重新显示时使用当前 token，不复用 URL 中的旧 token。 */
export function authenticatedReaderUrl(path: string, accessToken: string | null | undefined): string {
  if (!path.startsWith('/reader3/')) return path
  const url = new URL(path, 'https://reader-api.invalid')
  // URL 会规范化 ../、编码点段和反斜杠；不能只信任规范化前的前缀。
  if (url.origin !== 'https://reader-api.invalid' || !url.pathname.startsWith('/reader3/')) return path
  url.searchParams.delete('accessToken')
  // 已退出的旧资源 URL 也不能通过另一页 Cookie 获得身份。
  url.searchParams.set('readerAuth', 'access-token')
  for (const [name, value] of Object.entries(tokenAuthenticationParams(accessToken))) url.searchParams.set(name, value)
  return url.pathname + url.search + url.hash
}
