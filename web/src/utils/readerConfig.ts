/**
 * 阅读偏好配置：localStorage（reader_* 键）与服务器（GET/POST /reader3/getUserConfig|saveUserConfig）
 * 之间读写/合并的统一入口。阅读页（ReaderView）直接读写 localStorage 各键，
 * 设置页经此模块汇总为一份 JSON 上传 / 下发合并（服务器优先）。
 */

import { parsePageMode, type PageMode } from './readerPageMode'

export type { PageMode }

export type HanMode = 'auto' | 'simp' | 'trad'
/** 阅读内容主题（custom=自定义——颜色见 reader_theme_custom，仅阅读页可配置） */
export type Theme = 'light' | 'dark' | 'warm' | 'system' | 'custom'
export type TextAlign = 'left' | 'justify'
export type FontKind =
  | 'system'
  | 'song'
  | 'hei'
  | 'kai'
  | 'fangsong'
  | 'round'
  | 'lishu'
  | 'yahei'
  | 'pingfang'
  | 'wenkai'
  | 'hanserif'
  | 'serif'

/** 与阅读页各 localStorage 键一一对应（键名 = reader_* 后缀） */
export interface ReaderConfig {
  /** 简繁模式：auto=自动 / simp=简体 / trad=繁体（reader_han_mode） */
  hanMode: HanMode
  /** 主题：light/dark/warm/system/custom（reader_theme；旧值 paper 已迁移为 warm） */
  theme: Theme
  /** 正文字号（reader_font_size，14-22） */
  fontSize: number
  /** 行距（reader_line_height，1.5-2.5） */
  lineHeight: number
  /** 段距（reader_para_spacing，0.5-2） */
  paraSpacing: number
  /** 字重（reader_font_weight，300-500） */
  fontWeight: number
  /** 正文宽度（reader_content_width：720px/900px/1080px） */
  contentWidth: string
  /** 字体（reader_font_family，FontKind 字符串） */
  fontFamily: string
  /** 字距（reader_letter_spacing，0-2） */
  letterSpacing: number
  /** 首行缩进（reader_text_indent：'1'/'0'） */
  textIndent: boolean
  /** 对齐（reader_text_align：left/justify） */
  textAlign: TextAlign
  /** 翻页模式（reader_page_mode：scroll/slide） */
  pageMode: PageMode
}

export const READER_CONFIG_DEFAULTS: ReaderConfig = {
  hanMode: 'auto',
  theme: 'light',
  fontSize: 18,
  lineHeight: 1.9,
  paraSpacing: 1,
  fontWeight: 400,
  contentWidth: '900px',
  fontFamily: 'system',
  letterSpacing: 0,
  textIndent: true,
  textAlign: 'left',
  pageMode: 'scroll',
}

/** 阅读页各设置的 localStorage 键（与 ReaderView 保持一致） */
const KEY_MAP: Record<keyof ReaderConfig, string> = {
  hanMode: 'reader_han_mode',
  theme: 'reader_theme',
  fontSize: 'reader_font_size',
  lineHeight: 'reader_line_height',
  paraSpacing: 'reader_para_spacing',
  fontWeight: 'reader_font_weight',
  contentWidth: 'reader_content_width',
  fontFamily: 'reader_font_family',
  letterSpacing: 'reader_letter_spacing',
  textIndent: 'reader_text_indent',
  textAlign: 'reader_text_align',
  pageMode: 'reader_page_mode',
}

function num(v: unknown, min: number, max: number, fallback: number, step = 1): number {
  const n = Number(v)
  if (Number.isNaN(n) || n < min || n > max) return fallback
  return Math.round(n / step) * step
}

/** 从 localStorage 读取完整阅读偏好（缺失项用默认值） */
export function loadReaderConfig(): ReaderConfig {
  const ls = (k: string): string | null => {
    try {
      return localStorage.getItem(k)
    } catch {
      return null
    }
  }
  const raw = ls(KEY_MAP.hanMode)
  const themeRaw = ls(KEY_MAP.theme)
  const widthRaw = ls(KEY_MAP.contentWidth)
  const fontRaw = ls(KEY_MAP.fontFamily)
  return {
    hanMode: raw === 'simp' || raw === 'trad' ? raw : 'auto',
    // 旧值 paper（纸色）→ 迁移为 warm（暖色）
    theme:
      themeRaw === 'dark' || themeRaw === 'warm' || themeRaw === 'system' || themeRaw === 'custom'
        ? themeRaw
        : themeRaw === 'paper'
          ? 'warm'
          : 'light',
    fontSize: num(ls(KEY_MAP.fontSize), 14, 22, 18),
    lineHeight: num(ls(KEY_MAP.lineHeight), 1.5, 2.5, 1.9, 0.1),
    paraSpacing: num(ls(KEY_MAP.paraSpacing), 0.5, 2, 1, 0.1),
    fontWeight: num(ls(KEY_MAP.fontWeight), 300, 500, 400, 50),
    contentWidth: widthRaw === '720px' || widthRaw === '900px' || widthRaw === '1080px' ? widthRaw : '900px',
    fontFamily:
      fontRaw && /^(system|song|hei|kai|fangsong|round|lishu|yahei|pingfang|wenkai|hanserif|serif)$/.test(fontRaw)
        ? fontRaw
        : 'system',
    letterSpacing: num(ls(KEY_MAP.letterSpacing), 0, 2, 0, 0.5),
    textIndent: ls(KEY_MAP.textIndent) === '0' ? false : true,
    textAlign: ls(KEY_MAP.textAlign) === 'justify' ? 'justify' : 'left',
    pageMode: parsePageMode(ls(KEY_MAP.pageMode)),
  }
}

