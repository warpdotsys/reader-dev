/**
 * 全站共享简繁模式（响应式状态）。
 *
 * 背景：utils/chinese.ts 提供纯函数（applyHan / getHanMode / setHanMode），
 * 但各视图各自持有 ref 会导致「改模式后其他页面不响应」。
 * 本模块抽出单一响应式 ref（hanMode），所有展示简繁转换的视图共用：
 *   - 写入统一走 setGlobalHanMode（同时落 localStorage reader_han_mode）；
 *   - 跨标签页改动经 storage 事件监听同步（reader_han_mode 键）；
 *   - 视图挂载时可调 syncHanMode() 兜底（同标签页内由其他写入方直改 localStorage 的场景）。
 *
 * 阅读页（ReaderView）另有 per-book 覆盖的独立 hanMode ref（reader_book_config 本书覆盖），
 * 但其全局写入路径（saveSetting → setGlobalHanMode）同样会更新本状态，保证全站一致。
 */

import { ref } from 'vue'
// 显式 .ts 扩展名：node --test 直接执行 TS（与既有 utils 测试约定一致）
import { applyHan, getHanMode, setHanMode, type HanMode } from './chinese.ts'

/** 全站共享简繁模式（响应式；初始值取自 localStorage reader_han_mode） */
export const hanMode = ref<HanMode>(getHanMode())

/** 视图内使用：const mode = useHanMode() 后即可响应式读取 */
export function useHanMode() {
  return hanMode
}

/** 设置全局简繁模式：更新响应式状态 + 写 localStorage（阅读页/书海等写入方统一走这里） */
export function setGlobalHanMode(m: HanMode) {
  hanMode.value = m
  setHanMode(m)
}

/** 按当前全局模式转换文本（模板中直接调用，响应式跟随 hanMode） */
export function hanText(text: string): string {
  return applyHan(text, hanMode.value)
}

/** 从 localStorage 重新同步（视图挂载 / 服务器配置下发后调用，覆盖同标签页直写场景） */
export function syncHanMode() {
  hanMode.value = getHanMode()
}

/** 跨标签页响应：其他标签页修改 reader_han_mode 时同步本状态 */
if (typeof window !== 'undefined') {
  window.addEventListener('storage', (e) => {
    if (e.key === 'reader_han_mode') syncHanMode()
  })
}
