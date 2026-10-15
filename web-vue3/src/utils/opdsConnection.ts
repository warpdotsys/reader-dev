const ATOM_NS = 'http://www.w3.org/2005/Atom'
const CATALOG_LIMIT = 1024 * 1024

export function validateOpdsAccount(username: string, password: string, previous: {
  enabled: boolean; username: string; passwordSet: boolean
}): string {
  if (!username) return ''
  if (!/^[A-Za-z0-9_.-]{3,64}$/.test(username)) return '账号须为 3–64 位字母、数字、下划线、点或连字符'
  if (!password && previous.enabled && previous.passwordSet && previous.username === username) return ''
  if (password.length < 12 || password.length > 128 || /[\x00-\x1f\x7f-\x9f]/.test(password)) {
    return '密码须为 12–128 位，不能包含控制字符'
  }
  return ''
}

/** 从部署地址产生纯目录地址；永不复制 Reader token 或 Basic 密码。 */
export function opdsCatalogUrl(origin: string): string {
  const url = new URL('/reader3/opds/catalog', origin)
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) throw new Error('OPDS 地址无效')
  return url.href
}

export function validateOpdsFeed(xml: string): void {
  if (/<!DOCTYPE|<!ENTITY/i.test(xml)) throw new Error('目录包含不允许的 XML 声明')
  const doc = new DOMParser().parseFromString(xml, 'application/xml')
  const root = doc.documentElement
  const fields = ['id', 'title', 'updated']
  if (doc.getElementsByTagName('parsererror').length || root?.localName !== 'feed' || root.namespaceURI !== ATOM_NS ||
      fields.some(name => !Array.from(root.children).some(child => child.localName === name &&
        child.namespaceURI === ATOM_NS && child.textContent?.trim()))) {
    throw new Error('响应不是有效的 OPDS Atom 目录')
  }
}

/** Basic 仅发送同部署目录路由；不附 Cookie、不跟重定向、不回退主账号 token。 */
export async function probeOpdsCatalog(origin: string, username: string, password: string,
  signal: AbortSignal): Promise<void> {
  const url = new URL(opdsCatalogUrl(origin))
  const loopback = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname)
  if (url.protocol !== 'https:' && !loopback) throw new Error('非本机 OPDS 连接必须使用 HTTPS')
  const error = validateOpdsAccount(username, password, { enabled: false, username: '', passwordSet: false })
  if (!username || error) throw new Error(error || '请先配置并输入 OPDS 独立账号密码')
  const bytes = new TextEncoder().encode(`${username}:${password}`)
  const authorization = `Basic ${btoa(String.fromCharCode(...bytes))}`
  const response = await fetch(url.href, { headers: { Authorization: authorization, Accept: 'application/atom+xml' },
    credentials: 'omit', redirect: 'error', cache: 'no-store', signal })
  if (!response.ok || !/^application\/atom\+xml(?:\s*;|$)/i.test(response.headers.get('content-type') || '')) {
    await response.body?.cancel()
    if (response.status === 401 || response.status === 403) throw new Error('OPDS 独立账号认证未通过')
    throw new Error(`服务器未返回 OPDS 目录（HTTP ${response.status}）`)
  }
  const reader = response.body?.getReader()
  if (!reader) throw new Error('OPDS 目录为空')
  const chunks: Uint8Array[] = []
  let size = 0
  try {
    while (true) {
      const chunk = await reader.read()
      if (chunk.done) break
      size += chunk.value.byteLength
      if (size > CATALOG_LIMIT) throw new Error('OPDS 目录超出连接测试大小上限')
      chunks.push(chunk.value)
    }
    const bytes = new Uint8Array(size)
    let offset = 0
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength }
    validateOpdsFeed(new TextDecoder('utf-8', { fatal: true }).decode(bytes))
  } finally {
    await reader.cancel().catch(() => {})
    reader.releaseLock()
  }
}
