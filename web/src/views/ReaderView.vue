<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getBookshelf, deleteBook } from '@/api/bookshelf'
import { getBookInfo, getBookToc, getBookContent, searchBookSource, searchBookSourceSSE } from '@/api/books'
import {
  deleteBookmarks,
  parseBookmarksJson,
  saveBookmark,
  saveBookmarks,
} from '@/api/bookmarks'
import { getInvalidBookSources } from '@/api/sources'
import { saveBook } from '@/api/bookshelf'
import { getHttpTtsList } from '@/api/httpTts'
import { get, post } from '@/api/request'
import { getBookCacheChapters } from '@/api/cacheBook'
import { loadReplaceRules, saveReplaceRules } from '@/api/replaceRules'
import { getTtsVoices, synthesizeTts, type TtsVoice } from '@/api/tts'
import { getLocalChapter, listLocalChapterUrls, saveLocalChapter } from '@/utils/readerLocalCache'
import {
  loadCustomFont,
  removeCustomFont,
  saveCustomFont,
} from '@/utils/readerFont'
import ChapterCacheDialog from '@/components/ChapterCacheDialog.vue'
import { applyHan, getHanMode, type HanMode } from '@/utils/chinese'
import { setGlobalHanMode } from '@/utils/hanMode'
import { DAILY_STATS_KEY, accumulateDaily, parseDailyStats } from '@/utils/dailyStats'
import { proxyImageUrl } from '@/utils/imageProxy'
import { useUserStore } from '@/stores/user'
import { loadBookConfig, saveBookConfig, clearBookConfig } from '@/utils/bookConfig'
import { t } from '@/utils/i18n'
import { requestWakeLock, releaseWakeLock } from '@/utils/wakeLock'
import { parsePageMode, type PageMode } from '@/utils/readerPageMode'
import {
  loadBgMode,
  loadBgImagePath,
  loadBgPreset,
  saveBgMode,
  bgPresetUrl,
  bgImageUrl as bgImageUrlOf,
  type BgMode,
} from '@/utils/readerBg'
import {
  loadCustomTheme,
  saveCustomTheme,
  customThemeVars,
  customThemeIsDark,
  CUSTOM_THEME_DEFAULTS,
  type ReaderCustomTheme,
} from '@/utils/readerTheme'
import { relocateChapterIndex } from '@/utils/progressRelocate'
import type { Book, BookChapter, BookInfo, Bookmark, HttpTts, ReplaceRule, SearchBook } from '@/types'

const route = useRoute()
const router = useRouter()
const store = useUserStore()

/** /reader/:bookUrl —— vue-router 已自动解码 encodeURIComponent 参数 */
const bookUrl = computed(() => String(route.params.bookUrl ?? ''))

const MIN_FONT = 14
const MAX_FONT = 22
const FONT_KEY = 'reader_font_size'

/** 阅读进度：章节索引 + 滚动位置（localStorage: reader-progress-{bookUrl}） */
interface ReaderProgress {
  chapterIndex: number
  scrollY: number
  updatedAt: number
}

/* ---------------- 设置读取/持久化小工具 ---------------- */

function loadSetting(key: string, min: number, max: number, fallback: number, step = 1): number {
  const raw = Number(localStorage.getItem(key))
  if (Number.isNaN(raw) || raw < min || raw > max) return fallback
  return Math.round(raw / step) * step
}
function persist(key: string, value: unknown) {
  try {
    localStorage.setItem(key, String(value))
  } catch {
    /* ignore */
  }
}
const round1 = (v: number) => Math.round(v * 10) / 10

const shelfBook = ref<Book | null>(null)
const bookName = ref('')
const chapters = ref<BookChapter[]>([])
const chapterIndex = ref(0)
const content = ref('')
const loading = ref(true)
const loadError = ref(false)
const notFound = ref(false)
const drawerOpen = ref(false)
/** 临时书详情（init 中与目录并行拉取——退出挽留入架时补全作者/封面/目录等字段） */
const tempInfo = ref<BookInfo | null>(null)
const retentionOpen = ref(false)
const retentionBusy = ref(false)
let retentionResolve: ((ok: boolean) => void) | null = null
/** 章节缓存弹层（服务器 / 本机双向） */
const cacheOpen = ref(false)

/* ---------------- 非文本书（legacy BookType：0 文本/1 音频/2 漫画/3 文件/4 视频） ---------------- */

/** 书籍类型：书架书 type 优先；临时书（query.type）兑底 */
const bookType = computed<number>(() => {
  const t = shelfBook.value?.type
  if (typeof t === 'number' && t >= 0 && t <= 4) return t
  const q = Number(route.query.type)
  return Number.isInteger(q) && q >= 0 && q <= 4 ? q : 0
})
const isTextBook = computed(() => bookType.value === 0)
const isAudioBook = computed(() => bookType.value === 1)
const isComicBook = computed(() => bookType.value === 2)
const isFileBook = computed(() => bookType.value === 3)
const isVideoBook = computed(() => bookType.value === 4)
const isNonTextBook = computed(() => bookType.value !== 0)

/* ---------------- 1. 主题（亮/暗/暖/跟随系统/自定义） ---------------- */

type Theme = 'light' | 'dark' | 'warm' | 'system' | 'custom'
const THEME_KEY = 'reader_theme'
const THEME_ORDER: Theme[] = ['light', 'dark', 'warm', 'system', 'custom']
const theme = ref<Theme>('light')
/** 阅读页根元素（主题只作用于阅读页内，与界面主题 html[data-theme] 分离） */
const pageRef = ref<HTMLElement | null>(null)
/** 系统深色偏好（theme=system 时生效；matchMedia 监听切换） */
const systemDark = ref(false)
let mediaQuery: MediaQueryList | null = null
const systemTheme = (): Theme => (systemDark.value ? 'dark' : 'light')
function applyTheme(t: Theme) {
  // 阅读内容主题仅作用于 .reader-page（变量覆盖见 styles/main.css），不影响书架/设置等界面
  // 自定义主题：基座按背景明暗取 dark/light（color-scheme/滚动条），三色变量经内联样式注入
  if (!pageRef.value) return
  if (t === 'custom') {
    pageRef.value.dataset.readerTheme = customThemeIsDark(customTheme.value) ? 'dark' : 'light'
    return
  }
  const real = t === 'system' ? systemTheme() : t
  pageRef.value.dataset.readerTheme = real
}
function onSystemThemeChange(e: MediaQueryListEvent) {
  systemDark.value = e.matches
  if (theme.value === 'system') applyTheme(theme.value)
}
watch(theme, (t) => {
  applyTheme(t)
  saveSetting(THEME_KEY, t)
})
function cycleTheme() {
  const i = THEME_ORDER.indexOf(theme.value)
  theme.value = THEME_ORDER[(i + 1) % THEME_ORDER.length]
  // 循环落到「自定义」：顺带打开颜色弹层（第 5 档需配置三色）
  if (theme.value === 'custom') customOpen.value = true
}

/* ---------------- 1.1 自定义主题（第 5 档——背景色/文字色/强调色弹层；localStorage reader_theme_custom） ---------------- */

const customTheme = ref<ReaderCustomTheme>(loadCustomTheme())
const customOpen = ref(false)
watch(customTheme, (v) => saveCustomTheme(v), { deep: true })
// 自定义主题下改色：基座明暗可能翻转（浅↔深）→ 重刷 data-reader-theme
watch(
  customTheme,
  () => {
    if (theme.value === 'custom') applyTheme('custom')
  },
  { deep: true },
)
/** 主题选择（设置面板 seg）：选中「自定义」时顺带打开颜色弹层 */
function selectTheme(th: Theme) {
  theme.value = th
  if (th === 'custom') customOpen.value = true
}
/** 恢复默认三色（CUSTOM_THEME_DEFAULTS：米白纸色 + 深褐文字 + 棕金强调） */
function resetCustomTheme() {
  customTheme.value = { ...CUSTOM_THEME_DEFAULTS }
}

/* ---------------- GAP 5：纸纹（细噪点 radial-gradient 微纹理叠在阅读页背景上；3 主题通用，暖色主题最佳） ---------------- */

const TEXTURE_KEY = 'reader_texture'
const paperTexture = ref(false)
{
  const raw = localStorage.getItem(TEXTURE_KEY)
  if (raw === '1') paperTexture.value = true
}
watch(paperTexture, (v) => persist(TEXTURE_KEY, v ? '1' : '0'))

/* ---------------- 2. 排版（行距/段距/字重） ---------------- */

const MIN_LINE = 1.5
const MAX_LINE = 2.5
const MIN_PARA = 0.5
const MAX_PARA = 2
const MIN_WEIGHT = 300
const MAX_WEIGHT = 500

const fontSize = ref<number>(18)
const lineHeight = ref(1.9)
/** 正文宽度档位（窄/适中/宽——max-width） */
const WIDTH_OPTIONS = [
  { label: '窄', value: '720px' },
  { label: '适中', value: '900px' },
  { label: '宽', value: '1080px' },
]
const contentWidth = ref(WIDTH_OPTIONS[1].value)
function setWidth(v: string) {
  contentWidth.value = v
  saveSetting('reader_content_width', v)
}
/** 切章动画时长（legacy animateMSTime；0 = 立即切换） */
const animateMs = ref(loadSetting('reader_animate_ms', 0, 1000, 260, 10))
watch(animateMs, (v) => saveSetting('reader_animate_ms', v))
/** 章节目录/正文请求超时（秒；legacy chapterRequestTimeout） */
const chapterTimeout = ref(loadSetting('reader_chapter_timeout', 10, 120, 30, 5))
watch(chapterTimeout, (v) => saveSetting('reader_chapter_timeout', v))
const paraSpacing = ref(1)
const fontWeight = ref(400)
watch(fontSize, (v) => saveSetting(FONT_KEY, v))
watch(lineHeight, (v) => saveSetting('reader_line_height', v))
watch(paraSpacing, (v) => saveSetting('reader_para_spacing', v))
watch(fontWeight, (v) => saveSetting('reader_font_weight', v))

/* ---------------- 2.01 亮度（0.6-1.4 倍，filter: brightness() 作用于 .reader-main；localStorage: reader_brightness） ---------------- */

const MIN_BRIGHT = 0.6
const MAX_BRIGHT = 1.4
const BRIGHT_KEY = 'reader_brightness'
const brightness = ref(1)
{
  const raw = Number(localStorage.getItem(BRIGHT_KEY))
  if (!Number.isNaN(raw) && raw >= MIN_BRIGHT && raw <= MAX_BRIGHT) brightness.value = raw
}
const round2 = (v: number) => Math.round(v * 100) / 100
watch(brightness, (v) => persist(BRIGHT_KEY, round2(v)))
const brightnessOpen = ref(false)

/* ---------------- 2.1 字体（系统/衬线/圆体/黑体） ---------------- */

type FontKind = 'system' | 'song' | 'hei' | 'kai' | 'fangsong' | 'round' | 'lishu' | 'yahei' | 'pingfang' | 'wenkai' | 'hanserif' | 'serif'
const FONT_OPTIONS: { label: string; value: FontKind }[] = [
  { label: '系统', value: 'system' },
  { label: '宋体', value: 'song' },
  { label: '黑体', value: 'hei' },
  { label: '楷体', value: 'kai' },
  { label: '仿宋', value: 'fangsong' },
  { label: '圆体', value: 'round' },
  { label: '隶书', value: 'lishu' },
  { label: '雅黑', value: 'yahei' },
  { label: '苹方', value: 'pingfang' },
  { label: '文楷', value: 'wenkai' },
  { label: '思源宋', value: 'hanserif' },
  { label: '衬线', value: 'serif' },
]
const FONT_STACK: Record<FontKind, string> = {
  system: '',
  song: "'Songti SC', 'SimSun', 'NSimSun', '宋体', 'Noto Serif CJK SC', serif",
  hei: "'PingFang SC', 'HarmonyOS Sans SC', 'Microsoft YaHei', 'Hiragino Sans GB', sans-serif",
  kai: "'Kaiti SC', 'STKaiti', 'KaiTi', '楷体', '楷体_GB2312', serif",
  fangsong: "'FangSong', 'STFangsong', 'FangSong_GB2312', '仿宋', serif",
  round: "'Yuanti SC', 'YouYuan', '幼圆', 'PingFang SC', sans-serif",
  lishu: "'LiSu', 'STLiti', '隶书', serif",
  yahei: "'Microsoft YaHei', '微软雅黑', 'PingFang SC', sans-serif",
  pingfang: "'PingFang SC', 'SF Pro SC', 'HarmonyOS Sans SC', sans-serif",
  wenkai: "'LXGW WenKai', 'Kaiti SC', '楷体', serif",
  hanserif: "'Source Han Serif CN', 'Songti SC', 'SimSun', '宋体', serif",
  serif: "Georgia, 'Songti SC', 'SimSun', 'Noto Serif CJK SC', serif",
}
const fontKind = ref<FontKind>('system')
const fontOpen = ref(false)
const fontLabel = computed(() => FONT_OPTIONS.find((o) => o.value === fontKind.value)?.label ?? '系统')
watch(fontKind, (v) => saveSetting('reader_font_family', v))
/** 自定义字体（legacy 字体上传：IndexedDB 存文件 + Blob URL @font-face；优先级高于内置字体） */
const customFontUrl = ref('')
const customFontEnabled = ref(localStorage.getItem('reader_custom_font') !== '0')
const customFontInput = ref<HTMLInputElement | null>(null)
let customFontStyleEl: HTMLStyleElement | null = null
watch(customFontEnabled, (v) => {
  try {
    localStorage.setItem('reader_custom_font', v ? '1' : '0')
  } catch {
    /* ignore */
  }
})
async function initCustomFont() {
  try {
    const file = await loadCustomFont()
    if (file) applyCustomFontUrl(file)
  } catch {
    /* IndexedDB 不可用——忽略 */
  }
}
function applyCustomFontUrl(file: Blob) {
  if (customFontUrl.value) URL.revokeObjectURL(customFontUrl.value)
  customFontUrl.value = URL.createObjectURL(file)
  if (!customFontStyleEl) {
    customFontStyleEl = document.createElement('style')
    document.head.appendChild(customFontStyleEl)
  }
  customFontStyleEl.textContent = `@font-face{font-family:'ReaderCustomFont';src:url("${customFontUrl.value}") format("truetype");font-display:swap;}`
}
async function onCustomFontPick(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  const ok = await saveCustomFont(file)
  if (!ok) {
    ElMessage.error('字体保存失败（浏览器存储不可用）')
    return
  }
  applyCustomFontUrl(file)
  customFontEnabled.value = true
  ElMessage.success(`已启用自定义字体「${file.name}」`)
}
async function clearCustomFont() {
  await removeCustomFont()
  if (customFontUrl.value) {
    URL.revokeObjectURL(customFontUrl.value)
    customFontUrl.value = ''
  }
  if (customFontStyleEl) {
    customFontStyleEl.textContent = ''
  }
  customFontEnabled.value = false
  ElMessage.success('已移除自定义字体')
}
// 点击其他区域关闭字体下拉
function onDocClick(e: MouseEvent) {
  if (fontOpen.value && !(e.target as HTMLElement)?.closest('.font-picker')) {
    fontOpen.value = false
  }
}
onMounted(() => document.addEventListener('mousedown', onDocClick))
onBeforeUnmount(() => document.removeEventListener('mousedown', onDocClick))
const fontFamilyStyle = computed(() => {
  if (customFontUrl.value && customFontEnabled.value) {
    return "'ReaderCustomFont', 'PingFang SC', 'Microsoft YaHei', sans-serif"
  }
  return FONT_STACK[fontKind.value]
})

/* ---------------- 2.2 字距 / 首行缩进 / 对齐 ---------------- */

const letterSpacing = ref(0)
watch(letterSpacing, (v) => saveSetting('reader_letter_spacing', v))

const textIndent = ref(true)
watch(textIndent, (v) => saveSetting('reader_text_indent', v ? '1' : '0'))

const textAlign = ref<'left' | 'justify'>('left')
watch(textAlign, (v) => saveSetting('reader_text_align', v))

const settingsOpen = ref(false)
/** 移动端点击中间区域唤出/收起顶部工具栏（legacy 点击区域交互） */
const chromeHidden = ref(false)
/** 正文编辑（legacy saveBookContent：编辑当前章并保存服务器 + 本机缓存） */
const editOpen = ref(false)
const editText = ref('')
const editSaving = ref(false)
function resetTypography() {
  fontSize.value = 18
  lineHeight.value = 1.9
  paraSpacing.value = 1
  fontWeight.value = 400
  fontKind.value = 'system'
  letterSpacing.value = 0
  textIndent.value = true
  textAlign.value = 'left'
  brightness.value = 1
}

/* ---------------- 3. 翻页模式（滚动 / 上下翻页 / 左右滑动翻章 / 仿真翻页） ---------------- */

const pageMode = ref<PageMode>('scroll')
watch(pageMode, (m) => saveSetting('reader_page_mode', m))
/** 点击区域翻页开关（legacy 点击方式：左上上一页/右下下一页/中间菜单；默认开） */
const tapZonesEnabled = ref(localStorage.getItem('reader_tap_zones') !== '0')
watch(tapZonesEnabled, (v) => {
  try {
    localStorage.setItem('reader_tap_zones', v ? '1' : '0')
  } catch {
    /* ignore */
  }
})

/* ---------------- legacy quickKey：自定义快捷键（localStorage JSON：e.code → action） ---------------- */

const QUICK_KEY_STORAGE = 'reader_quick_keys'
const QUICK_KEY_ACTIONS: { value: string; label: string }[] = [
  { value: 'nextChapter', label: '下一章' },
  { value: 'prevChapter', label: '上一章' },
  { value: 'nextPage', label: '下一页（仿真）' },
  { value: 'prevPage', label: '上一页（仿真）' },
  { value: 'toggleMenu', label: '唤出/收起菜单' },
  { value: 'toggleTts', label: '听书' },
  { value: 'toggleAuto', label: '自动阅读' },
  { value: 'openToc', label: '目录' },
  { value: 'addBookmark', label: '添加书签' },
]
const quickKeys = ref<Record<string, string>>({})
{
  try {
    const raw = JSON.parse(localStorage.getItem(QUICK_KEY_STORAGE) ?? '{}') as unknown
    if (raw && typeof raw === 'object') {
      const valid = new Set(QUICK_KEY_ACTIONS.map((a) => a.value))
      for (const [k, v] of Object.entries(raw)) {
        if (typeof v === 'string' && valid.has(v)) quickKeys.value[k] = v
      }
    }
  } catch {
    /* ignore */
  }
}
function persistQuickKeys() {
  try {
    localStorage.setItem(QUICK_KEY_STORAGE, JSON.stringify(quickKeys.value))
  } catch {
    /* ignore */
  }
}
const quickKeysText = ref(JSON.stringify(quickKeys.value, null, 2))
function applyQuickKeys() {
  try {
    const raw = JSON.parse(quickKeysText.value) as Record<string, unknown>
    const next: Record<string, string> = {}
    const valid = new Set(QUICK_KEY_ACTIONS.map((a) => a.value))
    for (const [k, v] of Object.entries(raw)) {
      if (typeof v === 'string' && valid.has(v)) next[k] = v
    }
    quickKeys.value = next
    persistQuickKeys()
    ElMessage.success('快捷键已应用')
  } catch {
    ElMessage.warning('快捷键 JSON 解析失败（格式：{"KeyA":"nextChapter"}）')
  }
}
function resetQuickKeys() {
  quickKeys.value = {}
  quickKeysText.value = '{}'
  persistQuickKeys()
  ElMessage.success('已恢复默认快捷键')
}

/** 切章方向：1=下一章（新正文自右滑入）/ -1=上一章（自左滑入）；hslide 模式切章过渡动画用 */
const chapterDir = ref<1 | -1>(1)
/** hslide 模式切章过渡动画类（正文重新挂载时播放一次） */
const chapterAnimClass = computed(() =>
  pageMode.value !== 'hslide'
    ? {}
    : chapterDir.value === 1
      ? { 'chapter-slide-in-right': true }
      : { 'chapter-slide-in-left': true },
)
/** 正文样式（flip 模式叠加多栏分页：列宽 = 容器宽，一列一页） */
const contentStyle = computed(() => ({
  fontSize: `${fontSize.value}px`,
  lineHeight: `${lineHeight.value}`,
  fontWeight: `${fontWeight.value}`,
  fontFamily: fontFamilyStyle.value || undefined,
  letterSpacing: letterSpacing.value > 0 ? `${letterSpacing.value}px` : undefined,
  textAlign: textAlign.value,
  ...(pageMode.value === 'flip' && flipColWidth.value > 0
    ? { columnWidth: `${flipColWidth.value}px`, columnGap: `${FLIP_GAP}px`, height: '100%' }
    : {}),
}))

/* ---- 上下翻页（slide）：滚轮/触屏纵向手势/浮动按钮逐屏滚动 ---- */

let slideAcc = 0
let slideCooldown = false
let touchStartY = 0
const SLIDE_THRESHOLD = 48
const SLIDE_PAGE = 0.9

function slideFlip(dir: 1 | -1) {
  if (slideCooldown) return
  slideCooldown = true
  window.setTimeout(() => {
    slideCooldown = false
  }, 420)
  window.scrollBy({ top: dir * window.innerHeight * SLIDE_PAGE, behavior: 'smooth' })
}
function isInsideOverlay(el: EventTarget | null): boolean {
  return el instanceof HTMLElement && !!el.closest('.drawer-mask, .pop-mask, .sel-bar')
}
function onWheel(e: WheelEvent) {
  if (isInsideOverlay(e.target)) return
  if (pageMode.value === 'slide') {
    e.preventDefault()
    slideAcc += e.deltaY
    if (slideAcc >= SLIDE_THRESHOLD) {
      slideAcc = 0
      slideFlip(1)
    } else if (slideAcc <= -SLIDE_THRESHOLD) {
      slideAcc = 0
      slideFlip(-1)
    }
  } else if (pageMode.value === 'flip' && isTextBook.value) {
    // 仿真翻页：纵向/横向滚轮增量都折算为翻页
    e.preventDefault()
    slideAcc += e.deltaY + e.deltaX
    if (slideAcc >= SLIDE_THRESHOLD) {
      slideAcc = 0
      flipPage(1)
    } else if (slideAcc <= -SLIDE_THRESHOLD) {
      slideAcc = 0
      flipPage(-1)
    }
  }
}
function onTouchStart(e: TouchEvent) {
  touchStartY = e.touches[0]?.clientY ?? 0
}
function onTouchMove(e: TouchEvent) {
  if (pageMode.value !== 'slide' || isInsideOverlay(e.target)) return
  e.preventDefault()
}
function onTouchEnd(e: TouchEvent) {
  if (pageMode.value !== 'slide' || isInsideOverlay(e.target)) return
  const y = e.changedTouches[0]?.clientY ?? touchStartY
  const dy = touchStartY - y
  if (dy >= SLIDE_THRESHOLD) slideFlip(1)
  else if (dy <= -SLIDE_THRESHOLD) slideFlip(-1)
}

/* ---- 仿真翻页（flip）：CSS 多栏分页（列高 = 视口高，一列一页）+ 横向平滑页过渡 ---- */

/** 分页容器（flip 模式下正文横向分页滚动；其余模式为普通包裹层） */
const flipViewRef = ref<HTMLElement | null>(null)
/** 当前列宽（= 容器内容宽，进入 flip/窗口尺寸/内容宽度变化时重测） */
const flipColWidth = ref(0)
/** 当前所在页（0 基；初始 -1 表示尚未分页，用于禁用上一页按钮） */
const flipPageIdx = ref(-1)
/** 列间距（px） */
const FLIP_GAP = 48
const flipStep = () => flipColWidth.value + FLIP_GAP

function isFlipMode(): boolean {
  return pageMode.value === 'flip' && isTextBook.value
}

/** 重测分页列宽（容器宽度变化/进入 flip 模式/换章渲染后调用） */
function measureFlipColumns() {
  const v = flipViewRef.value
  if (!v) return
  flipColWidth.value = v.clientWidth
  flipPageIdx.value = 0
}

/** flip 模式当前横向位置（列轴 px；进度存取用） */
function flipScrollLeft(): number {
  return flipViewRef.value?.scrollLeft ?? 0
}

/** 平滑翻到指定页（越界自动钳制；无分页容器时为空操作） */
function flipGo(page: number) {
  const v = flipViewRef.value
  if (!v) return
  const step = flipStep()
  if (step <= 0) return
  const maxPage = Math.max(0, Math.ceil(v.scrollWidth / step) - 1)
  const target = Math.min(maxPage, Math.max(0, page))
  v.scrollTo({ left: target * step, behavior: 'smooth' })
}

/** 翻一页（dir：1=下一页 / -1=上一页） */
function flipPage(dir: 1 | -1) {
  const v = flipViewRef.value
  if (!v) return
  const step = flipStep()
  if (step <= 0) return
  flipGo(Math.round(v.scrollLeft / step) + dir)
}

/** flip 模式滚动同步：页号/进度/定时保存；近尾（最后 1.5 页）加载下一段并重测分页 */
function onFlipScroll() {
  const v = flipViewRef.value
  if (!v) return
  const step = flipStep()
  flipPageIdx.value = step > 0 ? Math.round(v.scrollLeft / step) : 0
  updateScrollFrac()
  hideSelBar()
  window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(saveProgress, 300)
  if (v.scrollLeft + v.clientWidth >= v.scrollWidth - step * 1.5) {
    if (maybeLoadMoreChunk()) {
      void nextTick(() => measureFlipColumns())
    }
  }
}

/** 窗口尺寸变化：flip 模式重测列宽 + 刷新进度 */
function onResize() {
  if (isFlipMode()) measureFlipColumns()
  updateScrollFrac()
}

watch(pageMode, (mode, old) => {
  const needWheel = mode === 'slide' || mode === 'flip'
  const hadWheel = old === 'slide' || old === 'flip'
  if (needWheel && !hadWheel) {
    window.addEventListener('wheel', onWheel, { passive: false })
    window.addEventListener('touchstart', onTouchStart, { passive: true })
    window.addEventListener('touchmove', onTouchMove, { passive: false })
    window.addEventListener('touchend', onTouchEnd, { passive: true })
  } else if (!needWheel && hadWheel) {
    window.removeEventListener('wheel', onWheel)
    window.removeEventListener('touchstart', onTouchStart)
    window.removeEventListener('touchmove', onTouchMove)
    window.removeEventListener('touchend', onTouchEnd)
  }
  if (mode === 'flip' && isTextBook.value) {
    // 进入仿真翻页：回到章首并重测分页
    window.scrollTo(0, 0)
    void nextTick(() => measureFlipColumns())
  } else if (old === 'flip' && isTextBook.value) {
    // 离开仿真翻页：列轴位置换算为纵向位置（同为距章首 px，直接沿用）
    window.scrollTo(0, flipScrollLeft())
  }
})

/** 正文内容宽度变化（720/900/1080）→ flip 模式重测列宽 */
watch(contentWidth, () => {
  if (isFlipMode()) void nextTick(() => measureFlipColumns())
})

/* ---- 左右滑动（hslide）：横向滑动手势/浮动按钮 → 章节级翻页（GAP 85 手势复用 + 切章过渡动画） ---- */

