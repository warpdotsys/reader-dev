<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { searchBookMulti, searchBookMultiSSE } from '@/api/search'
import { getBookSources } from '@/api/sources'
import { getBookInfo } from '@/api/books'
import { getBookshelf, saveBook } from '@/api/bookshelf'
import { useUserStore } from '@/stores/user'
import { clearSearchHistory, loadSearchHistory, pushSearchHistory } from '@/utils/searchHistory'
import { hanText, syncHanMode } from '@/utils/hanMode'
import { proxyImageUrl } from '@/utils/imageProxy'
import { t } from '@/utils/i18n'
import TopNav from '@/components/TopNav.vue'
import type { Book, BookInfo, ReturnData, SearchBook } from '@/types'

const router = useRouter()
const route = useRoute()

/* ================= 搜索 ================= */
const key = ref('')
/** 按书源分组搜索：空串 = 全部（结果列表本身仍按书源分组折叠） */
const activeSearchGroup = ref('')
const searchGroups = ref<string[]>([])
async function loadSearchGroups() {
  try {
    const res = await getBookSources()
    const set = new Set<string>()
    for (const s of res.data ?? []) {
      for (const g of (s.bookSourceGroup ?? '').split(/[,，、\s]+/)) {
        const t = g.trim()
        if (t && t !== '全部') set.add(t)
      }
    }
    searchGroups.value = Array.from(set).sort()
  } catch {
    searchGroups.value = []
  }
}
/** 精确匹配开关（默认关=模糊 contains；开启后请求带 exact=1，后端按书名/作者等值过滤） */
const EXACT_KEY = 'reader_search_exact'
function loadExact(): boolean {
  try {
    return localStorage.getItem(EXACT_KEY) === '1'
  } catch {
    return false
  }
}
const exact = ref(loadExact())
function toggleExact() {
  exact.value = !exact.value
  try {
    localStorage.setItem(EXACT_KEY, exact.value ? '1' : '0')
  } catch {
    /* localStorage 不可用时静默（仅本次会话生效） */
  }
}
/** 直接输入书籍 URL（legacy Index「精确搜书」：调 getBookInfo 打开详情） */
const isUrlInput = computed(() => /^https?:\/\/[^\s]+$/i.test(key.value.trim()))
async function openUrlBook() {
  const url = key.value.trim()
  if (!isUrlInput.value || searching.value) return
  searching.value = true
  errorMsg.value = ''
  try {
    const res = await getBookInfo(url, '', { silent: true })
    if (!res.isSuccess) {
      errorMsg.value = res.errorMsg || '未获取到书籍信息，请确认链接是否支持直接打开'
      searched.value = true
      return
    }
    pushHistory(url)
    void router.push(`/book/${encodeURIComponent(url)}`)
  } catch {
    errorMsg.value = '未获取到书籍信息，请确认链接是否支持直接打开'
    searched.value = true
  } finally {
    searching.value = false
  }
}
const searching = ref(false)
const searched = ref(false)
const errorMsg = ref('')
const stopped = ref(false)
/** 当前搜索是否走 SSE（决定实时源计数是否显示） */
const usingSSE = ref(false)

/** 合并后的结果（bookUrl 去重，同书多源合并来源标签） */
interface MergedResult {
  book: SearchBook
  /** 来源标签（按 origin 去重，显示 originName || origin） */
  origins: { key: string; label: string }[]
}

const results = ref<MergedResult[]>([])
const failedCovers = new Set<string>()
/** 已返回结果的书源数（SSE 每源一个 book 事件，lastIndex 去重计数） */
const searchedSources = ref(0)

const bookMap = new Map<string, MergedResult>()
const completedSources = new Set<number>()
let sseAbort: (() => void) | null = null
let batchAbort: AbortController | null = null
/** 搜索代数：取消/停止后使在途 SSE/批量响应失效 */
let searchSeq = 0

/* ================= GAP 100：搜索分页（批量模式逐页「加载更多」；SSE 已全量则提示已全部） ================= */

/** 批量模式当前已加载页（后端 searchBookMulti page 从 1 开始） */
const batchPage = ref(1)
/** 批量模式是否已全部（某页无新增去重结果 → 到底） */
const batchExhausted = ref(false)
const batchLoadingMore = ref(false)

function labelOf(b: SearchBook): string {
  return b.originName || b.origin
}

/** bookUrl 去重合并：新书入表；同书追加来源标签并补全缺失展示字段；返回新增书数 */
function mergeBooks(books: SearchBook[]): number {
  let added = 0
  for (const b of books) {
    let entry = bookMap.get(b.bookUrl)
    if (!entry) {
      entry = { book: b, origins: [] }
      bookMap.set(b.bookUrl, entry)
      added++
    }
    const okey = b.origin || labelOf(b)
    if (!entry.origins.some((o) => o.key === okey)) {
      entry.origins.push({ key: okey, label: labelOf(b) })
    }
    const cur = entry.book
    if (!cur.intro && b.intro) cur.intro = b.intro
    if (!cur.latestChapterTitle && b.latestChapterTitle) cur.latestChapterTitle = b.latestChapterTitle
    if (!cur.wordCount && b.wordCount) cur.wordCount = b.wordCount
    if (!cur.author && b.author) cur.author = b.author
  }
  results.value = Array.from(bookMap.values())
  return added
}

