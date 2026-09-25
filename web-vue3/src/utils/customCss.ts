/**
 * GAP 5：自定义样式（reader_custom_css → 注入 <style id="reader-custom-style">）。
 * 设置页 textarea 编辑 → saveCustomCss + applyCustomCss；App.vue 进入时恢复注入。
 * 样式作用于全局（阅读器 .reader-page 与界面均可覆盖）。
 */

export const CUSTOM_CSS_KEY = 'reader_custom_css'
const STYLE_ID = 'reader-custom-style'

export function loadCustomCss(): string {
  try {
    return localStorage.getItem(CUSTOM_CSS_KEY) ?? ''
  } catch {
    return ''
  }
}

export function saveCustomCss(css: string) {
  try {
    localStorage.setItem(CUSTOM_CSS_KEY, css)
  } catch {
    /* ignore */
  }
}

/** 注入/更新全局 <style>（空内容则移除已注入元素） */
export function applyCustomCss(css: string = loadCustomCss()) {
  if (typeof document === 'undefined') return
  let el = document.getElementById(STYLE_ID) as HTMLStyleElement | null
  if (!css.trim()) {
    el?.remove()
    return
  }
  if (!el) {
    el = document.createElement('style')
    el.id = STYLE_ID
    document.head.appendChild(el)
  }
  el.textContent = css
}
