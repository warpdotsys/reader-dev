<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getBookshelf, saveBook } from '@/api/bookshelf'
import { getBookInfo, getBookToc, searchBookSource, searchBookSourceSSE } from '@/api/books'
import { getInvalidBookSources } from '@/api/sources'
import { deleteBookCache, getShelfBookWithCacheInfo, searchBookContent } from '@/api/cache'
import { exportBook, type ExportEncoding, type ExportFormat } from '@/api/export'
import { hanText, syncHanMode } from '@/utils/hanMode'
import { proxyImageUrl } from '@/utils/imageProxy'
import { uploadFile, mkdir } from '@/api/file'
import { post } from '@/api/request'
import { downloadBlob } from '@/utils/download'
import { relocateChapterIndex } from '@/utils/progressRelocate'
import { buildTocEntries } from '@/utils/tocPreview'
import { clearLocalBook } from '@/utils/readerLocalCache'
import ChapterCacheDialog from '@/components/ChapterCacheDialog.vue'
import { useUserStore } from '@/stores/user'
import { isNotImplemented } from '@/utils/errors'
import type { Book, BookChapter, BookInfo, ContentSearchHit, SearchBook } from '@/types'

const route = useRoute()
const router = useRouter()
const store = useUserStore()

/** /book/:url —— vue-router 已自动解码 */
const bookUrl = computed(() => String(route.params.url ?? ''))

/** 非书架书的书源信息：入口（搜索结果等）通过 query 传入 */
const queryOrigin = computed(() => String(route.query.origin ?? ''))
const queryOriginName = computed(() => String(route.query.originName ?? ''))

const shelfBook = ref<Book | null>(null)
const info = ref<BookInfo | null>(null)
const loading = ref(true)
const loadFailed = ref(false)
const errorMsg = ref('')
const coverFailed = ref(false)
const saving = ref(false)

/* ================= GAP 82：单书缓存信息（GET /reader3/getShelfBookWithCacheInfo，silent——后端已有；未实现降级隐藏） ================= */

const shelfCache = ref<{ chapterCount: number; size: number } | null>(null)
/** 接口不可用（404/网络失败）：永久隐藏状态区，不再重复探测 */
const shelfCacheHidden = ref(false)