let swipeStartX = 0
let swipeStartY = 0
let swipeTracking = false
const SWIPE_THRESHOLD = 60

function onSwipeTouchStart(e: TouchEvent) {
  if (isInsideOverlay(e.target)) return
  const t = e.touches[0]
  if (!t) return
  swipeStartX = t.clientX
  swipeStartY = t.clientY
  swipeTracking = true
}

function onSwipeTouchEnd(e: TouchEvent) {
  if (!swipeTracking) return
  swipeTracking = false
  // 任一弹层/抽屉/划词工具条打开时不翻页/翻章
  if (
    selOpen.value ||
    drawerOpen.value ||
    settingsOpen.value ||
    ttsPanelOpen.value ||
    bookmarksOpen.value ||
    jumpOpen.value ||
    searchOpen.value ||
    brightnessOpen.value ||
    customOpen.value ||
    imgViewerOpen.value
  ) {
    return
  }
  const t = e.changedTouches[0]
  if (!t) return
  const dx = t.clientX - swipeStartX
  const dy = t.clientY - swipeStartY
  // 横向主导（|dx|>=60 且明显大于 |dy|）→ 仿真翻页翻页 / 其余模式翻章；纵向滚动/划词选择不处理
  if (Math.abs(dx) < SWIPE_THRESHOLD) return
  if (Math.abs(dy) > Math.abs(dx) * 1.2) return
  if ((window.getSelection()?.toString().trim() ?? '') !== '') return
  if (isFlipMode()) {
    if (dx < 0) flipPage(1)
    else flipPage(-1)
    return
  }
  if (dx < 0) nextChapter()
  else prevChapter()
}

/* ---------------- 屏幕常亮（Wake Lock：进入阅读页且页面可见时请求；切后台/离开释放；不支持环境静默跳过） ---------------- */

const WAKE_LOCK_KEY = 'reader_wake_lock'
/** 默认开启；localStorage 存 '0' 关闭 */
const wakeLockEnabled = ref(localStorage.getItem(WAKE_LOCK_KEY) !== '0')
watch(wakeLockEnabled, (v) => {
  persist(WAKE_LOCK_KEY, v ? '1' : '0')
  if (v) {
    if (document.visibilityState === 'visible') void requestWakeLock()
  } else {
    void releaseWakeLock()
  }
})

/** 页面可见性变化：可见 → 重新请求常亮；隐藏 → 释放 */
function onWakeVisibilityChange() {
  if (document.visibilityState === 'visible') {
    if (wakeLockEnabled.value) void requestWakeLock()
  } else {
    void releaseWakeLock()
  }
}

/* ---------------- 4. 进度显示 + 章节跳转 ---------------- */

const scrollFrac = ref(0)
const jumpOpen = ref(false)
const jumpNum = ref('')

function updateScrollFrac() {
  const v = flipViewRef.value
  if (isFlipMode() && v) {
    const max = v.scrollWidth - v.clientWidth
    scrollFrac.value = max > 0 ? Math.min(1, Math.max(0, v.scrollLeft / max)) : 0
    return
  }
  const max = document.documentElement.scrollHeight - window.innerHeight
  scrollFrac.value = max > 0 ? Math.min(1, Math.max(0, window.scrollY / max)) : 0
}
const progressPct = computed(() => {
  const n = realChapters.value.length
  if (!n) return 0
  // 非文本书：音频/视频按播放进度、漫画按页进度（scrollFrac 对媒体页无意义）
  if (isAudioBook.value || isVideoBook.value) {
    const dur = isAudioBook.value ? audioDuration.value : videoDuration.value
    const cur = isAudioBook.value ? audioCurrent.value : videoCurrent.value
    const frac = dur > 0 ? Math.min(1, Math.max(0, cur / dur)) : 0
    return Math.min(100, Math.max(0, Math.round(((flatIndex.value + frac) / n) * 100)))
  }
  if (isComicBook.value) {
    const total = comicImages.value.length
    const frac = total > 1 ? comicPage.value / (total - 1) : 0
    return Math.min(100, Math.max(0, Math.round(((flatIndex.value + frac) / n) * 100)))
  }
  const raw = ((flatIndex.value + scrollFrac.value) / n) * 100
  return Math.min(100, Math.max(0, Math.round(raw)))
})
function confirmJump() {
  const n = parseInt(jumpNum.value, 10)
  if (Number.isNaN(n) || n < 1 || n > realChapters.value.length) return
  const target = realChapters.value[n - 1]
  const idx = chapters.value.indexOf(target)
  jumpOpen.value = false
  jumpNum.value = ''
  if (idx >= 0) goToChapter(idx)
}

/* ---------------- 4.1 书签（服务端 /reader3/bookmarks） ---------------- */

const bookmarksOpen = ref(false)
const bookmarks = ref<Bookmark[]>([])
const bookmarkLoading = ref(false)
/** 书签多选集合（书签标题集合；空 = 非多选模式） */
const bookmarkSelected = ref<Set<string>>(new Set())
/** 书签编辑弹窗状态（null = 关闭；对象 = 正在编辑的书签副本） */
const bookmarkEditing = ref<Bookmark | null>(null)
/** 书签编辑是否正在保存 */
const bookmarkSaving = ref(false)
/** 书签 JSON 导入文件输入 */
const bookmarkImportRef = ref<HTMLInputElement | null>(null)
/** 书签跳转待恢复的段落序号（跨章时随 loadContent 消费） */
let restoreParagraphIdx: number | null = null

/** 书签所属书名（书架书优先，临时详情兜底） */
const bookAuthor = computed(
  () => tempInfo.value?.author || shelfBook.value?.author || '',
)

async function openBookmarks() {
  bookmarksOpen.value = true
  bookmarkSelected.value = new Set()
  bookmarkLoading.value = true
  try {
    const res = await get<Bookmark[]>('/getBookmarks', { bookUrl: bookUrl.value })
    bookmarks.value = res.data ?? []
  } catch {
    bookmarks.value = []
  } finally {
    bookmarkLoading.value = false
  }
}

/** 当前视口顶部附近的段落序号（书签锚点） */
function topParagraphIndex(): number {
  const paras = document.querySelectorAll<HTMLElement>('.reader-content .reader-para')
  let idx = 0
  paras.forEach((p, i) => {
    if (p.getBoundingClientRect().top <= 80) idx = i
  })
  return idx
}

async function addBookmark() {
  const ch = currentChapter.value
  if (!ch || !bookUrl.value) return
  const paraIdx = topParagraphIndex()
  const anchor = paragraphs.value[paraIdx]?.trim() ?? ''
  const title = anchor.slice(0, 24) || ch.title
  try {
    await saveBookmark({
      bookUrl: bookUrl.value,
      title,
      paragraphIndex: paraIdx,
      chapterIndex: chapterIndex.value,
      bookName: bookName.value,
      bookAuthor: bookAuthor.value,
      chapterName: ch.title,
      bookText: anchor,
      content: '',
      createdAt: Date.now(),
    })
    if (bookmarksOpen.value) await openBookmarks()
    ElMessage.success('已添加书签')
  } catch {
    /* request.ts 已提示 */
  }
}

function openEditChapter() {
  if (loading.value || loadError.value || !currentChapter.value) return
  editText.value = paragraphs.value.join('\n')
  editOpen.value = true
}

function closeEditChapter() {
  if (editSaving.value) return
  editOpen.value = false
}

async function saveEditChapter() {
  const ch = currentChapter.value
  if (!ch || editSaving.value) return
  const newContent = editText.value
  if (!newContent.trim()) {
    ElMessage.warning('正文不能为空')
    return
  }
  editSaving.value = true
  try {
    await post('/saveBookContent', {
      bookUrl: bookUrl.value,
      chapterUrl: ch.url,
      title: ch.title,
      content: newContent,
    })
    await saveLocalChapter({
      bookUrl: bookUrl.value,
      chapterUrl: ch.url,
      title: ch.title,
      index: flatIndex.value,
      content: newContent,
    })
    content.value = newContent
    resetSegments()
    editOpen.value = false
    void loadCacheMarkers()
    ElMessage.success('正文已保存')
  } catch {
    /* 错误提示已由拦截器统一处理 */
  } finally {
    editSaving.value = false
  }
}

/** 划词添加书签（把选中文本作为书签正文） */
async function addBookmarkFromSelection() {
  const text = selText.value.trim()
  const ch = currentChapter.value
  clearSelection()
  hideSelBar()
  if (!text || !ch || !bookUrl.value) return
  const paraIdx = topParagraphIndex()
  try {
    await saveBookmark({
      bookUrl: bookUrl.value,
      title: text.slice(0, 24),
      paragraphIndex: paraIdx,
      chapterIndex: chapterIndex.value,
      bookName: bookName.value,
      bookAuthor: bookAuthor.value,
      chapterName: ch.title,
      bookText: text,
      content: '',
      createdAt: Date.now(),
    })
    ElMessage.success('已添加书签')
  } catch {
    /* request.ts 已提示 */
  }
}

/** 划词添加过滤规则（选中文本作为替换规则 find，替换为空） */
async function addFilterFromSelection() {
  const text = selText.value.trim()
  clearSelection()
  hideSelBar()
  if (!text) return
  const rules = loadReplaceRules()
  if (rules.some((r) => r.find === text)) {
    ElMessage.info('已存在相同过滤规则')
    return
  }
  rules.push({
    id: `filter-${Date.now()}`,
    name: `过滤：${text.slice(0, 12)}`,
    find: text,
    replace: '',
    enabled: true,
    order: rules.length,
  })
  await saveReplaceRules(rules)
  refreshReplaceRules()
  ElMessage.success('已添加过滤规则')
}

/** 编辑书签（打开弹窗；副本保存到 bookmarkEditing） */
function editBookmark(b: Bookmark) {
  bookmarkEditing.value = { ...b }
}

async function saveBookmarkEdit() {
  const bm = bookmarkEditing.value
  if (!bm) return
  const title = bm.title.trim()
  if (!title) {
    ElMessage.warning('书签标题不能为空')
    return
  }
  bookmarkSaving.value = true
  try {
    await saveBookmark(bm)
    bookmarkEditing.value = null
    await openBookmarks()
    ElMessage.success('书签已更新')
  } catch {
    /* request.ts 已提示 */
  } finally {
    bookmarkSaving.value = false
  }
}

/** 批量删除选中的书签 */
async function deleteSelectedBookmarks() {
  const titles = [...bookmarkSelected.value]
  if (titles.length === 0) return
  try {
    const res = await deleteBookmarks(bookUrl.value, titles)
    ElMessage.success(`已删除 ${res.data?.count ?? titles.length} 条书签`)
    bookmarkSelected.value = new Set()
    await openBookmarks()
  } catch {
    /* request.ts 已提示 */
  }
}

/** 从 JSON 文件导入书签（数组或单对象；bookUrl 缺失时用当前书 URL） */
async function importBookmarksFile(file: File) {
  const text = await file.text()
  let parsed: Bookmark[]
  try {
    parsed = parseBookmarksJson(text, bookUrl.value)
  } catch {
    ElMessage.error('书签 JSON 解析失败')
    return
  }
  if (parsed.length === 0) {
    ElMessage.warning('未找到有效书签数据')
    return
  }
  const res = await saveBookmarks(parsed)
  ElMessage.success(`已导入 ${res.data?.count ?? parsed.length} 条书签`)
  await openBookmarks()
}

function onBookmarkImportChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) void importBookmarksFile(file)
  input.value = ''
}

