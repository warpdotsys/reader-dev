/**
 * 阅读页翻页模式（reader_page_mode）：
 * - scroll 连续滚动（默认）
 * - slide  上下翻页（滚轮/触屏/按钮逐屏滚动）
 * - hslide 左右滑动翻章（横向滑动手势/按钮 → 章节级翻页 + 切章过渡动画）
 * - flip   仿真翻页（CSS 多栏横向分页 + 页过渡动画）
 * 阅读页设置面板与 utils/readerConfig.ts（服务器配置合并）共用本模块，保证取值口径一致。
 */

export type PageMode = 'scroll' | 'slide' | 'hslide' | 'flip'

export const PAGE_MODES: PageMode[] = ['scroll', 'slide', 'hslide', 'flip']

/** 设置面板按钮文案 */
export const PAGE_MODE_LABELS: Record<PageMode, string> = {
  scroll: '滚动',
  slide: '上下',
  hslide: '左右',
  flip: '仿真',
}

/** 非法/缺失值回退默认（默认 scroll，兼容旧值：'slide' 语义沿用） */
export function parsePageMode(raw: string | null | undefined, fallback: PageMode = 'scroll'): PageMode {
  return raw === 'scroll' || raw === 'slide' || raw === 'hslide' || raw === 'flip' ? raw : fallback
}
