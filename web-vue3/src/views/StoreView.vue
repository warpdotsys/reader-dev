<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'
import { listFiles } from '@/api/file'
import { getBookshelf, saveBook } from '@/api/bookshelf'
import { probeSecureMode } from '@/api/users'
import type { Book, FileItem } from '@/types'

/**
 * GAP 10：共享书仓（storage/localStore，file/list home=__LOCAL_STORE__）。
 * 浏览书仓书籍文件（epub/txt 等）→ 勾选批量导入书架（saveBook 逐本，origin=loc_book
 * 路径型文件书，后端无批量导入接口——后端就绪后可替换为批量契约）。
 * secure 模式需用户开启「本地书仓」权限（后端返回「未开启本地书仓功能」）。
 */

const router = useRouter()
const store = useUserStore()

const path = ref('')
const files = ref<FileItem[]>([])
const loading = ref(true)
const loadError = ref('')

/** secure 模式探测（决定顶部权限提示是否显示） */
const secureMode = ref(false)

/** 可导入书架的文件扩展名（与书架「导入本地书」一致） */
const BOOK_EXTS = new Set(['epub', 'txt', 'mobi', 'azw3', 'pdf', 'fb2', 'docx', 'md'])

function isBookFile(item: FileItem): boolean {
  const ext = item.name.includes('.') ? item.name.slice(item.name.lastIndexOf('.') + 1).toLowerCase() : ''
  return BOOK_EXTS.has(ext)
}

/** 书仓文件 → 书架 loc_book bookUrl（storage/localStore/ 相对路径，与后端 resolve_loc_book_file 对应） */
function bookUrlOf(item: FileItem): string {
  return `storage/localStore/${item.path.replace(/^\/+/, '')}`
}

/* ---------------- 选择 ---------------- */
const selected = ref<Set<string>>(new Set())

function toggleSelect(item: FileItem) {
  const s = new Set(selected.value)
  if (s.has(item.path)) s.delete(item.path)
  else s.add(item.path)
  selected.value = s
}

const selectedItems = computed(() => files.value.filter((f) => selected.value.has(f.path)))
const selectedBookCount = computed(() => selectedItems.value.filter((f) => !f.isDirectory && isBookFile(f)).length)

/* ---------------- 列表 ---------------- */
async function loadList() {
  loading.value = true
  loadError.value = ''
  try {
    const res = await listFiles(path.value, '__LOCAL_STORE__')
    const list = (res.data ?? []) as FileItem[]
    files.value = [
      ...list.filter((f) => f.isDirectory).sort((a, b) => a.name.localeCompare(b.name)),
      ...list.filter((f) => !f.isDirectory).sort((a, b) => a.name.localeCompare(b.name)),
    ]
    // 清理失效勾选
    const valid = new Set(files.value.map((f) => f.path))
    selected.value = new Set(Array.from(selected.value).filter((p) => valid.has(p)))
  } catch (err) {
    files.value = []
    const e = err as { data?: unknown; message?: string } | null | undefined
    const msg = e?.data === 'NEED_SECURE_KEY' ? '需管理密码（secure 模式）' : e?.message ?? '加载失败'
    loadError.value = msg
  } finally {
    loading.value = false
  }
}

function enterDir(item: FileItem) {
  if (!item.isDirectory) return
  path.value = item.path.replace(/^\/+/, '').replace(/\/+$/, '')
  selected.value = new Set()
  void loadList()
}

function goCrumb(index: number) {
  path.value = index < 0 ? '' : crumbs.value[index].full
  selected.value = new Set()
  void loadList()
}

const crumbs = computed(() =>
  path.value
    .split('/')
    .filter(Boolean)
    .map((seg, i, arr) => ({ name: seg, full: arr.slice(0, i + 1).join('/'), last: i === arr.length - 1 })),
)

/* ---------------- 批量导入书架（逐本 saveBook：origin=loc_book 路径型文件书） ---------------- */
const importBusy = ref(false)
const importSummary = ref('')
const importError = ref('')

async function importSelected() {
  if (importBusy.value) return
  const items = selectedItems.value.filter((f) => !f.isDirectory && isBookFile(f))
  if (!items.length) {
    ElMessage.info('请先勾选书仓中的书籍文件')
    return
  }
  importBusy.value = true
  importSummary.value = ''
  importError.value = ''
  try {
    // 书架去重：同名 bookUrl 跳过（不重复入架）
    const shelfRes = await getBookshelf().catch(() => null)
    const shelfUrls = new Set((shelfRes?.data ?? []).map((b) => b.bookUrl))
    let ok = 0
    let skipped = 0
    let failed = 0
    for (const item of items) {
      const bookUrl = bookUrlOf(item)
      if (shelfUrls.has(bookUrl)) {
        skipped++
        continue
      }
      const name = item.name.replace(/\.[^.]+$/, '')
      try {
        await saveBook({
          bookUrl,
          tocUrl: bookUrl,
          origin: 'loc_book',
          originName: '本地书仓',
          name,
          author: '',
          type: 0,
          group: 0,
          latestChapterTime: 0,
        } as Book)
        ok++
        shelfUrls.add(bookUrl)
      } catch {
        failed++
      }
    }
    importSummary.value =
      failed > 0
        ? `导入完成：${ok} 本成功，${skipped} 本已在书架，${failed} 本失败`
        : skipped > 0
          ? `导入完成：${ok} 本成功，${skipped} 本已在书架`
          : `导入完成，共 ${ok} 本`
    if (ok > 0) ElMessage.success(`已导入 ${ok} 本`)
    selected.value = new Set()
  } catch {
    importError.value = '导入失败，请稍后重试'
  } finally {
    importBusy.value = false
  }
}

