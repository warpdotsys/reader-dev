<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import TopNav from '@/components/TopNav.vue'
import {
  listFiles,
  getFile,
  saveFile,
  downloadFile,
  uploadFile,
  mkdir,
  deleteFile,
  renameFile,
  setFileSecureKey,
  scanLocalBookDir,
} from '@/api/file'
import { restoreFromZip } from '@/api/backup'
import { isNeedSecureKey } from '@/api/users'
import { downloadBlob } from '@/utils/download'
import type { FileItem } from '@/types'

/** home 枚举（legacy 对齐）：书仓 / 用户数据 / WebDAV / 空=用户根 */
const HOME_OPTIONS: { label: string; value: string }[] = [
  { label: '书仓', value: '__LOCAL_STORE__' },
  { label: '用户数据', value: '__HOME__' },
  { label: 'WebDAV', value: '__WEBDAV__' },
  { label: '根', value: '' },
]

const home = ref('__LOCAL_STORE__')
const path = ref('')
const files = ref<FileItem[]>([])
const loading = ref(false)
const selectedPath = ref<string | null>(null)

/** 与后端 local_book::SUPPORTED_EXTENSIONS 对齐：文件页可导入书架的类型 */
const BOOK_EXTS = new Set(['epub', 'txt', 'mobi', 'azw3', 'pdf', 'fb2', 'docx', 'zip', 'cbz', 'umd'])

function fileExt(name: string): string {
  const idx = name.lastIndexOf('.')
  return idx >= 0 ? name.slice(idx + 1).toLowerCase() : ''
}

function isBookFile(item: FileItem | null): item is FileItem {
  return !!item && !item.isDirectory && BOOK_EXTS.has(fileExt(item.name))
}

const selectedItem = computed(() => files.value.find((f) => f.path === selectedPath.value) ?? null)

/* ---------------- 搜索（GAP 33：前端过滤当前目录列表，名称包含） ---------------- */
const searchKey = ref('')

/* ---------------- 排序（GAP 34：名称/大小/时间 + 升降序，目录恒优先） ---------------- */
type SortField = 'name' | 'size' | 'time'
const sortBy = ref<SortField>('name')
const sortDesc = ref(false)

const SORT_OPTIONS: { value: SortField; label: string }[] = [
  { value: 'name', label: '名称' },
  { value: 'size', label: '大小' },
  { value: 'time', label: '时间' },
]

/** 展示列表：名称过滤 → 目录在前 + 按字段/方向排序 */
const displayFiles = computed<FileItem[]>(() => {
  const kw = searchKey.value.trim().toLowerCase()
  let list = files.value
  if (kw) list = list.filter((f) => f.name.toLowerCase().includes(kw))
  const dirs = list.filter((f) => f.isDirectory)
  const others = list.filter((f) => !f.isDirectory)
  const cmp = (a: FileItem, b: FileItem): number => {
    let r = 0
    if (sortBy.value === 'size') r = (a.size ?? -1) - (b.size ?? -1)
    else if (sortBy.value === 'time') {
      const ta = typeof a.lastModified === 'number' ? a.lastModified : new Date(String(a.lastModified)).getTime() || 0
      const tb = typeof b.lastModified === 'number' ? b.lastModified : new Date(String(b.lastModified)).getTime() || 0
      r = ta - tb
    } else r = a.name.localeCompare(b.name)
    return sortDesc.value ? -r : r
  }
  return [...dirs.sort(cmp), ...others.sort(cmp)]
})

const homeLabel = computed(() => HOME_OPTIONS.find((o) => o.value === home.value)?.label ?? '根')

/* ---------------- 多选模式（右键/长按进入；底部操作条：下载/删除） ---------------- */
const multiMode = ref(false)
const multiSelected = ref<Set<string>>(new Set())
const multiBusy = ref(false)
let longPressTimer: number | undefined
let longPressFired = false

function exitMulti() {
  multiMode.value = false
  multiSelected.value = new Set()
}

function enterMulti(item: FileItem) {
  multiMode.value = true
  selectedPath.value = null
  multiSelected.value = new Set([item.path])
}

function toggleMulti(item: FileItem) {
  const s = new Set(multiSelected.value)
  if (s.has(item.path)) s.delete(item.path)
  else s.add(item.path)
  multiSelected.value = s
}

/** 右键：未多选时进入多选并选中该项；已多选时切换该项勾选 */
function onRowContext(item: FileItem, e: MouseEvent) {
  e.preventDefault()
  if (longPressFired) {
    // 长按已进入多选（合成 contextmenu 迟到），保持勾选状态
    longPressFired = false
    return
  }
  if (multiMode.value) {
    toggleMulti(item)
    return
  }
  enterMulti(item)
}

/** 长按 500ms 进入多选（与点击进文件互斥） */
function onRowTouchStart(item: FileItem) {
  longPressFired = false
  longPressTimer = window.setTimeout(() => {
    longPressFired = true
    enterMulti(item)
  }, 500)
}

function onRowTouchEnd() {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
    longPressTimer = undefined
  }
}

/** 行点击：多选模式 = 勾选切换；长按后合成 click 吞掉；否则原行为（进目录/预览/下载） */
function onRowClick(item: FileItem) {
  if (longPressFired) {
    longPressFired = false
    return
  }
  if (multiMode.value) {
    toggleMulti(item)
    return
  }
  enter(item)
}

/** 批量移动（GAP 补：后端无 MOVE 接口 → 文本文件 读→写→删 组合；目录/二进制提示） */
const moveOpen = ref(false)
const moveTarget = ref('')
const moveBusy = ref(false)

function openMultiMove() {
  if (!multiSelected.value.size) return
  moveTarget.value = ''
  moveOpen.value = true
}