/** 将偏好写入 localStorage（逐键覆盖）。
 *  阅读主题只作用于阅读页（ReaderView 将其挂到 .reader-page[data-reader-theme]），
 *  不再写入 html[data-theme]——界面主题（浅色/深色/跟随系统）见 utils/uiTheme.ts。 */
export function applyReaderConfig(cfg: ReaderConfig) {
  const set = (k: string, v: string) => {
    try {
      localStorage.setItem(k, v)
    } catch {
      /* ignore */
    }
  }
  set(KEY_MAP.hanMode, cfg.hanMode)
  set(KEY_MAP.theme, cfg.theme)
  set(KEY_MAP.fontSize, String(cfg.fontSize))
  set(KEY_MAP.lineHeight, String(cfg.lineHeight))
  set(KEY_MAP.paraSpacing, String(cfg.paraSpacing))
  set(KEY_MAP.fontWeight, String(cfg.fontWeight))
  set(KEY_MAP.contentWidth, cfg.contentWidth)
  set(KEY_MAP.fontFamily, cfg.fontFamily)
  set(KEY_MAP.letterSpacing, String(cfg.letterSpacing))
  set(KEY_MAP.textIndent, cfg.textIndent ? '1' : '0')
  set(KEY_MAP.textAlign, cfg.textAlign)
  set(KEY_MAP.pageMode, cfg.pageMode)
}

/**
 * 服务器下发配置与本地合并：服务器优先（逐键覆盖），缺失键保留本地值。
 * 返回值可直接 applyReaderConfig。
 */
export function mergeReaderConfig(local: ReaderConfig, server: Partial<ReaderConfig>): ReaderConfig {
  return { ...local, ...server }
}

/**
 * 提取「阅读偏好」JSON（上传用）：仅含本模块约定的键，值为原始可序列化类型。
 * 后端契约字段名沿用 localStorage 键（reader_*），便于后端原样存储/回传。
 */
export function toServerConfig(cfg: ReaderConfig): Record<string, unknown> {
  return {
    reader_han_mode: cfg.hanMode,
    reader_theme: cfg.theme,
    reader_font_size: cfg.fontSize,
    reader_line_height: cfg.lineHeight,
    reader_para_spacing: cfg.paraSpacing,
    reader_font_weight: cfg.fontWeight,
    reader_content_width: cfg.contentWidth,
    reader_font_family: cfg.fontFamily,
    reader_letter_spacing: cfg.letterSpacing,
    reader_text_indent: cfg.textIndent ? '1' : '0',
    reader_text_align: cfg.textAlign,
    reader_page_mode: cfg.pageMode,
  }
}

/** 将服务器配置 JSON（reader_* 键）解析为 ReaderConfig 局部对象（无效值忽略，不回退默认） */
export function fromServerConfig(raw: unknown): Partial<ReaderConfig> {
  if (!raw || typeof raw !== 'object') return {}
  const o = raw as Record<string, unknown>
  const out: Partial<ReaderConfig> = {}
  if (o.reader_han_mode === 'simp' || o.reader_han_mode === 'trad' || o.reader_han_mode === 'auto')
    out.hanMode = o.reader_han_mode
  if (
    o.reader_theme === 'light' ||
    o.reader_theme === 'dark' ||
    o.reader_theme === 'warm' ||
    o.reader_theme === 'system' ||
    o.reader_theme === 'custom' ||
    // 旧值 paper → 迁移为 warm
    o.reader_theme === 'paper'
  )
    out.theme = o.reader_theme === 'paper' ? 'warm' : o.reader_theme
  const fs = num(o.reader_font_size, 14, 22, NaN)
  if (!Number.isNaN(fs)) out.fontSize = fs
  const lh = num(o.reader_line_height, 1.5, 2.5, NaN, 0.1)
  if (!Number.isNaN(lh)) out.lineHeight = lh
  const ps = num(o.reader_para_spacing, 0.5, 2, NaN, 0.1)
  if (!Number.isNaN(ps)) out.paraSpacing = ps
  const fw = num(o.reader_font_weight, 300, 500, NaN, 50)
  if (!Number.isNaN(fw)) out.fontWeight = fw
  if (typeof o.reader_content_width === 'string' && ['720px', '900px', '1080px'].includes(o.reader_content_width))
    out.contentWidth = o.reader_content_width
  if (
    typeof o.reader_font_family === 'string' &&
    /^(system|song|hei|kai|fangsong|round|lishu|yahei|pingfang|wenkai|hanserif|serif)$/.test(o.reader_font_family)
  )
    out.fontFamily = o.reader_font_family
  const lsp = num(o.reader_letter_spacing, 0, 2, NaN, 0.5)
  if (!Number.isNaN(lsp)) out.letterSpacing = lsp
  if (o.reader_text_indent === '1') out.textIndent = true
  else if (o.reader_text_indent === '0') out.textIndent = false
  if (o.reader_text_align === 'left' || o.reader_text_align === 'justify') out.textAlign = o.reader_text_align
  if (
    o.reader_page_mode === 'scroll' ||
    o.reader_page_mode === 'slide' ||
    o.reader_page_mode === 'hslide' ||
    o.reader_page_mode === 'flip'
  )
    out.pageMode = o.reader_page_mode
  return out
}