function fmtSize(n: number | undefined): string {
  if (n == null || n < 0) return '—'
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

onMounted(async () => {
  // secure 模式探测（getUsers 返回 NEED_SECURE_KEY ⇒ secure）：决定权限提示文案
  secureMode.value = await probeSecureMode().catch(() => false)
  void loadList()
})
</script>

<template>
  <div class="store-page">
    <!-- 极简导航：字标 + 页面入口 -->
    <header class="topbar">
      <div class="brand">
        <img class="brand-logo" src="/logo.svg" alt="夜读" />
        <span class="brand-name">夜读<span class="brand-dot">.</span></span>
      </div>

      <div class="user-area">
        <button class="nav-link" type="button" @click="router.push('/')">书架</button>
        <button class="nav-link" type="button" @click="router.push('/search')">搜索</button>
        <button class="nav-link" type="button" @click="router.push('/files')">文件</button>
        <button class="nav-link active" type="button" @click="router.push('/store')">书仓</button>
        <button class="nav-link" type="button" @click="router.push('/settings')">设置</button>
        <span class="user-chip">{{ store.username || '未登录' }}</span>
      </div>
    </header>

    <main class="content">
      <div class="section-head">
        <h1 class="section-title">书仓</h1>
        <span class="count">{{ loading ? '…' : `${files.filter((f) => !f.isDirectory).length} 个文件` }}</span>
      </div>

      <!-- 权限提示（secure 模式需开启本地书仓） -->
      <div v-if="secureMode" class="perm-note">
        secure 模式：浏览书仓需当前用户已开启「本地书仓」权限（用户管理 → 权限），未开启将提示「未开启本地书仓功能」。
      </div>
      <div v-if="loadError" class="perm-note error">{{ loadError }}</div>

      <!-- 面包屑 -->
      <nav class="crumbs">
        <button class="crumb" :class="{ current: !path }" type="button" @click="goCrumb(-1)">书仓根</button>
        <template v-for="(c, i) in crumbs" :key="c.full">
          <span class="crumb-sep">/</span>
          <button class="crumb" :class="{ current: c.last }" type="button" @click="goCrumb(i)">
            {{ c.name }}
          </button>
        </template>
      </nav>

      <!-- 文件列表（书籍文件可勾选导入书架） -->
      <div class="file-list">
        <div v-if="loading" class="list-hint">加载中…</div>
        <div v-else-if="loadError" class="list-hint empty">书仓不可用：{{ loadError }}</div>
        <div v-else-if="files.length === 0" class="list-hint empty">书仓为空（storage/localStore）</div>
        <template v-else>
          <div
            v-for="item in files"
            :key="item.path"
            class="row"
            :class="{ selected: selected.has(item.path), book: !item.isDirectory && isBookFile(item) }"
          >
            <button
              class="row-select"
              type="button"
              :disabled="item.isDirectory"
              :title="item.isDirectory ? '目录不可导入' : selected.has(item.path) ? '取消勾选' : '勾选导入'"
              @click="toggleSelect(item)"
            >
              <span class="select-dot">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M5 12.5l4.5 4.5L19 7.5" />
                </svg>
              </span>
            </button>
            <button class="row-main" type="button" @click="item.isDirectory ? enterDir(item) : toggleSelect(item)">
              <svg
                v-if="item.isDirectory"
                class="row-icon dir"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
              </svg>
              <svg
                v-else
                class="row-icon file"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
                <path d="M14 3v5h5" />
              </svg>
              <span class="row-name" :title="item.name">{{ item.name }}</span>
              <span v-if="!item.isDirectory && isBookFile(item)" class="row-tag">可导入</span>
            </button>
            <span class="row-size">{{ item.isDirectory ? '—' : fmtSize(item.size) }}</span>
          </div>
        </template>
      </div>

      <p v-if="importSummary" class="import-note">{{ importSummary }}</p>
      <p v-if="importError" class="import-note error">{{ importError }}</p>

      <div class="import-bar">
        <span class="import-count">已选 {{ selectedBookCount }} 本</span>
        <button
          class="import-btn"
          type="button"
          :disabled="importBusy || selectedBookCount === 0"
          @click="importSelected"
        >
          {{ importBusy ? '导入中…' : '批量导入书架' }}
        </button>
      </div>
      <p class="card-note">
        点击文件勾选/取消；目录可进入浏览。导入 = 逐本加入书架（origin=loc_book 路径型文件书，
        阅读时按文件实时解析章节）——后端暂无批量导入接口，逐本导入中。已在书架的书自动跳过。
      </p>
    </main>
  </div>
</template>

<style scoped>
.store-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  animation: fade-in 0.2s ease both;
}

