<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  addBookGroup,
  addBookGroupMulti,
  deleteBook,
  deleteBooks,
  deleteBookGroup,
  getBookGroups,
  getBookshelf,
  refreshLocalBook,
  removeBookGroup,
  removeBookGroupMulti,
  saveBookGroup,
  saveBookGroupOrder,
  setBookGroups,
} from '@/api/bookshelf'
import {
  deleteBookmark,
  deleteBookmarks,
  getBookmarks,
  parseBookmarksJson,
  saveBookmark,
  saveBookmarks,
} from '@/api/bookmarks'
import { uploadLocalBook, importBookPreview } from '@/api/upload'
import { searchBookContent } from '@/api/cache'
import { exportBook, type ExportEncoding, type ExportFormat } from '@/api/export'
import { downloadBlob } from '@/utils/download'
import { canRescanBook } from '@/utils/localBook'
import { moveGroupTo } from '@/utils/groupOrder'
import { parseShelfView, shelfViewMetrics, type ShelfViewMode } from '@/utils/shelfView'
import { proxyImageUrl } from '@/utils/imageProxy'
import { useUserStore } from '@/stores/user'
import { probeSecureMode } from '@/api/users'
import { isNotImplemented } from '@/utils/errors'
import TopNav from '@/components/TopNav.vue'
import type { Book, BookGroup, Bookmark, ContentSearchHit, ImportPreview } from '@/types'

const router = useRouter()
const store = useUserStore()

/* ================= 网格密度（GAP 11：小/中/大——CSS 变量 --card-w，localStorage: reader_card_density） ================= */

type CardDensity = 's' | 'm' | 'l'
const DENSITY_OPTIONS: { value: CardDensity; label: string }[] = [
  { value: 's', label: '小' },
  { value: 'm', label: '中' },
  { value: 'l', label: '大' },
]
/** 桌面端卡片最小宽（宽屏） */
const DENSITY_MIN_W: Record<CardDensity, number> = { s: 128, m: 160, l: 204 }
/** 窄屏（<=720px）卡片最小宽 */
const DENSITY_NARROW_W: Record<CardDensity, number> = { s: 96, m: 120, l: 150 }
const DENSITY_KEY = 'reader_card_density'
const density = ref<CardDensity>('m')
{
  const raw = localStorage.getItem(DENSITY_KEY)
  if (raw === 's' || raw === 'm' || raw === 'l') density.value = raw
}
function setDensity(v: CardDensity) {
  density.value = v
  try {
    localStorage.setItem(DENSITY_KEY, v)
  } catch {
    /* ignore */
  }
}
const cardMinW = computed(() =>
  narrowMq.matches ? DENSITY_NARROW_W[density.value] : DENSITY_MIN_W[density.value],
)

/* ================= 视图切换（GAP 103：网格 / 列表 / 墙——localStorage: reader_shelf_view；
    wall 由 utils/shelfView.ts parseShelfView 统一解析，M4：接入三态切换） ================= */

const VIEW_KEY = 'reader_shelf_view'
const viewMode = ref<ShelfViewMode>('grid')
{
  viewMode.value = parseShelfView(localStorage.getItem(VIEW_KEY))
}
function setViewMode(v: ShelfViewMode) {
  viewMode.value = v
  try {
    localStorage.setItem(VIEW_KEY, v)
  } catch {
    /* ignore */
  }
}
/** 三态循环：网格 → 列表 → 墙 → 网格 */
const VIEW_CYCLE: ShelfViewMode[] = ['grid', 'list', 'wall']
const viewToggleTitle = computed(() => {
  const next = VIEW_CYCLE[(VIEW_CYCLE.indexOf(viewMode.value) + 1) % VIEW_CYCLE.length]
  if (next === 'wall') return '切换到墙视图（大封面网格）'
  if (next === 'list') return '切换到列表视图（小缩略图行）'
  return '切换到网格视图'
})
function cycleViewMode() {
  const i = VIEW_CYCLE.indexOf(viewMode.value)
  setViewMode(VIEW_CYCLE[(i + 1) % VIEW_CYCLE.length])
}
/** 墙视图卡片最小宽（shelfViewMetrics('wall')——固定大卡片，不受密度影响） */
const wallCardMinW = computed(() => shelfViewMetrics('wall', narrowMq.matches).cardMinW)
/** CSS 变量 --card-w 实际取值（wall 模式用墙尺寸；grid/list 用密度尺寸） */
const gridCardMinW = computed(() =>
  viewMode.value === 'wall' ? wallCardMinW.value : cardMinW.value,
)

/** 书卡进度角标（GAP 14）：durChapterIndex / totalChapterNum；无数据（缺字段/总章数 0）时返回 null 隐藏 */
function bookProgress(book: Book): number | null {
  const total = book.totalChapterNum
  const cur = book.durChapterIndex
  if (typeof total !== 'number' || total <= 0) return null
  if (typeof cur !== 'number' || cur < 0) return null
  return Math.min(100, Math.round((cur / total) * 100))
}

/** 未读更新数：读过则 total - (已读章序+1)，未读过则整本；无总数时隐藏 */
function unreadCount(book: Book): number | null {
  const total = book.totalChapterNum
  if (typeof total !== 'number' || total <= 0) return null
  const cur = book.durChapterIndex
  if (typeof cur === 'number' && cur >= 0) {
    const unread = total - (cur + 1)
    return unread > 0 ? unread : null
  }
  return total
}

/** 悬浮预览简介（桌面 hover 浮层：customIntro 优先，截取前 120 字；无简介返回 null 不显示浮层） */
function hoverPreview(book: Book): string | null {
  const intro = (book.customIntro || book.intro || '').trim()
  if (!intro) return null
  return intro.length > 120 ? `${intro.slice(0, 120)}…` : intro
}

const books = ref<Book[]>([])
const loading = ref(true)
const refreshing = ref(false)
/** 离线书架缓存（legacy helper.js 本地书架缓存：服务端不可达时展示最近一次数据） */
const OFFLINE_SHELF_KEY = 'reader_shelf_offline'
const offlineShelf = ref(false)

interface ShelfOfflineCache {
  books: Book[]
  groups: BookGroup[]
  ts: number
}

function saveOfflineShelf() {
  try {
    const data: ShelfOfflineCache = {
      books: books.value,
      groups: groups.value,
      ts: Date.now(),
    }
    localStorage.setItem(OFFLINE_SHELF_KEY, JSON.stringify(data))
  } catch {
    /* ignore */
  }
}

function loadOfflineShelf(): ShelfOfflineCache | null {
  try {
    const raw = JSON.parse(localStorage.getItem(OFFLINE_SHELF_KEY) ?? '') as unknown
    if (!raw || typeof raw !== 'object') return null
    const o = raw as Partial<ShelfOfflineCache>
    if (!Array.isArray(o.books) || !Array.isArray(o.groups)) return null
    return { books: o.books, groups: o.groups, ts: typeof o.ts === 'number' ? o.ts : 0 }
  } catch {
    return null
  }
}
const keyword = ref('')
watch(keyword, (k) => {
  if (searchMode.value === 'full') triggerContentSearch(k)
})

/** 主页搜索框 = 全网搜书入口（回车跳搜索页） */
function goSearchBooks() {
  const kw = keyword.value.trim()
  if (!kw) {
    void router.push('/search')
    return
  }
  void router.push(`/search?key=${encodeURIComponent(kw)}`)
}
const failedCovers = ref<Set<string>>(new Set())

/* ================= 多选模式 ================= */
const manageMode = ref(false)
const selected = ref<Set<string>>(new Set())
const manageBusy = ref(false)
const moveOpen = ref(false)

/* ================= 虚拟滚动（简易：视口可见行 + 上下缓冲 2 行） ================= */
const gridWrapRef = ref<HTMLElement | null>(null)
const narrowMq = window.matchMedia('(max-width: 720px)')
const cols = ref(1)
const rowH = ref(0)
const rowGap = ref(32)
const startRow = ref(0)
const endRow = ref(1)
let lastWrapW = 0
let viewportRaf: number | undefined
let wrapObserver: ResizeObserver | undefined

/* ================= 搜索范围：书名 / 全书 ================= */
/** 'name'=书名/作者（本地）；'full'=全书内容搜索（GET /reader3/searchBookContent——
 *  逐本地书并发聚合命中，结果面板展示书 + 章节命中，点击跳阅读器该章；
 *  网格仍按书名/作者/简介匹配兜底） */
const searchMode = ref<'name' | 'full'>('name')


/* ================= 全书内容搜索（GET /reader3/searchBookContent——逐本地书并发聚合） ================= */
const contentResults = ref<{ book: Book; hits: ContentSearchHit[] }[]>([])
const contentSearching = ref(false)
const contentSearchDone = ref(false)
let contentSearchTimer: number | undefined
let contentSearchSeq = 0
function triggerContentSearch(kw: string) {
  if (contentSearchTimer !== undefined) window.clearTimeout(contentSearchTimer)
  const seq = ++contentSearchSeq
  if (!kw.trim()) {
    contentResults.value = []
    contentSearching.value = false
    contentSearchDone.value = false
    return
  }
  contentSearchTimer = window.setTimeout(async () => {
    if (seq !== contentSearchSeq) return
    contentSearching.value = true
    contentSearchDone.value = false
    const localBooks = books.value.filter(
      (b) => b.origin === 'loc_book' || b.origin === 'local' || (b.bookUrl ?? '').startsWith('local://'),
    )
    const agg: { book: Book; hits: ContentSearchHit[] }[] = []
    await Promise.all(
      localBooks.map(async (b) => {
        try {
          const res = await searchBookContent(kw, b.bookUrl)
          const hits = (res.data ?? []).slice(0, 10)
          if (hits.length) agg.push({ book: b, hits })
        } catch {
          /* 单书失败跳过 */
        }
      }),
    )
    if (seq !== contentSearchSeq) return
    contentResults.value = agg.sort((a, b) => b.hits.length - a.hits.length)
    contentSearching.value = false
    contentSearchDone.value = true
  }, 600)
}
watch(searchMode, (m) => {
  if (m !== 'full') {
    contentResults.value = []
    contentSearchDone.value = false
  }
})

/** 点击命中 → 阅读器定位该章（与阅读页 ?chapter 语义一致） */
function goContentHit(book: Book, hit: ContentSearchHit) {
  void router.push(`/reader/${encodeURIComponent(book.bookUrl)}?chapter=${hit.chapterIndex}`)
}

/** 命中总数（面板标题：共 N 书 · M 章命中） */
const contentHitTotal = computed(() =>
  contentResults.value.reduce((n, r) => n + r.hits.length, 0),
)

/* ================= 书架排序（前端排序 books.value 副本——不改服务端顺序；localStorage: reader_shelf_sort） ================= */

type SortMode = 'recent' | 'added' | 'name' | 'author' | 'source' | 'group'
const SORT_OPTIONS: { value: SortMode; label: string }[] = [
  { value: 'recent', label: '最近阅读' },
  { value: 'added', label: '最近添加' },
  { value: 'name', label: '书名' },
  { value: 'author', label: '作者' },
  { value: 'source', label: '来源' },
  { value: 'group', label: '分组' },
]
const sortMode = ref<SortMode>('recent')
{
  const raw = localStorage.getItem('reader_shelf_sort')
  if (SORT_OPTIONS.some((o) => o.value === raw)) sortMode.value = raw as SortMode
}
watch(sortMode, (v) => {
  try {
    localStorage.setItem('reader_shelf_sort', v)
  } catch {
    /* ignore */
  }
})

/* ================= 分组折叠（排序=分组 的分组模式下：分组标题点击折叠；localStorage: reader_group_collapsed {groupId: bool}） ================= */

const GROUP_COLLAPSE_KEY = 'reader_group_collapsed'
const collapsedGroups = ref<Record<number, boolean>>({})
{
  try {
    const raw = JSON.parse(localStorage.getItem(GROUP_COLLAPSE_KEY) ?? '{}') as Record<string, unknown>
    if (raw && typeof raw === 'object') {
      for (const [k, v] of Object.entries(raw)) {
        const id = Number(k)
        if (Number.isFinite(id) && typeof v === 'boolean') collapsedGroups.value[id] = v
      }
    }
  } catch {
    /* ignore */
  }
}

function groupCollapsed(id: number): boolean {
  return !!collapsedGroups.value[id]
}

function toggleGroupCollapsed(id: number) {
  collapsedGroups.value = { ...collapsedGroups.value, [id]: !collapsedGroups.value[id] }
  try {
    localStorage.setItem(GROUP_COLLAPSE_KEY, JSON.stringify(collapsedGroups.value))
  } catch {
    /* ignore */
  }
}

/* ================= 书架分组 ================= */
const groups = ref<BookGroup[]>([])
const activeGroup = ref<number | null>(null) // null=全部
const groupOpen = ref(false)
const groupDialogRef = ref<HTMLElement | null>(null)
const newGroupName = ref('')
const groupSaving = ref(false)

/* ================= GAP 12：书架统计（顶部「共 N 本 · M 组」） ================= */
const shelfSummary = computed(() => ({
  books: books.value.length,
  groups: groups.value.length,
}))

/* ================= 分组管理：重命名 / 删除 ================= */
const renamingId = ref<number | null>(null)
const renameName = ref('')
const renameBusy = ref(false)

/** 分组内书数：优先用后端契约 bookCount，未返回时本地统计兜底 */
function groupCount(id: number): number {
  const g = groups.value.find((x) => x.id === id)
  if (g && typeof g.bookCount === 'number' && g.bookCount >= 0) return g.bookCount
  return books.value.filter((b) => inGroup(b, id)).length
}

/** 本地书分组变动后失效 API bookCount，回落本地统计（避免过期计数） */
function invalidateGroupCounts() {
  groups.value.forEach((g) => {
    g.bookCount = undefined
  })
}

/** 书籍多分组 ID 列表（groupIds 优先；旧单值 group 兜底） */
function bookGroupIds(book: Book): number[] {
  // 兼容后端可能返回的 JSON 字符串 / 逗号分隔文本（旧迁移数据）
  let raw: unknown = (book as { groupIds?: number[] | string }).groupIds
  if (typeof raw === 'string') {
    const text = raw
    try {
      const parsed: unknown = JSON.parse(text)
      raw = Array.isArray(parsed) ? parsed : text.split(/[,，、]/).map(Number)
    } catch {
      raw = text.split(/[,，、]/).map(Number)
    }
  }
  const ids = Array.isArray(raw)
    ? raw.filter((x): x is number => typeof x === 'number' && Number.isFinite(x) && x > 0)
    : book.group > 0
      ? [book.group]
      : []
  return Array.from(new Set(ids)).sort((a, b) => a - b)
}

/** 书籍是否属于分组（0 = 未分组：无任何多分组） */
function inGroup(book: Book, gid: number): boolean {
  return gid === 0 ? bookGroupIds(book).length === 0 : bookGroupIds(book).includes(gid)
}

/** 本地同步多分组（groupIds + 主分组 group 双字段一致） */
function setBookGroupIdsLocal(book: Book, ids: number[]) {
  const uniq = Array.from(new Set(ids.filter((x) => typeof x === 'number' && x > 0))).sort(
    (a, b) => a - b,
  )
  book.groupIds = uniq
  book.group = uniq[0] ?? 0
}

/** 可见分组（show=false 的隐藏分组不出现在分组栏筛选） */
const visibleGroups = computed(() => groups.value.filter((g) => g.show !== false))

/* ================= 书卡菜单（右键 / 长按 / hover ⋯） ================= */
const menuBook = ref<Book | null>(null)
const menuPos = ref({ x: 0, y: 0 })
const menuOpen = ref(false)
const menuBusy = ref(false)
const confirmRemoveOpen = ref(false)
const removeTarget = ref<Book | null>(null)
const bookGroupPanelOpen = ref(false)
const bookGroupPanelIds = ref<number[]>([])
let longPressTimer: number | undefined
let longPressFired = false
let suppressClick = false

/* ================= 导入本地书 ================= */
interface ImportItem {
  file: File
  status: 'pending' | 'uploading' | 'done' | 'error'
  progress: number
  error?: string
  /** 导入预览（POST /reader3/importBookPreview；undefined=探测中 / null=未实现或失败 → 直接上传） */
  preview?: ImportPreview | null
}

const importOpen = ref(false)
const dialogRef = ref<HTMLElement | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const isDragOver = ref(false)
const uploadBusy = ref(false)
const uploadIndex = ref(0)
const importDone = ref(false)
const importSummary = ref('')
const acceptTip = ref('')
const importItems = ref<ImportItem[]>([])

