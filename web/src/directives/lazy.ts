import type { Directive } from 'vue'

const observers = new WeakMap<HTMLImageElement, IntersectionObserver>()
/** 当前应加载的 src——observer 回调时读取（避免闭包捕获 mounted 时的旧绑定值，P2：v-lazy 闭包过期修复） */
const srcs = new WeakMap<HTMLImageElement, string>()

/** 立即加载并淡入（observer 已触发 / 元素已加载后换图） */
function loadNow(el: HTMLImageElement, src: string): void {
  el.src = src
  el.classList.add('is-loaded')
  observers.get(el)?.disconnect()
  observers.delete(el)
}

/** v-lazy：封面图懒加载（IntersectionObserver，进入视口附近才加载 + 淡入） */
export const lazy: Directive<HTMLImageElement, string> = {
  mounted(el, binding) {
    const src = binding.value
    if (!src) return
    srcs.set(el, src)
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            // 回调时读当前绑定值：绑定值已变化（updated）时加载最新图，而非旧闭包值
            loadNow(el, srcs.get(el) ?? src)
          }
        }
      },
      { rootMargin: '240px 0px' },
    )
    observers.set(el, observer)
    observer.observe(el)
  },
  updated(el, binding) {
    if (binding.value && binding.value !== binding.oldValue) {
      srcs.set(el, binding.value)
      if (el.classList.contains('is-loaded')) {
        // 已加载完成：直接换图（保留 is-loaded 避免闪烁）
        el.src = binding.value
      }
      // 未加载完成：observer 回调会读取 srcs 中的最新值
    }
  },
  unmounted(el) {
    observers.get(el)?.disconnect()
    observers.delete(el)
    srcs.delete(el)
  },
}
