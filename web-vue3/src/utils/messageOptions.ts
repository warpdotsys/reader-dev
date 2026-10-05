import { isVNode } from 'vue'
import type { MessageOptions, MessageParams } from 'element-plus'

/** Presentation only: preserve content, severity, timers, grouping and callbacks. */
export function readerMessageOptions(params: MessageParams, appendTo?: HTMLElement): MessageOptions {
  const options: MessageOptions = params == null || typeof params === 'string'
    || typeof params === 'function' || isVNode(params) ? { message: params } : params
  return {
    ...options,
    appendTo,
    placement: 'bottom-right',
    showClose: options.showClose ?? true,
    customClass: ['reader-message', options.customClass ?? ''].filter(Boolean).join(' '),
  }
}
