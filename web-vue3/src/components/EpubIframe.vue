<script setup lang="ts">
/**
 * P0-1 EPUB 原版渲染（对齐 Pro epubMode=iframe）：
 * - 沙箱 iframe（禁 script）+ srcdoc 渲染 spine 指定章的 XHTML
 * - 相对路径资源（CSS/图片/字体）在 srcdoc 生成时重写为 blob URL
 * - 内链跳转：锚点 href 重写为 postMessage 协议，由宿主切章
 * - 进度同步：滚动比例 ↔ 宿主 durChapterProgress
 */
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { type EpubDoc, epubResourceUrl, resolveHref } from '@/utils/epubLoader'

const props = defineProps<{
  doc: EpubDoc | null
  /** spine 下标 */
  index: number
}>()

const emit = defineEmits<{
  (e: 'navigate', href: string): void
  (e: 'progress', ratio: number): void
}>()

const frameRef = ref<HTMLIFrameElement | null>(null)
const srcdoc = ref('')
const loading = ref(false)

/** 当前章 manifest 条目 */
const currentItem = computed(() => {
  if (!props.doc) return null
  const sp = props.doc.spine[props.index]
  return sp ? props.doc.manifest.get(sp.idref) ?? null : null
})

/** zip 路径 → blob URL；未知资源返回 '#' 占位 */
function rewriteUrl(doc: EpubDoc, baseDir: string, raw: string): string {
  const clean = raw.trim()
  if (/^(https?:|data:|blob:)/i.test(clean)) return clean
  // 去掉锚点后解析真实路径，锚点转交宿主
  const hashIdx = clean.indexOf('#')
  const pathPart = hashIdx >= 0 ? clean.slice(0, hashIdx) : clean
  const frag = hashIdx >= 0 ? clean.slice(hashIdx + 1) : ''
  const target = resolveHref(baseDir, pathPart)
  const url = doc.files.has(target) ? epubResourceUrl(doc, target) : null
  if (!url) return '#'
  return frag ? `${url}#${frag}` : url
}

/** XHTML → srcdoc：重写外链引用并内联原书 CSS */
function buildSrcdoc(doc: EpubDoc, itemPath: string): string {
  const data = doc.files.get(itemPath)
  if (!data) return '<!doctype html><html><body><p>章节缺失</p></body></html>'
  const dec = new TextDecoder('utf-8')
  let html = dec.decode(data)
  const baseDir = itemPath.includes('/') ? itemPath.slice(0, itemPath.lastIndexOf('/')) : ''

  // <img src> / <image xlink:href>（SVG 封面）
  html = html.replace(/\b(src|xlink:href)\s*=\s*"([^"]+)"/gi, (m, attr, val) => {
    if (attr.toLowerCase() === 'src' && /\.(x?html?)($|#)/i.test(val)) return m
    return `${attr}="${rewriteUrl(doc, baseDir, val)}"`
  })

  // <link rel=stylesheet href>
  html = html.replace(/<link\b([^>]*)>/gi, (m, attrs: string) => {
    if (!/rel\s*=\s*["']stylesheet["']/i.test(attrs)) return m
    const hrefM = /\bhref\s*=\s*"([^"]+)"/i.exec(attrs) ?? /\bhref\s*=\s*'([^']+)'/i.exec(attrs)
    if (!hrefM) return m
    const cssPath = resolveHref(baseDir, hrefM[1])
    const cssData = doc.files.get(cssPath)
    if (!cssData) return ''
    let css = new TextDecoder('utf-8').decode(cssData)
    // CSS 内的相对引用（背景图/字体）→ blob URL
    css = css.replace(/url\(\s*(['"]?)([^)'"]+)\1\s*\)/gi, (_mm, q, v: string) =>
      `url(${q}${rewriteUrl(doc, baseDir, v)}${q})`)
    return `<style>${css}</style>`
  })

  // 锚点跳转：同书内 .xhtml 链接改由宿主处理
  html = html.replace(/\bhref\s*=\s*"([^"]+\.(?:x?html?))([^"]*)"/gi, (_m, path: string, rest: string) => {
    const target = resolveHref(baseDir, path)
    void target
    return `href="epub-nav:${path}${rest}"`
  })

  // 阅读基础样式：视口约束 + 图片不溢出（原书样式优先级更高，仅在缺省时生效）
  const shell = `<base target="_self"><style>
    html,body{margin:0;padding:1em;word-wrap:break-word}
    img,svg,video{max-width:100%!important;height:auto!important}
  </style>`
  // 注入到 head 最前（保证原书样式可覆盖）
  if (/<head[^>]*>/i.test(html)) {
    html = html.replace(/<head([^>]*)>/i, `<head$1>${shell}`)
  } else {
    html = `<!doctype html><html><head>${shell}</head><body>${html}</body></html>`
  }
  return html
}

async function renderCurrent(): Promise<void> {
  const doc = props.doc
  const item = currentItem.value
  if (!doc || !item) {
    srcdoc.value = ''
    return
  }
  loading.value = true
  try {
    srcdoc.value = buildSrcdoc(doc, item.href)
  } finally {
    loading.value = false
  }
}

/* iframe 内事件桥接 */
function onFrameLoad(): void {
  const win = frameRef.value?.contentWindow
  if (!win) return
  try {
    const docEl = win.document.documentElement
    win.addEventListener('scroll', () => {
      const max = docEl.scrollHeight - win.innerHeight
      if (max > 0) emit('progress', Math.min(1, Math.max(0, win.scrollY / max)))
    }, { passive: true })
  } catch {
    /* sandbox 同源策略下仍可访问（allow-same-origin），异常仅防御 */
  }
}

function onDocClick(e: MouseEvent): void {
  const a = (e.target as HTMLElement | null)?.closest?.('a')
  if (!a) return
  const href = a.getAttribute('href') ?? ''
  if (href.startsWith('epub-nav:')) {
    e.preventDefault()
    const nav = href.slice('epub-nav:'.length)
    const hashIdx = nav.indexOf('#')
    emit('navigate', hashIdx >= 0 ? nav.slice(0, hashIdx) : nav)
  }
}

onMounted(() => void renderCurrent())
watch(() => [props.doc, props.index] as const, () => void renderCurrent())
onBeforeUnmount(() => {
  /* blob URL 由持有方 destroyEpubDoc 统一回收 */
})
</script>

<template>
  <div class="epub-iframe-wrap">
    <div v-if="loading" class="epub-loading">加载中…</div>
    <iframe
      ref="frameRef"
      class="epub-frame"
      sandbox="allow-same-origin allow-popups"
      :srcdoc="srcdoc"
      title="EPUB 原版排版"
      @load="onFrameLoad"
      @click.capture="onDocClick"
    ></iframe>
  </div>
</template>

<style scoped>
.epub-iframe-wrap {
  position: relative;
  width: 100%;
  height: 100%;
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
</style>
