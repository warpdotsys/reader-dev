import type { ObjectDirective } from 'vue'

interface DialogState {
  element: HTMLElement
  opener: HTMLElement | null
  keydown: (event: KeyboardEvent) => void
  focusin: (event: FocusEvent) => void
}

const states = new WeakMap<HTMLElement, DialogState>()
const stacks = new WeakMap<Document, DialogState[]>()
const modalSelector = '[role="dialog"][aria-modal="true"], [role="alertdialog"][aria-modal="true"]'

/** 查询当前可用控件，异步加载/禁用按钮之后不沿用旧的 Tab 列表。 */
function controls(element: HTMLElement): HTMLElement[] {
  return Array.from(element.querySelectorAll<HTMLElement>(
    'button, input, textarea, select, a[href], [tabindex], [contenteditable="true"]',
  )).filter(item => item.tabIndex >= 0 && !item.matches(':disabled')
    && !item.closest('[hidden], [inert]') && item.getClientRects().length > 0
    && element.ownerDocument.defaultView?.getComputedStyle(item).visibility !== 'hidden')
    .sort((a, b) => (a.tabIndex > 0 ? a.tabIndex : Infinity) - (b.tabIndex > 0 ? b.tabIndex : Infinity))
}

/** null 表示让浏览器处理内部移动；越过边界/焦点不在控件中时才接管。 */
export function dialogTabBoundary(count: number, activeIndex: number, backwards: boolean): number | null {
  if (count === 0) return -1
  if (activeIndex < 0) return backwards ? count - 1 : 0
  if (backwards && activeIndex === 0) return count - 1
  if (!backwards && activeIndex === count - 1) return 0
  return null
}

function focusInitial(element: HTMLElement): void {
  ;(controls(element)[0] ?? element).focus({ preventScroll: true })
}

function top(element: HTMLElement): boolean {
  return stacks.get(element.ownerDocument)?.at(-1)?.element === element
}

/** Element Plus 的确认框有自己的焦点管理，不能被下层自定义弹窗抢回。 */
function foreignModal(target: Element | null, element: HTMLElement): boolean {
  const modal = target?.closest<HTMLElement>(modalSelector)
  return !!modal && modal !== element && !states.has(modal)
}

/** 只管理焦点。Esc/关闭/提交和忙碌状态仍完全由业务组件控制。 */
export const dialogFocus: ObjectDirective<HTMLElement, HTMLElement | null | undefined> = {
  mounted(element, binding) {
    const doc = element.ownerDocument
    // 文件选择器/异步预览可以显式传入原按钮，不能误把隐藏 file input 当触发者。
    const candidate = binding?.value ?? doc.activeElement as HTMLElement | null
    const state: DialogState = {
      element,
      opener: candidate && typeof candidate.focus === 'function' ? candidate : null,
      keydown(event) {
        if (!top(element) || event.key !== 'Tab' || event.defaultPrevented
          || event.altKey || event.ctrlKey || event.metaKey || foreignModal(doc.activeElement, element)) return
        const available = controls(element)
        const index = available.indexOf(doc.activeElement as HTMLElement)
        const boundary = dialogTabBoundary(available.length, index, event.shiftKey)
        if (boundary === null) return
        event.preventDefault()
        ;(available[boundary] ?? element).focus({ preventScroll: true })
      },
      focusin(event) {
        const target = event.target as Element | null
        if (top(element) && !element.contains(target) && !foreignModal(target, element)) focusInitial(element)
      },
    }
    const stack = stacks.get(doc) ?? []
    stacks.set(doc, stack)
    stack.push(state)
    states.set(element, state)
    doc.addEventListener('keydown', state.keydown, true)
    doc.addEventListener('focusin', state.focusin, true)
    focusInitial(element)
  },
  updated(element) {
    const active = element.ownerDocument.activeElement
    if (top(element) && !foreignModal(active, element)
      && (!element.contains(active) || active?.matches(':disabled'))) focusInitial(element)
  },
  beforeUnmount(element) {
    const state = states.get(element)
    if (!state) return
    const doc = element.ownerDocument
    const wasTop = top(element)
    doc.removeEventListener('keydown', state.keydown, true)
    doc.removeEventListener('focusin', state.focusin, true)
    states.delete(element)
    const stack = stacks.get(doc)!
    stack.splice(stack.indexOf(state), 1)
    if (!wasTop || foreignModal(doc.activeElement, element)) return
    if (state.opener?.isConnected && !state.opener.matches(':disabled')
      && state.opener.getClientRects().length > 0) state.opener.focus({ preventScroll: true })
    else if (stack.length > 0) focusInitial(stack.at(-1)!.element)
  },
}
