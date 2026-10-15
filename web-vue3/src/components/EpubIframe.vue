<script setup lang="ts">
/**
 * P0-1 EPUB 原版渲染（对齐 Pro epubMode=iframe）：
 * - 沙箱 iframe（禁 script）+ srcdoc 按 legacy TOC 的文件与锚点定位 XHTML
 * - 相对路径资源（CSS/图片/字体）在 srcdoc 生成时重写为 blob URL
 * - 内链跳转：宿主监听 iframe 文档点击；书内脚本无执行权限
 * - 进度：本机内滚动恢复；后端仍使用既有章节索引契约
 */
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { type EpubDoc, resolveEpubLink } from '@/utils/epubLoader'
import { epubAssetUrl, epubStylesheetUrl, rewriteEpubCss } from '@/utils/epubCss'

const props = defineProps<{
  doc: EpubDoc | null
  /** 根据 legacy TOC href 解析出的 ZIP 路径，不以 TOC 下标猜测 spine */
  path: string
  initialScroll: number | null
  fragment: string
  navigationId: number
}>()

const emit = defineEmits<{
  (e: 'navigate', path: string, fragment: string): void
  (e: 'progress', ratio: number, scrollY: number, persist: boolean): void
}>()

const frameRef = ref<HTMLIFrameElement | null>(null)
const srcdoc = ref('')
const loading = ref(false)
const restoring = ref(true)
const anchorMissing = ref(false)
const renderError = ref('')
let frameEpoch = 0

/** XHTML → srcdoc：CSS 依赖树和图片/字体重写成离线 blob URL。 */
function buildSrcdoc(doc: EpubDoc, itemPath: string): string {
  const data = doc.files.get(itemPath)
  if (!data) return '<!doctype html><html><body><p>章节缺失</p></body></html>'
  const dec = new TextDecoder('utf-8')
  let html = dec.decode(data)

  // <img src> / <image xlink:href>（SVG 封面）
  html = html.replace(/\b(src|xlink:href)\s*=\s*(["'])(.*?)\2/gi, (m, attr, _q, val) => {
    if (attr.toLowerCase() === 'src' && /\.(x?html?)($|#)/i.test(val)) return m
    return `${attr}="${epubAssetUrl(doc, itemPath, val)}"`
  })

  // 保留 link 的 media 等属性；CSS 不拼进 HTML，避免 </style> 成为注入节点。
  html = html.replace(/<link\b([^>]*)>/gi, (m, attrs: string) => {
    if (!/rel\s*=\s*["']stylesheet["']/i.test(attrs)) return m
    const hrefM = /\bhref\s*=\s*"([^"]+)"/i.exec(attrs) ?? /\bhref\s*=\s*'([^']+)'/i.exec(attrs)
    if (!hrefM) return m
    const url = epubStylesheetUrl(doc, itemPath, hrefM[1])
    return url ? m.replace(hrefM[0], `href="${url}"`) : ''
  })

  // 阅读基础样式：视口约束 + 图片不溢出（原书样式优先级更高，仅在缺省时生效）
  const shell = `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src blob: data:; media-src blob:; font-src blob:; style-src 'unsafe-inline' blob:; base-uri 'none'; form-action 'none'"><style>
    html,body{margin:0;padding:1em;word-wrap:break-word}
    img,svg,video{max-width:100%!important;height:auto!important}
  </style>`
  // 离线书不能外连、提交表单、刷新跳转或执行脚本；保留原 CSS、图片与字体。
  const parsed = new DOMParser().parseFromString(html, 'text/html')
  parsed.querySelectorAll('script,iframe,object,embed,form,base,meta[http-equiv]').forEach(el => el.remove())
  for (const style of parsed.querySelectorAll('style')) {
    style.textContent = rewriteEpubCss(doc, style.textContent ?? '', itemPath)
  }
  for (const el of parsed.querySelectorAll('*')) {
    const inlineStyle = el.getAttribute('style')
    if (inlineStyle) el.setAttribute('style', rewriteEpubCss(doc, inlineStyle, itemPath))
    for (const attr of [...el.attributes]) {
      if (/^on/i.test(attr.name) || ['srcset', 'action', 'formaction'].includes(attr.name)
        || (/^(src|href|xlink:href)$/i.test(attr.name) && /^(https?:|javascript:|\/\/)/i.test(attr.value.trim()))) {
        el.removeAttribute(attr.name)
      }
    }
  }
  // 由 DOM 设置属性，避免引号/编码锚点被正则拼接成 HTML；移除书内伪造的桥接字段。
  for (const anchor of parsed.querySelectorAll('a')) {
    anchor.removeAttribute('data-reader-epub-path')
    anchor.removeAttribute('data-reader-epub-fragment')
    const target = resolveEpubLink(doc, itemPath, anchor.getAttribute('href') ?? '')
    if (!target) { anchor.removeAttribute('href'); continue }
    anchor.setAttribute('href', `epub-nav:${target.path}#${encodeURIComponent(target.fragment)}`)
    anchor.setAttribute('data-reader-epub-path', target.path)
    anchor.setAttribute('data-reader-epub-fragment', target.fragment)
  }
  parsed.head.insertAdjacentHTML('afterbegin', shell)
  return `<!doctype html>${parsed.documentElement.outerHTML}`
}

async function renderCurrent(): Promise<void> {
  frameEpoch++
  clearFrameListeners?.()
  restoring.value = true
  anchorMissing.value = false
  renderError.value = ''
  const doc = props.doc
  if (!doc || !props.path) {
    srcdoc.value = ''
    return
  }
  loading.value = true
  try {
    srcdoc.value = buildSrcdoc(doc, props.path)
  } catch (error) {
    srcdoc.value = ''
    restoring.value = false
    renderError.value = error instanceof Error ? error.message : 'EPUB 原版排版失败'
  } finally {
    loading.value = false
  }
}

/* iframe 内事件桥接 */
let clearFrameListeners: (() => void) | null = null
function onFrameLoad(): void { void restoreFrame(false) }
async function restoreFrame(persistAfterRestore: boolean): Promise<void> {
  clearFrameListeners?.()
  const win = frameRef.value?.contentWindow
  if (!win) return
  const epoch = frameEpoch
  restoring.value = true
  try {
    const frameDoc = win.document
    const docEl = frameDoc.documentElement
    // load 已等待普通图片；字体仍可延迟改变高度。旧章/卸载后的等待不得回写新章。
    await Promise.race([frameDoc.fonts.ready, new Promise(resolve => window.setTimeout(resolve, 3000))])
    if (epoch !== frameEpoch || frameRef.value?.contentDocument !== frameDoc) return
    const hasPosition = props.initialScroll !== null && Number.isFinite(props.initialScroll)
    const anchor = props.fragment ? frameDoc.getElementById(props.fragment)
      ?? [...frameDoc.getElementsByName(props.fragment)].find(el => el.tagName.toLowerCase() === 'a') : null
    anchorMissing.value = !hasPosition && !!props.fragment && !anchor
    const target = hasPosition ? Math.max(0, props.initialScroll!)
      : anchor ? anchor.getBoundingClientRect().top + win.scrollY : 0
    // 书内 CSS 可以启用 smooth；恢复必须立即完成，不能在 aria-busy=false 后仍继续动画。
    win.scrollTo({ left: 0, top: Math.min(target, Math.max(0, docEl.scrollHeight - win.innerHeight)), behavior: 'instant' })
    const onScroll = () => {
      if (epoch !== frameEpoch || frameRef.value?.contentDocument !== frameDoc) return
      const max = docEl.scrollHeight - win.innerHeight
      emit('progress', max > 0 ? Math.min(1, Math.max(0, win.scrollY / max)) : 0,
        win.scrollY, !restoring.value)
    }
    win.addEventListener('scroll', onScroll, { passive: true })
    frameDoc.addEventListener('click', onDocClick)
    clearFrameListeners = () => {
      win.removeEventListener('scroll', onScroll)
      frameDoc.removeEventListener('click', onDocClick)
    }
    onScroll()
    await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())))
    if (epoch === frameEpoch) {
      restoring.value = false
      if (persistAfterRestore) onScroll()
    }
  } catch {
    /* sandbox 同源策略下仍可访问（allow-same-origin），异常仅防御 */
    if (epoch === frameEpoch) restoring.value = false
  }
}