/** 服务端业务错误（event: error）：NEED_LOGIN 跳登录，其余展示错误 */
function handleErrorEvent(ret: ReturnData) {
  if (ret.data === 'NEED_LOGIN' || (ret.errorMsg || '').includes('请登录')) {
    const store = useUserStore()
    store.clear()
    void router.replace({ path: '/login', query: { redirect: router.currentRoute.value.fullPath } })
    return
  }
  errorMsg.value = ret.errorMsg || '搜索失败，请稍后重试'
  searching.value = false
  searched.value = true
}

async function doSearch(kw?: string) {
  const word = (kw ?? key.value).trim()
  if (!word || searching.value) return
  key.value = word
  suggestOpen.value = false // 开始搜索时收起联想
  const seq = ++searchSeq
  sseAbort = null
  batchAbort = null
  searching.value = true
  searched.value = false
  errorMsg.value = ''
  stopped.value = false
  usingSSE.value = true
  results.value = []
  bookMap.clear()
  completedSources.clear()
  searchedSources.value = 0
  batchPage.value = 1
  batchExhausted.value = false
  batchLoadingMore.value = false

  // 1) 优先 SSE 流式搜索（增量显示）
  try {
    const handle = await searchBookMultiSSE(
      {
        key: word,
        bookSourceGroup: activeSearchGroup.value,
        lastIndex: -1,
        searchSize: 50,
        concurrentCount: 48,
        exact: exact.value,
      },
      {
        onBooks: (lastIndex, books) => {
          if (seq !== searchSeq) return
          if (lastIndex >= 0) completedSources.add(lastIndex)
          searchedSources.value = completedSources.size
          mergeBooks(books)
        },
        onEnd: () => {
          if (seq !== searchSeq) return
          searching.value = false
          searched.value = true
          pushHistory(word)
        },
        onErrorEvent: (ret) => {
          if (seq !== searchSeq) return
          handleErrorEvent(ret)
        },
        onStreamError: (msg) => {
          if (seq !== searchSeq) return
          errorMsg.value = msg
          searching.value = false
          searched.value = true
        },
      },
    )
    if (seq !== searchSeq) {
      // 等待连接期间已被停止
      handle.abort()
      return
    }
    sseAbort = handle.abort
  } catch {
    // 2) SSE 传输失败/不支持 → 降级批量模式
    if (seq !== searchSeq) return
    usingSSE.value = false
    await runBatch(word, seq)
  }
}

/** 批量降级：现有 searchBookMulti（maxSources=50，AbortSignal 可中止），按 page 分页累加 */
async function runBatch(word: string, seq: number, page = 1) {
  batchAbort = new AbortController()
  try {
    const res = await searchBookMulti(
      word,
      50,
      batchAbort.signal,
      page,
      exact.value,
      activeSearchGroup.value,
    )
    if (seq !== searchSeq) return
    if (!res.isSuccess) {
      if ((res.data as unknown) === 'NEED_LOGIN' || (res.errorMsg || '').includes('请登录')) {
        const store = useUserStore()
        store.clear()
        void router.replace({ path: '/login', query: { redirect: router.currentRoute.value.fullPath } })
        return
      }
      throw new Error(res.errorMsg || '搜索失败，请稍后重试')
    }
    const before = bookMap.size
    mergeBooks(res.data ?? [])
    batchPage.value = page
    // 该页无新增去重结果 → 已全部（后续「加载更多」不再出现）
    if (bookMap.size === before) batchExhausted.value = true
    searched.value = true
    pushHistory(word)
  } catch (err) {
    if (seq !== searchSeq) return
    errorMsg.value = err instanceof Error ? err.message : '搜索失败，请稍后重试'
  } finally {
    if (seq === searchSeq) searching.value = false
    if (batchAbort && seq !== searchSeq) batchAbort = null
  }
}

/** GAP 100：批量模式「加载更多」——下一页 searchBookMulti 合并去重 */
async function loadMore() {
  if (usingSSE.value || searching.value || batchLoadingMore.value || batchExhausted.value) return
  const word = key.value.trim()
  if (!word) return
  const seq = searchSeq
  const nextPage = batchPage.value + 1
  batchLoadingMore.value = true
  batchAbort = new AbortController()
  try {
    const res = await searchBookMulti(
      word,
      50,
      batchAbort.signal,
      nextPage,
      exact.value,
      activeSearchGroup.value,
    )
    if (seq !== searchSeq) return
    if (!res.isSuccess) throw new Error(res.errorMsg || '加载失败，请稍后重试')
    const before = bookMap.size
    mergeBooks(res.data ?? [])
    batchPage.value = nextPage
    if (bookMap.size === before) batchExhausted.value = true
  } catch (err) {
    if (seq !== searchSeq) return
    errorMsg.value = err instanceof Error ? err.message : '加载失败，请稍后重试'
  } finally {
    if (seq === searchSeq) batchLoadingMore.value = false
    batchAbort = null
  }
}

