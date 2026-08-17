<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import {
  deleteRssSource,
  getRssArticle,
  getRssArticles,
  getRssSources,
  markRssArticleRead,
  saveRssSource,
} from '@/api/rss'
import { t } from '@/utils/i18n'
import { sanitizeHtml } from '@/utils/sanitize'
import TopNav from '@/components/TopNav.vue'
import type { RssArticle, RssSource } from '@/types'

/** 分页启发：一次拉满一页即认为还有更多（后端未返回总数） */
const PAGE_SIZE = 20

/* ================= 订阅源列表 ================= */
const sources = ref<RssSource[]>([])
const loadingSources = ref(true)
const activeGroup = ref('全部')

/** 分组（sourceGroup 以空白分隔，可多分组） */
const groups = computed(() => {
  const set = new Set<string>()
  for (const s of sources.value) {
    for (const g of (s.sourceGroup ?? '').split(/\s+/)) {
      if (g) set.add(g)
    }
  }
  return Array.from(set).sort()
})

const filteredSources = computed(() =>
  activeGroup.value === '全部'
    ? sources.value
    : sources.value.filter((s) =>
        (s.sourceGroup ?? '').split(/\s+/).includes(activeGroup.value),
      ),
)

const enabledCount = computed(() => sources.value.filter((s) => s.enabled).length)

/* ================= 选中订阅源 + 文章列表 ================= */
const selectedUrl = ref('')
const selectedSourceName = computed(
  () => sources.value.find((s) => s.sourceUrl === selectedUrl.value)?.sourceName ?? '',
)
const articles = ref<RssArticle[]>([])
const loadingArticles = ref(false)
const articlePage = ref(1)
const hasMore = ref(false)
const loadingMore = ref(false)
const activeSortUrl = ref('')

/** legacy sortUrl 多段 `名称::地址`（&&/换行分隔）解析为分类 tab */
function sortTabsOf(s: RssSource | undefined): { name: string; url: string }[] {
  if (!s) return []
  const raw =
    ((s as unknown as { sortUrl?: string }).sortUrl ??
      (s as unknown as { sort_url?: string }).sort_url ??
      '') || ''
  const tabs: { name: string; url: string }[] = []
  for (const seg of raw.split(/[\n&]/)) {
    const i = seg.indexOf('::')
    if (i > 0 && seg.slice(i + 2).trim()) {
      tabs.push({ name: seg.slice(0, i).trim() || seg.slice(i + 2).trim(), url: seg.slice(i + 2).trim() })
    }
  }
  return tabs
}

const sortTabs = computed(() =>
  sortTabsOf(sources.value.find((s) => s.sourceUrl === selectedUrl.value)),
)

function clearArticles() {
  selectedUrl.value = ''
  articles.value = []
  articlePage.value = 1
  hasMore.value = false
  activeSortUrl.value = ''
  articleMode.value = false
  readingArticle.value = null
  articleContent.value = ''
  articleFilter.value = ''
}

async function loadSources(selectUrl?: string) {
  loadingSources.value = true
  try {
    const res = await getRssSources()
    sources.value = res.data ?? []
    const target =
      (selectUrl || selectedUrl.value) &&
      sources.value.some((s) => s.sourceUrl === selectUrl || s.sourceUrl === selectedUrl.value)
        ? selectUrl || selectedUrl.value
        : ''
    const next =
      target ||
      sources.value.find((s) => s.enabled)?.sourceUrl ||
      sources.value[0]?.sourceUrl ||
      ''
    if (next) await selectSource(next)
    else clearArticles()
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    loadingSources.value = false
  }
}

async function selectSource(url: string) {
  if (url === selectedUrl.value && articles.value.length > 0) return
  selectedUrl.value = url
  articles.value = []
  articlePage.value = 1
  hasMore.value = false
  const tabs = sortTabsOf(sources.value.find((s) => s.sourceUrl === url))
  activeSortUrl.value = tabs[0]?.url ?? ''
  articleMode.value = false
  readingArticle.value = null
  articleContent.value = ''
  articleFilter.value = ''
  await loadArticles(1)
}

async function loadArticles(page: number) {
  if (!selectedUrl.value) return
  if (page === 1) loadingArticles.value = true
  else loadingMore.value = true
  try {
    const res = await getRssArticles(selectedUrl.value, page, activeSortUrl.value || undefined)
    const list = res.data ?? []
    if (page === 1) {
      articles.value = list
    } else {
      // 防重复（后端若忽略 page 参数返回全量）
      const seen = new Set(articles.value.map((a) => a.url))
      articles.value.push(...list.filter((a) => !seen.has(a.url)))
    }
    hasMore.value = list.length >= PAGE_SIZE
    articlePage.value = page
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    loadingArticles.value = false
    loadingMore.value = false
  }
}

/** 切换分类 tab：清空列表回到第 1 页（后端按 sortUrl 抓对应分类 feed） */
function switchSort(url: string) {
  if (activeSortUrl.value === url) return
  activeSortUrl.value = url
  articles.value = []
  articlePage.value = 1
  hasMore.value = false
  articleFilter.value = ''
  articleMode.value = false
  readingArticle.value = null
  articleContent.value = ''
  void loadArticles(1)
}