/* ================= 顶部导航（与文件页一致） ================= */
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
.user-area {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-left: auto;
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
.nav-link:hover,
.nav-link.active {
  color: var(--accent);
}
.user-chip {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-2);
}

/* ================= 内容区 ================= */
.content {
  width: min(760px, 100%);
  margin: 0 auto;
  padding: 40px 32px 72px;
}
.section-head {
  display: flex;
  align-items: baseline;
  gap: 14px;
  margin-bottom: 20px;
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

/* 权限提示 */
.perm-note {
  margin-bottom: 14px;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--accent-soft);
  color: var(--text-2);
  font-size: 12px;
  font-weight: 300;
  line-height: 1.7;
  letter-spacing: 0.5px;
}
.perm-note.error {
  background: rgba(207, 68, 68, 0.07);
  border-color: rgba(207, 68, 68, 0.35);
  color: #cf4444;
}

/* 面包屑 */
.crumbs {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  margin-bottom: 14px;
  overflow-x: auto;
  scrollbar-width: none;
}
.crumbs::-webkit-scrollbar {
  display: none;
}
.crumb {
  flex-shrink: 0;
  padding: 4px 2px;
  border: none;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 0.5px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.crumb:hover {
  color: var(--accent);
}
.crumb.current {
  color: var(--text-1);
  font-weight: 400;
}
.crumb-sep {
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= 文件列表 ================= */
.file-list {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.list-hint {
  padding: 48px 0;
  text-align: center;
  font-size: 13px;
  font-weight: 300;
  color: var(--text-3);
}
.list-hint.empty {
  letter-spacing: 2px;
}
.row {
  display: grid;
  grid-template-columns: 30px 1fr 96px;
  align-items: center;
  gap: 8px;
  padding: 9px 16px;
  border-bottom: 1px solid var(--border);
  transition: background-color 0.2s ease;
}
.row:last-child {
  border-bottom: none;
}
.row:hover {
  background: var(--hover);
}
.row.selected {
  background: var(--accent-soft);
}
.row-select {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
}
.row-select:disabled {
  cursor: default;
  opacity: 0.35;
}
.select-dot {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 15px;
  height: 15px;
  border-radius: 50%;
  border: 1px solid var(--border-strong);
  color: transparent;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease;
}
.select-dot svg {
  width: 8px;
  height: 8px;
  display: none;
}
.row.selected .select-dot {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
}
.row.selected .select-dot svg {
  display: block;
}
.row-select:hover:not(:disabled) .select-dot {
  border-color: var(--accent);
}
.row-main {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  padding: 2px 0;
  border: none;
  background: none;
  font-family: inherit;
  cursor: pointer;
  text-align: left;
}
.row-icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
}
.row-icon.dir {
  color: var(--accent);
}
.row-icon.file {
  color: var(--text-3);
}
.row-name {
  min-width: 0;
  font-size: 13.5px;
  font-weight: 300;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.2s ease;
}
.row-main:hover .row-name {
  color: var(--accent);
}
.row-tag {
  flex-shrink: 0;
  padding: 1px 8px;
  border-radius: 999px;
  border: 1px solid var(--border);
  color: var(--text-3);
  font-size: 10.5px;
  font-weight: 300;
  letter-spacing: 1px;
}
.row.selected .row-tag {
  color: var(--accent);
  border-color: var(--accent);
}
.row-size {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  text-align: right;
}

/* ================= 导入区 ================= */
.import-note {
  margin: 14px 0 0;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 0.5px;
  color: var(--text-2);
}
.import-note.error {
  color: #cf4444;
}
.import-bar {
  position: sticky;
  bottom: 20px;
  margin-top: 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 10px 16px 10px 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 999px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}
.import-count {
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2);
}
.import-btn {
  padding: 7px 18px;
  border-radius: 999px;
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
    opacity 0.2s ease;
}
.import-btn:hover:not(:disabled) {
  background: var(--accent-deep);
}
.import-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.card-note {
  margin: 12px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  line-height: 1.7;
  letter-spacing: 0.5px;
  color: var(--text-3);
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    padding: 12px 16px;
  }
  .content {
    padding: 28px 16px 56px;
  }
  .row {
    grid-template-columns: 30px 1fr;
    padding: 11px 12px;
    min-height: 40px;
  }
  .row-size {
    display: none;
  }
  .import-bar {
    bottom: max(12px, env(safe-area-inset-bottom));
    border-radius: 14px;
    flex-wrap: wrap;
  }
}
</style>
