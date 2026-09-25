import type { ReplaceRule } from '@/types'

/** Shape persisted by the Java/Kotlin ReplaceRule data class. */
export interface LegacyReplaceRule {
  id: number
  name: string
  group: string | null
  pattern: string
  replacement: string
  scope: string | null
  scopeTitle: boolean
  scopeContent: boolean
  isEnabled: boolean
  isRegex: boolean
  timeoutMillisecond: number
  order: number
}

function asString(value: unknown, fallback = ''): string {
  return typeof value === 'string' ? value : fallback
}

function asBoolean(value: unknown, fallback: boolean): boolean {
  return typeof value === 'boolean' ? value : fallback
}

function asFiniteNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback
}

/**
 * Legacy IDs are Long values. Existing preview-only IDs such as `filter-...`
 * are converted deterministically into a positive, JavaScript-safe integer so
 * they can be sent through Vert.x/Jackson without a deserialization failure.
 */
export function legacyReplaceRuleId(value: unknown, identity: string): number {
  if (typeof value === 'number' && Number.isSafeInteger(value) && value > 0) return value
  if (typeof value === 'string' && /^\d+$/.test(value)) {
    const parsed = Number(value)
    if (Number.isSafeInteger(parsed) && parsed > 0) return parsed
  }

  let hash = 2166136261
  for (const char of identity || String(value ?? 'replace-rule')) {
    hash = Math.imul(hash ^ char.charCodeAt(0), 16777619)
  }
  // Keep generated values disjoint from current epoch-millisecond IDs while
  // staying below Number.MAX_SAFE_INTEGER and within Kotlin Long's range.
  return 1 + ((hash >>> 0) % 1_000_000_000_000)
}

export function newReplaceRuleId(now = Date.now(), random = Math.random()): string {
  const suffix = Math.max(0, Math.min(999, Math.floor(random * 1000)))
  return String(now * 1000 + suffix)
}

export function fromLegacyReplaceRule(value: unknown): ReplaceRule {
  const raw = value && typeof value === 'object' ? (value as Record<string, unknown>) : {}
  const name = asString(raw.name)
  const find = asString(raw.pattern, asString(raw.find))
  const replace = asString(raw.replacement, asString(raw.replace))
  const id = legacyReplaceRuleId(raw.id, `${name}\u0000${find}\u0000${replace}`)

  return {
    ...raw,
    id: String(id),
    name,
    find,
    replace,
    enabled: asBoolean(raw.isEnabled, asBoolean(raw.enabled, true)),
    order: asFiniteNumber(raw.order, 0),
  }
}

export function toLegacyReplaceRule(rule: ReplaceRule): LegacyReplaceRule {
  const id = legacyReplaceRuleId(rule.id, `${rule.name}\u0000${rule.find}\u0000${rule.replace}`)
  return {
    id,
    name: asString(rule.name),
    group: typeof rule.group === 'string' ? rule.group : null,
    pattern: asString(rule.find),
    replacement: asString(rule.replace),
    scope: typeof rule.scope === 'string' ? rule.scope : null,
    scopeTitle: asBoolean(rule.scopeTitle, false),
    scopeContent: asBoolean(rule.scopeContent, true),
    isEnabled: asBoolean(rule.enabled, true),
    isRegex: asBoolean(rule.isRegex, false),
    timeoutMillisecond: asFiniteNumber(rule.timeoutMillisecond, 3000),
    order: Math.trunc(asFiniteNumber(rule.order, 0)),
  }
}
