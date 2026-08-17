/**
 * 界面主题（与阅读内容主题分离）：浅色 / 深色 / 跟随系统。
 *
 * 界面主题 = 书架/设置/书源/文件管理等应用 UI 的配色（html[data-theme=light|dark]），
 * 与阅读页内容主题（reader_theme，仅作用于 .reader-page 内，见 styles/main.css）互不影响。
 *
 * 持久化：localStorage（键 ui_theme，迁移自旧 reader_theme 全局行为）+ 服务器
 * （随 saveUserConfig 整量 JSON 的 ui_theme 键，多端一致）。
 */

export type UiTheme = 'light' | 'dark' | 'system'

export const UI_THEME_KEY = 'ui_theme'

/** 解析出实际渲染主题（system → 跟随系统深色偏好） */
export function resolveUiTheme(t: UiTheme): 'light' | 'dark' {
  if (t === 'system') {
    try {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    } catch {
      return 'light'
    }
  }
  return t
}

/** 从 localStorage 读取界面主题（缺失时按旧 reader_theme 迁移，默认浅色） */
export function loadUiTheme(): UiTheme {
  try {
    const raw = localStorage.getItem(UI_THEME_KEY)
    if (raw === 'light' || raw === 'dark' || raw === 'system') return raw
    // 迁移：旧版阅读主题曾全局生效——按其初始化界面主题，行为更平滑
    const old = localStorage.getItem('reader_theme')
    if (old === 'dark' || old === 'system') {
      localStorage.setItem(UI_THEME_KEY, old)
      return old
    }
  } catch {
    /* ignore */
  }
  return 'light'
}

/** 应用界面主题：html[data-theme]（Element Plus 变量随之切换）+ 移动端状态栏配色 + 持久化 */
export function applyUiTheme(t: UiTheme): void {
  const real = resolveUiTheme(t)
  document.documentElement.dataset.theme = real
  const meta = document.querySelector<HTMLMetaElement>('meta[name="theme-color"]')
  if (meta) meta.content = real === 'dark' ? '#1a1a1a' : '#fafafa'
  try {
    localStorage.setItem(UI_THEME_KEY, t)
  } catch {
    /* ignore */
  }
}

/** 服务器整量 JSON 中的 ui_theme 键 */
export function uiThemeToServer(t: UiTheme): Record<string, unknown> {
  return { ui_theme: t }
}

/** 从服务器配置 JSON 解析界面主题（无效值返回 undefined，不覆盖本地） */
export function uiThemeFromServer(raw: unknown): UiTheme | undefined {
  return raw === 'light' || raw === 'dark' || raw === 'system' ? raw : undefined
}