/* ================= 按书源分组折叠（GAP 23：组头=源名+数量，点击展开/收起，localStorage 持久化） ================= */
interface SourceGroup {
  key: string
  label: string
  count: number
  entries: MergedResult[]
}

const COLLAPSE_KEY = 'reader_search_collapsed_sources'

function loadCollapsedSources(): Set<string> {
  try {
    const raw = localStorage.getItem(COLLAPSE_KEY)
    if (!raw) return new Set()
    const arr = JSON.parse(raw)
    return new Set(Array.isArray(arr) ? arr.filter((x): x is string => typeof x === 'string') : [])
  } catch {
    return new Set()
  }
}

const collapsedSources = ref<Set<string>>(loadCollapsedSources())

function persistCollapsedSources() {
  try {
    localStorage.setItem(COLLAPSE_KEY, JSON.stringify(Array.from(collapsedSources.value)))
  } catch {
    /* localStorage 不可用时静默 */
  }
}

function toggleSourceGroup(key: string) {
  const s = new Set(collapsedSources.value)
  if (s.has(key)) s.delete(key)
  else s.add(key)
  collapsedSources.value = s
  persistCollapsedSources()
}

/** 结果按书源分组：每组为该源命中的书（书在多源命中时出现在多个组）；按命中数降序、源名升序 */
const sourceGroups = computed<SourceGroup[]>(() => {
  const groups = new Map<string, SourceGroup>()
  for (const entry of results.value) {
    for (const o of entry.origins) {
      let g = groups.get(o.key)
      if (!g) {
        g = { key: o.key, label: o.label, count: 0, entries: [] }
        groups.set(o.key, g)
      }
      g.count++
      g.entries.push(entry)
    }
  }
  return Array.from(groups.values()).sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
})

/** 停止搜索：中断 SSE/批量请求，保留已到达的部分结果 */
function stopSearch() {
  if (!searching.value) return
  searchSeq++
  if (sseAbort) {
    sseAbort()
    sseAbort = null
  }
  if (batchAbort) {
    batchAbort.abort()
    batchAbort = null
  }
  stopped.value = true
  searched.value = true
  searching.value = false
  pushHistory(key.value.trim())
}

function onEnter() {
  void doSearch()
}

/** 切换书源分组：正在进行/已完成搜索时立即按新分组重搜 */
function pickSearchGroup(group: string) {
  if (activeSearchGroup.value === group) return
  activeSearchGroup.value = group
  if (searching.value) stopSearch()
  const word = key.value.trim()
  if (searched.value && word) void doSearch(word)
}

function openBook(book: SearchBook) {
  // 带 origin（书源 URL）——详情页非书架书分支需要它匹配书源（否则报"未找到这本书"）
  void router.push({
    path: `/book/${encodeURIComponent(book.bookUrl)}`,
    query: book.origin ? { origin: book.origin } : undefined,
  })
}

/* ================= 搜索历史（localStorage，最近 10 条——与探索页共用） ================= */
const history = ref<string[]>([])

function loadHistory() {
  history.value = loadSearchHistory()
}

function pushHistory(word: string) {
  history.value = pushSearchHistory(word)
}

function clearHistory() {
  clearSearchHistory()
  history.value = []
}

/* ================= GAP 121：搜索热词（后端无接口——前端静态列表 + localStorage 记录点击排序） ================= */

const HOT_SEARCHES = ['剑来', '诡秘之主', '凡人修仙传', '庆余年', '大奉打更人', '遮天', '雪中悍刀行', '斗破苍穹']
const HOT_CLICKS_KEY = 'reader_hot_clicks'

function loadHotClicks(): Record<string, number> {
  try {
    const raw = JSON.parse(localStorage.getItem(HOT_CLICKS_KEY) ?? '{}') as Record<string, unknown>
    const out: Record<string, number> = {}
    if (raw && typeof raw === 'object') {
      for (const [k, v] of Object.entries(raw)) {
        if (typeof v === 'number' && Number.isFinite(v)) out[k] = v
      }
    }
    return out
  } catch {
    return {}
  }
}

const hotClicks = ref<Record<string, number>>(loadHotClicks())
/** 热词列表：静态表按本地点击次数降序（未点过保持原顺序） */
const hotSearches = computed(() =>
  [...HOT_SEARCHES].sort((a, b) => (hotClicks.value[b] ?? 0) - (hotClicks.value[a] ?? 0)),
)
/** 占位符：显示当前最热词 */
const hotPlaceholder = computed(() =>
  hotSearches.value[0]
    ? t('search.hotPlaceholder', { h: hotSearches.value[0] })
    : t('search.placeholder'),
)
function recordHotClick(word: string) {
  hotClicks.value = { ...hotClicks.value, [word]: (hotClicks.value[word] ?? 0) + 1 }
  try {
    localStorage.setItem(HOT_CLICKS_KEY, JSON.stringify(hotClicks.value))
  } catch {
    /* ignore */
  }
}
function searchHot(word: string) {
  recordHotClick(word)
  void doSearch(word)
}