function onDocClick(e: MouseEvent): void {
  const a = (e.target as HTMLElement | null)?.closest?.('a')
  if (!a) return
  const path = a.getAttribute('data-reader-epub-path')
  if (path && props.doc?.files.has(path)) {
    e.preventDefault()
    emit('navigate', path, a.getAttribute('data-reader-epub-fragment') ?? '')
  }
}

onMounted(() => void renderCurrent())
watch(() => [props.doc, props.path] as const, () => void renderCurrent())
watch(() => props.navigationId, () => {
  frameEpoch++
  void restoreFrame(true)
})
onBeforeUnmount(() => {
  frameEpoch++
  clearFrameListeners?.()
  /* blob URL 由持有方 destroyEpubDoc 统一回收 */
})
</script>

<template>
  <div class="epub-iframe-wrap">
    <div v-if="loading" class="epub-loading">加载中…</div>
    <p v-if="anchorMissing" class="epub-anchor-warning" role="status">目录锚点不存在，已回到本页顶部</p>
    <p v-if="renderError" class="epub-render-error" role="alert">{{ renderError }}；可切换普通排版继续阅读</p>
    <iframe
      v-else
      ref="frameRef"
      class="epub-frame"
      sandbox="allow-same-origin"
      :srcdoc="srcdoc"
      :aria-busy="loading || restoring"
      title="EPUB 原版排版"
      @load="onFrameLoad"
    ></iframe>
  </div>
</template>

<style scoped>
.epub-iframe-wrap {
  position: relative;
  width: 100%;
  /* 父级按正文自然高度布局，100% 会回退为 iframe 默认的 150px。 */
  height: clamp(320px, 65vh, 960px);
  height: clamp(320px, 65dvh, 960px);
  overflow: hidden;
}
.epub-frame {
  width: 100%;
  height: 100%;
  border: none;
  background: transparent;
}
.epub-loading {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  font-size: var(--font-size-sm);
  color: var(--text-3);
  pointer-events: none;
}
.epub-render-error {
  margin: 0;
  padding: 1em;
  color: var(--text-2, #555);
}
.epub-anchor-warning {
  position: absolute;
  z-index: 1;
  inset: 0 0 auto;
  margin: 0;
  padding: .5em;
  background: var(--bg-2, #fff);
  color: var(--text-2, #555);
  font-size: var(--font-size-sm);
}
</style>