/** 未读计数（标题旁展示） */
const unreadCount = computed(() => articles.value.filter((a) => !a.hasRead).length)

/* ================= GAP 46：文章列表前端搜索/过滤（标题包含匹配，不请求后端） ================= */
const articleFilter = ref('')

const filteredArticles = computed(() => {
  const kw = articleFilter.value.trim()
  if (!kw) return articles.value
  return articles.value.filter((a) => (a.title || '').includes(kw))
})

/** 列表计数：过滤时显示命中数 */
const articleCountText = computed(() => {
  const kw = articleFilter.value.trim()
  const shown = filteredArticles.value.length
  const total = articles.value.length
  return kw ? `${shown} / ${total} 篇` : `共 ${total} 篇`
})

/* ================= 文章阅读 ================= */
const articleMode = ref(false)
const readingArticle = ref<RssArticle | null>(null)
const articleContent = ref('')
const loadingArticle = ref(false)
const listEl = ref<HTMLElement | null>(null)
const listScrollTop = ref(0)

async function openArticle(a: RssArticle) {
  listScrollTop.value = listEl.value?.scrollTop ?? 0
  articleMode.value = true
  readingArticle.value = a
  articleContent.value = ''
  loadingArticle.value = true
  // 点击即已读（乐观更新 + 后端落库；失败静默，下次进入再同步）
  if (!a.hasRead) {
    a.hasRead = true
    void markRssArticleRead(a.url, true).catch(() => {})
  }
  try {
    const res = await getRssArticle(a.url)
    articleContent.value = sanitizeHtml(res.data?.content ?? '')
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    loadingArticle.value = false
  }
  void nextTick(() => window.scrollTo({ top: 0 }))
}

function backToList() {
  articleMode.value = false
  readingArticle.value = null
  articleContent.value = ''
  void nextTick(() => {
    listEl.value?.scrollTo({ top: listScrollTop.value })
  })
}

/* ================= 文章图片全屏预览（legacy RssArticle 点击图片调起预览） ================= */
const rssImgPreview = ref('')
function onRssContentClick(e: MouseEvent) {
  const img = (e.target as HTMLElement | null)?.closest('img')
  if (img?.src) rssImgPreview.value = img.src
}
function closeRssImgPreview() {
  rssImgPreview.value = ''
}

/* P1-4：净化器已抽到 @/utils/sanitize（sanitizeHtml）——实体解码后校验，
   移除 javascript:/data:/vbscript: 协议 href/src/xlink:href（含编码变体），无外部依赖 */

/* ================= GAP 45：刷新全部（逐源重新抓取 feed；后端无批量接口时逐源循环） ================= */

const refreshingAll = ref(false)
const refreshAllIndex = ref(0)
const refreshAllTotal = ref(0)

async function refreshAll() {
  if (refreshingAll.value) return
  // 全部订阅源（含停用的也刷新，保持列表新鲜；失败单源跳过）
  const list = sources.value
  if (list.length === 0) {
    ElMessage.info('还没有订阅源')
    return
  }
  refreshingAll.value = true
  refreshAllIndex.value = 0
  refreshAllTotal.value = list.length
  let ok = 0
  for (let i = 0; i < list.length; i++) {
    refreshAllIndex.value = i + 1
    try {
      // getRssArticles 后端每次重新抓取 feed → 逐源即刷新；静默失败不打断
      await getRssArticles(list[i].sourceUrl, 1, undefined, { silent: true })
      ok++
    } catch {
      // 单源失败继续
    }
  }
  // 全部刷新后重拉当前选中源列表（若选中的源刷新失败，loadArticles 会提示）
  if (selectedUrl.value) await loadArticles(1)
  refreshingAll.value = false
  ElMessage.success(`刷新完成：${ok}/${list.length} 个订阅源已更新`)
}

/* ================= 新增订阅弹窗 ================= */
const addOpen = ref(false)
const addBusy = ref(false)
const addUrlInput = ref<HTMLInputElement | null>(null)
const addForm = ref({ sourceUrl: '', sourceName: '', sourceGroup: '' })

function openAdd() {
  addForm.value = { sourceUrl: '', sourceName: '', sourceGroup: '' }
  addOpen.value = true
  document.body.style.overflow = 'hidden'
  void nextTick(() => addUrlInput.value?.focus())
}

function closeAdd() {
  if (addBusy.value) return
  addOpen.value = false
  document.body.style.overflow = ''
}

async function confirmAdd() {
  const url = addForm.value.sourceUrl.trim()
  if (!url || addBusy.value) return
  addBusy.value = true
  try {
    await saveRssSource({
      sourceUrl: url,
      sourceName: addForm.value.sourceName.trim() || url,
      sourceGroup: addForm.value.sourceGroup.trim(),
      enabled: true,
    })
    closeAdd()
    await loadSources(url) // 刷新并选中新订阅源
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    addBusy.value = false
  }
}