/* ================= GAP 158：搜索结果快捷加入（行内「+」——saveBook 入架；失败静默不打断浏览） ================= */

const shelfUrlSet = ref<Set<string>>(new Set())
const shelfLoadedOnce = ref(false)
const addingUrls = ref<Set<string>>(new Set())

function loadShelfOnceForAdd() {
  if (shelfLoadedOnce.value) return
  shelfLoadedOnce.value = true
  void getBookshelf()
    .then((res) => {
      shelfUrlSet.value = new Set((res.data ?? []).map((b) => b.bookUrl))
    })
    .catch(() => {
      /* 静默：入架按钮仍可用，已入架状态未知 */
    })
}

async function quickAdd(book: SearchBook) {
  if (addingUrls.value.has(book.bookUrl) || shelfUrlSet.value.has(book.bookUrl)) return
  addingUrls.value = new Set([...addingUrls.value, book.bookUrl])
  try {
    // 入架前先拉详情：搜索结果 tocUrl 常为空、部分书源搜索不含封面/作者——
    // 直接用搜索结果入架会得到「首字封面 + 佚名 + 未获取到章节目录」
    let detail: BookInfo | null = null
    try {
      const res = await getBookInfo(book.bookUrl, book.origin, { silent: true })
      if (res.isSuccess && res.data) detail = res.data
    } catch {
      /* 详情失败仍按搜索结果字段入架，不阻断加书 */
    }
    await saveBook({
      bookUrl: book.bookUrl,
      name: detail?.name || book.name,
      author: detail?.author || book.author,
      origin: detail?.origin || book.origin,
      originName: detail?.originName || book.originName,
      tocUrl: detail?.tocUrl || book.tocUrl || book.bookUrl,
      intro: detail?.intro ?? book.intro ?? '',
      coverUrl: detail?.coverUrl ?? book.coverUrl ?? '',
      kind: detail?.kind ?? book.kind ?? null,
      latestChapterTitle: detail?.latestChapterTitle ?? book.latestChapterTitle ?? null,
      group: 0,
      type: detail?.type ?? book.type ?? 0,
    } as Book)
    shelfUrlSet.value = new Set([...shelfUrlSet.value, book.bookUrl])
    ElMessage.success('已加入书架')
  } catch {
    // GAP 158：入架失败静默（不弹全局错误，不打断浏览）
  } finally {
    addingUrls.value = new Set([...addingUrls.value].filter((u) => u !== book.bookUrl))
  }
}

onMounted(() => {
  loadHistory()
  loadShelfOnceForAdd()
  void loadSearchGroups()
  // 简繁模式可能在其他页面改动（同标签页直写 localStorage 场景）→ 挂载时同步全站状态
  syncHanMode()
  // 支持 /search?key=xxx 预填并自动搜索（阅读页划词「搜索」跳转）
  const kw = typeof route.query.key === 'string' ? route.query.key.trim() : ''
  if (kw) void doSearch(kw)
})

/* ================= GAP 22：搜索建议（输入联想——搜索历史 + 本地书架书匹配；debounce 250ms） ================= */

interface Suggestion {
  text: string
  kind: 'history' | 'book'
  sub?: string
}

const suggestOpen = ref(false)
const suggestions = ref<Suggestion[]>([])
const shelfBooks = ref<Book[]>([])
let shelfLoaded = false
let suggestTimer: number | undefined

/** 书架书懒加载一次（失败静默：仅历史联想） */
function loadShelfOnce() {
  if (shelfLoaded) return
  shelfLoaded = true
  void getBookshelf()
    .then((res) => {
      shelfBooks.value = res.data ?? []
    })
    .catch(() => {
      /* 静默 */
    })
}

/** 联想：历史命中优先（最多 8 条，去重），再补书架书书名命中 */
function computeSuggestions() {
  const kw = key.value.trim()
  if (!kw || searching.value) {
    suggestions.value = []
    suggestOpen.value = false
    return
  }
  const out: Suggestion[] = []
  const seen = new Set<string>()
  for (const h of history.value) {
    if (out.length >= 8) break
    if (h.includes(kw) && !seen.has(h)) {
      seen.add(h)
      out.push({ text: h, kind: 'history' })
    }
  }
  for (const b of shelfBooks.value) {
    if (out.length >= 8) break
    const n = b.name.trim()
    if (n && n.includes(kw) && !seen.has(n)) {
      seen.add(n)
      out.push({ text: n, kind: 'book', sub: b.author || b.originName || undefined })
    }
  }
  suggestions.value = out
  suggestOpen.value = out.length > 0
}

