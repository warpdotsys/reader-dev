import type { CookieRow } from '../types'

/** Never display any credential value, including a prefix of a manual cookie. */
export function redactSourceCookiePreview(raw: unknown): string {
  if (typeof raw !== 'string' || !raw.trim()) return '内容已隐藏'
  const names = raw.split(';').filter(part => part.includes('='))
    .map(part => part.slice(0, part.indexOf('=')).trim())
    .filter(name => /^[!#$%&'*+\-.^_`|~0-9a-zA-Z]+$/.test(name))
  return [...new Set(names)].map(name => `${name}=***`).join('; ').slice(0, 256) || '内容已隐藏'
}

/** Only the current authenticated server response can establish cookie presence. */
export function savedSourceCookieRows(input: unknown): CookieRow[] {
  if (!Array.isArray(input)) throw new TypeError('Cookie 状态响应不是数组')
  const rows = new Map<string, CookieRow>()
  for (const row of input) {
    if (!row || typeof row !== 'object' || row.hasCookie !== true ||
        typeof row.sourceUrl !== 'string' || !row.sourceUrl.trim()) continue
    const preview = redactSourceCookiePreview(row.cookiePreview || row.cookie)
    rows.set(row.sourceUrl, {
      sourceUrl: row.sourceUrl, hasCookie: true, cookie: preview, cookiePreview: preview,
      // The listing is a status surface, not a place to echo authentication headers.
      userAgent: '', loginHeader: '',
      updatedAt: typeof row.updatedAt === 'number' && Number.isFinite(row.updatedAt) ? row.updatedAt : 0,
    })
  }
  return [...rows.values()]
}