/* ================= 编辑订阅源（完整字段：规则/排序/图标等经 raw_json 保底） ================= */
const editOpen = ref(false)
const editBusy = ref(false)
const editUrlInput = ref<HTMLInputElement | null>(null)
const editForm = ref({
  sourceUrl: '',
  sourceName: '',
  sourceGroup: '',
  sortUrl: '',
  sourceIcon: '',
  ruleArticles: '',
  ruleTitle: '',
  ruleContent: '',
  enableJs: false,
  enabled: true,
})

function strOf(s: RssSource, key: string): string {
  const v = s[key]
  return typeof v === 'string' ? v : ''
}

function openEdit(s: RssSource) {
  editForm.value = {
    sourceUrl: s.sourceUrl || '',
    sourceName: s.sourceName || '',
    sourceGroup: s.sourceGroup || strOf(s, 'sourceGroup'),
    sortUrl: strOf(s, 'sortUrl'),
    sourceIcon: strOf(s, 'sourceIcon'),
    ruleArticles: strOf(s, 'ruleArticles'),
    ruleTitle: strOf(s, 'ruleTitle'),
    ruleContent: strOf(s, 'ruleContent'),
    enableJs: Boolean(s.enableJs),
    enabled: Boolean(s.enabled),
  }
  editOpen.value = true
  document.body.style.overflow = 'hidden'
  void nextTick(() => editUrlInput.value?.focus())
}

function closeEdit() {
  if (editBusy.value) return
  editOpen.value = false
  document.body.style.overflow = ''
}

async function confirmEdit() {
  const url = editForm.value.sourceUrl.trim()
  if (!url || editBusy.value) return
  editBusy.value = true
  try {
    await saveRssSource({
      sourceUrl: url,
      sourceName: editForm.value.sourceName.trim() || url,
      sourceGroup: editForm.value.sourceGroup.trim() || null,
      sortUrl: editForm.value.sortUrl.trim() || null,
      sourceIcon: editForm.value.sourceIcon.trim() || null,
      ruleArticles: editForm.value.ruleArticles.trim() || null,
      ruleTitle: editForm.value.ruleTitle.trim() || null,
      ruleContent: editForm.value.ruleContent.trim() || null,
      enableJs: editForm.value.enableJs,
      enabled: editForm.value.enabled,
    })
    closeEdit()
    await loadSources(url)
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    editBusy.value = false
  }
}

/* ================= JSON 批量导入订阅源 ================= */
const jsonOpen = ref(false)
const jsonBusy = ref(false)
const jsonText = ref('')
const jsonMsg = ref('')