/** 整体进度：按文件大小加权 */
const totalProgress = computed(() => {
  const items = importItems.value
  if (!items.length) return 0
  const totalSize = items.reduce((s, it) => s + it.file.size, 0) || 1
  const loaded = items.reduce((s, it) => s + (it.file.size * it.progress) / 100, 0)
  return Math.min(99, Math.round((loaded / totalSize) * 100))
})
const hasPending = computed(() => importItems.value.some((it) => it.status === 'pending'))
const hasPendingCount = computed(() => importItems.value.filter((it) => it.status === 'pending').length)
const failedCount = computed(() => importItems.value.filter((it) => it.status === 'error').length)

/* ================= 导入预览（POST /reader3/importBookPreview：选文件后先探测；404/未实现 → 直接上传） ================= */

/** 是否任一文件拿到预览数据（后端实现判定） */
const previewSupported = computed(() => importItems.value.some((it) => it.preview != null))
/** 是否仍有文件在探测预览中 */
const previewChecking = computed(() => importItems.value.some((it) => it.preview === undefined))
/** 拿到预览数据的文件（用于弹窗展示） */
const previewedItems = computed(() => importItems.value.filter((it) => it.preview != null))

/** 预览章节标题（兼容 chapters / chapterList 命名与字符串项，截取前 5 章） */
function previewChapters(item: ImportItem): string[] {
  const p = item.preview
  if (!p) return []
  const raw = Array.isArray(p.chapters) ? p.chapters : p.chapterList
  if (!Array.isArray(raw)) return []
  return raw
    .map((c) => (typeof c === 'string' ? c : (c?.title ?? '')))
    .filter((t) => !!t)
    .slice(0, 5)
}

function previewChapterCount(item: ImportItem): number {
  const p = item.preview
  if (!p) return 0
  if (typeof p.chapterCount === 'number' && p.chapterCount >= 0) return p.chapterCount
  const raw = Array.isArray(p.chapters) ? p.chapters : p.chapterList
  return Array.isArray(raw) ? raw.length : 0
}

/** 单文件导入预览：成功 → 预览数据（弹窗展示，确认后仍走 uploadLocalBook）；404/未实现/失败 → preview=null 直接上传 */
async function checkPreview(item: ImportItem) {
  try {
    const res = await importBookPreview(item.file)
    item.preview = res.data ?? null
  } catch {
    item.preview = null
  }
}

function fmtSize(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

function isSupported(file: File): boolean {
  const name = file.name.toLowerCase()
  return (
    name.endsWith('.epub') ||
    name.endsWith('.txt') ||
    name.endsWith('.mobi') ||
    name.endsWith('.azw3') ||
    name.endsWith('.pdf') ||
    name.endsWith('.fb2') ||
    name.endsWith('.docx') ||
    file.type === 'application/epub+zip' ||
    file.type === 'text/plain' ||
    file.type.startsWith('text/')
  )
}

function openImport() {
  importOpen.value = true
  uploadBusy.value = false
  importDone.value = false
  importSummary.value = ''
  acceptTip.value = ''
  importItems.value = []
  document.body.style.overflow = 'hidden'
  void nextTick(() => dialogRef.value?.focus())
}

function closeImport() {
  if (uploadBusy.value) return
  importOpen.value = false
  document.body.style.overflow = ''
}

function addFiles(files: File[]) {
  if (uploadBusy.value) return
  const valid = files.filter(isSupported)
  const ignored = files.length - valid.length
  const added: ImportItem[] = []
  for (const f of valid) {
    const item: ImportItem = { file: f, status: 'pending', progress: 0, preview: undefined }
    importItems.value.push(item)
    added.push(item)
  }
  acceptTip.value = ignored > 0 ? `已忽略 ${ignored} 个不支持的文件（支持 .epub / .txt / .mobi / .azw3 / .pdf / .fb2 / .docx）` : ''
  if (valid.length > 0) {
    importDone.value = false
    importSummary.value = ''
  }
  // 导入预览（后端 /reader3/importBookPreview；404/未实现 → 直接上传降级）
  for (const item of added) void checkPreview(item)
}

function onPick(e: Event) {
  const input = e.target as HTMLInputElement
  addFiles(Array.from(input.files ?? []))
  input.value = '' // 清空以便重复选择同一文件
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = true
}

function onDragLeave(e: DragEvent) {
  const cur = e.currentTarget as HTMLElement | null
  if (!cur || !cur.contains(e.relatedTarget as Node | null)) isDragOver.value = false
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = false
  if (uploadBusy.value) return
  addFiles(Array.from(e.dataTransfer?.files ?? []))
}

function removeItem(i: number) {
  if (uploadBusy.value) return
  importItems.value.splice(i, 1)
}

/**
 * GAP 126：导入去重——上传前 getBookshelf 检查同名书，命中则弹确认窗；
 * 返回 true=继续导入 / false=用户取消。书架拉取失败不阻塞导入。
 */
async function checkImportDuplicates(): Promise<boolean> {
  let shelf: Book[] = books.value
  try {
    const res = await getBookshelf()
    shelf = res.data ?? []
  } catch {
    return true // 拉取失败不阻塞导入（错误提示由拦截器处理）
  }
  const shelfNames = new Set(shelf.map((b) => b.name.trim()).filter(Boolean))
  const dups = new Set<string>()
  for (const item of importItems.value) {
    const name = (item.preview?.name || item.file.name.replace(/\.[^.]+$/, '')).trim()
    if (name && shelfNames.has(name)) dups.add(name)
  }
  if (dups.size === 0) return true
  const names = Array.from(dups).slice(0, 5)
  const more = dups.size > 5 ? ` 等 ${dups.size} 本` : ''
  try {
    await ElMessageBox.confirm(
      `书架中已有同名书：${names.join('、')}${more}。继续导入将新增重复条目，仍要导入吗？`,
      '检测到同名书籍',
      { confirmButtonText: '仍要导入', cancelButtonText: '取消', type: 'warning' },
    )
    return true
  } catch {
    return false // 用户取消
  }
}

/** 逐个上传（每个文件一次 multipart POST），完成后自动刷新书架 */
async function startUpload() {
  if (uploadBusy.value || importItems.value.length === 0) return
  // GAP 126：同名书确认弹窗（取消则中止本次导入）
  if (!(await checkImportDuplicates())) {
    ElMessage.info('已取消导入')
    return
  }
  uploadBusy.value = true
  importDone.value = false
  let ok = 0
  for (let i = 0; i < importItems.value.length; i++) {
    const item = importItems.value[i]
    uploadIndex.value = i
    item.status = 'uploading'
    item.progress = 0
    try {
      await uploadLocalBook(item.file, (p) => (item.progress = p))
      item.status = 'done'
      item.progress = 100
      ok++
    } catch (err) {
      item.status = 'error'
      item.error = err instanceof Error ? err.message : '导入失败'
    }
  }
  uploadBusy.value = false
  importDone.value = true
  const failed = importItems.value.length - ok
  importSummary.value =
    failed > 0 ? `导入完成：${ok} 本成功，${failed} 本失败` : `导入完成，共 ${ok} 本`
  await load() // 刷新书架（getBookshelf）
  if (failed === 0) window.setTimeout(() => closeImport(), 800)
}

/* ================= OPDS 服务器入口（外部阅读器：legado/静读等） ================= */
const opdsOpen = ref(false)
const opdsCopied = ref(false)

/** OPDS 地址 = 当前 host + /opds（secure 模式附带 accessToken=用户名:token） */
const opdsUrl = computed(() => {
  const base = `${window.location.origin}/opds`
  return store.accessToken ? `${base}?accessToken=${encodeURIComponent(store.accessToken)}` : base
})

function openOpds() {
  opdsOpen.value = true
  opdsCopied.value = false
  document.body.style.overflow = 'hidden'
}

function closeOpds() {
  opdsOpen.value = false
  document.body.style.overflow = ''
}

async function copyOpdsUrl() {
  const text = opdsUrl.value
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    // 剪贴板 API 不可用（非 https 等）：textarea 降级
    const ta = document.createElement('textarea')
    ta.value = text
    ta.style.position = 'fixed'
    ta.style.opacity = '0'
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
  }
  opdsCopied.value = true
  window.setTimeout(() => (opdsCopied.value = false), 1600)
}

onBeforeUnmount(() => {
  if (longPressTimer) clearTimeout(longPressTimer)
  wrapObserver?.disconnect()
  window.removeEventListener('scroll', onWindowScroll)
  window.removeEventListener('resize', onWindowResize)
  // GAP 86：下拉刷新监听清理
  window.removeEventListener('touchstart', onPullTouchStart)
  window.removeEventListener('touchmove', onPullTouchMove)
  window.removeEventListener('touchend', onPullTouchEnd)
  // GAP 70：全局快捷键监听清理
  window.removeEventListener('keydown', onGlobalKeydown)
  if (viewportRaf !== undefined) cancelAnimationFrame(viewportRaf)
  document.body.style.overflow = ''
})

/** 封面占位 = 莫兰迪低饱和纯色块（按书名 hash 取色） */
const MORANDI = [
  '#9aa8a0', // 鼠尾草绿
  '#a5a0b0', // 雾紫灰
  '#b0a59a', // 暖沙
  '#9aa8b5', // 雾蓝灰
  '#b0a0a5', // 灰玫瑰
  '#a3a79a', // 橄榄灰
  '#a89fb0', // 藕荷灰
  '#b5a89c', // 陶土灰
]

function hashName(name: string): number {
  let h = 0
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0
  return h
}

function coverColor(name: string): string {
  return MORANDI[hashName(name) % MORANDI.length]
}

function coverSrc(book: Book): string | null {
  const src = book.customCoverUrl || book.coverUrl || null
  return src ? proxyImageUrl(resolveCoverUrl(src)) ?? null : null
}

/** 自定义封面走 file/download 内联流（GAP 19）：展示时补当前 accessToken（重新登录后仍可显示） */
function resolveCoverUrl(url: string): string {
  if (!url.startsWith('/reader3/file/')) return url
  const token = store.accessToken
  if (!token || url.includes('accessToken=')) return url
  return `${url}${url.includes('?') ? '&' : '?'}accessToken=${encodeURIComponent(token)}`
}

function hasCover(book: Book): boolean {
  const src = coverSrc(book)
  return !!src && !failedCovers.value.has(book.bookUrl)
}

function onCoverError(book: Book) {
  failedCovers.value.add(book.bookUrl)
}

function coverInitial(name: string): string {
  const ch = name.trim().charAt(0)
  return ch ? ch.toUpperCase() : '书'
}

/* ================= GAP 146：书架置顶（localStorage: reader_pinned {url: ts}——排序优先置顶） ================= */

const PINNED_KEY = 'reader_pinned'
const pinned = ref<Record<string, number>>({})
{
  try {
    const raw = JSON.parse(localStorage.getItem(PINNED_KEY) ?? '{}') as Record<string, unknown>
    if (raw && typeof raw === 'object') {
      for (const [k, v] of Object.entries(raw)) {
        if (typeof v === 'number' && Number.isFinite(v)) pinned.value[k] = v
      }
    }
  } catch {
    /* ignore */
  }
}
function persistPinned() {
  try {
    localStorage.setItem(PINNED_KEY, JSON.stringify(pinned.value))
  } catch {
    /* ignore */
  }
}
const isPinned = (book: Book) => !!pinned.value[book.bookUrl]

/** 书卡菜单「置顶/取消置顶」（长按菜单 / 右键 / ⋯） */
function togglePin() {
  const book = menuBook.value
  if (!book) return
  if (pinned.value[book.bookUrl]) {
    const next = { ...pinned.value }
    delete next[book.bookUrl]
    pinned.value = next
  } else {
    pinned.value = { ...pinned.value, [book.bookUrl]: Date.now() }
  }
  persistPinned()
  ElMessage.success(isPinned(book) ? '已置顶（排序优先）' : '已取消置顶')
  closeMenu()
}

/* ================= GAP 78：重新扫描本地书（POST /reader3/refreshLocalBook） ================= */

const rescanBusy = ref(false)

/** 书卡菜单「重新扫描」：重解析本地书原文件（local:// 双轨书 / loc_book 文件书）→ 刷新书架 */
async function rescanBook() {
  const book = menuBook.value
  if (!book || rescanBusy.value) return
  rescanBusy.value = true
  closeMenu()
  try {
    const res = await refreshLocalBook(book.bookUrl)
    const count = res.data?.totalChapterNum ?? res.data?.chapterCount
    ElMessage.success(
      count !== undefined
        ? `已重新扫描「${book.name}」（${count} 章）`
        : `已重新扫描「${book.name}」`,
    )
    await load(true)
  } catch (err) {
    const e = err as { response?: { status?: number }; message?: string } | null | undefined
    if (e?.response?.status === 404 || e?.response?.status === 501) {
      ElMessage.warning('重新扫描接口后端暂未提供（POST /reader3/refreshLocalBook）')
    } else {
      ElMessage.error(`重新扫描失败：${err instanceof Error ? err.message : '请稍后重试'}`)
    }
  } finally {
    rescanBusy.value = false
  }
}

const filtered = computed(() => {
  const kw = keyword.value.trim().toLowerCase()
  const gid = activeGroup.value
  const list = books.value.filter((b) => {
    if (gid !== null && !inGroup(b, gid)) return false
    if (!kw) return true
    if (searchMode.value === 'full') {
      // 全书模式：本地书正文内容搜索（并发逐本）——书名/作者匹配兜底
      return (
        b.name.toLowerCase().includes(kw) ||
        b.author.toLowerCase().includes(kw) ||
        (b.intro ?? '').toLowerCase().includes(kw)
      )
    }
    return b.name.toLowerCase().includes(kw) || b.author.toLowerCase().includes(kw)
  })
  // 前端排序 books 副本（不改变服务端 books.value 顺序）
  const sorted = [...list]
  switch (sortMode.value) {
    case 'added':
      // 最近添加：优先按 rowid（入库序）倒序；后端 getBookshelf 未透出 rowid/created_at 时
      // 保持服务端列表顺序（list_books 按 dur_chapter_time DESC, rowid DESC），标注「按入库顺序」
      if (list.some((b) => typeof (b as { rowid?: unknown }).rowid === 'number')) {
        sorted.sort(
          (a, b) => ((b as { rowid?: number }).rowid ?? 0) - ((a as { rowid?: number }).rowid ?? 0),
        )
      }
      break
    case 'name':
      sorted.sort((a, b) => a.name.localeCompare(b.name, 'zh') || a.author.localeCompare(b.author, 'zh'))
      break
    case 'author':
      sorted.sort((a, b) => a.author.localeCompare(b.author, 'zh') || a.name.localeCompare(b.name, 'zh'))
      break
    case 'source':
      sorted.sort(
        (a, b) =>
          (a.originName || a.origin || '').localeCompare(b.originName || b.origin || '', 'zh') ||
          a.name.localeCompare(b.name, 'zh'),
      )
      break
    case 'group':
      sorted.sort(
        (a, b) =>
          (bookGroupIds(a)[0] ?? 0) - (bookGroupIds(b)[0] ?? 0) ||
          a.name.localeCompare(b.name, 'zh'),
      )
      break
    default:
      // 最近阅读：服务端进度时间优先，其次最新章节时间
      sorted.sort(
        (a, b) =>
          (b.durChapterTime ?? 0) - (a.durChapterTime ?? 0) ||
          (b.latestChapterTime ?? 0) - (a.latestChapterTime ?? 0),
      )
  }
  // GAP 146：置顶优先（置顶时间倒序；无置顶书时保持原排序；sort 稳定故未置顶书相对顺序不变）
  if (Object.keys(pinned.value).length > 0) {
    const pinTs = (b: Book) => pinned.value[b.bookUrl] ?? 0
    sorted.sort((a, b) => pinTs(b) - pinTs(a))
  }
  return sorted
})

const emptyText = computed(() => {
  if (keyword.value) return '没有找到匹配的书籍'
  if (activeGroup.value !== null) return '该分组下暂无书籍'
  return '书架空空如也，去搜索添加第一本书吧'
})

/* ================= GAP 199：最近添加标注（后端未透出 rowid/created_at → 按服务端列表顺序近似） ================= */
const addedSortNote = computed(() => {
  if (sortMode.value !== 'added') return ''
  return books.value.some((b) => typeof (b as { rowid?: unknown }).rowid === 'number')
    ? '按入库时间'
    : '按入库顺序'
})
const addedSortTip = computed(() =>
  sortMode.value === 'added' &&
  !books.value.some((b) => typeof (b as { rowid?: unknown }).rowid === 'number')
    ? '后端 books 表未透出 created_at/rowid：按服务端当前列表顺序近似'
    : '',
)

/* ================= 多选模式 ================= */
function toggleManage() {
  manageMode.value = !manageMode.value
  selected.value = new Set()
  moveOpen.value = false
}

