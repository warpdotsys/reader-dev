/** Stable identity rules for TXT TOC rules returned by the Java/Kotlin backend. */
export function isDefaultTxtTocRuleId(id: unknown): boolean {
  if (typeof id === 'number') return Number.isFinite(id) && id < 0
  if (typeof id !== 'string') return false

  const value = id.trim()
  if (value.startsWith('default-')) return true
  return /^-\d+$/.test(value)
}

export function txtTocRuleKey(id: unknown): string {
  return String(id ?? '')
}
