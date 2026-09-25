/**
 * 书架视图模式（GAP 103 扩展）：网格 / 列表 / 墙（大封面网格）。
 *
 * - localStorage 键 reader_shelf_view 扩展 'wall'（parseShelfView 统一解析，非法值回落 grid）
 * - 墙模式：卡片更大、间距更宽、书名在封面下——虚拟滚动行高按 shelfViewMetrics 计算
 *    （列数 = floor((容器宽 + 列距) / (墙卡片最小宽 + 列距))，行高 = 封面高(4:3 宽) + 元信息高）
 * 纯函数，可单测。
 */

export type ShelfViewMode = 'grid' | 'list' | 'wall'

export const SHELF_VIEW_KEY = 'reader_shelf_view'

export type CardDensity = 's' | 'm' | 'l'

export interface ShelfViewMetrics {
  /** 卡片最小宽（--card-w；list 模式为 1px 占位，实际 1fr 单列） */
  cardMinW: number
  /** 列间距 */
  colGap: number
  /** 行间距（虚拟滚动 stride 用） */
  rowGap: number
  /** 卡片元信息区高度（无 DOM 可测时的行高兜底加数：封面 4:3 + metaH） */
  metaH: number
}

/** 桌面端卡片最小宽（宽屏） */
const DENSITY_MIN_W: Record<CardDensity, number> = { s: 128, m: 160, l: 204 }
/** 窄屏（<=720px）卡片最小宽 */
const DENSITY_NARROW_W: Record<CardDensity, number> = { s: 96, m: 120, l: 150 }
/** 墙模式卡片最小宽（比大密度更宽，间距更宽） */

/** 解析 localStorage 值：'grid' | 'list' | 'wall'，其余回落 'grid' */
export function parseShelfView(raw: string | null | undefined): ShelfViewMode {
  return raw === 'list' || raw === 'wall' ? raw : 'grid'
}

/** 各视图模式的布局尺寸（虚拟滚动列数 / 行高按此计算） */
export function shelfViewMetrics(
  mode: ShelfViewMode,
  narrow: boolean,
  density: CardDensity = 'm',
): ShelfViewMetrics {
  if (mode === 'wall') {
    // 墙视图卡片尺寸跟随密度（用户可调——不再固定大卡）
    const w = density === 's' ? 176 : density === 'l' ? 240 : 208
    return {
      cardMinW: narrow ? Math.round(w * 0.72) : w,
      colGap: 40,
      rowGap: 48,
      metaH: 96,
    }
  }
  if (mode === 'list') {
    return { cardMinW: 1, colGap: narrow ? 16 : 28, rowGap: narrow ? 24 : 32, metaH: 76 }
  }
  return {
    cardMinW: narrow ? DENSITY_NARROW_W[density] : DENSITY_MIN_W[density],
    colGap: narrow ? 16 : 28,
    rowGap: narrow ? 24 : 32,
    metaH: 78,
  }
}