function toggleSelect(book: Book) {
  const s = new Set(selected.value)
  if (s.has(book.bookUrl)) s.delete(book.bookUrl)
  else s.add(book.bookUrl)
  selected.value = s
}

/** 多选模式下右键也视为点选，不弹书卡菜单 */
function onCardMenu(book: Book, e: MouseEvent) {
  if (manageMode.value) {
    e.preventDefault()
    toggleSelect(book)
    return
  }
  openCardMenu(book, e)
}

/** 批量删除：优先 POST /reader3/deleteBooks（批量契约 {bookUrls}），后端未实现时降级逐本 deleteBook */
async function bulkRemove() {
  const urls = Array.from(selected.value)
  if (!urls.length || manageBusy.value) return
  try {
    await ElMessageBox.confirm(`确定将选中的 ${urls.length} 本书从书架删除吗？`, '删除书籍', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    })
  } catch {
    return // 用户取消
  }
  manageBusy.value = true
  let ok = 0
  const removed = new Set<string>()
  try {
    const res = await deleteBooks(urls, { silent: true })
    ok = typeof res.data?.count === 'number' ? res.data.count : urls.length
    urls.forEach((u) => removed.add(u))
  } catch {
    // 批量接口未就绪（404）/失败：降级逐本 deleteBook（单本失败不中断）
    for (const url of urls) {
      try {
        await deleteBook(url)
        ok++
        removed.add(url)
      } catch {
        // 单本失败继续
      }
    }
  }
  if (removed.size) books.value = books.value.filter((b) => !removed.has(b.bookUrl))
  selected.value = new Set()
  manageBusy.value = false
  const failed = urls.length - ok
  ElMessage.success(failed > 0 ? `已删除 ${ok} 本，${failed} 本失败` : `已删除 ${ok} 本书`)
}

/* ================= 导出（GET /reader3/exportBook：txt/epub/html blob 下载 + txt 编码选择） ================= */

const EXPORT_FORMATS: { value: ExportFormat; label: string; tip: string }[] = [
  { value: 'txt', label: 'TXT', tip: '纯文本' },
  { value: 'epub', label: 'EPUB', tip: '电子书' },
  { value: 'html', label: 'HTML', tip: '网页' },
]

const exportOpen = ref(false)
const exportBookUrl = ref('')
const exportName = ref('')
const exportFormat = ref<ExportFormat>('txt')
/** GAP 144：txt 导出编码（UTF-8 / GBK——后端并行实现中，未就绪时仍输出 UTF-8） */
const exportEncoding = ref<ExportEncoding>('utf-8')
const exportBusy = ref(false)
const exportMsg = ref('')
const exportMsgError = ref(false)