function fmtCacheSize(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return '0 B'
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

async function loadShelfCacheInfo() {
  if (shelfCacheHidden.value) return
  try {
    const res = await getShelfBookWithCacheInfo(bookUrl.value)
    const d = res.data
    if (res.isSuccess && d && typeof d.cacheChapterCount === 'number') {
      shelfCache.value = {
        chapterCount: d.cacheChapterCount,
        size: Number(d.cacheSize ?? 0),
      }
    } else {
      shelfCache.value = null
    }
  } catch {
    // 接口未实现（404）：静默隐藏状态区
    shelfCache.value = null
    shelfCacheHidden.value = true
  }
}

/** 展示数据：实时详情优先，书架数据兜底；自定义封面（GAP 19）最优先；自定义简介（GAP 145）优先于书源解析 intro */
const display = computed(() => ({
  name: info.value?.name || shelfBook.value?.name || '未知书名',
  author: info.value?.author || shelfBook.value?.author || '',
  cover:
    proxyImageUrl(shelfBook.value?.customCoverUrl || info.value?.coverUrl || shelfBook.value?.coverUrl) || '',
  intro: shelfBook.value?.customIntro || info.value?.intro || shelfBook.value?.intro || '',
  latestChapterTitle:
    info.value?.latestChapterTitle || shelfBook.value?.latestChapterTitle || '',
}))

/** 标签（GAP 145：customTag 逗号分隔 → 展示 chips） */
const displayTags = computed<string[]>(() => {
  const raw = shelfBook.value?.customTag
  if (typeof raw !== 'string' || !raw.trim()) return []
  return raw
    .split(/[,，]/)
    .map((s) => s.trim())
    .filter(Boolean)
})

/** 自定义封面走 file/download 内联流：展示时补当前 accessToken（重新登录后仍可显示） */
function resolveCoverUrl(url: string): string {
  if (!url.startsWith('/reader3/file/')) return url
  const token = store.accessToken
  if (!token || url.includes('accessToken=')) return url
  return `${url}${url.includes('?') ? '&' : '?'}accessToken=${encodeURIComponent(token)}`
}

function coverInitial(name: string): string {
  const ch = name.trim().charAt(0)
  return ch ? ch.toUpperCase() : '书'
}

/** 本地书（local:// 或文件型 .txt）：后端 local 分支直查书架，不依赖书源 */
function isLocalBookUrl(url: string): boolean {
  return url.startsWith('local://') || url.endsWith('.txt')
}

async function load() {
  loading.value = true
  loadFailed.value = false
  errorMsg.value = ''
  info.value = null
  // 目录预览缓存随书重置（换源/重进详情后重新拉取）
  tocLoaded.value = false
  tocChapters.value = []
  activeTab.value = 'detail'
  try {
    // ① 先查书架定位本书
    const res = await getBookshelf()
    const found = (res.data ?? []).find((b) => b.bookUrl === bookUrl.value) ?? null
    shelfBook.value = found

    if (found?.origin) {
      // ② 书架书：详情接口 bookSource=book.origin，实时详情优先，失败用书架数据兜底
      try {
        const infoRes = await getBookInfo(bookUrl.value, found.origin)
        if (infoRes.isSuccess) info.value = infoRes.data
      } catch {
        // 实时详情失败：用书架数据兜底展示
      }
    } else if (isLocalBookUrl(bookUrl.value)) {
      // ③ 本地书：后端 local 分支直查书架返回（无需 bookSource；不在书架则报错）
      try {
        const infoRes = await getBookInfo(bookUrl.value, '')
        if (infoRes.isSuccess) info.value = infoRes.data
      } catch (err) {
        loadFailed.value = true
        errorMsg.value = err instanceof Error ? err.message : '未找到这本书（可能不在书架中）'
      }
    } else if (queryOrigin.value) {
      // ④ 非书架书：直接调详情接口（后端已支持非书架书，bookSource=入口传入的 origin）
      try {
        const infoRes = await getBookInfo(bookUrl.value, queryOrigin.value)
        if (infoRes.isSuccess) info.value = infoRes.data
      } catch (err) {
        loadFailed.value = true
        errorMsg.value = err instanceof Error ? err.message : '获取详情失败'
      }
    } else {
      // ⑤ 非书架书且无书源信息：无法获取详情
      loadFailed.value = true
      errorMsg.value = '未找到这本书（可能不在书架中）'
    }
  } catch {
    loadFailed.value = true
    errorMsg.value = '书架拉取失败，请稍后重试'
  } finally {
    loading.value = false
  }
  // GAP 82：书架书 → 拉取单书缓存状态（silent；未实现隐藏）
  if (shelfBook.value) void loadShelfCacheInfo()
}

/** 由详情信息组装完整 Book JSON（saveBook 入架 body：type/group 用默认值 0） */
function buildShelfBook(): Book {
  const i = info.value
  return {
    bookUrl: i?.bookUrl || bookUrl.value,
    tocUrl: i?.tocUrl || '',
    origin: i?.origin || queryOrigin.value,
    originName: i?.originName || queryOriginName.value,
    name: i?.name || '',
    author: i?.author || '',
    kind: i?.kind ?? null,
    coverUrl: i?.coverUrl ?? null,
    intro: i?.intro ?? null,
    charset: null,
    // 非文本书（音频/漫画等）：getBookInfo 返回的 type 透传入架——阅读器按此分派
    type: typeof i?.type === 'number' && i.type >= 0 && i.type <= 4 ? i.type : 0,
    group: 0,
    latestChapterTitle: i?.latestChapterTitle ?? null,
    latestChapterTime: 0,
  }
}

/**
 * GAP 108：入架前同书多源去重（getBookshelf 检查）。
 * ① 同 bookUrl 已在书架 → 提示「已在书架」（并同步本地书架态）；
 * ② 同名不同源（bookUrl 不同）→ 弹窗「已有同名书（其他源）——保留 / 仍加入」；
 * ③ 书架拉取失败不拦截（saveBook 自身会提示错误）。
 */
async function checkDupBeforeAdd(book: { bookUrl: string; name: string }): Promise<'ok' | 'exists' | 'same-name-cancel'> {
  let list: Book[] = []
  try {
    const res = await getBookshelf()
    list = res.data ?? []
  } catch {
    return 'ok' // 书架拉取失败：不拦截
  }
  const sameUrl = list.find((b) => b.bookUrl === book.bookUrl)
  if (sameUrl) {
    if (sameUrl.bookUrl === bookUrl.value) shelfBook.value = sameUrl
    return 'exists'
  }
  const name = (book.name || '').trim()
  const sameName = name ? list.find((b) => (b.name || '').trim() === name) : undefined
  if (sameName) {
    try {
      await ElMessageBox.confirm(
        `书架已有同名书《${sameName.name}》（${sameName.originName || sameName.origin || '其他书源'}）。仍要加入当前这本书吗？`,
        '已有同名书（其他源）',
        {
          confirmButtonText: '仍加入',
          cancelButtonText: '保留',
          type: 'warning',
        },
      )
    } catch {
      return 'same-name-cancel' // 用户选择「保留」
    }
  }
  return 'ok'
}

/** 加入书架（非书架书）：先查重（GAP 108）→ POST /reader3/saveBook，成功即视为书架书 */
async function addToShelf() {
  if (saving.value || !info.value) {
    if (!info.value) ElMessage.warning('书籍详情尚未加载完成，请稍后重试')
    return
  }
  const dup = await checkDupBeforeAdd({ bookUrl: bookUrl.value, name: info.value.name || '' })
  if (dup === 'exists') {
    ElMessage.info('已在书架')
    return
  }
  if (dup === 'same-name-cancel') return
  saving.value = true
  try {
    await saveBook(buildShelfBook())
    shelfBook.value = buildShelfBook()
    ElMessage.success('已加入书架')
  } catch {
    // 失败提示由 request.ts 统一 toast，按钮保持「加入书架」
  } finally {
    saving.value = false
  }
}

function startReading() {
  if (!shelfBook.value) return
  void router.push(`/reader/${encodeURIComponent(shelfBook.value.bookUrl)}`)
}

/** GAP 157：续读进度——durChapterIndex>0 时「开始阅读」显示「续读 第 N 章」（N=章节号 1 起） */
const resumeLabel = computed(() => {
  const idx = shelfBook.value?.durChapterIndex
  if (typeof idx === 'number' && idx > 0) return `续读 第 ${idx + 1} 章`
  return '开始阅读'
})

/** 阅读进度环（书架数据 durChapterIndex / totalChapterNum；未入架或缺数据（0）时隐藏） */
const readProgress = computed<{ percent: number; cur: number; total: number } | null>(() => {
  const b = shelfBook.value
  const total = b?.totalChapterNum
  const cur = b?.durChapterIndex
  if (typeof total !== 'number' || total <= 0) return null
  if (typeof cur !== 'number' || cur <= 0) return null
  return {
    percent: Math.min(100, Math.round((cur / total) * 100)),
    cur,
    total,
  }
})

/** 非书架书直接阅读（不加入书架——退出时阅读器提醒入架） */
function startReadingTemp() {
  const b = shelfBook.value ?? info.value
  if (!b) return
  const cover =
    typeof b.customCoverUrl === 'string' && b.customCoverUrl
      ? b.customCoverUrl
      : b.coverUrl || ''
  const q = new URLSearchParams({
    source: b.origin || '',
    sourceName: b.originName || '',
    toc: b.tocUrl || b.bookUrl || '',
    name: b.name || '',
    author: b.author || '',
    cover,
    // 非文本书临时直读：阅读器按 type 分派渲染（0 文本/1 音频/2 漫画/3 文件/4 视频）
    type: String(typeof b.type === 'number' && b.type >= 0 && b.type <= 4 ? b.type : 0),
  })
  void router.push(`/reader/${encodeURIComponent(b.bookUrl)}?${q.toString()}`)
}

/* ================= GAP 19：自定义封面（图片上传到 __HOME__/covers → saveBook customCoverUrl → 展示） ================= */

const coverInputRef = ref<HTMLInputElement | null>(null)
const coverBusy = ref(false)
const COVER_MAX_MB = 10

/** 上传的封面经 file/download（stream=1 内联）展示，URL 存 customCoverUrl */
function coverDownloadUrl(name: string): string {
  const base = `/reader3/file/download?path=covers/${encodeURIComponent(name)}&home=__HOME__&stream=1`
  return store.accessToken
    ? `${base}&accessToken=${encodeURIComponent(store.accessToken)}`
    : base
}

function openCoverPicker() {
  if (coverBusy.value || !shelfBook.value) return
  coverInputRef.value?.click()
}

async function onCoverPick(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = '' // 允许重复选择同一文件
  if (!file || coverBusy.value || !shelfBook.value) return
  if (!file.type.startsWith('image/')) {
    ElMessage.warning('请选择图片文件')
    return
  }
  if (file.size > COVER_MAX_MB * 1024 * 1024) {
    ElMessage.warning(`图片不能超过 ${COVER_MAX_MB} MB`)
    return
  }
  coverBusy.value = true
  try {
    // ① 确保 __HOME__/covers 目录存在（后端 file/upload 要求目标目录已存在；已存在则忽略）
    try {
      await mkdir('', 'covers', '__HOME__', { silent: true })
    } catch {
      /* 目录已存在 / 后端未实现：继续上传 */
    }
    // ② 上传到 __HOME__/covers（后端 file/upload 已实现；未就绪时 404 由请求层提示）
    const ext = (file.name.match(/\.(png|jpe?g|gif|webp|bmp|svg)$/i)?.[0] || '.jpg').toLowerCase()
    const name = `cover-${Date.now().toString(36)}${ext}`
    await uploadFile(file, 'covers', '__HOME__')
    // ③ saveBook 更新 customCoverUrl（存在即增量 patch 该字段）
    await saveBook({ bookUrl: shelfBook.value.bookUrl, customCoverUrl: coverDownloadUrl(name) } as Book)
    // ③ 本地同步 + 刷新封面展示
    shelfBook.value.customCoverUrl = coverDownloadUrl(name)
    coverFailed.value = false
    ElMessage.success('自定义封面已更新')
  } catch (err) {
    ElMessage.error(err instanceof Error ? err.message : '封面上传失败（请确认后端 file/upload 可用）')
  } finally {
    coverBusy.value = false
  }
}

/* ================= GAP 18：目录预览（getBookToc → 前 50 章 → 点击进阅读器跳章） ================= */

const activeTab = ref<'detail' | 'toc'>('detail')
const tocChapters = ref<BookChapter[]>([])
const tocLoading = ref(false)
const tocLoaded = ref(false)
const tocError = ref(false)
const TOC_PREVIEW_MAX = 50

/** 目录数据源：tocUrl 取实时详情/书架，缺省用 bookUrl 兜底（与阅读页一致）；origin 同上 */
function tocParams(): { tocUrl: string; origin: string } | null {
  const origin = shelfBook.value?.origin || info.value?.origin || queryOrigin.value
  if (!origin) return null
  const tocUrl = info.value?.tocUrl || shelfBook.value?.tocUrl || bookUrl.value
  return { tocUrl, origin }
}

async function openToc() {
  activeTab.value = 'toc'
  if (tocLoaded.value) return
  const p = tocParams()
  if (!p) {
    tocError.value = true
    return
  }
  tocLoading.value = true
  tocError.value = false
  try {
    const res = await getBookToc(p.tocUrl, p.origin)
    tocChapters.value = res.data ?? []
    tocLoaded.value = true
    if (tocChapters.value.length === 0) tocError.value = true
  } catch {
    tocError.value = true
  } finally {
    tocLoading.value = false
  }
}

/** 前 50 章（跳过卷标题行；index 为完整目录数组下标，与阅读页 ?chapter 语义一致） */
const tocPreview = computed(() => {
  const out: { index: number; title: string }[] = []
  tocChapters.value.forEach((c, i) => {
    if (out.length >= TOC_PREVIEW_MAX) return
    if (c.isVolume) return
    out.push({ index: i, title: c.title })
  })
  return out
})

/** 目录预览渲染条目（GAP 91：卷标题行 isVolume 渲染分隔行；GAP 147：当前章高亮 durChapterIndex；简繁按全站模式转换）
 *  P2：前 50 章截断后不再追加任何行（含分卷标题——分卷标题无限追加修复，逻辑见 utils/tocPreview.ts） */
const tocEntries = computed<{ kind: 'volume' | 'chapter'; index: number; title: string }[]>(
  () => buildTocEntries(tocChapters.value, TOC_PREVIEW_MAX, hanText),
)

/** GAP 147：书架进度章（durChapterIndex）——目录 tab 当前章高亮 */
const currentChapterIndex = computed(() =>
  typeof shelfBook.value?.durChapterIndex === 'number' && shelfBook.value.durChapterIndex >= 0
    ? shelfBook.value.durChapterIndex
    : -1,
)

/** 点击目录项 → 阅读器并跳章 */
function goToChapterFromToc(idx: number) {
  void router.push(`/reader/${encodeURIComponent(bookUrl.value)}?chapter=${idx}`)
}

/* ================= 全书搜索（GET /reader3/searchBookContent，本地书正文逐章匹配） ================= */

const searchOpen = ref(false)
const searchKey = ref('')
const searchBusy = ref(false)
const searchHits = ref<ContentSearchHit[]>([])
const searchMsg = ref('')
const searchMsgError = ref(false)

function openSearch() {
  searchKey.value = ''
  searchHits.value = []
  searchMsg.value = ''
  searchMsgError.value = false
  searchOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeSearch() {
  if (searchBusy.value) return
  searchOpen.value = false
  document.body.style.overflow = ''
}

async function runSearch() {
  if (searchBusy.value) return
  const key = searchKey.value.trim()
  if (!key) {
    searchMsg.value = '请输入搜索关键词'
    searchMsgError.value = true
    return
  }
  searchBusy.value = true
  searchMsg.value = ''
  searchMsgError.value = false
  searchHits.value = []
  try {
    const res = await searchBookContent(key, bookUrl.value)
    const hits = res.data ?? []
    searchHits.value = hits
    if (hits.length === 0) {
      searchMsg.value = '未找到匹配内容'
      searchMsgError.value = false
    } else {
      searchMsg.value = `共 ${hits.length} 个章节命中`
    }
  } catch (err) {
    // 接口未实现（404）/失败：在弹层内提示，不弹全局 toast
    searchMsg.value =
      err instanceof Error && err.message
        ? `搜索失败：${err.message}`
        : '搜索失败，请稍后重试'
    searchMsgError.value = true
  } finally {
    searchBusy.value = false
  }
}

/** 点击命中 → 跳阅读页并定位到该章（/reader/:bookUrl?chapter=index） */
function goToHit(hit: ContentSearchHit) {
  closeSearch()
  void router.push(`/reader/${encodeURIComponent(bookUrl.value)}?chapter=${hit.chapterIndex}`)
}

/* ================= 换源（GET /reader3/searchBookSource：搜索同书其他书源，点击切换） ================= */

const sourceOpen = ref(false)
const sourceBusy = ref(false)
const sourceSwitching = ref(false)
const sourceResults = ref<SearchBook[]>([])
const sourceKeyword = ref('')
const sourceFiltered = computed(() => {
  const kw = sourceKeyword.value.trim().toLowerCase()
  if (!kw) return sourceResults.value
  return sourceResults.value.filter(
    (r) =>
      (r.originName || '').toLowerCase().includes(kw) ||
      (r.origin || '').toLowerCase().includes(kw),
  )
})
function refreshSource() {
  if (sourceBusy.value) return
  sourceSSEHandle?.abort()
  sourceResults.value = []
  void runSourceSearch()
}
const sourceMsg = ref('')
const sourceMsgError = ref(false)
const currentOrigin = ref('')
/** GAP 81：SSE 已返回结果的书源数（流式进度展示） */
const sourceDoneCount = ref(0)
/** 当前换源 SSE 句柄（关闭弹层时中止） */
let sourceSSEHandle: { abort: () => void } | null = null
/** 失效书源 URL 集合（GAP 20：getInvalidBookSources 探测，失败静默 → 空集不标注） */
const invalidSourceUrls = ref<Set<string>>(new Set())

/** 书架书且有书源才可换源（本地书无 origin 不显示入口） */
function canSwitchSource(): boolean {
  return !!shelfBook.value && !!shelfBook.value.origin
}

function openSource() {
  sourceOpen.value = true
  document.body.style.overflow = 'hidden'
  void runSourceSearch()
}

function closeSource() {
  if (sourceBusy.value || sourceSwitching.value) return
  sourceSSEHandle?.abort()
  sourceSSEHandle = null
  sourceOpen.value = false
  document.body.style.overflow = ''
}

/** GAP 20：结果排序——当前源置顶 → 其余按 originName 升序（失效源沉底，带「失效」标注） */
function sortSourceResults(list: SearchBook[]): SearchBook[] {
  const cur = list.filter((r) => r.origin === currentOrigin.value)
  const rest = list.filter((r) => r.origin !== currentOrigin.value)
  const invalid = rest.filter((r) => invalidSourceUrls.value.has(r.origin))
  const valid = rest.filter((r) => !invalidSourceUrls.value.has(r.origin))
  const byName = (a: SearchBook, c: SearchBook) =>
    (a.originName || a.origin || '').localeCompare(c.originName || c.origin || '')
  return [...cur, ...valid.sort(byName), ...invalid.sort(byName)]
}

/** 按书源去重后追加（同一源多条结果只留首个） */
function appendSourceResults(books: SearchBook[]) {
  const seen = new Set(sourceResults.value.map((r) => r.origin || r.originName))
  for (const r of books) {
    const k = r.origin || r.originName
    if (!k || seen.has(k)) continue
    seen.add(k)
    sourceResults.value = [...sourceResults.value, r]
  }
}

/** 流结束收尾：排序 + 空态文案 */
function finalizeSourceResults() {
  sourceResults.value = sortSourceResults(sourceResults.value)
  if (sourceResults.value.length === 0) {
    sourceMsg.value = '未找到其他书源'
    sourceMsgError.value = false
  }
}

/** 搜索同书其他书源：url=当前书 bookUrl + bookSource=当前源 */
async function runSourceSearch() {
  const b = shelfBook.value
  if (!b || !b.origin) return
  sourceBusy.value = true
  sourceResults.value = []
  sourceMsg.value = ''
  sourceMsgError.value = false
  sourceDoneCount.value = 0
  currentOrigin.value = b.origin
  invalidSourceUrls.value = new Set()
  try {
    // 失效书源探测（后端并行实现中，可能 404：silent 降级 → 不标注）
    try {
      const inv = await getInvalidBookSources()
      invalidSourceUrls.value = new Set(
        (inv.data ?? []).map((x) => (typeof x === 'string' ? x : x.bookSourceUrl)),
      )
    } catch {
      invalidSourceUrls.value = new Set()
    }
    // GAP 81：优先 SSE 流式换源（逐书源增量推送；连接失败降级普通接口）
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
          // 流中途断开：保留已收结果（若一个都没到则走降级提示）
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
      // 降级：普通 searchBookSource 全量拉取
      const res = await searchBookSource(b.bookUrl, b.origin, { silent: true })
      appendSourceResults(res.data ?? [])
      finalizeSourceResults()
      sourceBusy.value = false
    }
  } catch (err) {
    sourceMsg.value = isNotImplemented(err)
      ? '换源搜索接口后端暂未提供（GET /reader3/searchBookSource）'
      : `换源搜索失败：${err instanceof Error ? err.message : '请稍后重试'}`
    sourceMsgError.value = true
    sourceBusy.value = false
  } finally {
    sourceBusy.value = false
  }
}

/** 点击结果 → 切换书源：saveBook 更新 origin/originName/tocUrl（bookUrl 保持书架主键不变） */
async function switchSource(r: SearchBook) {
  const b = shelfBook.value
  if (!b || sourceSwitching.value) return
  if (!r.origin || r.origin === currentOrigin.value) return
  sourceSwitching.value = true
  try {
    await saveBook({
      bookUrl: b.bookUrl,
      origin: r.origin,
      originName: r.originName,
      tocUrl: r.tocUrl,
    } as Book)
    // 本地同步书架条目 + 用新源刷新详情
    b.origin = r.origin
    b.originName = r.originName
    b.tocUrl = r.tocUrl
    currentOrigin.value = r.origin
    info.value = null
    tocLoaded.value = false
    tocChapters.value = []
    try {
      const infoRes = await getBookInfo(bookUrl.value, r.origin)
      if (infoRes.isSuccess) info.value = infoRes.data
    } catch {
      // 详情刷新失败：书架数据兜底展示
    }
    // GAP 6：换源后阅读位置保留——新源章节目录索引可能失效，拉新目录重定位 durChapterIndex
    // （标题跨源匹配优先，越界就近钳制；重定位后写回服务端进度，阅读器续读不丢位置）
    await relocateProgressAfterSwitch(r)
    ElMessage.success(`已切换到「${r.originName || r.origin}」`)
    closeSource()
  } catch {
    // 失败提示由 request.ts 统一 toast（saveBook 非 silent）
  } finally {
    sourceSwitching.value = false
  }
}

/**
 * GAP 6：换源成功后重定位阅读进度。
 * 旧源 durChapterIndex 在新源目录中可能越界/指向卷标行——拉新目录后
 * 用 relocateChapterIndex（标题匹配 → 就近钳制）算新索引，并 POST saveBookProgress
 * 写回服务端（失败静默——阅读器内还有范围守卫兜底）。
 */
async function relocateProgressAfterSwitch(r: SearchBook) {
  const b = shelfBook.value
  if (!b) return
  const oldIdx = typeof b.durChapterIndex === 'number' ? b.durChapterIndex : -1
  if (oldIdx < 0) return // 无进度不处理
  try {
    const tocRes = await getBookToc(r.tocUrl || b.tocUrl, r.origin)
    const toc = tocRes.isSuccess ? (tocRes.data ?? []) : []
    const newIdx = relocateChapterIndex(oldIdx, b.durChapterTitle, toc)
    if (newIdx < 0) return // 目录为空等异常：不动服务端进度
    const newTitle = toc[newIdx]?.title ?? b.durChapterTitle ?? ''
    b.durChapterIndex = newIdx
    b.durChapterTitle = newTitle
    await post('/saveBookProgress', {
      bookUrl: b.bookUrl,
      durChapterIndex: newIdx,
      durChapterPos: 0,
      durChapterTime: Date.now(),
      durChapterTitle: newTitle,
    }).catch(() => {
      /* 写回失败静默——阅读器内有范围守卫 */
    })
  } catch {
    // 新目录拉取失败：不动服务端进度（阅读器内范围守卫兜底）
  }
}

/* ================= 导出（GET /reader3/exportBook：txt/epub/html blob 下载 + txt 编码选择） ================= */

const EXPORT_FORMATS: { value: ExportFormat; label: string; tip: string }[] = [
  { value: 'txt', label: 'TXT', tip: '纯文本' },
  { value: 'epub', label: 'EPUB', tip: '电子书' },
  { value: 'html', label: 'HTML', tip: '网页' },
]

const exportOpen = ref(false)
const exportFormat = ref<ExportFormat>('txt')
/** GAP 144：txt 导出编码（UTF-8 / GBK——GBK 中文环境兼容；后端并行实现中，未就绪时仍输出 UTF-8） */
const exportEncoding = ref<ExportEncoding>('utf-8')
const exportBusy = ref(false)
const exportMsg = ref('')
const exportMsgError = ref(false)

function openExport() {
  exportFormat.value = 'txt'
  exportEncoding.value = 'utf-8'
  exportMsg.value = ''
  exportMsgError.value = false
  exportOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeExport() {
  if (exportBusy.value) return
  exportOpen.value = false
  document.body.style.overflow = ''
}

/** 导出并下载（失败在弹窗内提示，不弹全局 toast——接口可能未实现） */
async function confirmExport() {
  if (exportBusy.value) return
  exportBusy.value = true
  exportMsg.value = ''
  exportMsgError.value = false
  try {
    const { blob, warning } = await exportBook(
      bookUrl.value,
      exportFormat.value,
      exportEncoding.value,
    )
    // 后端错误体（HTTP 200 + JSON）：在此识别并展示（encoding 未就绪时后端忽略参数仍输出 UTF-8）
    if (blob.type.includes('application/json')) {
      try {
        const parsed = JSON.parse(await blob.text()) as { isSuccess?: boolean; errorMsg?: string }
        if (parsed && typeof parsed === 'object' && (parsed.isSuccess === false || parsed.errorMsg)) {
          exportMsg.value = parsed.errorMsg || '导出失败'
          exportMsgError.value = true
          return
        }
      } catch {
        /* 非错误 JSON：按文件正常下载 */
      }
    }
    const name = `${(display.value.name || 'book').replace(/[\\/:*?"<>|]/g, '_')}.${exportFormat.value}`
    const ok = await downloadBlob(blob, name)
    if (ok) {
      // P2：导出警告（并发抓章失败章节 / GBK 不可映射转义）——随下载成功提示展示
      const warnParts: string[] = []
      const failed = warning?.failedChapters?.length ?? 0
      if (failed > 0) warnParts.push(`${failed} 章抓取失败已跳过`)
      if (warning?.unmappableChars) warnParts.push(`GBK 无法编码 ${warning.unmappableChars} 个字符（已转义保留）`)
      exportMsg.value = warnParts.length
        ? `已下载 ${name}（警告：${warnParts.join('；')}）`
        : `已下载 ${name}`
      window.setTimeout(() => {
        if (!exportBusy.value) closeExport()
      }, 900)
    }
  } catch (err) {
    exportMsg.value = isNotImplemented(err)
      ? '导出接口后端暂未提供（GET /reader3/exportBook）'
      : `导出失败：${err instanceof Error ? err.message : '请稍后重试'}`
    exportMsgError.value = true
  } finally {
    exportBusy.value = false
  }
}

/* ================= 追更开关（legacy canUpdate：书架书保存开关，后端 F-35 更新任务按此刷新） ================= */
const updateBusy = ref(false)

async function toggleCanUpdate() {
  const b = shelfBook.value
  if (!b || updateBusy.value) return
  updateBusy.value = true
  const next = !b.canUpdate
  try {
    await saveBook({ bookUrl: b.bookUrl, canUpdate: next } as Book)
    b.canUpdate = next
    ElMessage.success(next ? '已开启追更' : '已关闭追更')
  } catch {
    // 失败提示已由拦截器统一处理
  } finally {
    updateBusy.value = false
  }
}

/* ================= GAP 145：元数据编辑（书名/作者/标签/简介弹窗表单——saveBook patch 字段，详情即时刷新） ================= */

const editOpen = ref(false)
const editBusy = ref(false)
const editMsg = ref('')
const editMsgError = ref(false)
const editForm = ref({ name: '', author: '', tags: '', intro: '' })

function openEdit() {
  const b = shelfBook.value
  if (!b) return
  editForm.value = {
    name: b.name || '',
    author: b.author || '',
    tags: typeof b.customTag === 'string' ? b.customTag : '',
    intro: typeof b.customIntro === 'string' ? b.customIntro : b.intro || '',
  }
  editMsg.value = ''
  editMsgError.value = false
  editOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeEdit() {
  if (editBusy.value) return
  editOpen.value = false
  document.body.style.overflow = ''
}

/** 保存：saveBook 对已存在书按 body 字段增量 patch（name/author/customTag/customIntro，后端 patch_book 已支持） */
async function confirmEdit() {
  const b = shelfBook.value
  if (!b || editBusy.value) return
  const name = editForm.value.name.trim()
  if (!name) {
    editMsg.value = '书名不能为空'
    editMsgError.value = true
    return
  }
  editBusy.value = true
  editMsg.value = ''
  try {
    await saveBook({
      bookUrl: b.bookUrl,
      name,
      author: editForm.value.author.trim(),
      customTag: editForm.value.tags.trim() || null,
      customIntro: editForm.value.intro.trim() || null,
    } as Book)
    // 本地同步 + 详情即时刷新（display.intro 优先 customIntro）
    b.name = name
    b.author = editForm.value.author.trim()
    b.customTag = editForm.value.tags.trim() || null
    b.customIntro = editForm.value.intro.trim() || null
    if (info.value) {
      info.value.name = name
      info.value.author = b.author
    }
    introExpanded.value = false
    introMeasured.value = false
    introLong.value = false
    ElMessage.success('已保存')
    closeEdit()
  } catch {
    // 失败提示由 request.ts 统一 toast（saveBook 非 silent）
  } finally {
    editBusy.value = false
  }
}

/* ================= 章节缓存（ChapterCacheDialog：服务器 / 本机双向，支持单章/至末尾/全本/范围） ================= */

const cacheOpen = ref(false)
const cacheScope = ref<'chapter' | 'rest' | 'all' | 'range'>('all')
const cacheFrom = ref(1)

/** 目录实章（过滤卷标题；后端 range 参数按此顺序 0 基） */
const realTocChapters = computed(() => tocChapters.value.filter((c) => !c.isVolume))

function openCacheDialog() {
  cacheScope.value = 'all'
  cacheFrom.value = 1
  cacheOpen.value = true
}

/** 目录页单章缓存：按目录原数组下标定位实章序号（1 基）后打开弹层 */
function cacheChapter(index: number) {
  const ch = tocChapters.value[index]
  if (!ch || ch.isVolume) return
  const idx = realTocChapters.value.findIndex((x) => x.url === ch.url)
  if (idx < 0) return
  cacheScope.value = 'chapter'
  cacheFrom.value = idx + 1
  cacheOpen.value = true
}

/** 缓存完成（服务器/本机）→ 刷新单书服务器缓存状态 */
function onCacheDone() {
  void loadShelfCacheInfo()
}

/* ================= GAP 79：清除本书缓存（POST /reader3/deleteBookCache） ================= */

const cacheClearBusy = ref(false)

/** 确认后删除单书缓存（正文/目录缓存——删后下次打开重新拉取；接口 404 时提示后端待实现） */
async function clearBookCache() {
  const b = shelfBook.value
  if (!b || cacheClearBusy.value) return
  try {
    await ElMessageBox.confirm('清除本书服务器与本机缓存后，正文/目录将重新从书源拉取。确定清除？', '清除缓存', {
      confirmButtonText: '清除',
      cancelButtonText: '取消',
      type: 'warning',
    })
  } catch {
    return // 用户取消
  }
  cacheClearBusy.value = true
  try {
    const res = await deleteBookCache(b.bookUrl)
    const deleted = typeof res.data?.deleted === 'number' ? res.data.deleted : 0
    const localDeleted = await clearLocalBook(b.bookUrl)
    ElMessage.success(`已清除本书缓存（服务器 ${deleted} 条，本机 ${localDeleted} 条）`)
    // GAP 82：清除后刷新单书缓存状态
    void loadShelfCacheInfo()
  } catch (err) {
    const e = err as { response?: { status?: number }; message?: string } | null | undefined
    if (e?.response?.status === 404 || e?.response?.status === 501) {
      ElMessage.warning('清除单书缓存接口后端暂未提供（POST /reader3/deleteBookCache）')
    } else {
      ElMessage.error(`清除缓存失败：${err instanceof Error ? err.message : '请稍后重试'}`)
    }
  } finally {
    cacheClearBusy.value = false
  }
}

/* ================= 简介展开/收起（超过 4 行显示「展开/收起」；展开移除 -webkit-line-clamp 限制） ================= */

const introRef = ref<HTMLElement | null>(null)
const introExpanded = ref(false)
/** 是否已完成首测（未测前先按 4 行截断渲染，保证测量时 clamp 已生效） */
const introMeasured = ref(false)
const introLong = ref(false)
/** 4 行截断：未展开且（未测量或确实超 4 行）时生效 */
const introClamped = computed(() => !introExpanded.value && (introLong.value || !introMeasured.value))

watch(
  () => display.value.intro,
  async () => {
    introExpanded.value = false
    introMeasured.value = false
    introLong.value = false
    await nextTick()
    const el = introRef.value
    if (el) {
      introLong.value = el.scrollHeight > el.clientHeight + 2
      introMeasured.value = true
    }
  },
)

/* ================= 相关推荐（后端 ruleRelated 契约：getBookInfo 返回 relatedBooks；未实现则整区隐藏） ================= */

interface RelatedBook {
  bookUrl: string
  name: string
  author: string
  origin: string
  originName: string
  tocUrl: string
  coverUrl?: string | null
  [key: string]: unknown
}

const relatedBooks = computed<RelatedBook[]>(() => {
  const raw = info.value?.relatedBooks
  return Array.isArray(raw) ? (raw as RelatedBook[]) : []
})

/** 点击推荐书：组装 Book JSON 调 saveBook 加入书架（bookUrl 为主键；GAP 108：入架前查重） */
async function addRelated(r: RelatedBook) {
  const dup = await checkDupBeforeAdd({ bookUrl: r.bookUrl, name: r.name || '' })
  if (dup === 'exists') {
    ElMessage.info('已在书架')
    return
  }
  if (dup === 'same-name-cancel') return
  try {
    await saveBook({
      bookUrl: r.bookUrl,
      tocUrl: r.tocUrl || '',
      origin: r.origin,
      originName: r.originName,
      name: r.name,
      author: r.author,
      coverUrl: r.coverUrl ?? null,
      charset: null,
      type: 0,
      group: 0,
      latestChapterTime: 0,
    } as Book)
    ElMessage.success(`《${r.name}》已加入书架`)
  } catch {
    // 错误提示已由拦截器统一处理（saveBook 非 silent）
  }
}

onMounted(() => {
  // 简繁模式可能在其他页面改动 → 挂载时同步全站状态（目录 tab 展示随其响应）
  syncHanMode()
  load()
})

// P2：路由参数变化（/book/A → /book/B 复用同一组件实例）时重新加载——
// vue-router 仅替换 params，不会重新挂载组件，若不 watch 则展示旧书数据
watch(bookUrl, () => {
  void load()
})
</script>

<template>
  <div class="detail-page">
    <!-- 极简顶栏：返回书架 -->
    <header class="topbar">
      <button class="back-btn" type="button" @click="router.push('/')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 12H5" />
          <path d="M11 18l-6-6 6-6" />
        </svg>
        <span>书架</span>
      </button>
      <img class="brand-logo" src="/logo.svg" alt="夜读" />
        <span class="brand">夜读<span class="brand-dot">.</span></span>
    </header>

    <main class="content">
      <!-- GAP 18：详情 / 目录 tab -->
      <div class="tabs">
        <button
          type="button"
          class="tab"
          :class="{ active: activeTab === 'detail' }"
          @click="activeTab = 'detail'"
        >
          详情
        </button>
        <button
          type="button"
          class="tab"
          :class="{ active: activeTab === 'toc' }"
          @click="openToc"
        >
          目录
        </button>
      </div>

      <!-- 加载骨架（浅灰静置块） -->
      <div v-if="loading" class="detail-layout" aria-label="加载中">
        <div class="skeleton-cover"></div>
        <div class="skeleton-info">
          <div class="skeleton-line wide"></div>
          <div class="skeleton-line"></div>
          <div class="skeleton-line"></div>
          <div class="skeleton-line short"></div>
        </div>
      </div>

      <!-- 错误态：不在书架 / 书源获取失败 / 书架拉取失败 -->
      <div v-else-if="loadFailed" class="empty-state">
        <p class="empty-text">{{ errorMsg || '未找到这本书（可能不在书架中）' }}</p>
        <div class="empty-actions">
          <button class="ghost-btn" type="button" @click="load">重试</button>
          <button class="ghost-btn" type="button" @click="router.push('/')">返回书架</button>
        </div>
      </div>

      <!-- GAP 18：目录预览面板（前 50 章，点击进阅读器跳章） -->
      <div v-else-if="activeTab === 'toc'" class="toc-panel">
        <p v-if="tocLoading" class="toc-state">目录加载中…</p>
        <p v-else-if="tocError" class="toc-state">
          目录获取失败（接口 GET /reader3/getBookToc 或书源不可用）
        </p>
        <template v-else>
          <p class="toc-hint">
            共 {{ tocChapters.filter((c) => !c.isVolume).length }} 章 · 预览前 {{ Math.min(TOC_PREVIEW_MAX, tocPreview.length) }} 章，点击进入阅读器并跳转
          </p>
          <ul class="toc-list">
            <li v-for="c in tocEntries" :key="`${c.kind}-${c.index}-${c.title}`" class="toc-row">
              <!-- GAP 91：卷标题分隔行（isVolume） -->
              <div v-if="c.kind === 'volume'" class="toc-volume">{{ c.title }}</div>
              <!-- GAP 147：当前章高亮（书架进度 durChapterIndex） -->
              <template v-else>
                <button
                  class="toc-item"
                  :class="{ current: c.index === currentChapterIndex }"
                  type="button"
                  @click="goToChapterFromToc(c.index)"
                >
                  <span class="toc-idx">{{ c.index + 1 }}</span>
                  <span class="toc-title" :title="c.title">{{ c.title }}</span>
                  <span v-if="c.index === currentChapterIndex" class="toc-cur">读到</span>
                </button>
                <button class="toc-cache" type="button" title="缓存本章" @click.stop="cacheChapter(c.index)">
                  缓存
                </button>
              </template>
            </li>
          </ul>
        </template>
      </div>

      <!-- 详情 -->
      <div v-else class="detail-layout">
        <!-- 封面 -->
        <div class="cover-wrap">
          <!-- 右上角控件组：阅读进度环（常显）+ 换封面（hover 显现，位于环下方不重叠） -->
          <div class="cover-corner">
            <div
              v-if="readProgress"
              class="read-ring"
              :class="{ done: readProgress.percent >= 100 }"
              :title="`读到第 ${readProgress.cur + 1} 章/共 ${readProgress.total} 章`"
            >
              <svg class="read-ring-svg" viewBox="0 0 36 36" aria-hidden="true">
                <circle class="ring-track" cx="18" cy="18" r="15.5" />
                <circle
                  class="ring-bar"
                  cx="18"
                  cy="18"
                  r="15.5"
                  pathLength="100"
                  :stroke-dasharray="`${readProgress.percent} 100`"
                />
              </svg>
              <span class="read-ring-text">{{ readProgress.percent }}%</span>
            </div>
            <!-- GAP 19：换封面（右上角；仅书架书可保存） -->
            <button
              v-if="shelfBook"
              class="cover-change"
              type="button"
              :disabled="coverBusy"
              :title="coverBusy ? '上传中…' : '更换封面（上传图片到服务器）'"
              @click="openCoverPicker"
            >
              {{ coverBusy ? '上传中…' : '换封面' }}
            </button>
          </div>
          <img
            v-if="display.cover && !coverFailed"
            :src="resolveCoverUrl(display.cover)"
            class="cover-img"
            :alt="display.name"
            @error="coverFailed = true"
          />
          <div v-else class="cover-ph">
            <span class="cover-ph-char">{{ coverInitial(display.name) }}</span>
          </div>
          <input
            ref="coverInputRef"
            class="visually-hidden"
            type="file"
            accept="image/*"
            @change="onCoverPick"
          />
        </div>

        <!-- 信息 -->
        <div class="book-info">
          <h1 class="book-name">{{ display.name }}</h1>
          <p v-if="display.author" class="book-author">{{ display.author }}</p>
          <!-- GAP 145：标签 chips（customTag 逗号分隔） -->
          <p v-if="displayTags.length" class="book-tags">
            <span v-for="t in displayTags" :key="t" class="tag-chip">{{ t }}</span>
          </p>
          <p v-if="display.latestChapterTitle" class="book-latest">
            最新章节：{{ display.latestChapterTitle }}
          </p>

          <p v-if="display.intro" ref="introRef" class="book-intro" :class="{ clamped: introClamped }">
            {{ display.intro }}
          </p>
          <button
            v-if="display.intro && introMeasured && introLong"
            class="intro-toggle"
            type="button"
            @click="introExpanded = !introExpanded"
          >
            {{ introExpanded ? '收起' : '展开' }}
          </button>

          <div class="actions">
            <!-- GAP 21：书架书显示「已在书架」标记 + 开始阅读；非书架书 → 加入书架（入架成功后变开始阅读） -->
            <span v-if="shelfBook" class="onshelf-tag" title="本书已在书架中">已在书架</span>
            <button
              v-if="shelfBook"
              class="update-toggle"
              type="button"
              role="switch"
              :aria-checked="!!shelfBook.canUpdate"
              :disabled="updateBusy"
              :title="shelfBook.canUpdate ? '关闭追更（不再自动检查新章节）' : '开启追更（自动检查新章节）'"
              @click="toggleCanUpdate"
            >
              <span class="update-label">追更</span>
              <span class="update-switch" :class="{ on: !!shelfBook.canUpdate }">
                <span class="update-knob"></span>
              </span>
            </button>
            <button v-if="shelfBook" class="read-btn" type="button" @click="startReading">{{ resumeLabel }}</button>
            <template v-else>
              <button class="read-btn ghost" type="button" @click="startReadingTemp">直接阅读</button>
              <button class="add-btn" type="button" :disabled="saving" @click="addToShelf">
                加入书架
              </button>
            </template>
            <!-- 全书搜索（书架书本地正文搜索；命中后跳阅读页该章） -->
            <button v-if="shelfBook" class="search-btn" type="button" @click="openSearch">全书搜索</button>
            <!-- GAP 145：编辑元数据（书名/作者/标签/简介弹窗） -->
            <button v-if="shelfBook" class="search-btn" type="button" @click="openEdit">编辑</button>
            <!-- 换源（书架书且带书源：搜索同书其他书源并切换） -->
            <button v-if="canSwitchSource()" class="search-btn" type="button" @click="openSource">换源</button>
            <!-- 导出（GET /reader3/exportBook：txt/epub/html blob 下载） -->
            <button class="search-btn" type="button" @click="openExport">导出</button>
            <!-- 章节缓存（服务器 / 本机双向：单章、至末尾、全本、指定范围） -->
            <button class="search-btn" type="button" @click="openCacheDialog">缓存</button>
            <!-- GAP 82：单书缓存状态（getShelfBookWithCacheInfo silent；后端未实现时隐藏） -->
            <span
              v-if="shelfBook && shelfCache"
              class="cache-state"
              :title="`已缓存 ${shelfCache.chapterCount} 章 · ${fmtCacheSize(shelfCache.size)}`"
            >
              已缓存 {{ shelfCache.chapterCount }} 章 · {{ fmtCacheSize(shelfCache.size) }}
            </span>
            <!-- GAP 79：清除本书缓存（POST /reader3/deleteBookCache：删 book_chapters 该书行） -->
            <button v-if="shelfBook" class="search-btn" type="button" :disabled="cacheClearBusy" @click="clearBookCache">
              {{ cacheClearBusy ? '清理中…' : '清缓存' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 相关推荐（后端 ruleRelated 返回 relatedBooks 时显示；未实现则整区隐藏） -->
      <section v-if="activeTab === 'detail' && relatedBooks.length" class="related-section">
        <h2 class="related-title">相关推荐</h2>
        <div class="related-grid">
          <button
            v-for="r in relatedBooks"
            :key="r.bookUrl"
            class="related-card"
            type="button"
            :title="`将《${r.name}》加入书架`"
            @click="addRelated(r)"
          >
            <span class="related-name">{{ r.name }}</span>
            <span class="related-author">{{ r.author || '佚名' }}</span>
          </button>
        </div>
      </section>
    </main>

    <!-- 全书搜索弹层（GET /reader3/searchBookContent） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="searchOpen" class="dlg-overlay" @click.self="closeSearch">
          <div
            class="dlg dlg-search"
            role="dialog"
            aria-modal="true"
            aria-label="全书搜索"
            tabindex="-1"
            @keydown.esc="closeSearch"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">全书搜索</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="searchBusy" @click="closeSearch">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="runSearch">
              <div class="search-row">
                <input
                  v-model="searchKey"
                  class="search-input"
                  type="text"
                  placeholder="搜索《{{ display.name }}》正文"
                  spellcheck="false"
                />
                <button class="accent-btn" type="submit" :disabled="searchBusy || !searchKey.trim()">
                  {{ searchBusy ? '搜索中…' : '搜索' }}
                </button>
              </div>
              <p class="field-tip">搜索本书全部章节正文（本地书），命中后点击跳转阅读页对应章节。</p>
              <p v-if="searchMsg" class="search-msg" :class="{ error: searchMsgError }">{{ searchMsg }}</p>
              <ul v-if="searchHits.length" class="search-hits">
                <li v-for="(hit, i) in searchHits" :key="`${hit.chapterIndex}-${i}`">
                  <button class="hit-btn" type="button" @click="goToHit(hit)">
                    <span class="hit-title">{{ hit.title || `第 ${hit.chapterIndex + 1} 章` }}</span>
                    <span class="hit-snippet">{{ hit.snippet }}</span>
                  </button>
                </li>
              </ul>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- 换源弹层（GAP 81：优先 /reader3/searchBookSourceSSE 流式——逐源增量；连接失败降级 GET /reader3/searchBookSource） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="sourceOpen" class="dlg-overlay" @click.self="closeSource">
          <div
            class="dlg dlg-source"
            role="dialog"
            aria-modal="true"
            aria-label="换源"
            tabindex="-1"
            @keydown.esc="closeSource"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">换源 · {{ display.name }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="sourceBusy || sourceSwitching" @click="closeSource">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="source-body">
              <p class="field-tip">搜索《{{ display.name }}》的其他书源（当前：{{ currentOrigin || '—' }}），点击结果即可切换。</p>

              <!-- 搜索中（GAP 81：SSE 流式——实时显示已返回源数） -->
              <div v-if="sourceBusy" class="source-busy">
                <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                  <path d="M21 12a9 9 0 1 1-6.2-8.56" />
                </svg>
                <span v-if="sourceDoneCount > 0">正在搜索其他书源…（已返回 {{ sourceDoneCount }} 个源）</span>
                <span v-else>正在搜索其他书源…</span>
              </div>

              <!-- 结果工具条：搜索过滤（书源名二次过滤）+ 刷新 -->
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

              <!-- 结果列表：当前源置顶 + originName 排序 + 失效标注（GAP 20；SSE 增量到达实时追加）——搜索过滤后展示（P1-6：v-for 用 sourceFiltered，修复过滤失效 bug） -->
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
                    <span class="source-name">{{ r.originName || r.origin || '未知书源' }}</span>
                    <span class="source-url">{{ r.origin }}</span>
                    <span v-if="r.origin === currentOrigin" class="source-cur">当前</span>
                    <span v-else-if="invalidSourceUrls.has(r.origin)" class="source-cur invalid">失效</span>
                  </button>
                </li>
              </ul>

              <!-- 过滤后无匹配提示 -->
              <p v-else-if="!sourceBusy && sourceResults.length > 0 && sourceFiltered.length === 0" class="source-empty">
                未找到匹配「{{ sourceKeyword }}」的书源
              </p>

              <!-- 空 / 失败提示（搜索结束后仍无结果时） -->
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
    <!-- 导出弹层（GET /reader3/exportBook：txt/epub/html） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="exportOpen" class="dlg-overlay" @click.self="closeExport">
          <div
            class="dlg dlg-export"
            role="dialog"
            aria-modal="true"
            aria-label="导出书籍"
            tabindex="-1"
            @keydown.esc="closeExport"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">导出 · {{ display.name }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="exportBusy" @click="closeExport">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="export-formats">
              <button
                v-for="f in EXPORT_FORMATS"
                :key="f.value"
                class="fmt-btn"
                :class="{ active: exportFormat === f.value }"
                type="button"
                :disabled="exportBusy"
                @click="exportFormat = f.value"
              >
                <span class="fmt-label">{{ f.label }}</span>
                <span class="fmt-tip">{{ f.tip }}</span>
              </button>
            </div>
            <!-- GAP 144：txt 编码选择（UTF-8 / GBK——后端并行实现中，未就绪时仍输出 UTF-8） -->
            <div v-if="exportFormat === 'txt'" class="export-enc">
              <span class="enc-label">编码</span>
              <div class="enc-seg">
                <button
                  class="enc-btn"
                  :class="{ active: exportEncoding === 'utf-8' }"
                  type="button"
                  :disabled="exportBusy"
                  @click="exportEncoding = 'utf-8'"
                >
                  UTF-8
                </button>
                <button
                  class="enc-btn"
                  :class="{ active: exportEncoding === 'gbk' }"
                  type="button"
                  :disabled="exportBusy"
                  @click="exportEncoding = 'gbk'"
                >
                  GBK
                </button>
              </div>
            </div>
            <p class="field-tip">由服务器生成 {{ exportFormat.toUpperCase() }} 文件并下载{{ exportFormat === 'txt' ? `（${exportEncoding.toUpperCase()} 编码）` : '' }}。</p>
            <p v-if="exportMsg" class="search-msg" :class="{ error: exportMsgError }">{{ exportMsg }}</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="exportBusy" @click="closeExport">取消</button>
              <button class="accent-btn" type="button" :disabled="exportBusy" @click="confirmExport">
                {{ exportBusy ? '导出中…' : '导出' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- GAP 145：元数据编辑弹窗（书名/作者/标签/简介——saveBook patch 字段） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="editOpen" class="dlg-overlay" @click.self="closeEdit">
          <div
            class="dlg dlg-edit"
            role="dialog"
            aria-modal="true"
            aria-label="编辑书籍信息"
            tabindex="-1"
            @keydown.esc="closeEdit"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">编辑 · {{ display.name }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="editBusy" @click="closeEdit">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmEdit">
              <label class="field">
                <span class="field-label">书名<em>*</em></span>
                <input v-model="editForm.name" class="field-input" type="text" maxlength="100" spellcheck="false" :disabled="editBusy" />
              </label>
              <label class="field">
                <span class="field-label">作者</span>
                <input v-model="editForm.author" class="field-input" type="text" maxlength="60" spellcheck="false" :disabled="editBusy" />
              </label>
              <label class="field">
                <span class="field-label">标签</span>
                <input v-model="editForm.tags" class="field-input" type="text" placeholder="多个标签用逗号分隔，如：科幻, 连载中" spellcheck="false" :disabled="editBusy" />
                <span class="field-tip">保存到 customTag，详情页以标签展示</span>
              </label>
              <label class="field">
                <span class="field-label">简介</span>
                <textarea
                  v-model="editForm.intro"
                  class="intro-textarea"
                  rows="6"
                  placeholder="自定义简介（留空则显示书源解析的简介）"
                  spellcheck="false"
                  :disabled="editBusy"
                ></textarea>
                <span class="field-tip">保存到 customIntro，展示优先于书源简介</span>
              </label>
              <p v-if="editMsg" class="field-tip" :class="{ error: editMsgError }">{{ editMsg }}</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="editBusy" @click="closeEdit">取消</button>
                <button class="accent-btn" type="submit" :disabled="editBusy || !editForm.name.trim()">
                  {{ editBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 章节缓存弹层（ChapterCacheDialog：服务器 / 本机双向） -->
    <ChapterCacheDialog
      v-model="cacheOpen"
      :book-url="bookUrl"
      :book-name="display.name"
      :chapters="realTocChapters"
      :origin="shelfBook?.origin || info?.origin || ''"
      :default-from="cacheFrom"
      :default-scope="cacheScope"
      :allow-server="!!shelfBook"
      @done="onCacheDone"
    />
  </div>
</template>

<style scoped>
.detail-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  animation: fade-in 0.2s ease both;
}

/* ================= 顶栏 ================= */
.topbar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 32px;
  background: var(--bg-float);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}
.back-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px 6px 8px;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.back-btn:hover {
  color: var(--text-1);
  border-color: var(--border);
}
.back-btn svg {
  width: 15px;
  height: 15px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 15px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--text-1);}
.brand-dot {
  color: var(--accent);
  font-weight: 400;
}

/* ================= 详情 / 目录 tab（GAP 18） ================= */
.tabs {
  display: flex;
  gap: 32px;
  margin-bottom: 32px;
  border-bottom: 1px solid var(--border);
}
.tab {
  position: relative;
  padding: 4px 2px 12px;
  border: none;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 3px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.tab:hover {
  color: var(--text-2);
}
.tab.active {
  color: var(--accent);
  font-weight: 400;
}
.tab.active::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 2px;
  border-radius: 2px;
  background: var(--accent);
}

/* ================= 目录预览面板（GAP 18） ================= */
.toc-panel {
  min-height: 40vh;
}
.toc-state {
  padding: 64px 0;
  margin: 0;
  text-align: center;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.toc-hint {
  margin: 0 0 16px;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.toc-list {
  list-style: none;
  margin: 0;
  padding: 0;
  border-top: 1px solid var(--border);
}
.toc-list li + li {
  border-top: 1px solid var(--border);
}
.toc-row {
  display: flex;
  align-items: stretch;
}
.toc-item {
  flex: 1;
  min-width: 0;
  width: auto;
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 11px 6px;
  border: none;
  background: none;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}
.toc-item:hover {
  background: var(--accent-soft);
}
.toc-item:hover .toc-title {
  color: var(--accent);
}
.toc-idx {
  flex-shrink: 0;
  min-width: 34px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.toc-title {
  flex: 1;
  min-width: 0;
  font-size: 13.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.15s ease;
}
.toc-cache {
  flex-shrink: 0;
  margin: 6px 6px 6px 0;
  padding: 0 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.15s ease,
    border-color 0.15s ease;
}
.toc-cache:hover {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 自定义封面（GAP 19）+ 阅读进度环（右上角） ================= */
/* 右上角控件组：进度环常显，换封面 hover 显现（位于环下方，不重叠） */
.cover-corner {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 3;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
}
/* 阅读进度环：SVG circle + stroke-dasharray 百分比（pathLength=100 归一）；读完变绿 */
.read-ring {
  position: relative;
  width: 44px;
  height: 44px;
}
.read-ring::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: rgba(20, 20, 24, 0.55);
  backdrop-filter: blur(1px);
}
.read-ring-svg {
  position: relative;
  width: 100%;
  height: 100%;
  display: block;
  transform: rotate(-90deg);
}
.ring-track {
  fill: none;
  stroke: rgba(255, 255, 255, 0.3);
  stroke-width: 3;
}
.ring-bar {
  fill: none;
  stroke: var(--accent);
  stroke-width: 3;
  stroke-linecap: round;
  transition: stroke 0.3s ease;
}
/* 读完（100%）变绿 */
.read-ring.done .ring-bar {
  stroke: #2f9e44;
}
.read-ring-text {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 500;
  color: #fff;
  font-variant-numeric: tabular-nums;
}
.read-ring.done .read-ring-text {
  color: #7ee2a8;
}
.cover-change {
  padding: 4px 10px;
  border: none;
  border-radius: 999px;
  background: rgba(20, 20, 24, 0.66);
  color: #fff;
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  opacity: 0;
  backdrop-filter: blur(2px);
  transition: opacity 0.2s ease, background-color 0.2s ease;
}
.cover-wrap:hover .cover-change,
.cover-change:focus-visible {
  opacity: 1;
}
.cover-change:hover:not(:disabled) {
  background: var(--accent);
}
.cover-change:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}
.cover-file-input {
  display: none;
}

/* ================= 内容 ================= */
.content {
  flex: 1;
  width: min(860px, 100%);
  margin: 0 auto;
  padding: 56px 32px 80px;
}

.detail-layout {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 48px;
  align-items: start;
}

/* 封面 */
.cover-wrap {
  position: relative;
  aspect-ratio: 3 / 4;
  width: 220px;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid var(--border);
  background: var(--surface);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}
.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.cover-ph {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #9aa8a0;
}
.cover-ph-char {
  font-size: 56px;
  font-weight: 300;
  color: rgba(255, 255, 255, 0.94);
  letter-spacing: 2px;
}

/* 信息 */
.book-info {
  padding-top: 4px;
}
.book-name {
  margin: 0;
  font-size: 32px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-1);
  line-height: 1.4;
}
.book-author {
  margin: 14px 0 0;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.book-latest {
  margin: 10px 0 0;
  font-size: 13px;
  font-weight: 300;
  color: var(--text-3);
  max-width: 520px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.book-latest::before {
  content: '';
  display: inline-block;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent);
  margin-right: 8px;
  vertical-align: 2px;
}

/* 简介：留白段落（超过 4 行截断，展开移除限制） */
.book-intro {
  margin: 28px 0 0;
  max-width: 560px;
  font-size: 14px;
  font-weight: 300;
  line-height: 2;
  letter-spacing: 0.5px;
  color: var(--text-2);
  white-space: pre-line;
}
.book-intro.clamped {
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.intro-toggle {
  margin: 12px 0 0;
  padding: 0;
  border: none;
  background: none;
  color: var(--accent);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 2px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.intro-toggle:hover {
  color: var(--accent-deep);
}

/* 操作区 */
.actions {
  margin-top: 40px;
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}
/* 已在书架标记（GAP 21）：细字徽标，紧邻开始阅读 */
.onshelf-tag {
  padding: 2px 10px;
  border-radius: 4px;
  border: 1px solid var(--accent);
  color: var(--accent);
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 2px;
}
/* 追更开关（legacy canUpdate）：细字开关，紧邻已在书架标记 */
.update-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 3px 10px;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.update-toggle:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.update-toggle:disabled {
  opacity: 0.6;
  cursor: default;
}
.update-label {
  line-height: 1;
}
.update-switch {
  position: relative;
  width: 30px;
  height: 16px;
  border-radius: 999px;
  background: var(--border-strong);
  transition: background-color 0.2s ease;
}
.update-switch.on {
  background: var(--accent);
}
.update-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s ease;
}
.update-switch.on .update-knob {
  transform: translateX(14px);
}
/* 单书缓存状态（GAP 82）：细字徽标，紧邻缓存本书按钮 */
.cache-state {
  padding: 2px 10px;
  border-radius: 4px;
  border: 1px solid var(--border-strong);
  color: var(--text-3);
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  font-variant-numeric: tabular-nums;
}
.read-btn {
  padding: 13px 44px;
  border: none;
  border-radius: var(--radius);
  background: var(--accent);
  color: var(--on-accent);
  font-family: inherit;
  font-size: 14.5px;
  font-weight: 400;
  letter-spacing: 4px;
  cursor: pointer;
  transition: background 0.2s ease;
}
.read-btn:hover {
  background: var(--accent-deep);
}
.read-btn:active {
  background: var(--accent-deep);
}

/* 加入书架：细字描边 → hover 强调色 */
.add-btn {
  padding: 13px 44px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 14.5px;
  font-weight: 300;
  letter-spacing: 4px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.add-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.add-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

/* 全书搜索：细字描边按钮（次于主按钮） */
.search-btn {
  padding: 13px 32px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 3px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.search-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* ================= 相关推荐 ================= */
.related-section {
  margin-top: 64px;
  padding-top: 32px;
  border-top: 1px solid var(--border);
}
.related-title {
  margin: 0 0 20px;
  font-size: 15px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-1);
}
.related-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}
.related-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.related-card:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.related-name {
  font-size: 13.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
.related-author {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* ================= 骨架 / 空态 ================= */
.skeleton-cover {
  width: 220px;
  aspect-ratio: 3 / 4;
  border-radius: 10px;
  background: #f0f0f2;
  border: 1px solid var(--border);
}
.skeleton-info {
  padding-top: 8px;
}
.skeleton-line {
  height: 13px;
  margin-bottom: 18px;
  border-radius: 4px;
  background: #f0f0f2;
}
.skeleton-line.wide {
  width: 60%;
  height: 26px;
}
.skeleton-line.short {
  width: 40%;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
  padding: 120px 0;
}
.empty-text {
  margin: 0;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.empty-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}
.ghost-btn {
  padding: 9px 28px;
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
.ghost-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 全书搜索弹层 ================= */
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
  width: min(520px, 100%);
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
.dlg-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
}
.search-row {
  display: flex;
  gap: 8px;
}
.search-input {
  flex: 1;
  min-width: 0;
  height: 36px;
  padding: 0 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  outline: none;
  transition: border-color 0.2s ease;
}
.search-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.search-input:focus {
  border-color: var(--accent);
  background: var(--surface);
}
.accent-btn {
  flex-shrink: 0;
  padding: 0 20px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--on-accent);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease;
}
.accent-btn:hover:not(:disabled) {
  background: var(--accent-deep);
  border-color: var(--accent-deep);
}
.accent-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.field-tip {
  margin: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.search-msg {
  margin: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.search-msg.error {
  color: #cf4444;
}
.search-hits {
  list-style: none;
  margin: 4px 0 0;
  padding: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.hit-btn {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 9px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.hit-btn:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.hit-title {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.hit-snippet {
  font-size: 12px;
  font-weight: 300;
  line-height: 1.6;
  color: var(--text-3);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.source-tools {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 10px;
}
.source-filter {
  flex: 1;
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
.source-empty {
  padding: 14px 4px;
  font-size: 13px;
  font-weight: 300;
  color: var(--text-2, #888);
}

/* ================= 换源弹层 ================= */
.dlg-source {
  width: min(480px, 100%);
}
.source-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  overflow-y: auto;
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
.source-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.source-row {
  width: 100%;
  display: flex;
  align-items: baseline;
  gap: 10px;
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
.source-name {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
}
.source-url {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
/* 失效书源标注（GAP 20）：行置灰 + 红色「失效」徽标 */
.source-row.invalid {
  opacity: 0.55;
}
.source-cur.invalid {
  border-color: rgba(207, 68, 68, 0.5);
  color: #cf4444;
}
.source-retry {
  display: flex;
  justify-content: flex-start;
}

/* ================= 导出弹层 ================= */
.dlg-export {
  width: min(440px, 100%);
}
.export-formats {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}
.fmt-btn {
  flex: 1 1 88px;
  min-width: 72px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 10px 0;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.fmt-btn:hover:not(:disabled) {
  border-color: var(--accent);
}
.fmt-btn.active {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.fmt-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
.fmt-label {
  font-size: 13.5px;
  font-weight: 400;
  letter-spacing: 2px;
  color: var(--text-1);
}
.fmt-tip {
  font-size: 10.5px;
  font-weight: 300;
  color: var(--text-3);
}
.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
.ghost-btn {
  padding: 7px 18px;
  border-radius: var(--radius);
  border: 1px solid var(--border-strong);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.ghost-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.ghost-btn:disabled,
.accent-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.accent-btn {
  padding: 7px 18px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--on-accent);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease;
}
.accent-btn:hover:not(:disabled) {
  background: var(--accent-deep);
  border-color: var(--accent-deep);
}

/* ================= 缓存本书弹层 ================= */
.dlg-cache {
  width: min(420px, 100%);
}
.cache-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 12px 0 2px;
}
.cache-bar {
  flex: 1;
  height: 5px;
  border-radius: 999px;
  background: var(--hover);
  overflow: hidden;
}
.cache-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
  transition: width 0.3s ease;
}
.cache-percent {
  flex-shrink: 0;
  min-width: 56px;
  text-align: right;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}

/* txt 编码选择（GAP 144：UTF-8 / GBK） */
.export-enc {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}
.enc-label {
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.enc-seg {
  display: flex;
  gap: 6px;
}
.enc-btn {
  padding: 5px 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    color 0.2s ease,
    background-color 0.2s ease;
}
.enc-btn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.enc-btn.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}
.enc-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

/* ================= 元数据编辑弹层（GAP 145） ================= */
.dlg-edit {
  width: min(440px, 100%);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.field-label {
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-2);
}
.field-label em {
  color: #cf4444;
  font-style: normal;
}
.field-input {
  width: 100%;
  height: 34px;
  padding: 0 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  outline: none;
  transition: border-color 0.2s ease;
}
.field-input:focus {
  border-color: var(--accent);
}
.field-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.intro-textarea {
  width: 100%;
  padding: 8px 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  line-height: 1.7;
  resize: vertical;
  outline: none;
  transition: border-color 0.2s ease;
}
.intro-textarea:focus {
  border-color: var(--accent);
}
.intro-textarea::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.field-tip.error {
  color: #cf4444;
}

/* ================= 标签 chips（GAP 145） ================= */
.book-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 12px 0 0;
}
.tag-chip {
  padding: 2px 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-2);
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 1px;
}

/* ================= 目录卷标题分隔行（GAP 91）+ 当前章高亮（GAP 147） ================= */
.toc-volume {
  padding: 12px 6px 6px;
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--accent);
  background: var(--accent-soft);
}
.toc-item.current .toc-title {
  color: var(--accent);
  font-weight: 500;
}
.toc-item.current .toc-idx {
  color: var(--accent);
}
.toc-cur {
  flex-shrink: 0;
  padding: 1px 8px;
  border-radius: 999px;
  border: 1px solid var(--accent);
  color: var(--accent);
  font-size: 10.5px;
  font-weight: 400;
  letter-spacing: 1px;
}

/* 弹窗动画：fade 200ms */
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

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    padding: 12px 16px;
  }
  .content {
    padding: 36px 20px 64px;
  }
  .detail-layout {
    grid-template-columns: 1fr;
    gap: 28px;
  }
  .cover-wrap,
  .skeleton-cover {
    width: 168px;
  }
  .book-name {
    font-size: 26px;
  }
}
</style>
