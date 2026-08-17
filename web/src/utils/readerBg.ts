/**
 * GAP 4：阅读背景（纯色 / 纸纹 / 内置预设图 / 图片）。
 * localStorage：reader_bg_mode（color/texture/preset/image）、reader_bg_image（相对用户根路径）、
 * reader_bg_preset（内置图名称）。
 * 背景图上传到服务器 assets/background/（file/upload，home=用户根），本地只记路径；
 * 展示时经 file/download 拉取（附 accessToken，BookshelfView 封面同款方式）。
 */

export type BgMode = 'color' | 'texture' | 'preset' | 'image'

export const BG_MODE_KEY = 'reader_bg_mode'
export const BG_IMAGE_KEY = 'reader_bg_image'
export const BG_PRESET_KEY = 'reader_bg_preset'

/** 内置阅读背景图（web-ui/public/bg/*.jpg，随前端静态资源发布） */
export const BG_PRESETS = [
  '午后沙滩',
  '宁静夜色',
  '山水墨影',
  '山水画',
  '护眼漫绿',
  '新羊皮纸',
  '明媚倾城',
  '深宫魅影',
  '清新时光',
  '羊皮纸1',
  '羊皮纸2',
  '羊皮纸3',
  '羊皮纸4',
  '边彩画布',
] as const

export function loadBgMode(): BgMode {
  try {
    const raw = localStorage.getItem(BG_MODE_KEY)
    if (raw === 'texture' || raw === 'preset' || raw === 'image') return raw
  } catch {
    /* ignore */
  }
  return 'color'
}

export function saveBgMode(mode: BgMode) {
  try {
    localStorage.setItem(BG_MODE_KEY, mode)
  } catch {
    /* ignore */
  }
}

export function loadBgImagePath(): string {
  try {
    return localStorage.getItem(BG_IMAGE_KEY) ?? ''
  } catch {
    return ''
  }
}

export function saveBgImagePath(path: string) {
  try {
    if (path) localStorage.setItem(BG_IMAGE_KEY, path)
    else localStorage.removeItem(BG_IMAGE_KEY)
  } catch {
    /* ignore */
  }
}

/** 当前内置背景图名称（空时回退首张，保证 preset 模式总有图） */
export function loadBgPreset(): string {
  try {
    const v = localStorage.getItem(BG_PRESET_KEY)
    if (v && (BG_PRESETS as readonly string[]).includes(v)) return v
  } catch {
    /* ignore */
  }
  return BG_PRESETS[0]
}

export function saveBgPreset(name: string) {
  try {
    localStorage.setItem(BG_PRESET_KEY, name)
  } catch {
    /* ignore */
  }
}

/** 内置背景图展示 URL（vite public 静态资源，无需 accessToken） */
export function bgPresetUrl(name: string): string {
  return `/bg/${encodeURIComponent(name)}.jpg`
}

/** 背景图展示 URL：file/download + accessToken（path 相对用户根；空路径返回 ''） */
export function bgImageUrl(path: string, accessToken: string): string {
  if (!path) return ''
  const base = `/reader3/file/download?path=${encodeURIComponent(path)}`
  return accessToken ? `${base}&accessToken=${encodeURIComponent(accessToken)}` : base
}