async function doMultiMove() {
  if (moveBusy.value) return
  const target = moveTarget.value.trim().replace(/^\/+|\/+$/g, '')
  if (!target) {
    ElMessage.warning('请输入目标目录')
    return
  }
  const items = files.value.filter((f) => multiSelected.value.has(f.path))
  if (!items.length) return
  // 目标不能是选中项本身或其子路径
  if (items.some((it) => it.path === target || target.startsWith(it.path + '/'))) {
    ElMessage.warning('目标目录不能是选中项本身或其子目录')
    return
  }
  moveBusy.value = true
  try {
    let ok = 0
    const skipped: string[] = []
    for (const item of items) {
      if (item.isDirectory) {
        skipped.push(`${item.name}（目录）`)
        continue
      }
      if (!isTextFile(item.name)) {
        skipped.push(`${item.name}（非文本）`)
        continue
      }
      const newPath = joinPath(target, item.name)
      if (newPath === item.path) {
        skipped.push(`${item.name}（已在目标目录）`)
        continue
      }
      if (files.value.some((f) => f.path !== item.path && f.path === newPath)) {
        skipped.push(`${item.name}（同名已存在）`)
        continue
      }
      const moved = await runWrite(async () => {
        // file/save 自动建父目录；组合：读旧 → 写新 → 删旧（写失败则旧文件保留）
        const res = await getFile(item.path, home.value)
        await saveFile(newPath, res.data ?? '', home.value)
        await deleteFile(item.path, home.value)
      })
      if (moved) {
        ok++
      } else {
        skipped.push(item.name)
      }
    }
    moveOpen.value = false
    exitMulti()
    const base = ok > 0 ? `已移动 ${ok} 项` : '未移动任何文件'
    const extra = skipped.length
      ? `；跳过 ${skipped.length} 项（${skipped.slice(0, 3).join('、')}${skipped.length > 3 ? '…' : ''}）`
      : ''
    if (ok > 0) ElMessage.success(base + extra)
    else ElMessage.warning(base + extra)
    await loadList()
  } finally {
    moveBusy.value = false
  }
}

/** 多选下载：循环 GET file/download → downloadBlob */
async function multiDownload() {
  if (multiBusy.value) return
  const items = files.value.filter((f) => multiSelected.value.has(f.path))
  if (!items.length) return
  multiBusy.value = true
  let ok = 0
  for (const item of items) {
    try {
      const blob = await downloadFile(item.path, home.value)
      await downloadBlob(blob, item.name)
      ok++
    } catch {
      // 单个失败继续
    }
  }
  multiBusy.value = false
  ElMessage.success(ok === items.length ? `已下载 ${ok} 个文件` : `已下载 ${ok}/${items.length} 个文件`)
}