watch(key, () => {
  window.clearTimeout(suggestTimer)
  if (!key.value.trim()) {
    suggestions.value = []
    suggestOpen.value = false
    return
  }
  loadShelfOnce()
  suggestTimer = window.setTimeout(computeSuggestions, 250)
})

function pickSuggestion(s: Suggestion) {
  suggestOpen.value = false
  void doSearch(s.text)
}

function onSearchFocus() {
  suggestOpen.value = suggestions.value.length > 0
}

function closeSuggest() {
  window.clearTimeout(suggestTimer)
  suggestOpen.value = false
}

onBeforeUnmount(() => {
  window.clearTimeout(suggestTimer)
})
</script>

<template>
  <div class="search-page">
    <!-- 极简顶栏：返回书架（P3-A：共享 TopNav minimal） -->
    <TopNav variant="minimal" :back-label="t('nav.backShelf')" @back="router.push('/')" />

    <main class="content">
      <h1 class="page-title">{{ t('search.title') }}</h1>

      <!-- 极简搜索框：细字 + 下划线 focus 强调色 + 搜索按钮 -->
      <div class="search-bar">
        <svg
          class="search-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
        >
          <circle cx="11" cy="11" r="6.5" />
          <path d="M20 20l-3.8-3.8" />
        </svg>
        <input
          v-model="key"
          class="search-input"
          type="text"
          :placeholder="hotPlaceholder"
          spellcheck="false"
          @keydown.enter="onEnter"
          @focus="onSearchFocus"
          @blur="closeSuggest"
        />
        <!-- 精确匹配开关：默认关（模糊）；开启后请求 exact=1，后端按书名/作者等值过滤 -->
        <button
          class="exact-toggle"
          type="button"
          :class="{ on: exact }"
          :title="t('search.exactTip')"
          :aria-pressed="exact"
          @click="toggleExact"
        >
          {{ t('search.exact') }}
        </button>
        <button
          v-if="isUrlInput"
          class="url-open-btn"
          type="button"
          title="直接按链接获取书籍详情（legacy 精确搜书）"
          :disabled="searching"
          @click="openUrlBook"
        >
          {{ searching ? t('common.searching') : '打开链接' }}
        </button>
        <button class="search-btn" type="button" :disabled="searching || !key.trim()" @click="onEnter">
          {{ searching ? t('common.searching') : t('common.search') }}
        </button>
      </div>

      <!-- 按书源分组搜索：胶囊选择，切换后立即重搜 -->
      <div v-if="searchGroups.length" class="group-filter">
        <button
          class="group-chip"
          :class="{ active: activeSearchGroup === '' }"
          type="button"
          @click="pickSearchGroup('')"
        >
          {{ t('common.all') }}
        </button>
        <button
          v-for="g in searchGroups"
          :key="g"
          class="group-chip"
          :class="{ active: activeSearchGroup === g }"
          type="button"
          @click="pickSearchGroup(g)"
        >
          {{ g }}
        </button>
      </div>

      <!-- GAP 22：输入联想（搜索历史 + 书架书匹配，debounce 250ms；mousedown.prevent 保证点击前不丢焦点） -->
      <div v-if="suggestOpen && !searching && key.trim()" class="suggest">
        <button
          v-for="(s, i) in suggestions"
          :key="`${s.kind}-${s.text}-${i}`"
          class="suggest-item"
          type="button"
          @mousedown.prevent
          @click="pickSuggestion(s)"
        >
          <span class="suggest-text">{{ s.text }}</span>
          <span v-if="s.sub" class="suggest-sub">{{ s.sub }}</span>
          <span class="suggest-tag" :class="s.kind">{{ s.kind === 'history' ? t('search.tag.history') : t('search.tag.shelf') }}</span>
        </button>
      </div>

      <!-- GAP 121：热门搜索（静态列表 + 本地点击计数排序） -->
      <div v-if="!searching && !searched && !errorMsg" class="history hot">
        <div class="history-head">
          <span class="history-label">{{ t('search.hot') }}</span>
        </div>
        <div class="history-chips">
          <button
            v-for="h in hotSearches"
            :key="h"
            class="history-chip hot"
            type="button"
            :title="`搜索「${h}」`"
            @click="searchHot(h)"
          >
            {{ h }}
          </button>
        </div>
      </div>

      <!-- 搜索历史 -->
      <div v-if="history.length && !searching && !searched && !errorMsg" class="history">
        <div class="history-head">
          <span class="history-label">{{ t('search.history') }}</span>
          <button class="history-clear" type="button" @click="clearHistory">{{ t('common.clear') }}</button>
        </div>
        <div class="history-chips">
          <button
            v-for="h in history"
            :key="h"
            class="history-chip"
            type="button"
            @click="doSearch(h)"
          >
            {{ h }}
          </button>
        </div>
      </div>

      <!-- 加载态：实时源计数（SSE）+ 停止按钮 -->
      <div v-if="searching" class="state-row" aria-live="polite">
        <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <path d="M21 12a9 9 0 1 1-6.2-8.56" />
        </svg>
        <span class="state-text">
          {{ usingSSE ? t('search.progress', { n: searchedSources }) : t('search.multi') }}
        </span>
        <button class="stop-btn" type="button" @click="stopSearch">{{ t('search.stop') }}</button>
      </div>

      <!-- GAP 72：搜索首载骨架（shimmer 行；首屏无结果时展示） -->
      <div v-if="searching && results.length === 0" class="skeleton-list" :aria-label="t('common.searching')">
        <div v-for="i in 6" :key="i" class="skeleton-row">
          <div class="skeleton-cover"></div>
          <div class="skeleton-body">
            <div class="skeleton-line w60"></div>
            <div class="skeleton-line w40"></div>
            <div class="skeleton-line w85"></div>
          </div>
        </div>
      </div>

      <!-- 错误态（无结果时整行展示） -->
      <div v-else-if="errorMsg && !results.length" class="state-row">
        <span class="state-text error">{{ errorMsg }}</span>
        <button class="retry-btn" type="button" @click="doSearch()">{{ t('common.retry') }}</button>
      </div>

      <!-- 空结果 / 已停止 -->
      <div v-else-if="searched && !results.length" class="state-row">
        <span class="state-text">{{ stopped ? t('search.stopped') : t('search.noResults', { k: key.trim() }) }}</span>
      </div>

      <!-- 结果列表（SSE 增量累积；按书源分组折叠，GAP 23） -->
      <div v-if="results.length" class="results-wrap">
        <p v-if="errorMsg" class="result-note error">{{ errorMsg }}</p>
        <p v-else-if="stopped" class="result-note">{{ t('search.stoppedPartial') }}</p>
        <p class="result-meta">
          {{ t('search.resultsMeta', { n: results.length }) }}<span v-if="searchedSources">{{ t('search.resultsFrom', { n: searchedSources }) }}</span>
        </p>
        <ul class="result-list">
          <li v-for="g in sourceGroups" :key="g.key" class="source-group">
            <button class="group-head" type="button" @click="toggleSourceGroup(g.key)">
              <svg
                class="group-chevron"
                :class="{ open: !collapsedSources.has(g.key) }"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M9 6l6 6-6 6" />
              </svg>
              <span class="group-name" :title="hanText(g.label)">{{ hanText(g.label) }}</span>
              <span class="group-count">{{ g.count }} {{ t('common.unit.book') }}</span>
            </button>
            <div v-if="!collapsedSources.has(g.key)" class="group-body">
              <div
                v-for="entry in g.entries"
                :key="entry.book.bookUrl"
                class="result-item"
                @click="openBook(entry.book)"
              >
                <span v-if="entry.book.coverUrl && !failedCovers.has(entry.book.bookUrl)" class="result-cover">
                  <img
                    :src="proxyImageUrl(entry.book.coverUrl) ?? ''"
                    :alt="hanText(entry.book.name)"
                    loading="lazy"
                    @error="failedCovers.add(entry.book.bookUrl)"
                  />
                </span>
                <span v-else class="result-cover placeholder">{{ hanText(entry.book.name).charAt(0) }}</span>
                <div class="result-main">
                  <p class="result-name" :title="hanText(entry.book.name)">{{ hanText(entry.book.name) }}</p>
                  <p class="result-sub">
                    <span class="result-author">{{ hanText(entry.book.author || t('search.unknownAuthor')) }}</span>
                    <span
                      v-for="o in entry.origins"
                      :key="o.key"
                      class="source-badge"
                      :title="hanText(o.label)"
                    >{{ hanText(o.label) }}</span>
                    <span v-if="entry.book.latestChapterTitle" class="result-chapter" :title="hanText(entry.book.latestChapterTitle)">
                      {{ hanText(entry.book.latestChapterTitle) }}
                    </span>
                  </p>
                  <p v-if="entry.book.intro" class="result-intro">{{ hanText(entry.book.intro) }}</p>
                </div>
                <!-- GAP 158：快捷加入书架（hover 显示「+」；已在书架显示 ✓；失败静默） -->
                <button
                  class="quick-add"
                  type="button"
                  :class="{ done: shelfUrlSet.has(entry.book.bookUrl), busy: addingUrls.has(entry.book.bookUrl) }"
                  :disabled="shelfUrlSet.has(entry.book.bookUrl) || addingUrls.has(entry.book.bookUrl)"
                  :title="shelfUrlSet.has(entry.book.bookUrl) ? t('search.onShelf') : t('search.addShelf')"
                  @click.stop="quickAdd(entry.book)"
                >
                  <svg v-if="shelfUrlSet.has(entry.book.bookUrl)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M4.5 12.5l5 5L19.5 7" />
                  </svg>
                  <span v-else>{{ addingUrls.has(entry.book.bookUrl) ? '…' : '+' }}</span>
                </button>
                <svg class="result-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M9 6l6 6-6 6" />
                </svg>
              </div>
            </div>
          </li>
        </ul>

        <!-- GAP 100：底部加载更多（批量模式逐页；SSE 已全量则提示已全部） -->
        <div v-if="searched && results.length" class="load-more">
          <template v-if="usingSSE">
            <span class="load-all">{{ t('search.loadAll') }}</span>
          </template>
          <template v-else>
            <button
              v-if="!batchExhausted"
              class="load-btn"
              type="button"
              :disabled="batchLoadingMore"
              @click="loadMore"
            >
              {{ batchLoadingMore ? t('common.loading') : t('common.loadMore') }}
            </button>
            <span v-else class="load-all">{{ t('search.loadAllShort') }}</span>
          </template>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.search-page {
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
  gap: 16px;
  padding: 14px 32px;
  background: var(--bg-float);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.back-btn:hover {
  color: var(--accent);
}
.back-btn svg {
  width: 14px;
  height: 14px;
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

/* ================= 内容区 ================= */
.content {
  width: min(720px, 100%);
  margin: 0 auto;
  padding: 44px 32px 72px;
}
.page-title {
  margin: 0 0 26px;
  font-size: 22px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-1);
}

/* 搜索框：细字 + 下划线 focus 强调色 */
.search-bar {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 4px 0 10px;
  border-bottom: 1px solid var(--border);
  transition: border-color 0.2s ease;
}
.search-bar:focus-within {
  border-bottom-color: var(--accent);
}
.search-icon {
  width: 17px;
  height: 17px;
  flex-shrink: 0;
  color: var(--text-3);
  transition: color 0.2s ease;
}
.search-bar:focus-within .search-icon {
  color: var(--accent);
}
.search-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: none;
  color: var(--text-1);
  font-family: inherit;
  font-size: 19px;
  font-weight: 300;
  letter-spacing: 1px;
  outline: none;
}
.search-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.search-btn {
  flex-shrink: 0;
  padding: 6px 18px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: none;
  color: var(--accent);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.search-btn:hover:not(:disabled) {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}
.search-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

/* 精确匹配开关：细字胶囊；开启后 accent 高亮 */
.exact-toggle {
  flex-shrink: 0;
  padding: 4px 12px;
  border-radius: 999px;
  border: 1px solid var(--border-strong);
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.exact-toggle:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.exact-toggle.on {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.url-open-btn {
  flex-shrink: 0;
  padding: 5px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
}
.url-open-btn:hover:not(:disabled) {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
}
.url-open-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 按书源分组：横向滚动胶囊 */
.group-filter {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding: 12px 2px 2px;
  scrollbar-width: none;
}
.group-filter::-webkit-scrollbar {
  display: none;
}
.group-chip {
  flex-shrink: 0;
  padding: 4px 12px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  cursor: pointer;
  transition: color 0.2s ease, border-color 0.2s ease, background 0.2s ease;
}
.group-chip:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.group-chip.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* ================= 搜索联想（GAP 22） ================= */
.suggest {
  display: flex;
  flex-direction: column;
  margin-top: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-float);
  overflow: hidden;
}
.suggest-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 14px;
  border: none;
  border-bottom: 1px solid var(--border);
  background: none;
  color: var(--text-1);
  font-family: inherit;
  font-size: 13.5px;
  font-weight: 300;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.15s ease;
}
.suggest-item:last-child {
  border-bottom: none;
}
.suggest-item:hover {
  background: var(--accent-soft);
}
.suggest-text {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.suggest-sub {
  flex-shrink: 0;
  max-width: 180px;
  font-size: 11.5px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.suggest-tag {
  flex-shrink: 0;
  padding: 1px 7px;
  border-radius: 4px;
  border: 1px solid var(--border-strong);
  color: var(--text-3);
  font-size: 10.5px;
  letter-spacing: 1px;
}
.suggest-tag.book {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 搜索历史 ================= */
.history {
  margin-top: 22px;
}
.history-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.history-label {
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.history-clear {
  border: none;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  cursor: pointer;
  transition: color 0.2s ease;
}
.history-clear:hover {
  color: var(--accent);
}
.history-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.history-chip {
  padding: 4px 12px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.history-chip:hover {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 状态行（加载 / 空 / 错误） ================= */
.state-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 72px 0;
}
.state-text {
  font-size: 13.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.state-text.error {
  color: #cf4444;
}
.mini-spin {
  width: 13px;
  height: 13px;
  color: var(--accent);
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.stop-btn {
  padding: 5px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: none;
  color: var(--accent);
  font-family: inherit;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.stop-btn:hover {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}
.retry-btn {
  padding: 5px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border-strong);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 400;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.retry-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 结果列表（按书源分组） ================= */
.results-wrap {
  margin-top: 22px;
}
.result-meta {
  margin: 0;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.result-note {
  margin: 0 0 10px;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.result-note.error {
  color: #cf4444;
}
.result-list {
  list-style: none;
  margin: 10px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}
/* 书源分组：组头 = 源名 + 数量，点击展开/收起 */
.source-group {
  border-bottom: 1px solid var(--border);
}
.source-group:last-child {
  border-bottom: none;
}
.group-head {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 11px 6px;
  border: none;
  background: none;
  font-family: inherit;
  cursor: pointer;
  transition: background-color 0.2s ease;
}
.group-head:hover {
  background: var(--surface);
}
.group-chevron {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
  color: var(--text-3);
  transition: transform 0.2s ease;
}
.group-chevron.open {
  transform: rotate(90deg);
}
.group-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.2s ease;
}
.group-head:hover .group-name {
  color: var(--accent);
}
.group-count {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.group-body .result-item {
  padding-left: 22px;
}
.group-body .result-item:first-child {
  border-top: none;
}
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 4px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background-color 0.2s ease;
}
.result-item:first-child {
  border-top: 1px solid var(--border);
}
.result-item:hover {
  background: var(--surface);
}
.result-cover {
  flex-shrink: 0;
  width: 44px;
  height: 58px;
  border-radius: 6px;
  overflow: hidden;
  background: var(--accent-soft, #eef2ff);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 300;
  color: var(--accent, #4f46e5);
}
.result-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.result-cover.placeholder {
  font-size: 18px;
  color: var(--text-3, #999);
  background: var(--border, #ececec);
}
.result-main {
  flex: 1;
  min-width: 0;
}
.result-name {
  margin: 0;
  font-size: 14.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.result-sub {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px 10px;
  margin: 5px 0 0;
  min-width: 0;
}
.result-author {
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
  flex-shrink: 0;
}
/* 来源徽标：细字描边（同书多源时展示多枚） */
.source-badge {
  flex-shrink: 0;
  max-width: 140px;
  padding: 1px 8px;
  border-radius: 4px;
  border: 1px solid var(--border-strong);
  color: var(--text-2);
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 0.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.result-chapter {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.result-intro {
  margin: 7px 0 0;
  font-size: 12.5px;
  font-weight: 300;
  line-height: 1.6;
  color: var(--text-2);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
/* GAP 158：快捷加入按钮（hover 显示；触屏常显） */
.quick-add {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  border: 1px solid var(--accent);
  background: none;
  color: var(--accent);
  font-family: inherit;
  font-size: 17px;
  font-weight: 300;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
  transform: scale(0.9);
  transition:
    opacity 0.15s ease,
    transform 0.15s ease,
    background-color 0.2s ease,
    color 0.2s ease;
}
.result-item:hover .quick-add,
.quick-add:focus-visible {
  opacity: 1;
  transform: scale(1);
}
.quick-add:hover {
  background: var(--accent);
  color: var(--on-accent);
}
.quick-add:disabled {
  cursor: default;
}
.quick-add.done {
  border-color: #529b2e;
  color: #529b2e;
  opacity: 1;
  transform: scale(1);
  pointer-events: none;
}
.quick-add.busy {
  opacity: 1;
}
.quick-add svg {
  width: 13px;
  height: 13px;
}

/* GAP 72：搜索骨架（shimmer 行） */
.skeleton-list {
  margin-top: 22px;
  display: flex;
  flex-direction: column;
}
.skeleton-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 4px;
  border-bottom: 1px solid var(--border);
}
.skeleton-cover {
  flex-shrink: 0;
  width: 44px;
  height: 58px;
  border-radius: 6px;
  background: #f0f0f2;
  position: relative;
  overflow: hidden;
}
.skeleton-body {
  flex: 1;
  min-width: 0;
}
.skeleton-line {
  height: 11px;
  border-radius: 4px;
  background: #f0f0f2;
  position: relative;
  overflow: hidden;
}
.skeleton-line.w60 {
  width: 60%;
}
.skeleton-line.w40 {
  width: 40%;
  margin-top: 9px;
}
.skeleton-line.w85 {
  width: 85%;
  margin-top: 9px;
}
.skeleton-cover::after,
.skeleton-line::after {
  content: '';
  position: absolute;
  inset: 0;
  transform: translateX(-100%);
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.65), transparent);
  animation: skeleton-shimmer 1.5s ease-in-out infinite;
}
@keyframes skeleton-shimmer {
  100% {
    transform: translateX(100%);
  }
}

/* GAP 121：热门搜索 chips */
.history.hot .history-chip {
  border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
  color: var(--text-2);
}
.history.hot .history-chip:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.result-arrow {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  color: var(--text-3);
  transition: color 0.2s ease, transform 0.2s ease;
}
.result-item:hover .result-arrow {
  color: var(--accent);
  transform: translateX(2px);
}

/* GAP 100：加载更多（批量模式逐页；SSE 已全部提示） */
.load-more {
  display: flex;
  justify-content: center;
  padding: 28px 0 8px;
}
.load-btn {
  padding: 9px 36px;
  border-radius: 999px;
  border: 1px solid var(--border-strong);
  background: var(--surface);
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.load-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.load-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
.load-all {
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    padding: 12px 16px;
  }
  .content {
    padding: 32px 16px 56px;
  }
  .search-input {
    font-size: 17px;
  }
  .search-btn {
    padding: 6px 14px;
  }
  /* GAP 158：触屏无 hover——快捷加入常显 */
  .quick-add {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