function openExportFor(book: Book) {
  exportBookUrl.value = book.bookUrl
  exportName.value = book.name || 'book'
  exportFormat.value = 'txt'
  exportEncoding.value = 'utf-8'
  exportMsg.value = ''
  exportMsgError.value = false
  closeMenu()
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
      exportBookUrl.value,
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
    const name = `${exportName.value.replace(/[\\/:*?"<>|]/g, '_')}.${exportFormat.value}`
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

function openMovePanel() {
  if (!selected.value.size || manageBusy.value) return
  resetMoveSelections()
  moveOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeMovePanel() {
  if (manageBusy.value) return
  moveOpen.value = false
  document.body.style.overflow = ''
}

/* ================= 批量加入/移除分组（多分组语义：保留原有分组，只增删勾选的分组） ================= */

const moveAddIds = ref<number[]>([])
const moveRemoveIds = ref<number[]>([])
const moveClearAll = ref(false)

function resetMoveSelections() {
  moveAddIds.value = []
  moveRemoveIds.value = []
  moveClearAll.value = false
}

function toggleMoveAdd(gid: number) {
  moveAddIds.value = moveAddIds.value.includes(gid)
    ? moveAddIds.value.filter((x) => x !== gid)
    : [...moveAddIds.value, gid]
  if (moveAddIds.value.includes(gid)) {
    moveRemoveIds.value = moveRemoveIds.value.filter((x) => x !== gid)
  }
}

function toggleMoveRemove(gid: number) {
  moveRemoveIds.value = moveRemoveIds.value.includes(gid)
    ? moveRemoveIds.value.filter((x) => x !== gid)
    : [...moveRemoveIds.value, gid]
  if (moveRemoveIds.value.includes(gid)) {
    moveAddIds.value = moveAddIds.value.filter((x) => x !== gid)
  }
}

/** 批量更新分组：先批量加入勾选组，再批量移除勾选组/清空全部；接口未就绪时逐本降级 */
async function performMove() {
  const urls = Array.from(selected.value)
  if (!urls.length || manageBusy.value) return
  if (!moveAddIds.value.length && !moveRemoveIds.value.length && !moveClearAll.value) {
    ElMessage.warning('请先选择要加入或移除的分组')
    return
  }
  manageBusy.value = true
  let joined = 0
  let removed = 0
  try {
    for (const gid of moveAddIds.value) {
      try {
        const res = await addBookGroupMulti(urls, gid, { silent: true })
        joined += typeof res.data?.count === 'number' ? res.data.count : 0
      } catch {
        for (const url of urls) {
          try {
            await addBookGroup(url, gid)
            joined++
          } catch {
            /* 单本失败继续 */
          }
        }
      }
    }
    if (moveClearAll.value) {
      try {
        await removeBookGroupMulti(urls, undefined, { silent: true })
        removed = urls.length
      } catch {
        for (const url of urls) {
          try {
            await setBookGroups(url, [])
            removed++
          } catch {
            /* 单本失败继续 */
          }
        }
      }
    } else {
      for (const gid of moveRemoveIds.value) {
        try {
          const res = await removeBookGroupMulti(urls, gid, { silent: true })
          removed += typeof res.data?.count === 'number' ? res.data.count : 0
        } catch {
          for (const url of urls) {
            try {
              await removeBookGroup(url, gid)
              removed++
            } catch {
              /* 单本失败继续 */
            }
          }
        }
      }
    }
  } finally {
    for (const url of urls) {
      const b = books.value.find((x) => x.bookUrl === url)
      if (!b) continue
      let ids = bookGroupIds(b)
      for (const gid of moveAddIds.value) {
        if (!ids.includes(gid)) ids.push(gid)
      }
      if (moveClearAll.value) ids = []
      else for (const gid of moveRemoveIds.value) ids = ids.filter((x) => x !== gid)
      setBookGroupIdsLocal(b, ids)
    }
    invalidateGroupCounts()
    selected.value = new Set()
    manageBusy.value = false
    moveOpen.value = false
    resetMoveSelections()
    document.body.style.overflow = ''
  }
  const parts: string[] = []
  if (joined > 0) parts.push(`加入 ${joined} 本`)
  if (removed > 0) parts.push(`移出 ${removed} 本`)
  ElMessage.success(parts.length ? `已更新分组（${parts.join(' · ')}）` : '分组未变化')
}

/* ================= 虚拟滚动（行列表：分组模式含分组标题行；折叠组只留标题行，计数随之调整） ================= */

/** 分组标题行高（书行间距由 rowGap 提供，标题行同样追加） */
const HEADER_ROW_H = 44

type ShelfRow =
  | { kind: 'books'; books: Book[] }
  | { kind: 'header'; groupId: number; name: string; count: number }

function rowStride(row: ShelfRow): number {
  return (row.kind === 'header' ? HEADER_ROW_H : rowH.value) + rowGap.value
}

/** 行列表：排序=分组 时按分组分节（未分组在前，其余按 orderNum/order）；折叠组不产出书行 */
const gridRows = computed<ShelfRow[]>(() => {
  const list = filtered.value
  if (list.length === 0) return []
  const c = Math.max(1, cols.value)
  const chunk = (books: Book[]): ShelfRow[] => {
    const rows: ShelfRow[] = []
    for (let i = 0; i < books.length; i += c) rows.push({ kind: 'books', books: books.slice(i, i + c) })
    return rows
  }
  if (sortMode.value !== 'group') return chunk(list)
  const gs = [...groups.value].sort(
    (a, b) => (a.orderNum ?? a.order ?? a.id) - (b.orderNum ?? b.order ?? b.id),
  )
  const sections: { id: number; name: string; books: Book[] }[] = []
  const ungrouped = list.filter((b) => bookGroupIds(b).length === 0)
  if (ungrouped.length) sections.push({ id: 0, name: '未分组', books: ungrouped })
  for (const g of gs) {
    const bs = list.filter((b) => inGroup(b, g.id))
    if (bs.length) sections.push({ id: g.id, name: g.name, books: bs })
  }
  const rows: ShelfRow[] = []
  for (const sec of sections) {
    rows.push({ kind: 'header', groupId: sec.id, name: sec.name, count: sec.books.length })
    if (!groupCollapsed(sec.id)) rows.push(...chunk(sec.books))
  }
  return rows
})

/** 行前缀高度（分组标题行与书行高度不同，按行累计精确撑高） */
const rowOffsets = computed(() => {
  const rows = gridRows.value
  const offs = new Array<number>(rows.length + 1)
  offs[0] = 0
  for (let i = 0; i < rows.length; i++) offs[i + 1] = offs[i] + rowStride(rows[i])
  return offs
})

const totalRows = computed(() => gridRows.value.length)
const visibleRows = computed(() => gridRows.value.slice(startRow.value, endRow.value))
const padTop = computed(() => rowOffsets.value[startRow.value] ?? 0)
const padBottom = computed(() => {
  const offs = rowOffsets.value
  return (offs[offs.length - 1] ?? 0) - (offs[endRow.value] ?? 0)
})

function rowKey(row: ShelfRow): string {
  if (row.kind === 'header') return `h-${row.groupId}`
  return `r-${row.books.map((b) => b.bookUrl).join('|')}`
}

/** 按视口高度计算可见行（上下各缓冲 2 行），滚动时更新 */
function updateViewport() {
  const wrap = gridWrapRef.value
  if (!wrap) return
  const total = totalRows.value
  if (total <= 0) return
  if (rowH.value <= 0) {
    // 尚未测得行高：先渲染首行作为测量锚点
    startRow.value = 0
    endRow.value = Math.min(total, 2)
    return
  }
  const offs = rowOffsets.value
  const gridTop = wrap.getBoundingClientRect().top + window.scrollY
  const st = Math.max(0, window.scrollY - gridTop)
  const vh = window.innerHeight
  // 按累计行高定位首行（行高不一：分组标题行 / 书行）
  let r0 = 0
  for (let i = 0; i < total; i++) {
    if (offs[i + 1] > st) {
      r0 = i
      break
    }
    r0 = i
  }
  r0 = Math.max(0, r0 - 2)
  let r1 = r0
  for (let i = r0; i < total; i++) {
    if (offs[i + 1] - offs[r0] > vh) {
      r1 = i
      break
    }
    r1 = i
  }
  r1 = Math.min(total, r1 + 3)
  startRow.value = r0
  // endRow 为「结束索引（不含）」：slice(start, end) 要包含最后满足的行 r1——必须 r1+1，
  // 否则滚动到底部时最后一行（r1 = total-1）被 slice 切掉不渲染（bug：最下行整排不显示）
  endRow.value = Math.max(r0 + 1, Math.min(total, r1 + 1))
}

/** 计算列数并测量行高（卡片宽高比固定，任取一张测量）；
 *  M4：尺寸统一取自 shelfViewMetrics——墙模式（wall）大卡片 + 宽间距，行高按墙元信息区 */
function measureGrid() {
  const wrap = gridWrapRef.value
  if (!wrap) return
  const w = wrap.clientWidth
  if (w <= 0) return
  const m = shelfViewMetrics(viewMode.value, narrowMq.matches, density.value)
  const minW = m.cardMinW
  const colGap = m.colGap
  rowGap.value = m.rowGap
  cols.value = Math.max(1, Math.floor((w + colGap) / (minW + colGap)))
  const card = wrap.querySelector('.book-card')
  if (card) {
    rowH.value = Math.round(card.getBoundingClientRect().height)
  } else if (viewMode.value === 'list') {
    // 列表行：缩略图 42px(3:4≈56px) + 上下内边距 20px
    rowH.value = 76
  } else {
    const cw = (w - (cols.value - 1) * colGap) / cols.value
    rowH.value = Math.round((cw * 4) / 3 + m.metaH)
  }
  updateViewport()
}

function onWindowScroll() {
  if (viewportRaf !== undefined) return
  viewportRaf = requestAnimationFrame(() => {
    viewportRaf = undefined
    updateViewport()
  })
}

function onWindowResize() {
  measureGrid()
}

watch(
  filtered,
  () => {
    const wrap = gridWrapRef.value
    if (wrap && wrapObserver) wrapObserver.observe(wrap)
    measureGrid()
  },
  { flush: 'post' },
)

// 分组折叠/排序切换后行结构变化：重测行高并刷新可见窗口
watch(
  gridRows,
  () => {
    measureGrid()
  },
  { flush: 'post' },
)

// GAP 11/103：密度 / 视图切换后重测（列表模式下行高由 .book-card 实际布局测得）
watch([density, viewMode], () => {
  void nextTick(() => measureGrid())
})

// 切换关键词 / 分组 / 排序后回到网格顶部
watch([keyword, activeGroup, sortMode], () => {
  const wrap = gridWrapRef.value
  if (!wrap) return
  window.scrollTo({ top: Math.max(0, wrap.getBoundingClientRect().top + window.scrollY - 96) })
})

async function load(silent = false) {
  if (!silent) loading.value = true
  else refreshing.value = true
  try {
    const [res, gRes] = await Promise.all([
      getBookshelf(silent),
      getBookGroups().catch(() => ({ isSuccess: false, errorMsg: '', data: [] as BookGroup[] })),
    ])
    books.value = res.data ?? []
    groups.value = gRes.data ?? []
    offlineShelf.value = false
    saveOfflineShelf()
    // 数据刷新后清理已失效的选中项
    if (selected.value.size) {
      const valid = new Set(books.value.map((b) => b.bookUrl))
      selected.value = new Set(Array.from(selected.value).filter((u) => valid.has(u)))
    }
    // 分组被删/失效时回退到「全部」
    if (activeGroup.value !== null && !groups.value.some((g) => g.id === activeGroup.value)) {
      activeGroup.value = null
    }
  } catch {
    // 错误提示已由拦截器统一处理；服务端不可达时降级最近一次本地缓存（离线书架）
    const cached = loadOfflineShelf()
    if (cached) {
      books.value = cached.books
      groups.value = cached.groups
      offlineShelf.value = true
      if (!silent) ElMessage.warning('服务端暂不可用，已展示离线书架缓存')
    }
  } finally {
    loading.value = false
    refreshing.value = false
  }
}

function logout() {
  store.clear()
  void router.replace('/login')
}



/* ================= 分组管理 ================= */
function groupName(id: number): string {
  return groups.value.find((g) => g.id === id)?.name ?? (id === 0 ? '未分组' : `分组 ${id}`)
}

function openGroups() {
  groupOpen.value = true
  newGroupName.value = ''
  document.body.style.overflow = 'hidden'
  void nextTick(() => groupDialogRef.value?.focus())
}

function closeGroups() {
  if (groupSaving.value) return
  groupOpen.value = false
  document.body.style.overflow = ''
}

async function createGroup() {
  const name = newGroupName.value.trim()
  if (!name) return
  groupSaving.value = true
  try {
    const res = await saveBookGroup({ name })
    groups.value.push(res.data)
    newGroupName.value = ''
    ElMessage.success('已新建分组')
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    groupSaving.value = false
  }
}

/** 重命名分组：saveBookGroup 带 id 覆盖（id>0） */
function startRename(g: BookGroup) {
  renamingId.value = g.id
  renameName.value = g.name
  renameBusy.value = false
}

function cancelRename() {
  if (renameBusy.value) return
  renamingId.value = null
}

async function saveRename() {
  if (renamingId.value === null || renameBusy.value) return
  const name = renameName.value.trim()
  if (!name) return
  const g = groups.value.find((x) => x.id === renamingId.value)
  if (!g) return
  renameBusy.value = true
  try {
    const res = await saveBookGroup({ id: g.id, name })
    if (res.isSuccess && res.data) Object.assign(g, res.data)
    renamingId.value = null
    ElMessage.success('已重命名')
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    renameBusy.value = false
  }
}

/* ================= 分组元数据：封面 / 显隐（legacy BookGroup.cover + show） ================= */

const groupCoverDraft = ref<Record<number, string>>({})

function onGroupCoverInput(g: BookGroup, value: string) {
  groupCoverDraft.value = { ...groupCoverDraft.value, [g.id]: value }
}

async function saveGroupMeta(g: BookGroup) {
  if (groupSaving.value) return
  groupSaving.value = true
  try {
    const cover = (groupCoverDraft.value[g.id] ?? g.cover ?? '').trim()
    await saveBookGroup({
      id: g.id,
      name: g.name,
      cover: cover || null,
      show: g.show !== false,
      order: g.order ?? g.orderNum ?? g.id,
    })
    g.cover = cover || null
    groupCoverDraft.value = { ...groupCoverDraft.value, [g.id]: g.cover ?? '' }
    ElMessage.success(`分组「${g.name}」已更新`)
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    groupSaving.value = false
  }
}

async function toggleGroupShow(g: BookGroup) {
  g.show = !(g.show !== false)
  try {
    await saveGroupMeta(g)
  } catch {
    g.show = !(g.show !== false)
  }
}

/**
 * 删除分组：POST /reader3/deleteBookGroup（后端并行实现中）——
 * 成功后端将组内书置未分组，本地同步；接口未实现（404）时友好提示。
 */
async function deleteGroup(g: BookGroup) {
  const n = groupCount(g.id)
  try {
    await ElMessageBox.confirm(
      `确定删除分组「${g.name}」吗？组内 ${n} 本书将移至未分组。`,
      '删除分组',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  if (groupSaving.value) return
  groupSaving.value = true
  try {
    await deleteBookGroup(g.id, { silent: true })
    groups.value = groups.value.filter((x) => x.id !== g.id)
    books.value.forEach((b) => {
      setBookGroupIdsLocal(b, bookGroupIds(b).filter((x) => x !== g.id))
    })
    invalidateGroupCounts()
    if (activeGroup.value === g.id) activeGroup.value = null
    ElMessage.success(`已删除分组「${g.name}」`)
  } catch (err) {
    if (isNotImplemented(err)) {
      ElMessage.info('删除分组接口后端暂未提供（POST /reader3/deleteBookGroup）')
    } else {
      ElMessage.error(err instanceof Error ? err.message : '删除失败')
    }
  } finally {
    groupSaving.value = false
  }
}

/* ================= GAP 13：分组拖拽排序（HTML5 drag——拖拽后调 saveBookGroupOrder 保存） ================= */

const draggingId = ref<number | null>(null)
const groupOrderDirty = ref(false)
const groupOrderSaving = ref(false)

/** 拖拽开始（记录源分组 id） */
function onGroupDragStart(g: BookGroup, e: DragEvent) {
  draggingId.value = g.id
  groupOrderDirty.value = true
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', String(g.id))
  }
}

/** 经过目标行：阻止默认（允许 drop） */
function onGroupDragOver(g: BookGroup, e: DragEvent) {
  if (draggingId.value === null || draggingId.value === g.id) return
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
}

/** 放下：把拖拽分组插入到目标分组位置（本地重排）
 *  P2：目标索引在移除源分组后重新定位——直接使用移除前的 toIdx 在向下拖拽时偏移一位 */
function onGroupDrop(g: BookGroup, e: DragEvent) {
  e.preventDefault()
  const from = draggingId.value
  draggingId.value = null
  if (from === null || from === g.id) return
  groups.value = moveGroupTo(groups.value, from, g.id)
}

function onGroupDragEnd() {
  draggingId.value = null
}

/** 保存排序：POST /reader3/saveBookGroupOrder（body {order:[{id,orderNum}]}） */
async function saveGroupOrder() {
  if (groupOrderSaving.value) return
  const order = groups.value.map((g, i) => ({ id: g.id, orderNum: i }))
  if (!order.length) return
  groupOrderSaving.value = true
  try {
    await saveBookGroupOrder(order)
    groupOrderDirty.value = false
    ElMessage.success('分组排序已保存')
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    groupOrderSaving.value = false
  }
}

/* ================= GAP 89：跨书书签列表（逐书 getBookmarks 汇总；点击跳阅读器该章；删除） ================= */

interface AllBookmark {
  bookUrl: string
  bookName: string
  bookmark: Bookmark
}

const bmOpen = ref(false)
const bmLoading = ref(false)
const bmItems = ref<AllBookmark[]>([])
const bmMsg = ref('')
const bmDeleting = ref(false)
/** 跨书书签多选集合（key = `${bookUrl}|${title}`） */
const bmSelected = ref<Set<string>>(new Set())
/** 跨书书签编辑弹窗状态 */
const bmEditing = ref<AllBookmark | null>(null)
const bmSaving = ref(false)
/** 书签 JSON 导入文件输入 */
const bmImportRef = ref<HTMLInputElement | null>(null)

function openBookmarks() {
  bmOpen.value = true
  bmMsg.value = ''
  bmItems.value = []
  bmSelected.value = new Set()
  document.body.style.overflow = 'hidden'
  void loadAllBookmarks()
}

function closeBookmarks() {
  if (bmLoading.value || bmDeleting.value) return
  bmOpen.value = false
  document.body.style.overflow = ''
}

/** 后端无批量接口：循环书架书逐本 getBookmarks（silent——无书签的书静默跳过） */
async function loadAllBookmarks() {
  bmLoading.value = true
  bmItems.value = []
  const out: AllBookmark[] = []
  try {
    const shelfRes = await getBookshelf()
    const list = shelfRes.data ?? []
    // 逐书串行拉取（避免并发打爆后端；每本失败静默跳过）
    for (const b of list) {
      try {
        const res = await getBookmarks(b.bookUrl, { silent: true })
        for (const bm of res.data ?? []) {
          out.push({ bookUrl: b.bookUrl, bookName: b.name || b.bookUrl, bookmark: bm })
        }
      } catch {
        /* 单书书签拉取失败：跳过 */
      }
    }
    out.sort((a, b) => b.bookmark.createdAt - a.bookmark.createdAt)
    bmItems.value = out
    if (out.length === 0) bmMsg.value = '暂无书签——阅读时点「＋书签」添加'
  } catch {
    bmMsg.value = '书架拉取失败，请稍后重试'
  } finally {
    bmLoading.value = false
  }
}

/** 点击 → 阅读器该章（chapterIndex 与阅读页 ?chapter 语义一致） */
function goBookmark(item: AllBookmark) {
  closeBookmarks()
  void router.push(`/reader/${encodeURIComponent(item.bookUrl)}?chapter=${item.bookmark.chapterIndex}`)
}

/** 删除单条书签（POST /reader3/deleteBookmark：bookUrl + title） */
async function removeBookmarkItem(item: AllBookmark) {
  if (bmDeleting.value) return
  bmDeleting.value = true
  try {
    await deleteBookmark(item.bookUrl, item.bookmark.title)
    bmItems.value = bmItems.value.filter(
      (x) => !(x.bookUrl === item.bookUrl && x.bookmark.title === item.bookmark.title),
    )
    if (bmItems.value.length === 0) bmMsg.value = '暂无书签——阅读时点「＋书签」添加'
    ElMessage.success('已删除书签')
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    bmDeleting.value = false
  }
}

function fmtBmTime(ts: number): string {
  if (!ts) return ''
  const d = new Date(ts)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

/* ================= GAP 86：移动端下拉刷新（touch 下拉 >60px 触发 load(true)） ================= */

const pullDist = ref(0)
const pullReady = ref(false)
const PULL_THRESHOLD = 60
let pullStartY = -1
let pullTracking = false

function onPullTouchStart(e: TouchEvent) {
  // 仅页面顶部且无弹层时开始跟踪
  if (window.scrollY > 0) return
  if (document.querySelector('.dlg-overlay, .ctx-overlay, .manage-bar')) return
  pullStartY = e.touches[0]?.clientY ?? -1
  pullTracking = pullStartY >= 0
  pullDist.value = 0
  pullReady.value = false
}

function onPullTouchMove(e: TouchEvent) {
  if (!pullTracking || pullStartY < 0) return
  const y = e.touches[0]?.clientY ?? pullStartY
  const dy = y - pullStartY
  if (dy <= 0) {
    pullDist.value = 0
    pullReady.value = false
    return
  }
  // 阻尼：前 60px 1:1，之后减半
  pullDist.value = dy > PULL_THRESHOLD ? PULL_THRESHOLD + (dy - PULL_THRESHOLD) * 0.5 : dy
  pullReady.value = pullDist.value >= PULL_THRESHOLD
}

function onPullTouchEnd() {
  if (!pullTracking) return
  pullTracking = false
  pullStartY = -1
  if (pullReady.value) {
    pullDist.value = 0
    pullReady.value = false
    void load(true)
  } else {
    pullDist.value = 0
    pullReady.value = false
  }
}

/* ================= GAP 70：全局快捷键（书架：g 搜索 / s 设置 / r 刷新——输入框聚焦时不触发） ================= */

function onGlobalKeydown(e: KeyboardEvent) {
  const t = e.target
  if (t instanceof HTMLElement && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)) {
    return
  }
  if (e.metaKey || e.ctrlKey || e.altKey) return
  switch (e.key.toLowerCase()) {
    case 'g': {
      e.preventDefault()
      const input = document.querySelector<HTMLInputElement>('.search-main .search-input')
      input?.focus()
      input?.select()
      break
    }
    case 's':
      e.preventDefault()
      void router.push('/settings')
      break
    case 'r':
      e.preventDefault()
      void load(true)
      break
  }
}

/* ================= 书卡菜单 ================= */
function openMenuAt(book: Book, x: number, y: number) {
  menuBook.value = book
  menuPos.value = {
    x: Math.min(Math.max(8, x), window.innerWidth - 190),
    y: Math.min(Math.max(8, y), window.innerHeight - 220),
  }
  menuOpen.value = true
}

function openCardMenu(book: Book, e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  openMenuAt(book, e.clientX, e.clientY)
}

function openMenuAtEl(book: Book, e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  openMenuAt(book, rect.left, rect.bottom + 6)
}

/** 触屏长按 500ms 唤出菜单（与点击进详情互斥） */
function onCardTouchStart(book: Book, e: TouchEvent) {
  if (manageMode.value) return // 多选模式：点击即选中，不触发长按菜单
  longPressFired = false
  suppressClick = false
  const t = e.touches[0]
  longPressTimer = window.setTimeout(() => {
    longPressFired = true
    suppressClick = true
    openMenuAt(book, t.clientX, t.clientY)
  }, 500)
}

function onCardTouchEnd() {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
    longPressTimer = undefined
  }
}

function onCardClick(book: Book) {
  if (manageMode.value) {
    toggleSelect(book)
    return
  }
  if (longPressFired) {
    longPressFired = false
    return // 长按已触发菜单，忽略本次点击
  }
  // 点击封面/书名直接进入阅读
  void router.push(`/reader/${encodeURIComponent(book.bookUrl)}`)
}

/** 详情（封面左上角按钮） */
function openDetail(book: Book) {
  void router.push(`/book/${encodeURIComponent(book.bookUrl)}`)
}

function closeMenu() {
  menuOpen.value = false
  menuBook.value = null
}

/** 长按后手指抬起产生的合成 click 会落在遮罩上，吞掉一次防止菜单秒关 */
function onOverlayClick() {
  if (suppressClick) {
    suppressClick = false
    return
  }
  closeMenu()
}

/* ================= 单书多分组面板（勾选多个分组，setBookGroups 整体保存） ================= */

function openBookGroupPanel() {
  const book = menuBook.value
  if (!book) return
  bookGroupPanelIds.value = bookGroupIds(book)
  menuOpen.value = false
  bookGroupPanelOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeBookGroupPanel() {
  if (menuBusy.value) return
  bookGroupPanelOpen.value = false
  document.body.style.overflow = ''
}

function toggleBookGroupPanel(id: number) {
  bookGroupPanelIds.value = bookGroupPanelIds.value.includes(id)
    ? bookGroupPanelIds.value.filter((x) => x !== id)
    : [...bookGroupPanelIds.value, id]
}

async function saveBookGroupPanel() {
  const book = menuBook.value
  if (!book || menuBusy.value) return
  menuBusy.value = true
  try {
    await setBookGroups(book.bookUrl, bookGroupPanelIds.value)
    setBookGroupIdsLocal(book, bookGroupPanelIds.value)
    invalidateGroupCounts()
    ElMessage.success('分组已更新')
    closeBookGroupPanel()
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    menuBusy.value = false
  }
}

/* ================= 拖拽移组（分组模式：书卡拖到分组标题追加该分组；开关防误拖） ================= */
const dragMoveMode = ref(false)
const dragBookUrl = ref<string | null>(null)
const dragOverGroupId = ref<number | null>(null)

watch(sortMode, (v) => {
  if (v !== 'group') dragMoveMode.value = false
})

function onBookDragStart(book: Book, e: DragEvent) {
  if (!dragMoveMode.value) return
  dragBookUrl.value = book.bookUrl
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', book.bookUrl)
  }
}

function onBookDragEnd() {
  dragBookUrl.value = null
  dragOverGroupId.value = null
}

function onGroupHeadDragOver(gid: number, e: DragEvent) {
  if (dragBookUrl.value === null) return
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  dragOverGroupId.value = gid
}

function onGroupHeadDragLeave(e: DragEvent) {
  const cur = e.currentTarget as HTMLElement | null
  if (!cur || !cur.contains(e.relatedTarget as Node | null)) {
    dragOverGroupId.value = null
  }
}

/** 放下书卡到分组标题：追加该分组（多分组语义；拖到「未分组」清空全部分组） */
async function onGroupHeadDrop(gid: number, e: DragEvent) {
  e.preventDefault()
  const url = e.dataTransfer?.getData('text/plain') || dragBookUrl.value
  dragBookUrl.value = null
  dragOverGroupId.value = null
  if (!url) return
  const book = books.value.find((x) => x.bookUrl === url)
  if (!book) return
  if (gid !== 0 && inGroup(book, gid)) {
    ElMessage.info(`《${book.name}》已在「${groupName(gid)}」`)
    return
  }
  try {
    if (gid === 0) {
      await setBookGroups(url, [])
      setBookGroupIdsLocal(book, [])
    } else {
      await addBookGroup(url, gid)
      setBookGroupIdsLocal(book, [...bookGroupIds(book), gid])
    }
    invalidateGroupCounts()
    ElMessage.success(
      gid === 0 ? `已将《${book.name}》移出分组` : `已将《${book.name}》移动到「${groupName(gid)}」`,
    )
  } catch {
    // 错误提示已由拦截器统一处理
  }
}

function removeFromShelf() {
  const book = menuBook.value
  if (!book || menuBusy.value) return
  removeTarget.value = book
  closeMenu()
  confirmRemoveOpen.value = true
}

/** 批量删除勾选书签（按书分组调后端批量接口） */
async function removeSelectedBookmarks() {
  if (bmDeleting.value || bmSelected.value.size === 0) return
  bmDeleting.value = true
  try {
    const byBook = new Map<string, string[]>()
    for (const key of bmSelected.value) {
      const sep = key.indexOf('|')
      if (sep < 0) continue
      const bookUrl = key.slice(0, sep)
      const title = key.slice(sep + 1)
      const arr = byBook.get(bookUrl) ?? []
      arr.push(title)
      byBook.set(bookUrl, arr)
    }
    let deleted = 0
    for (const [bookUrl, titles] of byBook) {
      const res = await deleteBookmarks(bookUrl, titles)
      deleted += res.data?.count ?? titles.length
    }
    ElMessage.success(`已删除 ${deleted} 条书签`)
    bmSelected.value = new Set()
    await loadAllBookmarks()
  } catch {
    /* 错误提示已由拦截器统一处理 */
  } finally {
    bmDeleting.value = false
  }
}

function toggleBmSelect(item: AllBookmark) {
  const key = `${item.bookUrl}|${item.bookmark.title}`
  const next = new Set(bmSelected.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  bmSelected.value = next
}

function editBookmarkItem(item: AllBookmark) {
  bmEditing.value = { ...item, bookmark: { ...item.bookmark } }
}

async function saveBookmarkEdit() {
  const item = bmEditing.value
  if (!item) return
  const title = item.bookmark.title.trim()
  if (!title) {
    ElMessage.warning('书签标题不能为空')
    return
  }
  bmSaving.value = true
  try {
    await saveBookmark(item.bookmark)
    bmEditing.value = null
    await loadAllBookmarks()
    ElMessage.success('书签已更新')
  } catch {
    /* 错误提示已由拦截器统一处理 */
  } finally {
    bmSaving.value = false
  }
}

async function importBookmarks(file: File) {
  const text = await file.text()
  let parsed: Bookmark[]
  try {
    parsed = parseBookmarksJson(text, '')
  } catch {
    ElMessage.error('书签 JSON 解析失败')
    return
  }
  if (parsed.length === 0) {
    ElMessage.warning('未找到有效书签数据')
    return
  }
  // 书架中无对应书 URL 的书签（legacy 导出只有 bookName）尽量按书名回填
  const shelfRes = await getBookshelf()
  const shelf = shelfRes.data ?? []
  for (const bm of parsed) {
    if (!bm.bookUrl) {
      const hit = shelf.find((b) => b.name === bm.bookName || b.bookUrl === bm.bookUrl)
      bm.bookUrl = hit?.bookUrl ?? bm.bookUrl
    }
  }
  const res = await saveBookmarks(parsed.filter((b) => b.bookUrl))
  ElMessage.success(`已导入 ${res.data?.count ?? parsed.length} 条书签`)
  await loadAllBookmarks()
}

function onBmImportChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) void importBookmarks(file)
  input.value = ''
}

async function doRemoveFromShelf() {
  const book = removeTarget.value
  confirmRemoveOpen.value = false
  removeTarget.value = null
  if (!book || menuBusy.value) return
  menuBusy.value = true
  try {
    await deleteBook(book.bookUrl)
    books.value = books.value.filter((b) => b.bookUrl !== book.bookUrl)
    ElMessage.success('已移出书架')
  } catch {
    // 错误提示已由拦截器统一处理
  } finally {
    menuBusy.value = false
  }
}

onMounted(() => {
  // 旧会话可能未带 isAdmin 标记：后台探测一次，管理员入口/系统配置按钮据此恢复显示
  void probeSecureMode().catch(() => false)
  wrapObserver = new ResizeObserver(() => {
    const w = gridWrapRef.value?.clientWidth ?? 0
    if (w === lastWrapW) return
    lastWrapW = w
    measureGrid()
  })
  window.addEventListener('scroll', onWindowScroll, { passive: true })
  window.addEventListener('resize', onWindowResize)
  // GAP 86：移动端下拉刷新（passive，不干扰纵向滚动）
  window.addEventListener('touchstart', onPullTouchStart, { passive: true })
  window.addEventListener('touchmove', onPullTouchMove, { passive: true })
  window.addEventListener('touchend', onPullTouchEnd, { passive: true })
  // GAP 70：全局快捷键
  window.addEventListener('keydown', onGlobalKeydown)
  load()
})
</script>

<template>
  <div class="bookshelf-page">
    <!-- 顶部导航（P3-A：共享 TopNav——品牌/导航链接/用户区） -->
    <TopNav
      active="/"
      :links="['search', 'explore', 'sources', 'rules', 'rss', 'files', 'store', 'monitor', 'users', 'settings']"
      show-logout
      show-users-link
      @logout="logout"
    >
      <div class="search-box">
        <div class="search-main">
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
            v-model="keyword"
            class="search-input"
            type="text"
            placeholder="搜书（全站书源）…"
            spellcheck="false"
            @keyup.enter="goSearchBooks"
          />
          <button
            v-if="keyword"
            class="search-clear"
            type="button"
            title="清空"
            @click="keyword = ''"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <path d="M6 6l12 12M18 6L6 18" />
            </svg>
          </button>
        </div>
        <div class="search-sub">
          <span class="search-mode-label">范围</span>
          <button
            class="search-mode-btn"
            :class="{ active: searchMode === 'name' }"
            type="button"
            title="按书名 / 作者筛选书架"
            @click="searchMode = 'name'"
          >
            书名/作者
          </button>
          <button
            class="search-mode-btn"
            :class="{ active: searchMode === 'full' }"
            type="button"
            title="全书内容搜索（本地书正文）"
            @click="searchMode = 'full'"
          >
            全书
          </button>
          <span v-if="searchMode === 'full'" class="search-note">
            全书内容搜索（本地书正文——输入后自动搜索）
          </span>
        </div>
      </div>

      <template #extra>
        <button class="nav-link" type="button" title="全部书签（跨书）" @click="openBookmarks">书签</button>
        <button class="nav-link" type="button" title="OPDS 服务器（外部阅读器连接）" @click="openOpds">OPDS</button>
      </template>
    </TopNav>

    <Transition name="pull">
      <div v-if="offlineShelf" class="offline-shelf-banner">
        <span>离线书架缓存（服务端暂不可用）</span>
        <button type="button" title="重新连接服务器" @click="load(true)">重试</button>
      </div>
    </Transition>

    <main class="content" :class="{ 'with-manage-bar': manageMode }">
      <!-- GAP 86：移动端下拉刷新指示（下拉 >60px 释放触发刷新） -->
      <Transition name="pull">
        <div v-if="pullDist > 0" class="pull-indicator" :class="{ ready: pullReady }">
          <svg
            class="pull-arrow"
            :style="{ transform: `rotate(${Math.min(pullDist / PULL_THRESHOLD, 1) * 180}deg)` }"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M12 5v14" />
            <path d="M6 13l6 6 6-6" />
          </svg>
          <span>{{ pullReady ? '释放刷新' : '下拉刷新' }}</span>
        </div>
      </Transition>

      <!-- 全书搜索命中面板（范围=全书：本地书正文命中，书 + 章节列表，点击跳阅读器该章） -->
      <div v-if="searchMode === 'full' && keyword.trim()" class="content-hits">
        <div v-if="contentSearching" class="chits-state">
          <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
            <path d="M21 12a9 9 0 1 1-6.2-8.56" />
          </svg>
          <span>正在全书搜索（逐本地书并发）…</span>
        </div>
        <template v-else-if="contentSearchDone">
          <p v-if="contentResults.length === 0" class="chits-state">
            全书未找到匹配内容（仅本地书正文参与搜索）
          </p>
          <div v-else class="chits-panel">
            <p class="chits-title">
              全书命中：{{ contentResults.length }} 本书 · {{ contentHitTotal }} 章——点击跳转阅读器该章
            </p>
            <div class="chits-list">
              <div v-for="r in contentResults" :key="r.book.bookUrl" class="chits-book">
                <p class="chits-book-name" :title="r.book.name">
                  {{ r.book.name }}
                  <span class="chits-book-meta">{{ r.book.author || '佚名' }} · {{ r.hits.length }} 章命中</span>
                </p>
                <ul class="chits-hits">
                  <li v-for="(hit, i) in r.hits" :key="`${r.book.bookUrl}-${hit.chapterIndex}-${i}`">
                    <button class="chits-hit" type="button" @click="goContentHit(r.book, hit)">
                      <span class="chits-hit-ch">第 {{ hit.chapterIndex + 1 }} 章</span>
                      <span class="chits-hit-title" :title="hit.title">{{ hit.title || '（无标题）' }}</span>
                      <span class="chits-hit-snippet" :title="hit.snippet">{{ hit.snippet }}</span>
                    </button>
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- 标题区 -->
      <div class="section-head">
        <h1 class="section-title">我的书架</h1>
        <span class="count" title="书架统计">共 {{ shelfSummary.books }} 本 · {{ shelfSummary.groups }} 组</span>
        <button
          class="manage-btn"
          type="button"
          :class="{ active: manageMode }"
          :title="manageMode ? '退出多选' : '进入多选模式'"
          @click="toggleManage"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 11.5l2 2 4-4.5" />
            <rect x="4" y="4" width="16" height="16" rx="3" />
          </svg>
          <span>{{ manageMode ? '完成' : '管理' }}</span>
        </button>
        <button class="import-btn" type="button" title="导入本地书" @click="openImport">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 16V4" />
            <path d="M7 9l5-5 5 5" />
            <path d="M4 16v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3" />
          </svg>
          <span>导入本地书</span>
        </button>
        <button
          class="refresh-btn"
          type="button"
          title="刷新书架"
          :class="{ spinning: refreshing }"
          @click="load(true)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12a9 9 0 1 1-2.64-6.36" />
            <path d="M21 3v6h-6" />
          </svg>
        </button>
      </div>

      <!-- 排序胶囊（前端排序 books 副本，不改服务端顺序；localStorage: reader_shelf_sort） -->
      <div class="sort-bar">
        <span class="sort-label">排序</span>
        <button
          v-for="opt in SORT_OPTIONS"
          :key="opt.value"
          class="sort-capsule"
          :class="{ active: sortMode === opt.value }"
          type="button"
          @click="sortMode = opt.value"
        >
          {{ opt.label }}
        </button>
        <span v-if="addedSortNote" class="sort-note" :title="addedSortTip">{{ addedSortNote }}</span>
      </div>

      <!-- 显示设置：网格密度（GAP 11，localStorage: reader_card_density）+ 网格/列表/墙切换（GAP 103 + M4，localStorage: reader_shelf_view） -->
      <div class="view-bar">
        <span class="sort-label">密度</span>
        <button
          v-for="opt in DENSITY_OPTIONS"
          :key="opt.value"
          class="sort-capsule"
          :class="{ active: density === opt.value }"
          type="button"
          :title="`卡片：${opt.label}（${cardMinW}px 起）`"
          @click="setDensity(opt.value)"
        >
          {{ opt.label }}
        </button>
        <span class="view-sep"></span>
        <button
          class="view-toggle"
          type="button"
          :title="viewToggleTitle"
          @click="cycleViewMode"
        >
          <svg v-if="viewMode === 'wall'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round">
            <rect x="4" y="4" width="6.5" height="6.5" rx="1.2" />
            <rect x="13.5" y="4" width="6.5" height="6.5" rx="1.2" />
            <rect x="4" y="13.5" width="6.5" height="6.5" rx="1.2" />
            <rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.2" />
          </svg>
          <svg v-else-if="viewMode === 'list'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round">
            <rect x="4" y="4" width="16" height="5" rx="1.5" />
            <rect x="4" y="9.5" width="16" height="5" rx="1.5" />
            <rect x="4" y="15" width="16" height="5" rx="1.5" />
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round">
            <rect x="4" y="4" width="7" height="7" rx="1.5" />
            <rect x="13" y="4" width="7" height="7" rx="1.5" />
            <rect x="4" y="13" width="7" height="7" rx="1.5" />
            <rect x="13" y="13" width="7" height="7" rx="1.5" />
          </svg>
        </button>
      </div>

      <!-- 分组栏：全部 / 分组名 胶囊筛选（细字，active 强调色下划线） -->
      <div class="group-bar">
        <div class="group-tabs" role="tablist" aria-label="书架分组筛选">
          <button
            type="button"
            class="group-tab"
            :class="{ active: activeGroup === null }"
            @click="activeGroup = null"
          >
            全部
          </button>
          <button
            v-for="g in visibleGroups"
            :key="g.id"
            type="button"
            class="group-tab"
            :class="{ active: activeGroup === g.id }"
            @click="activeGroup = g.id"
          >
            {{ g.name }}
          </button>
        </div>
        <button class="group-manage" type="button" @click="openGroups">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 7h10" />
            <path d="M18 7h2" />
            <circle cx="16" cy="7" r="2" />
            <path d="M4 17h2" />
            <path d="M10 17h10" />
            <circle cx="8" cy="17" r="2" />
          </svg>
          <span>管理</span>
        </button>
        <button
          class="group-manage"
          :class="{ active: dragMoveMode }"
          type="button"
          :disabled="sortMode !== 'group'"
          :title="sortMode !== 'group' ? '需先切换排序为「分组」' : (dragMoveMode ? '退出拖拽移组' : '拖拽移组：把书卡拖到分组标题即可移动分组（开关防止误拖）')"
          @click="dragMoveMode = !dragMoveMode"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 6h.01M16 6h.01M8 12h.01M16 12h.01M8 18h.01M16 18h.01" />
            <path d="M11 6h2M11 12h2M11 18h2" />
          </svg>
          <span>拖拽移组</span>
        </button>
      </div>

      <!-- 加载骨架（浅灰静置块） -->
      <div v-if="loading" class="book-grid" aria-label="加载中">
        <div v-for="i in 12" :key="i" class="skeleton-card">
          <div class="skeleton-cover"></div>
          <div class="skeleton-line"></div>
          <div class="skeleton-line short"></div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else-if="filtered.length === 0" class="empty-state">
        <p class="empty-text">{{ emptyText }}</p>
      </div>

      <!-- 书封网格（虚拟滚动：仅渲染可见行 + 上下缓冲；分组模式含可点击折叠的分组标题行） -->
      <div v-else ref="gridWrapRef" class="virtual-grid">
        <div class="virtual-pad" :style="{ height: padTop + 'px' }"></div>
        <div
          class="book-grid"
          :class="{ list: viewMode === 'list', wall: viewMode === 'wall' }"
          :style="{ '--card-w': gridCardMinW + 'px' }"
        >
          <template v-for="row in visibleRows" :key="rowKey(row)">
            <!-- 分组标题行（排序=分组；点击折叠/展开该组，折叠后该组书隐藏） -->
            <div v-if="row.kind === 'header'" class="group-head-row">
              <button
                class="group-head"
                :class="{ 'drop-target': dragMoveMode && dragOverGroupId === row.groupId }"
                type="button"
                :title="dragMoveMode ? `拖放书卡到此移入「${row.name}」` : (groupCollapsed(row.groupId) ? '展开该组' : '折叠该组')"
                @click="toggleGroupCollapsed(row.groupId)"
                @dragover="onGroupHeadDragOver(row.groupId, $event)"
                @dragleave="onGroupHeadDragLeave"
                @drop="onGroupHeadDrop(row.groupId, $event)"
              >
                <span class="group-caret" :class="{ open: !groupCollapsed(row.groupId) }">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M6 9l6 6 6-6" />
                  </svg>
                </span>
                <span class="group-head-name">{{ row.name }}</span>
                <span class="group-head-count">{{ row.count }} 本</span>
              </button>
            </div>
            <template v-else>
              <div
                v-for="book in row.books"
                :key="book.bookUrl"
                class="book-card"
                :class="{ managing: manageMode, selected: selected.has(book.bookUrl), dragging: dragBookUrl === book.bookUrl }"
                :draggable="dragMoveMode"
                @click="onCardClick(book)"
                @contextmenu="onCardMenu(book, $event)"
                @touchstart.passive="onCardTouchStart(book, $event)"
                @touchend="onCardTouchEnd"
                @touchcancel="onCardTouchEnd"
                @dragstart="onBookDragStart(book, $event)"
                @dragend="onBookDragEnd"
              >
            <span class="select-dot" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 12.5l4.5 4.5L19 7.5" />
              </svg>
            </span>
            <button
              class="card-menu-btn"
              type="button"
              title="更多操作"
              :tabindex="manageMode ? -1 : 0"
              @click="openMenuAtEl(book, $event)"
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <circle cx="5" cy="12" r="1.6" />
                <circle cx="12" cy="12" r="1.6" />
                <circle cx="19" cy="12" r="1.6" />
              </svg>
            </button>
            <div class="cover-wrap">
              <button
                v-if="!manageMode"
                class="detail-btn"
                type="button"
                title="查看详情"
                :tabindex="-1"
                @click.stop="openDetail(book)"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="9" />
                  <path d="M12 11v5" />
                  <path d="M12 8h.01" />
                </svg>
              </button>
              <img
                v-if="hasCover(book)"
                v-lazy="coverSrc(book) as string"
                class="cover-img"
                :alt="book.name"
                loading="lazy"
                @error="onCoverError(book)"
              />
              <div v-else class="cover-ph" :style="{ background: coverColor(book.name) }">
                <span class="cover-ph-char">{{ coverInitial(book.name) }}</span>
              </div>
              <!-- GAP 14：阅读进度角标（durChapterIndex/totalChapterNum；无数据隐藏） -->
              <span v-if="bookProgress(book) !== null" class="progress-badge" :title="`已读 ${bookProgress(book)}%（第 ${(book.durChapterIndex ?? 0) + 1}/${book.totalChapterNum} 章）`">
                {{ bookProgress(book) }}%
              </span>
              <!-- GAP 146：置顶角标 -->
              <span v-if="isPinned(book)" class="pin-badge" title="已置顶（长按/右键菜单可取消）">置顶</span>
              <!-- 未读更新数 -->
              <span
                v-if="unreadCount(book) !== null"
                class="unread-badge"
                :title="book.durChapterTitle ? `距上次阅读（${book.durChapterTitle}）更新 ${unreadCount(book)} 章` : `未读更新 ${unreadCount(book)} 章`"
              >
                +{{ unreadCount(book) }}
              </span>
            </div>
            <div class="book-meta">
              <p class="book-name" :title="book.name">{{ book.name }}</p>
              <p class="book-author">{{ book.author || '佚名' }}</p>
              <p v-if="book.durChapterTitle" class="book-read" :title="`读到：${book.durChapterTitle}`">
                读到：{{ book.durChapterTitle }}
              </p>
              <p v-if="book.latestChapterTitle" class="book-chapter" :title="book.latestChapterTitle">
                最新：{{ book.latestChapterTitle }}
              </p>
            </div>
            <!-- 悬浮简介预览（桌面 hover：卡片上方浮层，鼠标移出关闭；touch 无 hover 不启用） -->
            <div v-if="hoverPreview(book)" class="hover-preview">
              <p class="hp-name" :title="book.name">{{ book.name }}</p>
              <p class="hp-author">{{ book.author || '佚名' }}</p>
              <p v-if="book.durChapterTitle" class="hp-chapter" :title="book.durChapterTitle">
                读到：{{ book.durChapterTitle }}
              </p>
              <p v-if="book.latestChapterTitle" class="hp-chapter" :title="book.latestChapterTitle">
                最新章：{{ book.latestChapterTitle }}
              </p>
              <p v-if="unreadCount(book) !== null" class="hp-chapter unread">
                {{ unreadCount(book) }} 章未读
              </p>
              <p class="hp-intro">{{ hoverPreview(book) }}</p>
            </div>
            </div>
          </template>
          </template>
        </div>
        <div class="virtual-pad" :style="{ height: padBottom + 'px' }"></div>
      </div>
    </main>

    <!-- 导入本地书弹窗（自写轻量，无 Element Plus 重组件） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="importOpen" class="dlg-overlay" @click.self="closeImport">
          <div
            ref="dialogRef"
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="导入本地书籍"
            tabindex="-1"
            @keydown.esc="closeImport"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">导入本地书籍</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="uploadBusy" @click="closeImport">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>

            <!-- 虚线拖拽区：点击选择 / 拖入文件 -->
            <div
              class="dropzone"
              :class="{ over: isDragOver, busy: uploadBusy }"
              @click="!uploadBusy && fileInput?.click()"
              @dragover="onDragOver"
              @dragleave="onDragLeave"
              @drop="onDrop"
            >
              <svg class="dz-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 16V4" />
                <path d="M7 9l5-5 5 5" />
                <path d="M4 16v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3" />
              </svg>
              <p class="dz-text">点击选择文件，或将文件拖拽到此处</p>
              <p class="dz-sub">支持 .epub / .txt / .mobi / .azw3 / .pdf / .fb2 / .docx · 可多选</p>
              <input
                ref="fileInput"
                class="visually-hidden"
                type="file"
                accept=".epub,.txt,.mobi,.azw3,.pdf,.fb2,.docx,application/epub+zip,text/plain"
                multiple
                @change="onPick"
              />
            </div>
            <p v-if="acceptTip" class="accept-tip">{{ acceptTip }}</p>

            <!-- 文件列表：逐个状态 + 细字进度 -->
            <ul v-if="importItems.length" class="file-list">
              <li v-for="(item, i) in importItems" :key="`${item.file.name}-${i}`" class="file-row">
                <span class="file-name" :title="item.file.name">{{ item.file.name }}</span>
                <span class="file-size">{{ fmtSize(item.file.size) }}</span>
                <span class="file-state" :class="item.status">
                  <template v-if="item.status === 'pending'">待导入</template>
                  <template v-else-if="item.status === 'uploading'">
                    <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                      <path d="M21 12a9 9 0 1 1-6.2-8.56" />
                    </svg>
                    {{ item.progress }}%
                  </template>
                  <svg v-else-if="item.status === 'done'" class="state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M4.5 12.5l5 5L19.5 7" />
                  </svg>
                  <template v-else>{{ item.error || '导入失败' }}</template>
                </span>
                <button
                  v-if="item.status === 'pending' && !uploadBusy"
                  class="file-remove"
                  type="button"
                  title="移除"
                  @click="removeItem(i)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                    <path d="M6 6l12 12M18 6L6 18" />
                  </svg>
                </button>
              </li>
            </ul>

            <!-- 导入预览（后端 /reader3/importBookPreview：书名/作者/格式/章节数/前 5 章标题；未实现则隐藏并直接上传） -->
            <div v-if="previewSupported" class="preview-panel">
              <p class="preview-title">导入预览</p>
              <div v-for="(item, i) in previewedItems" :key="`pv-${i}`" class="preview-item">
                <p class="preview-head">
                  <span class="preview-name" :title="item.preview?.name || item.file.name">
                    {{ item.preview?.name || item.file.name }}
                  </span>
                  <span class="preview-meta">
                    {{ item.preview?.author ? item.preview.author + ' · ' : '' }}{{ item.preview?.format || '未知格式' }} · {{ previewChapterCount(item) }} 章
                  </span>
                </p>
                <ol v-if="previewChapters(item).length" class="preview-chapters">
                  <li v-for="(ch, j) in previewChapters(item)" :key="j">{{ ch }}</li>
                </ol>
              </div>
              <p class="preview-tip">预览由服务器解析 · 确认无误后点击「开始导入」（确认后仍走上传接口）</p>
            </div>
            <p v-else-if="importItems.length && !previewChecking" class="preview-tip muted">
              服务器未提供导入预览（POST /reader3/importBookPreview），将直接上传
            </p>

            <!-- 底部：整体进度 / 摘要 + 操作 -->
            <div class="dlg-foot">
              <div v-if="uploadBusy" class="overall">
                <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                  <path d="M21 12a9 9 0 1 1-6.2-8.56" />
                </svg>
                <span>正在导入 {{ uploadIndex + 1 }} / {{ importItems.length }} · {{ totalProgress }}%</span>
              </div>
              <div v-else-if="importDone" class="overall" :class="{ hasError: failedCount > 0 }">
                {{ importSummary }}
              </div>
              <div v-else class="overall"></div>

              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="uploadBusy" @click="closeImport">取消</button>
                <button
                  class="accent-btn"
                  type="button"
                  :disabled="uploadBusy || !hasPending"
                  @click="startUpload"
                >
                  {{ uploadBusy ? '导入中…' : hasPending ? `开始导入（${hasPendingCount}）` : '开始导入' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- OPDS 服务器弹窗（外部阅读器连接地址 + 复制） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="opdsOpen" class="dlg-overlay" @click.self="closeOpds">
          <div
            class="dlg dlg-opds"
            role="dialog"
            aria-modal="true"
            aria-label="OPDS 服务器"
            tabindex="-1"
            @keydown.esc="closeOpds"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">OPDS 服务器</h2>
              <button class="dlg-close" type="button" title="关闭" @click="closeOpds">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <p class="opds-tip">将以下地址粘贴到外部阅读器（legado、静读天下等）的 OPDS 地址栏，即可同步书架与阅读。地址已附带登录凭证，请勿分享给他人。</p>
            <div class="opds-row">
              <span class="opds-url mono" :title="opdsUrl">{{ opdsUrl }}</span>
              <button class="accent-btn" type="button" @click="copyOpdsUrl">
                {{ opdsCopied ? '已复制' : '复制' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 书卡菜单（右键 / 长按 / hover ⋯） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="menuOpen && menuBook" class="ctx-overlay" @click="onOverlayClick" @contextmenu.prevent="closeMenu">
          <div class="ctx-menu" :style="{ left: menuPos.x + 'px', top: menuPos.y + 'px' }" @click.stop>
            <button class="ctx-item" type="button" @click="togglePin">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 4h6l-1 6 3 3v2H7v-2l3-3z" />
                <path d="M12 15v5" />
              </svg>
              {{ isPinned(menuBook) ? '取消置顶' : '置顶' }}
            </button>
            <button class="ctx-item" type="button" @click="openBookGroupPanel">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 5.5A1.5 1.5 0 0 1 5.5 4h4L12 6.5h6.5A1.5 1.5 0 0 1 20 8v10.5a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 18.5z" />
              </svg>
              设置分组
            </button>
            <button class="ctx-item" type="button" @click="openExportFor(menuBook)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 15V4" />
                <path d="M7 9.5L12 4.5l5 5" />
                <path d="M4 15v3.5A1.5 1.5 0 0 0 5.5 20h13a1.5 1.5 0 0 0 1.5-1.5V15" />
              </svg>
              导出
            </button>
            <!-- GAP 78：重新扫描（仅本地书：local:// 双轨书 / loc_book 文件书——重解析原文件刷新章节） -->
            <button v-if="menuBook && canRescanBook(menuBook)" class="ctx-item" type="button" :disabled="rescanBusy" @click="rescanBook">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12a9 9 0 1 1-2.64-6.36" />
                <path d="M21 3v6h-6" />
              </svg>
              重新扫描
            </button>
            <button class="ctx-item danger" type="button" :disabled="menuBusy" @click="removeFromShelf">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16" />
                <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                <path d="M6.5 7l.8 12a1.5 1.5 0 0 0 1.5 1.4h6.4a1.5 1.5 0 0 0 1.5-1.4l.8-12" />
              </svg>
              移出书架
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 单书设置分组弹层（多分组勾选，setBookGroups 整体保存） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="bookGroupPanelOpen && menuBook" class="dlg-overlay" @click.self="closeBookGroupPanel">
          <div
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="设置分组"
            tabindex="-1"
            @keydown.esc="closeBookGroupPanel"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">设置分组 · {{ menuBook.name }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="menuBusy" @click="closeBookGroupPanel">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <ul class="book-group-panel-list">
              <li v-for="g in groups" :key="g.id" class="book-group-panel-row">
                <label class="book-group-panel-check">
                  <input
                    type="checkbox"
                    :checked="bookGroupPanelIds.includes(g.id)"
                    :disabled="menuBusy"
                    @change="toggleBookGroupPanel(g.id)"
                  />
                  <span :title="g.name">{{ g.name }}</span>
                </label>
                <span class="book-group-panel-count">{{ groupCount(g.id) }} 本</span>
              </li>
            </ul>
            <p v-if="!groups.length" class="group-empty">还没有分组，先到书架「分组管理」新建</p>
            <div class="dlg-foot">
              <span class="overall">可同时勾选多个分组；不勾选任何分组 = 未分组</span>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="menuBusy" @click="closeBookGroupPanel">取消</button>
                <button class="accent-btn" type="button" :disabled="menuBusy" @click="saveBookGroupPanel">
                  {{ menuBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 移出书架确认（自写轻量弹窗，与导入/OPDS 同风格） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div
          v-if="confirmRemoveOpen && removeTarget"
          class="dlg-overlay"
          @click.self="confirmRemoveOpen = false"
        >
          <div
            class="dlg dlg-confirm"
            role="dialog"
            aria-modal="true"
            aria-label="移出书架"
            tabindex="-1"
            @keydown.esc="confirmRemoveOpen = false"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">移出书架</h2>
              <button
                class="dlg-close"
                type="button"
                title="关闭"
                :disabled="menuBusy"
                @click="confirmRemoveOpen = false"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <p class="dlg-confirm-text">
              确定将《{{ removeTarget.name }}》移出书架吗？移出后阅读进度不会保留。
            </p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="menuBusy" @click="confirmRemoveOpen = false">
                取消
              </button>
              <button class="accent-btn danger" type="button" :disabled="menuBusy" @click="doRemoveFromShelf">
                {{ menuBusy ? '移出中…' : '移出' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 分组管理弹窗（极简：新建 + 列表 + 删除提示） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="groupOpen" class="dlg-overlay" @click.self="closeGroups">
          <div
            ref="groupDialogRef"
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="分组管理"
            tabindex="-1"
            @keydown.esc="closeGroups"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">分组管理</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="groupSaving" @click="closeGroups">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>

            <!-- 新建分组 -->
            <div class="group-create">
              <input
                v-model="newGroupName"
                class="group-input"
                type="text"
                placeholder="新分组名称"
                maxlength="20"
                spellcheck="false"
                @keydown.enter="createGroup"
              />
              <button
                class="accent-btn"
                type="button"
                :disabled="groupSaving || !newGroupName.trim()"
                @click="createGroup"
              >
                {{ groupSaving ? '创建中…' : '新建' }}
              </button>
            </div>

            <!-- 分组列表：名称 + 本书数 + 重命名 / 删除（GAP 13：拖拽排序——拖到目标行释放即重排，保存排序提交） -->
            <ul v-if="groups.length" class="group-list">
              <li
                v-for="g in groups"
                :key="g.id"
                class="group-row"
                :class="{ dragging: draggingId === g.id }"
                draggable="true"
                @dragstart="onGroupDragStart(g, $event)"
                @dragover="onGroupDragOver(g, $event)"
                @drop="onGroupDrop(g, $event)"
                @dragend="onGroupDragEnd"
              >
                <!-- GAP 13：拖拽手柄 -->
                <span class="group-drag" title="拖拽排序">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <circle cx="9" cy="6" r="1.4" />
                    <circle cx="15" cy="6" r="1.4" />
                    <circle cx="9" cy="12" r="1.4" />
                    <circle cx="15" cy="12" r="1.4" />
                    <circle cx="9" cy="18" r="1.4" />
                    <circle cx="15" cy="18" r="1.4" />
                  </svg>
                </span>
                <!-- 重命名：行内输入 -->
                <template v-if="renamingId === g.id">
                  <input
                    v-model="renameName"
                    class="group-input group-rename-input"
                    type="text"
                    maxlength="20"
                    spellcheck="false"
                    @keydown.enter="saveRename"
                    @keydown.esc="cancelRename"
                  />
                  <button
                    class="group-del"
                    type="button"
                    title="保存重命名"
                    :disabled="renameBusy || !renameName.trim()"
                    @click="saveRename"
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M4.5 12.5l5 5L19.5 7" />
                    </svg>
                  </button>
                  <button class="group-del" type="button" title="取消" :disabled="renameBusy" @click="cancelRename">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                      <path d="M6 6l12 12M18 6L6 18" />
                    </svg>
                  </button>
                </template>
                <template v-else>
                  <span class="group-row-name" :title="g.name">{{ g.name }}</span>
                  <span class="group-row-count">{{ groupCount(g.id) }} 本</span>
                  <span class="group-cover-cell">
                    <input
                      class="group-cover-input"
                      type="text"
                      placeholder="封面 URL"
                      spellcheck="false"
                      :value="groupCoverDraft[g.id] ?? g.cover ?? ''"
                      @input="onGroupCoverInput(g, ($event.target as HTMLInputElement).value)"
                      @keydown.enter="saveGroupMeta(g)"
                      @blur="saveGroupMeta(g)"
                    />
                  </span>
                  <button
                    class="group-del"
                    :class="{ 'show-off': g.show === false }"
                    type="button"
                    :title="g.show === false ? '显示该分组' : '隐藏该分组'"
                    :disabled="groupSaving"
                    @click="toggleGroupShow(g)"
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Z" />
                      <circle cx="12" cy="12" r="2.6" />
                      <path v-if="g.show === false" d="M4 4l16 16" />
                    </svg>
                  </button>
                  <button class="group-del" type="button" title="重命名" @click="startRename(g)">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z" />
                    </svg>
                  </button>
                  <button class="group-del" type="button" title="删除分组" @click="deleteGroup(g)">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M4 7h16" />
                      <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                      <path d="M6.5 7l.8 12a1.5 1.5 0 0 0 1.5 1.4h6.4a1.5 1.5 0 0 0 1.5-1.4l.8-12" />
                    </svg>
                  </button>
                </template>
              </li>
            </ul>
            <p v-else class="group-empty">还没有分组，输入名称新建一个吧</p>

            <!-- GAP 13：排序保存（拖拽后显示） -->
            <div v-if="groupOrderDirty" class="group-order-save">
              <button
                class="accent-btn"
                type="button"
                :disabled="groupOrderSaving"
                @click="saveGroupOrder"
              >
                {{ groupOrderSaving ? '保存中…' : '保存排序' }}
              </button>
            </div>

            <div class="dlg-foot">
              <span class="overall">重命名：改名称保存 · 封面 URL 回车/失焦保存 · 眼睛切换显隐 · 拖拽手柄调整顺序后保存</span>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="groupSaving" @click="closeGroups">关闭</button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 多选底部操作条（细字：已选 N 本 + 移出书架 + 移动到分组） -->
    <Transition name="bar">
      <div v-if="manageMode" class="manage-bar">
        <span class="manage-count">已选 {{ selected.size }} 本</span>
        <div class="manage-actions">
          <button
            class="manage-act danger"
            type="button"
            :disabled="selected.size === 0 || manageBusy"
            @click="bulkRemove"
          >
            删除
          </button>
          <button
            class="manage-act accent"
            type="button"
            :disabled="selected.size === 0 || manageBusy"
            @click="openMovePanel"
          >
            移动到分组
          </button>
        </div>
      </div>
    </Transition>

    <!-- 批量分组弹层（多选加入 / 移除，可同时处理多个分组） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="moveOpen" class="dlg-overlay" @click.self="closeMovePanel">
          <div
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="移动到分组"
            tabindex="-1"
            @keydown.esc="closeMovePanel"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">移动到分组</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="manageBusy" @click="closeMovePanel">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <ul class="move-group-list">
              <li v-for="g in groups" :key="g.id">
                <div class="move-group-row">
                  <span class="move-group-name" :title="g.name">{{ g.name }}</span>
                  <span class="move-group-count">{{ groupCount(g.id) }} 本</span>
                  <button
                    class="move-group-act add"
                    :class="{ active: moveAddIds.includes(g.id) }"
                    type="button"
                    :disabled="manageBusy"
                    @click="toggleMoveAdd(g.id)"
                  >
                    {{ moveAddIds.includes(g.id) ? '已选加入' : '加入' }}
                  </button>
                  <button
                    class="move-group-act remove"
                    :class="{ active: moveRemoveIds.includes(g.id) }"
                    type="button"
                    :disabled="manageBusy"
                    @click="toggleMoveRemove(g.id)"
                  >
                    {{ moveRemoveIds.includes(g.id) ? '已选移除' : '移除' }}
                  </button>
                </div>
              </li>
            </ul>
            <label class="move-clear">
              <input v-model="moveClearAll" type="checkbox" :disabled="manageBusy" />
              <span>同时清空全部所选书的分组（移除所有分组）</span>
            </label>
            <div class="dlg-foot">
              <span class="overall">
                {{
                  manageBusy
                    ? '正在更新…'
                    : `已选 ${selected.size} 本 · 加入 ${moveAddIds.length} 组 · 移除 ${moveRemoveIds.length} 组${moveClearAll ? ' · 清空全部分组' : ''}`
                }}
              </span>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="manageBusy" @click="closeMovePanel">取消</button>
                <button
                  class="accent-btn"
                  type="button"
                  :disabled="manageBusy || (!moveAddIds.length && !moveRemoveIds.length && !moveClearAll)"
                  @click="performMove"
                >
                  {{ manageBusy ? '更新中…' : '应用' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 导出弹层（GET /reader3/exportBook：txt/epub/html blob 下载） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="exportOpen" class="dlg-overlay" @click.self="closeExport">
          <div
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="导出书籍"
            tabindex="-1"
            @keydown.esc="closeExport"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">导出 · {{ exportName }}</h2>
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
            <p v-if="exportMsg" class="export-msg" :class="{ error: exportMsgError }">{{ exportMsg }}</p>
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

    <!-- GAP 89：跨书书签列表弹层（逐书 getBookmarks 汇总；点击跳阅读器该章；删除） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="bmOpen" class="dlg-overlay" @click.self="closeBookmarks">
          <div
            class="dlg dlg-bookmarks"
            role="dialog"
            aria-modal="true"
            aria-label="全部书签"
            tabindex="-1"
            @keydown.esc="closeBookmarks"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">全部书签{{ bmItems.length ? ` · ${bmItems.length} 条` : '' }}</h2>
              <div class="dlg-head-actions">
                <input
                  ref="bmImportRef"
                  class="visually-hidden"
                  type="file"
                  accept="application/json,.json"
                  @change="onBmImportChange"
                />
                <button
                  class="ghost-btn"
                  type="button"
                  title="导入书签 JSON（按 bookUrl/bookName 回填书架）"
                  :disabled="bmLoading || bmDeleting"
                  @click="bmImportRef?.click()"
                >
                  导入
                </button>
                <button
                  v-if="bmSelected.size > 0"
                  class="ghost-btn danger"
                  type="button"
                  :disabled="bmLoading || bmDeleting"
                  @click="removeSelectedBookmarks"
                >
                  删除勾选 ({{ bmSelected.size }})
                </button>
                <button class="dlg-close" type="button" title="关闭" :disabled="bmLoading || bmDeleting" @click="closeBookmarks">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                    <path d="M6 6l12 12M18 6L6 18" />
                  </svg>
                </button>
              </div>
            </div>

            <div v-if="bmLoading" class="bm-state">
              <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <path d="M21 12a9 9 0 1 1-6.2-8.56" />
              </svg>
              <span>正在汇总全部书签…</span>
            </div>
            <p v-else-if="bmMsg" class="bm-state">{{ bmMsg }}</p>

            <ul v-else class="bm-list">
              <li v-for="(item, i) in bmItems" :key="`${item.bookUrl}-${item.bookmark.title}-${i}`" class="bm-item">
                <input
                  class="bm-check"
                  type="checkbox"
                  :checked="bmSelected.has(`${item.bookUrl}|${item.bookmark.title}`)"
                  :title="'多选后批量删除'"
                  @change="toggleBmSelect(item)"
                />
                <button
                  type="button"
                  class="bm-jump"
                  :title="`跳转到《${item.bookName}》第 ${item.bookmark.chapterIndex + 1} 章`"
                  @click="goBookmark(item)"
                >
                  <span class="bm-book" :title="item.bookName">{{ item.bookName }}</span>
                  <span class="bm-chapter">
                    {{ item.bookmark.chapterName || `第 ${item.bookmark.chapterIndex + 1} 章` }}
                  </span>
                  <span class="bm-text" :title="item.bookmark.title">{{ item.bookmark.title }}</span>
                  <span v-if="item.bookmark.bookText" class="bm-quote" :title="item.bookmark.bookText">{{ item.bookmark.bookText }}</span>
                  <span v-if="item.bookmark.content" class="bm-note">{{ item.bookmark.content }}</span>
                  <span class="bm-time">{{ fmtBmTime(item.bookmark.createdAt) }}</span>
                </button>
                <button
                  type="button"
                  class="bm-edit"
                  title="编辑书签"
                  :disabled="bmDeleting"
                  @click="editBookmarkItem(item)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
                  </svg>
                </button>
                <button
                  type="button"
                  class="bm-del"
                  title="删除书签"
                  :disabled="bmDeleting"
                  @click="removeBookmarkItem(item)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                    <path d="M6 6l12 12M18 6L6 18" />
                  </svg>
                </button>
              </li>
            </ul>

            <div class="dlg-foot">
              <span class="overall">点击跳转到对应章节 · 勾选后批量删除 · 支持 JSON 导入</span>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="bmLoading || bmDeleting" @click="closeBookmarks">关闭</button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 跨书书签编辑弹层 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="bmEditing" class="dlg-overlay" @click.self="bmEditing = null">
          <div class="dlg dlg-bookmark-edit" role="dialog" aria-modal="true" aria-label="编辑书签">
            <div class="dlg-head">
              <h2 class="dlg-title">编辑书签</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="bmSaving" @click="bmEditing = null">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="bm-edit-form">
              <label class="edit-field">
                <span>标题</span>
                <input v-model="bmEditing.bookmark.title" type="text" placeholder="书签标题" />
              </label>
              <label class="edit-field">
                <span>备注</span>
                <textarea v-model="bmEditing.bookmark.content" rows="3" placeholder="备注（可选）"></textarea>
              </label>
              <label class="edit-field">
                <span>正文</span>
                <textarea v-model="bmEditing.bookmark.bookText" rows="4" placeholder="书签段落文本（可选）"></textarea>
              </label>
            </div>
            <div class="dlg-foot">
              <span class="overall">{{ bmEditing.bookName }}</span>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="bmSaving" @click="bmEditing = null">取消</button>
                <button class="primary-btn" type="button" :disabled="bmSaving" @click="saveBookmarkEdit">
                  {{ bmSaving ? '保存中…' : '保存' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.bookshelf-page {
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
  gap: 24px;
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
  width: 30px;
  height: 30px;
  border-radius: 8px;
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

/* 搜索框（细边框圆角 8px）：输入行 + 范围切换行 */
.search-box {
  flex: 1;
  max-width: 420px;
  margin: 0 auto;
}
.search-main {
  position: relative;
}
.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 15px;
  height: 15px;
  color: var(--text-3);
  pointer-events: none;
  transition: color 0.2s ease;
}
.search-box:focus-within .search-icon {
  color: var(--accent);
}
.search-input {
  width: 100%;
  height: 38px;
  padding: 0 34px 0 36px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13.5px;
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
}
.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 22px;
  height: 22px;
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
.search-clear:hover {
  color: var(--text-1);
}
.search-clear svg {
  width: 11px;
  height: 11px;
}

/* 范围切换（书名/作者 · 全书） + 待实现标注 */
.search-sub {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
  min-height: 20px;
}
.search-mode-label {
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.search-mode-btn {
  padding: 2px 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.search-mode-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.search-mode-btn.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}
.search-note {
  flex: 1;
  min-width: 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 用户区 */
.user-area {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-shrink: 0;
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
.user-chip {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-2);
}
.logout-btn {
  padding: 6px 14px;
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
.logout-btn:hover {
  color: var(--text-1);
  border-color: var(--border-strong);
}

/* ================= 内容区 ================= */
.content {
  width: min(1200px, 100%);
  margin: 0 auto;
  padding: 48px 32px 72px;
}

.section-head {
  display: flex;
  align-items: baseline;
  gap: 14px;
  margin-bottom: 36px;
}
.section-title {
  margin: 0;
  font-size: 22px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-1);
}
.count {
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* 导入本地书按钮（细字描边，hover 加深） */
.import-btn {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: none;
  color: var(--accent);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.import-btn:hover {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}
.import-btn svg {
  width: 13px;
  height: 13px;
}

.refresh-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.refresh-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.refresh-btn.spinning svg {
  animation: spin 0.8s linear infinite;
}
.refresh-btn svg {
  width: 14px;
  height: 14px;
}

/* ================= 书封网格（大间距 28-32px；列宽由 GAP 11 密度变量 --card-w 控制） ================= */
.book-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--card-w, 160px), 1fr));
  gap: 32px 28px;
}

/* GAP 103：列表视图（小缩略图行） */
.book-grid.list {
  grid-template-columns: 1fr;
  gap: 0;
}
.book-grid.list .book-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 8px;
  border-bottom: 1px solid var(--border);
}
.book-grid.list .book-card:hover {
  transform: none;
  background: var(--hover);
}
.book-grid.list .cover-wrap {
  flex-shrink: 0;
  width: 42px;
  border-radius: 6px;
}
.book-grid.list .book-meta {
  flex: 1;
  min-width: 0;
  padding: 0;
  display: flex;
  align-items: baseline;
  gap: 14px;
}
.book-grid.list .book-name {
  flex: 0 1 auto;
  max-width: 38%;
  font-size: 14px;
}
.book-grid.list .book-author {
  margin: 0;
  flex-shrink: 0;
  font-size: 12px;
}
.book-grid.list .book-chapter {
  flex: 1;
  min-width: 0;
  margin: 0;
  font-size: 12px;
  text-align: right;
}
.book-grid.list .detail-btn,
.book-grid.list .card-menu-btn {
  top: auto;
  left: auto;
  right: 8px;
  bottom: 8px;
}
.book-grid.list .progress-badge {
  font-size: 9.5px;
  padding: 1px 5px;
  bottom: 4px;
  right: 4px;
}
.book-grid.list .pin-badge {
  font-size: 9.5px;
  padding: 1px 5px;
  left: 4px;
  bottom: 4px;
}

/* GAP 103 扩展（M4）：墙视图（大封面网格——固定大卡片，间距更宽；
   列数/行高由 shelfViewMetrics('wall') 计算，--card-w 由 gridCardMinW 内联） */
.book-grid.wall {
  gap: 48px 40px;
}

.book-card {
  position: relative;
  cursor: pointer;
  transition: transform 0.2s ease;
}
.book-card:hover {
  z-index: 10;
  transform: translateY(-4px);
}
.book-card.dragging {
  opacity: 0.55;
  transform: scale(0.97);
}

.detail-btn {
  position: absolute;
  top: 6px;
  left: 6px;
  z-index: 3;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  background: rgba(255, 255, 255, 0.85);
  border: none;
  border-radius: 999px;
  color: var(--text-2, #666);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s ease, color 0.2s ease;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
}
.detail-btn svg {
  width: 14px;
  height: 14px;
}
.book-card:hover .detail-btn {
  opacity: 1;
}
.detail-btn:hover {
  color: var(--accent, #4f46e5);
}
.cover-wrap {
  position: relative;
  aspect-ratio: 3 / 4;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid var(--border);
  background: var(--surface);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  transition: border-color 0.2s ease;
}
.book-card:hover .cover-wrap {
  border-color: var(--accent);
}

.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  opacity: 0;
  transition: opacity 0.3s ease;
}
.cover-img.is-loaded {
  opacity: 1;
}

/* GAP 14：阅读进度角标（封面右下角） */
.progress-badge {
  position: absolute;
  right: 6px;
  bottom: 6px;
  z-index: 2;
  padding: 2px 7px;
  border-radius: 999px;
  background: rgba(20, 20, 24, 0.72);
  color: #fff;
  font-size: 10.5px;
  font-weight: 400;
  letter-spacing: 0.5px;
  line-height: 1.4;
  font-variant-numeric: tabular-nums;
  backdrop-filter: blur(2px);
  pointer-events: none;
}

/* 莫兰迪纯色占位 + 细体首字 */
.cover-ph {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.cover-ph-char {
  font-size: 44px;
  font-weight: 300;
  color: rgba(255, 255, 255, 0.94);
  letter-spacing: 2px;
}

/* 书籍信息 */
.book-meta {
  padding: 12px 2px 0;
}
.book-name {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.book-author {
  margin: 4px 0 0;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.book-chapter {
  margin: 6px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.book-read {
  margin: 4px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ================= 悬浮简介预览（桌面 hover：卡片上方浮层；touch 无 hover 不启用） ================= */
.hover-preview {
  position: absolute;
  left: 50%;
  bottom: calc(100% + 10px);
  transform: translateX(-50%);
  z-index: 1000;
  width: max-content;
  max-width: min(280px, calc(100vw - 24px));
  padding: 12px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.12);
  text-align: left;
  pointer-events: none;
  opacity: 0;
  visibility: hidden;
  transition:
    opacity 0.15s ease,
    visibility 0.15s ease;
}
.book-card:hover .hover-preview {
  opacity: 1;
  visibility: visible;
}
.hp-name {
  margin: 0;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.hp-author {
  margin: 3px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.hp-chapter {
  margin: 6px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.hp-intro {
  margin: 8px 0 0;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-2);
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
/* touch 设备（无 hover 概念）不启用悬浮预览 */
@media (hover: none), (pointer: coarse) {
  .hover-preview {
    display: none;
  }
}

/* ================= 骨架屏（浅灰静置 + GAP 72 shimmer 扫光） ================= */
.skeleton-cover,
.skeleton-line {
  position: relative;
  overflow: hidden;
  background: #f0f0f2;
}
.skeleton-cover {
  aspect-ratio: 3 / 4;
  border-radius: 10px;
  border: 1px solid var(--border);
}
.skeleton-line {
  height: 11px;
  margin-top: 12px;
  border-radius: 4px;
}
.skeleton-line.short {
  width: 55%;
  margin-top: 8px;
}
/* GAP 72：shimmer 扫光（渐变平移） */
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

/* GAP 146：置顶角标（封面左下角细字） */
.pin-badge {
  position: absolute;
  left: 6px;
  bottom: 6px;
  z-index: 2;
  padding: 1px 7px;
  border-radius: 4px;
  background: rgba(24, 24, 27, 0.72);
  color: #fff;
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 1px;
  pointer-events: none;
}
.unread-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  z-index: 2;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--accent);
  color: var(--on-accent);
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.5px;
  pointer-events: none;
}

/* ================= 空状态 ================= */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 96px 0;
}
.empty-text {
  margin: 0;
  font-size: 14px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}

/* ================= 全书搜索命中面板（范围=全书；书 + 章节命中，点击跳阅读器） ================= */
.content-hits {
  margin: -8px 0 28px;
}
.chits-state {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 18px 0;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.chits-panel {
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.05);
}
.chits-title {
  margin: 0 0 10px;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--accent);
}
.chits-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 340px;
  overflow-y: auto;
}
.chits-book + .chits-book {
  padding-top: 12px;
  border-top: 1px dashed var(--border);
}
.chits-book-name {
  margin: 0 0 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.chits-book-meta {
  margin-left: 8px;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
}
.chits-hits {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.chits-hit {
  width: 100%;
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 6px 8px;
  border: none;
  border-radius: 6px;
  background: none;
  text-align: left;
  font-family: inherit;
  cursor: pointer;
  transition: background-color 0.15s ease;
}
.chits-hit:hover {
  background: var(--hover);
}
.chits-hit-ch {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.chits-hit-title {
  flex-shrink: 0;
  max-width: 36%;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.chits-hit-snippet {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 旋转（刷新按钮 / 登录 spinner 共用） */
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* ================= 导入本地书弹窗（自写轻量） ================= */
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
  width: min(460px, 100%);
  max-height: calc(100vh - 64px);
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

/* 虚线拖拽区：hover 变强调色 */
.dropzone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 34px 16px;
  border: 1.5px dashed var(--border-strong);
  border-radius: var(--radius);
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.dropzone:hover,
.dropzone.over {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.dropzone.busy {
  cursor: default;
  opacity: 0.6;
}
.dz-icon {
  width: 26px;
  height: 26px;
  color: var(--text-3);
  transition: color 0.2s ease;
}
.dropzone:hover .dz-icon,
.dropzone.over .dz-icon {
  color: var(--accent);
}
.dz-text {
  margin: 8px 0 0;
  font-size: 13.5px;
  font-weight: 400;
  color: var(--text-2);
}
.dz-sub {
  margin: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.file-input {
  display: none;
}
.accept-tip {
  margin: 10px 2px 0;
  font-size: 12px;
  font-weight: 300;
  color: #cf4444;
}

/* 文件列表 */
.file-list {
  list-style: none;
  margin: 14px 0 0;
  padding: 0;
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.file-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.file-name {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.file-size {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.file-state {
  flex-shrink: 0;
  min-width: 52px;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.file-state.uploading {
  color: var(--accent);
  font-weight: 400;
}
.file-state.done {
  color: #529b2e;
}
.file-state.error {
  color: #cf4444;
  min-width: 0;
  max-width: 130px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.state-icon {
  width: 12px;
  height: 12px;
}
.mini-spin {
  width: 12px;
  height: 12px;
  animation: spin 0.8s linear infinite;
}
.file-remove {
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
  transition: color 0.2s ease;
}
.file-remove:hover {
  color: #cf4444;
}
.file-remove svg {
  width: 10px;
  height: 10px;
}

/* 底部：进度 / 摘要 + 操作 */
.dlg-foot {
  margin-top: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.overall {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.overall.hasError {
  color: #cf4444;
}
.dlg-actions {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

/* ================= OPDS 弹窗 ================= */
.dlg-opds {
  width: min(520px, 100%);
}
.opds-tip {
  margin: 0 0 14px;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.8;
  color: var(--text-3);
}
.opds-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
}
.opds-url {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
.accent-btn.danger {
  background: #cf4444;
  border-color: #cf4444;
}
.accent-btn.danger:hover:not(:disabled) {
  background: #b93a3a;
  border-color: #b93a3a;
}

/* 移出书架确认弹窗 */
.dlg-confirm-text {
  margin: 0 0 18px;
  font-size: 13px;
  font-weight: 400;
  line-height: 1.8;
  color: var(--text-2);
  white-space: pre-wrap;
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

/* ================= 排序胶囊 ================= */
.sort-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: -14px 0 26px;
  flex-wrap: wrap;
}
.sort-label {
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.sort-capsule {
  padding: 4px 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
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
.sort-capsule:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.sort-note {
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  border-bottom: 1px dashed var(--border);
  cursor: help;
}

/* ================= 显示设置：密度 + 视图切换（GAP 11 / GAP 103） ================= */
.view-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 26px;
  flex-wrap: wrap;
}
.view-sep {
  width: 1px;
  height: 16px;
  margin: 0 4px;
  background: var(--border);
}
.view-toggle {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.view-toggle:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.view-toggle svg {
  width: 14px;
  height: 14px;
}

/* ================= 分组标题行（排序=分组；点击折叠） ================= */
.group-head-row {
  grid-column: 1 / -1;
}
.group-head {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 4px;
  border: none;
  border-bottom: 1px solid var(--border);
  background: none;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: color 0.2s ease;
}
.group-head:hover {
  color: var(--accent);
}
.group-head.drop-target {
  border-color: var(--accent);
  background: var(--accent-soft);
  border-radius: 6px;
}
.group-head.drop-target .group-head-name {
  color: var(--accent);
}
.group-caret {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  color: var(--text-3);
  transition: transform 0.2s ease;
}
.group-caret svg {
  width: 12px;
  height: 12px;
}
.group-caret.open {
  transform: rotate(180deg);
}
.group-head-name {
  font-size: 13.5px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.group-head-count {
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= 导入预览 ================= */
.preview-panel {
  margin-top: 14px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
  max-height: 180px;
  overflow-y: auto;
}
.preview-title {
  margin: 0 0 8px;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 2px;
  color: var(--accent);
}
.preview-item + .preview-item {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed var(--border);
}
.preview-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin: 0;
}
.preview-name {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.preview-meta {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
}
.preview-chapters {
  margin: 6px 0 0;
  padding-left: 18px;
  font-size: 11.5px;
  font-weight: 300;
  line-height: 1.8;
  color: var(--text-3);
}
.preview-tip {
  margin: 8px 2px 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
}
.preview-tip.muted {
  color: var(--text-3);
}

/* ================= 分组栏（胶囊筛选：细字 + 强调色下划线） ================= */
.group-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin: 0 0 32px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
  overflow-x: auto;
  scrollbar-width: none;
}
.group-bar::-webkit-scrollbar {
  display: none;
}
.group-tabs {
  display: flex;
  align-items: center;
  gap: 22px;
  flex: 1;
  min-width: 0;
}
.group-tab {
  position: relative;
  flex-shrink: 0;
  padding: 4px 2px 8px;
  border: none;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.group-tab:hover {
  color: var(--text-2);
}
.group-tab.active {
  color: var(--accent);
  font-weight: 400;
}
.group-tab.active::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 2px;
  border-radius: 2px;
  background: var(--accent);
}
.group-manage {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.group-manage:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.group-manage.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.group-manage:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.group-manage svg {
  width: 12px;
  height: 12px;
}

/* 书卡右上角 ⋯（hover 显现） */
.card-menu-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 2;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.92);
  color: var(--text-2);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
  cursor: pointer;
  opacity: 0;
  transition:
    opacity 0.2s ease,
    color 0.2s ease;
}
.book-card:hover .card-menu-btn,
.card-menu-btn:focus-visible {
  opacity: 1;
}
.card-menu-btn:hover {
  color: var(--accent);
}
.card-menu-btn svg {
  width: 13px;
  height: 13px;
}

/* ================= 书卡菜单（右键 / 长按 / ⋯） ================= */
.ctx-overlay {
  position: fixed;
  inset: 0;
  z-index: 120;
}
.ctx-menu {
  position: fixed;
  z-index: 121;
  min-width: 168px;
  max-width: 220px;
  max-height: 320px;
  overflow-y: auto;
  padding: 6px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.1);
}
.ctx-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  text-align: left;
  cursor: pointer;
  transition:
    color 0.15s ease,
    background-color 0.15s ease;
}
.ctx-item:hover:not(:disabled) {
  color: var(--text-1);
  background: var(--hover);
}
.ctx-item:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
.ctx-item.danger {
  color: #cf4444;
}
.ctx-item.danger:hover:not(:disabled) {
  color: #b33535;
  background: rgba(207, 68, 68, 0.07);
}
.ctx-item svg {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
}
.ctx-title {
  padding: 4px 10px 8px;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}

/* ================= 分组管理弹窗 ================= */
.group-create {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
}
.group-input {
  flex: 1;
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
.group-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.group-input:focus {
  border-color: var(--accent);
}
/* 重命名行内输入（紧凑） */
.group-rename-input {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 10px;
  font-size: 12.5px;
}
.group-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 260px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.group-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.group-row-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.group-row-count {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.group-del {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 4px;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    background-color 0.2s ease;
}
.group-del:hover {
  color: #cf4444;
  background: rgba(207, 68, 68, 0.08);
}
.group-del:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
/* 重命名保存/取消按钮：hover 用强调色而非危险色 */
.group-row .group-del[title='保存重命名']:hover,
.group-row .group-del[title='取消']:hover {
  color: var(--accent);
  background: var(--accent-soft);
}
.group-del svg {
  width: 12px;
  height: 12px;
}
/* GAP 13：拖拽手柄 + 拖拽中行样式 */
.group-drag {
  flex-shrink: 0;
  width: 18px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
  cursor: grab;
}
.group-drag svg {
  width: 13px;
  height: 13px;
}
.group-row.dragging {
  opacity: 0.45;
  border-style: dashed;
}
.group-row[draggable='true']:active .group-drag {
  cursor: grabbing;
}
.group-order-save {
  display: flex;
  justify-content: flex-end;
  margin-top: 10px;
}
.group-empty {
  margin: 0;
  padding: 28px 0;
  text-align: center;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}
/* 分组封面 URL（行内细输入框） */
.group-cover-cell {
  flex-shrink: 0;
  width: 150px;
}
.group-cover-input {
  width: 100%;
  height: 26px;
  padding: 0 8px;
  border-radius: 5px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-3);
  font-family: inherit;
  font-size: 11px;
  font-weight: 300;
  outline: none;
  transition: border-color 0.2s ease;
}
.group-cover-input:focus {
  border-color: var(--accent);
  color: var(--text-1);
}
.group-del.show-off {
  color: var(--text-3);
}
.group-del.show-off svg {
  opacity: 0.55;
}

/* ================= 导出弹层 ================= */
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

/* ================= 跨书书签弹层（GAP 89） ================= */
.dlg-bookmarks {
  width: min(560px, 100%);
}
.dlg-head-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.dlg-head-actions .ghost-btn.danger {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.45);
}
.dlg-head-actions .ghost-btn.danger:hover {
  background: rgba(207, 68, 68, 0.08);
}
.bm-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 36px 0;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}
.bm-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 46vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.bm-item {
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  transition: border-color 0.2s ease;
}
.bm-item:hover {
  border-color: var(--accent);
}
.bm-check {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  margin-left: 10px;
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
  padding: 10px 12px;
  border: none;
  background: none;
  text-align: left;
  cursor: pointer;
}
.bm-book {
  flex-shrink: 0;
  max-width: 130px;
  font-size: 13px;
  font-weight: 500;
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bm-chapter {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.bm-text {
  max-width: 100%;
  font-size: 12.5px;
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
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.bm-edit {
  flex-shrink: 0;
  width: 30px;
  height: 30px;
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
.bm-edit:hover:not(:disabled) {
  color: var(--accent);
  background: var(--accent-soft);
}
.bm-edit:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
.bm-edit svg {
  width: 12px;
  height: 12px;
}
.bm-del {
  flex-shrink: 0;
  width: 30px;
  height: 30px;
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
.bm-del:hover:not(:disabled) {
  color: #cf4444;
  background: rgba(207, 68, 68, 0.08);
}
.bm-del:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
.bm-del svg {
  width: 12px;
  height: 12px;
}

/* 跨书书签编辑弹窗 */
.dlg-bookmark-edit {
  width: min(420px, 100%);
}
.bm-edit-form {
  padding: 0 22px;
}
.edit-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 14px;
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
  border-radius: 6px;
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
.primary-btn {
  height: 34px;
  padding: 0 18px;
  border: 1px solid var(--accent);
  border-radius: 6px;
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

/* ================= GAP 86：下拉刷新指示 ================= */
.pull-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 44px;
  margin: -24px 0 4px;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.pull-indicator.ready {
  color: var(--accent);
}
.pull-arrow {
  width: 15px;
  height: 15px;
  transition: transform 0.15s ease;
}
.offline-shelf-banner {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  background: var(--accent-soft, rgba(120, 160, 255, 0.08));
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.offline-shelf-banner button {
  padding: 2px 8px;
  border: 1px solid var(--border-strong);
  border-radius: 4px;
  background: none;
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
}
.pull-enter-active,
.pull-leave-active {
  transition: opacity 0.15s ease;
}
.pull-enter-from,
.pull-leave-to {
  opacity: 0;
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
.export-msg {
  margin: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.export-msg.error {
  color: #cf4444;
}
.field-tip {
  margin: 0 0 10px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= 多选模式 ================= */
.manage-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.manage-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.manage-btn.active {
  color: var(--on-accent);
  background: var(--accent);
  border-color: var(--accent);
}
.manage-btn.active:hover {
  background: var(--accent-deep);
  border-color: var(--accent-deep);
}
.manage-btn svg {
  width: 13px;
  height: 13px;
}

/* 虚拟滚动容器：上下占位撑高，中间网格只渲染可见行 */
.virtual-grid {
  position: relative;
}
.virtual-pad {
  width: 100%;
}

/* 多选选择点（左上圆形，选中 accent 填充 + 对勾） */
.select-dot {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 2;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.92);
  border: 1.5px solid var(--border-strong);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  color: transparent;
  opacity: 0;
  transform: scale(0.82);
  pointer-events: none;
  transition:
    opacity 0.2s ease,
    transform 0.2s ease,
    background-color 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease;
}
.book-card.managing .select-dot {
  opacity: 1;
  transform: scale(1);
}
.book-card.selected .select-dot {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
}
.select-dot svg {
  width: 11px;
  height: 11px;
}

/* 多选时隐藏书卡右上角 ⋯（避免与点选冲突） */
.book-card.managing .card-menu-btn {
  opacity: 0;
  pointer-events: none;
}

/* 多选底部操作条（细字胶囊，fade 200ms） */
.manage-bar {
  position: fixed;
  left: 50%;
  bottom: 24px;
  transform: translateX(-50%);
  z-index: 60;
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 10px 16px 10px 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 999px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}
.manage-count {
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2);
}
.manage-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.manage-act {
  padding: 6px 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: none;
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.manage-act.danger {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.35);
}
.manage-act.danger:hover:not(:disabled) {
  background: rgba(207, 68, 68, 0.08);
  border-color: #cf4444;
}
.manage-act.accent {
  color: var(--accent);
  border-color: var(--accent);
}
.manage-act.accent:hover:not(:disabled) {
  background: var(--accent-soft);
  border-color: var(--accent-deep);
  color: var(--accent-deep);
}
.manage-act:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

/* 底部操作条动画：fade + 轻微上移 200ms */
.bar-enter-active,
.bar-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}
.bar-enter-from,
.bar-leave-to {
  opacity: 0;
  transform: translate(-50%, 8px);
}

/* 多选时给内容区底部留出操作条空间 */
.content.with-manage-bar {
  padding-bottom: 150px;
}

/* 多选移动到分组弹层列表 */
.move-group-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 260px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.move-group-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.move-group-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.move-group-count {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.move-group-act {
  flex-shrink: 0;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: none;
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.move-group-act.add {
  color: var(--accent);
  border-color: var(--accent);
}
.move-group-act.add.active,
.move-group-act.add:hover:not(:disabled) {
  background: var(--accent-soft);
  border-color: var(--accent-deep);
  color: var(--accent-deep);
}
.move-group-act.remove {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.35);
}
.move-group-act.remove.active,
.move-group-act.remove:hover:not(:disabled) {
  background: rgba(207, 68, 68, 0.08);
  border-color: #cf4444;
}
.move-group-act:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.move-clear {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 10px 2px 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  cursor: pointer;
}
.move-clear input {
  accent-color: var(--accent);
}

/* 单书设置分组弹层 */
.book-group-panel-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 280px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.book-group-panel-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.book-group-panel-check {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  cursor: pointer;
}
.book-group-panel-check input {
  flex-shrink: 0;
  accent-color: var(--accent);
}
.book-group-panel-check span {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.book-group-panel-count {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .dlg-overlay {
    padding: 16px;
  }
  .dlg {
    max-height: calc(100vh - 32px);
  }
  .import-btn {
    padding: 6px 12px;
    font-size: 12.5px;
  }
  .import-btn span {
    display: none;
  }
  .topbar {
    flex-wrap: wrap;
    gap: 12px;
    padding: 12px 16px;
  }
  .search-box {
    order: 3;
    max-width: none;
    flex-basis: 100%;
    margin: 0;
  }
  .user-area {
    overflow-x: auto;
    max-width: 100%;
    scrollbar-width: none;
    -webkit-overflow-scrolling: touch;
  }
  .user-area::-webkit-scrollbar {
    display: none;
  }
  .user-area .nav-link,
  .user-area .user-chip,
  .user-area .logout-btn {
    flex-shrink: 0;
  }
  .content {
    padding: 32px 16px 56px;
  }
  .book-grid {
    grid-template-columns: repeat(auto-fill, minmax(var(--card-w, 120px), 1fr));
    gap: 24px 16px;
  }
  .section-head {
    margin-bottom: 28px;
  }
  .manage-bar {
    bottom: 16px;
    padding: 8px 12px 8px 16px;
    gap: 12px;
  }
  .manage-act {
    padding: 5px 12px;
    font-size: 12px;
  }
}

/* 小屏手机：书架列数继续加密 + 底部操作栏避开手势区 */
@media (max-width: 480px) {
  .book-grid {
    grid-template-columns: repeat(auto-fill, minmax(var(--card-w, 104px), 1fr));
    gap: 20px 12px;
  }
  .brand-name {
    font-size: 15px;
    letter-spacing: 2px;
  }
  .search-mode-label,
  .search-note {
    display: none;
  }
  .manage-bar {
    padding-bottom: max(8px, env(safe-area-inset-bottom));
  }
}
</style>