/** 多选删除：确认后循环 POST file/delete */
async function multiRemove() {
  if (multiBusy.value) return
  const items = files.value.filter((f) => multiSelected.value.has(f.path))
  if (!items.length) return
  try {
    await ElMessageBox.confirm(
      `确定删除选中的 ${items.length} 项吗？${items.some((f) => f.isDirectory) ? '目录及其内容将一并删除。' : ''}`,
      '批量删除',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  multiBusy.value = true
  let ok = 0
  for (const item of items) {
    const removed = await runWrite(async () => {
      await deleteFile(item.path, home.value)
    })
    if (removed) {
      ok++
    }
  }
  multiBusy.value = false
  exitMulti()
  ElMessage.success(ok === items.length ? `已删除 ${ok} 项` : `已删除 ${ok}/${items.length} 项`)
  await loadList()
}

/* ---------------- 弹窗状态 ---------------- */
const uploadOpen = ref(false)
const pickedFile = ref<File | null>(null)
const uploading = ref(false)
const uploadProgress = ref(0)
const mkdirOpen = ref(false)
const folderName = ref('')
const previewOpen = ref(false)
const previewItem = ref<FileItem | null>(null)
const previewContent = ref('')
const previewLoading = ref(false)
const previewEditing = ref(false)
const previewSaving = ref(false)
const renameOpen = ref(false)
const renameBusy = ref(false)
const renameTarget = ref<FileItem | null>(null)
const renameName = ref('')

const pickedName = computed(() => pickedFile.value?.name || '')

/* ---------------- 文本文件识别 ---------------- */
const TEXT_EXTS = new Set([
  'txt', 'json', 'md', 'markdown', 'log', 'ini', 'conf', 'cfg', 'xml', 'html', 'htm',
  'css', 'js', 'mjs', 'ts', 'csv', 'yml', 'yaml', 'toml', 'srt', 'vtt', 'lrc',
  'properties', 'sh', 'bat', 'cmd', 'sql', 'nfo', 'py', 'java', 'c', 'h', 'cpp', 'rs', 'go',
])

function isTextFile(name: string): boolean {
  const ext = name.includes('.') ? name.slice(name.lastIndexOf('.') + 1).toLowerCase() : ''
  return TEXT_EXTS.has(ext)
}

/** 预览大小上限：超过则点击文件直接下载（getFile 整体读入内存） */
const PREVIEW_MAX_SIZE = 5 * 1024 * 1024

/** secure 模式书仓（__LOCAL_STORE__）写/删需管理密码：弹窗输入并自动重试 */
const secureKey = ref('')
const secureKeyOpen = ref(false)
const secureBusy = ref(false)
let pendingSecureRetry: (() => Promise<void>) | null = null

async function runWrite(action: () => Promise<void>): Promise<boolean> {
  try {
    await action()
    return true
  } catch (err) {
    if (isNeedSecureKey(err)) {
      pendingSecureRetry = action
      secureKeyOpen.value = true
    }
    return false
  }
}

async function confirmSecureKey() {
  if (secureBusy.value) return
  secureBusy.value = true
  try {
    setFileSecureKey(secureKey.value.trim())
    const retry = pendingSecureRetry
    pendingSecureRetry = null
    secureKeyOpen.value = false
    if (retry) await retry()
  } finally {
    secureBusy.value = false
  }
}

function clearSecureKey() {
  setFileSecureKey('')
  secureKey.value = ''
  pendingSecureRetry = null
  secureKeyOpen.value = false
}

/* ---------------- 路径工具 ---------------- */
function joinPath(parent: string, name: string): string {
  if (!parent) return name
  return parent.replace(/\/+$/, '') + '/' + name
}

const crumbs = computed(() =>
  path.value
    .split('/')
    .filter(Boolean)
    .map((seg, i, arr) => ({ name: seg, full: arr.slice(0, i + 1).join('/'), last: i === arr.length - 1 })),
)

/* ---------------- 列表 ---------------- */
async function loadList() {
  loading.value = true
  try {
    const res = await listFiles(path.value, home.value)
    const list = (res.data || []) as FileItem[]
    // 目录在前，其余按名称排序
    files.value = [
      ...list.filter((f) => f.isDirectory).sort((a, b) => a.name.localeCompare(b.name)),
      ...list.filter((f) => !f.isDirectory).sort((a, b) => a.name.localeCompare(b.name)),
    ]
  } catch {
    files.value = []
  } finally {
    loading.value = false
  }
}

function switchHome(value: string) {
  if (value === home.value) return
  home.value = value
  path.value = ''
  selectedPath.value = null
  if (multiMode.value) exitMulti()
  void loadList()
}

function goCrumb(index: number) {
  path.value = index < 0 ? '' : crumbs.value[index].full
  selectedPath.value = null
  if (multiMode.value) exitMulti()
  void loadList()
}

function enter(item: FileItem) {
  if (item.isDirectory) {
    path.value = joinPath(path.value, item.name)
    selectedPath.value = null
    if (multiMode.value) exitMulti()
    void loadList()
    return
  }
  // 文本类小文件：预览（GET file/get）；其余：下载
  if (isTextFile(item.name) && (!(typeof item.size === 'number') || item.size <= PREVIEW_MAX_SIZE)) {
    void openPreview(item)
  } else {
    void download(item)
  }
}

function toggleSelect(item: FileItem) {
  if (multiMode.value) {
    toggleMulti(item)
    return
  }
  selectedPath.value = selectedPath.value === item.path ? null : item.path
}

/* ---------------- 下载 ---------------- */
async function download(item: FileItem) {
  try {
    const blob = await downloadFile(item.path, home.value)
    await downloadBlob(blob, item.name)
  } catch {
    // 请求层已提示
  }
}

/* ---------------- 预览（文本类文件，GET file/get） ---------------- */
async function openPreview(item: FileItem) {
  previewItem.value = item
  previewContent.value = ''
  previewOpen.value = true
  previewLoading.value = true
  try {
    const res = await getFile(item.path, home.value)
    previewContent.value = res.data ?? ''
  } catch {
    previewContent.value = ''
  } finally {
    previewLoading.value = false
  }
}

function closePreview() {
  previewOpen.value = false
  previewEditing.value = false
  previewSaving.value = false
  previewItem.value = null
  previewContent.value = ''
}

/** 编辑文本文件（legacy JSON 编辑器/文本查看 → 可编辑并保存） */
async function savePreview() {
  const item = previewItem.value
  if (!item || previewSaving.value) return
  previewSaving.value = true
  const saved = await runWrite(async () => {
    await saveFile(item.path, previewContent.value, home.value)
  })
  if (saved) {
    ElMessage.success('已保存')
    previewEditing.value = false
    await loadList()
  }
  previewSaving.value = false
}

/* ---------------- 重命名（POST /reader3/file/rename，文件/目录通用） ---------------- */
function openRename() {
  const target = files.value.find((f) => f.path === selectedPath.value)
  if (!target) return
  renameTarget.value = target
  renameName.value = target.name
  renameOpen.value = true
}

async function doRename() {
  const target = renameTarget.value
  if (!target || renameBusy.value) return
  const name = renameName.value.trim()
  if (!name) {
    ElMessage.warning('请输入新名称')
    return
  }
  if (name.includes('/') || name.includes('\\')) {
    ElMessage.warning('名称不能包含路径分隔符')
    return
  }
  if (name.startsWith('.')) {
    ElMessage.warning('名称不能以 . 开头')
    return
  }
  const dir = target.path.includes('/') ? target.path.slice(0, target.path.lastIndexOf('/')) : ''
  const newPath = joinPath(dir, name)
  if (newPath === target.path) {
    renameOpen.value = false
    return
  }
  if (files.value.some((f) => f.path !== target.path && f.path === newPath)) {
    ElMessage.warning('同名文件已存在')
    return
  }
  renameBusy.value = true
  const renamed = await runWrite(async () => {
    await renameFile(target.path, name, home.value)
  })
  if (renamed) {
    ElMessage.success('重命名成功')
    renameOpen.value = false
    selectedPath.value = null
    await loadList()
  }
  renameBusy.value = false
}

/* ---------------- 上传 ---------------- */
function openUpload() {
  pickedFile.value = null
  uploadOpen.value = true
}

function onPick(e: Event) {
  const input = e.target as HTMLInputElement
  pickedFile.value = input.files?.[0] ?? null
}

async function doUpload() {
  const file = pickedFile.value
  if (!file) {
    ElMessage.warning('请先选择文件')
    return
  }
  uploading.value = true
  uploadProgress.value = 0
  const uploaded = await runWrite(async () => {
    await uploadFile(file, path.value, home.value, (p) => (uploadProgress.value = p))
  })
  if (uploaded) {
    ElMessage.success('上传成功')
    uploadOpen.value = false
    pickedFile.value = null
    await loadList()
  }
  uploading.value = false
}

/* ---------------- 新建文件夹 ---------------- */
async function doMkdir() {
  const name = folderName.value.trim()
  if (!name) {
    ElMessage.warning('请输入文件夹名称')
    return
  }
  const created = await runWrite(async () => {
    await mkdir(path.value, name, home.value)
  })
  if (created) {
    ElMessage.success('创建成功')
    mkdirOpen.value = false
    folderName.value = ''
    await loadList()
  }
}

/* ---------------- 删除 ---------------- */
async function removeSelected() {
  const target = files.value.find((f) => f.path === selectedPath.value)
  if (!target) return
  try {
    await ElMessageBox.confirm(
      `确定删除「${target.name}」吗？${target.isDirectory ? '目录及其内容将一并删除。' : ''}`,
      '删除',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  const removed = await runWrite(async () => {
    await deleteFile(target.path, home.value)
  })
  if (removed) {
    ElMessage.success('已删除')
    selectedPath.value = null
    await loadList()
  }
}

/* ---------------- 从备份 zip 还原（restoreFromZip：上传备份文件恢复书源/书架等） ---------------- */
const restoreOpen = ref(false)
const restoreFile = ref<File | null>(null)
const restoreOverwrite = ref(false)
const restoreBusy = ref(false)

function openRestore() {
  restoreFile.value = null
  restoreOverwrite.value = false
  restoreBusy.value = false
  restoreOpen.value = true
}

function onRestorePick(e: Event) {
  const input = e.target as HTMLInputElement
  restoreFile.value = input.files?.[0] ?? null
}

async function doRestore() {
  if (restoreBusy.value) return
  const file = restoreFile.value
  if (!file) {
    ElMessage.warning('请选择备份文件')
    return
  }
  restoreBusy.value = true
  const restored = await runWrite(async () => {
    const res = await restoreFromZip(file, file.name, restoreOverwrite.value)
    const data = res.data
    const restored = data?.restored ?? {}
    const restoredText = Object.entries(restored)
      .filter(([, n]) => n > 0)
      .map(([k, n]) => `${k} ${n}`)
      .join('、')
    ElMessage.success(
      restoredText
        ? `已还原：${restoredText}`
        : '还原完成（未覆盖已存在数据，可开启覆盖后重试）',
    )
    restoreOpen.value = false
    restoreFile.value = null
    restoreBusy.value = false
  })
  if (!restored) {
    restoreBusy.value = false
  }
}

/* ---------------- 导入书架（书仓/用户目录/WebDAV 直接解析，无需重新上传） ---------------- */
const importOpen = ref(false)
const importBusy = ref(false)

function openImportBook() {
  if (!selectedItem.value || !isBookFile(selectedItem.value)) return
  importOpen.value = true
}

async function doImportBook() {
  const item = selectedItem.value
  if (importBusy.value || !item || !isBookFile(item)) return
  importBusy.value = true
  try {
    const res = await scanLocalBookDir(item.path, home.value, false)
    if (!res.isSuccess) {
      ElMessage.error(res.errorMsg || '导入失败')
    } else if ((res.data?.imported ?? 0) > 0) {
      ElMessage.success(`已导入书架：${item.name}`)
      importOpen.value = false
    } else {
      ElMessage.warning(res.data?.errors?.[0]?.error || '未解析到章节内容')
    }
  } catch {
    // 请求层已提示
  } finally {
    importBusy.value = false
  }
}

/** 目录整体导入：递归扫描当前目录下所有书籍文件，直接解析入架 */
const scanDirOpen = ref(false)
const scanDirBusy = ref(false)

function openScanDir() {
  scanDirOpen.value = true
}

async function doScanDir() {
  if (scanDirBusy.value) return
  scanDirBusy.value = true
  try {
    const res = await scanLocalBookDir(path.value || '/', home.value, true)
    if (!res.isSuccess) {
      ElMessage.error(res.errorMsg || '导入失败')
    } else {
      const data = res.data
      const imported = data?.imported ?? 0
      const failed = data?.failed ?? 0
      if (imported > 0) {
        ElMessage.success(`已导入书架 ${imported} 本${failed ? `，${failed} 本失败` : ''}`)
      } else if (failed > 0) {
        ElMessage.warning(data?.errors?.[0]?.error || `导入失败 ${failed} 本`)
      } else {
        ElMessage.warning('目录中未找到可导入的书籍文件')
      }
      scanDirOpen.value = false
    }
  } catch {
    // 请求层已提示
  } finally {
    scanDirBusy.value = false
  }
}

/** 多选批量导入：逐文件直接解析入架 */
async function multiImport() {
  if (multiBusy.value) return
  const items = files.value.filter((f) => multiSelected.value.has(f.path) && isBookFile(f))
  if (!items.length) {
    ElMessage.warning('请选择书籍文件')
    return
  }
  multiBusy.value = true
  let ok = 0
  const errors: string[] = []
  for (const item of items) {
    try {
      const res = await scanLocalBookDir(item.path, home.value, false)
      if (res.isSuccess && (res.data?.imported ?? 0) > 0) ok++
      else errors.push(`${item.name}：${res.data?.errors?.[0]?.error || res.errorMsg || '导入失败'}`)
    } catch {
      errors.push(`${item.name}：导入失败`)
    }
  }
  multiBusy.value = false
  if (ok > 0) ElMessage.success(`已导入书架 ${ok} 本${errors.length ? `，失败 ${errors.length} 本` : ''}`)
  else if (errors.length) ElMessage.warning(errors[0])
  else ElMessage.warning('未选择书籍文件')
}

/* ---------------- 展示格式化 ---------------- */
function formatSize(n: number | undefined): string {
  if (n == null || n < 0) return '—'
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`
}

function formatTime(v: number | string | undefined): string {
  if (v == null || v === '') return '—'
  if (typeof v === 'number') {
    const d = new Date(v)
    if (Number.isNaN(d.getTime())) return '—'
    const p = (n: number) => String(n).padStart(2, '0')
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
  }
  return String(v)
}

onMounted(() => {
  void loadList()
})

onBeforeUnmount(() => {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
    longPressTimer = undefined
  }
})
</script>

<template>
  <div class="file-page">
    <!-- 顶部导航（P3-A：共享 TopNav） -->
    <TopNav active="/files" :links="['bookshelf', 'search', 'sources', 'files', 'users', 'settings']" show-users-link />

    <main class="content" :class="{ 'with-multi-bar': multiMode }">
      <div class="section-head">
        <h1 class="section-title">文件</h1>
        <span class="count">{{ loading ? '…' : `${displayFiles.length} 项` }}</span>
      </div>

      <!-- home 切换胶囊 -->
      <div class="home-pills">
        <button
          v-for="opt in HOME_OPTIONS"
          :key="opt.value"
          class="pill"
          :class="{ active: home === opt.value }"
          type="button"
          @click="switchHome(opt.value)"
        >
          {{ opt.label }}
        </button>
      </div>

      <!-- 面包屑 + 工具栏 -->
      <div class="file-bar">
        <nav class="crumbs">
          <button class="crumb" :class="{ current: !path }" type="button" @click="goCrumb(-1)">
            根
          </button>
          <template v-for="(c, i) in crumbs" :key="c.full">
            <span class="crumb-sep">/</span>
            <button class="crumb" :class="{ current: c.last }" type="button" @click="goCrumb(i)">
              {{ c.name }}
            </button>
          </template>
        </nav>

        <div class="toolbar">
          <div class="sort-box" title="排序">
            <select v-model="sortBy" class="sort-select" aria-label="排序字段">
              <option v-for="o in SORT_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
            </select>
            <button
              class="sort-dir"
              type="button"
              :title="sortDesc ? '降序' : '升序'"
              @click="sortDesc = !sortDesc"
            >
              {{ sortDesc ? '↓' : '↑' }}
            </button>
          </div>
          <button class="tool-btn" type="button" title="从备份 zip 恢复书源、书架、分组、RSS、替换规则、书签、配置等" @click="openRestore">
            还原备份
          </button>
          <button
            class="tool-btn"
            type="button"
            :disabled="!selectedItem || !isBookFile(selectedItem)"
            :title="selectedItem && !isBookFile(selectedItem) ? '请先选中一个书籍文件（epub/txt/mobi/azw3/pdf/fb2/docx/zip/cbz/umd）' : '选中书籍文件后导入书架'"
            @click="openImportBook"
          >
            导入书架
          </button>
          <button
            class="tool-btn"
            type="button"
            title="递归扫描当前目录中已有的书籍文件并直接导入书架"
            @click="openScanDir"
          >
            导入目录
          </button>
          <button class="tool-btn" type="button" @click="openUpload">上传</button>
          <button class="tool-btn" type="button" @click="mkdirOpen = true">新建文件夹</button>
          <button
            class="tool-btn"
            type="button"
            :disabled="!selectedPath"
            title="文本文件可用"
            @click="openRename"
          >
            重命名
          </button>
          <button
            class="tool-btn danger"
            type="button"
            :disabled="!selectedPath"
            @click="removeSelected"
          >
            删除
          </button>
        </div>
      </div>

      <!-- 搜索（GAP 33：前端过滤当前目录，名称包含） -->
      <div class="filter-bar">
        <div class="search-box">
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
            v-model="searchKey"
            class="search-input"
            type="text"
            placeholder="搜索当前目录"
            spellcheck="false"
          />
          <button v-if="searchKey" class="search-clear" type="button" title="清空" @click="searchKey = ''">
            ✕
          </button>
        </div>
      </div>

      <!-- 文件列表 -->
      <div class="file-list">
        <div v-if="loading" class="list-hint">加载中…</div>
        <div v-else-if="files.length === 0" class="list-hint empty">此目录为空</div>
        <div v-else-if="displayFiles.length === 0" class="list-hint empty">无匹配「{{ searchKey.trim() }}」的文件</div>
        <template v-else>
          <div
            v-for="item in displayFiles"
            :key="item.path"
            class="row"
            :class="{
              selected: selectedPath === item.path,
              multi: multiMode && multiSelected.has(item.path),
            }"
            @contextmenu="onRowContext(item, $event)"
            @touchstart.passive="onRowTouchStart(item)"
            @touchend="onRowTouchEnd"
            @touchcancel="onRowTouchEnd"
          >
            <button
              class="row-select"
              type="button"
              :title="multiMode ? (multiSelected.has(item.path) ? '取消勾选' : '勾选') : selectedPath === item.path ? '取消选中' : '选中'"
              @click="toggleSelect(item)"
            >
              <span class="select-dot">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M5 12.5l4.5 4.5L19 7.5" />
                </svg>
              </span>
            </button>
            <button class="row-main" type="button" @click="onRowClick(item)">
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
              <span class="row-name">{{ item.name }}</span>
            </button>
            <span class="row-size">{{ item.isDirectory ? '—' : formatSize(item.size) }}</span>
            <span class="row-time">{{ formatTime(item.lastModified) }}</span>
          </div>
        </template>
      </div>
    </main>

    <!-- 多选底部操作条（细字：已选 N 项 + 下载/删除/完成；fade 200ms） -->
    <Transition name="bar">
      <div v-if="multiMode" class="multi-bar">
        <span class="multi-count">已选 {{ multiSelected.size }} 项</span>
        <div class="multi-actions">
          <button
            class="multi-act accent"
            type="button"
            :disabled="multiSelected.size === 0 || multiBusy"
            @click="multiImport"
          >
            {{ multiBusy ? '处理中…' : '导入书架' }}
          </button>
          <button
            class="multi-act"
            type="button"
            :disabled="multiSelected.size === 0 || multiBusy"
            @click="multiDownload"
          >
            {{ multiBusy ? '处理中…' : '下载' }}
          </button>
          <button
            class="multi-act"
            type="button"
            :disabled="multiSelected.size === 0 || multiBusy"
            @click="openMultiMove"
          >
            移动
          </button>
          <button
            class="multi-act danger"
            type="button"
            :disabled="multiSelected.size === 0 || multiBusy"
            @click="multiRemove"
          >
            删除
          </button>
          <button class="multi-act accent" type="button" :disabled="multiBusy" @click="exitMulti">
            完成
          </button>
        </div>
      </div>
    </Transition>

    <!-- 上传弹窗 -->
    <div v-if="uploadOpen" class="dlg-overlay" @click.self="uploadOpen = false">
      <div class="dlg">
        <h3 class="dlg-title">上传文件</h3>
        <p class="dlg-path">目标目录：{{ path || '根目录' }}</p>
        <label class="file-pick" :class="{ picked: pickedFile }">
          <input type="file" @change="onPick" />
          <span>{{ pickedName || '选择文件' }}</span>
        </label>
        <div v-if="uploading" class="upload-progress">
          <div class="upload-progress-wrap">
            <div class="upload-progress-bar" :style="{ width: `${uploadProgress}%` }" />
          </div>
          <span class="upload-progress-text">{{ uploadProgress }}%</span>
        </div>
        <div class="dlg-actions">
          <button class="btn-plain" type="button" @click="uploadOpen = false">取消</button>
          <button
            class="btn-primary"
            type="button"
            :disabled="!pickedFile || uploading"
            @click="doUpload"
          >
            {{ uploading ? '上传中…' : '上传' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 从备份 zip 还原弹窗 -->
    <div v-if="restoreOpen" class="dlg-overlay" @click.self="restoreBusy ? null : (restoreOpen = false)">
      <div class="dlg">
        <h3 class="dlg-title">从备份还原</h3>
        <p class="dlg-path">恢复书源、书架、分组、RSS 订阅、替换规则、书签、用户配置等</p>
        <label class="file-pick" :class="{ picked: restoreFile }">
          <input type="file" accept=".zip" @change="onRestorePick" />
          <span>{{ restoreFile?.name || '选择备份 zip' }}</span>
        </label>
        <label class="restore-opt">
          <button
            class="switch"
            :class="{ on: restoreOverwrite }"
            type="button"
            role="switch"
            :aria-checked="restoreOverwrite"
            @click="restoreOverwrite = !restoreOverwrite"
          >
            <span class="switch-knob"></span>
          </button>
          <span class="restore-opt-label">覆盖已存在数据</span>
        </label>
        <p class="rename-tip">默认逐项幂等：已存在的书源/书籍/分组等跳过，不会重复导入；开启覆盖后以备份为准。</p>
        <div class="dlg-actions">
          <button class="btn-plain" type="button" :disabled="restoreBusy" @click="restoreOpen = false">取消</button>
          <button class="btn-primary" type="button" :disabled="!restoreFile || restoreBusy" @click="doRestore">
            {{ restoreBusy ? '还原中…' : '开始还原' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 导入书架确认弹窗 -->
    <div v-if="importOpen" class="dlg-overlay" @click.self="importBusy ? null : (importOpen = false)">
      <div class="dlg">
        <h3 class="dlg-title">导入书架</h3>
        <p class="dlg-path" :title="selectedItem?.name">{{ selectedItem?.name }}</p>
        <p class="rename-tip">直接读取该文件并解析加入书架，无需重新上传。</p>
        <div class="dlg-actions">
          <button class="btn-plain" type="button" :disabled="importBusy" @click="importOpen = false">取消</button>
          <button class="btn-primary" type="button" :disabled="importBusy" @click="doImportBook">
            {{ importBusy ? '导入中…' : '确认导入' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 目录整体导入确认弹窗 -->
    <div v-if="scanDirOpen" class="dlg-overlay" @click.self="scanDirBusy ? null : (scanDirOpen = false)">
      <div class="dlg">
        <h3 class="dlg-title">导入目录</h3>
        <p class="dlg-path" :title="path || '根目录'">{{ path || '根目录' }}</p>
        <p class="rename-tip">递归读取当前目录及子目录中的书籍文件并直接导入书架。</p>
        <div class="dlg-actions">
          <button class="btn-plain" type="button" :disabled="scanDirBusy" @click="scanDirOpen = false">取消</button>
          <button class="btn-primary" type="button" :disabled="scanDirBusy" @click="doScanDir">
            {{ scanDirBusy ? '导入中…' : '确认导入' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 新建文件夹弹窗 -->
    <div v-if="mkdirOpen" class="dlg-overlay" @click.self="mkdirOpen = false">
      <div class="dlg">
        <h3 class="dlg-title">新建文件夹</h3>
        <p class="dlg-path">位置：{{ path || '根目录' }}</p>
        <input
          v-model="folderName"
          class="dlg-input"
          type="text"
          placeholder="文件夹名称"
          spellcheck="false"
          @keyup.enter="doMkdir"
        />
        <div class="dlg-actions">
          <button class="btn-plain" type="button" @click="mkdirOpen = false">取消</button>
          <button
            class="btn-primary"
            type="button"
            :disabled="!folderName.trim()"
            @click="doMkdir"
          >
            创建
          </button>
        </div>
      </div>
    </div>

    <!-- 文件预览弹窗（txt 类，GET file/get） -->
    <div v-if="previewOpen" class="dlg-overlay" @click.self="closePreview">
      <div class="dlg preview-dlg">
        <div class="preview-head">
          <h3 class="dlg-title" :title="previewItem?.name">{{ previewItem?.name }}</h3>
          <div class="preview-actions">
            <button
              class="btn-plain"
              type="button"
              :disabled="!previewItem"
              @click="previewItem && download(previewItem)"
            >
              下载
            </button>
            <button
              v-if="!previewEditing"
              class="btn-plain"
              type="button"
              :disabled="previewLoading || !previewItem || !isTextFile(previewItem.name)"
              :title="previewItem && isTextFile(previewItem.name) ? '编辑并保存到服务器' : '仅文本文件可编辑'"
              @click="previewEditing = true"
            >
              编辑
            </button>
            <button
              v-else
              class="btn-primary"
              type="button"
              :disabled="previewSaving"
              @click="savePreview"
            >
              {{ previewSaving ? '保存中…' : '保存' }}
            </button>
            <button class="btn-plain" type="button" @click="closePreview">关闭</button>
          </div>
        </div>
        <div class="preview-body">
          <p v-if="previewLoading" class="list-hint">加载中…</p>
          <textarea
            v-else-if="previewEditing"
            v-model="previewContent"
            class="preview-editor"
            spellcheck="false"
            :disabled="previewSaving"
          ></textarea>
          <pre v-else class="preview-content">{{ previewContent }}</pre>
        </div>
      </div>
    </div>

    <!-- 批量移动弹窗（GAP 补：文本文件 读→写→删 组合；目录/二进制提示后端 MOVE 未就绪） -->
    <div v-if="moveOpen" class="dlg-overlay" @click.self="moveOpen = false">
      <div class="dlg">
        <h3 class="dlg-title">移动 {{ multiSelected.size }} 项</h3>
        <p class="dlg-path">目标目录（{{ homeLabel }} 根下相对路径）</p>
        <input
          v-model="moveTarget"
          class="dlg-input"
          type="text"
          placeholder="如 books/子目录（不存在将自动创建）"
          spellcheck="false"
          @keyup.enter="doMultiMove"
        />
        <p class="rename-tip">
          以「读取内容 → 写入新路径 → 删除旧文件」组合实现（仅文本文件）；目录与二进制文件暂不支持——后端 MOVE 接口未就绪。
        </p>
        <div class="dlg-actions">
          <button class="btn-plain" type="button" :disabled="moveBusy" @click="moveOpen = false">取消</button>
          <button
            class="btn-primary"
            type="button"
            :disabled="moveBusy || !moveTarget.trim()"
            @click="doMultiMove"
          >
            {{ moveBusy ? '移动中…' : '移动' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 重命名弹窗（POST /reader3/file/rename，文件/目录通用） -->
    <div v-if="renameOpen" class="dlg-overlay" @click.self="renameOpen = false">
      <div class="dlg">
        <h3 class="dlg-title">重命名</h3>
        <p class="dlg-path">{{ renameTarget?.path }}</p>
        <input
          v-model="renameName"
          class="dlg-input"
          type="text"
          placeholder="新名称"
          spellcheck="false"
          @keyup.enter="doRename"
        />
        <p class="rename-tip">文件与目录均支持重命名；secure 模式书仓写操作需管理密码。</p>
        <div class="dlg-actions">
          <button class="btn-plain" type="button" :disabled="renameBusy" @click="renameOpen = false">
            取消
          </button>
          <button
            class="btn-primary"
            type="button"
            :disabled="renameBusy || !renameName.trim()"
            @click="doRename"
          >
            {{ renameBusy ? '重命名中…' : '确定' }}
          </button>
        </div>
      </div>
    </div>

    <!-- secure 模式书仓写/删管理密码 -->
    <div v-if="secureKeyOpen" class="dlg-overlay" @click.self="secureBusy ? null : (secureKeyOpen = false)">
      <div class="dlg">
        <h3 class="dlg-title">管理密码</h3>
        <p class="dlg-path">当前为安全模式（secure），书仓写/删操作需要管理密码（secureKey）。</p>
        <input
          v-model="secureKey"
          class="dlg-input mono"
          type="password"
          placeholder="secureKey"
          autocomplete="off"
          spellcheck="false"
          @keyup.enter="confirmSecureKey"
        />
        <div class="dlg-actions">
          <button class="btn-plain" type="button" :disabled="secureBusy" @click="clearSecureKey">
            取消
          </button>
          <button class="btn-primary" type="button" :disabled="secureBusy || !secureKey.trim()" @click="confirmSecureKey">
            {{ secureBusy ? '验证中…' : '确认' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  animation: fade-in 0.2s ease both;
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
  margin-bottom: 24px;
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

/* ================= home 切换胶囊 ================= */
.home-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 18px;
}
.pill {
  padding: 6px 16px;
  border-radius: 999px;
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
    background-color 0.2s ease,
    border-color 0.2s ease;
}
.pill:hover {
  color: var(--accent);
  border-color: var(--border-strong);
}
.pill.active {
  color: var(--accent);
  background: var(--accent-soft);
  border-color: var(--accent);
}

/* ================= 面包屑 + 工具栏 ================= */
.file-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 14px;
}
.crumbs {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  flex: 1;
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
.toolbar {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.tool-btn {
  padding: 5px 14px;
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
    border-color 0.2s ease,
    opacity 0.2s ease;
}
.tool-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.tool-btn.danger:hover:not(:disabled) {
  color: #cf4444;
  border-color: #cf4444;
}
.tool-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

/* ================= 搜索 + 排序 ================= */
.filter-bar {
  margin-bottom: 10px;
}
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 38px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--surface);
  transition: border-color 0.2s ease;
}
.search-box:focus-within {
  border-color: var(--accent);
}
.search-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--text-3);
  transition: color 0.2s ease;
}
.search-box:focus-within .search-icon {
  color: var(--accent);
}
.search-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: none;
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 0.5px;
  outline: none;
}
.search-input::placeholder {
  color: var(--text-3);
}
.search-clear {
  flex-shrink: 0;
  border: none;
  background: none;
  color: var(--text-3);
  font-size: 12px;
  cursor: pointer;
  padding: 2px 4px;
}
.search-clear:hover {
  color: var(--accent);
}
.sort-box {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.sort-select {
  height: 28px;
  padding: 0 6px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 300;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.sort-select:focus {
  border-color: var(--accent);
}
.sort-dir {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.sort-dir:hover {
  color: var(--accent);
  border-color: var(--accent);
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
  grid-template-columns: 30px 1fr 96px 148px;
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
.row.multi {
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
.row.multi .select-dot {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
}
.row.multi .select-dot svg {
  display: block;
}
.row-select:hover .select-dot {
  border-color: var(--accent);
}
.row.selected .select-dot {
  background: var(--accent);
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
.row-size,
.row-time {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
}
.row-time {
  font-variant-numeric: tabular-nums;
}

/* ================= 弹窗 ================= */
.dlg-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(24, 24, 27, 0.32);
  animation: fade-in 0.2s ease both;
}
.dlg {
  width: min(360px, 100%);
  padding: 24px 24px 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 12px 40px rgba(24, 24, 27, 0.12);
}
.dlg-title {
  margin: 0 0 6px;
  font-size: 15px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.dlg-path {
  margin: 0 0 16px;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.file-pick {
  display: block;
  padding: 12px 14px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius);
  color: var(--text-3);
  font-size: 12.5px;
  font-weight: 300;
  text-align: center;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.file-pick:hover,
.file-pick.picked {
  color: var(--accent);
  border-color: var(--accent);
}
.file-pick input {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  white-space: nowrap;
  border: 0;
}
.dlg-input {
  box-sizing: border-box;
  width: 100%;
  padding: 10px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  outline: none;
  transition: border-color 0.2s ease;
}
.dlg-input::placeholder {
  color: var(--text-3);
}
.dlg-input:focus {
  border-color: var(--accent);
}

/* 上传进度条 */
.upload-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
}
.upload-progress-wrap {
  flex: 1;
  height: 4px;
  border-radius: 2px;
  background: var(--border);
  overflow: hidden;
}
.upload-progress-bar {
  height: 100%;
  border-radius: 2px;
  background: var(--accent);
  transition: width 0.15s ease;
}
.upload-progress-text {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  font-variant-numeric: tabular-nums;
  color: var(--text-3);
}

/* 预览弹窗 */
.preview-dlg {
  width: min(620px, 100%);
  display: flex;
  flex-direction: column;
  max-height: 82vh;
}
.preview-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}
.preview-head .dlg-title {
  margin: 0;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.preview-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.preview-body {
  min-height: 120px;
  max-height: 56vh;
  overflow: auto;
  padding: 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
}
.preview-body .list-hint {
  padding: 40px 0;
}
.preview-content {
  margin: 0;
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-2);
  white-space: pre-wrap;
  word-break: break-all;
}
.preview-editor {
  display: block;
  box-sizing: border-box;
  width: 100%;
  min-height: 38vh;
  margin: 0;
  padding: 12px 14px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--bg-2, var(--bg));
  color: var(--text-1);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.7;
  resize: vertical;
  outline: none;
}
.preview-editor:focus {
  border-color: var(--accent);
}

/* 重命名提示 */
.rename-tip {
  margin: 10px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-3);
}
.restore-opt {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
  padding: 10px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
}
.restore-opt-label {
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2);
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
.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
}
.btn-plain,
.btn-primary {
  padding: 7px 18px;
  border-radius: var(--radius);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    background-color 0.2s ease,
    border-color 0.2s ease,
    opacity 0.2s ease;
}
.btn-plain {
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
}
.btn-plain:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.btn-primary {
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--on-accent);
}
.btn-primary:hover:not(:disabled) {
  background: var(--accent-deep);
  border-color: var(--accent-deep);
}
.btn-primary:disabled {
  opacity: 0.45;
  cursor: default;
}

/* ================= 多选底部操作条（细字胶囊，fade 200ms） ================= */
.multi-bar {
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
.multi-count {
  font-size: 12.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2);
}
.multi-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.multi-act {
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
.multi-act:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.multi-act.danger {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.35);
}
.multi-act.danger:hover:not(:disabled) {
  background: rgba(207, 68, 68, 0.08);
  border-color: #cf4444;
}
.multi-act.accent {
  color: var(--accent);
  border-color: var(--accent);
}
.multi-act.accent:hover:not(:disabled) {
  background: var(--accent-soft);
  border-color: var(--accent-deep);
  color: var(--accent-deep);
}
.multi-act:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
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
.content.with-multi-bar {
  padding-bottom: 150px;
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    padding: 12px 16px;
  }
  .content {
    padding: 28px 16px 56px;
  }
  .file-bar {
    align-items: stretch;
    flex-wrap: nowrap;
  }
  .toolbar {
    flex: 1 1 auto;
    min-width: 0;
    width: 100%;
    overflow-x: auto;
    flex-wrap: nowrap;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
  }
  .toolbar::-webkit-scrollbar {
    display: none;
  }
  .toolbar .tool-btn,
  .toolbar .sort-box {
    flex-shrink: 0;
  }
  .home-pills {
    flex-wrap: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .home-pills::-webkit-scrollbar {
    display: none;
  }
  .pill {
    flex-shrink: 0;
    min-height: 40px;
  }
  .file-bar {
    flex-direction: column;
    align-items: stretch;
  }
  .toolbar {
    justify-content: flex-start;
  }
  .row {
    /* 移动端简化：仅选择点 + 名称 */
    grid-template-columns: 30px 1fr;
    padding: 11px 12px;
    min-height: 40px;
  }
  .row-size,
  .row-time {
    display: none;
  }
  .dlg-overlay {
    padding: 16px;
  }
}
</style>
