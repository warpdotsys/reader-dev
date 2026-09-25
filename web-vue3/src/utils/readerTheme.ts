/**
 * 阅读页自定义主题：背景色 / 文字色 / 强调色 三个颜色 + 派生完整 CSS 变量集。
 * 存储：localStorage reader_theme_custom（JSON { bg, text, accent }，#rrggbb）。
 * 使用：ReaderView 主题选择器第 5 档「自定义」——theme==='custom' 时把
 * customThemeVars() 的变量注入 .reader-page 内联样式，覆盖内置主题变量。
 */

export interface ReaderCustomTheme {
  /** 背景色 */
  bg: string
  /** 正文文字色 */
  text: string
  /** 强调色 */
  accent: string
}

export const CUSTOM_THEME_KEY = 'reader_theme_custom'

/** 默认自定义主题：米白纸色 + 深褐文字 + 棕金强调（暖色系） */
export const CUSTOM_THEME_DEFAULTS: ReaderCustomTheme = {
  bg: '#f4f1ea',
  text: '#2f2b26',
  accent: '#8a6d3b',
}

/* ---------------- 颜色工具 ---------------- */

function clamp255(n: number): number {
  return Math.min(255, Math.max(0, Math.round(n)))
}

/** '#rgb' / '#rrggbb' → [r,g,b]；非法返回 null */
export function parseHexColor(hex: string): [number, number, number] | null {
  const m = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex.trim())
  if (!m) return null
  const s = m[1]
  if (s.length === 3) {
    return [parseInt(s[0] + s[0], 16), parseInt(s[1] + s[1], 16), parseInt(s[2] + s[2], 16)]
  }
  return [parseInt(s.slice(0, 2), 16), parseInt(s.slice(2, 4), 16), parseInt(s.slice(4, 6), 16)]
}

function toHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((v) => clamp255(v).toString(16).padStart(2, '0')).join('')}`
}

/** 相对亮度（0 黑 – 1 白，WCAG 近似） */
export function hexLuminance(hex: string): number {
  const rgb = parseHexColor(hex)
  if (!rgb) return 0.5
  const [r, g, b] = rgb.map((v) => {
    const c = v / 255
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
  })
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}

/** 两色按比例混合：ratio=0 全 a，ratio=1 全 b */
function mixHex(a: string, b: string, ratio: number): string {
  const ca = parseHexColor(a)
  const cb = parseHexColor(b)
  if (!ca || !cb) return a
  return toHex(
    ca[0] + (cb[0] - ca[0]) * ratio,
    ca[1] + (cb[1] - ca[1]) * ratio,
    ca[2] + (cb[2] - ca[2]) * ratio,
  )
}

function rgbaHex(hex: string, alpha: number): string {
  const c = parseHexColor(hex)
  if (!c) return `rgba(0, 0, 0, ${alpha})`
  return `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${alpha})`
}

/* ---------------- 存取 ---------------- */

function isValidColor(v: unknown): v is string {
  return typeof v === 'string' && parseHexColor(v) !== null
}

/** 读取自定义主题（缺失/损坏回退默认） */
export function loadCustomTheme(): ReaderCustomTheme {
  try {
    const raw = JSON.parse(localStorage.getItem(CUSTOM_THEME_KEY) ?? '')
    if (raw && typeof raw === 'object') {
      const bg = isValidColor(raw.bg) ? raw.bg : CUSTOM_THEME_DEFAULTS.bg
      const text = isValidColor(raw.text) ? raw.text : CUSTOM_THEME_DEFAULTS.text
      const accent = isValidColor(raw.accent) ? raw.accent : CUSTOM_THEME_DEFAULTS.accent
      return { bg, text, accent }
    }
  } catch {
    /* ignore */
  }
  return { ...CUSTOM_THEME_DEFAULTS }
}

export function saveCustomTheme(t: ReaderCustomTheme) {
  try {
    localStorage.setItem(CUSTOM_THEME_KEY, JSON.stringify(t))
  } catch {
    /* ignore */
  }
}

/** 背景是否深色（自定义主题的 data-reader-theme 基座取 dark/light） */
export function customThemeIsDark(t: ReaderCustomTheme): boolean {
  return hexLuminance(t.bg) < 0.4
}

/**
 * 自定义主题 → .reader-page 完整 CSS 变量集（覆盖内置主题变量）。
 * 派生规则：surface/card 略提亮；hover/border 按背景明暗反向微调；
 * text-2/text-3 由文字色向背景色混合降级；accent-soft 用强调色透明底；
 * on-accent 按强调色亮度取黑白保证对比。
 */
export function customThemeVars(t: ReaderCustomTheme): Record<string, string> {
  const dark = customThemeIsDark(t)
  const surface = dark ? mixHex(t.bg, '#ffffff', 0.07) : mixHex(t.bg, '#ffffff', 0.5)
  return {
    '--bg': t.bg,
    '--bg-float': rgbaHex(t.bg, 0.86),
    '--hover': dark ? mixHex(t.bg, '#ffffff', 0.06) : mixHex(t.bg, '#000000', 0.03),
    '--surface': surface,
    '--card': surface,
    '--border': dark ? mixHex(t.bg, '#ffffff', 0.1) : mixHex(t.bg, '#000000', 0.07),
    '--border-strong': dark ? mixHex(t.bg, '#ffffff', 0.16) : mixHex(t.bg, '#000000', 0.13),
    '--text-1': t.text,
    '--text-2': mixHex(t.text, t.bg, 0.45),
    '--text-3': mixHex(t.text, t.bg, 0.68),
    '--accent': t.accent,
    '--accent-deep': dark ? mixHex(t.accent, '#ffffff', 0.1) : mixHex(t.accent, '#000000', 0.12),
    '--accent-soft': rgbaHex(t.accent, dark ? 0.16 : 0.08),
    '--on-accent': hexLuminance(t.accent) > 0.5 ? '#1a1a1a' : '#ffffff',
  }
}