function openJsonImport() {
  jsonText.value = ''
  jsonMsg.value = ''
  jsonOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeJsonImport() {
  if (jsonBusy.value) return
  jsonOpen.value = false
  document.body.style.overflow = ''
}

async function importJson() {
  if (jsonBusy.value) return
  let arr: unknown
  try {
    arr = JSON.parse(jsonText.value)
  } catch {
    jsonMsg.value = 'JSON 解析失败，请检查格式'
    return
  }
  if (!Array.isArray(arr)) {
    jsonMsg.value = 'JSON 必须是订阅源对象数组'
    return
  }
  const list = (arr as Array<Record<string, unknown>>).filter(
    (x) => x && typeof x === 'object' && typeof (x.sourceUrl ?? x.rssSourceUrl) === 'string',
  )
  if (!list.length) {
    jsonMsg.value = '未找到有效的订阅源（需要 sourceUrl 字段）'
    return
  }
  jsonBusy.value = true
  let ok = 0
  try {
    for (const raw of list) {
      const url = String(raw.sourceUrl ?? raw.rssSourceUrl ?? '')
      if (!url) continue
      await saveRssSource({
        ...raw,
        sourceUrl: url,
        sourceName: String(raw.sourceName ?? raw.rssSourceName ?? url),
        enabled: raw.enabled !== false,
      })
      ok += 1
    }
    jsonMsg.value = `已导入 ${ok}/${list.length} 个订阅源`
    if (ok) {
      await loadSources(urlOf(list[0]))
    }
  } catch {
    jsonMsg.value = `导入中断：已导入 ${ok}/${list.length} 个订阅源`
  } finally {
    jsonBusy.value = false
  }
}

function urlOf(raw: Record<string, unknown>): string {
  return String(raw.sourceUrl ?? raw.rssSourceUrl ?? '')
}

/** 订阅源图标（sourceIcon 可能为 http(s) 或 data: 图片） */
function sourceIconUrl(s: RssSource): string {
  const v = s.sourceIcon || strOf(s, 'sourceIcon')
  return /^(https?:|data:image\/)/i.test(v) ? v : ''
}

function onIconError(ev: Event) {
  ;(ev.target as HTMLImageElement).style.display = 'none'
}

/* ================= 删除确认弹窗 ================= */
const deleting = ref<RssSource | null>(null)
const deleteBusy = ref(false)

function askDelete(s: RssSource) {
  deleting.value = s
  document.body.style.overflow = 'hidden'
}

async function confirmDelete() {
  const s = deleting.value
  if (!s || deleteBusy.value) return
  deleteBusy.value = true
  try {
    await deleteRssSource(s.sourceUrl)
    closeDelete()
    await loadSources() // 若删除的是当前选中源，自动落到第一个可用源
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    deleteBusy.value = false
  }
}

function closeDelete() {
  deleting.value = null
  document.body.style.overflow = ''
}

/* ================= 时间格式化（兼容秒/毫秒时间戳） ================= */
function fmtTime(t: number | undefined | null): string {
  if (!t) return ''
  const ms = t < 1e12 ? t * 1000 : t
  const d = new Date(ms)
  if (Number.isNaN(d.getTime())) return ''
  const pad = (n: number) => String(n).padStart(2, '0')
  const now = new Date()
  const hm = `${pad(d.getHours())}:${pad(d.getMinutes())}`
  if (d.toDateString() === now.toDateString()) return hm
  if (d.getFullYear() === now.getFullYear()) {
    return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${hm}`
  }
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

onMounted(() => loadSources())

onBeforeUnmount(() => {
  document.body.style.overflow = ''
})
</script>

<template>
  <div class="rss-page">
    <!-- 顶部导航（P3-A：共享 TopNav） -->
    <TopNav active="/rss" :links="['bookshelf', 'search', 'sources', 'rss', 'users', 'settings']" show-users-link />

    <main class="rss-main">
      <!-- 左栏：订阅源（分组胶囊 + 列表） -->
      <aside class="source-col">
        <div class="col-head">
          <h2 class="col-title">{{ t('rss.title') }}</h2>
          <span class="col-count">{{ enabledCount }}/{{ sources.length }}</span>
          <button
            class="add-btn refresh-all-btn"
            type="button"
            :disabled="refreshingAll"
            :title="refreshingAll ? t('rss.refreshing', { i: refreshAllIndex, t: refreshAllTotal }) : t('rss.refreshAll')"
            @click="refreshAll"
          >
            <svg v-if="!refreshingAll" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M20 11a8 8 0 1 0-2.3 6.3" />
              <path d="M20 5v6h-6" />
            </svg>
            <svg v-else class="refresh-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <path d="M21 12a9 9 0 1 1-6.2-8.56" />
            </svg>
          </button>
          <button class="add-btn" type="button" :title="t('rss.addTip')" @click="openAdd">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
              <path d="M12 5v14M5 12h14" />
            </svg>
          </button>
          <button class="head-text-btn" type="button" title="导入订阅源 JSON" @click="openJsonImport">导入</button>
        </div>

        <!-- 分组胶囊 -->
        <div class="group-pills">
          <button
            class="pill"
            type="button"
            :class="{ active: activeGroup === '全部' }"
            @click="activeGroup = '全部'"
          >
            {{ t('common.all') }}
          </button>
          <button
            v-for="g in groups"
            :key="g"
            class="pill"
            type="button"
            :class="{ active: activeGroup === g }"
            @click="activeGroup = g"
          >
            {{ g }}
          </button>
        </div>

        <div v-if="loadingSources" class="col-state">{{ t('common.loading') }}</div>
        <div v-else-if="filteredSources.length === 0" class="col-state">
          {{ sources.length === 0 ? t('rss.empty') : t('rss.emptyGroup') }}
        </div>
        <ul v-else class="source-list">
          <li
            v-for="s in filteredSources"
            :key="s.sourceUrl"
            class="source-item"
            :class="{ active: s.sourceUrl === selectedUrl }"
            @click="selectSource(s.sourceUrl)"
          >
            <span class="source-dot" :class="{ on: s.enabled }"></span>
            <img v-if="sourceIconUrl(s)" class="source-icon" :src="sourceIconUrl(s)" alt="" @error="onIconError" />
            <span class="source-name" :title="s.sourceName">{{ s.sourceName }}</span>
            <button class="source-edit" type="button" title="编辑订阅源" @click.stop="openEdit(s)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 20h9" />
                <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
              </svg>
            </button>
            <button class="source-del" type="button" :title="t('rss.deleteTip')" @click.stop="askDelete(s)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                <path d="M6 6l12 12M18 6L6 18" />
              </svg>
            </button>
          </li>
        </ul>
      </aside>

      <!-- 右栏：文章列表 / 阅读区 -->
      <section class="content-col">
        <!-- 阅读模式 -->
        <div v-if="articleMode" class="article-read">
          <button class="back-btn" type="button" @click="backToList">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M15 18l-6-6 6-6" />
            </svg>
            <span>{{ t('rss.backList') }}</span>
          </button>
          <h1 class="article-title">{{ readingArticle?.title || t('rss.noTitle') }}</h1>
          <p class="article-meta">
            <span>{{ readingArticle?.author || selectedSourceName }}</span>
            <template v-if="fmtTime(readingArticle?.time)">
              <span class="meta-sep">·</span>
              <span>{{ fmtTime(readingArticle?.time) }}</span>
            </template>
          </p>
          <div v-if="loadingArticle" class="state-text loading">{{ t('rss.articleLoading') }}</div>
          <div v-else-if="!articleContent" class="state-text">{{ t('rss.articleEmpty') }}</div>
          <div v-else class="rss-content" v-html="articleContent" @click="onRssContentClick"></div>
        </div>

        <!-- 列表模式 -->
        <template v-else>
          <div class="col-head">
            <h2 class="col-title">{{ selectedSourceName || t('rss.articles') }}</h2>
            <span class="col-count"
              >{{ articleCountText }}<span v-if="!articleFilter.trim() && unreadCount"> · {{ t('rss.unread', { n: unreadCount }) }}</span></span
            >
          </div>
          <!-- RSS 分类 tab（legacy sortUrl 多段 `名称::地址`） -->
          <div v-if="sortTabs.length" class="sort-pills">
            <button
              v-for="tab in sortTabs"
              :key="tab.url"
              class="pill"
              type="button"
              :class="{ active: activeSortUrl === tab.url }"
              @click="switchSort(tab.url)"
            >
              {{ tab.name }}
            </button>
          </div>
          <!-- GAP 46：文章标题前端过滤 -->
          <div v-if="selectedUrl && articles.length" class="article-filter">
            <svg class="article-filter-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
              <circle cx="11" cy="11" r="6.5" />
              <path d="M20 20l-3.8-3.8" />
            </svg>
            <input
              v-model="articleFilter"
              class="article-filter-input"
              type="text"
              :placeholder="t('rss.filterPlaceholder')"
              spellcheck="false"
            />
            <button
              v-if="articleFilter"
              class="article-filter-clear"
              type="button"
              :title="t('rss.clearFilter')"
              @click="articleFilter = ''"
            >
              ×
            </button>
          </div>
          <div v-if="loadingArticles" class="state-text loading">{{ t('rss.articlesLoading') }}</div>
          <div v-else-if="!selectedUrl" class="state-text">{{ t('rss.pleaseSelect') }}</div>
          <div v-else-if="articles.length === 0" class="state-text">{{ t('rss.noArticles') }}</div>
          <div v-else-if="filteredArticles.length === 0" class="state-text">{{ t('rss.noMatch', { k: articleFilter.trim() }) }}</div>
          <ul v-else ref="listEl" class="article-list">
            <li
              v-for="a in filteredArticles"
              :key="a.url"
              class="article-item"
              :class="{ read: a.hasRead }"
              @click="openArticle(a)"
            >
              <img
                v-if="a.cover"
                v-lazy="a.cover"
                class="article-item-cover"
                :alt="''"
                loading="lazy"
              />
              <div class="article-item-body">
                <p class="article-item-title" :title="a.title">{{ a.title || t('rss.noTitle') }}</p>
                <p class="article-item-meta">
                  <span>{{ a.author || selectedSourceName }}</span>
                  <template v-if="fmtTime(a.time)">
                    <span class="meta-sep">·</span>
                    <span>{{ fmtTime(a.time) }}</span>
                  </template>
                </p>
              </div>
            </li>
          </ul>
          <div v-if="hasMore" class="load-more">
            <button
              class="more-btn"
              type="button"
              :disabled="loadingMore"
              @click="loadArticles(articlePage + 1)"
            >
              {{ loadingMore ? t('common.loading') : t('common.loadMore') }}
            </button>
          </div>
        </template>
      </section>
    </main>

    <!-- 文章图片全屏预览 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="rssImgPreview" class="rss-img-mask" @click.self="closeRssImgPreview">
          <div class="rss-img-stage">
            <img :src="rssImgPreview" alt="文章配图" />
            <button class="rss-img-close" type="button" title="关闭" @click="closeRssImgPreview">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <path d="M6 6l12 12M18 6L6 18" />
              </svg>
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 新增订阅弹窗（自写轻量，Teleport + fade 200ms） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="addOpen" class="dlg-overlay" @click.self="closeAdd">
          <div
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="新增订阅"
            tabindex="-1"
            @keydown.esc="closeAdd"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">{{ t('rss.add') }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="addBusy" @click="closeAdd">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmAdd">
              <label class="field">
                <span class="field-label">订阅地址 *</span>
                <input
                  ref="addUrlInput"
                  v-model="addForm.sourceUrl"
                  class="field-input"
                  type="url"
                  placeholder="https://example.com/feed.xml"
                  spellcheck="false"
                  required
                />
              </label>
              <label class="field">
                <span class="field-label">名称</span>
                <input
                  v-model="addForm.sourceName"
                  class="field-input"
                  type="text"
                  placeholder="留空则使用订阅地址"
                  spellcheck="false"
                />
              </label>
              <label class="field">
                <span class="field-label">分组</span>
                <input
                  v-model="addForm.sourceGroup"
                  class="field-input"
                  type="text"
                  placeholder="如：新闻 / 博客（多个以空格分隔）"
                  spellcheck="false"
                />
              </label>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="addBusy" @click="closeAdd">取消</button>
                <button
                  class="accent-btn"
                  type="submit"
                  :disabled="addBusy || !addForm.sourceUrl.trim()"
                >
                  {{ addBusy ? '保存中…' : '添加' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 删除确认弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="deleting" class="dlg-overlay" @click.self="closeDelete">
          <div
            class="dlg"
            role="alertdialog"
            aria-modal="true"
            aria-label="删除订阅源"
            tabindex="-1"
            @keydown.esc="closeDelete"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">删除订阅源</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="deleteBusy" @click="closeDelete">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <p class="dlg-text">确定删除「{{ deleting?.sourceName }}」吗？</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="deleteBusy" @click="closeDelete">取消</button>
              <button class="danger-btn" type="button" :disabled="deleteBusy" @click="confirmDelete">
                {{ deleteBusy ? '删除中…' : '删除' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 编辑订阅源弹窗（完整字段：规则/排序/图标等经 raw_json 保底） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="editOpen" class="dlg-overlay" @click.self="closeEdit">
          <div
            class="dlg dlg-edit"
            role="dialog"
            aria-modal="true"
            aria-label="编辑订阅源"
            tabindex="-1"
            @keydown.esc="closeEdit"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">编辑订阅源</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="editBusy" @click="closeEdit">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmEdit">
              <label class="field">
                <span class="field-label">订阅地址 *</span>
                <input
                  ref="editUrlInput"
                  v-model="editForm.sourceUrl"
                  class="field-input"
                  type="url"
                  spellcheck="false"
                  required
                />
              </label>
              <label class="field">
                <span class="field-label">名称</span>
                <input v-model="editForm.sourceName" class="field-input" type="text" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">分组</span>
                <input v-model="editForm.sourceGroup" class="field-input" type="text" placeholder="多个以空格分隔" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">排序/分类地址（sortUrl）</span>
                <textarea
                  v-model="editForm.sortUrl"
                  class="field-textarea"
                  rows="2"
                  placeholder="多段格式：分类1::https://…&#10;分类2::https://…"
                  spellcheck="false"
                ></textarea>
              </label>
              <label class="field">
                <span class="field-label">订阅源图标</span>
                <input v-model="editForm.sourceIcon" class="field-input" type="url" placeholder="http(s) 或 data:image/…" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">列表规则（ruleArticles）</span>
                <input v-model="editForm.ruleArticles" class="field-input" type="text" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">标题规则（ruleTitle）</span>
                <input v-model="editForm.ruleTitle" class="field-input" type="text" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">正文规则（ruleContent）</span>
                <input v-model="editForm.ruleContent" class="field-input" type="text" spellcheck="false" />
              </label>
              <div class="field">
                <span class="field-label">启用</span>
                <button
                  class="switch"
                  :class="{ on: editForm.enabled }"
                  type="button"
                  role="switch"
                  :aria-checked="editForm.enabled"
                  @click="editForm.enabled = !editForm.enabled"
                >
                  <span class="switch-knob"></span>
                </button>
              </div>
              <div class="field">
                <span class="field-label">启用 JS</span>
                <button
                  class="switch"
                  :class="{ on: editForm.enableJs }"
                  type="button"
                  role="switch"
                  :aria-checked="editForm.enableJs"
                  @click="editForm.enableJs = !editForm.enableJs"
                >
                  <span class="switch-knob"></span>
                </button>
              </div>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="editBusy" @click="closeEdit">取消</button>
                <button class="accent-btn" type="submit" :disabled="editBusy || !editForm.sourceUrl.trim()">
                  {{ editBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- JSON 批量导入订阅源 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="jsonOpen" class="dlg-overlay" @click.self="closeJsonImport">
          <div
            class="dlg dlg-edit"
            role="dialog"
            aria-modal="true"
            aria-label="导入订阅源 JSON"
            tabindex="-1"
            @keydown.esc="closeJsonImport"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">导入订阅源 JSON</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="jsonBusy" @click="closeJsonImport">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <label class="field">
              <span class="field-label">订阅源数组</span>
              <textarea
                v-model="jsonText"
                class="field-textarea"
                rows="12"
                placeholder='[{"sourceUrl":"https://…/feed.xml","sourceName":"示例源"}]'
                spellcheck="false"
              ></textarea>
            </label>
            <p class="field-tip">支持 legacy 字段：sourceUrl/sourceName/sourceGroup/sortUrl/ruleArticles 等。</p>
            <p v-if="jsonMsg" class="json-msg">{{ jsonMsg }}</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="jsonBusy" @click="closeJsonImport">取消</button>
              <button class="accent-btn" type="button" :disabled="jsonBusy || !jsonText.trim()" @click="importJson">
                {{ jsonBusy ? '导入中…' : '导入' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.rss-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  animation: fade-in 0.2s ease both;
}

/* ================= 顶部导航 ================= */
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

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.brand-logo {
  width: 26px;
  height: 26px;
}
.brand-name {
  font-size: 17px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--text-1);
}
.brand-dot {
  color: var(--accent);
  font-weight: 400;
}

.nav-area {
  display: flex;
  align-items: center;
  gap: 18px;
}
.nav-link {
  padding: 5px 2px;
  border: none;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.nav-link:hover {
  color: var(--accent);
}
.nav-link.active {
  color: var(--accent);
  font-weight: 400;
}

/* ================= 两栏布局 ================= */
.rss-main {
  width: min(1200px, 100%);
  margin: 0 auto;
  padding: 32px;
  flex: 1;
  display: grid;
  grid-template-columns: 272px 1fr;
  gap: 28px;
  align-items: start;
}

/* ================= 左栏：订阅源 ================= */
.source-col {
  position: sticky;
  top: 76px;
  display: flex;
  flex-direction: column;
  padding: 16px 14px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.col-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 0 6px 12px;
}
.col-title {
  margin: 0;
  font-size: 15px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.col-count {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= GAP 46：文章过滤框 ================= */
.article-filter {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 6px 12px;
  padding: 2px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
  transition: border-color 0.2s ease;
}
.article-filter:focus-within {
  border-color: var(--accent);
}
.article-filter-icon {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  color: var(--text-3);
}
.article-filter-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: none;
  outline: none;
  color: var(--text-1);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
}
.article-filter-input::placeholder {
  color: var(--text-3);
}
.article-filter-clear {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: var(--text-3);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  border-radius: 50%;
}
.article-filter-clear:hover {
  color: var(--text-1);
  background: var(--hover);
}

/* 新增按钮（细描边圆形 +） */
.add-btn {
  margin-left: auto;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: none;
  color: var(--text-2);
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.add-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.add-btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
.add-btn svg {
  width: 11px;
  height: 11px;
}
/* GAP 45：刷新全部按钮（刷新中旋转） */
.refresh-all-btn {
  margin-left: 0;
}
.refresh-spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 分组胶囊 */
.group-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 0 6px 12px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 10px;
}

/* RSS 分类 tab（legacy sortUrl 多段） */
.sort-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 8px 0 4px;
}
.pill {
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.pill:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.pill.active {
  color: var(--on-accent);
  border-color: var(--accent);
  background: var(--accent);
  font-weight: 400;
}

/* 订阅源列表 */
.source-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 56vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.source-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}
.source-item:hover {
  background: var(--hover);
}
.source-item.active {
  background: var(--accent-soft);
}
.source-dot {
  flex-shrink: 0;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--border-strong);
  transition: background-color 0.2s ease;
}
.source-dot.on {
  background: var(--accent);
}
.source-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-del {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 4px;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  opacity: 0;
  transition:
    opacity 0.2s ease,
    color 0.2s ease,
    background-color 0.2s ease;
}
.source-item:hover .source-del,
.source-del:focus-visible {
  opacity: 1;
}
.source-del:hover {
  color: #cf4444;
  background: var(--hover);
}
.source-del svg {
  width: 10px;
  height: 10px;
}
.source-edit {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 4px;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  opacity: 0;
  transition:
    opacity 0.2s ease,
    color 0.2s ease,
    background-color 0.2s ease;
}
.source-item:hover .source-edit,
.source-edit:focus-visible {
  opacity: 1;
}
.source-edit:hover {
  color: var(--accent);
  background: var(--hover);
}
.source-edit svg {
  width: 10px;
  height: 10px;
}
.source-icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  object-fit: cover;
  background: var(--bg);
}

.col-state {
  padding: 28px 10px;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  text-align: center;
}

/* ================= 右栏：内容 ================= */
.content-col {
  display: flex;
  flex-direction: column;
  min-height: 60vh;
  padding: 18px 20px 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

/* 文章列表 */
.article-list {
  list-style: none;
  margin: 0;
  padding: 0;
  border-top: 1px solid var(--border);
}
.article-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 15px 8px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background-color 0.2s ease;
  border-radius: 6px;
}
.article-item-cover {
  flex-shrink: 0;
  width: 72px;
  height: 48px;
  object-fit: cover;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--surface);
}
.article-item-body {
  flex: 1;
  min-width: 0;
}
.article-item:hover {
  background: var(--hover);
}
.article-item-title {
  margin: 0;
  font-size: 14.5px;
  font-weight: 400;
  line-height: 1.6;
  color: var(--text-1);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
/* 未读加粗 / 已读置灰 */
.article-item:not(.read) .article-item-title {
  font-weight: 600;
}
.article-item.read .article-item-title {
  font-weight: 400;
  color: var(--text-3);
}
.article-item.read .article-item-meta {
  color: var(--text-3);
  opacity: 0.7;
}
.article-item-meta {
  margin: 6px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.meta-sep {
  margin: 0 4px;
}

/* 加载更多 */
.load-more {
  display: flex;
  justify-content: center;
  padding: 18px 0 4px;
}
.more-btn {
  padding: 6px 22px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.more-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.more-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

/* 阅读区 */
.article-read {
  padding: 12px 8px 40px;
}
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 2px;
  border: none;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.back-btn:hover {
  color: var(--accent);
}
.back-btn svg {
  width: 12px;
  height: 12px;
}
.article-title {
  margin: 18px 0 0;
  font-size: 21px;
  font-weight: 300;
  line-height: 1.5;
  letter-spacing: 1px;
  color: var(--text-1);
}
.article-meta {
  margin: 10px 0 0;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* 正文排版：行高 1.9 / 段落间距 1em */
.rss-content {
  margin-top: 26px;
  font-size: 15px;
  font-weight: 400;
  line-height: 1.9;
  color: var(--text-1);
  word-break: break-word;
}
.rss-content :deep(p) {
  margin: 0 0 1em;
}
.rss-content :deep(h1),
.rss-content :deep(h2),
.rss-content :deep(h3),
.rss-content :deep(h4) {
  margin: 1.6em 0 0.8em;
  font-weight: 400;
  line-height: 1.5;
  color: var(--text-1);
}
.rss-content :deep(a) {
  color: var(--accent);
  text-decoration: none;
  word-break: break-all;
}
.rss-content :deep(a:hover) {
  text-decoration: underline;
}
.rss-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 6px;
  margin: 0.6em 0;
  cursor: zoom-in;
}
.rss-content :deep(blockquote) {
  margin: 1em 0;
  padding: 2px 0 2px 16px;
  border-left: 2px solid var(--border-strong);
  color: var(--text-2);
}
.rss-content :deep(pre) {
  margin: 1em 0;
  padding: 12px 14px;
  border-radius: 6px;
  background: var(--hover);
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.7;
}
.rss-content :deep(code) {
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  font-size: 0.92em;
  background: var(--hover);
  border-radius: 4px;
  padding: 1px 5px;
}
.rss-content :deep(pre code) {
  background: none;
  padding: 0;
}

/* 文章图片全屏预览 */
.rss-img-mask {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(12, 12, 14, 0.78);
}
.rss-img-stage {
  position: relative;
  max-width: 92vw;
  max-height: 90vh;
}
.rss-img-stage img {
  display: block;
  max-width: 100%;
  max-height: 90vh;
  object-fit: contain;
  border-radius: 8px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
}
.rss-img-close {
  position: absolute;
  top: -38px;
  right: -4px;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.14);
  color: #fff;
  cursor: pointer;
}
.rss-img-close svg {
  width: 15px;
  height: 15px;
}
.rss-content :deep(ul),
.rss-content :deep(ol) {
  margin: 0 0 1em;
  padding-left: 1.6em;
}
.rss-content :deep(li) {
  margin: 0.3em 0;
}
.rss-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--border);
  margin: 1.8em 0;
}
.rss-content :deep(table) {
  border-collapse: collapse;
  margin: 1em 0;
  max-width: 100%;
}
.rss-content :deep(th),
.rss-content :deep(td) {
  border: 1px solid var(--border);
  padding: 6px 10px;
  font-size: 13.5px;
}

/* ================= 状态 ================= */
.state-text {
  padding: 48px 0;
  text-align: center;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.state-text.loading {
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

/* ================= 弹窗（自写轻量，Teleport + fade 200ms） ================= */
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
  width: min(400px, 100%);
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
  background: var(--hover);
}
.dlg-close:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
.dlg-close svg {
  width: 13px;
  height: 13px;
}

/* 表单 */
.dlg-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.field-label {
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2);
}
.field-input {
  height: 38px;
  padding: 0 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13.5px;
  font-weight: 400;
  outline: none;
  transition: border-color 0.2s ease;
}
.field-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.field-input:focus {
  border-color: var(--accent);
}

.dlg-text {
  margin: 4px 0 0;
  font-size: 13.5px;
  font-weight: 400;
  line-height: 1.8;
  color: var(--text-1);
  word-break: break-all;
}

.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
.ghost-btn {
  padding: 7px 16px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.ghost-btn:hover:not(:disabled) {
  color: var(--text-1);
  border-color: var(--border-strong);
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
.danger-btn {
  padding: 7px 18px;
  border-radius: var(--radius);
  border: 1px solid #cf4444;
  background: none;
  color: #cf4444;
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    background-color 0.2s ease,
    color 0.2s ease;
}
.danger-btn:hover:not(:disabled) {
  background: #cf4444;
  color: #ffffff;
}
.ghost-btn:disabled,
.accent-btn:disabled,
.danger-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

/* 弹窗动画：fade 200ms（遮罩 + 面板轻微上移） */
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

/* ================= 编辑 / 导入补充样式 ================= */
.head-text-btn {
  flex-shrink: 0;
  height: 26px;
  padding: 0 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
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
.head-text-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.dlg-edit {
  width: min(560px, 100%);
  max-height: calc(100vh - 48px);
  overflow-y: auto;
}
.field-textarea {
  width: 100%;
  min-height: 64px;
  padding: 9px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  line-height: 1.7;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.2s ease;
}
.field-textarea:focus {
  border-color: var(--accent);
  background: var(--surface);
}
.json-msg {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--accent-deep);
}
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
.switch.on {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.switch.on .switch-knob {
  transform: translateX(16px);
  background: var(--accent);
}

/* ================= 响应式：移动端单栏 ================= */
@media (max-width: 760px) {
  .topbar {
    padding: 12px 16px;
  }
  .rss-main {
    grid-template-columns: 1fr;
    gap: 16px;
    padding: 16px;
  }
  .source-col {
    position: static;
  }
  .source-list {
    max-height: 40vh;
  }
  .content-col {
    min-height: 50vh;
    padding: 14px 14px 18px;
  }
  .article-read {
    padding: 8px 4px 32px;
  }
  .source-item {
    min-height: 40px;
  }
  .dlg-overlay {
    padding: 16px;
  }
}
</style>
