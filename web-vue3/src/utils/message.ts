import { ElMessage as nativeMessage } from 'element-plus'
import type { Message, MessageParams } from 'element-plus'
import { readerMessageOptions } from './messageOptions'

function installMessageRegionLayout(region: HTMLElement) {
  let queued = false
  let watched: Element[] = []
  const refresh = () => {
    const top = Array.from(document.querySelectorAll('.topbar,.search-box'))
    const lower = Array.from(document.querySelectorAll('.manage-bar,.reader-page .progress-bar,.update-banner'))
    const visibleRect = (element: Element) => {
      const rect = element.getBoundingClientRect()
      return rect.width > 0 && rect.height > 0 && rect.bottom > 0 && rect.top < innerHeight ? rect : null
    }
    const topEdge = Math.max(16, ...top.map(element => visibleRect(element)?.bottom ?? 0))
    const lowerEdge = Math.max(16, ...lower.map(element => {
      const rect = visibleRect(element)
      if (!rect) return 0
      // Reserve the final fixed-bar position during its entrance transform as well.
      // ResizeObserver tracks size, not the translate animation's changing position.
      const style = getComputedStyle(element)
      const bottom = Number.parseFloat(style.bottom)
      const top = style.position === 'fixed' && Number.isFinite(bottom)
        ? Math.min(rect.top, innerHeight - bottom - rect.height) : rect.top
      return innerHeight - top + 12
    }))
    region.style.setProperty('--reader-message-safe-top', `${Math.ceil(topEdge + 12)}px`)
    region.style.setProperty('--reader-message-bottom', `${Math.ceil(lowerEdge)}px`)
    const targets = [...top, ...lower]
    if (targets.length !== watched.length || targets.some((element, index) => element !== watched[index])) {
      resize.disconnect()
      targets.forEach(element => resize.observe(element))
      watched = targets
    }
  }
  const schedule = () => {
    if (queued) return
    queued = true
    requestAnimationFrame(() => { queued = false; refresh() })
  }
  const resize = new ResizeObserver(schedule)
  const app = document.getElementById('app')
  if (app) {
    new MutationObserver(schedule).observe(app, {
      childList: true, subtree: true, attributes: true, attributeFilter: ['class', 'style'],
    })
    app.addEventListener('transitionend', schedule)
  }
  window.addEventListener('resize', schedule)
  refresh()
}

function messageRegion(): HTMLElement | undefined {
  if (typeof document === 'undefined') return undefined
  const existing = document.getElementById('reader-message-stack')
  if (existing) return existing

  const region = document.createElement('div')
  region.id = 'reader-message-region'
  region.className = 'reader-message-region'
  region.setAttribute('role', 'region')
  region.setAttribute('aria-label', '操作消息，可滚动查看或关闭')

  const dismiss = document.createElement('button')
  dismiss.type = 'button'
  dismiss.className = 'reader-message-dismiss'
  dismiss.textContent = '关闭消息'
  dismiss.setAttribute('aria-controls', 'reader-message-stack')
  dismiss.addEventListener('click', () => nativeMessage.closeAll())

  const stack = document.createElement('div')
  stack.id = 'reader-message-stack'
  stack.className = 'reader-message-stack'
  stack.tabIndex = 0
  stack.setAttribute('aria-label', '消息列表，可使用键盘滚动')
  region.append(dismiss, stack)
  document.body.append(region)
  installMessageRegionLayout(region)
  // No focus change, global cap, history, credential access or extra network request.
  return stack
}

const message = ((params?: MessageParams, context?: Parameters<Message>[1]) =>
  nativeMessage(readerMessageOptions(params, messageRegion()), context)) as Message

message.primary = (params, context) => nativeMessage.primary(readerMessageOptions(params, messageRegion()), context)
message.success = (params, context) => nativeMessage.success(readerMessageOptions(params, messageRegion()), context)
message.info = (params, context) => nativeMessage.info(readerMessageOptions(params, messageRegion()), context)
message.warning = (params, context) => nativeMessage.warning(readerMessageOptions(params, messageRegion()), context)
message.error = (params, context) => nativeMessage.error(readerMessageOptions(params, messageRegion()), context)
message.closeAll = nativeMessage.closeAll
message.closeAllByPlacement = nativeMessage.closeAllByPlacement

export { message as ElMessage }