/** 导出当前书签为 JSON 文件 */
function exportBookmarksJson() {
  if (bookmarks.value.length === 0) {
    ElMessage.info('暂无书签可导出')
    return
  }
  const blob = new Blob([JSON.stringify(bookmarks.value, null, 2)], {
    type: 'application/json',
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `bookmarks-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function toggleBookmarkSelect(title: string) {
  const next = new Set(bookmarkSelected.value)
  if (next.has(title)) next.delete(title)
  else next.add(title)
  bookmarkSelected.value = next
}

function chapterTitleAt(idx: number): string {
  const ch = chapters.value[idx]
  return ch ? hanConvert(ch.title) : ''
}

/** 书签跳转：同章直接滚段落；跨章切章后按段落序号恢复 */
function jumpToBookmark(b: Bookmark) {
  bookmarksOpen.value = false
  const ch = chapters.value[b.chapterIndex]
  if (!ch || ch.isVolume) return
  restoreParagraphIdx = b.paragraphIndex
  if (b.chapterIndex === chapterIndex.value) {
    void applyRestoreParagraph()
  } else {
    goToChapter(b.chapterIndex)
  }
}

async function deleteBookmarkItem(b: Bookmark) {
  try {
    await post('/deleteBookmark', { bookUrl: bookUrl.value, title: b.title })
    bookmarks.value = bookmarks.value.filter((x) => x.title !== b.title)
    ElMessage.success('已删除书签')
  } catch {
    /* request.ts 已提示 */
  }
}

function fmtBookmarkTime(ts: number): string {
  if (!ts) return ''
  const d = new Date(ts)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

/** 按段落序号滚动恢复（含图片撑高后的二次校正） */
async function applyRestoreParagraph() {
  if (restoreParagraphIdx == null) return
  // GAP 155：目标段未渲染时先逐块加载（书签跨段跳转）
  await ensureParagraphRendered(restoreParagraphIdx)
  await nextTick()
  await settleFrames()
  scrollToParagraph(restoreParagraphIdx)
  await imagesReady()
  await settleFrames()
  scrollToParagraph(restoreParagraphIdx)
  restoreParagraphIdx = null
  updateScrollFrac()
}
function scrollToParagraph(idx: number) {
  const p = document.querySelectorAll<HTMLElement>('.reader-content .reader-para')[idx]
  if (!p) return
  if (isFlipMode()) {
    // 仿真翻页：滚动到该段所在列（换算为容器滚动坐标）
    const v = flipViewRef.value
    if (!v) return
    const target = p.getBoundingClientRect().left - v.getBoundingClientRect().left + v.scrollLeft
    v.scrollTo({ left: Math.max(0, target), behavior: 'smooth' })
    return
  }
  const top = Math.max(0, p.getBoundingClientRect().top + window.scrollY - 64)
  window.scrollTo(0, top)
}

/* ---------------- 4.2 页内搜索（前端本地：当前章正文关键词 <mark> 高亮 + 上一个/下一个跳转 + 计数） ---------------- */

const searchOpen = ref(false)
const searchKeyword = ref('')
const searchSearched = ref(false)
/** 每段命中数（与 paragraphs 对齐；未参与搜索时为全 0） */
const paraMatchCounts = ref<number[]>([])
/** 当前命中序号（全局 0 基；-1 = 无命中/未定位） */
const curMatch = ref(-1)
/** 命中总数 */
const matchTotal = computed(() => paraMatchCounts.value.reduce((a, b) => a + b, 0))
/** 搜索是否生效（已搜索且有非空关键词 → 正文渲染 <mark> 高亮） */
const searchActive = computed(() => searchSearched.value && searchKeyword.value.trim().length > 0)
/** 跳转后短暂高亮的目标段落 */
const flashParaIdx = ref(-1)
const searchInputRef = ref<HTMLInputElement | null>(null)
let flashTimer: number | undefined
/** 单段命中上限（防超长段死循环） */
const SEARCH_MAX_PER_PARA = 500

function openChapterSearch() {
  searchKeyword.value = ''
  searchSearched.value = false
  paraMatchCounts.value = []
  curMatch.value = -1
  searchOpen.value = true
  void nextTick(() => searchInputRef.value?.focus())
}

/** 关闭页内搜索并清除全部高亮状态 */
function closeChapterSearch() {
  searchOpen.value = false
  searchKeyword.value = ''
  searchSearched.value = false
  paraMatchCounts.value = []
  curMatch.value = -1
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

/** 全文高亮 HTML：对转义后的段落按关键词（大小写不敏感）把所有命中包 <mark> */
function highlightAll(text: string, kw: string): string {
  const esc = escapeHtml(text)
  const escKw = escapeHtml(kw)
  if (!escKw) return esc
  const lower = esc.toLowerCase()
  const kwLower = escKw.toLowerCase()
  const parts: string[] = []
  let from = 0
  let found = 0
  let i = lower.indexOf(kwLower, from)
  while (i !== -1 && found < SEARCH_MAX_PER_PARA) {
    parts.push(esc.slice(from, i), `<mark>${esc.slice(i, i + escKw.length)}</mark>`)
    from = i + escKw.length
    i = lower.indexOf(kwLower, from)
    found++
  }
  parts.push(esc.slice(from))
  return parts.join('')
}

/** 正文段落渲染：搜索生效且有命中 → 返回带 <mark> 的高亮 HTML；否则 null（走纯文本插值） */
function paraDisplayHtml(i: number): string | null {
  if (!searchActive.value || !paraMatchCounts.value[i]) return null
  return highlightAll(paragraphs.value[i], searchKeyword.value.trim())
}

/** 在当前章段落中统计每个关键词命中数（大小写不敏感；正文按简繁/替换规则转换后的可见文本匹配） */
function runChapterSearch() {
  const kw = searchKeyword.value.trim()
  searchSearched.value = true
  paraMatchCounts.value = []
  curMatch.value = -1
  if (!kw) return
  const lower = kw.toLowerCase()
  const counts: number[] = new Array(paragraphs.value.length).fill(0)
  paragraphs.value.forEach((p, i) => {
    if (!p.toLowerCase().includes(lower)) return
    let n = 0
    let from = 0
    while (n < SEARCH_MAX_PER_PARA) {
      const j = p.toLowerCase().indexOf(lower, from)
      if (j === -1) break
      n++
      from = j + kw.length
    }
    counts[i] = n
  })
  paraMatchCounts.value = counts
  const total = counts.reduce((a, b) => a + b, 0)
  if (total > 0) void scrollToMatch(0)
}

/** 全局命中序号 → 所在段下标 + 段内第几个 mark */
function locateMatch(globalIdx: number): { para: number; mark: number } {
  let acc = 0
  for (let i = 0; i < paraMatchCounts.value.length; i++) {
    const c = paraMatchCounts.value[i]
    if (globalIdx < acc + c) return { para: i, mark: globalIdx - acc }
    acc += c
  }
  return { para: -1, mark: -1 }
}

/** 跳转到第 globalIdx 个命中：滚动到对应段内的 <mark>（GAP 155：目标段未渲染时先加载对应块） */
async function scrollToMatch(globalIdx: number) {
  if (globalIdx < 0 || globalIdx >= matchTotal.value) return
  const { para, mark } = locateMatch(globalIdx)
  if (para < 0) return
  curMatch.value = globalIdx
  await ensureParagraphRendered(para)
  scrollToParagraph(para)
  await nextTick()
  const paraEl = document.querySelector(`.reader-content [data-para="${para}"]`)
  const markEl = paraEl?.querySelectorAll('mark')[mark]
  if (markEl instanceof HTMLElement) {
    // 当前命中 mark 加粗描边（其他命中淡黄底）
    paraEl?.querySelectorAll('mark').forEach((m) => m.classList.remove('mark-current'))
    markEl.classList.add('mark-current')
    const top = markEl.getBoundingClientRect().top + window.scrollY - window.innerHeight / 2
    window.scrollTo({ top: Math.max(0, top), behavior: 'smooth' })
  }
  flashParaIdx.value = para
  window.clearTimeout(flashTimer)
  flashTimer = window.setTimeout(() => {
    flashParaIdx.value = -1
  }, 1600)
}

/** 下一个命中（到尾回绕） */
function nextMatch() {
  if (matchTotal.value === 0) return
  const next = curMatch.value + 1 >= matchTotal.value ? 0 : curMatch.value + 1
  void scrollToMatch(next)
}

/** 上一个命中（到头回绕） */
function prevMatch() {
  if (matchTotal.value === 0) return
  const prev = curMatch.value - 1 < 0 ? matchTotal.value - 1 : curMatch.value - 1
  void scrollToMatch(prev)
}

/* ---------------- 5. 简繁转换（legacy chinese.js 移植） ---------------- */

/** 简繁模式（全局：reader_han_mode——auto 默认，检测繁体自动转简体） */
const hanMode = ref<HanMode>(getHanMode())
const hanTrad = ref(hanMode.value === 'trad')
watch(hanMode, (v) => {
  hanTrad.value = v === 'trad'
  saveSetting('reader_han_mode', v)
})
const detectTraditional = (text: string): boolean => detectTraditionalFn(text)
// chinese.ts 的完整转换表检测（更准）
import { detectTraditional as detectTraditionalFn } from '@/utils/chinese'
watch(content, (c) => {
  if (hanMode.value === 'auto') hanTrad.value = detectTraditional(c)
  // 换章/正文变化：页内搜索命中失效 → 清除高亮与计数
  closeChapterSearch()
  // GAP 155：正文变化 → 分段渲染重置为首块
  resetSegments()
})
function toggleHan() {
  // 三态循环：自动 → 简 → 繁 → 自动
  hanMode.value = hanMode.value === 'auto' ? 'simp' : hanMode.value === 'simp' ? 'trad' : 'auto'
  if (hanMode.value === 'auto') hanTrad.value = detectTraditional(content.value)
}
const hanConvert = (text: string) => applyHan(text, hanMode.value)
const hanTargetLabel = computed(() =>
  hanMode.value === 'auto'
    ? t('han.auto')
    : hanMode.value === 'trad'
      ? t('reader.hanToSimp')
      : t('reader.hanToTrad'),
)

/* ---------------- GAP 6：本书设置（per-book 覆盖 12 项——localStorage reader_book_config_{bookUrl}，本书优先于全局） ---------------- */

/** 本书配置（reader_book_config_{bookUrl}，键 = reader_* 键名，值为字符串） */
const bookOverrides = ref<Record<string, string>>({})
/** 设置面板当前 tab：global=全局偏好 / book=本书设置（默认：本书有覆盖则进本书 tab） */
const bookCfgTab = ref<'global' | 'book'>('global')

function persistBookOverrides() {
  if (bookUrl.value) saveBookConfig(bookUrl.value, bookOverrides.value)
}

/** 从 localStorage 装载本书配置并决定初始 tab */
function loadBookOverrides() {
  bookOverrides.value = bookUrl.value ? loadBookConfig(bookUrl.value) : {}
  bookCfgTab.value = Object.keys(bookOverrides.value).length > 0 ? 'book' : 'global'
}

/**
 * 设置持久化路由：本书 tab → 写入本书配置（覆盖全局）；全局 tab → 写全局键
 * 并清除同名本书覆盖（否则覆盖会静默压过全局修改）。
 */
function saveSetting(key: string, value: unknown) {
  if (bookCfgTab.value === 'book' && bookUrl.value) {
    const o = { ...bookOverrides.value }
    o[key] = String(value)
    bookOverrides.value = o
    persistBookOverrides()
    return
  }
  if (key in bookOverrides.value) {
    const o = { ...bookOverrides.value }
    delete o[key]
    bookOverrides.value = o
    persistBookOverrides()
  }
  if (key === 'reader_han_mode') setGlobalHanMode(value as HanMode)
  else persist(key, value)
}

/** 从全局 localStorage 重新装载 12 项设置 ref（初始化 / 恢复全局默认时调用） */
function loadGlobalIntoRefs() {
  const rawTheme = localStorage.getItem(THEME_KEY)
  // 旧值 paper（纸色）→ 迁移为 warm（暖色）
  theme.value =
    rawTheme === 'dark' || rawTheme === 'warm' || rawTheme === 'system' || rawTheme === 'custom'
      ? rawTheme
      : rawTheme === 'paper'
        ? 'warm'
        : 'light'
  fontSize.value = loadSetting(FONT_KEY, MIN_FONT, MAX_FONT, 18)
  lineHeight.value = loadSetting('reader_line_height', MIN_LINE, MAX_LINE, 1.9, 0.1)
  paraSpacing.value = loadSetting('reader_para_spacing', MIN_PARA, MAX_PARA, 1, 0.1)
  fontWeight.value = loadSetting('reader_font_weight', MIN_WEIGHT, MAX_WEIGHT, 400, 50)
  const w = localStorage.getItem('reader_content_width')
  contentWidth.value = w === '720px' || w === '1080px' ? w : '900px'
  const f = localStorage.getItem('reader_font_family')
  fontKind.value = FONT_OPTIONS.some((o) => o.value === f) ? (f as FontKind) : 'system'
  letterSpacing.value = loadSetting('reader_letter_spacing', 0, 2, 0, 0.5)
  textIndent.value = localStorage.getItem('reader_text_indent') !== '0'
  textAlign.value = localStorage.getItem('reader_text_align') === 'justify' ? 'justify' : 'left'
  pageMode.value = parsePageMode(localStorage.getItem('reader_page_mode'))
  const hm = localStorage.getItem('reader_han_mode')
  hanMode.value = hm === 'simp' || hm === 'trad' ? hm : 'auto'
}

/** 本书覆盖项应用到当前 refs（12 项；值形态与全局键一致，逐项校验） */
function applyBookOverridesToRefs() {
  const o = bookOverrides.value
  const get = (k: string) => o[k]
  const t = get(THEME_KEY)
  // 旧值 paper → 迁移为 warm
  if (t === 'light' || t === 'dark' || t === 'warm' || t === 'system' || t === 'custom') theme.value = t
  else if (t === 'paper') theme.value = 'warm'
  const fs = Number(get(FONT_KEY))
  if (fs >= MIN_FONT && fs <= MAX_FONT) fontSize.value = fs
  const lh = Number(get('reader_line_height'))
  if (lh >= MIN_LINE && lh <= MAX_LINE) lineHeight.value = Math.round(lh * 10) / 10
  const ps = Number(get('reader_para_spacing'))
  if (ps >= MIN_PARA && ps <= MAX_PARA) paraSpacing.value = Math.round(ps * 10) / 10
  const fw = Number(get('reader_font_weight'))
  if (fw >= MIN_WEIGHT && fw <= MAX_WEIGHT) fontWeight.value = Math.round(fw / 50) * 50
  const w = get('reader_content_width')
  if (w === '720px' || w === '900px' || w === '1080px') contentWidth.value = w
  const f = get('reader_font_family')
  if (FONT_OPTIONS.some((o2) => o2.value === f)) fontKind.value = f as FontKind
  const lsp = Number(get('reader_letter_spacing'))
  if (lsp >= 0 && lsp <= 2) letterSpacing.value = Math.round(lsp * 2) / 2
  const ti = get('reader_text_indent')
  if (ti === '1') textIndent.value = true
  else if (ti === '0') textIndent.value = false
  const ta = get('reader_text_align')
  if (ta === 'left' || ta === 'justify') textAlign.value = ta
  const pm = get('reader_page_mode')
  pageMode.value = parsePageMode(pm)
  const hm = get('reader_han_mode')
  if (hm === 'auto' || hm === 'simp' || hm === 'trad') hanMode.value = hm
}

/** 恢复全局默认：清除本书配置 → 回退全局偏好 */
async function restoreGlobalDefaults() {
  try {
    await ElMessageBox.confirm(
      '确定清除本书设置吗？本书将恢复使用全局阅读偏好（字体/字号/行距/主题等 12 项）。',
      '恢复全局默认',
      { confirmButtonText: '清除本书设置', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  if (bookUrl.value) clearBookConfig(bookUrl.value)
  bookOverrides.value = {}
  bookCfgTab.value = 'global'
  loadGlobalIntoRefs()
  ElMessage.success('已恢复全局默认（本书设置已清除）')
}

// 初始化顺序：全局值装载 → 本书覆盖装载（决定 tab）→ 覆盖应用（watcher 按 tab 路由持久化，同值不回写）
loadGlobalIntoRefs()
loadBookOverrides()
applyBookOverridesToRefs()

/* ---------------- GAP 4：自定义背景图（纯色/纸纹/图片——设置页选择器；图片经 file/download 展示 + 遮罩保证可读） ---------------- */

const bgMode = ref<BgMode>(loadBgMode())
const bgImagePath = ref(loadBgImagePath())
const bgPreset = ref(loadBgPreset())
/** 背景图 URL（file/download + accessToken；路径为空返回 ''） */
const bgImageUrl = computed(() => bgImageUrlOf(bgImagePath.value, store.accessToken))
/** 内置背景图 URL（vite public 静态资源，无 token） */
const bgPresetUrlValue = computed(() => bgPresetUrl(bgPreset.value))
/** 图片背景遮罩（按当前阅读主题取色，保证文字可读；custom 由 bgStyle 按背景明暗动态取 dark/light） */
const BG_OVERLAY: Record<Theme, string> = {
  light: 'rgba(250, 250, 250, 0.86)',
  dark: 'rgba(17, 17, 20, 0.88)',
  warm: 'rgba(247, 240, 230, 0.88)',
  system: 'rgba(250, 250, 250, 0.86)',
  custom: 'rgba(250, 250, 250, 0.86)',
}
/** 图片背景样式：遮罩渐变叠在图上（cover 铺满 + 固定），失败时回退 var(--bg) */
const bgStyle = computed(() => {
  const imgUrl =
    bgMode.value === 'preset' ? bgPresetUrlValue.value : bgMode.value === 'image' ? bgImageUrl.value : ''
  if (!imgUrl) return undefined
  let overlay: string
  if (theme.value === 'custom') {
    overlay = customThemeIsDark(customTheme.value) ? BG_OVERLAY.dark : BG_OVERLAY.light
  } else {
    const real = theme.value === 'system' ? systemTheme() : theme.value
    overlay = BG_OVERLAY[real] ?? BG_OVERLAY.light
  }
  return {
    backgroundImage: `linear-gradient(${overlay}, ${overlay}), url("${imgUrl}")`,
    backgroundSize: 'cover',
    backgroundPosition: 'center',
    backgroundAttachment: 'fixed',
  }
})
/** 阅读页根元素样式：背景图样式 + 自定义主题 CSS 变量（theme=custom 时注入三色及派生变量） */
const pageStyle = computed(() => {
  if (theme.value !== 'custom') return bgStyle.value
  return [bgStyle.value, customThemeVars(customTheme.value)] as Array<Record<string, string> | undefined>
})
/** 纸纹生效：设置页模式=纸纹，或阅读页开关开启（图片/内置图模式优先，不叠加纸纹） */
const effectiveTexture = computed(
  () =>
    (bgMode.value === 'color' || bgMode.value === 'texture') &&
    (paperTexture.value || bgMode.value === 'texture'),
)
/** 纸纹开关：与背景模式联动（设置页选择器同样写这两个键） */
function toggleTexture() {
  paperTexture.value = !paperTexture.value
  if (paperTexture.value) {
    if (bgMode.value === 'color') {
      bgMode.value = 'texture'
      saveBgMode('texture')
    }
  } else if (bgMode.value === 'texture') {
    bgMode.value = 'color'
    saveBgMode('color')
  }
}

/* ---------------- 6. 替换规则（localStorage: reader_replace_rules，见 api/replaceRules.ts 契约注释） ---------------- */

const REPLACE_KEY = 'reader_replace_enabled'
/** 阅读页总开关：是否应用替换规则（默认开；规则页另有每条规则的启用开关） */
const replaceEnabled = ref(true)
{
  const raw = localStorage.getItem(REPLACE_KEY)
  if (raw === '0') replaceEnabled.value = false
}
/** 当前生效的规则（仅 enabled 且 find 非空） */
const replaceRules = ref<ReplaceRule[]>([])

function refreshReplaceRules() {
  replaceRules.value = loadReplaceRules().filter((r) => r.enabled && r.find && r.find.trim().length > 0)
}

/** 逐条 replaceAll（字面替换，非正则）；空 find 已在上层过滤 */
function applyReplace(text: string): string {
  let out = text
  for (const r of replaceRules.value) {
    out = out.split(r.find).join(r.replace ?? '')
  }
  return out
}

watch(replaceEnabled, (v) => {
  persist(REPLACE_KEY, v ? '1' : '0')
  if (v) refreshReplaceRules()
})

/* ---------------- 7. 听书（后端语音合成：POST /reader3/tts → blob → audio 元素播放；Edge/HttpTTS 引擎；本章播完自动下一章） ---------------- */

const TTS_DEFAULT_VOICE = 'zh-CN-XiaoxiaoNeural'
/** 后端单次合成文本上限（service/tts.rs MAX_TEXT_CHARS） */
const TTS_MAX_CHARS = 20000
const TTS_VOICE_KEY = 'reader_tts_voice'
const TTS_RATE_KEY = 'reader_tts_rate'
const TTS_PITCH_KEY = 'reader_tts_pitch'
const TTS_VOLUME_KEY = 'reader_tts_volume'
const TTS_STYLE_KEY = 'reader_tts_style'
const TTS_ENGINE_KEY = 'reader_tts_engine'
const TTS_HTTP_URL_KEY = 'reader_tts_http_url'

type TtsState = 'idle' | 'loading' | 'playing' | 'paused'
const ttsState = ref<TtsState>('idle')
const ttsPanelOpen = ref(false)
const ttsVoices = ref<TtsVoice[]>([])
const ttsVoicesLoaded = ref(false)
const ttsHttpList = ref<HttpTts[]>([])
const ttsHttpLoaded = ref(false)
const ttsVoice = ref(TTS_DEFAULT_VOICE)
const ttsRate = ref(1)
const ttsPitch = ref(0)
/** 音量百分比 0-200（100 = +0%） */
const ttsVolume = ref(100)
const ttsStyle = ref('')
const ttsEngine = ref<'edge' | 'http'>('edge')
const ttsHttpUrl = ref('')
const ttsAudioRef = ref<HTMLAudioElement | null>(null)
/** 当前播放 blob 的 objectURL（换源/停止时 revoke） */
let ttsObjectUrl = ''
/** 合成请求序号：切章/停止时自增，丢弃过期结果 */
let ttsLoadSeq = 0
/** 本章播完、待连播下一章标记（loadContent 成功后消费） */
let ttsAutoNext = false

{
  const v = localStorage.getItem(TTS_VOICE_KEY)
  if (v) ttsVoice.value = v
  ttsRate.value = round1(loadSetting(TTS_RATE_KEY, 0.5, 2, 1, 0.1))
  ttsPitch.value = loadSetting(TTS_PITCH_KEY, -10, 10, 0)
  ttsVolume.value = loadSetting(TTS_VOLUME_KEY, 0, 200, 100)
  ttsStyle.value = localStorage.getItem(TTS_STYLE_KEY) ?? ''
  const e = localStorage.getItem(TTS_ENGINE_KEY)
  if (e === 'edge' || e === 'http') ttsEngine.value = e
  ttsHttpUrl.value = localStorage.getItem(TTS_HTTP_URL_KEY) ?? ''
}
watch(ttsVoice, (v) => persist(TTS_VOICE_KEY, v))
watch(ttsRate, (v) => persist(TTS_RATE_KEY, v))
watch(ttsPitch, (v) => persist(TTS_PITCH_KEY, v))
watch(ttsVolume, (v) => persist(TTS_VOLUME_KEY, v))
watch(ttsStyle, (v) => persist(TTS_STYLE_KEY, v))
watch(ttsEngine, (v) => persist(TTS_ENGINE_KEY, v))
watch(ttsHttpUrl, (v) => persist(TTS_HTTP_URL_KEY, v))

/** 播放中（顶栏按钮高亮） */
const ttsPlaying = computed(() => ttsState.value === 'playing')

/* ---------------- GAP 7：朗读段落高亮（播放中定时取当前段落，.tts-reading 浅背景） ---------------- */

const ttsReadingPara = ref(-1)
let ttsParaTimer: number | undefined
/** 朗读锚点线：视口高度 32% 处（当前段落 = 最后一个顶部越过锚点的段落） */
const TTS_READ_LINE = 0.32

function updateTtsReadingPara() {
  const paras = document.querySelectorAll<HTMLElement>('.reader-content .reader-para')
  const line = window.innerHeight * TTS_READ_LINE
  let idx = -1
  paras.forEach((p, i) => {
    if (p.getBoundingClientRect().top <= line) idx = i
  })
  if (ttsReadingPara.value !== idx) ttsReadingPara.value = idx
}

/** 播放/继续时启动定时器（暂停时保留高亮位置） */
function startTtsParaTracking() {
  stopTtsParaTracking()
  updateTtsReadingPara()
  ttsParaTimer = window.setInterval(updateTtsReadingPara, 400)
}

/** 暂停：仅停定时器，保留当前高亮段落 */
function pauseTtsParaTracking() {
  if (ttsParaTimer !== undefined) {
    window.clearInterval(ttsParaTimer)
    ttsParaTimer = undefined
  }
}

/** 停止/切章：停定时器并清除高亮 */
function stopTtsParaTracking() {
  pauseTtsParaTracking()
  ttsReadingPara.value = -1
}
/** 顶栏按钮文案 */
const ttsTopLabel = computed(() =>
  ttsState.value === 'playing' || ttsState.value === 'loading'
    ? t('reader.stop')
    : ttsState.value === 'paused'
      ? t('reader.resumeShort')
      : t('reader.ttsLabel'),
)

/** 语速 0.5-2.0 → Edge 百分比（+0% / +10% / -50%） */
const ttsRateParam = computed(() => {
  const pct = Math.round((ttsRate.value - 1) * 100)
  return `${pct >= 0 ? '+' : ''}${pct}%`
})
/** 音调 → Edge Hz（+0Hz / -2Hz） */
const ttsPitchParam = computed(() => `${ttsPitch.value >= 0 ? '+' : ''}${ttsPitch.value}Hz`)
/** 音量 → Edge 百分比（100 = +0%，50 = -50%，150 = +50%） */
const ttsVolumeParam = computed(() => {
  const pct = ttsVolume.value - 100
  return `${pct >= 0 ? '+' : ''}${pct}%`
})

/** 音色下拉按 locale 分组 */
const ttsLocaleGroups = computed(() => {
  const map = new Map<string, TtsVoice[]>()
  for (const v of ttsVoices.value) {
    const arr = map.get(v.locale) ?? []
    arr.push(v)
    map.set(v.locale, arr)
  }
  return Array.from(map.entries()).map(([label, voices]) => ({ label, voices }))
})

/** 首次打开面板时加载语音列表 + HttpTTS 列表（记忆值失效时回退默认） */
async function loadTtsOptions() {
  if (!ttsVoicesLoaded.value) {
    ttsVoicesLoaded.value = true
    try {
      const res = await getTtsVoices()
      ttsVoices.value = res.data ?? []
    } catch {
      ttsVoices.value = []
    }
    if (ttsVoices.value.length > 0 && !ttsVoices.value.some((v) => v.value === ttsVoice.value)) {
      ttsVoice.value = TTS_DEFAULT_VOICE
    }
  }
  if (!ttsHttpLoaded.value) {
    ttsHttpLoaded.value = true
    try {
      const res = await getHttpTtsList()
      ttsHttpList.value = res.data ?? []
    } catch {
      ttsHttpList.value = []
    }
    if (ttsHttpList.value.length > 0) {
      if (!ttsHttpList.value.some((t) => t.url === ttsHttpUrl.value)) {
        ttsHttpUrl.value = ttsHttpList.value[0].url
      }
    } else if (ttsEngine.value === 'http') {
      ttsEngine.value = 'edge'
    }
  }
}
watch(ttsPanelOpen, (open) => {
  if (open) void loadTtsOptions()
})

/** 朗读文本：正文段落（含替换规则/简繁转换），截断到后端上限 */
function ttsText(): string {
  return paragraphs.value.join('。').slice(0, TTS_MAX_CHARS)
}

/** 播放当前章：合成 → blob → audio 播放 */
async function startTts() {
  ttsSelectionMode = false // 整章朗读：结束允许自动连播
  const text = ttsText()
  if (!text) {
    ElMessage.info('本章暂无内容可朗读')
    return
  }
  await loadTtsOptions()
  const audio = ttsAudioRef.value
  if (!audio) return
  if (ttsEngine.value === 'http' && !ttsHttpUrl.value) {
    ElMessage.info('请先在设置页添加 HttpTTS 源')
    return
  }
  const seq = ++ttsLoadSeq
  ttsState.value = 'loading'
  let blob: Blob
  try {
    blob = await synthesizeTts({
      text,
      voice: ttsVoice.value,
      rate: ttsRateParam.value,
      pitch: ttsPitchParam.value,
      volume: ttsVolumeParam.value,
      style: ttsStyle.value || undefined,
      engine: ttsEngine.value,
      httpUrl: ttsEngine.value === 'http' ? ttsHttpUrl.value : undefined,
    })
  } catch (e) {
    if (seq === ttsLoadSeq) {
      ttsState.value = 'idle'
      ElMessage.error(e instanceof Error ? e.message : '语音合成失败')
    }
    return
  }
  const url = URL.createObjectURL(blob)
  if (seq !== ttsLoadSeq) {
    // 期间已停止/切章：丢弃过期结果
    URL.revokeObjectURL(url)
    return
  }
  if (ttsObjectUrl) URL.revokeObjectURL(ttsObjectUrl)
  ttsObjectUrl = url
  audio.pause()
  audio.src = url
  ttsState.value = 'playing'
  // 后台标签页被浏览器暂停时，切回自动恢复（防"听书几分钟后中断"）
  document.addEventListener(
    'visibilitychange',
    () => {
      const a = ttsAudioRef.value
      if (document.visibilityState === 'visible' && ttsState.value === 'playing' && a && a.paused && a.src) {
        void a.play().catch(() => {})
      }
    },
    { once: true },
  )
  try {
    await audio.play()
    startTtsParaTracking()
  } catch {
    // 自动播放被拦截（异步 fetch 后手势已失效）：保持待播，面板点「播放」即可恢复
    if (ttsState.value === 'playing') ttsState.value = 'paused'
  }
}

function stopTts() {
  ttsLoadSeq++
  ttsAutoNext = false
  ttsSelectionMode = false
  ttsState.value = 'idle'
  stopTtsParaTracking()
  const audio = ttsAudioRef.value
  if (audio) {
    audio.pause()
    audio.removeAttribute('src')
    audio.load()
  }
  if (ttsObjectUrl) {
    URL.revokeObjectURL(ttsObjectUrl)
    ttsObjectUrl = ''
  }
}

function pauseTts() {
  const audio = ttsAudioRef.value
  if (!audio || ttsState.value !== 'playing') return
  audio.pause()
  ttsState.value = 'paused'
  pauseTtsParaTracking()
}

function resumeTts() {
  const audio = ttsAudioRef.value
  if (!audio || ttsState.value !== 'paused') return
  void audio.play().catch(() => {
    /* 保持暂停 */
  })
  startTtsParaTracking()
}

/** 面板播放/暂停/继续 */
function ttsPlayPause() {
  if (ttsState.value === 'playing') pauseTts()
  else if (ttsState.value === 'paused') resumeTts()
  else if (ttsState.value === 'idle') void startTts()
}

/** 顶栏听书按钮：展开面板 + 播放/继续/停止 */
function toggleTts() {
  ttsPanelOpen.value = true
  if (ttsState.value === 'playing' || ttsState.value === 'loading') stopTts()
  else if (ttsState.value === 'paused') resumeTts()
  else void startTts()
}

/** 本章播完：自动连播下一章；最后一章则停止；划词朗读播完即止 */
function onTtsEnded() {
  if (ttsState.value === 'idle') return
  stopTtsParaTracking()
  if (ttsObjectUrl) {
    URL.revokeObjectURL(ttsObjectUrl)
    ttsObjectUrl = ''
  }
  const audio = ttsAudioRef.value
  if (audio) {
    audio.removeAttribute('src')
    audio.load()
  }
  // GAP 92：划词朗读播完即止（不自动切章）
  if (ttsSelectionMode) {
    ttsSelectionMode = false
    ttsState.value = 'idle'
    return
  }
  if (hasNext.value) {
    ttsAutoNext = true
    ttsState.value = 'idle'
    nextChapter()
  } else {
    ttsState.value = 'idle'
    ElMessage.info('本书已播放完毕')
  }
}

let ttsErrorRetries = 0
function onTtsError() {
  if (ttsState.value === 'idle') return
  const audio = ttsAudioRef.value
  if (audio && !audio.getAttribute('src')) return
  // 自动重试一次（网络/解码抖动），仍失败才停止
  if (ttsErrorRetries < 1 && audio && audio.src) {
    ttsErrorRetries++
    const src = audio.src
    audio.pause()
    audio.load()
    audio.src = src
    audio
      .play()
      .then(() => {
        ttsErrorRetries = 0
      })
      .catch(() => {
        stopTts()
        ElMessage.error('语音播放失败（已重试）')
      })
    return
  }
  ttsErrorRetries = 0
  stopTts()
  ElMessage.error('语音播放失败')
}

/* ---------------- GAP 92：划词朗读（选中文本 TTS——复用现有合成/播放流程，播完不自动切章） ---------------- */

/** 划词朗读模式：本章播完只停止不连播 */
let ttsSelectionMode = false

/** 朗读指定文本（划词朗读入口） */
async function speakText(text: string) {
  const clipped = text.slice(0, TTS_MAX_CHARS)
  if (!clipped.trim()) return
  await loadTtsOptions()
  const audio = ttsAudioRef.value
  if (!audio) return
  if (ttsEngine.value === 'http' && !ttsHttpUrl.value) {
    ElMessage.info('请先在设置页添加 HttpTTS 源')
    return
  }
  const seq = ++ttsLoadSeq
  ttsState.value = 'loading'
  let blob: Blob
  try {
    blob = await synthesizeTts({
      text: clipped,
      voice: ttsVoice.value,
      rate: ttsRateParam.value,
      pitch: ttsPitchParam.value,
      volume: ttsVolumeParam.value,
      style: ttsStyle.value || undefined,
      engine: ttsEngine.value,
      httpUrl: ttsEngine.value === 'http' ? ttsHttpUrl.value : undefined,
    })
  } catch (e) {
    if (seq === ttsLoadSeq) {
      ttsState.value = 'idle'
      ElMessage.error(e instanceof Error ? e.message : '语音合成失败')
    }
    return
  }
  const url = URL.createObjectURL(blob)
  if (seq !== ttsLoadSeq) {
    // 期间已停止/切章：丢弃过期结果
    URL.revokeObjectURL(url)
    return
  }
  if (ttsObjectUrl) URL.revokeObjectURL(ttsObjectUrl)
  ttsObjectUrl = url
  audio.pause()
  audio.src = url
  ttsState.value = 'playing'
  ttsSelectionMode = true
  try {
    await audio.play()
  } catch {
    // 自动播放被拦截（异步合成后手势已失效）：保持待播，面板点「播放」即可恢复
    if (ttsState.value === 'playing') ttsState.value = 'paused'
  }
}

/** 划词工具条「朗读」按钮：朗读选中文本 */
function speakSelection() {
  const text = selText.value
  clearSelection()
  hideSelBar()
  if (!text) return
  void speakText(text)
}

/* ---------------- 8. 自动阅读（定时滚动，到底自动切章） ---------------- */

const autoPlaying = ref(false)
const autoSpeed = ref(3)
autoSpeed.value = loadSetting('reader_auto_speed', 1, 5, 3)
watch(autoSpeed, (v) => persist('reader_auto_speed', v))
let autoTimer: number | undefined
const AUTO_BOTTOM_GAP = 40
const autoInterval = () => (6 - autoSpeed.value) * 1000

function autoTick() {
  if (loading.value || loadError.value || !currentChapter.value) return
  const doc = document.documentElement
  if (doc.scrollHeight - window.innerHeight <= 0) return
  // GAP 155：长章还有未渲染段且已近底——先加载下一块，不切章
  if (maybeLoadMoreChunk()) return
  const atBottom = window.scrollY + window.innerHeight >= doc.scrollHeight - AUTO_BOTTOM_GAP
  if (atBottom) {
    if (hasNext.value) nextChapter()
    else stopAuto()
    return
  }
  // 每 tick 滚动一行（行高 = 字号 × 行距）
  window.scrollBy({ top: fontSize.value * lineHeight.value, behavior: 'auto' })
}

function startAuto() {
  if (autoTimer != null) return
  autoPlaying.value = true
  autoTick()
  autoTimer = window.setInterval(autoTick, autoInterval())
}
function stopAuto() {
  autoPlaying.value = false
  if (autoTimer != null) {
    window.clearInterval(autoTimer)
    autoTimer = undefined
  }
}
function toggleAuto() {
  if (autoPlaying.value) stopAuto()
  else startAuto()
}
// 运行中调速：重建定时器
watch(autoSpeed, () => {
  if (autoPlaying.value && autoTimer != null) {
    window.clearInterval(autoTimer)
    autoTimer = window.setInterval(autoTick, autoInterval())
  }
})

/* ---------------- 9. 划词操作（复制 / 搜索） ---------------- */

const selText = ref('')
const selOpen = ref(false)
const selX = ref(0)
const selY = ref(0)

function hideSelBar() {
  selOpen.value = false
  selText.value = ''
}

function clearSelection() {
  window.getSelection()?.removeAllRanges()
}

/** mouseup/touchend 后延迟计算：确保选中态已就绪 */
function onSelectionUp() {
  window.setTimeout(computeSelection, 0)
}

function computeSelection() {
  const sel = window.getSelection()
  const text = sel?.toString().trim() ?? ''
  if (!text || !sel || sel.rangeCount === 0) {
    hideSelBar()
    return
  }
  const node = sel.anchorNode
  const el =
    node && node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as HTMLElement | null)
  if (!el || !el.closest('.reader-content')) {
    hideSelBar()
    return
  }
  const rect = sel.getRangeAt(0).getBoundingClientRect()
  if (rect.width === 0 && rect.height === 0) {
    hideSelBar()
    return
  }
  selText.value = text
  selX.value = Math.min(Math.max(rect.left + rect.width / 2, 60), window.innerWidth - 60)
  selY.value = Math.max(10, rect.top - 44)
  selOpen.value = true
}

/** 点击正文/他处收起工具条（mousedown 先于 mouseup，避免选中后立即被 click 误关） */
function onDocMouseDown(e: MouseEvent) {
  if (e.target instanceof HTMLElement && e.target.closest('.sel-bar')) return
  hideSelBar()
}

/** 点击区域是否被任一弹层/抽屉占用 */
function readerAreaOverlayOpen(): boolean {
  return (
    selOpen.value ||
    drawerOpen.value ||
    settingsOpen.value ||
    ttsPanelOpen.value ||
    bookmarksOpen.value ||
    jumpOpen.value ||
    searchOpen.value ||
    brightnessOpen.value ||
    customOpen.value ||
    imgViewerOpen.value ||
    sourceOpen.value ||
    retentionOpen.value
  )
}

/**
 * legacy 手机端点击区域：左上 1/3×1/3=上一页、右下 1/3×1/3=下一页、
 * 中间=唤出/收起菜单；滚动模式逐屏滚动，上下/仿真/左右模式沿用各自翻页语义
 */
function onReaderAreaClick(e: MouseEvent) {
  if (!tapZonesEnabled.value || readerAreaOverlayOpen()) return
  const t = e.target
  if (
    t instanceof HTMLElement &&
    t.closest('button, a, input, textarea, select, img, .chapter-nav, .reading-progress, .progress-bar, .flip-nav-btn, .flip-nav-vert')
  ) {
    return
  }
  if ((window.getSelection()?.toString().trim() ?? '') !== '') return
  const x = e.clientX / window.innerWidth
  const y = e.clientY / window.innerHeight
  if (x < 1 / 3 && y < 1 / 3) {
    tapPage(-1)
  } else if (x >= 2 / 3 && y >= 2 / 3) {
    tapPage(1)
  } else {
    chromeHidden.value = !chromeHidden.value
  }
}

/** 点击区域翻页：仿真翻页/左右滑动翻章/上下翻页按模式走，滚动模式逐屏滚动 */
function tapPage(dir: 1 | -1) {
  if (pageMode.value === 'flip' && isTextBook.value) {
    flipPage(dir)
  } else if (pageMode.value === 'hslide') {
    if (dir === 1) nextChapter()
    else prevChapter()
  } else if (pageMode.value === 'slide') {
    slideFlip(dir)
  } else {
    window.scrollBy({ top: dir * window.innerHeight * 0.9, behavior: 'smooth' })
  }
}

async function copySelection() {
  const text = selText.value
  clearSelection()
  hideSelBar()
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    try {
      const ta = document.createElement('textarea')
      ta.value = text
      document.body.appendChild(ta)
      ta.select()
      document.execCommand('copy')
      document.body.removeChild(ta)
    } catch {
      /* ignore */
    }
  }
  ElMessage.success('已复制')
}

function searchSelection() {
  const text = selText.value
  clearSelection()
  hideSelBar()
  if (!text) return
  void router.push({ path: '/search', query: { key: text } })
}

/* ---------------- GAP 124：复制本章（navigator.clipboard 全文；失败回退 execCommand；提示字数） ---------------- */

async function copyChapter() {
  const text = paragraphs.value.join('\n')
  if (!text) {
    ElMessage.info('本章暂无内容可复制')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    try {
      const ta = document.createElement('textarea')
      ta.value = text
      document.body.appendChild(ta)
      ta.select()
      document.execCommand('copy')
      document.body.removeChild(ta)
    } catch {
      ElMessage.error('复制失败，请手动长按选择复制')
      return
    }
  }
  ElMessage.success(`已复制本章（${text.length} 字）`)
}

/* ---------------- 10. 章节图片预加载（下一章前 5 张） ---------------- */

const preloadedChapters = new Set<string>()

/** 提取正文图片 URL：markdown 图片语法 + 裸图片扩展名 URL */
function extractImageUrls(text: string): string[] {
  const urls: string[] = []
  const mdRe = /!\[[^\]]*]\(\s*([^)\s]+)\s*\)/g
  let m: RegExpExecArray | null
  while ((m = mdRe.exec(text))) urls.push(m[1])
  const bareRe =
    /https?:\/\/[^\s"'<>，。！？、；：“”‘’（）【】]+\.(?:png|jpe?g|gif|webp|bmp|svg)(?:\?[^\s"'<>]*)?/gi
  let b: RegExpExecArray | null
  while ((b = bareRe.exec(text))) urls.push(b[0])
  return urls
}

/** 当前章渲染后调用：静默拉取下一章正文并预热前 5 张图片（仅当含图片 URL）；文本书专用 */
function preloadNextChapterImages() {
  if (!isTextBook.value) return
  if (!shelfBook.value?.origin) return
  const fi = flatIndex.value
  if (fi < 0 || fi >= realChapters.value.length - 1) return
  const next = realChapters.value[fi + 1]
  if (preloadedChapters.has(next.url)) return
  preloadedChapters.add(next.url)
  void getBookContent(next.url, shelfBook.value.origin, {
    timeout: chapterTimeout.value * 1000,
  })
    .then((res) => {
      for (const u of extractImageUrls(res.data?.content ?? '').slice(0, 5)) {
        const img = new Image()
        img.src = u
      }
    })
    .catch(() => {
      /* 静默 */
    })
}

/* ---------------- GAP 102：正文图片全屏查看（段落为单张图片时渲染 <img>，点击全屏） ---------------- */

const imgViewerOpen = ref(false)
const imgViewerUrl = ref('')

function openImgViewer(url: string) {
  imgViewerUrl.value = url
  imgViewerOpen.value = true
}

function closeImgViewer() {
  imgViewerOpen.value = false
  imgViewerUrl.value = ''
}

/** 段落是否为单张图片（markdown 语法或裸图片 URL）→ 返回图片地址，否则 null */
function singleImageUrl(para: string): string | null {
  const md = /^!\[[^\]]*]\(\s*([^)\s]+)\s*\)$/.exec(para)
  if (md) return proxyImageUrl(md[1]) ?? md[1]
  if (
    /^https?:\/\/[^\s"'<>，。！？、；：“”‘’（）【】]+\.(?:png|jpe?g|gif|webp|bmp|svg)(?:\?[^\s"'<>]*)?$/i.test(
      para,
    )
  ) {
    return proxyImageUrl(para) ?? para
  }
  return null
}

/** 与 paragraphs 同源切分（原始文本，不经替换/简繁转换），逐段标注图片地址 */
const paraImgs = computed<(string | null)[]>(() =>
  content.value
    .split(/[\r\n]+/)
    .map((s) => s.trim())
    .filter(Boolean)
    .map(singleImageUrl),
)

/* ---------------- 目录/章节 ---------------- */

/** 有效章节（跳过卷标题分隔行） */
const realChapters = computed(() => chapters.value.filter((c) => !c.isVolume))
const currentChapter = computed(() => chapters.value[chapterIndex.value] ?? null)
const flatIndex = computed(() =>
  currentChapter.value
    ? realChapters.value.findIndex((c) => c.url === currentChapter.value?.url)
    : -1,
)
const hasPrev = computed(() => flatIndex.value > 0)
const hasNext = computed(() => flatIndex.value >= 0 && flatIndex.value < realChapters.value.length - 1)

const paragraphs = computed(() =>
  content.value
    .split(/[\r\n]+/)
    .map((s) => s.trim())
    .filter(Boolean)
    .map((p) => applyReplace(hanConvert(p))),
)

/* ---------------- GAP 155：长章分段渲染（>200 段按 200 段/块渐进渲染——滚动到底加载下一块，防超长章 DOM 过大） ---------------- */

const SEGMENT_SIZE = 200
/** 已渲染段数（resetSegments 初始化 / 滚动到底 +200；始终 ≤ paragraphs.length） */
const renderedCount = ref(0)
const visibleParagraphs = computed(() => paragraphs.value.slice(0, renderedCount.value))
/** 与 visibleParagraphs 对齐的图片标注（GAP 102 单图段落） */
const visibleParaImgs = computed(() => paraImgs.value.slice(0, renderedCount.value))
const hasMoreParagraphs = computed(() => renderedCount.value < paragraphs.value.length)

/** 换章/正文变化时重置为首块 */
function resetSegments() {
  renderedCount.value = Math.min(SEGMENT_SIZE, paragraphs.value.length)
}

/** 滚动近底（600px）时加载下一块；返回是否加载了新块 */
function maybeLoadMoreChunk(): boolean {
  if (!hasMoreParagraphs.value) return false
  if (loading.value || loadError.value) return false
  const doc = document.documentElement
  if (doc.scrollHeight - window.innerHeight - window.scrollY > 600) return false
  renderedCount.value = Math.min(renderedCount.value + SEGMENT_SIZE, paragraphs.value.length)
  return true
}

/** 确保第 idx 段已渲染（书签/搜索跳转/滚动恢复前逐块加载直到覆盖目标） */
async function ensureParagraphRendered(idx: number) {
  while (renderedCount.value <= idx && hasMoreParagraphs.value) {
    renderedCount.value = Math.min(renderedCount.value + SEGMENT_SIZE, paragraphs.value.length)
    await nextTick()
    await settleFrames()
  }
}
/* ---------------- 章节字数（目录抽屉：后端 chapterWordCount 优先；书源书从前端已缓存正文估算，未加载章省略） ---------------- */

/** 当前会话已加载正文的字数：chapterUrl → 字符数（与后端 length(content) 口径一致） */
const chapterWordCounts = ref<Record<string, number>>({})
/** 章节字数：后端字段（本地书）→ 本会话缓存正文估算 → 未知返回 null（目录省略显示） */
function chapterWordCountOf(ch: BookChapter): number | null {
  const wc = typeof ch.chapterWordCount === 'number' ? ch.chapterWordCount : chapterWordCounts.value[ch.url]
  return typeof wc === 'number' && wc > 0 ? wc : null
}
/** 格式化：≥1 万显示「x.x万字」，否则「n字」 */
function fmtWordCount(n: number): string {
  return n >= 10000 ? `${(n / 10000).toFixed(1)}万字` : `${n}字`
}
/** 目录项字数文案（null = 不显示） */
function chapterWordCountLabel(ch: BookChapter): string | null {
  const n = chapterWordCountOf(ch)
  return n === null ? null : fmtWordCount(n)
}

const displayBookName = computed(() => hanConvert(bookName.value))
const displayChapterTitle = computed(() => (currentChapter.value ? hanConvert(currentChapter.value.title) : ''))
/** 目录搜索关键词（按展示标题过滤，含卷标题） */
const tocKeyword = ref('')
/** 目录倒序（legacy PopCatalog 顺序/倒序；localStorage reader_toc_reverse） */
const tocReverse = ref(false)
{
  const raw = localStorage.getItem('reader_toc_reverse')
  if (raw === '1') tocReverse.value = true
}
watch(tocReverse, (v) => {
  try {
    localStorage.setItem('reader_toc_reverse', v ? '1' : '0')
  } catch {
    /* ignore */
  }
})

/** 目录项（含卷标题）统一按当前简繁模式转换；wcLabel=字数文案（未加载章为 null）；
 *  支持关键词过滤与顺序/倒序（保留原始 index 供跳转/高亮） */
const drawerChapters = computed(() => {
  let list = chapters.value.map((c) => ({
    ...c,
    title: hanConvert(c.title),
    wcLabel: chapterWordCountLabel(c),
  }))
  const kw = tocKeyword.value.trim().toLowerCase()
  if (kw) list = list.filter((c) => c.title.toLowerCase().includes(kw))
  if (tocReverse.value) list = [...list].reverse()
  return list
})

/** 已缓存章标记（服务器 book_chapters + 本机 IndexedDB 的实章索引；0 基） */
const cachedChapterIndexes = ref<Set<number>>(new Set())
async function loadCacheMarkers() {
  const set = new Set<number>()
  try {
    const res = await getBookCacheChapters(bookUrl.value)
    for (const ch of res.data?.chapters ?? []) {
      if (typeof ch.index === 'number') set.add(ch.index)
    }
  } catch {
    /* 服务器缓存接口未就绪/未入架——忽略，仍显示本机缓存 */
  }
  try {
    const urls = await listLocalChapterUrls(bookUrl.value)
    const byUrl = new Map<string, number>()
    chapters.value.forEach((c, idx) => {
      if (!c.isVolume) byUrl.set(c.url, idx)
    })
    for (const u of urls) {
      const idx = byUrl.get(u)
      if (typeof idx === 'number') set.add(idx)
    }
  } catch {
    /* IndexedDB 不可用——忽略 */
  }
  cachedChapterIndexes.value = set
}

/* ---------------- 目录卷折叠（点击卷标题折叠/展开；localStorage reader_toc_collapsed {volTitle: bool}） ---------------- */

const TOC_COLLAPSED_KEY = 'reader_toc_collapsed'
/** 折叠状态：卷标题（简繁转换后展示名）→ 是否折叠 */
const tocCollapsed = ref<Record<string, boolean>>({})
{
  try {
    const raw = JSON.parse(localStorage.getItem(TOC_COLLAPSED_KEY) ?? '{}')
    if (raw && typeof raw === 'object' && !Array.isArray(raw)) tocCollapsed.value = raw
  } catch {
    /* ignore */
  }
}

/** 点击卷标题行：折叠/展开 + 持久化 */
function toggleVolume(title: string) {
  tocCollapsed.value = { ...tocCollapsed.value, [title]: !tocCollapsed.value[title] }
  try {
    localStorage.setItem(TOC_COLLAPSED_KEY, JSON.stringify(tocCollapsed.value))
  } catch {
    /* ignore */
  }
}

/** 与 drawerChapters 对齐：非卷条目是否被所在卷折叠隐藏（卷标题行本身始终显示） */
const chapterHidden = computed(() => {
  const out: boolean[] = []
  let currentCollapsed = false
  for (const c of drawerChapters.value) {
    if (c.isVolume) currentCollapsed = !!tocCollapsed.value[c.title]
    out.push(currentCollapsed)
  }
  return out
})

/** 首次进入需要恢复的滚动位置（正文渲染完成后应用一次） */
let restoreScrollY: number | null = null
let saveTimer: number | undefined

/* ---------------- 进度存取 ---------------- */

function progressKey(): string {
  return `reader-progress-${bookUrl.value}`
}

/** 当前阅读位置（纵向滚动 px；仿真翻页为列轴 px；非文本书为媒体秒数/漫画页索引） */
function currentPos(): number {
  if (isNonTextBook.value) return mediaPosition()
  return isFlipMode() ? flipScrollLeft() : window.scrollY
}

function saveProgress() {
  if (!currentChapter.value) return
  try {
    localStorage.setItem(
      progressKey(),
      JSON.stringify({
        chapterIndex: chapterIndex.value,
        // 非文本书：scrollY 槽位存音频/视频秒数或漫画页索引
        scrollY: currentPos(),
        updatedAt: Date.now(),
      } satisfies ReaderProgress),
    )
  } catch {
    /* ignore */
  }
  syncServerProgress()
}

/* ---------------- GAP 110：每日阅读时长累计（本机——统计弹窗近 7 天柱状图数据源） ---------------- */

/** 累计周期（秒）：每 30s 向当日桶 +30s；仅页面可见且正文就绪时累计 */
const DAILY_TICK_SECONDS = 30
let dailyTimer: number | undefined

function startDailyTracker() {
  stopDailyTracker()
  dailyTimer = window.setInterval(() => {
    if (document.visibilityState !== 'visible') return
    if (loading.value || loadError.value || !currentChapter.value) return
    try {
      const map = parseDailyStats(localStorage.getItem(DAILY_STATS_KEY))
      localStorage.setItem(DAILY_STATS_KEY, JSON.stringify(accumulateDaily(map, DAILY_TICK_SECONDS)))
    } catch {
      /* ignore */
    }
  }, DAILY_TICK_SECONDS * 1000)
}

function stopDailyTracker() {
  if (dailyTimer !== undefined) {
    window.clearInterval(dailyTimer)
    dailyTimer = undefined
  }
}

/** 进度服务端同步（POST /reader3/saveBookProgress；失败静默，不影响本地阅读） */
function syncServerProgress() {
  if (!shelfBook.value || !currentChapter.value) return
  void post('/saveBookProgress', {
    bookUrl: bookUrl.value,
    durChapterIndex: chapterIndex.value,
    durChapterPos: Math.round(currentPos()),
    durChapterTime: Date.now(),
    durChapterTitle: currentChapter.value.title,
  }).catch(() => {
    /* 静默失败 */
  })
}

function restoreProgress(): ReaderProgress | null {
  try {
    const raw = localStorage.getItem(progressKey())
    if (!raw) return null
    const p = JSON.parse(raw) as ReaderProgress
    if (typeof p.chapterIndex === 'number' && typeof p.scrollY === 'number') return p
  } catch {
    /* ignore */
  }
  return null
}

/* ---------------- B1 修复：进度恢复双保险（渲染 + 图片 load 后各校正一次） ---------------- */

function settleFrames(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })
}

/** 等待正文内图片加载完成（带超时兜底），避免图片撑高后落点偏移 */
function imagesReady(): Promise<void> {
  const imgs = Array.from(document.querySelectorAll<HTMLImageElement>('.reader-content img'))
  if (imgs.length === 0) return Promise.resolve()
  const wait = (img: HTMLImageElement) =>
    img.complete && img.naturalWidth > 0
      ? Promise.resolve()
      : new Promise<void>((resolve) => {
          const done = () => resolve()
          img.addEventListener('load', done, { once: true })
          img.addEventListener('error', done, { once: true })
          window.setTimeout(done, 3000)
        })
  return Promise.all(imgs.map(wait)).then(() => undefined)
}

async function applyRestoreScroll() {
  if (restoreScrollY == null) return
  const v = flipViewRef.value
  if (isFlipMode() && v) {
    // 仿真翻页：目标为列轴 px；不足时逐块加载直到可分页到目标
    const target = restoreScrollY
    while (hasMoreParagraphs.value) {
      if (v.scrollWidth > target) break
      renderedCount.value = Math.min(renderedCount.value + SEGMENT_SIZE, paragraphs.value.length)
      await nextTick()
      await settleFrames()
    }
    await nextTick()
    await settleFrames()
    const clamp = () => Math.min(target, Math.max(0, v.scrollWidth - v.clientWidth))
    v.scrollTo(0, clamp())
    // 双保险：图片加载（或 3s 超时）后再校正一次
    await imagesReady()
    await settleFrames()
    v.scrollTo(0, clamp())
    restoreScrollY = null
    updateScrollFrac()
    return
  }
  // GAP 155：恢复目标超出当前已渲染高度时，先逐块加载直到可滚动到目标位置
  while (hasMoreParagraphs.value) {
    const doc = document.documentElement
    if (doc.scrollHeight - window.innerHeight >= restoreScrollY) break
    renderedCount.value = Math.min(renderedCount.value + SEGMENT_SIZE, paragraphs.value.length)
    await nextTick()
    await settleFrames()
  }
  await nextTick()
  await settleFrames()
  window.scrollTo(0, restoreScrollY)
  // 双保险：图片加载（或 3s 超时）后再校正一次
  await imagesReady()
  await settleFrames()
  window.scrollTo(0, restoreScrollY)
  restoreScrollY = null
  updateScrollFrac()
}

/* ---------------- 正文加载 ---------------- */

async function loadContent(chapterUrl: string) {
  if (!shelfBook.value?.origin) return
  loading.value = true
  loadError.value = false
  content.value = ''
  // 本机缓存优先；未命中再走服务器缓存/书源（getBookContent 命中服务器缓存，未命中自动抓取并写回）
  const local = await getLocalChapter(bookUrl.value, chapterUrl)
  let text = local?.content ?? ''
  let fetchedWordCount: number | null = null
  try {
    if (!text) {
      const res = await getBookContent(chapterUrl, shelfBook.value.origin, {
        timeout: chapterTimeout.value * 1000,
      })
      text = res.data?.content ?? ''
      if (typeof res.data?.chapterWordCount === 'number') {
        fetchedWordCount = res.data.chapterWordCount
      }
      const fi = flatIndex.value
      const ch = currentChapter.value
      if (ch && text) {
        void saveLocalChapter({
          bookUrl: bookUrl.value,
          chapterUrl,
          title: ch.title,
          index: fi >= 0 ? fi : 0,
          content: text,
        })
      }
    }
    content.value = text
    // 章节字数：后端 chapterWordCount（本地书正文接口附带）优先；缺失用已缓存正文估算
    chapterWordCounts.value = {
      ...chapterWordCounts.value,
      [chapterUrl]: fetchedWordCount ?? text.length,
    }
    // 听书：播放中切章 / 本章播完自动连播 → 新章正文就绪后自动续播
    if (ttsState.value !== 'idle' || ttsAutoNext) {
      ttsAutoNext = false
      void startTts()
    }
  } catch {
    ttsAutoNext = false
    loadError.value = true
    return
  } finally {
    loading.value = false
  }
  // 等正文真正渲染（loading 置 false 后）再滚动，避免被加载态高度钳制
  await nextTick()
  if (isFlipMode()) measureFlipColumns()
  if (restoreParagraphIdx != null) {
    await applyRestoreParagraph()
  } else if (isFlipMode()) {
    const v = flipViewRef.value
    if (v) v.scrollTo(0, restoreScrollY ?? 0)
    if (restoreScrollY != null) {
      await applyRestoreScroll()
    } else {
      restoreScrollY = null
      updateScrollFrac()
    }
  } else {
    window.scrollTo(0, restoreScrollY ?? 0)
    if (restoreScrollY != null) {
      await applyRestoreScroll()
    } else {
      restoreScrollY = null
      updateScrollFrac()
    }
  }
  // 当前章渲染后：预加载下一章前 5 张图片（仅当下一章含图片 URL）
  preloadNextChapterImages()
  // 切章后清除朗读高亮（连播/手动切章后由 startTts 重新跟踪）
  ttsReadingPara.value = -1
}

function cancelRetention() {
  if (retentionBusy.value) return
  retentionOpen.value = false
  // 暂不加入：放行路由离开（resolve false 会取消导航，导致用户无法退出）
  retentionResolve?.(true)
  retentionResolve = null
}

/** 挽留确认：补全详情字段入架（临时书只有 query 传入的少量字段——否则书架会是首字封面 + 佚名） */
async function confirmRetention() {
  const b = shelfBook.value
  if (!b || retentionBusy.value) return
  retentionBusy.value = true
  try {
    // 详情并行请求失败时重试一次；仍失败则按已有字段入架，避免写空书名/作者/封面
    let info = tempInfo.value
    if (!info) {
      try {
        const res = await getBookInfo(b.bookUrl, b.origin, { silent: true })
        if (res.isSuccess && res.data) {
          info = res.data
          tempInfo.value = info
        }
      } catch {
        /* 详情失败不阻断入架 */
      }
    }
    await saveBook({
      bookUrl: b.bookUrl,
      name: info?.name || b.name,
      author: info?.author || b.author,
      origin: info?.origin || b.origin || '',
      originName: info?.originName || b.originName || '',
      tocUrl: info?.tocUrl || b.tocUrl || b.bookUrl,
      intro: info?.intro ?? b.intro ?? '',
      coverUrl: info?.coverUrl ?? b.coverUrl ?? '',
      kind: info?.kind ?? b.kind ?? null,
      latestChapterTitle: info?.latestChapterTitle ?? null,
      type: bookType.value,
      group: 0,
    } as Book)
    ElMessage.success('已加入书架')
  } catch {
    /* 入架失败不阻断离开；错误提示由请求层处理 */
  } finally {
    retentionBusy.value = false
    retentionOpen.value = false
    retentionResolve?.(true)
    retentionResolve = null
  }
}

// 临时书（先读后入架）：离开时提醒加入书架
onBeforeRouteLeave(() => {
  const b = shelfBook.value
  if (!b || !(b as unknown as { isTemp?: boolean }).isTemp) return true
  retentionOpen.value = true
  return new Promise<boolean>((resolve) => {
    retentionResolve = (ok) => {
      retentionOpen.value = false
      resolve(ok)
    }
  })
})

function goToChapter(idx: number) {
  const ch = chapters.value[idx]
  if (!ch || ch.isVolume) return
  drawerOpen.value = false
  if (idx === chapterIndex.value) return
  // 切章方向（hslide 模式正文滑入过渡动画用）
  chapterDir.value = idx > chapterIndex.value ? 1 : -1
  saveProgress()
  chapterIndex.value = idx
  if (isNonTextBook.value) void loadNonTextChapter(ch.url)
  else void loadContent(ch.url)
}

/** 缓存完成：组件已展示结果；阅读页无需额外刷新（详情页才刷新单书缓存状态） */
function onCacheDone() {
  void loadCacheMarkers()
}

/* ================= 阅读内换源（GAP：ReaderView 直接换源——作者/最新章节/当前章末尾预览） ================= */

/** 单个书源的当前章预览（懒加载：换源列表渲染后按源拉目录 + 当前章正文末段） */
interface SourcePreview {
  author: string
  latestChapter: string
  currentLast: string
  status: 'loading' | 'done' | 'error'
}

const sourceOpen = ref(false)
const sourceBusy = ref(false)
const sourceSwitching = ref(false)
const sourceResults = ref<SearchBook[]>([])
const sourceKeyword = ref('')
const sourceMsg = ref('')
const sourceMsgError = ref(false)
const sourceDoneCount = ref(0)
const currentOrigin = ref('')
const invalidSourceUrls = ref<Set<string>>(new Set())
const sourcePreviews = ref<Record<string, SourcePreview>>({})
let sourceSSEHandle: { abort: () => void } | null = null

const sourceFiltered = computed(() => {
  const kw = sourceKeyword.value.trim().toLowerCase()
  if (!kw) return sourceResults.value
  return sourceResults.value.filter(
    (r) =>
      (r.originName || '').toLowerCase().includes(kw) ||
      (r.origin || '').toLowerCase().includes(kw),
  )
})

function canSwitchSource(): boolean {
  const b = shelfBook.value
  return !!b && !!b.origin && !loading.value && !loadError.value && chapters.value.length > 0
}

function openSource() {
  sourceOpen.value = true
  document.body.style.overflow = 'hidden'
  void runSourceSearch()
}

function closeSource() {
  if (sourceBusy.value || sourceSwitching.value) return
  forceCloseSource()
}

function forceCloseSource() {
  sourceSSEHandle?.abort()
  sourceSSEHandle = null
  sourceOpen.value = false
  document.body.style.overflow = ''
}

function refreshSource() {
  if (sourceBusy.value) return
  sourceSSEHandle?.abort()
  sourceResults.value = []
  sourcePreviews.value = {}
  void runSourceSearch()
}

function previewOf(r: SearchBook): SourcePreview | undefined {
  return sourcePreviews.value[r.origin || r.originName || '']
}

function sortSourceResults(list: SearchBook[]): SearchBook[] {
  const cur = list.filter((r) => r.origin === currentOrigin.value)
  const rest = list.filter((r) => r.origin !== currentOrigin.value)
  const invalid = rest.filter((r) => invalidSourceUrls.value.has(r.origin))
  const valid = rest.filter((r) => !invalidSourceUrls.value.has(r.origin))
  const byName = (a: SearchBook, c: SearchBook) =>
    (a.originName || a.origin || '').localeCompare(c.originName || c.origin || '')
  return [...cur, ...valid.sort(byName), ...invalid.sort(byName)]
}

function appendSourceResults(books: SearchBook[]) {
  const seen = new Set(sourceResults.value.map((r) => r.origin || r.originName))
  for (const r of books) {
    const k = r.origin || r.originName
    if (!k || seen.has(k)) continue
    seen.add(k)
    sourceResults.value = [...sourceResults.value, r]
    void ensurePreview(r)
  }
}

function finalizeSourceResults() {
  sourceResults.value = sortSourceResults(sourceResults.value)
  if (sourceResults.value.length === 0) {
    sourceMsg.value = '未找到其他书源'
    sourceMsgError.value = false
  }
}

async function runSourceSearch() {
  const b = shelfBook.value
  if (!b || !b.origin) return
  sourceBusy.value = true
  sourceResults.value = []
  sourcePreviews.value = {}
  sourceMsg.value = ''
  sourceMsgError.value = false
  sourceDoneCount.value = 0
  currentOrigin.value = b.origin
  invalidSourceUrls.value = new Set()
  try {
    try {
      const inv = await getInvalidBookSources()
      invalidSourceUrls.value = new Set(
        (inv.data ?? []).map((x) => (typeof x === 'string' ? x : x.bookSourceUrl)),
      )
    } catch {
      invalidSourceUrls.value = new Set()
    }
    let sseFailed = false
    try {
      const handle = await searchBookSourceSSE(b.bookUrl, b.origin, {
        onBooks: (_lastIndex, books) => {
          appendSourceResults(books)
          sourceDoneCount.value += 1
        },
        onEnd: () => {
          sourceBusy.value = false
          finalizeSourceResults()
        },
        onErrorEvent: (ret) => {
          sourceMsg.value = ret.errorMsg || '换源搜索失败'
          sourceMsgError.value = true
          sourceBusy.value = false
        },
        onStreamError: () => {
          sourceBusy.value = false
          if (sourceResults.value.length === 0) {
            sourceMsg.value = '流式换源中断，请重试'
            sourceMsgError.value = true
          } else {
            finalizeSourceResults()
          }
        },
      })
      sourceSSEHandle = handle
    } catch {
      sseFailed = true
    }
    if (sseFailed) {
      const res = await searchBookSource(b.bookUrl, b.origin, { silent: true })
      appendSourceResults(res.data ?? [])
      finalizeSourceResults()
      sourceBusy.value = false
    }
  } catch (err) {
    sourceMsg.value = `换源搜索失败：${err instanceof Error ? err.message : '请稍后重试'}`
    sourceMsgError.value = true
    sourceBusy.value = false
  } finally {
    sourceBusy.value = false
  }
}

/* ---- 当前章末尾预览：懒加载（≤4 并发），按源拉目录定位当前章 → 正文最后一段 ---- */

const PREVIEW_CONCURRENCY = 4
let previewQueue: Array<() => Promise<void>> = []
let previewRunning = 0

function enqueuePreview(task: () => Promise<void>) {
  previewQueue.push(task)
  drainPreviewQueue()
}

function drainPreviewQueue() {
  while (previewRunning < PREVIEW_CONCURRENCY) {
    const task = previewQueue.shift()
    if (!task) return
    previewRunning += 1
    void task().finally(() => {
      previewRunning -= 1
      drainPreviewQueue()
    })
  }
}

function setPreview(key: string, p: SourcePreview) {
  sourcePreviews.value = { ...sourcePreviews.value, [key]: p }
}

function ensurePreview(r: SearchBook) {
  const key = r.origin || r.originName
  if (!key) return
  const cur = sourcePreviews.value[key]
  if (cur && cur.status !== 'error') return
  setPreview(key, { author: '', latestChapter: '', currentLast: '', status: 'loading' })
  enqueuePreview(() => loadSourcePreview(r, key))
}

async function loadSourcePreview(r: SearchBook, key: string) {
  const b = shelfBook.value
  if (!b) return
  try {
    const tocRes = await getBookToc(r.tocUrl || b.tocUrl, r.origin, {
      timeout: chapterTimeout.value * 1000,
    })
    const toc = tocRes.isSuccess ? (tocRes.data ?? []) : []
    const oldIdx = currentChapter.value ? chapterIndex.value : -1
    const oldTitle = currentChapter.value?.title || b.durChapterTitle || ''
    let idx = relocateChapterIndex(oldIdx, oldTitle, toc)
    if (idx < 0 && toc.length) idx = toc.findIndex((c) => !c.isVolume)
    let currentLast = ''
    if (idx >= 0 && toc[idx] && !toc[idx].isVolume && isTextBook.value) {
      const ch = toc[idx]
      const contentRes = await getBookContent(ch.url, r.origin, {
        timeout: chapterTimeout.value * 1000,
      })
      const text = contentRes.data?.content ?? ''
      const paras = text
        .split(/\n+/)
        .map((s) => s.trim())
        .filter(Boolean)
      currentLast = paras.length ? paras[paras.length - 1] : ''
    }
    setPreview(key, {
      author: r.author || '',
      latestChapter: r.latestChapterTitle || '',
      currentLast,
      status: 'done',
    })
  } catch {
    setPreview(key, { author: r.author || '', latestChapter: r.latestChapterTitle || '', currentLast: '', status: 'error' })
  }
}

/** 点击结果 → 阅读内切换书源：保留当前章进度（标题匹配/就近钳制），刷新目录并续读 */
async function switchSource(r: SearchBook) {
  const b = shelfBook.value
  if (!b || sourceSwitching.value) return
  if (!r.origin || r.origin === currentOrigin.value) return
  sourceSwitching.value = true
  try {
    const oldIdx = chapterIndex.value
    const oldTitle = currentChapter.value?.title ?? ''
    const oldPos = currentPos()
    const isTemp = !!(b as unknown as { isTemp?: boolean }).isTemp
    if (!isTemp) {
      await saveBook({
        bookUrl: b.bookUrl,
        origin: r.origin,
        originName: r.originName,
        tocUrl: r.tocUrl,
      } as Book)
    }
    b.origin = r.origin
    b.originName = r.originName
    b.tocUrl = r.tocUrl
    currentOrigin.value = r.origin
    const tocRes = await getBookToc(r.tocUrl || b.tocUrl, r.origin, {
      timeout: chapterTimeout.value * 1000,
    })
    if (!tocRes.isSuccess || !tocRes.data?.length) {
      ElMessage.error('新书源目录获取失败，请重试')
      return
    }
    const toc = tocRes.data
    let startIdx = relocateChapterIndex(oldIdx, oldTitle, toc)
    if (startIdx < 0) startIdx = toc.findIndex((c) => !c.isVolume)
    if (startIdx < 0) startIdx = 0
    const ch = toc[startIdx]
    chapters.value = toc
    void loadCacheMarkers()
    chapterIndex.value = startIdx
    content.value = ''
    loadError.value = false
    loading.value = true
    chapterWordCounts.value = {}
    tempInfo.value = null
    bookName.value = r.name || b.name || bookName.value
    // 详情并行刷新（换源后挽留入架/书名展示用新源数据；失败不阻断）
    try {
      const infoRes = await getBookInfo(b.bookUrl, r.origin, { silent: true })
      if (infoRes.isSuccess && infoRes.data) {
        tempInfo.value = infoRes.data
        bookName.value = infoRes.data.name || bookName.value
      }
    } catch {
      /* 详情失败不阻断换源 */
    }
    if (!isTemp) {
      b.durChapterIndex = startIdx
      b.durChapterTitle = ch.title
      b.durChapterPos = oldPos
      b.durChapterTime = Date.now()
      void post('/saveBookProgress', {
        bookUrl: b.bookUrl,
        durChapterIndex: startIdx,
        durChapterPos: oldPos,
        durChapterTime: Date.now(),
        durChapterTitle: ch.title,
      }).catch(() => {
        /* 静默失败 */
      })
    }
    if (isNonTextBook.value) await loadNonTextChapter(ch.url)
    else await loadContent(ch.url)
    sourcePreviews.value = {}
    ElMessage.success(`已切换到「${r.originName || r.origin}」`)
    forceCloseSource()
  } catch (err) {
    ElMessage.error(`换源失败：${err instanceof Error ? err.message : '请稍后重试'}`)
  } finally {
    sourceSwitching.value = false
  }
}

function prevChapter() {
  const fi = flatIndex.value
  if (fi <= 0) return
  goToChapter(chapters.value.indexOf(realChapters.value[fi - 1]))
}

function nextChapter() {
  const fi = flatIndex.value
  if (fi < 0 || fi >= realChapters.value.length - 1) return
  goToChapter(chapters.value.indexOf(realChapters.value[fi + 1]))
}

/* ---------------- GAP 4：长按快进/快退（按住 250ms 后每 300ms 连续翻章；pointer 事件覆盖鼠标/触屏） ---------------- */

interface HoldRepeat {
  start: (e: PointerEvent) => void
  stop: () => void
  /** 长按已触发连翻时吞掉释放后的合成 click（避免多翻一章） */
  swallowClick: () => boolean
}

function createHoldRepeat(fn: () => void): HoldRepeat {
  let timer: number | undefined
  let interval: number | undefined
  let fired = false
  function start(e: PointerEvent) {
    // 仅响应主键（鼠标左键 / 触屏笔尖）
    if (e.pointerType === 'mouse' && e.button !== 0) return
    stop()
    fired = false
    timer = window.setTimeout(() => {
      fired = true
      fn()
      interval = window.setInterval(fn, 300)
    }, 250)
  }
  function stop() {
    if (timer !== undefined) {
      window.clearTimeout(timer)
      timer = undefined
    }
    if (interval !== undefined) {
      window.clearInterval(interval)
      interval = undefined
    }
  }
  function swallowClick(): boolean {
    if (fired) {
      fired = false
      return true
    }
    return false
  }
  return { start, stop, swallowClick }
}

const prevHold = createHoldRepeat(prevChapter)
const nextHold = createHoldRepeat(nextChapter)

function onPrevClick() {
  if (prevHold.swallowClick()) return
  prevChapter()
}

function onNextClick() {
  if (nextHold.swallowClick()) return
  nextChapter()
}

function retry() {
  if (chapters.value.length === 0) void init()
  else if (currentChapter.value) {
    if (isNonTextBook.value) void loadNonTextChapter(currentChapter.value.url)
    else void loadContent(currentChapter.value.url)
  }
}

/* ---------------- B4 修复：阅读页移出书架入口 ---------------- */

const removeOpen = ref(false)
const removeBusy = ref(false)

function openRemoveConfirm() {
  removeOpen.value = true
}

function closeRemoveConfirm() {
  if (removeBusy.value) return
  removeOpen.value = false
}

async function confirmRemoveFromShelf() {
  if (removeBusy.value) return
  removeBusy.value = true
  try {
    await deleteBook(bookUrl.value)
    // request.ts 拦截器已处理失败提示；走到这里即成功
    try {
      localStorage.removeItem(progressKey())
    } catch {
      /* ignore */
    }
    ElMessage.success('已移出书架')
    void router.replace('/')
  } catch {
    /* 已提示 */
  } finally {
    removeBusy.value = false
    removeOpen.value = false
  }
}

/* ---------------- 非文本书渲染：音频/视频播放器 + 漫画逐页 + 文件下载 ---------------- */

/* ---- 音频书 ---- */
const mediaAudioRef = ref<HTMLAudioElement | null>(null)
const audioUrl = ref('')
const audioContentType = ref('')
const audioPlaying = ref(false)
const audioCurrent = ref(0)
const audioDuration = ref(0)
const audioBuffering = ref(false)
/** m3u8 → hls.js 实例（CDN 动态加载；Safari 原生支持时无需） */
let hlsInstance: { destroy: () => void } | null = null
const hlsFailed = ref(false)

/** 动态加载 hls.js（仅 m3u8 且浏览器无原生 HLS 支持时）
 *  P3-A：固定版本 + SRI integrity（防 CDN 投毒；版本浮动 @1 会让 SRI 失效，故钉死 1.5.20） */
function loadHlsJs(): Promise<{ default?: unknown } | null> {
  return new Promise((resolve) => {
    const win = window as unknown as { Hls?: { isSupported: () => boolean; new (): unknown } }
    if (win.Hls) return resolve({ default: win.Hls })
    const script = document.createElement('script')
    script.src = 'https://cdn.jsdelivr.net/npm/hls.js@1.5.20/dist/hls.min.js'
    // sha384 与 jsdelivr 发布的 hls.js@1.5.20 文件一致（本地计算 + registry sha256 双重核验）
    script.integrity =
      'sha384-V5ruNBgmYcC3SJRUQeNykAAAgde5gOFq/Hu0CZj7bygDP0yRIhkvX8+w0u/7mRvr'
    script.crossOrigin = 'anonymous'
    script.onload = () => resolve(win.Hls ? { default: win.Hls } : null)
    script.onerror = () => resolve(null)
    document.head.appendChild(script)
  })
}

async function setupHls(el: HTMLAudioElement, url: string) {
  hlsInstance?.destroy()
  hlsInstance = null
  hlsFailed.value = false
  const win = window as unknown as { Hls?: { isSupported: () => boolean; new (config?: object): { loadSource: (u: string) => void; attachMedia: (m: HTMLMediaElement) => void; destroy: () => void } } }
  // 原生支持（Safari）→ 直接 src
  if (el.canPlayType('application/vnd.apple.mpegurl')) return
  const mod = await loadHlsJs()
  if (!mod || !win.Hls?.isSupported()) {
    hlsFailed.value = true
    return
  }
  const hls = new win.Hls()
  hls.loadSource(url)
  hls.attachMedia(el)
  hlsInstance = hls
}

function fmtTime(s: number): string {
  if (!Number.isFinite(s) || s < 0) s = 0
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  const h = Math.floor(m / 60)
  if (h > 0) return `${h}:${String(m % 60).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
  return `${m}:${String(sec).padStart(2, '0')}`
}

function onAudioLoadedMeta() {
  const el = mediaAudioRef.value
  if (el && Number.isFinite(el.duration)) audioDuration.value = el.duration
}
function onAudioTimeUpdate() {
  const el = mediaAudioRef.value
  if (!el) return
  audioCurrent.value = el.currentTime
  if (Number.isFinite(el.duration)) audioDuration.value = el.duration
}
function onAudioPlay() {
  audioPlaying.value = true
}
function onAudioPause() {
  audioPlaying.value = false
}
function onAudioWaiting() {
  audioBuffering.value = true
}
function onAudioPlaying() {
  audioBuffering.value = false
}
/** 本章播完 → 自动下一章（与文本书 TTS 连播语义一致） */
function onAudioEnded() {
  audioPlaying.value = false
  if (hasNext.value) nextChapter()
}
function toggleAudioPlay() {
  const el = mediaAudioRef.value
  if (!el || !audioUrl.value) return
  if (el.paused) void el.play().catch(() => { /* 自动播放被拦截——用户已点击，一般可播 */ })
  else el.pause()
}
function seekAudio() {
  const el = mediaAudioRef.value
  if (!el) return
  el.currentTime = audioCurrent.value
}

/* ---- 视频书 ---- */
const videoElRef = ref<HTMLVideoElement | null>(null)
const videoUrl = ref('')
const videoCurrent = ref(0)
const videoDuration = ref(0)
function onVideoTimeUpdate() {
  const el = videoElRef.value
  if (!el) return
  videoCurrent.value = el.currentTime
  if (Number.isFinite(el.duration)) videoDuration.value = el.duration
}

/* ---- 漫画书：横向滑动 + 点击左右边缘翻页 + 懒加载占位 ---- */
const comicScrollRef = ref<HTMLElement | null>(null)
const comicImages = ref<string[]>([])
const comicPage = ref(0)

function onComicScroll() {
  const el = comicScrollRef.value
  if (!el) return
  const page = Math.round(el.scrollLeft / Math.max(1, el.clientWidth))
  comicPage.value = Math.min(comicImages.value.length - 1, Math.max(0, page))
}
/** 点击图片左/右 1/3 区域 → 上一页/下一页（点击中间不翻页，便于看图） */
function comicClickPage(e: MouseEvent, idx: number) {
  const el = comicScrollRef.value
  if (!el) return
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const x = e.clientX - rect.left
  const third = rect.width / 3
  if (x < third) {
    if (idx > 0) el.scrollTo({ left: (idx - 1) * el.clientWidth, behavior: 'smooth' })
  } else if (x > third * 2) {
    if (idx < comicImages.value.length - 1)
      el.scrollTo({ left: (idx + 1) * el.clientWidth, behavior: 'smooth' })
  }
}

/* ---- 文件书 ---- */
const fileUrl = ref('')

/** 非文本书章节加载（按 bookType 分派：不走正文分段解析/替换/简繁转换） */
async function loadNonTextChapter(chapterUrl: string) {
  if (!shelfBook.value?.origin) return
  loading.value = true
  loadError.value = false
  // 清空上一章媒体态
  content.value = ''
  audioUrl.value = ''
  videoUrl.value = ''
  fileUrl.value = ''
  comicImages.value = []
  comicPage.value = 0
  hlsFailed.value = false
  try {
    const res = await getBookContent(chapterUrl, shelfBook.value.origin, {
      timeout: chapterTimeout.value * 1000,
    })
    const data = res.data ?? {}
    if (isAudioBook.value) {
      const url = data.audioUrl
      if (!url) {
        loadError.value = true
        return
      }
      audioUrl.value = url
      audioContentType.value = data.contentType ?? ''
      await nextTick()
      const el = mediaAudioRef.value
      if (el) {
        el.pause()
        audioPlaying.value = false
        audioCurrent.value = 0
        audioDuration.value = 0
        el.src = url
        el.load()
        const isHls =
          url.toLowerCase().includes('.m3u8') ||
          (audioContentType.value ?? '').toLowerCase().includes('mpegurl')
        if (isHls) await setupHls(el, url)
        // 恢复该章播放位置（saveProgress 存秒数）
        const savedPos = restoreScrollY ?? 0
        restoreScrollY = null
        if (savedPos > 0) {
          try {
            el.currentTime = savedPos
          } catch {
            /* ignore */
          }
        }
        void el.play().catch(() => {
          audioPlaying.value = false
        })
      }
    } else if (isComicBook.value) {
      const images = data.images
      if (!Array.isArray(images) || images.length === 0) {
        loadError.value = true
        return
      }
      comicImages.value = images.map((u) => proxyImageUrl(u) ?? u)
      await nextTick()
      // 恢复该章页位置（saveProgress 存页索引）
      const savedPage = restoreScrollY ?? 0
      restoreScrollY = null
      const el = comicScrollRef.value
      if (el && savedPage > 0 && savedPage < images.length) {
        el.scrollLeft = savedPage * el.clientWidth
        comicPage.value = savedPage
      }
    } else if (isVideoBook.value) {
      videoUrl.value = data.videoUrl ?? ''
      if (!videoUrl.value) {
        loadError.value = true
        return
      }
      await nextTick()
      const el = videoElRef.value
      const savedPos = restoreScrollY ?? 0
      restoreScrollY = null
      if (el && savedPos > 0) {
        el.currentTime = savedPos
      }
    } else if (isFileBook.value) {
      fileUrl.value = data.downloadUrl ?? ''
      if (!fileUrl.value) {
        loadError.value = true
        return
      }
    }
  } catch {
    loadError.value = true
  } finally {
    loading.value = false
  }
}

/** 非文本书播放/暂停（空格键） */
function toggleMediaPlay() {
  if (isAudioBook.value) toggleAudioPlay()
  else if (isVideoBook.value) {
    const el = videoElRef.value
    if (!el) return
    if (el.paused) void el.play().catch(() => {})
    else el.pause()
  }
}

/** 非文本书当前进度位置（音频/视频=秒；漫画=页索引；文件=0） */
function mediaPosition(): number {
  if (isAudioBook.value) return mediaAudioRef.value?.currentTime ?? audioCurrent.value
  if (isVideoBook.value) return videoElRef.value?.currentTime ?? videoCurrent.value
  if (isComicBook.value) return comicPage.value
  return 0
}

/* ---------------- 初始化 ---------------- */

async function init() {
  loading.value = true
  loadError.value = false
  notFound.value = false
  try {
    // 正文/目录接口需要 bookSource=book.origin，先从书架定位本书
    const shelfRes = await getBookshelf()
    const found = (shelfRes.data ?? []).find((b) => b.bookUrl === bookUrl.value)
    if (!found?.origin) {
      // 非书架书（详情页「直接阅读」跳入）：用 query 构建临时书——先读后提示入架
      const q = route.query as Record<string, string>
      if (q.source) {
        const qType = Number(q.type)
        shelfBook.value = {
          bookUrl: bookUrl.value,
          origin: q.source,
          originName: q.sourceName || '',
          tocUrl: q.toc || bookUrl.value,
          name: q.name || '未命名',
          author: q.author || '',
          coverUrl: q.cover || '',
          // 非文本书临时直读：详情页透传 type（0 文本/1 音频/2 漫画/3 文件/4 视频）
          type: Number.isInteger(qType) && qType >= 0 && qType <= 4 ? qType : 0,
          group: 0,
        } as Book
        shelfBook.value.isTemp = true as never
        bookName.value = shelfBook.value.name
      } else {
        notFound.value = true
        return
      }
    }
    // 本地书 tocUrl 可能为空——用 bookUrl 兜底
    if (found && !found.tocUrl) found.tocUrl = found.bookUrl
    if (found) {
      shelfBook.value = found
      bookName.value = found.name
    }
    // 替换规则：进入阅读页时读取一次（localStorage 占位；后端就绪后走 GET /reader3/getReplaceRules）
    if (replaceEnabled.value) refreshReplaceRules()

    // 目录 + 详情并行拉取
    const [tocRes, infoRes] = await Promise.allSettled([
      getBookToc(shelfBook.value!.tocUrl, shelfBook.value!.origin, {
        timeout: chapterTimeout.value * 1000,
      }),
      getBookInfo(shelfBook.value!.bookUrl, shelfBook.value!.origin, { silent: true }),
    ])
    if (tocRes.status === 'fulfilled' && tocRes.value.isSuccess) {
      chapters.value = tocRes.value.data ?? []
      void loadCacheMarkers()
    } else {
      loadError.value = true
      return
    }
    if (infoRes.status === 'fulfilled' && infoRes.value.isSuccess && infoRes.value.data) {
      tempInfo.value = infoRes.value.data
      bookName.value = tempInfo.value.name || bookName.value
    }

    // 起始章节：① 全书搜索跳转 ?chapter=index 显式指定（优先级最高）；
    // ② 服务端进度（getBookshelf 已带 durChapter*；durChapterTime>0 才算存过）；
    // ③ 无服务端进度则回退 localStorage
    let startIndex = realChapters.value.length ? chapters.value.indexOf(realChapters.value[0]) : 0
    const qc = Number(route.query.chapter)
    const queryChapter =
      Number.isInteger(qc) && qc >= 0 && qc < chapters.value.length && !chapters.value[qc].isVolume
        ? qc
        : -1
    const srvIdx = shelfBook.value?.durChapterIndex ?? -1
    const serverSaved =
      typeof srvIdx === 'number' &&
      srvIdx >= 0 &&
      srvIdx < chapters.value.length &&
      !chapters.value[srvIdx].isVolume &&
      (shelfBook.value?.durChapterTime ?? 0 ?? 0) > 0
    if (queryChapter >= 0) {
      startIndex = queryChapter
    } else if (serverSaved) {
      startIndex = srvIdx
      const srvPos = shelfBook.value?.durChapterPos ?? 0 ?? 0
      restoreScrollY = srvPos > 0 ? srvPos : 0
    } else {
      const saved = restoreProgress()
      if (
        saved &&
        saved.chapterIndex >= 0 &&
        saved.chapterIndex < chapters.value.length &&
        !chapters.value[saved.chapterIndex].isVolume
      ) {
        startIndex = saved.chapterIndex
        restoreScrollY = saved.scrollY
      }
    }
    chapterIndex.value = startIndex
    const start = chapters.value[startIndex]
    if (start) {
      // 非文本书：按类型分派（音频/视频/漫画/文件）——不走正文分段解析
      if (isNonTextBook.value) await loadNonTextChapter(start.url)
      else await loadContent(start.url)
    }
  } catch {
    loadError.value = true
  } finally {
    loading.value = false
  }
}

function goBack() {
  if (window.history.length > 1) router.back()
  else void router.replace(`/book/${encodeURIComponent(bookUrl.value)}`)
}

/* ---------------- B2 修复：目录抽屉打开时定位当前章（.current 高亮 + scrollIntoView block:center，仅打开时） ---------------- */

const drawerListRef = ref<HTMLElement | null>(null)
watch(drawerOpen, async (open) => {
  if (!open) return
  await nextTick()
  await nextTick()
  const list = drawerListRef.value
  const el = list?.querySelector<HTMLElement>('.chapter-item.current')
  if (list && el) {
    // 抽屉为 fixed 定位容器，scrollIntoView 只会滚动抽屉内部列表，不会带动页面
    el.scrollIntoView({ block: 'center' })
  }
})

/* ---------------- 键盘翻页（←/→ 翻章；PageUp/PageDown 滚动；Space 自动阅读暂停/恢复；输入框聚焦时不触发） ---------------- */

function isTypingTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false
  if (el.isContentEditable) return true
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
}

/** quickKey 自定义动作（legacy quickKey；仅自定义键走此分发，默认键仍走下方默认分支） */
function runQuickAction(action: string) {
  switch (action) {
    case 'nextChapter':
      if (!drawerOpen.value) nextChapter()
      break
    case 'prevChapter':
      if (!drawerOpen.value) prevChapter()
      break
    case 'nextPage':
      if (isFlipMode()) flipPage(1)
      break
    case 'prevPage':
      if (isFlipMode()) flipPage(-1)
      break
    case 'toggleMenu':
      chromeHidden.value = !chromeHidden.value
      break
    case 'toggleTts':
      ttsPanelOpen.value = !ttsPanelOpen.value
      break
    case 'toggleAuto':
      toggleAuto()
      break
    case 'openToc':
      drawerOpen.value = !drawerOpen.value
      break
    case 'addBookmark':
      void addBookmark()
      break
  }
}

function onKeydown(e: KeyboardEvent) {
  if (isTypingTarget(e.target)) return
  if (e.metaKey || e.ctrlKey || e.altKey) return
  const custom = quickKeys.value[e.code]
  if (custom) {
    e.preventDefault()
    runQuickAction(custom)
    return
  }
  switch (e.code) {
    case 'ArrowLeft':
      // 目录抽屉打开时 ←/→ 不翻章（避免浏览目录时误翻）
      if (drawerOpen.value) return
      e.preventDefault()
      if (isFlipMode()) flipPage(-1)
      else prevChapter()
      break
    case 'ArrowRight':
      if (drawerOpen.value) return
      e.preventDefault()
      if (isFlipMode()) flipPage(1)
      else nextChapter()
      break
    case 'PageUp':
      e.preventDefault()
      if (isFlipMode()) flipPage(-1)
      else window.scrollBy({ top: -window.innerHeight * 0.9, behavior: 'auto' })
      break
    case 'PageDown':
      e.preventDefault()
      if (isFlipMode()) flipPage(1)
      else window.scrollBy({ top: window.innerHeight * 0.9, behavior: 'auto' })
      break
    case 'Space':
      // 非文本书：空格 = 播放/暂停；文本书：仿真翻页翻页 / 其余模式自动阅读暂停/恢复（阻止默认的页面滚动）
      e.preventDefault()
      if (isNonTextBook.value) toggleMediaPlay()
      else if (isFlipMode()) flipPage(1)
      else toggleAuto()
      break
    case 'Escape':
      // 图片全屏查看：Esc 关闭
      if (imgViewerOpen.value) {
        e.preventDefault()
        closeImgViewer()
      }
      break
  }
}

/* ---------------- 生命周期 ---------------- */

function onScroll() {
  updateScrollFrac()
  hideSelBar()
  // GAP 155：滚动近底时渐进加载下一段
  maybeLoadMoreChunk()
  window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(saveProgress, 300)
}

onMounted(() => {
  // 跟随系统：matchMedia 监听系统深色偏好（先于 applyTheme，保证 system 主题首帧正确）
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  systemDark.value = mediaQuery.matches
  mediaQuery.addEventListener('change', onSystemThemeChange)
  applyTheme(theme.value)
  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('resize', onResize, { passive: true })
  window.addEventListener('mouseup', onSelectionUp)
  window.addEventListener('touchend', onSelectionUp, { passive: true })
  window.addEventListener('mousedown', onDocMouseDown)
  window.addEventListener('keydown', onKeydown)
  // GAP 85：左右滑动翻章（passive——不 preventDefault，不干扰纵向滚动/选择）
  window.addEventListener('touchstart', onSwipeTouchStart, { passive: true })
  window.addEventListener('touchend', onSwipeTouchEnd, { passive: true })
  if (pageMode.value === 'slide' || pageMode.value === 'flip') {
    window.addEventListener('wheel', onWheel, { passive: false })
    window.addEventListener('touchstart', onTouchStart, { passive: true })
    window.addEventListener('touchmove', onTouchMove, { passive: false })
    window.addEventListener('touchend', onTouchEnd, { passive: true })
  }
  // 屏幕常亮（Wake Lock）：进入阅读页且页面可见时请求；切后台自动释放，回前台自动恢复
  document.addEventListener('visibilitychange', onWakeVisibilityChange)
  if (wakeLockEnabled.value && document.visibilityState === 'visible') void requestWakeLock()
  // GAP 110：每日阅读时长累计（本机）
  startDailyTracker()
  void initCustomFont()
  void init()
})

onBeforeUnmount(() => {
  mediaQuery?.removeEventListener('change', onSystemThemeChange)
  window.removeEventListener('scroll', onScroll)
  window.removeEventListener('resize', onResize)
  window.removeEventListener('wheel', onWheel)
  window.removeEventListener('touchstart', onTouchStart)
  window.removeEventListener('touchmove', onTouchMove)
  window.removeEventListener('touchend', onTouchEnd)
  window.removeEventListener('mouseup', onSelectionUp)
  window.removeEventListener('touchend', onSelectionUp)
  window.removeEventListener('mousedown', onDocMouseDown)
  window.removeEventListener('keydown', onKeydown)
  // GAP 85：左右滑动翻章监听清理
  window.removeEventListener('touchstart', onSwipeTouchStart)
  window.removeEventListener('touchend', onSwipeTouchEnd)
  window.clearTimeout(saveTimer)
  window.clearTimeout(flashTimer)
  prevHold.stop()
  nextHold.stop()
  stopTts()
  stopAuto()
  if (customFontUrl.value) URL.revokeObjectURL(customFontUrl.value)
  if (customFontStyleEl) customFontStyleEl.remove()
  stopDailyTracker()
  // 屏幕常亮：离开阅读页释放
  document.removeEventListener('visibilitychange', onWakeVisibilityChange)
  void releaseWakeLock()
  // 非文本书：暂停媒体 + 销毁 hls.js 实例
  mediaAudioRef.value?.pause()
  videoElRef.value?.pause()
  hlsInstance?.destroy()
  hlsInstance = null
  saveProgress()
})
</script>

<template>
  <div
    ref="pageRef"
    class="reader-page"
    :class="{ texture: effectiveTexture, 'flip-layout': pageMode === 'flip' && isTextBook, 'chrome-hidden': chromeHidden }"
    :style="pageStyle"
  >
    <!-- GAP 149：顶部细进度条（scroll 比例，1px 强调色；点击可跳章） -->
    <button
      v-if="!loading && !loadError && !notFound && realChapters.length > 0"
      class="reading-progress"
      type="button"
      :title="t('reader.jumpTip')"
      @click="jumpOpen = true"
    >
      <i class="reading-progress-fill" :style="{ width: `${progressPct}%` }"></i>
    </button>

    <!-- 顶部极简栏 -->
    <header class="topbar">
      <button class="icon-btn" type="button" :title="t('reader.back')" @click="goBack">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 12H5" />
          <path d="M11 18l-6-6 6-6" />
        </svg>
      </button>

      <span class="book-name" :title="displayBookName">{{ displayBookName || t('reader.title') }}</span>

      <div class="top-actions">
        <button class="font-btn" type="button" title="书籍详情（换源 / 缓存 / 编辑）" @click="router.push(`/book/${encodeURIComponent(bookUrl)}`)">
          详情
        </button>
        <button
          v-if="canSwitchSource()"
          class="font-btn"
          type="button"
          title="阅读中换源（作者 / 最新章节 / 当前章末尾预览）"
          @click="openSource"
        >
          换源
        </button>
        <button
          v-if="isTextBook"
          class="font-btn"
          type="button"
          :disabled="fontSize <= MIN_FONT"
          :title="t('reader.fontDec')"
          @click="fontSize = Math.max(MIN_FONT, fontSize - 1)"
        >
          A-
        </button>
        <button
          v-if="isTextBook"
          class="font-btn"
          type="button"
          :disabled="fontSize >= MAX_FONT"
          :title="t('reader.fontInc')"
          @click="fontSize = Math.min(MAX_FONT, fontSize + 1)"
        >
          A+
        </button>
        <button
          v-if="isTextBook"
          class="font-btn"
          type="button"
          :title="hanMode === 'auto' ? '自动检测（当前：' + (hanTrad ? '繁体→简体' : '简体') + '），点击切换' : hanMode === 'trad' ? '当前繁体，点击切换' : '当前简体，点击切换'"
          @click="toggleHan"
        >
          {{ hanTargetLabel }}
        </button>
        <button class="font-btn" type="button" :title="t('reader.themeTip', { t: t('theme.' + theme) })" @click="cycleTheme">
          {{ t('theme.' + theme) }}
        </button>
        <button
          class="font-btn"
          type="button"
          :class="{ active: brightness !== 1 }"
          :title="t('reader.brightnessTip')"
          @click="brightnessOpen = true"
        >
          {{ t('reader.brightness') }}
        </button>
        <button
          v-if="isTextBook"
          class="font-btn tts-btn"
          type="button"
          :class="{ active: ttsPlaying }"
          :title="
            ttsState === 'playing' || ttsState === 'loading'
              ? t('reader.ttsStop')
              : ttsState === 'paused'
                ? t('reader.ttsResume')
                : t('reader.tts')
          "
          @click="toggleTts"
        >
          {{ ttsTopLabel }}
        </button>
        <button v-if="isTextBook" class="font-btn" type="button" :title="t('reader.addBookmarkTip')" @click="addBookmark">
          {{ t('reader.addBookmark') }}
        </button>
        <button v-if="isTextBook" class="font-btn" type="button" :title="t('reader.bookmarksTip')" @click="openBookmarks">
          {{ t('reader.bookmarks') }}
        </button>
        <button v-if="isTextBook" class="font-btn" type="button" :title="t('reader.searchChapterTip')" @click="openChapterSearch">
          {{ t('reader.searchChapter') }}
        </button>
        <button
          v-if="isTextBook"
          class="font-btn"
          type="button"
          :disabled="loading || loadError || paragraphs.length === 0"
          :title="t('reader.copyChapterTip')"
          @click="copyChapter"
        >
          {{ t('reader.copyChapter') }}
        </button>
        <button
          v-if="isTextBook"
          class="font-btn"
          type="button"
          :disabled="loading || loadError || !currentChapter"
          title="编辑本章正文并保存（服务器 + 本机缓存）"
          @click="openEditChapter"
        >
          编辑
        </button>
        <button
          v-if="isTextBook"
          class="font-btn auto-btn"
          type="button"
          :class="{ active: autoPlaying }"
          :title="autoPlaying ? t('reader.stopAutoTip') : t('reader.autoTip')"
          @click="toggleAuto"
        >
          {{ autoPlaying ? t('reader.stop') : t('reader.auto') }}
        </button>
        <button class="font-btn" type="button" :title="t('reader.layoutTip')" @click="settingsOpen = true">
          {{ t('reader.layout') }}
        </button>
        <button
          v-if="isTextBook"
          class="font-btn"
          type="button"
          title="缓存章节到服务器或本机"
          @click="cacheOpen = true"
        >
          缓存
        </button>
        <button class="toc-btn" type="button" :title="t('reader.tocTip')" @click="drawerOpen = true">
          {{ t('reader.toc') }}
        </button>
      </div>
    </header>

    <!-- 正文 -->
    <main
      class="reader-main"
      :style="{ maxWidth: contentWidth, filter: `brightness(${brightness})` }"
      @click="onReaderAreaClick"
    >
      <!-- 不在书架 -->
      <div v-if="notFound" class="state">
        <p class="state-text">{{ t('reader.notFound') }}</p>
        <button class="retry-btn" type="button" @click="router.replace('/')">{{ t('reader.backShelf') }}</button>
      </div>

      <!-- 目录为空 -->
      <div v-else-if="!loading && !loadError && chapters.length === 0" class="state">
        <p class="state-text">{{ t('reader.noToc') }}</p>
        <button class="retry-btn" type="button" @click="retry">{{ t('common.retry') }}</button>
      </div>

      <template v-else>
        <h1 v-if="currentChapter" class="chapter-title">{{ displayChapterTitle }}</h1>

        <!-- ============ 文本书：现有正文链路 ============ -->
        <template v-if="isTextBook">
          <!-- 加载态：细字 -->
          <div v-if="loading" class="state">
            <p class="state-text loading-text">{{ t('reader.loading') }}</p>
          </div>

          <!-- 错误态 -->
          <div v-else-if="loadError" class="state">
            <p class="state-text">{{ t('reader.loadError') }}</p>
            <button class="retry-btn" type="button" @click="retry">{{ t('common.retry') }}</button>
          </div>

          <!-- 空内容 -->
          <div v-else-if="paragraphs.length === 0" class="state">
            <p class="state-text">{{ t('reader.emptyChapter') }}</p>
          </div>

          <!-- 正文（flip 模式：横向分页容器；其余模式：普通纵向流） -->
          <div
            v-else
            ref="flipViewRef"
            class="flip-view"
            :class="{ active: pageMode === 'flip' }"
            @scroll.passive="onFlipScroll"
          >
            <article
              class="reader-content"
              :class="chapterAnimClass"
              :style="[contentStyle, { '--chapter-anim-duration': `${animateMs}ms` }]"
            >
            <template v-for="(para, i) in visibleParagraphs" :key="i">
              <!-- 单张图片段落（GAP 102）：渲染图片，点击全屏查看 -->
              <img
                v-if="visibleParaImgs[i]"
                v-lazy="visibleParaImgs[i] as string"
                class="reader-img"
                :alt="`正文图片 ${i + 1}`"
                loading="lazy"
                @click="openImgViewer(visibleParaImgs[i] as string)"
              />
              <p
                v-else
                class="reader-para"
                :data-para="i"
                :class="{ flash: flashParaIdx === i, 'tts-reading': ttsReadingPara === i }"
                :style="{ marginBottom: `${paraSpacing}em`, textIndent: textIndent ? '2em' : '0' }"
              >
                <span v-if="paraDisplayHtml(i) !== null" v-html="paraDisplayHtml(i)"></span>
                <template v-else>{{ para }}</template>
              </p>
            </template>
            </article>
          </div>

          <!-- 底部极简导航（长按快进/快退：按住 250ms 后每 300ms 连续翻章） -->
          <nav class="chapter-nav">
            <button
              class="nav-btn"
              type="button"
              :disabled="!hasPrev"
              :title="t('reader.prevTip')"
              @click="onPrevClick"
              @pointerdown="prevHold.start"
              @pointerup="prevHold.stop"
              @pointerleave="prevHold.stop"
              @pointercancel="prevHold.stop"
            >
              {{ t('common.prevChapter') }}
            </button>
            <button
              class="nav-btn"
              type="button"
              :disabled="!hasNext"
              :title="t('reader.nextTip')"
              @click="onNextClick"
              @pointerdown="nextHold.start"
              @pointerup="nextHold.stop"
              @pointerleave="nextHold.stop"
              @pointercancel="nextHold.stop"
            >
              {{ t('common.nextChapter') }}
            </button>
          </nav>
        </template>

        <!-- ============ 非文本书（音频/漫画/视频/文件——按 book_type 分派） ============ -->
        <template v-else>
          <!-- 加载态 -->
          <div v-if="loading" class="state">
            <p class="state-text loading-text">{{ t('reader.loading') }}</p>
          </div>

          <!-- 错误态 -->
          <div v-else-if="loadError" class="state">
            <p class="state-text">内容获取失败，请稍后重试</p>
            <button class="retry-btn" type="button" @click="retry">{{ t('common.retry') }}</button>
          </div>

          <!-- 音频书：播放器（播放/暂停/进度/上一章下一章；m3u8 走 hls.js） -->
          <div v-else-if="isAudioBook" class="media-stage audio-stage">
            <div class="media-card">
              <div
                class="media-art"
                :style="shelfBook?.coverUrl || shelfBook?.customCoverUrl ? { backgroundImage: `url(${shelfBook?.customCoverUrl || shelfBook?.coverUrl})` } : {}"
              >
                <span class="media-badge">{{ t('reader.audioBook') }}</span>
                <span v-if="audioBuffering" class="media-buffering">{{ t('reader.buffering') }}</span>
              </div>
              <p class="media-chapter">{{ displayChapterTitle }}</p>
              <div class="media-controls">
                <button class="media-nav" type="button" :disabled="!hasPrev" :title="t('common.prevChapter')" @click="prevChapter">
                  {{ t('common.prevChapter') }}
                </button>
                <button
                  class="media-play"
                  type="button"
                  :disabled="!audioUrl"
                  :title="audioPlaying ? t('reader.pause') : t('reader.play')"
                  @click="toggleAudioPlay"
                >
                  {{ audioPlaying ? '❚❚' : '▶' }}
                </button>
                <button class="media-nav" type="button" :disabled="!hasNext" :title="t('common.nextChapter')" @click="nextChapter">
                  {{ t('common.nextChapter') }}
                </button>
              </div>
              <div class="media-progress">
                <span class="media-time">{{ fmtTime(audioCurrent) }}</span>
                <input
                  class="media-slider"
                  type="range"
                  min="0"
                  :max="audioDuration || 0"
                  step="1"
                  v-model.number="audioCurrent"
                  :disabled="!audioDuration"
                  @input="seekAudio"
                />
                <span class="media-time">{{ fmtTime(audioDuration) }}</span>
              </div>
              <p v-if="hlsFailed" class="media-hint">
                {{ t('reader.hlsFailed') }}
              </p>
              <p class="media-hint">{{ t('reader.autoNext') }}</p>
            </div>
          </div>

          <!-- 漫画书：横向滑动 + 点击左右边缘翻页 + 懒加载占位 -->
          <div v-else-if="isComicBook" class="media-stage comic-stage">
            <div v-if="comicImages.length === 0" class="state">
              <p class="state-text">{{ t('reader.noComic') }}</p>
            </div>
            <template v-else>
              <div ref="comicScrollRef" class="comic-scroll" @scroll="onComicScroll">
                <div
                  v-for="(img, i) in comicImages"
                  :key="`${currentChapter?.url}-${i}`"
                  class="comic-page"
                  :class="{ active: i === comicPage }"
                >
                  <!-- 加载中占位（懒加载完成前显示） -->
                  <div class="comic-placeholder">{{ t('reader.loading') }}</div>
                  <img
                    v-lazy="img"
                    class="comic-img"
                    :alt="`第 ${i + 1} 页`"
                    loading="lazy"
                    @click="comicClickPage($event, i)"
                  />
                </div>
              </div>
              <div class="comic-foot">
                <span class="comic-hint">{{ t('reader.comicTip') }}</span>
                <span class="comic-page-indicator">{{ t('reader.comicPage', { c: comicPage + 1, t: comicImages.length }) }}</span>
              </div>
            </template>
          </div>

          <!-- 视频书：原生视频播放器 -->
          <div v-else-if="isVideoBook" class="media-stage video-stage">
            <div v-if="!videoUrl" class="state">
              <p class="state-text">{{ t('reader.noVideo') }}</p>
            </div>
            <video
              v-else
              ref="videoElRef"
              class="video-player"
              :src="videoUrl"
              controls
              autoplay
              playsinline
              preload="metadata"
              @timeupdate="onVideoTimeUpdate"
            ></video>
          </div>

          <!-- 文件书：下载按钮 + 说明 -->
          <div v-else-if="isFileBook" class="media-stage file-stage">
            <div class="file-card">
              <p class="file-title">{{ displayBookName }}</p>
              <p class="file-intro">{{ shelfBook?.customIntro || shelfBook?.intro || t('reader.fileIntro') }}</p>
              <a
                v-if="fileUrl"
                class="download-btn"
                :href="fileUrl"
                target="_blank"
                rel="noopener noreferrer"
                :download="displayBookName || 'download'"
              >
                {{ t('reader.download') }}
              </a>
              <p v-else class="media-hint">{{ t('reader.noDownload') }}</p>
            </div>
          </div>

          <!-- 底部导航（长按快进/快退） -->
          <nav v-if="realChapters.length > 0" class="chapter-nav">
            <button
              class="nav-btn"
              type="button"
              :disabled="!hasPrev"
              :title="t('reader.prevTip')"
              @click="onPrevClick"
              @pointerdown="prevHold.start"
              @pointerup="prevHold.stop"
              @pointerleave="prevHold.stop"
              @pointercancel="prevHold.stop"
            >
              {{ t('common.prevChapter') }}
            </button>
            <button
              class="nav-btn"
              type="button"
              :disabled="!hasNext"
              :title="t('reader.nextTip')"
              @click="onNextClick"
              @pointerdown="nextHold.start"
              @pointerup="nextHold.stop"
              @pointerleave="nextHold.stop"
              @pointercancel="nextHold.stop"
            >
              下一章
            </button>
          </nav>
        </template>
      </template>
    </main>

    <!-- 进度：底部细字 + 可点击跳章 -->
    <button
      v-if="!loading && !loadError && !notFound && realChapters.length > 0"
      class="progress-bar"
      type="button"
      title="跳转章节"
      @click="jumpOpen = true"
    >
      <span class="progress-track"><i class="progress-fill" :style="{ width: `${progressPct}%` }"></i></span>
      <span class="progress-text">第 {{ flatIndex + 1 }}/{{ realChapters.length }} 章 · {{ progressPct }}%</span>
    </button>

    <!-- 章节跳转弹层 -->
    <transition name="pop">
      <div v-if="jumpOpen" class="pop-mask" @click="jumpOpen = false">
        <div class="pop-card" @click.stop>
          <p class="pop-title">跳转章节</p>
          <p class="pop-hint">共 {{ realChapters.length }} 章（1 – {{ realChapters.length }}）</p>
          <div class="pop-row">
            <input
              v-model="jumpNum"
              class="pop-input"
              type="number"
              min="1"
              :max="realChapters.length"
              placeholder="章节号"
              @keyup.enter="confirmJump"
            />
            <button class="pop-btn" type="button" @click="confirmJump">跳转</button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 页内搜索弹层（本地搜索当前章：正文 <mark> 高亮 + 上一个/下一个跳转 + 计数） -->
    <transition name="pop">
      <div v-if="searchOpen" class="pop-mask" @click="closeChapterSearch">
        <div class="pop-card search-card" @click.stop>
          <p class="pop-title">搜索本章</p>
          <div class="pop-row">
            <input
              ref="searchInputRef"
              v-model="searchKeyword"
              class="pop-input"
              type="text"
              placeholder="输入关键词，搜索当前章正文"
              spellcheck="false"
              @keyup.enter="runChapterSearch"
              @keyup.esc="closeChapterSearch"
            />
            <button class="pop-btn" type="button" @click="runChapterSearch">搜索</button>
          </div>
          <p v-if="searchSearched && !searchKeyword.trim()" class="pop-hint">请输入关键词</p>
          <p v-else-if="searchSearched && matchTotal === 0" class="pop-hint">
            本章未找到「{{ searchKeyword.trim() }}」
          </p>
          <template v-else-if="searchSearched && matchTotal > 0">
            <p class="pop-hint">
              共 {{ matchTotal }} 处命中 · 当前第 {{ curMatch + 1 }} 处（正文已高亮，上一个/下一个跳转）
            </p>
            <div class="search-nav">
              <button class="pop-btn" type="button" @click="prevMatch">上一个</button>
              <button class="pop-btn" type="button" @click="nextMatch">下一个</button>
              <button class="text-btn" type="button" @click="closeChapterSearch">完成</button>
            </div>
          </template>
          <p v-else class="pop-hint">搜索范围：当前章（前端本地匹配，正文高亮，上一个/下一个跳转）</p>
        </div>
      </div>
    </transition>

    <!-- 亮度弹层（滑条 0.6-1.4，filter: brightness() 作用于 .reader-main） -->
    <transition name="pop">
      <div v-if="brightnessOpen" class="pop-mask" @click="brightnessOpen = false">
        <div class="pop-card" @click.stop>
          <p class="pop-title">亮度</p>
          <p class="pop-hint">0.6 – 1.4 倍 · 作用于正文区域</p>
          <div class="bright-row">
            <input
              v-model.number="brightness"
              class="bright-slider"
              type="range"
              min="0.6"
              max="1.4"
              step="0.05"
              @input="brightness = round2(Number(brightness))"
            />
            <span class="bright-value">{{ brightness.toFixed(2) }}</span>
          </div>
          <div class="set-foot">
            <button class="text-btn" type="button" @click="brightness = 1">恢复默认</button>
            <button class="pop-btn" type="button" @click="brightnessOpen = false">关闭</button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 排版设置弹层（GAP 6：全局 / 本书 两个 tab——本书设置 12 项 per-book 覆盖，优先于全局） -->
    <transition name="pop">
      <div v-if="settingsOpen" class="pop-mask" @click="settingsOpen = false">
        <div class="pop-card settings-card" @click.stop>
          <p class="pop-title">排版设置</p>

          <div class="cfg-tabs">
            <button
              class="cfg-tab"
              :class="{ active: bookCfgTab === 'global' }"
              type="button"
              title="修改对所有书生效"
              @click="bookCfgTab = 'global'"
            >
              全局
            </button>
            <button
              class="cfg-tab"
              :class="{ active: bookCfgTab === 'book' }"
              type="button"
              title="仅本书记忆，优先于全局"
              @click="bookCfgTab = 'book'"
            >
              本书设置
            </button>
          </div>
          <p v-if="bookCfgTab === 'book'" class="pop-hint">本书设置优先于全局（12 项仅本书记忆）</p>

          <!-- 本书专属：主题 / 字号 / 简繁 -->
          <div v-if="bookCfgTab === 'book'" class="set-row">
            <span class="set-label">主题</span>
            <div class="seg">
              <button
                v-for="th in THEME_ORDER"
                :key="th"
                class="seg-btn"
                :class="{ active: theme === th }"
                type="button"
                @click="selectTheme(th)"
              >
                {{ t('theme.' + th) }}
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'book'" class="set-row">
            <span class="set-label">字号</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="fontSize <= MIN_FONT"
                @click="fontSize = Math.max(MIN_FONT, fontSize - 1)"
              >
                −
              </button>
              <span class="set-value">{{ fontSize }}px</span>
              <button
                class="set-btn"
                type="button"
                :disabled="fontSize >= MAX_FONT"
                @click="fontSize = Math.min(MAX_FONT, fontSize + 1)"
              >
                ＋
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'book'" class="set-row">
            <span class="set-label">简繁</span>
            <div class="seg">
              <button
                class="seg-btn"
                :class="{ active: hanMode === 'auto' }"
                type="button"
                @click="hanMode = 'auto'"
              >
                自动
              </button>
              <button
                class="seg-btn"
                :class="{ active: hanMode === 'simp' }"
                type="button"
                @click="hanMode = 'simp'"
              >
                简体
              </button>
              <button
                class="seg-btn"
                :class="{ active: hanMode === 'trad' }"
                type="button"
                @click="hanMode = 'trad'"
              >
                繁体
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">行距</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="lineHeight <= MIN_LINE"
                @click="lineHeight = round1(lineHeight - 0.1)"
              >
                −
              </button>
              <span class="set-value">{{ lineHeight.toFixed(1) }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="lineHeight >= MAX_LINE"
                @click="lineHeight = round1(lineHeight + 0.1)"
              >
                ＋
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">段距</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="paraSpacing <= MIN_PARA"
                @click="paraSpacing = round1(paraSpacing - 0.1)"
              >
                −
              </button>
              <span class="set-value">{{ paraSpacing.toFixed(1) }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="paraSpacing >= MAX_PARA"
                @click="paraSpacing = round1(paraSpacing + 0.1)"
              >
                ＋
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">字重</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="fontWeight <= MIN_WEIGHT"
                @click="fontWeight = Math.max(MIN_WEIGHT, fontWeight - 50)"
              >
                −
              </button>
              <span class="set-value">{{ fontWeight }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="fontWeight >= MAX_WEIGHT"
                @click="fontWeight = Math.min(MAX_WEIGHT, fontWeight + 50)"
              >
                ＋
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">字体</span>
            <div class="font-picker">
              <button
                type="button"
                class="font-trigger"
                :style="{ fontFamily: fontFamilyStyle || undefined }"
                @click="fontOpen = !fontOpen"
              >
                {{ fontLabel }}
                <span class="font-caret" :class="{ open: fontOpen }">▾</span>
              </button>
              <transition name="fade-drop">
                <div v-if="fontOpen" class="font-menu">
                  <button
                    v-for="opt in FONT_OPTIONS"
                    :key="opt.value"
                    type="button"
                    class="font-item"
                    :class="{ active: fontKind === opt.value }"
                    :style="{ fontFamily: FONT_STACK[opt.value] || undefined }"
                    @click="fontKind = opt.value; fontOpen = false"
                  >
                    <span class="font-item-name">{{ opt.label }}</span>
                    <span class="font-item-demo">永州之野产异蛇，黑质而白章</span>
                  </button>
                </div>
              </transition>
            </div>
          </div>

          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">自定义字体</span>
            <div class="set-controls custom-font-row">
              <input
                ref="customFontInput"
                class="visually-hidden"
                type="file"
                accept=".ttf,.otf,.woff,.woff2,font/ttf,font/otf,font/woff,font/woff2"
                @change="onCustomFontPick"
              />
              <button
                class="manage-link"
                type="button"
                title="上传字体文件（本机 IndexedDB 保存，离线可用）"
                @click="customFontInput?.click()"
              >
                {{ customFontUrl ? '更换' : '上传' }}
              </button>
              <button
                v-if="customFontUrl"
                class="switch"
                :class="{ on: customFontEnabled }"
                type="button"
                role="switch"
                :aria-checked="customFontEnabled"
                :title="customFontEnabled ? '关闭自定义字体' : '开启自定义字体'"
                @click="customFontEnabled = !customFontEnabled"
              >
                <span class="switch-knob"></span>
              </button>
              <button
                v-if="customFontUrl"
                class="manage-link"
                type="button"
                title="删除自定义字体"
                @click="clearCustomFont"
              >
                删除
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">字距</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="letterSpacing <= 0"
                @click="letterSpacing = round1(Math.max(0, letterSpacing - 0.5))"
              >
                −
              </button>
              <span class="set-value">{{ letterSpacing }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="letterSpacing >= 2"
                @click="letterSpacing = round1(Math.min(2, letterSpacing + 0.5))"
              >
                ＋
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">缩进</span>
            <div class="seg">
              <button
                class="seg-btn"
                type="button"
                :class="{ active: !textIndent }"
                @click="textIndent = false"
              >
                无
              </button>
              <button
                class="seg-btn"
                type="button"
                :class="{ active: textIndent }"
                @click="textIndent = true"
              >
                2em
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">对齐</span>
            <div class="seg">
              <button
                class="seg-btn"
                type="button"
                :class="{ active: textAlign === 'left' }"
                @click="textAlign = 'left'"
              >
                左
              </button>
              <button
                class="seg-btn"
                type="button"
                :class="{ active: textAlign === 'justify' }"
                @click="textAlign = 'justify'"
              >
                两端
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">纸纹</span>
            <div class="set-controls">
              <button
                class="switch"
                :class="{ on: effectiveTexture }"
                type="button"
                role="switch"
                :aria-checked="effectiveTexture"
                :title="effectiveTexture ? '关闭纸纹' : '开启纸纹（细噪点微纹理，暖色主题效果最佳）'"
                @click="toggleTexture"
              >
                <span class="switch-knob"></span>
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">翻页</span>
            <div class="seg compact">
              <button
                class="seg-btn"
                type="button"
                :class="{ active: pageMode === 'scroll' }"
                title="连续滚动（默认）"
                @click="pageMode = 'scroll'"
              >
                滚动
              </button>
              <button
                class="seg-btn"
                type="button"
                :class="{ active: pageMode === 'hslide' }"
                title="左右滑动翻章（带切章过渡动画）"
                @click="pageMode = 'hslide'"
              >
                左右
              </button>
              <button
                class="seg-btn"
                type="button"
                :class="{ active: pageMode === 'slide' }"
                title="上下翻页（滚轮/触屏/按钮逐屏滚动）"
                @click="pageMode = 'slide'"
              >
                上下
              </button>
              <button
                class="seg-btn"
                type="button"
                :class="{ active: pageMode === 'flip' }"
                title="仿真翻页（横向页过渡）"
                @click="pageMode = 'flip'"
              >
                仿真
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">点击翻页</span>
            <div class="set-controls">
              <button
                class="switch"
                :class="{ on: tapZonesEnabled }"
                type="button"
                role="switch"
                :aria-checked="tapZonesEnabled"
                :title="tapZonesEnabled ? '关闭点击区域（左上上一页 / 右下下一页 / 中间菜单）' : '开启点击区域翻页'"
                @click="tapZonesEnabled = !tapZonesEnabled"
              >
                <span class="switch-knob"></span>
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">屏幕常亮</span>
            <div class="set-controls">
              <button
                class="switch"
                :class="{ on: wakeLockEnabled }"
                type="button"
                role="switch"
                :aria-checked="wakeLockEnabled"
                :title="wakeLockEnabled ? '关闭屏幕常亮（阅读时允许锁屏）' : '开启屏幕常亮（阅读时保持亮屏）'"
                @click="wakeLockEnabled = !wakeLockEnabled"
              >
                <span class="switch-knob"></span>
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">替换规则</span>
            <div class="set-controls">
              <button
                class="switch"
                :class="{ on: replaceEnabled }"
                type="button"
                role="switch"
                :aria-checked="replaceEnabled"
                :title="replaceEnabled ? '关闭正文替换' : '开启正文替换'"
                @click="replaceEnabled = !replaceEnabled"
              >
                <span class="switch-knob"></span>
              </button>
              <button class="manage-link" type="button" title="管理替换规则" @click="router.push('/rules')">
                管理
              </button>
            </div>
          </div>

          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">自动速度</span>
            <div class="seg-row">
              <button
                v-for="s in 5"
                :key="s"
                type="button"
                class="seg-btn"
                :class="{ active: autoSpeed === s }"
                :title="`${s}：约每 ${6 - s} 秒滚一行`"
                @click="autoSpeed = s"
              >
                {{ s }}
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">宽度</span>
            <div class="seg-row">
              <button
                v-for="opt in WIDTH_OPTIONS"
                :key="opt.value"
                type="button"
                class="seg-btn"
                :class="{ active: contentWidth === opt.value }"
                @click="setWidth(opt.value)"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>
          <div v-if="bookCfgTab === 'global'" class="set-row quick-keys-row">
            <span class="set-label">快捷键</span>
            <div class="quick-keys-box">
              <textarea
                v-model="quickKeysText"
                class="quick-keys-input"
                rows="3"
                spellcheck="false"
                placeholder='{"KeyA":"nextChapter"}'
              ></textarea>
              <div class="quick-keys-actions">
                <button class="text-btn" type="button" @click="resetQuickKeys">恢复默认</button>
                <button class="pop-btn" type="button" @click="applyQuickKeys">应用</button>
              </div>
              <p class="quick-keys-hint">可用动作：{{ QUICK_KEY_ACTIONS.map((a) => a.value).join(' / ') }}</p>
            </div>
          </div>
          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">切章动画</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                title="缩短动画"
                @click="animateMs = Math.max(0, animateMs - 50)"
              >
                −
              </button>
              <span class="set-value">{{ animateMs }}ms</span>
              <button
                class="set-btn"
                type="button"
                title="加长动画"
                @click="animateMs = Math.min(1000, animateMs + 50)"
              >
                ＋
              </button>
            </div>
          </div>
          <div v-if="bookCfgTab === 'global'" class="set-row">
            <span class="set-label">章节超时</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                title="缩短超时"
                @click="chapterTimeout = Math.max(10, chapterTimeout - 5)"
              >
                −
              </button>
              <span class="set-value">{{ chapterTimeout }}s</span>
              <button
                class="set-btn"
                type="button"
                title="加长超时"
                @click="chapterTimeout = Math.min(120, chapterTimeout + 5)"
              >
                ＋
              </button>
            </div>
          </div>
          <div class="set-foot">
            <button v-if="bookCfgTab === 'global'" class="text-btn" type="button" @click="resetTypography">
              恢复默认
            </button>
            <button v-else class="text-btn" type="button" title="清除本书设置，恢复使用全局阅读偏好" @click="restoreGlobalDefaults">
              恢复全局默认
            </button>
            <button
              class="text-btn danger"
              type="button"
              title="将本书移出书架"
              @click="openRemoveConfirm"
            >
              移出书架
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 临时书退出挽留：加入书架（项目 dlg 风格） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="retentionOpen" class="dlg-overlay" @click.self="cancelRetention">
          <div
            class="dlg dlg-retention"
            role="dialog"
            aria-modal="true"
            aria-label="加入书架"
            tabindex="-1"
            @keydown.esc="cancelRetention"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">加入书架</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="retentionBusy" @click="cancelRetention">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="dlg-body">
              <p class="dlg-hint">《{{ shelfBook?.name || '本书' }}》加入书架后可保存阅读进度、续读更方便。</p>
            </div>
            <div class="dlg-actions">
              <button class="text-btn" type="button" :disabled="retentionBusy" @click="cancelRetention">
                暂不加入
              </button>
              <button class="pop-btn" type="button" :disabled="retentionBusy" @click="confirmRetention">
                {{ retentionBusy ? '加入中…' : '加入书架' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 移出书架确认（项目 dlg 风格，替代阅读页设置里的二次点击） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="removeOpen" class="dlg-overlay" @click.self="closeRemoveConfirm">
          <div
            class="dlg dlg-remove"
            role="dialog"
            aria-modal="true"
            aria-label="移出书架"
            tabindex="-1"
            @keydown.esc="closeRemoveConfirm"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">移出书架</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="removeBusy" @click="closeRemoveConfirm">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="dlg-body">
              <p class="dlg-hint">确定将《{{ shelfBook?.name || '本书' }}》移出书架吗？本书的阅读进度将一并移除。</p>
            </div>
            <div class="dlg-actions">
              <button class="text-btn" type="button" :disabled="removeBusy" @click="closeRemoveConfirm">
                取消
              </button>
              <button class="text-btn danger" type="button" :disabled="removeBusy" @click="confirmRemoveFromShelf">
                {{ removeBusy ? '移出中…' : '确认移出' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 阅读内换源（作者 / 最新章节 / 当前章末尾预览；点击直接切换并保留进度） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="sourceOpen" class="dlg-overlay" @click.self="closeSource">
          <div
            class="dlg dlg-reader-source"
            role="dialog"
            aria-modal="true"
            aria-label="换源"
            tabindex="-1"
            @keydown.esc="closeSource"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">换源 · {{ displayBookName }}</h2>
              <button
                class="dlg-close"
                type="button"
                title="关闭"
                :disabled="sourceBusy || sourceSwitching"
                @click="closeSource"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="dlg-body source-body">
              <p class="dlg-hint">当前：{{ currentOrigin || '—' }} · 点击结果直接切换并保留当前章进度。</p>

              <div v-if="sourceBusy" class="source-busy">
                <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                  <path d="M21 12a9 9 0 1 1-6.2-8.56" />
                </svg>
                <span v-if="sourceDoneCount > 0">正在搜索其他书源…（已返回 {{ sourceDoneCount }} 个源）</span>
                <span v-else>正在搜索其他书源…</span>
              </div>

              <div v-if="!sourceBusy && sourceResults.length" class="source-tools">
                <input
                  v-model="sourceKeyword"
                  class="source-filter"
                  type="text"
                  placeholder="搜索过滤书源名…"
                  spellcheck="false"
                />
                <button class="source-refresh" type="button" title="重新搜索" @click="refreshSource">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                    <path d="M21 12a9 9 0 1 1-2.64-6.36M21 3v6h-6" />
                  </svg>
                  刷新
                </button>
              </div>

              <ul v-if="sourceFiltered.length" class="source-list">
                <li v-for="(r, i) in sourceFiltered" :key="i">
                  <button
                    class="source-row"
                    :class="{ invalid: invalidSourceUrls.has(r.origin) }"
                    type="button"
                    :disabled="sourceSwitching || r.origin === currentOrigin || invalidSourceUrls.has(r.origin)"
                    :title="r.origin === currentOrigin ? '当前书源' : invalidSourceUrls.has(r.origin) ? '失效书源（不可切换）' : '切换到该书源'"
                    @click="switchSource(r)"
                  >
                    <span class="source-topline">
                      <span class="source-name">{{ r.originName || r.origin || '未知书源' }}</span>
                      <span v-if="previewOf(r)?.author" class="source-author">{{ previewOf(r)?.author }}</span>
                      <span v-if="r.origin === currentOrigin" class="source-cur">当前</span>
                      <span v-else-if="invalidSourceUrls.has(r.origin)" class="source-cur invalid">失效</span>
                    </span>
                    <span class="source-latest" :title="previewOf(r)?.latestChapter || r.latestChapterTitle || ''">
                      最新：{{ previewOf(r)?.latestChapter || r.latestChapterTitle || '—' }}
                    </span>
                    <span
                      class="source-preview"
                      :class="{
                        loading: previewOf(r)?.status === 'loading',
                        error: previewOf(r)?.status === 'error',
                      }"
                    >
                      <template v-if="previewOf(r)?.status === 'loading'">正在获取当前章末尾预览…</template>
                      <template v-else-if="previewOf(r)?.status === 'error'">当前章末尾预览获取失败</template>
                      <template v-else>当前章末尾：{{ previewOf(r)?.currentLast || '—' }}</template>
                    </span>
                  </button>
                </li>
              </ul>

              <p v-else-if="!sourceBusy && sourceResults.length > 0 && sourceFiltered.length === 0" class="source-empty">
                未找到匹配「{{ sourceKeyword }}」的书源
              </p>

              <template v-else-if="!sourceBusy">
                <p v-if="sourceMsg" class="search-msg" :class="{ error: sourceMsgError }">{{ sourceMsg }}</p>
                <div v-if="sourceMsgError" class="source-retry">
                  <button class="ghost-btn" type="button" @click="runSourceSearch">重试</button>
                </div>
              </template>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 自定义主题弹层（第 5 档主题：背景色/文字色/强调色 + 恢复默认；localStorage reader_theme_custom） -->
    <transition name="pop">
      <div v-if="customOpen" class="pop-mask custom-mask" @click="customOpen = false">
        <div class="pop-card" @click.stop>
          <p class="pop-title">自定义主题</p>
          <p class="pop-hint">背景色 / 文字色 / 强调色 · 仅保存在本机</p>
          <div class="custom-row">
            <span class="set-label">背景色</span>
            <input v-model="customTheme.bg" class="custom-color" type="color" title="背景色" />
          </div>
          <div class="custom-row">
            <span class="set-label">文字色</span>
            <input v-model="customTheme.text" class="custom-color" type="color" title="文字色" />
          </div>
          <div class="custom-row">
            <span class="set-label">强调色</span>
            <input v-model="customTheme.accent" class="custom-color" type="color" title="强调色" />
          </div>
          <div class="set-foot">
            <button
              class="text-btn"
              type="button"
              title="恢复默认配色（米白纸色 + 深褐文字 + 棕金强调）"
              @click="resetCustomTheme"
            >
              恢复默认
            </button>
            <button class="pop-btn" type="button" @click="customOpen = false">完成</button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 听书面板（后端语音合成） -->
    <transition name="pop">
      <div v-if="ttsPanelOpen" class="pop-mask" @click="ttsPanelOpen = false">
        <div class="pop-card tts-card" @click.stop>
          <p class="pop-title">听书</p>
          <p class="pop-hint">后端语音合成 · 本章播完自动连播下一章</p>

          <div class="set-row">
            <span class="set-label">引擎</span>
            <div class="seg">
              <button
                class="seg-btn"
                type="button"
                :class="{ active: ttsEngine === 'edge' }"
                @click="ttsEngine = 'edge'"
              >
                Edge
              </button>
              <button
                class="seg-btn"
                type="button"
                :class="{ active: ttsEngine === 'http' }"
                :disabled="ttsHttpList.length === 0"
                :title="ttsHttpList.length === 0 ? '未配置 HttpTTS 源（设置页添加）' : ''"
                @click="ttsEngine = 'http'"
              >
                HttpTTS
              </button>
            </div>
          </div>

          <div v-if="ttsEngine === 'http'" class="set-row">
            <span class="set-label">音源</span>
            <select v-model="ttsHttpUrl" class="tts-select">
              <option v-for="t in ttsHttpList" :key="t.url" :value="t.url">{{ t.name }}</option>
            </select>
          </div>

          <div class="set-row">
            <span class="set-label">音色</span>
            <select v-model="ttsVoice" class="tts-select" :title="ttsVoice">
              <optgroup v-for="g in ttsLocaleGroups" :key="g.label" :label="g.label">
                <option v-for="v in g.voices" :key="v.value" :value="v.value">
                  {{ v.name }} · {{ v.gender === 'Female' ? '女声' : '男声' }}
                </option>
              </optgroup>
              <option v-if="ttsVoices.length === 0" :value="ttsVoice">{{ ttsVoice }}</option>
            </select>
          </div>

          <div class="set-row">
            <span class="set-label">语速</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="ttsRate <= 0.5"
                title="减慢"
                @click="ttsRate = round1(Math.max(0.5, ttsRate - 0.1))"
              >
                −
              </button>
              <span class="set-value">{{ ttsRate.toFixed(1) }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="ttsRate >= 2"
                title="加快"
                @click="ttsRate = round1(Math.min(2, ttsRate + 0.1))"
              >
                ＋
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">音调</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="ttsPitch <= -10"
                title="降低音调"
                @click="ttsPitch = Math.max(-10, ttsPitch - 1)"
              >
                −
              </button>
              <span class="set-value">{{ ttsPitchParam }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="ttsPitch >= 10"
                title="升高音调"
                @click="ttsPitch = Math.min(10, ttsPitch + 1)"
              >
                ＋
              </button>
            </div>
          </div>

          <div class="set-row">
            <span class="set-label">音量</span>
            <div class="set-controls">
              <button
                class="set-btn"
                type="button"
                :disabled="ttsVolume <= 0"
                title="降低音量"
                @click="ttsVolume = Math.max(0, ttsVolume - 10)"
              >
                −
              </button>
              <span class="set-value">{{ ttsVolumeParam }}</span>
              <button
                class="set-btn"
                type="button"
                :disabled="ttsVolume >= 200"
                title="提高音量"
                @click="ttsVolume = Math.min(200, ttsVolume + 10)"
              >
                ＋
              </button>
            </div>
          </div>

          <div v-if="ttsEngine === 'edge'" class="set-row">
            <span class="set-label">风格</span>
            <select v-model="ttsStyle" class="tts-select">
              <option value="">无</option>
              <option value="cheerful">开心</option>
              <option value="sad">悲伤</option>
              <option value="angry">生气</option>
              <option value="fearful">害怕</option>
              <option value="excited">兴奋</option>
              <option value="friendly">友好</option>
              <option value="gentle">温柔</option>
              <option value="hopeful">希望</option>
              <option value="lyrical">抒情</option>
              <option value="newscast">新闻</option>
              <option value="poetry-reading">朗读</option>
              <option value="serious">严肃</option>
              <option value="shouting">呼喊</option>
              <option value="whispering">耳语</option>
            </select>
          </div>

          <div class="tts-controls">
            <button
              class="pop-btn tts-play"
              type="button"
              :disabled="ttsState === 'loading'"
              @click="ttsPlayPause"
            >
              {{ ttsState === 'playing' ? '暂停' : ttsState === 'paused' ? '继续' : ttsState === 'loading' ? '加载中…' : '播放' }}
            </button>
            <button class="tts-stop" type="button" :disabled="ttsState === 'idle'" @click="stopTts">
              停止
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 隐藏音频元素：后端 TTS 音频流播放（ended → 自动连播） -->
    <audio ref="ttsAudioRef" preload="auto" @ended="onTtsEnded" @error="onTtsError"></audio>

    <!-- 音频书播放元素（书源音频流直连；ended → 自动下一章；m3u8 经 hls.js 接管） -->
    <audio
      v-if="isAudioBook"
      ref="mediaAudioRef"
      preload="auto"
      @loadedmetadata="onAudioLoadedMeta"
      @timeupdate="onAudioTimeUpdate"
      @play="onAudioPlay"
      @pause="onAudioPause"
      @waiting="onAudioWaiting"
      @playing="onAudioPlaying"
      @ended="onAudioEnded"
    ></audio>

    <!-- GAP 102：正文图片全屏查看（遮罩 + 原始图 + 关闭；点击遮罩/关闭按钮退出） -->
    <transition name="pop">
      <div v-if="imgViewerOpen" class="img-viewer" @click="closeImgViewer">
        <img :src="imgViewerUrl" class="img-viewer-original" alt="正文图片" @click.stop />
        <button class="img-viewer-close" type="button" title="关闭" @click="closeImgViewer">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
            <path d="M6 6l12 12M18 6L6 18" />
          </svg>
        </button>
      </div>
    </transition>

    <!-- 书签列表弹层 -->
    <transition name="pop">
      <div v-if="bookmarksOpen" class="pop-mask" @click="bookmarksOpen = false">
        <div class="pop-card bookmark-card" @click.stop>
          <div class="pop-head">
            <p class="pop-title">书签{{ bookmarkSelected.size ? ` · 已选 ${bookmarkSelected.size}` : '' }}</p>
            <div class="pop-actions">
              <input
                ref="bookmarkImportRef"
                class="visually-hidden"
                type="file"
                accept="application/json,.json"
                @change="onBookmarkImportChange"
              />
              <button type="button" class="pop-btn" title="导入书签 JSON" @click="bookmarkImportRef?.click()">导入</button>
              <button type="button" class="pop-btn" title="导出当前书签 JSON" :disabled="bookmarks.length === 0" @click="exportBookmarksJson">导出</button>
              <button
                v-if="bookmarkSelected.size > 0"
                type="button"
                class="pop-btn danger"
                title="删除勾选书签"
                @click="deleteSelectedBookmarks"
              >
                删除勾选 ({{ bookmarkSelected.size }})
              </button>
              <button
                v-else
                type="button"
                class="pop-btn"
                title="进入多选模式"
                @click="bookmarkSelected = new Set(bookmarks.map((b) => b.title))"
              >
                全选
              </button>
            </div>
          </div>
          <p v-if="bookmarkLoading" class="pop-hint">加载中…</p>
          <p v-else-if="bookmarks.length === 0" class="pop-hint">暂无书签，点顶栏「＋书签」添加</p>
          <ul v-else class="bm-list">
            <li v-for="(b, i) in bookmarks" :key="`${b.title}-${b.createdAt}-${i}`" class="bm-item">
              <input
                v-if="bookmarkSelected.size > 0 || true"
                class="bm-check"
                type="checkbox"
                :checked="bookmarkSelected.has(b.title)"
                :title="bookmarkSelected.size === 0 ? '多选模式：勾选后批量删除' : '勾选/取消'"
                @change="toggleBookmarkSelect(b.title)"
              />
              <button
                type="button"
                class="bm-jump"
                :title="`跳转：${chapterTitleAt(b.chapterIndex)}`"
                @click="jumpToBookmark(b)"
              >
                <span class="bm-chapter">
                  {{ b.bookName ? `${b.bookName} · ` : '' }}{{ chapterTitleAt(b.chapterIndex) || b.chapterName || `第 ${b.chapterIndex + 1} 章` }}
                </span>
                <span class="bm-text">{{ b.title }}</span>
                <span v-if="b.bookText" class="bm-quote" :title="b.bookText">{{ b.bookText }}</span>
                <span v-if="b.content" class="bm-note">{{ b.content }}</span>
                <span class="bm-time">{{ fmtBookmarkTime(b.createdAt) }}</span>
              </button>
              <button type="button" class="bm-edit" title="编辑书签" @click="editBookmark(b)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M12 20h9" />
                  <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
                </svg>
              </button>
              <button type="button" class="bm-del" title="删除书签" @click="deleteBookmarkItem(b)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </li>
          </ul>
        </div>
      </div>
    </transition>

    <!-- 书签编辑弹层 -->
    <transition name="pop">
      <div v-if="bookmarkEditing" class="pop-mask" @click="bookmarkEditing = null">
        <div class="pop-card bookmark-edit-card" @click.stop>
          <p class="pop-title">编辑书签</p>
          <label class="edit-field">
            <span>标题</span>
            <input v-model="bookmarkEditing.title" type="text" placeholder="书签标题" />
          </label>
          <label class="edit-field">
            <span>备注</span>
            <textarea v-model="bookmarkEditing.content" rows="3" placeholder="备注（可选）"></textarea>
          </label>
          <label class="edit-field">
            <span>正文</span>
            <textarea v-model="bookmarkEditing.bookText" rows="4" placeholder="书签段落文本（可选）"></textarea>
          </label>
          <div class="edit-actions">
            <button type="button" class="ghost-btn" @click="bookmarkEditing = null">取消</button>
            <button type="button" class="primary-btn" :disabled="bookmarkSaving" @click="saveBookmarkEdit">
              {{ bookmarkSaving ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 划词工具条（复制 / 搜索 / 朗读） -->
    <transition name="pop">
      <div v-if="selOpen" class="sel-bar" :style="{ left: `${selX}px`, top: `${selY}px` }">
        <button type="button" class="sel-btn" @click="copySelection">复制</button>
        <button type="button" class="sel-btn" @click="searchSelection">搜索</button>
        <button type="button" class="sel-btn" title="朗读选中文本" @click="speakSelection">朗读</button>
        <button type="button" class="sel-btn" title="把选中文本添加为书签" @click="addBookmarkFromSelection">书签</button>
        <button type="button" class="sel-btn" title="把选中文本添加为过滤规则（替换为空）" @click="addFilterFromSelection">过滤</button>
      </div>
    </transition>

    <!-- 章节侧栏 -->
    <transition name="drawer">
      <div v-if="drawerOpen" class="drawer-mask" @click="drawerOpen = false">
        <aside class="chapter-drawer" @click.stop>
          <header class="drawer-head">
            <span class="drawer-title">目录</span>
            <div class="drawer-tools">
              <input
                v-model="tocKeyword"
                class="drawer-search"
                type="text"
                placeholder="搜索章节"
                spellcheck="false"
              />
              <button
                class="drawer-reverse"
                type="button"
                :class="{ active: tocReverse }"
                :title="tocReverse ? '恢复正序' : '倒序显示'"
                @click="tocReverse = !tocReverse"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M4 7h10" />
                  <path d="M4 12h7" />
                  <path d="M4 17h4" />
                  <path d="M15 15l4 4 4-4" />
                  <path d="M19 19V5" />
                </svg>
              </button>
            </div>
            <button class="drawer-close" type="button" title="关闭" @click="drawerOpen = false">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <path d="M6 6l12 12M18 6L6 18" />
              </svg>
            </button>
          </header>
          <div ref="drawerListRef" class="drawer-list">
            <template v-for="(ch, i) in drawerChapters" :key="`${ch.url}-${i}`">
              <button
                v-if="ch.isVolume"
                type="button"
                class="chapter-volume"
                :class="{ collapsed: tocCollapsed[ch.title] }"
                :title="tocCollapsed[ch.title] ? '展开本卷' : '折叠本卷'"
                @click="toggleVolume(ch.title)"
              >
                <span class="vol-arrow">{{ tocCollapsed[ch.title] ? '▸' : '▾' }}</span>
                <span class="vol-title">{{ ch.title }}</span>
              </button>
              <button
                v-else
                v-show="!chapterHidden[i]"
                type="button"
                class="chapter-item"
                :class="{ current: ch.index === chapterIndex }"
                @click="goToChapter(ch.index)"
              >
                <span class="chapter-item-title">{{ ch.title }}</span>
                <span v-if="ch.wcLabel" class="chapter-item-wc">{{ ch.wcLabel }}</span>
                <span
                  v-if="cachedChapterIndexes.has(ch.index)"
                  class="chapter-item-cached"
                  title="已缓存（服务器或本机）"
                >已缓存</span>
              </button>
            </template>
          </div>
        </aside>
      </div>
    </transition>

    <!-- 翻页模式浮动按钮：上下翻页 ▲/▼ 逐屏；左右滑动 ‹/› 翻章；仿真翻页 ‹/› 翻页 -->
    <template v-if="isTextBook && !loading && !loadError && paragraphs.length > 0">
      <div v-if="pageMode === 'slide'" class="flip-nav-vert">
        <button class="flip-nav-btn" type="button" title="上一屏" @click="slideFlip(-1)">▲</button>
        <button class="flip-nav-btn" type="button" title="下一屏" @click="slideFlip(1)">▼</button>
      </div>
      <template v-else-if="pageMode === 'hslide' || pageMode === 'flip'">
        <button
          class="flip-nav-btn flip-nav-side prev"
          type="button"
          :disabled="pageMode === 'hslide' ? !hasPrev : flipPageIdx <= 0"
          :title="pageMode === 'flip' ? '上一页' : t('common.prevChapter')"
          @click="pageMode === 'flip' ? flipPage(-1) : prevChapter()"
        >
          ‹
        </button>
        <button
          class="flip-nav-btn flip-nav-side next"
          type="button"
          :disabled="pageMode === 'hslide' ? !hasNext : false"
          :title="pageMode === 'flip' ? '下一页' : t('common.nextChapter')"
          @click="pageMode === 'flip' ? flipPage(1) : nextChapter()"
        >
          ›
        </button>
      </template>
    </template>

    <!-- 正文编辑弹层（legacy saveBookContent：保存服务器 + 本机缓存） -->
    <transition name="pop">
      <div v-if="editOpen" class="pop-mask" @click.self="closeEditChapter">
        <div class="pop-card edit-card" @click.stop>
          <p class="pop-title">编辑本章正文</p>
          <p class="pop-hint">保存后写入服务器缓存并同步本机，正文将按换行分段。</p>
          <textarea
            v-model="editText"
            class="edit-textarea"
            rows="14"
            spellcheck="false"
            :disabled="editSaving"
            placeholder="正文内容…"
          ></textarea>
          <div class="pop-actions">
            <button class="text-btn" type="button" :disabled="editSaving" @click="closeEditChapter">取消</button>
            <button class="pop-btn" type="button" :disabled="editSaving || !editText.trim()" @click="saveEditChapter">
              {{ editSaving ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- 章节缓存弹层（服务器 / 本机双向：当前章、至末尾、全本、指定范围） -->
    <ChapterCacheDialog
      v-model="cacheOpen"
      :book-url="bookUrl"
      :book-name="bookName || displayBookName"
      :chapters="realChapters"
      :origin="shelfBook?.origin || ''"
      :default-from="flatIndex + 1"
      :default-scope="'all'"
      :allow-server="!shelfBook?.isTemp"
      @done="onCacheDone"
    />
  </div>
</template>

<style scoped>
.reader-page {
  min-height: 100vh;
  /* 阅读主题变量在 .reader-page[data-reader-theme] 上覆盖（与界面主题分离），此处显式取背景 */
  background: var(--bg);
  animation: fade-in 0.2s ease both;
}

/* ================= GAP 5：纸纹（细噪点——radial-gradient 微纹理叠在背景色上；亮色主题黑点、深色主题白点） ================= */
.reader-page.texture {
  background-image:
    radial-gradient(rgba(0, 0, 0, 0.03) 0.7px, transparent 0.9px),
    radial-gradient(rgba(0, 0, 0, 0.018) 0.5px, transparent 0.8px);
  background-size: 4px 4px, 6px 6px;
  background-position: 0 0, 2px 3px;
}
.reader-page.texture[data-reader-theme='dark'] {
  background-image:
    radial-gradient(rgba(255, 255, 255, 0.025) 0.7px, transparent 0.9px),
    radial-gradient(rgba(255, 255, 255, 0.015) 0.5px, transparent 0.8px);
}

/* ================= GAP 149：顶部细进度条（1px 强调色，scroll 比例） ================= */
.reading-progress {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 25;
  height: 2px;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
}
.reading-progress-fill {
  display: block;
  height: 100%;
  background: var(--accent);
  transition: width 0.15s ease;
}

/* ================= 顶部极简栏 ================= */
.topbar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 24px;
  background: var(--bg-float);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}
/* 点击区域收起/唤出菜单：隐藏顶部栏与浮动导航（保留正文完整可读） */
.topbar,
.chapter-nav,
.flip-nav-vert,
.flip-nav-side {
  transition:
    opacity 0.22s ease,
    transform 0.22s ease;
}
.reader-page.chrome-hidden .topbar {
  opacity: 0;
  pointer-events: none;
  transform: translateY(-8px);
}
.reader-page.chrome-hidden .chapter-nav,
.reader-page.chrome-hidden .flip-nav-vert,
.reader-page.chrome-hidden .flip-nav-side {
  opacity: 0;
  pointer-events: none;
}
.icon-btn {
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.icon-btn:hover {
  color: var(--text-1);
  border-color: var(--border);
}
.icon-btn svg {
  width: 16px;
  height: 16px;
}
.book-name {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: center;
}
.top-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.font-btn {
  min-width: 34px;
  height: 30px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.font-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.font-btn.tts-btn.active,
.font-btn.auto-btn.active,
.font-btn.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.font-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.toc-btn {
  height: 30px;
  padding: 0 12px;
  margin-left: 4px;
  border: none;
  border-radius: var(--radius);
  background: var(--accent-soft);
  color: var(--accent);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition: background 0.2s ease;
}
.toc-btn:hover {
  background: var(--accent-soft);
  filter: brightness(1.06);
}

/* ================= 正文 ================= */
.reader-main {
  width: 100%;
  margin: 0 auto;
  padding: 48px 24px 150px;
}

.chapter-title {
  margin: 0 0 36px;
  font-size: 20px;
  font-weight: 300;
  letter-spacing: 2px;
  text-align: center;
  color: var(--text-1);
}

.reader-content {
  color: var(--text-1);
}
.reader-para {
  margin: 0 0 1em;
  text-indent: 2em;
  word-break: break-word;
}
/* 搜索跳转后的短暂高亮 */
.reader-para.flash {
  animation: search-flash 1.6s ease;
}
@keyframes search-flash {
  0%,
  55% {
    background: rgba(255, 193, 7, 0.35);
  }
  100% {
    background: transparent;
  }
}

/* 页内搜索：正文 <mark> 高亮（v-html 注入 → :deep）——普通命中淡黄底，当前命中描边突出 */
.reader-para :deep(mark) {
  background: rgba(255, 193, 7, 0.45);
  color: inherit;
  border-radius: 2px;
  padding: 0 1px;
}
.reader-para :deep(mark.mark-current) {
  background: rgba(255, 170, 0, 0.75);
  box-shadow: 0 0 0 2px rgba(255, 170, 0, 0.5);
  color: #000;
}

/* GAP 7：TTS 朗读中当前段落浅背景高亮 */
.reader-para.tts-reading {
  background: var(--tts-reading-bg, rgba(79, 70, 229, 0.12));
  border-radius: 4px;
  box-shadow: 0 0 0 4px var(--tts-reading-bg, rgba(79, 70, 229, 0.12));
  transition: background 0.25s ease;
}

/* GAP 102：正文图片（段落内单张图片） */
.reader-img {
  display: block;
  max-width: 100%;
  max-height: 72vh;
  margin: 0 auto 1em;
  border-radius: 8px;
  cursor: zoom-in;
  opacity: 0;
  transition: opacity 0.3s ease;
}
.reader-img.is-loaded {
  opacity: 1;
}

/* GAP 102：图片全屏查看层 */
.img-viewer {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  background: rgba(10, 10, 12, 0.92);
}
.img-viewer-original {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 4px;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.5);
}
.img-viewer-close {
  position: fixed;
  top: 18px;
  right: 18px;
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(255, 255, 255, 0.25);
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.9);
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease;
}
.img-viewer-close:hover {
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
}
.img-viewer-close svg {
  width: 16px;
  height: 16px;
}

/* ================= 加载 / 错误 / 空 ================= */
.state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 80px 0;
}
.state-text {
  margin: 0;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.loading-text {
  animation: pulse 1.2s ease-in-out infinite;
}
@keyframes pulse {
  0%,
  100% {
    opacity: 0.45;
  }
  50% {
    opacity: 1;
  }
}
.retry-btn {
  padding: 8px 30px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.retry-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 底部导航 ================= */
.chapter-nav {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-top: 64px;
  padding-top: 32px;
  border-top: 1px solid var(--border);
}
.nav-btn {
  height: 42px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 3px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.nav-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.nav-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

/* ================= 非文本书（音频/漫画/视频/文件） ================= */
.media-stage {
  padding: 24px 0 8px;
}
.media-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  padding: 36px 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
}
.media-art {
  position: relative;
  width: 150px;
  height: 150px;
  border-radius: 12px;
  background: var(--border) center/cover no-repeat;
  background-image: linear-gradient(135deg, var(--border-strong), var(--border));
  display: flex;
  align-items: center;
  justify-content: center;
}
.media-badge {
  padding: 5px 14px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  font-size: 12px;
  letter-spacing: 2px;
}
.media-buffering {
  position: absolute;
  right: 8px;
  bottom: 8px;
  padding: 3px 10px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.5);
  color: #fff;
  font-size: 11px;
}
.media-chapter {
  margin: 0;
  font-size: 15px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
  text-align: center;
}
.media-controls {
  display: flex;
  align-items: center;
  gap: 22px;
}
.media-nav {
  height: 36px;
  padding: 0 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.media-nav:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.media-nav:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.media-play {
  width: 62px;
  height: 62px;
  border: none;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
  font-size: 20px;
  font-family: inherit;
  cursor: pointer;
  box-shadow: 0 4px 18px rgba(0, 0, 0, 0.18);
  transition: transform 0.15s ease;
}
.media-play:hover:not(:disabled) {
  transform: scale(1.06);
}
.media-play:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.media-progress {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  max-width: 520px;
}
.media-time {
  min-width: 44px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-3);
}
.media-slider {
  flex: 1;
  accent-color: var(--accent);
  cursor: pointer;
}
.media-hint {
  margin: 0;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  text-align: center;
}

/* ---- 漫画书 ---- */
.comic-scroll {
  display: flex;
  overflow-x: auto;
  scroll-snap-type: x mandatory;
  gap: 10px;
  padding: 4px 2px 14px;
  scrollbar-width: thin;
  -webkit-overflow-scrolling: touch;
}
.comic-page {
  position: relative;
  flex: 0 0 min(96vw, 860px);
  scroll-snap-align: center;
  border-radius: 6px;
  overflow: hidden;
  background: var(--border);
  min-height: 220px;
}
.comic-page.active {
  outline: 1px solid var(--accent);
}
.comic-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  letter-spacing: 2px;
  color: var(--text-3);
  animation: pulse 1.2s ease-in-out infinite;
}
.comic-img {
  position: relative;
  display: block;
  width: 100%;
  height: auto;
  min-height: 220px;
  object-fit: contain;
  cursor: pointer;
}
.comic-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 6px 2px 0;
  flex-wrap: wrap;
}
.comic-hint {
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.comic-page-indicator {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  letter-spacing: 1px;
  color: var(--text-2);
}

/* ---- 视频书 ---- */
.video-player {
  display: block;
  width: 100%;
  max-height: 72vh;
  border-radius: var(--radius);
  background: #000;
}

/* ---- 文件书 ---- */
.file-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  padding: 48px 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  text-align: center;
}
.file-title {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
  letter-spacing: 1px;
  color: var(--text-1);
}
.file-intro {
  margin: 0;
  max-width: 560px;
  font-size: 13px;
  font-weight: 300;
  line-height: 1.9;
  color: var(--text-2);
  white-space: pre-wrap;
}
.download-btn {
  display: inline-block;
  padding: 12px 44px;
  border-radius: var(--radius);
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  letter-spacing: 3px;
  text-decoration: none;
  box-shadow: 0 4px 18px rgba(0, 0, 0, 0.15);
  transition: transform 0.15s ease;
}
.download-btn:hover {
  transform: scale(1.04);
}

/* ================= 进度条（底部细字） ================= */
.progress-bar {
  position: fixed;
  left: 50%;
  bottom: 0;
  transform: translateX(-50%);
  z-index: 30;
  width: min(680px, 100%);
  padding: 10px 24px 10px;
  border: none;
  border-top: 1px solid var(--border);
  background: var(--bg);
  box-shadow: 0 -4px 16px rgba(0, 0, 0, 0.04);
  cursor: pointer;
  font-family: inherit;
  text-align: center;
}
.progress-track {
  display: block;
  height: 2px;
  background: var(--border);
  border-radius: 1px;
  overflow: hidden;
}
.progress-fill {
  display: block;
  height: 100%;
  background: var(--accent);
  transition: width 0.2s ease;
}
.progress-text {
  display: block;
  margin-top: 7px;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1.5px;
  color: var(--text-3);
  transition: color 0.2s ease;
}
.progress-bar:hover .progress-text {
  color: var(--accent);
}

.seg-row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.font-picker {
  position: relative;
}
.font-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.font-trigger:hover {
  border-color: var(--accent);
}
.font-caret {
  font-size: 10px;
  color: var(--text-3);
  transition: transform 0.2s ease;
}
.font-caret.open {
  transform: rotate(180deg);
}
.font-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 50;
  min-width: 300px;
  max-height: 300px;
  overflow-y: auto;
  background: var(--bg-float);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  padding: 4px;
}
.font-item {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 9px 12px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  background: none;
  border: none;
  border-radius: 6px;
  text-align: left;
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}
.font-item-name {
  flex-shrink: 0;
  font-size: 13px;
}
.font-item-demo {
  font-size: 12px;
  color: var(--text-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.font-item:hover {
  background: var(--accent-soft);
  color: var(--accent);
}
.font-item.active {
  color: var(--accent);
  background: var(--accent-soft);
}
.fade-drop-enter-active,
.fade-drop-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-drop-enter-from,
.fade-drop-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
.seg-btn {
  padding: 4px 14px;
  font-size: 12px;
  font-weight: 400;
  color: var(--text-2);
  background: none;
  border: 1px solid var(--border);
  border-radius: 999px;
  cursor: pointer;
  transition: all 0.2s ease;
}
.seg-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.seg-btn.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}

/* ================= 弹层（设置 / 跳章） ================= */
/* ================= 项目 dlg 弹窗（加入书架挽留 / 移出书架确认） ================= */
.dlg-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(24, 24, 27, 0.35);
}
.dlg {
  width: min(440px, 100%);
  max-height: min(560px, 86vh);
  display: flex;
  flex-direction: column;
  padding: 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  outline: none;
}
.dlg-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.dlg-title {
  margin: 0;
  font-size: 15px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.dlg-close {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    background-color 0.2s ease;
}
.dlg-close:hover:not(:disabled) {
  color: var(--text-1);
  background: #f4f4f5;
}
.dlg-close:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
.dlg-close svg {
  width: 13px;
  height: 13px;
}
.dlg-body {
  min-height: 0;
  overflow-y: auto;
}
.dlg-hint {
  margin: 0;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  line-height: 1.8;
  color: var(--text-3);
}
.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
}
/* ================= 阅读内换源弹层（作者 / 最新章节 / 当前章末尾预览） ================= */
.dlg-reader-source {
  width: min(560px, 100%);
}
.source-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
}
.source-busy {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 18px 4px;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}
.mini-spin {
  width: 13px;
  height: 13px;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
.source-tools {
  display: flex;
  gap: 8px;
  align-items: center;
}
.source-filter {
  flex: 1;
  min-width: 0;
  padding: 6px 10px;
  font-size: 13px;
  font-weight: 300;
  border: 1px solid var(--border, #ececec);
  border-radius: 8px;
  background: var(--card, #fff);
  color: var(--text-1, #333);
  outline: none;
  transition: border-color 0.2s ease;
}
.source-filter:focus {
  border-color: var(--accent, #4f46e5);
}
.source-refresh {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2, #666);
  background: none;
  border: 1px solid var(--border, #ececec);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}
.source-refresh:hover {
  border-color: var(--accent, #4f46e5);
  color: var(--accent, #4f46e5);
}
.source-refresh svg {
  width: 13px;
  height: 13px;
}
.source-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.source-row {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.source-row:hover:not(:disabled) {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.source-row:disabled {
  cursor: default;
  opacity: 0.6;
}
.source-topline {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}
.source-name {
  flex-shrink: 0;
  max-width: 220px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-author {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-latest {
  display: block;
  min-width: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-preview {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 34px;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.45;
  color: var(--text-2);
  word-break: break-word;
}
.source-preview.loading {
  color: var(--text-3);
}
.source-preview.error {
  color: rgba(207, 68, 68, 0.85);
}
.source-cur {
  flex-shrink: 0;
  padding: 1px 7px;
  border-radius: 999px;
  border: 1px solid var(--accent);
  color: var(--accent);
  font-size: 10.5px;
  font-weight: 400;
  letter-spacing: 1px;
}
.source-row.invalid {
  opacity: 0.55;
}
.source-cur.invalid {
  border-color: rgba(207, 68, 68, 0.5);
  color: #cf4444;
}
.source-empty {
  padding: 14px 4px;
  font-size: 13px;
  font-weight: 300;
  color: var(--text-2, #888);
}
.source-retry {
  display: flex;
  justify-content: flex-start;
}
.search-msg {
  margin: 0;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-2);
}
.search-msg.error {
  color: rgba(207, 68, 68, 0.9);
}
.ghost-btn {
  padding: 6px 14px;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-2);
  background: none;
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}
.ghost-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.dlg-enter-active,
.dlg-leave-active {
  transition: opacity 0.2s ease;
}
.dlg-enter-from,
.dlg-leave-to {
  opacity: 0;
}
.dlg-enter-active .dlg,
.dlg-leave-active .dlg {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}
.dlg-enter-from .dlg,
.dlg-leave-to .dlg {
  opacity: 0;
  transform: translateY(6px);
}

.pop-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(24, 24, 27, 0.28);
}
.pop-enter-active,
.pop-leave-active {
  transition: opacity 0.2s ease;
}
.pop-enter-from,
.pop-leave-to {
  opacity: 0;
}
.pop-card {
  width: min(320px, 86vw);
  padding: 26px 26px 20px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface);
  /* GAP 6：本书设置 12 行较高——弹层超高时内部滚动 */
  max-height: 84vh;
  overflow-y: auto;
}
.pop-title {
  margin: 0;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--text-1);
  text-align: center;
}
.pop-hint {
  margin: 10px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  text-align: center;
}
.edit-card {
  width: min(560px, 92vw);
}
.edit-textarea {
  width: 100%;
  margin-top: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13.5px;
  font-weight: 300;
  line-height: 1.8;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.2s ease;
}
.edit-textarea:focus {
  border-color: var(--accent);
}
.edit-textarea:disabled {
  opacity: 0.6;
}

/* ================= 自定义主题弹层 ================= */
.custom-mask {
  z-index: 45;
}
.custom-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 16px;
}
.custom-color {
  width: 46px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: none;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.custom-color:hover {
  border-color: var(--accent);
}

/* ================= 亮度弹层 ================= */
.bright-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 22px;
}
.bright-slider {
  flex: 1;
  min-width: 0;
  height: 4px;
  accent-color: var(--accent);
  cursor: pointer;
}
.bright-value {
  flex-shrink: 0;
  min-width: 38px;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-1);
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.pop-row {
  display: flex;
  gap: 10px;
  margin-top: 18px;
}
.pop-input {
  flex: 1;
  min-width: 0;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  outline: none;
  transition: border-color 0.2s ease;
}
.pop-input:focus {
  border-color: var(--accent);
}
.pop-input::-webkit-outer-spin-button,
.pop-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.pop-btn {
  flex-shrink: 0;
  height: 36px;
  padding: 0 18px;
  border: none;
  border-radius: var(--radius);
  background: var(--accent);
  color: var(--on-accent);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 2px;
  cursor: pointer;
  transition: background 0.2s ease;
}
.pop-btn:hover {
  background: var(--accent-deep);
}

/* ================= 页内搜索弹层 ================= */
.search-card {
  width: min(460px, 92vw);
}
.search-nav {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}
.search-nav .pop-btn {
  flex: 1;
}

/* ================= 排版设置 ================= */
.cfg-tabs {
  display: flex;
  gap: 6px;
  margin-top: 18px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--bg);
}
.cfg-tab {
  flex: 1;
  height: 28px;
  border: none;
  border-radius: 999px;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    background 0.2s ease;
}
.cfg-tab.active {
  color: var(--accent);
  background: var(--accent-soft);
}
.set-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 20px;
}
.set-label {
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-2);
}
.set-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}
.quick-keys-row {
  align-items: flex-start;
}
.quick-keys-box {
  flex: 1;
  max-width: 330px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-end;
}
.quick-keys-input {
  box-sizing: border-box;
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--text-1);
  font: 11px/1.5 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  resize: vertical;
  outline: none;
}
.quick-keys-input:focus {
  border-color: var(--accent);
}
.quick-keys-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.quick-keys-hint {
  width: 100%;
  margin: 0;
  font-size: 10.5px;
  font-weight: 300;
  line-height: 1.5;
  color: var(--text-3);
  word-break: break-all;
}
.set-btn {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.set-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.set-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
/* 极简开关（替换规则） */
.switch {
  position: relative;
  flex-shrink: 0;
  width: 36px;
  height: 20px;
  border-radius: 999px;
  border: 1px solid var(--border-strong);
  background: none;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.switch .switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--text-3);
  transition:
    transform 0.2s ease,
    background-color 0.2s ease;
}
.switch:hover {
  border-color: var(--accent);
}
.switch.on {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.switch.on .switch-knob {
  transform: translateX(16px);
  background: var(--accent);
}
.manage-link {
  padding: 3px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.manage-link:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.set-value {
  min-width: 34px;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-1);
  text-align: center;
  font-variant-numeric: tabular-nums;
}
.seg {
  display: flex;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.seg-btn {
  height: 28px;
  padding: 0 14px;
  border: none;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    background 0.2s ease;
}
.seg-btn + .seg-btn {
  border-left: 1px solid var(--border);
}
.seg-btn.active {
  color: var(--accent);
  background: var(--accent-soft);
}
.seg-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.set-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}
.text-btn {
  padding: 0;
  border: none;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.text-btn:hover {
  color: var(--text-1);
}
.text-btn.danger {
  color: var(--text-3);
}
.text-btn.danger:hover {
  color: #cf4444;
}

/* ================= 章节侧栏 ================= */
.drawer-mask {
  position: fixed;
  inset: 0;
  z-index: 50;
  background: rgba(24, 24, 27, 0.32);
}
.chapter-drawer {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: min(320px, 86vw);
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border-left: 1px solid var(--border);
}
.drawer-enter-active,
.drawer-leave-active {
  transition: opacity 0.2s ease;
}
.drawer-enter-active .chapter-drawer,
.drawer-leave-active .chapter-drawer {
  transition: transform 0.2s ease;
}
.drawer-enter-from,
.drawer-leave-to {
  opacity: 0;
}
.drawer-enter-from .chapter-drawer,
.drawer-leave-to .chapter-drawer {
  transform: translateX(100%);
}

.drawer-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}
.drawer-title {
  flex-shrink: 0;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--text-1);
}
.drawer-tools {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0 10px;
}
.drawer-search {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  outline: none;
  transition: border-color 0.2s ease;
}
.drawer-search:focus {
  border-color: var(--accent);
}
.drawer-search::placeholder {
  color: var(--text-3);
}
.drawer-reverse {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.drawer-reverse:hover,
.drawer-reverse.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.drawer-reverse svg {
  width: 13px;
  height: 13px;
}
.drawer-close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition: color 0.2s ease;
}
.drawer-close:hover {
  color: var(--text-1);
}
.drawer-close svg {
  width: 13px;
  height: 13px;
}

.drawer-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px 0 24px;
}
.chapter-volume {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 20px 20px 8px;
  border: none;
  background: none;
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
  text-align: left;
  cursor: pointer;
  transition: color 0.15s ease;
}
.chapter-volume:hover {
  color: var(--accent);
}
.vol-arrow {
  flex-shrink: 0;
  font-size: 10px;
  letter-spacing: 0;
  transition: transform 0.15s ease;
}
.vol-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chapter-item {
  display: flex;
  align-items: baseline;
  gap: 8px;
  width: 100%;
  padding: 11px 20px;
  border: none;
  border-left: 2px solid transparent;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  text-align: left;
  cursor: pointer;
  transition:
    color 0.2s ease,
    background 0.2s ease,
    border-color 0.2s ease;
}
.chapter-item-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
/* 章节字数（目录抽屉）：后端返回或本会话已加载章显示，未加载章省略 */
.chapter-item-wc {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 0.5px;
  font-variant-numeric: tabular-nums;
  color: var(--text-3);
}
.chapter-item:hover {
  color: var(--text-1);
  background: var(--hover);
}
.chapter-item.current {
  color: var(--accent);
  border-left-color: var(--accent);
  background: var(--accent-soft);
  font-weight: 400;
}
.chapter-item.current .chapter-item-wc {
  color: var(--accent);
}

/* ================= 听书面板 ================= */
.tts-card {
  width: min(320px, 86vw);
}
.tts-select {
  width: 150px;
  height: 30px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.tts-select:focus {
  border-color: var(--accent);
}
.tts-controls {
  display: flex;
  gap: 10px;
  margin-top: 22px;
}
.tts-play {
  flex: 1;
}
.tts-stop {
  flex-shrink: 0;
  height: 36px;
  padding: 0 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.tts-stop:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.tts-stop:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

/* ================= 书签弹层 ================= */
.bookmark-card {
  width: min(440px, 92vw);
}
.pop-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.pop-head .pop-title {
  margin: 0;
}
.pop-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.pop-btn {
  height: 26px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease, border-color 0.2s ease;
}
.pop-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.pop-btn.danger:hover:not(:disabled) {
  color: #cf4444;
  border-color: #cf4444;
}
.pop-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.bm-list {
  list-style: none;
  margin: 16px 0 0;
  padding: 0;
  max-height: 46vh;
  overflow-y: auto;
}
.bm-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 2px;
  border-bottom: 1px solid var(--border);
}
.bm-item:last-child {
  border-bottom: none;
}
.bm-check {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}
.bm-jump {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 0;
  border: none;
  background: none;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
}
.bm-chapter {
  max-width: 100%;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bm-text {
  max-width: 100%;
  font-size: 13px;
  font-weight: 300;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bm-quote,
.bm-note {
  max-width: 100%;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.45;
  color: var(--text-2);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.bm-note {
  color: var(--accent);
}
.bm-time {
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.chapter-item-cached {
  flex-shrink: 0;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface);
  font-size: 10px;
  font-weight: 300;
  letter-spacing: 0.5px;
  color: var(--text-3);
}
.chapter-item.current .chapter-item-cached {
  border-color: var(--accent);
  color: var(--accent);
}
.bm-edit {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition: color 0.2s ease;
}
.bm-edit:hover {
  color: var(--accent);
}
.bm-edit svg {
  width: 12px;
  height: 12px;
}
.bm-del {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition: color 0.2s ease;
}
.bm-del:hover {
  color: #cf4444;
}
.bm-del svg {
  width: 12px;
  height: 12px;
}

/* 书签编辑弹窗 */
.bookmark-edit-card {
  width: min(380px, 90vw);
}
.edit-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 12px;
}
.edit-field > span {
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2);
}
.edit-field input,
.edit-field textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  line-height: 1.5;
  outline: none;
  resize: vertical;
  transition: border-color 0.2s ease;
}
.edit-field input:focus,
.edit-field textarea:focus {
  border-color: var(--accent);
}
.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 16px;
}
.primary-btn {
  height: 34px;
  padding: 0 18px;
  border: 1px solid var(--accent);
  border-radius: var(--radius);
  background: var(--accent);
  color: #fff;
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 2px;
  cursor: pointer;
  transition: opacity 0.2s ease;
}
.primary-btn:hover:not(:disabled) {
  opacity: 0.88;
}
.primary-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* ================= 划词工具条 ================= */
.sel-bar {
  position: fixed;
  z-index: 60;
  display: flex;
  transform: translateX(-50%);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: 0 4px 18px rgba(0, 0, 0, 0.1);
}
.sel-btn {
  padding: 6px 14px;
  border: none;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.sel-btn + .sel-btn {
  border-left: 1px solid var(--border);
}
.sel-btn:hover {
  color: var(--accent);
}

/* ================= 翻页模式：仿真翻页（横向分页）/ 切章过渡 / 浮动按钮 ================= */
/* 仿真翻页：阅读页锁高，正文区弹性撑满，禁止整页纵向滚动 */
.reader-page.flip-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.reader-page.flip-layout .reader-main {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding-bottom: 32px;
}
.reader-page.flip-layout .chapter-title,
.reader-page.flip-layout .chapter-nav {
  flex-shrink: 0;
}
.flip-view {
  width: 100%;
}
.flip-view.active {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.flip-view.active .reader-content {
  height: 100%;
  column-fill: auto;
}
.flip-view.active .reader-img {
  break-inside: avoid;
}
/* hslide 切章过渡：正文随方向滑入（翻页动画） */
.chapter-slide-in-right {
  animation: chapter-slide-in-right var(--chapter-anim-duration, 0.32s) ease both;
}
.chapter-slide-in-left {
  animation: chapter-slide-in-left var(--chapter-anim-duration, 0.32s) ease both;
}
@keyframes chapter-slide-in-right {
  from {
    opacity: 0.25;
    transform: translateX(48px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}
@keyframes chapter-slide-in-left {
  from {
    opacity: 0.25;
    transform: translateX(-48px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}
/* 翻页模式浮动按钮 */
.flip-nav-vert {
  position: fixed;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 15;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.flip-nav-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface);
  color: var(--text-2);
  font-family: inherit;
  font-size: 15px;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.14);
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    opacity 0.2s ease;
}
.flip-nav-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.flip-nav-btn:disabled {
  opacity: 0.35;
  cursor: default;
}
.flip-nav-side {
  position: fixed;
  top: 50%;
  transform: translateY(-50%);
  z-index: 15;
}
.flip-nav-side.prev {
  left: 8px;
}
.flip-nav-side.next {
  right: 8px;
}
/* 翻页模式 4 按钮塞入 320px 弹层：收紧内边距 */
.seg.compact .seg-btn {
  padding: 0 9px;
  font-size: 11.5px;
  letter-spacing: 0.5px;
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    flex-wrap: wrap;
    padding: 8px 10px;
    gap: 8px;
  }
  /* 竖屏阅读：正文留白收窄，底部进度条不遮挡正文 */
  .reader-main {
    padding: 26px 14px 118px;
  }
  .icon-btn {
    width: 32px;
    height: 32px;
  }
  .dlg-overlay {
    padding: 14px;
  }
  .dlg-actions {
    flex-wrap: wrap;
  }
  .chapter-drawer {
    width: min(340px, 92vw);
  }
  .drawer-head {
    flex-wrap: wrap;
    gap: 8px;
  }
  .drawer-tools {
    order: 2;
    width: 100%;
    margin: 0;
  }
  .set-foot {
    flex-wrap: wrap;
    gap: 8px;
  }
  .book-name {
    order: 2;
    flex: 1;
    font-size: 13px;
  }
  /* 顶部操作栏：第二行整宽，横向滚动（触屏滑动），按钮不压缩 */
  .top-actions {
    order: 3;
    width: 100%;
    gap: 6px;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    padding-bottom: 2px;
  }
  .top-actions::-webkit-scrollbar {
    display: none;
  }
  .top-actions .font-btn,
  .top-actions .toc-btn {
    flex-shrink: 0;
    min-width: 32px;
  }
  .font-btn {
    padding: 0 8px;
  }
  .toc-btn {
    padding: 0 10px;
    margin-left: 0;
  }
  .reader-main {
    padding: 32px 16px 130px;
  }
  .chapter-nav {
    gap: 12px;
  }
  .progress-bar {
    padding: 0 12px 10px;
    padding-bottom: max(10px, env(safe-area-inset-bottom));
  }
}
</style>
