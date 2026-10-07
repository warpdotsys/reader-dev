/** 缺失/空设置使用默认值；显式保存的 0（例如静音）仍按原值保留。 */
export function parseReaderNumericSetting(stored: string | null, min: number, max: number, fallback: number, step = 1): number {
  if (stored === null || !stored.trim()) return fallback
  const raw = Number(stored)
  if (!Number.isFinite(raw) || raw < min || raw > max) return fallback
  return Math.round(raw / step) * step
}
