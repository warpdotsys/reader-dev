<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  deleteBookSource,
  deleteBookSources,
  getBookSources,
  getInvalidBookSources,
  previewRemoteSource,
  saveBookSource,
  saveBookSources,
  setAsDefaultBookSources,
} from '@/api/sources'
import {
  deleteSourceSub,
  deleteSourceSubs,
  getSourceSubs,
  previewSourceSub,
  refreshSourceSub,
  saveSourceSub,
  setSourceSubEnabled,
} from '@/api/sourceSubs'
import { exportBookSources } from '@/api/system'
import { bookSourceDebugSSE, type DebugAction } from '@/api/sourceDebug'
import {
  getCaptcha,
  getBookSourceCookie,
  loginBookSource,
  setBookSourceCookie,
  submitCaptcha,
  type BookSourceLoginResult,
  type CaptchaProbe,
} from '@/api/sourceLogin'
import { downloadBlob } from '@/utils/download'
import { t } from '@/utils/i18n'
import TopNav from '@/components/TopNav.vue'
import { useUserStore } from '@/stores/user'
import { hanText, syncHanMode } from '@/utils/hanMode'
import { isNotImplemented } from '@/utils/errors'
import type { BookSource, CookieRow, SourceSub } from '@/types'

const router = useRouter()
const store = useUserStore()

/* ================= 列表 ================= */
const sources = ref<BookSource[]>([])
const loading = ref(true)
const errorMsg = ref('')

async function load() {
  loading.value = true
  errorMsg.value = ''
  try {
    const res = await getBookSources()
    sources.value = res.data ?? []
    // 探测默认书源字段：后端 getBookSources 返回 data[].isDefault 才显示「默认」标记（无则整列隐藏）
    defaultField.value = sources.value.some((s) => typeof (s as { isDefault?: unknown }).isDefault === 'boolean')
  } catch (err) {
    errorMsg.value = err instanceof Error ? err.message : '加载书源失败'
  } finally {
    loading.value = false
  }
}

/* ================= 分组筛选（细字胶囊） ================= */
const activeGroup = ref('全部')
/** 分组 token 拆分：兼容 legacy 逗号/顿号/全角逗号/空白分隔（旧数据常为 "漫画,已检验"） */
function splitGroups(raw: string | null | undefined): string[] {
  if (!raw) return []
  const seen = new Set<string>()
  for (const token of raw.split(/[,，、\s]+/)) {
    const g = token.trim()
    if (g && !seen.has(g)) seen.add(g)
  }
  return Array.from(seen)
}

const groups = computed(() => {
  const set = new Set<string>()
  for (const s of sources.value) {
    for (const g of splitGroups(s.bookSourceGroup)) {
      if (g !== '全部') set.add(g)
    }
  }
  return Array.from(set).sort()
})

/* ================= GAP 28：分组管理（胶囊长按/右键 → 重命名/删除组，批量改 bookSourceGroup） ================= */

const groupMenu = ref<{ name: string; x: number; y: number } | null>(null)
const groupMenuBusy = ref(false)
let groupLongPressTimer: number | undefined
let groupLongPressFired = false

function openGroupMenu(name: string, x: number, y: number) {
  groupMenu.value = {
    name,
    x: Math.min(Math.max(8, x), window.innerWidth - 170),
    y: Math.min(Math.max(8, y), window.innerHeight - 120),
  }
}

function closeGroupMenu() {
  groupMenu.value = null
}

/** 触屏长按 500ms 唤出分组菜单（与点击筛选互斥） */
function onGroupTouchStart(name: string, e: TouchEvent) {
  groupLongPressFired = false
  const t = e.touches[0]
  window.clearTimeout(groupLongPressTimer)
  groupLongPressTimer = window.setTimeout(() => {
    groupLongPressFired = true
    openGroupMenu(name, t.clientX, t.clientY)
  }, 500)
}

/** 胶囊点击：长按唤出菜单后吞掉紧随的合成点击，避免误切筛选 */
function onGroupCapsuleClick(g: string) {
  if (groupLongPressFired) {
    groupLongPressFired = false
    return
  }
  activeGroup.value = g
}

function onGroupTouchEnd() {
  window.clearTimeout(groupLongPressTimer)
  groupLongPressTimer = undefined
}

function onGroupTouchMove() {
  window.clearTimeout(groupLongPressTimer)
  groupLongPressTimer = undefined
}

/** 分组 token 替换（newName=null 表示删除该分组） */
function replaceGroupToken(src: BookSource, oldName: string, newName: string | null): BookSource {
  const tokens = new Set(splitGroups(src.bookSourceGroup))
  tokens.delete(oldName)
  if (newName) tokens.add(newName)
  return { ...src, bookSourceGroup: Array.from(tokens).join(' ') || null }
}

/**
 * 批量改分组：逐源 saveBookSource 优先；接口未实现（404）时降级 saveBookSources 一次性批量保存。
 * 返回成功更新的书源数。
 */
async function applyGroupChange(oldName: string, newName: string | null): Promise<number> {
  const targets = sources.value.filter((s) =>
    splitGroups(s.bookSourceGroup).includes(oldName),
  )
  if (targets.length === 0) return 0
  const updated = targets.map((s) => replaceGroupToken(s, oldName, newName))
  let ok = 0
  let fellBack = false
  for (let i = 0; i < updated.length; i++) {
    try {
      await saveBookSource(updated[i])
      ok++
    } catch (err) {
      if (!fellBack && isNotImplemented(err)) {
        // 单个保存接口未就绪：整批降级 saveBookSources
        fellBack = true
        try {
          const res = await saveBookSources(updated)
          return res.data?.count ?? updated.length
        } catch {
          return ok // 降级也失败：已成功的部分返回
        }
      }
      // 单源失败（非 404）：跳过继续
    }
  }
  return ok
}

/** 重命名分组（输入新名；空名取消） */
async function renameGroup(oldName: string) {
  closeGroupMenu()
  if (groupMenuBusy.value) return
  try {
    const { value } = await ElMessageBox.prompt(
      `将分组「${oldName}」重命名为：`,
      '重命名分组',
      {
        confirmButtonText: '保存',
        cancelButtonText: '取消',
        inputValue: oldName,
        inputPattern: /\S+/,
        inputErrorMessage: '名称不能为空',
      },
    )
    const name = String(value ?? '').trim()
    if (!name || name === oldName) return
    groupMenuBusy.value = true
    try {
      const n = await applyGroupChange(oldName, name)
      await load()
      if (activeGroup.value === oldName) activeGroup.value = name
      ElMessage.success(`已重命名分组「${oldName}」→「${name}」（更新 ${n} 个书源）`)
    } catch {
      // 错误提示已由拦截器处理
    } finally {
      groupMenuBusy.value = false
    }
  } catch {
    // 用户取消
  }
}

/** 删除分组（从所有书源的 bookSourceGroup 中移除该 token） */
async function deleteGroupByName(name: string) {
  closeGroupMenu()
  if (groupMenuBusy.value) return
  try {
    await ElMessageBox.confirm(
      `确定删除分组「${name}」吗？组内书源将从该分组移除（书源本身保留）。`,
      '删除分组',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  groupMenuBusy.value = true
  try {
    const n = await applyGroupChange(name, null)
    await load()
    if (activeGroup.value === name) activeGroup.value = '全部'
    ElMessage.success(`已删除分组「${name}」（更新 ${n} 个书源）`)
  } catch {
    // 错误提示已由拦截器处理
  } finally {
    groupMenuBusy.value = false
  }
}

/* ================= 搜索过滤 ================= */
const filterKey = ref('')
const filtered = computed(() => {
  const kw = filterKey.value.trim().toLowerCase()
  return sources.value.filter((s) => {
    if (activeGroup.value !== '全部') {
      const gs = splitGroups(s.bookSourceGroup)
      if (!gs.includes(activeGroup.value)) return false
    }
    if (!kw) return true
    return (
      s.bookSourceName.toLowerCase().includes(kw) ||
      s.bookSourceUrl.toLowerCase().includes(kw) ||
      (s.bookSourceGroup ?? '').toLowerCase().includes(kw)
    )
  })
})

const enabledCount = computed(() => sources.value.filter((s) => s.enabled).length)

/* ================= 失效检测（GET /reader3/getInvalidBookSources） ================= */

const invalidChecking = ref(false)
const invalidSources = ref<Set<string>>(new Set())
const invalidMsg = ref('')
const invalidMsgError = ref(false)

/** 归一化后端返回：string[] 或含 bookSourceUrl 的对象数组 */
function normalizeInvalid(raw: unknown): string[] {
  if (!Array.isArray(raw)) return []
  const out: string[] = []
  for (const item of raw) {
    if (typeof item === 'string') out.push(item)
    else if (item && typeof item === 'object') {
      const u = (item as Record<string, unknown>).bookSourceUrl
      if (typeof u === 'string') out.push(u)
    }
  }
  return out
}

async function checkInvalid() {
  if (invalidChecking.value) return
  invalidChecking.value = true
  invalidMsg.value = ''
  invalidMsgError.value = false
  try {
    const res = await getInvalidBookSources()
    invalidSources.value = new Set(normalizeInvalid(res.data))
    const n = invalidSources.value.size
    invalidMsg.value = n === 0 ? '检测完成：未发现失效书源' : `检测完成：发现 ${n} 个失效书源（已红色标记）`
  } catch (err) {
    invalidMsg.value = isNotImplemented(err)
      ? '失效检测接口后端暂未提供（GET /reader3/getInvalidBookSources）'
      : `检测失败：${err instanceof Error ? err.message : '请稍后重试'}`
    invalidMsgError.value = true
  } finally {
    invalidChecking.value = false
  }
}

/* ================= 书源调试（GET /reader3/bookSourceDebugSSE：SSE 逐步日志） ================= */

const DEBUG_ACTIONS: { value: DebugAction; label: string; tip: string; needKey: boolean; needUrl: boolean }[] = [
  { value: 'search', label: '搜索', tip: '关键词搜索', needKey: true, needUrl: false },
  { value: 'toc', label: '目录', tip: '获取章节目录', needKey: false, needUrl: true },
  { value: 'content', label: '正文', tip: '获取章节正文', needKey: false, needUrl: true },
]

const debugOpen = ref(false)
const debugSource = ref<BookSource | null>(null)
const debugAction = ref<DebugAction>('search')
const debugInput = ref('')
const debugRunning = ref(false)
const debugLogs = ref<{ text: string; error: boolean }[]>([])
const debugMsg = ref('')
const debugMsgError = ref(false)
let debugHandle: { close: () => void } | null = null

function debugActionMeta(a: DebugAction) {
  return DEBUG_ACTIONS.find((x) => x.value === a) ?? DEBUG_ACTIONS[0]
}

const debugPlaceholder = computed(() => {
  if (debugAction.value === 'search') return '搜索关键词，如：斗破苍穹'
  if (debugAction.value === 'toc') return '书籍 URL（目录页地址，可留空）'
  return '章节 URL'
})

const debugTip = computed(() => {
  const meta = debugActionMeta(debugAction.value)
  return `动作：${meta.label}（${meta.tip}）· 步骤经 SSE 实时输出`
})

const debugCanRun = computed(() => {
  const meta = debugActionMeta(debugAction.value)
  return !meta.needKey || debugInput.value.trim().length > 0
})

function openDebug(s: BookSource) {
  debugSource.value = s
  debugAction.value = 'search'
  debugInput.value = ''
  debugLogs.value = []
  debugMsg.value = ''
  debugMsgError.value = false
  debugOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeDebug() {
  if (debugRunning.value) return
  debugHandle?.close()
  debugHandle = null
  debugOpen.value = false
  document.body.style.overflow = ''
}

function pushLog(text: string, error = false) {
  debugLogs.value.push({ text, error })
}

/** 运行调试：建立 SSE 连接，逐步追加日志；失败红色标记 */
async function runDebug() {
  const s = debugSource.value
  if (!s || debugRunning.value) return
  const meta = debugActionMeta(debugAction.value)
  const input = debugInput.value.trim()
  if (meta.needKey && !input) {
    debugMsg.value = '请输入搜索关键词'
    debugMsgError.value = true
    return
  }
  debugRunning.value = true
  debugMsg.value = ''
  debugMsgError.value = false
  debugLogs.value = []
  pushLog(`开始调试「${s.bookSourceName}」· ${meta.label}`)
  try {
    const handle = await bookSourceDebugSSE(
      {
        bookSourceUrl: s.bookSourceUrl,
        action: debugAction.value,
        key: meta.needKey ? input : undefined,
        chapterUrl: meta.needUrl && input ? input : undefined,
      },
      {
        onStep: (message) => pushLog(message),
        onResult: (data) => {
          let summary: string
          try {
            summary = typeof data === 'string' ? data : JSON.stringify(data, null, 2)
          } catch {
            summary = String(data)
          }
          pushLog(`结果：${summary}`)
          debugMsg.value = '调试完成'
          debugMsgError.value = false
        },
        onEnd: () => {
          debugRunning.value = false
          debugMsg.value = debugMsg.value || '调试完成'
        },
        onStreamError: (msg) => {
          debugRunning.value = false
          debugMsg.value = `调试失败：${msg}`
          debugMsgError.value = true
          pushLog(`错误：${msg}`, true)
        },
      },
    )
    debugHandle = handle
  } catch {
    debugRunning.value = false
    debugMsg.value = '调试接口后端暂未提供（GET /reader3/bookSourceDebugSSE）'
    debugMsgError.value = true
    pushLog('连接失败：后端未就绪或网络异常', true)
  }
}

function stopDebug() {
  debugHandle?.close()
  debugHandle = null
  debugRunning.value = false
  debugMsg.value = '已停止调试'
  debugMsgError.value = false
}

/* 切换动作时清空输入与日志 */
watch(debugAction, () => {
  if (debugRunning.value) return
  debugInput.value = ''
  debugLogs.value = []
  debugMsg.value = ''
  debugMsgError.value = false
})

/* ================= 默认书源标记（探测 getBookSources data[].isDefault；有则显示星标 + 点击调 POST setAsDefaultBookSources） ================= */

/** 后端是否返回 isDefault 字段（无则隐藏默认标记 UI） */
const defaultField = ref(false)
const defaultBusy = ref<Set<string>>(new Set())

function isDefaultSource(s: BookSource): boolean {
  return (s as { isDefault?: unknown }).isDefault === true
}

/** 点击星标：设本源为默认书源（乐观更新，失败回滚）；后端未实现（404）时静默隐藏能力 */
async function setDefault(s: BookSource) {
  if (defaultBusy.value.has(s.bookSourceUrl)) return
  defaultBusy.value.add(s.bookSourceUrl)
  const before = sources.value.map((x) => ({ url: x.bookSourceUrl, def: isDefaultSource(x) }))
  sources.value.forEach((x) => {
    ;(x as { isDefault?: boolean }).isDefault = x.bookSourceUrl === s.bookSourceUrl
  })
  try {
    await setAsDefaultBookSources([s.bookSourceUrl])
    ElMessage.success(`已将「${s.bookSourceName}」设为默认书源`)
  } catch (err) {
    // 回滚
    sources.value.forEach((x) => {
      const b = before.find((y) => y.url === x.bookSourceUrl)
      ;(x as { isDefault?: boolean }).isDefault = b?.def ?? false
    })
    if (!isNotImplemented(err)) {
      ElMessage.error(err instanceof Error ? err.message : '设置默认书源失败')
    }
  } finally {
    defaultBusy.value.delete(s.bookSourceUrl)
  }
}

/* ================= 启用开关 ================= */
const toggling = ref<Set<string>>(new Set())

async function toggleSource(s: BookSource) {
  if (toggling.value.has(s.bookSourceUrl)) return
  toggling.value.add(s.bookSourceUrl)
  const prev = s.enabled
  s.enabled = !prev // 乐观更新
  try {
    await saveBookSource({ ...s, enabled: !prev })
  } catch {
    s.enabled = prev // 失败回滚（错误提示由拦截器处理）
  } finally {
    toggling.value.delete(s.bookSourceUrl)
  }
}

/* ================= GAP 27：多选模式（勾选行 → 底部批量启用/禁用/删除） ================= */
const manageMode = ref(false)
const selectedSources = ref<Set<string>>(new Set())
const batchBusy = ref(false)

/** 勾选的书源对象（按列表顺序） */
const selectedList = computed(() =>
  sources.value.filter((s) => selectedSources.value.has(s.bookSourceUrl)),
)
const selectedCount = computed(() => selectedSources.value.size)
/** 当前筛选结果是否全部勾选 */
const allFilteredSelected = computed(
  () => filtered.value.length > 0 && filtered.value.every((s) => selectedSources.value.has(s.bookSourceUrl)),
)

function toggleManage() {
  manageMode.value = !manageMode.value
  if (!manageMode.value) selectedSources.value.clear()
}

function toggleSelect(s: BookSource) {
  const set = new Set(selectedSources.value)
  if (set.has(s.bookSourceUrl)) set.delete(s.bookSourceUrl)
  else set.add(s.bookSourceUrl)
  selectedSources.value = set
}

function toggleSelectAll() {
  const set = new Set(selectedSources.value)
  if (allFilteredSelected.value) {
    for (const s of filtered.value) set.delete(s.bookSourceUrl)
  } else {
    for (const s of filtered.value) set.add(s.bookSourceUrl)
  }
  selectedSources.value = set
}

/* ================= 排序模式（书源行拖拽排序——手柄列 HTML5 drag；保存 = 按新顺序重排 weight 递减） ================= */
const sortMode = ref(false)
const sortDirty = ref(false)
const sortSaving = ref(false)
const dragSourceUrl = ref<string | null>(null)
const dragOverUrl = ref<string | null>(null)

/** 拖拽开始（排序模式下手柄 draggable；事件冒泡到行） */
function onSourceDragStart(s: BookSource, e: DragEvent) {
  dragSourceUrl.value = s.bookSourceUrl
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', s.bookSourceUrl)
  }
}

/** 经过其他行：阻止默认（允许放下）并高亮目标行 */
function onSourceDragOver(s: BookSource, e: DragEvent) {
  if (dragSourceUrl.value === null || dragSourceUrl.value === s.bookSourceUrl) return
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  dragOverUrl.value = s.bookSourceUrl
}

/** 放下：把拖拽书源移到目标书源位置（本地重排 sources；保存栏出现） */
function onSourceDrop(s: BookSource, e: DragEvent) {
  e.preventDefault()
  const from = dragSourceUrl.value
  dragSourceUrl.value = null
  dragOverUrl.value = null
  if (from === null || from === s.bookSourceUrl) return
  const list = sources.value.slice()
  const fromIdx = list.findIndex((x) => x.bookSourceUrl === from)
  const toIdx = list.findIndex((x) => x.bookSourceUrl === s.bookSourceUrl)
  if (fromIdx < 0 || toIdx < 0) return
  const [moved] = list.splice(fromIdx, 1)
  list.splice(toIdx, 0, moved)
  sources.value = list
  sortDirty.value = true
}

function onSourceDragEnd() {
  dragSourceUrl.value = null
  dragOverUrl.value = null
}

/** 保存排序：按新顺序分配 weight（递减——越靠前越大）→ 批量 saveBookSources 优先；批量接口未实现（404）时逐源 saveBookSource */
async function saveSourceOrder() {
  if (sortSaving.value) return
  const ordered = sources.value.map((s, i) => ({ ...s, weight: sources.value.length - i }))
  sortSaving.value = true
  try {
    const res = await saveBookSources(ordered)
    sortDirty.value = false
    ElMessage.success(`书源排序已保存（${res.data?.count ?? ordered.length} 个书源）`)
  } catch (err) {
    if (isNotImplemented(err)) {
      // 批量保存接口未就绪：降级逐源 saveBookSource 更新 weight
      let ok = 0
      for (const s of ordered) {
        try {
          await saveBookSource(s)
          ok++
        } catch {
          // 单源失败跳过
        }
      }
      if (ok > 0) {
        sortDirty.value = false
        ElMessage.success(`书源排序已保存（${ok}/${ordered.length} 个书源）`)
      } else {
        ElMessage.error('保存排序失败，请重试')
      }
    }
    // 其余错误（网络/后端失败）已由请求拦截器提示
  } finally {
    sortSaving.value = false
  }
}

/** 切换排序模式：退出时若有未保存排序自动保存（防丢失） */
function toggleSortMode() {
  sortMode.value = !sortMode.value
  dragSourceUrl.value = null
  dragOverUrl.value = null
  if (!sortMode.value && sortDirty.value) void saveSourceOrder()
}

/**
 * 批量启用/禁用：逐源 saveBookSource 循环；单源接口未实现（404）时降级 saveBookSources 整批。
 * 返回成功更新的书源数。
 */
async function batchSetEnabled(enabled: boolean): Promise<number> {
  const targets = selectedList.value.filter((s) => s.enabled !== enabled)
  if (targets.length === 0) return 0
  const updated = targets.map((s) => ({ ...s, enabled }))
  let ok = 0
  let fellBack = false
  for (let i = 0; i < updated.length; i++) {
    try {
      await saveBookSource(updated[i])
      ok++
    } catch (err) {
      if (!fellBack && isNotImplemented(err)) {
        // 单个保存接口未就绪：整批降级 saveBookSources
        fellBack = true
        try {
          const res = await saveBookSources(updated)
          return res.data?.count ?? updated.length
        } catch {
          return ok // 降级也失败：已成功的部分返回
        }
      }
      // 单源失败（非 404）：跳过继续
    }
  }
  return ok
}

async function batchEnable() {
  await runBatchEnabled(true)
}

async function batchDisable() {
  await runBatchEnabled(false)
}

async function runBatchEnabled(enabled: boolean) {
  if (batchBusy.value || selectedCount.value === 0) return
  batchBusy.value = true
  try {
    const n = await batchSetEnabled(enabled)
    if (n > 0) {
      ElMessage.success(`已${enabled ? '启用' : '禁用'} ${n} 个书源`)
      await load()
    } else {
      ElMessage.info(`所选书源已全部处于${enabled ? '启用' : '停用'}状态`)
    }
  } finally {
    batchBusy.value = false
  }
}

/** 批量删除：优先 POST /reader3/deleteBookSources；接口未实现（404）时降级逐源 deleteBookSource */
async function batchDelete() {
  if (batchBusy.value || selectedCount.value === 0) return
  try {
    await ElMessageBox.confirm(
      `确定删除选中的 ${selectedCount.value} 个书源吗？此操作不可恢复。`,
      '批量删除书源',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  batchBusy.value = true
  const urls = selectedList.value.map((s) => s.bookSourceUrl)
  try {
    const res = await deleteBookSources(urls, { silent: true })
    const deleted = res.data?.deleted ?? urls.length
    ElMessage.success(`已删除 ${deleted} 个书源`)
    selectedSources.value.clear()
    await load()
  } catch (err) {
    if (isNotImplemented(err)) {
      // 批量接口未实现：降级逐源删除
      let ok = 0
      for (const u of urls) {
        try {
          await deleteBookSource(u)
          ok++
        } catch {
          // 单源失败跳过
        }
      }
      ElMessage.success(`已删除 ${ok} 个书源`)
      if (ok > 0) {
        selectedSources.value.clear()
        await load()
      }
    } else {
      ElMessage.error(err instanceof Error ? err.message : '批量删除失败')
    }
  } finally {
    batchBusy.value = false
  }
}

/* ================= 删除（极简确认弹窗） ================= */
const deleting = ref<BookSource | null>(null)
const deleteBusy = ref(false)

function askDelete(s: BookSource) {
  deleting.value = s
  document.body.style.overflow = 'hidden'
}

async function confirmDelete() {
  const s = deleting.value
  if (!s || deleteBusy.value) return
  deleteBusy.value = true
  try {
    await deleteBookSource(s.bookSourceUrl)
    await load()
    closeDelete()
  } catch {
    // 错误提示已由拦截器处理
  } finally {
    deleteBusy.value = false
  }
}

function closeDelete() {
  deleting.value = null
  document.body.style.overflow = ''
}

/* ================= 新增（极简弹窗表单） ================= */
const addOpen = ref(false)
const addBusy = ref(false)
const addForm = ref({ bookSourceUrl: '', bookSourceName: '', bookSourceGroup: '' })

function openAdd() {
  addForm.value = { bookSourceUrl: '', bookSourceName: '', bookSourceGroup: '' }
  addOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeAdd() {
  if (addBusy.value) return
  addOpen.value = false
  document.body.style.overflow = ''
}

/** 后端要求完整 BookSource：补默认值；base 传入时保留其全部字段（编辑场景——登录/header/规则等原样保留） */
function buildSource(
  form: { bookSourceUrl: string; bookSourceName: string; bookSourceGroup: string },
  base?: BookSource,
): BookSource {
  return {
    ...(base ?? {}),
    bookSourceUrl: form.bookSourceUrl.trim(),
    bookSourceName: form.bookSourceName.trim() || form.bookSourceUrl.trim(),
    bookSourceGroup: form.bookSourceGroup.trim() || null,
    bookSourceType: base?.bookSourceType ?? 0,
    customOrder: base?.customOrder ?? 0,
    enabled: base?.enabled ?? true,
    enabledExplore: base?.enabledExplore ?? false,
    lastUpdateTime: base?.lastUpdateTime ?? 0,
    respondTime: base?.respondTime ?? 0,
    weight: base?.weight ?? 0,
  }
}

async function confirmAdd() {
  if (addBusy.value) return
  const url = addForm.value.bookSourceUrl.trim()
  if (!url) return
  addBusy.value = true
  try {
    await saveBookSource(buildSource(addForm.value))
    closeAdd()
    await load()
  } catch {
    // 错误提示已由拦截器处理
  } finally {
    addBusy.value = false
  }
}

/* ================= 编辑（基本信息 + 规则字段 textarea JSON；saveBookSource 整源覆盖，其余字段保留） ================= */

interface RuleField {
  key: string
  label: string
  tip: string
  kind: 'json' | 'text'
}

/** 规则字段清单：对齐后端 BookSource 模型（legacy + legado 两套命名；json = 嵌套对象按 JSON 文本编辑，每字段一个 textarea 单条规则） */
const RULE_FIELDS: RuleField[] = [
  { key: 'searchUrl', label: 'searchUrl', tip: '搜索 URL 模板（{{key}} 为关键词占位符）', kind: 'text' },
  { key: 'ruleSearch', label: 'ruleSearch', tip: '搜索规则 JSON：bookList / name / author / kind / coverUrl / intro / bookUrl / wordCount / latestChapterTitle', kind: 'json' },
  { key: 'searchRule', label: 'searchRule', tip: '搜索规则 JSON（legado 命名别名，与 ruleSearch 二选一）', kind: 'json' },
  { key: 'ruleBookInfo', label: 'ruleBookInfo', tip: '详情规则 JSON：init / name / author / kind / coverUrl / intro / tocUrl', kind: 'json' },
  { key: 'bookInfoRule', label: 'bookInfoRule', tip: '详情规则 JSON（legado 命名别名，与 ruleBookInfo 二选一）', kind: 'json' },
  { key: 'ruleToc', label: 'ruleToc', tip: '目录规则 JSON：chapterList / chapterName / chapterUrl / nextTocUrl / isVolume', kind: 'json' },
  { key: 'tocRule', label: 'tocRule', tip: '目录规则 JSON（legado 命名别名，与 ruleToc 二选一）', kind: 'json' },
  { key: 'ruleContent', label: 'ruleContent', tip: '正文规则 JSON：content / nextContentUrl / sourceRegex（contentType 等字段可一并写入）', kind: 'json' },
  { key: 'contentRule', label: 'contentRule', tip: '正文规则 JSON（legado 命名别名，与 ruleContent 二选一）', kind: 'json' },
  { key: 'ruleExplore', label: 'ruleExplore', tip: '探索规则 JSON：bookList / name / author / kind / coverUrl / intro / bookUrl / wordCount / latestChapterTitle', kind: 'json' },
  { key: 'exploreRule', label: 'exploreRule', tip: '探索规则 JSON（legado 命名别名，与 ruleExplore 二选一）', kind: 'json' },
  { key: 'exploreUrl', label: 'exploreUrl', tip: '探索 URL 模板（每行一个分类地址）', kind: 'text' },
]

const editOpen = ref(false)
const editBusy = ref(false)
const editSource = ref<BookSource | null>(null)
const editForm = ref({ bookSourceUrl: '', bookSourceName: '', bookSourceGroup: '' })
const editRules = ref<Record<string, string>>({})
const editWeight = ref(0)
/* GAP 107：header（JSON）/ loginUrl / cookie 编辑 */
const editHeader = ref('')
const editLoginUrl = ref('')
const editCookie = ref('')
const editMsg = ref('')
const editMsgError = ref(false)

/** 书源编辑器常用符号快捷插入（legacy AppConst 书源编辑键盘符号栏对应） */
const RULE_SYMBOLS = [
  '{{',
  '}}',
  '{{key}}',
  '@js:',
  '@css:',
  '@json:',
  '@regex:',
  '@xpath:',
  '@get:',
  '@put:',
  '||',
  '##',
  '::',
  '\n',
]

/** 向当前聚焦的规则 textarea 光标处插入符号（Vue v-model 经 input 事件同步） */
function insertRuleSymbol(tok: string) {
  const el = document.activeElement
  if (!(el instanceof HTMLTextAreaElement)) {
    ElMessage.info('请先点击规则输入框，再插入符号')
    return
  }
  const start = el.selectionStart ?? el.value.length
  const end = el.selectionEnd ?? start
  el.value = el.value.slice(0, start) + tok + el.value.slice(end)
  el.selectionStart = el.selectionEnd = start + tok.length
  el.focus()
  el.dispatchEvent(new Event('input', { bubbles: true }))
}

/** header 字段：字符串存的 JSON（legado 契约）——可解析时 pretty-print 便于编辑 */
function headerToText(s: BookSource | null): string {
  const v = s?.header
  if (v === undefined || v === null || v === '') return ''
  try {
    return JSON.stringify(JSON.parse(v), null, 2)
  } catch {
    return String(v)
  }
}

/** 书源字段 → textarea 文本（json 字段 pretty-print，text 字段原样） */
function ruleToText(s: BookSource | null, f: RuleField): string {
  if (!s) return ''
  const v = s[f.key]
  if (v === undefined || v === null) return ''
  if (f.kind === 'json') {
    try {
      return JSON.stringify(v, null, 2)
    } catch {
      return String(v)
    }
  }
  return String(v)
}

function openEdit(s: BookSource) {
  editSource.value = s
  editForm.value = {
    bookSourceUrl: s.bookSourceUrl,
    bookSourceName: s.bookSourceName,
    bookSourceGroup: s.bookSourceGroup ?? '',
  }
  editRules.value = {}
  for (const f of RULE_FIELDS) editRules.value[f.key] = ruleToText(s, f)
  editWeight.value = typeof s.weight === 'number' && Number.isFinite(s.weight) ? s.weight : 0
  editHeader.value = headerToText(s)
  editLoginUrl.value = typeof s.loginUrl === 'string' ? s.loginUrl : ''
  editCookie.value = '' // cookie 不入书源 JSON（后端模型无此字段）——保存时非空走 setBookSourceCookie
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

/** 校验 + 保存：JSON 字段逐个 parse（失败定位到具体字段），合并进原书源后 saveBookSource 整源覆盖，刷新列表 */
/* ================= 书源登录（POST /reader3/loginBookSource 等；登录态 localStorage 持久 reader_src_login_{url}） ================= */

const LOGIN_KEY_PREFIX = 'reader_src_login_'
const LOGIN_KEY = (url: string) => `${LOGIN_KEY_PREFIX}${url}`

/** 已登录书源 URL 集合（localStorage 缓存 reader_src_login_{url}，刷新页面后仍显示「已登录」） */
const loggedUrls = ref<Set<string>>(new Set())

function syncLoggedUrls() {
  const set = new Set<string>()
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i)
    if (k && k.startsWith(LOGIN_KEY_PREFIX) && localStorage.getItem(k) === '1') {
      set.add(k.slice(LOGIN_KEY_PREFIX.length))
    }
  }
  loggedUrls.value = set
}

function markLoggedIn(url: string) {
  localStorage.setItem(LOGIN_KEY(url), '1')
  syncLoggedUrls()
}

function markLoggedOut(url: string) {
  localStorage.removeItem(LOGIN_KEY(url))
  syncLoggedUrls()
}

const loginOpen = ref(false)
const loginSource = ref<BookSource | null>(null)
const loginBusy = ref(false)
const loginForm = ref({ username: '', password: '' })
/** 登录态：unknown=登录态未知（探测中/未探测） logged=已登录 not=登录失败 */
const loginState = ref<'unknown' | 'logged' | 'not'>('unknown')
const loginProbe = ref('') // 状态区探测提示（getCaptcha 结果）
const loginMsg = ref('') // 操作结果提示
const loginMsgError = ref(false)
const cookieSummary = ref('') // 登录成功后的 cookie 摘要（前 20 字符）
const captcha = ref<{ captchaId: string; captchaUrl: string; message: string } | null>(null)
const captchaFrom = ref<'probe' | 'login'>('probe') // 验证码来源（登录返回的验证码不被探测覆盖）
const captchaText = ref('')
const showManual = ref(false) // 手动 Cookie 区（needManualCaptcha / 探测到点选验证码）
const manualCookie = ref('')
const probing = ref(false)

function openLogin(s: BookSource) {
  loginSource.value = s
  loginForm.value = { username: '', password: '' }
  loginState.value = 'unknown'
  loginProbe.value = ''
  loginMsg.value = ''
  loginMsgError.value = false
  cookieSummary.value = ''
  captcha.value = null
  captchaFrom.value = 'probe'
  captchaText.value = ''
  showManual.value = false
  manualCookie.value = ''
  loginOpen.value = true
  document.body.style.overflow = 'hidden'
  if (loggedUrls.value.has(s.bookSourceUrl)) {
    loginState.value = 'logged'
    loginProbe.value = '本地缓存：该书源已登录（Cookie 存于服务端）'
  } else {
    void probeCaptcha() // 登录态未知 → getCaptcha 探测
  }
}

function closeLogin() {
  if (loginBusy.value) return
  loginOpen.value = false
  document.body.style.overflow = ''
}

/** 探测验证码（POST /reader3/getCaptcha）：结果仅更新状态区与验证码区；失败不影响直接登录。force=true 时强制刷新验证码图 */
async function probeCaptcha(force = false) {
  const s = loginSource.value
  if (!s || probing.value) return
  probing.value = true
  if (!loginProbe.value) loginProbe.value = '探测中…'
  try {
    const res = await getCaptcha(s.bookSourceUrl, { silent: true })
    const d = res.data as CaptchaProbe | null | undefined
    const kind = d?.captchaType
    if (kind === 'image' && d?.captchaUrl && d?.captchaId) {
      if (force || captchaFrom.value !== 'login') {
        captcha.value = {
          captchaId: d.captchaId,
          captchaUrl: d.captchaUrl,
          message: d.message || '需要图片验证码',
        }
        captchaFrom.value = 'probe'
        captchaText.value = ''
      }
      loginProbe.value = '检测到图片验证码——填写用户名/密码后可提交'
    } else if (kind === 'slider') {
      loginProbe.value = d?.message || '检测到滑块验证码——点击「登录」由浏览器自动处理'
    } else if (kind === 'click') {
      showManual.value = true
      loginProbe.value = d?.message || '检测到点选类验证码——请在浏览器登录后粘贴 Cookie'
    } else {
      loginProbe.value = d?.message || '未检测到验证码，可直接登录'
    }
  } catch (err) {
    loginProbe.value =
      err instanceof Error && err.message
        ? `探测失败：${err.message}（可直接登录或粘贴 Cookie）`
        : '探测失败（可直接登录或粘贴 Cookie）'
  } finally {
    probing.value = false
  }
}

/** 登录结果统一处理（loginBookSource 的 success / submitCaptcha 的 isLogin 两套标记） */
function applyLoginResult(d: BookSourceLoginResult | null | undefined) {
  const url = loginSource.value?.bookSourceUrl
  if (!d || !url) return
  if (d.isLogin === true || d.success === true) {
    markLoggedIn(url)
    loginState.value = 'logged'
    cookieSummary.value = d.cookie ? d.cookie.slice(0, 20) : ''
    loginMsg.value = '登录成功'
    loginMsgError.value = false
    captcha.value = null
    captchaFrom.value = 'probe'
    captchaText.value = ''
    showManual.value = false
    loginProbe.value = ''
    return
  }
  if (d.needCaptcha) {
    captcha.value = {
      captchaId: d.captchaId ?? '',
      captchaUrl: d.captchaUrl ?? '',
      message: d.message || '需要图片验证码',
    }
    captchaFrom.value = 'login'
    captchaText.value = ''
    loginMsg.value = d.message || '需要图片验证码，请输入后提交'
    loginMsgError.value = false
    return
  }
  if (d.needManualCaptcha) {
    showManual.value = true
    manualCookie.value = ''
    loginMsg.value = d.message || '需手动验证码：请在浏览器登录该书源后，在下方粘贴 Cookie'
    loginMsgError.value = false
    return
  }
  // 登录失败（无验证码）
  loginState.value = 'not'
  loginMsg.value = d.message || '登录失败'
  loginMsgError.value = true
}

/** 「登录」按钮：POST /reader3/loginBookSource（username/password） */
async function doLogin() {
  const s = loginSource.value
  if (!s || loginBusy.value) return
  loginBusy.value = true
  loginMsg.value = ''
  try {
    const res = await loginBookSource({
      bookSource: s.bookSourceUrl,
      username: loginForm.value.username,
      password: loginForm.value.password,
    })
    applyLoginResult(res.data)
  } catch {
    // 硬错误（ReturnData::err / 网络）已由拦截器提示
  } finally {
    loginBusy.value = false
  }
}

/** 「提交验证码」：POST /reader3/submitCaptcha（带 username/password 覆盖会话值）→ isLogin 显示结果 */
async function doSubmitCaptcha() {
  const s = loginSource.value
  const c = captcha.value
  if (!s || !c || loginBusy.value) return
  const text = captchaText.value.trim()
  if (!c.captchaId || !text) return
  loginBusy.value = true
  loginMsg.value = ''
  try {
    const res = await submitCaptcha({
      bookSource: s.bookSourceUrl,
      captchaId: c.captchaId,
      captchaText: text,
      username: loginForm.value.username,
      password: loginForm.value.password,
    })
    applyLoginResult(res.data)
  } catch {
    // 拦截器已提示
  } finally {
    loginBusy.value = false
  }
}

/** 「保存 Cookie」：POST /reader3/setBookSourceCookie（手动 Cookie 落库） */
async function saveManualCookie() {
  const s = loginSource.value
  if (!s || loginBusy.value) return
  const cookie = manualCookie.value.trim()
  if (!cookie) return
  loginBusy.value = true
  loginMsg.value = ''
  try {
    const res = await setBookSourceCookie(s.bookSourceUrl, cookie)
    if (res.data?.success) {
      markLoggedIn(s.bookSourceUrl)
      loginState.value = 'logged'
      cookieSummary.value = cookie.slice(0, 20)
      showManual.value = false
      manualCookie.value = ''
      loginMsg.value = 'Cookie 已保存'
      loginMsgError.value = false
    }
  } catch {
    // 拦截器已提示
  } finally {
    loginBusy.value = false
  }
}

/** 「清除 Cookie」：POST /reader3/setBookSourceCookie（空 cookie = 清除） */
async function clearLoginCookie() {
  const s = loginSource.value
  if (!s || loginBusy.value) return
  loginBusy.value = true
  loginMsg.value = ''
  try {
    const res = await setBookSourceCookie(s.bookSourceUrl, '')
    if (res.data?.success) {
      markLoggedOut(s.bookSourceUrl)
      loginState.value = 'unknown'
      cookieSummary.value = ''
      captcha.value = null
      captchaFrom.value = 'probe'
      captchaText.value = ''
      showManual.value = false
      manualCookie.value = ''
      loginMsg.value = '已清除 Cookie（登录态失效）'
      loginMsgError.value = false
    }
  } catch {
    // 拦截器已提示
  } finally {
    loginBusy.value = false
  }
}

/* ================= 书源 Cookie 管理（GAP 196：已登录书源列表 + 摘要 + 清除；后端 getBookSourceCookie 读取登录态） ================= */

const cookieMgrOpen = ref(false)
const cookieMgrBusy = ref<Set<string>>(new Set())
/** 服务端登录态行（getBookSourceCookie：cookie/userAgent/loginHeader 摘要） */
const cookieRows = ref<CookieRow[]>([])
const cookieRowsMsg = ref('')

/** 已登录书源列表：服务端登录态优先，补充本地登录态标记（无服务端 cookie 行的书源） */
const loggedSources = computed(() => {
  const server = new Set(cookieRows.value.map((r) => r.sourceUrl))
  const merged = new Map<string, BookSource>()
  for (const r of cookieRows.value) {
    const s = sources.value.find((x) => x.bookSourceUrl === r.sourceUrl)
    if (s) merged.set(s.bookSourceUrl, s)
    else {
      merged.set(r.sourceUrl, {
        bookSourceUrl: r.sourceUrl,
        bookSourceName: r.sourceUrl,
        bookSourceType: 0,
        customOrder: 0,
        enabled: true,
        enabledExplore: false,
        respondTime: 0,
        weight: 0,
        lastUpdateTime: 0,
      } as BookSource)
    }
  }
  for (const s of sources.value) {
    if (loggedUrls.value.has(s.bookSourceUrl) && !server.has(s.bookSourceUrl)) {
      merged.set(s.bookSourceUrl, s)
    }
  }
  return Array.from(merged.values())
})

/** 服务端登录态摘要（cookie 前 30 字符；无则空） */
function cookiePreview(r: CookieRow): string {
  const c = r.cookie?.trim() || ''
  return c ? c.slice(0, 30) + (c.length > 30 ? '…' : '') : ''
}

/** 域名提取（cookie 作用域按源 URL host） */
function hostOf(url: string): string {
  try {
    return new URL(url).host
  } catch {
    return url
  }
}

function openCookieMgr() {
  syncLoggedUrls() // 打开时重扫 localStorage，避免其他标签页变更未同步
  cookieRows.value = []
  cookieRowsMsg.value = ''
  void getBookSourceCookie()
    .then((res) => {
      cookieRows.value = res.data ?? []
    })
    .catch(() => {
      cookieRowsMsg.value = '服务端登录态读取失败，仅显示本地标记'
    })
  cookieMgrOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeCookieMgr() {
  cookieMgrOpen.value = false
  document.body.style.overflow = ''
}

/** 清除 Cookie：POST /reader3/setBookSourceCookie（空 cookie = 清除）→ 同步移除本地登录态 */
async function clearSourceCookie(s: BookSource) {
  if (cookieMgrBusy.value.has(s.bookSourceUrl)) return
  cookieMgrBusy.value = new Set(cookieMgrBusy.value).add(s.bookSourceUrl)
  try {
    const res = await setBookSourceCookie(s.bookSourceUrl, '')
    if (res.data?.success) {
      markLoggedOut(s.bookSourceUrl)
      cookieRows.value = cookieRows.value.filter((r) => r.sourceUrl !== s.bookSourceUrl)
      ElMessage.success(`已清除「${s.bookSourceName}」的 Cookie`)
    }
  } catch {
    // 拦截器已提示
  } finally {
    const next = new Set(cookieMgrBusy.value)
    next.delete(s.bookSourceUrl)
    cookieMgrBusy.value = next
  }
}

async function confirmEdit() {
  if (editBusy.value) return
  const base = editSource.value
  if (!base) return
  const url = editForm.value.bookSourceUrl.trim()
  if (!url) {
    editMsg.value = 'URL 不能为空'
    editMsgError.value = true
    return
  }
  for (const f of RULE_FIELDS) {
    const v = editRules.value[f.key]?.trim() ?? ''
    if (f.kind !== 'json' || !v) continue
    try {
      JSON.parse(v)
    } catch (err) {
      editMsg.value = `「${f.label}」不是有效 JSON：${err instanceof Error ? err.message : '语法错误'}`
      editMsgError.value = true
      return
    }
  }
  // GAP 107：header 必须为合法 JSON
  let headerJson: unknown = null
  if (editHeader.value.trim()) {
    try {
      headerJson = JSON.parse(editHeader.value)
    } catch (err) {
      editMsg.value = `「header」不是有效 JSON：${err instanceof Error ? err.message : '语法错误'}`
      editMsgError.value = true
      return
    }
  }
  editBusy.value = true
  editMsg.value = ''
  try {
    const merged = buildSource(editForm.value, base)
    // 权重（GAP 29：数字输入 → BookSource.weight 字段提交）
    merged.weight = Number.isFinite(editWeight.value) ? Math.max(0, Math.round(editWeight.value)) : 0
    for (const f of RULE_FIELDS) {
      const v = editRules.value[f.key]?.trim() ?? ''
      if (v === '') {
        delete merged[f.key] // 留空 = 清除该规则
      } else if (f.kind === 'json') {
        merged[f.key] = JSON.parse(v) as unknown
      } else {
        merged[f.key] = v
      }
    }
    // GAP 107：header（JSON 字符串存储）/ loginUrl 合并；留空 = 清除
    if (headerJson === null) {
      delete merged.header
    } else {
      merged.header = JSON.stringify(headerJson)
    }
    if (editLoginUrl.value.trim()) {
      merged.loginUrl = editLoginUrl.value.trim()
    } else {
      delete merged.loginUrl
    }
    await saveBookSource(merged)
    // GAP 107：cookie 非空 → 单独走 setBookSourceCookie（后端书源模型无 cookie 字段，cookie 存服务端 cookie 表）
    if (editCookie.value.trim()) {
      try {
        await setBookSourceCookie(merged.bookSourceUrl, editCookie.value.trim())
      } catch {
        ElMessage.warning('书源已保存，但 Cookie 写入失败')
      }
    }
    editBusy.value = false // 先复位再关闭（closeEdit 忙碌中不允许关闭）
    closeEdit()
    ElMessage.success(`已保存「${merged.bookSourceName}」`)
    await load()
  } catch {
    // 错误提示已由拦截器处理
  } finally {
    editBusy.value = false
  }
}

/* ================= 导出（blob 下载 bookSource.json） ================= */
const exporting = ref(false)

/**
 * 导出：勾选多选时逐源构造 JSON 下载（后端 exportBookSources 不支持筛选参数）；
 * 未勾选 → 全量走 GET /reader3/exportBookSources。
 */
async function doExport() {
  if (exporting.value) return
  exporting.value = true
  try {
    const picked = selectedList.value
    if (manageMode.value && picked.length > 0) {
      // 勾选导出：前端按勾选书源构造 bookSource.json（与全量导出同构）
      const blob = new Blob([JSON.stringify(picked, null, 2)], {
        type: 'application/json;charset=utf-8',
      })
      await downloadBlob(blob, 'bookSource.json')
      ElMessage.success(`已导出 ${picked.length} 个书源`)
    } else {
      const blob = await exportBookSources()
      await downloadBlob(blob, 'bookSource.json')
    }
  } catch {
    // 请求层已提示
  } finally {
    exporting.value = false
  }
}

/** 导出当前分组：按组过滤书源构造 bookSource.json（blob 下载，与勾选导出同构） */
async function doExportGroup() {
  const g = activeGroup.value
  if (exporting.value || g === '全部') return
  exporting.value = true
  try {
    const list = sources.value.filter((s) =>
      (s.bookSourceGroup ?? '').split(/\s+/).includes(g),
    )
    if (list.length === 0) {
      ElMessage.warning(`分组「${g}」暂无书源`)
      return
    }
    const blob = new Blob([JSON.stringify(list, null, 2)], {
      type: 'application/json;charset=utf-8',
    })
    await downloadBlob(blob, 'bookSource.json')
    ElMessage.success(`已导出「${g}」分组 ${list.length} 个书源`)
  } catch {
    // 请求层已提示
  } finally {
    exporting.value = false
  }
}

/* ================= 本地文件导入（input file → 解析 JSON → saveBookSources） ================= */
const localFileInput = ref<HTMLInputElement | null>(null)
const localImportBusy = ref(false)

function openLocalImport() {
  localFileInput.value?.click()
}

async function onLocalFilePick(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = '' // 允许再次选择同一文件
  if (!file || localImportBusy.value) return
  localImportBusy.value = true
  try {
    const raw: unknown = JSON.parse(await file.text())
    const list = normalizeSources(raw)
    if (list.length === 0) {
      ElMessage.warning('未识别到书源（需为书源数组或含 bookSourceList 的对象）')
      return
    }
    const existing = new Set(sources.value.map((s) => s.bookSourceUrl))
    openPreview(list, existing, '导入本地书源', 'local')
  } catch (err) {
    if (err instanceof SyntaxError) {
      ElMessage.error('文件不是有效的 JSON')
    }
    // 其余错误（网络/后端失败）已由请求拦截器提示
  } finally {
    localImportBusy.value = false
  }
}

/* ================= 导入预览（选择 / 全选 / 反选 / 选择新增 / 排序） ================= */
const previewOpen = ref(false)
const previewTitle = ref('')
const previewBusy = ref(false)
const previewMode = ref<'local' | 'remote' | 'sub'>('local')
const previewList = ref<BookSource[]>([])
const previewSelected = ref<boolean[]>([])
const previewExisting = ref<Set<string>>(new Set())
const previewRemoteUrl = ref('')

function openPreview(
  list: BookSource[],
  existing: Set<string>,
  title: string,
  mode: 'local' | 'remote' | 'sub',
) {
  previewList.value = list
  previewSelected.value = list.map(() => true)
  previewExisting.value = existing
  previewTitle.value = title
  previewMode.value = mode
  previewOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closePreview() {
  if (previewBusy.value) return
  previewOpen.value = false
  document.body.style.overflow = ''
}

function previewSelectAll() {
  previewSelected.value = previewList.value.map(() => true)
}

function previewSelectNone() {
  previewSelected.value = previewList.value.map(() => false)
}

function previewInvert() {
  previewSelected.value = previewSelected.value.map((v) => !v)
}

function previewSelectNew() {
  previewSelected.value = previewList.value.map(
    (s) => !previewExisting.value.has(s.bookSourceUrl),
  )
}

function previewMove(i: number, dir: -1 | 1) {
  const j = i + dir
  if (j < 0 || j >= previewList.value.length) return
  const list = previewList.value
  const sel = previewSelected.value
  ;[list[i], list[j]] = [list[j], list[i]]
  ;[sel[i], sel[j]] = [sel[j], sel[i]]
}

function previewCount() {
  return previewSelected.value.filter(Boolean).length
}

async function confirmPreview() {
  if (previewBusy.value) return
  const selected = previewList.value
    .filter((_, i) => previewSelected.value[i])
    .map((s, i) => ({ ...s, customOrder: i }))
  if (selected.length === 0) {
    ElMessage.warning('请至少选择一个书源')
    return
  }
  previewBusy.value = true
  try {
    if (previewMode.value === 'sub') {
      const urls = selected.map((s) => s.bookSourceUrl)
      const res = await saveSourceSub(previewRemoteUrl.value, '', urls)
      if (res.isSuccess) {
        const name = (res.data?.name as string) || previewRemoteUrl.value
        const existing = subs.value.find((x) => x.url === previewRemoteUrl.value)
        if (existing) existing.name = name
        else subs.value.push({ url: previewRemoteUrl.value, name })
        setSubMsg(`订阅成功：已导入 ${res.data?.count ?? selected.length} 个书源`)
      } else {
        // 后端不可达降级：本地导入所选书源 + 订阅记录（api 已写入 localStorage）
        const saveRes = await saveBookSources(selected)
        const existing = subs.value.find((x) => x.url === previewRemoteUrl.value)
        if (existing) existing.name = previewRemoteUrl.value
        else subs.value.push({ url: previewRemoteUrl.value, name: previewRemoteUrl.value })
        setSubMsg(
          `订阅成功（本地通道）：已导入 ${saveRes.data?.count ?? selected.length} 个书源`,
        )
      }
      previewRemoteUrl.value = ''
      subUrl.value = ''
    } else {
      const res = await saveBookSources(selected)
      ElMessage.success(`成功导入 ${res.data?.count ?? selected.length} 个书源`)
    }
    previewOpen.value = false
    document.body.style.overflow = ''
    await load()
  } catch (err) {
    if (previewMode.value === 'sub') {
      setSubMsg(
        `导入失败：${err instanceof Error && err.message ? err.message : '未知错误'}`,
        true,
      )
    }
    // 其余错误由请求拦截器提示
  } finally {
    previewBusy.value = false
  }
}

/* ================= 远程导入（fetch JSON → 批量 saveBookSources） ================= */
const importOpen = ref(false)
const importBusy = ref(false)
const importUrl = ref('')
const importTip = ref('')

function openImport() {
  importUrl.value = ''
  importTip.value = ''
  importOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeImport() {
  if (importBusy.value) return
  importOpen.value = false
  document.body.style.overflow = ''
}

/** 兼容三种格式：数组 / {bookSourceList:[...]} / 单个书源对象；缺失必填字段补默认 */
function normalizeSources(raw: unknown): BookSource[] {
  let arr: unknown[] = []
  if (Array.isArray(raw)) {
    arr = raw
  } else if (raw && typeof raw === 'object') {
    const obj = raw as Record<string, unknown>
    if (Array.isArray(obj.bookSourceList)) arr = obj.bookSourceList
    else if (obj.bookSourceUrl) arr = [obj]
  }
  const out: BookSource[] = []
  for (const item of arr) {
    if (!item || typeof item !== 'object') continue
    const s = item as Record<string, unknown>
    const url = typeof s.bookSourceUrl === 'string' ? s.bookSourceUrl.trim() : ''
    if (!url) continue
    out.push({
      ...(s as unknown as BookSource),
      bookSourceUrl: url,
      bookSourceName:
        typeof s.bookSourceName === 'string' && s.bookSourceName.trim()
          ? s.bookSourceName.trim()
          : url,
      bookSourceType: typeof s.bookSourceType === 'number' ? s.bookSourceType : 0,
      customOrder: typeof s.customOrder === 'number' ? s.customOrder : 0,
      enabled: typeof s.enabled === 'boolean' ? s.enabled : true,
      enabledExplore: typeof s.enabledExplore === 'boolean' ? s.enabledExplore : false,
      lastUpdateTime: typeof s.lastUpdateTime === 'number' ? s.lastUpdateTime : 0,
      respondTime: typeof s.respondTime === 'number' ? s.respondTime : 0,
      weight: typeof s.weight === 'number' ? s.weight : 0,
    })
  }
  return out
}

async function confirmImport() {
  if (importBusy.value) return
  const url = importUrl.value.trim()
  if (!url) return
  importBusy.value = true
  importTip.value = ''
  try {
    const res = await previewRemoteSource(url)
    const list = res.data?.sources
    if (!res.isSuccess || !list || list.length === 0) {
      importTip.value = '未识别到书源（需为书源数组或含 bookSourceList 的对象）'
      return
    }
    importBusy.value = false
    closeImport()
    previewRemoteUrl.value = ''
    openPreview(list, new Set(res.data?.existing ?? []), '导入远程书源', 'remote')
  } catch (err) {
    importTip.value =
      err instanceof Error && err.message
        ? `导入失败：${err.message}（若为浏览器跨域限制，可下载后手动新增）`
        : '导入失败，请检查地址'
  } finally {
    importBusy.value = false
  }
}

/* ================= 订阅源（远程书源订阅，后端 /reader3/getSourceSubs 等为主，localStorage 降级，见 api/sourceSubs.ts） ================= */
const subs = ref<SourceSub[]>([])
const subUrl = ref('')
const subBusy = ref(false)
const subBusyUrls = ref<Set<string>>(new Set())
const subMsg = ref('')
const subMsgError = ref(false)

function setSubMsg(msg: string, isError = false) {
  subMsg.value = msg
  subMsgError.value = isError
}

/** 拉取远程书源 JSON 并批量导入，返回导入数量 */
async function fetchAndImport(url: string): Promise<number> {
  const resp = await fetch(url, { mode: 'cors' })
  if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
  const raw: unknown = await resp.json()
  const list = normalizeSources(raw)
  if (list.length === 0) throw new Error('未识别到书源（需为书源数组或含 bookSourceList 的对象）')
  const res = await saveBookSources(list)
  return res.data?.count ?? list.length
}

/**
 * 刷新订阅并导入书源：后端 POST /reader3/refreshSourceSub 优先（服务端拉取远程 JSON 并导入书源表）；
 * 后端不可用时降级为前端 fetch + saveBookSources（preFetched 可复用已拉取的列表，避免二次请求）。
 */
async function refreshAndImport(url: string, preFetched?: BookSource[]): Promise<number> {
  const res = await refreshSourceSub(url)
  if (res.isSuccess) return res.data?.count ?? preFetched?.length ?? 0
  if (preFetched) {
    const saveRes = await saveBookSources(preFetched)
    return saveRes.data?.count ?? preFetched.length
  }
  return fetchAndImport(url)
}

/** 新增订阅：服务端抓取（saveSourceSub 后端拉取远程 JSON——避免浏览器 CORS）+ 导入 */
async function confirmAddSub() {
  if (subBusy.value) return
  const url = subUrl.value.trim()
  if (!url) return
  subBusy.value = true
  setSubMsg('')
  try {
    // 先预览：服务端抓取优先，失败降级前端 fetch，均只展示不写库
    let list: BookSource[] | null = null
    let existing: Set<string> = new Set()
    const preview = await previewSourceSub(url)
    if (preview.isSuccess && preview.data?.sources?.length) {
      list = preview.data.sources
      existing = new Set(preview.data.existing ?? [])
    } else {
      try {
        const resp = await fetch(url, { mode: 'cors' })
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
        const raw: unknown = await resp.json()
        const parsed = normalizeSources(raw)
        if (parsed.length > 0) {
          list = parsed
          existing = new Set(sources.value.map((s) => s.bookSourceUrl))
        }
      } catch {
        // 下方统一报错
      }
    }
    if (!list) {
      throw new Error(
        preview.errorMsg || '未识别到书源（需为书源数组或含 bookSourceList 的对象）',
      )
    }
    previewRemoteUrl.value = url
    openPreview(list, existing, '添加订阅书源', 'sub')
    setSubMsg('请选择要导入的书源后确认（支持全选/反选/选择新增/排序）')
  } catch (err) {
    setSubMsg(
      `订阅失败：${err instanceof Error && err.message ? err.message : '未知错误'}`,
      true,
    )
  } finally {
    subBusy.value = false
  }
}

/** 刷新订阅：重新拉取远程书源并批量导入（后端 refreshSourceSub / 降级前端导入） */
async function refreshSub(sub: SourceSub) {
  if (subBusyUrls.value.has(sub.url)) return
  subBusyUrls.value.add(sub.url)
  try {
    const count = await refreshAndImport(sub.url)
    setSubMsg(`已刷新「${sub.name}」，导入 ${count} 个书源`)
    await load()
  } catch (err) {
    setSubMsg(
      `刷新失败：${err instanceof Error && err.message ? err.message : '未知错误'}（若为浏览器跨域限制，可下载后手动新增）`,
      true,
    )
  } finally {
    subBusyUrls.value.delete(sub.url)
  }
}

/** 单条订阅启停：禁用后停止自动刷新，订阅记录与已导入书源保留 */
async function toggleSubEnabled(sub: SourceSub) {
  if (subBusyUrls.value.has(sub.url)) return
  const next = !(sub.enabled ?? true)
  const prev = sub.enabled ?? true
  sub.enabled = next
  try {
    const res = await setSourceSubEnabled(sub.url, next)
    if (!res.isSuccess) {
      sub.enabled = prev
      setSubMsg(res.errorMsg || '操作失败', true)
      return
    }
    setSubMsg(
      next
        ? `已启用订阅「${sub.name}」：恢复自动刷新`
        : `已禁用订阅「${sub.name}」：停止自动刷新（已导入书源保留）`,
    )
  } catch {
    sub.enabled = prev
  }
}

/** 订阅全选/取消全选（顶部批量工具栏） */
function toggleSubSelectAll() {
  if (subSelected.value.size === subs.value.length) {
    subSelected.value = new Set()
  } else {
    subSelected.value = new Set(subs.value.map((s) => s.url))
  }
}

/** 批量启停选中订阅 */
async function batchSetSubsEnabled(enabled: boolean) {
  const targets = subs.value.filter(
    (s) => subSelected.value.has(s.url) && (s.enabled ?? true) !== enabled,
  )
  if (!targets.length) {
    setSubMsg(`所选订阅已全部处于${enabled ? '启用' : '禁用'}状态`)
    return
  }
  subBusy.value = true
  try {
    let ok = 0
    for (const s of targets) {
      try {
        const res = await setSourceSubEnabled(s.url, enabled)
        if (res.isSuccess) {
          s.enabled = enabled
          ok++
        }
      } catch {
        // 单条失败继续
      }
    }
    setSubMsg(`已${enabled ? '启用' : '禁用'} ${ok} 个订阅（保留订阅记录与已导入书源）`)
    if (ok === targets.length) subSelected.value = new Set()
  } finally {
    subBusy.value = false
  }
}

/* 删除订阅（后端优先；降级删除本地记录） */
const deletingSub = ref<SourceSub | null>(null)
const deletingSubs = ref<SourceSub[]>([])
const deleteSubBusy = ref(false)
/** 订阅批量选择 */
const subSelected = ref<Set<string>>(new Set())

function toggleSubSel(url: string) {
  const next = new Set(subSelected.value)
  if (next.has(url)) next.delete(url)
  else next.add(url)
  subSelected.value = next
}

function askDeleteSub(sub: SourceSub) {
  deletingSub.value = sub
  deletingSubs.value = []
  document.body.style.overflow = 'hidden'
}

function askDeleteSubs(list: SourceSub[]) {
  if (!list.length) return
  deletingSub.value = null
  deletingSubs.value = list
  document.body.style.overflow = 'hidden'
}

async function confirmDeleteSub() {
  const s = deletingSub.value
  const list = deletingSubs.value
  if ((!s && !list.length) || deleteSubBusy.value) return
  deleteSubBusy.value = true
  try {
    if (list.length) {
      const res = await deleteSourceSubs(list.map((x) => x.url))
      const removed = new Set(list.map((x) => x.url))
      subs.value = subs.value.filter((x) => !removed.has(x.url))
      subSelected.value = new Set()
      setSubMsg(
        `已删除 ${res.data?.deleted ?? list.length} 个订阅：自动刷新不再导入书源（已导入的书源保留）`,
      )
    } else if (s) {
      await deleteSourceSub(s.url)
      subs.value = subs.value.filter((x) => x.url !== s.url)
      setSubMsg('已删除订阅：自动刷新不再导入书源（已导入的书源保留）')
    }
    closeDeleteSub()
  } catch {
    // 已提示
  } finally {
    deleteSubBusy.value = false
  }
}

function closeDeleteSub() {
  deletingSub.value = null
  deletingSubs.value = []
  document.body.style.overflow = ''
}

async function loadSubs() {
  const res = await getSourceSubs() // 后端优先；失败降级 localStorage（api 层已处理）
  subs.value = res.data ?? []
}

onMounted(() => {
  syncLoggedUrls()
  load()
  void loadSubs()
  // 简繁模式可能在其他页面改动 → 挂载时同步全站状态（书源名展示随其响应）
  syncHanMode()
})

onBeforeUnmount(() => {
  window.clearTimeout(groupLongPressTimer)
  groupLongPressTimer = undefined
})
</script>

<template>
  <div class="sources-page">
    <!-- 极简顶栏：返回书架（P3-A：共享 TopNav minimal） -->
    <TopNav variant="minimal" back-label="书架" @back="router.push('/')" />

    <main class="content">
      <div class="section-head">
        <h1 class="page-title">书源管理</h1>
        <span class="count">{{ sources.length }} 个 · {{ enabledCount }} 启用</span>
        <div class="head-actions">
          <button class="ghost-btn" type="button" :disabled="localImportBusy" @click="openLocalImport">
            {{ localImportBusy ? '导入中…' : '本地导入' }}
          </button>
          <button class="ghost-btn" type="button" @click="openImport">远程导入</button>
          <button
            class="ghost-btn"
            type="button"
            :disabled="invalidChecking"
            title="检测失效书源（GET /reader3/getInvalidBookSources）"
            @click="checkInvalid"
          >
            {{ invalidChecking ? '检测中…' : '检测失效' }}
          </button>
          <button
            class="ghost-btn"
            type="button"
            :disabled="exporting"
            :title="manageMode && selectedCount > 0 ? `导出勾选的 ${selectedCount} 个书源（bookSource.json）` : '下载当前账号全部书源（bookSource.json）'"
            @click="doExport"
          >
            {{ exporting ? '导出中…' : manageMode && selectedCount > 0 ? `导出勾选 (${selectedCount})` : '导出' }}
          </button>
          <button
            class="ghost-btn"
            type="button"
            :class="{ active: manageMode }"
            :title="manageMode ? '退出多选模式' : '多选模式：勾选行后批量启用/禁用/删除/导出'"
            @click="toggleManage"
          >
            {{ manageMode ? '完成' : '多选' }}
          </button>
          <button
            class="ghost-btn"
            type="button"
            :class="{ active: sortMode }"
            :title="sortMode ? '退出排序模式（未保存的排序将自动保存）' : '排序模式：拖动手柄调整书源顺序，保存到权重（越大越靠前）'"
            @click="toggleSortMode"
          >
            {{ sortMode ? '完成' : '排序' }}
          </button>
          <button
            class="ghost-btn"
            type="button"
            title="Cookie 管理：已登录书源列表 + 清除登录态（POST /reader3/setBookSourceCookie）"
            @click="openCookieMgr"
          >
            Cookie 管理
          </button>
          <button class="accent-outline-btn" type="button" @click="openAdd">新增书源</button>
          <input
            ref="localFileInput"
            class="visually-hidden"
            type="file"
            accept=".json,application/json"
            @change="onLocalFilePick"
          />
        </div>
      </div>

      <!-- 系统配置模式提示（仅管理员） -->
      <p v-if="store.isAdmin && store.defaultConfigMode" class="default-mode-note">
        正在编辑系统配置（default）：书源等公用数据对所有用户生效
      </p>

      <!-- 多选模式批量操作栏（GAP 27：批量启用/禁用/删除 + 勾选导出） -->
      <div v-if="manageMode" class="batch-bar">
        <button class="ghost-btn batch-all" type="button" :disabled="batchBusy || filtered.length === 0" @click="toggleSelectAll">
          {{ allFilteredSelected ? '取消全选' : '全选' }}
        </button>
        <span class="batch-count">已选 {{ selectedCount }} 个</span>
        <span class="batch-sep"></span>
        <button class="batch-btn" type="button" :disabled="batchBusy || selectedCount === 0" @click="batchEnable">
          批量启用
        </button>
        <button class="batch-btn" type="button" :disabled="batchBusy || selectedCount === 0" @click="batchDisable">
          批量禁用
        </button>
        <button class="batch-btn danger" type="button" :disabled="batchBusy || selectedCount === 0" @click="batchDelete">
          {{ batchBusy ? '处理中…' : '批量删除' }}
        </button>
        <button class="batch-btn" type="button" :disabled="exporting || selectedCount === 0" @click="doExport">
          导出勾选
        </button>
      </div>

      <!-- 订阅源：远程书源订阅（后端 /reader3/getSourceSubs 等为主，localStorage 降级，见 api/sourceSubs.ts） -->
      <section class="subs-section">
        <div class="subs-head">
          <h2 class="subs-title">订阅源</h2>
          <span class="subs-sub">远程书源订阅 · 已接入服务端（账号内多设备一致；服务不可用时降级本地存储）</span>
          <div v-if="subs.length > 0" class="subs-toolbar">
            <label class="subs-all">
              <input
                type="checkbox"
                :checked="subSelected.size === subs.length"
                :indeterminate="subSelected.size > 0 && subSelected.size < subs.length"
                @change="toggleSubSelectAll"
              />
              <span>全选</span>
            </label>
            <span v-if="subSelected.size" class="subs-bulk-count">已选 {{ subSelected.size }} 个</span>
            <button
              v-if="subSelected.size"
              class="batch-btn"
              type="button"
              :disabled="subBusy"
              @click="batchSetSubsEnabled(true)"
            >
              批量启用
            </button>
            <button
              v-if="subSelected.size"
              class="batch-btn"
              type="button"
              :disabled="subBusy"
              @click="batchSetSubsEnabled(false)"
            >
              批量禁用
            </button>
            <button
              v-if="subSelected.size"
              class="batch-btn danger"
              type="button"
              :disabled="subBusy"
              @click="askDeleteSubs(subs.filter((s) => subSelected.has(s.url)))"
            >
              删除选中
            </button>
            <button
              v-if="subSelected.size"
              class="ghost-btn"
              type="button"
              @click="subSelected = new Set()"
            >
              取消选择
            </button>
          </div>
        </div>
        <form class="subs-add" @submit.prevent="confirmAddSub">
          <input
            v-model="subUrl"
            class="filter-input subs-input"
            type="text"
            placeholder="订阅书源 JSON 地址，如 https://…/bookSource.json"
            spellcheck="false"
          />
          <button class="accent-outline-btn" type="submit" :disabled="subBusy || !subUrl.trim()">
            {{ subBusy ? '订阅中…' : '订阅' }}
          </button>
        </form>
        <p v-if="subMsg" class="subs-msg" :class="{ error: subMsgError }">{{ subMsg }}</p>
        <p v-if="subs.length === 0" class="subs-empty">
          暂无订阅。订阅后书源将批量导入；禁用订阅即停止自动刷新（已导入书源保留）。
        </p>
        <ul v-else class="subs-list">
          <li v-for="sub in subs" :key="sub.url" class="subs-row">
            <input
              class="sub-check"
              type="checkbox"
              :checked="subSelected.has(sub.url)"
              :aria-label="`选择订阅 ${sub.name}`"
              @click.stop
              @change="toggleSubSel(sub.url)"
            />
            <div class="subs-main">
              <p class="subs-name" :title="hanText(sub.name)">{{ hanText(sub.name) }}</p>
              <p class="subs-url" :title="sub.url">{{ sub.url }}</p>
            </div>
            <button
              class="subs-toggle"
              type="button"
              :class="{ off: !(sub.enabled ?? true) }"
              :title="(sub.enabled ?? true) ? '启用中：点击禁用（停止自动刷新）' : '已禁用：点击启用（恢复自动刷新）'"
              :disabled="subBusyUrls.has(sub.url)"
              @click="toggleSubEnabled(sub)"
            >
              <span class="subs-toggle-track">
                <span class="subs-toggle-thumb"></span>
              </span>
              <span class="subs-toggle-text">{{ (sub.enabled ?? true) ? '启用' : '禁用' }}</span>
            </button>
            <button
              class="refresh-btn"
              type="button"
              title="刷新订阅（重新拉取并导入书源）"
              :disabled="subBusyUrls.has(sub.url)"
              @click="refreshSub(sub)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 11a8 8 0 1 0-2.3 6.3" />
                <path d="M20 5v6h-6" />
              </svg>
            </button>
            <button class="delete-btn" type="button" title="删除订阅（停止自动刷新；已导入书源保留）" @click="askDeleteSub(sub)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16" />
                <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                <path d="M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13" />
              </svg>
            </button>
          </li>
        </ul>
      </section>

      <!-- 分组筛选（细字胶囊）+ 搜索过滤 -->
      <div class="filter-row">
        <div class="group-capsules">
          <button
            class="capsule"
            :class="{ active: activeGroup === '全部' }"
            type="button"
            @click="activeGroup = '全部'"
          >
            全部
          </button>
          <button
            v-for="g in groups"
            :key="g"
            class="capsule"
            :class="{ active: activeGroup === g }"
            type="button"
            :title="`右键/长按管理分组「${g}」`"
            @click="onGroupCapsuleClick(g)"
            @contextmenu.prevent="openGroupMenu(g, $event.clientX, $event.clientY)"
            @touchstart.passive="onGroupTouchStart(g, $event)"
            @touchend="onGroupTouchEnd"
            @touchmove.passive="onGroupTouchMove"
            @touchcancel="onGroupTouchEnd"
          >
            {{ g }}
          </button>
        </div>
        <button
          v-if="activeGroup !== '全部'"
          class="group-export-btn"
          type="button"
          :disabled="exporting"
          :title="`导出「${activeGroup}」组内全部书源（bookSource.json）`"
          @click="doExportGroup"
        >
          {{ exporting ? t('common.exporting') : `导出本组（${activeGroup}）` }}
        </button>
        <div class="filter-box">
          <svg class="filter-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
            <circle cx="11" cy="11" r="6.5" />
            <path d="M20 20l-3.8-3.8" />
          </svg>
          <input
            v-model="filterKey"
            class="filter-input"
            type="text"
            placeholder="筛选名称 / 地址"
            spellcheck="false"
          />
        </div>
      </div>

      <!-- 失效检测结果提示 -->
      <p v-if="invalidMsg" class="invalid-note" :class="{ error: invalidMsgError }">{{ invalidMsg }}</p>

      <!-- 加载态 -->
      <div v-if="loading" class="state-row">
        <svg class="mini-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <path d="M21 12a9 9 0 1 1-6.2-8.56" />
        </svg>
        <span class="state-text">加载中…</span>
      </div>

      <!-- 错误态 -->
      <div v-else-if="errorMsg" class="state-row">
        <span class="state-text error">{{ errorMsg }}</span>
        <button class="retry-btn" type="button" @click="load">重试</button>
      </div>

      <!-- 空状态 -->
      <div v-else-if="filtered.length === 0" class="state-row">
        <span class="state-text">
          {{ sources.length === 0 ? '暂无书源，点击右上角新增或远程导入' : '没有匹配的书源' }}
        </span>
      </div>

      <!-- 书源列表 -->
      <ul v-else class="source-list">
        <li
          v-for="s in filtered"
          :key="s.bookSourceUrl"
          class="source-row"
          :class="{ invalid: invalidSources.has(s.bookSourceUrl), selected: selectedSources.has(s.bookSourceUrl), sorting: sortMode, 'drag-over': dragOverUrl === s.bookSourceUrl }"
          @dragover="onSourceDragOver(s, $event)"
          @drop="onSourceDrop(s, $event)"
        >
          <!-- 排序模式：拖拽手柄列（仅手柄可拖，避免误拖） -->
          <span
            v-if="sortMode"
            class="source-drag"
            draggable="true"
            title="拖拽调整顺序（越靠前权重越大；松手后点「保存排序」）"
            @dragstart="onSourceDragStart(s, $event)"
            @dragend="onSourceDragEnd"
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <circle cx="9" cy="6" r="1.4" />
              <circle cx="15" cy="6" r="1.4" />
              <circle cx="9" cy="12" r="1.4" />
              <circle cx="15" cy="12" r="1.4" />
              <circle cx="9" cy="18" r="1.4" />
              <circle cx="15" cy="18" r="1.4" />
            </svg>
          </span>
          <button
            v-if="manageMode"
            class="select-box"
            :class="{ on: selectedSources.has(s.bookSourceUrl) }"
            type="button"
            role="checkbox"
            :aria-checked="selectedSources.has(s.bookSourceUrl)"
            :title="selectedSources.has(s.bookSourceUrl) ? '取消勾选' : '勾选'"
            @click="toggleSelect(s)"
          >
            <svg v-if="selectedSources.has(s.bookSourceUrl)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M5 12.5l4.5 4.5L19 7" />
            </svg>
          </button>
          <div class="source-main">
            <p class="source-name" :title="hanText(s.bookSourceName)">{{ hanText(s.bookSourceName) }}</p>
            <p class="source-url" :title="s.bookSourceUrl">{{ s.bookSourceUrl }}</p>
          </div>
          <span v-if="s.bookSourceGroup" class="source-group" :title="s.bookSourceGroup">
            {{ s.bookSourceGroup }}
          </span>
          <span v-if="invalidSources.has(s.bookSourceUrl)" class="source-badge invalid">失效</span>
          <span v-if="loggedUrls.has(s.bookSourceUrl)" class="source-badge logged" title="已登录（Cookie 存于服务端，本地缓存）">已登录</span>
          <button
            v-if="defaultField"
            class="default-btn"
            :class="{ on: isDefaultSource(s) }"
            type="button"
            :disabled="defaultBusy.has(s.bookSourceUrl)" :title="isDefaultSource(s) ? '当前默认书源' : '设为默认书源'"
            @click="setDefault(s)"
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2.8l2.8 5.9 6.4.9-4.7 4.5 1.2 6.4L12 17.6l-5.7 3 1.2-6.4L2.8 9.6l6.4-.9z" />
            </svg>
          </button>
          <span class="source-state" :class="{ on: s.enabled }">{{ s.enabled ? '启用' : '停用' }}</span>
          <button
            class="test-btn"
            type="button"
            title="调试书源（搜索 / 目录 / 正文，SSE 逐步日志）"
            @click="openDebug(s)"
          >
            测试
          </button>
          <button class="test-btn" type="button" title="登录书源（用户名/密码 · 验证码 · 手动 Cookie）" @click="openLogin(s)">
            登录
          </button>
          <button class="test-btn" type="button" title="编辑书源（基本信息 + 规则字段）" @click="openEdit(s)">
            编辑
          </button>
          <button
            class="switch"
            :class="{ on: s.enabled }"
            type="button"
            role="switch"
            :aria-checked="s.enabled"
            :title="s.enabled ? '停用' : '启用'"
            @click="toggleSource(s)"
          >
            <span class="switch-knob"></span>
          </button>
          <button class="delete-btn" type="button" title="删除书源" @click="askDelete(s)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 7h16" />
              <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
              <path d="M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13" />
            </svg>
          </button>
        </li>
      </ul>

      <!-- 排序模式：拖拽保存栏（拖拽后出现「保存排序」；顺序变动 → 按新顺序重排 weight） -->
      <div v-if="sortMode" class="sort-save-bar">
        <span class="sort-save-tip">拖动手柄调整书源顺序 · 越靠前权重越大（保存后影响搜索/探索排序）</span>
        <button
          v-if="sortDirty"
          class="accent-btn"
          type="button"
          :disabled="sortSaving"
          @click="saveSourceOrder"
        >
          {{ sortSaving ? '保存中…' : '保存排序' }}
        </button>
        <span v-else class="sort-save-idle">顺序未变动</span>
      </div>

    </main>

    <!-- 新增书源弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="addOpen" class="dlg-overlay" @click.self="closeAdd">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="新增书源" tabindex="-1" @keydown.esc="closeAdd">
            <div class="dlg-head">
              <h2 class="dlg-title">新增书源</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="addBusy" @click="closeAdd">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmAdd">
              <label class="field">
                <span class="field-label">URL<em>*</em></span>
                <input v-model="addForm.bookSourceUrl" class="field-input" type="text" placeholder="https://example.com" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">名称</span>
                <input v-model="addForm.bookSourceName" class="field-input" type="text" placeholder="留空则使用 URL" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">分组</span>
                <input v-model="addForm.bookSourceGroup" class="field-input" type="text" placeholder="可留空，多个分组用空格分隔" spellcheck="false" />
              </label>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="addBusy" @click="closeAdd">取消</button>
                <button class="accent-btn" type="submit" :disabled="addBusy || !addForm.bookSourceUrl.trim()">
                  {{ addBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 远程导入弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="importOpen" class="dlg-overlay" @click.self="closeImport">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="远程导入书源" tabindex="-1" @keydown.esc="closeImport">
            <div class="dlg-head">
              <h2 class="dlg-title">远程导入书源</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="importBusy" @click="closeImport">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmImport">
              <label class="field">
                <span class="field-label">书源 JSON 地址<em>*</em></span>
                <input v-model="importUrl" class="field-input" type="text" placeholder="https://…/bookSource.json" spellcheck="false" />
              </label>
              <p class="field-tip">支持书源数组 / {bookSourceList: [...]} / 单个书源对象</p>
              <p v-if="importTip" class="field-tip" :class="{ error: importBusy }">{{ importTip }}</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="importBusy" @click="closeImport">取消</button>
                <button class="accent-btn" type="submit" :disabled="importBusy || !importUrl.trim()">
                  {{ importBusy ? '导入中…' : '导入' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 书源导入预览弹窗（选择 / 排序 / 便捷操作） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="previewOpen" class="dlg-overlay" @click.self="closePreview">
          <div
            class="dlg preview-dlg"
            role="dialog"
            aria-modal="true"
            :aria-label="previewTitle"
            tabindex="-1"
            @keydown.esc="closePreview"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">{{ previewTitle }}</h2>
              <button
                class="dlg-close"
                type="button"
                title="关闭"
                :disabled="previewBusy"
                @click="closePreview"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="preview-tools">
              <button class="ghost-btn mini-btn" type="button" @click="previewSelectAll">全选</button>
              <button class="ghost-btn mini-btn" type="button" @click="previewSelectNone">取消全选</button>
              <button class="ghost-btn mini-btn" type="button" @click="previewInvert">反选</button>
              <button class="ghost-btn mini-btn" type="button" @click="previewSelectNew">选择新增</button>
              <span class="preview-count">已选 {{ previewCount() }} / {{ previewList.length }}</span>
            </div>
            <div class="preview-list">
              <div
                v-for="(s, i) in previewList"
                :key="s.bookSourceUrl"
                class="preview-row"
                :class="{ checked: previewSelected[i] }"
              >
                <label class="preview-check">
                  <input v-model="previewSelected[i]" type="checkbox" />
                  <span class="preview-idx">{{ i + 1 }}</span>
                </label>
                <div class="preview-meta">
                  <div class="preview-name">
                    {{ s.bookSourceName || s.bookSourceUrl }}
                    <em v-if="previewExisting.has(s.bookSourceUrl)" class="preview-dup" title="库中已存在，覆盖更新">重复</em>
                  </div>
                  <div class="preview-url" :title="s.bookSourceUrl">{{ s.bookSourceUrl }}</div>
                </div>
                <div class="preview-move">
                  <button class="mini-icon" type="button" title="上移" :disabled="i === 0" @click="previewMove(i, -1)">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M18 15l-6-6-6 6" />
                    </svg>
                  </button>
                  <button class="mini-icon" type="button" title="下移" :disabled="i === previewList.length - 1" @click="previewMove(i, 1)">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M6 9l6 6 6-6" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="previewBusy" @click="closePreview">取消</button>
              <button class="accent-btn" type="button" :disabled="previewBusy || previewCount() === 0" @click="confirmPreview">
                {{ previewBusy ? '导入中…' : `导入 ${previewCount()} 个` }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 删除确认弹窗（极简） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="deleting" class="dlg-overlay" @click.self="closeDelete">
          <div class="dlg dlg-confirm" role="alertdialog" aria-modal="true" aria-label="删除书源" tabindex="-1" @keydown.esc="closeDelete">
            <div class="dlg-head">
              <h2 class="dlg-title">删除书源</h2>
            </div>
            <p class="confirm-text">
              确定删除「{{ deleting.bookSourceName }}」吗？此操作不可恢复。
            </p>
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

    <!-- 删除订阅确认弹窗（极简） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="deletingSub || deletingSubs.length" class="dlg-overlay" @click.self="closeDeleteSub">
          <div class="dlg dlg-confirm" role="alertdialog" aria-modal="true" aria-label="删除订阅" tabindex="-1" @keydown.esc="closeDeleteSub">
            <div class="dlg-head">
              <h2 class="dlg-title">删除订阅</h2>
            </div>
            <p class="confirm-text">
              <template v-if="deletingSubs.length">
                确定删除选中的 {{ deletingSubs.length }} 个订阅吗？删除后自动刷新不再导入书源，已导入的书源保留。
              </template>
              <template v-else>
                确定删除订阅「{{ deletingSub?.name }}」吗？删除后自动刷新不再导入书源，已导入的书源保留。
              </template>
            </p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="deleteSubBusy" @click="closeDeleteSub">取消</button>
              <button class="danger-btn" type="button" :disabled="deleteSubBusy" @click="confirmDeleteSub">
                {{ deleteSubBusy ? '删除中…' : '删除' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- GAP 28：分组胶囊右键/长按菜单（重命名 / 删除组） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="groupMenu" class="ctx-overlay" @click="closeGroupMenu" @contextmenu.prevent="closeGroupMenu">
          <div
            class="ctx-menu"
            :style="{ left: groupMenu.x + 'px', top: groupMenu.y + 'px' }"
            @click.stop
          >
            <div class="ctx-title" :title="groupMenu.name">分组：{{ groupMenu.name }}</div>
            <button
              class="ctx-item"
              type="button"
              :disabled="groupMenuBusy"
              @click="renameGroup(groupMenu.name)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z" />
              </svg>
              重命名分组
            </button>
            <button
              class="ctx-item danger"
              type="button"
              :disabled="groupMenuBusy"
              @click="deleteGroupByName(groupMenu.name)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16" />
                <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                <path d="M6.5 7l.8 12a1.5 1.5 0 0 0 1.5 1.4h6.4a1.5 1.5 0 0 0 1.5-1.4l.8-12" />
              </svg>
              删除分组
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- 书源调试弹窗（GET /reader3/bookSourceDebugSSE：动作选择 + 输入 + SSE 逐步日志） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="debugOpen" class="dlg-overlay" @click.self="closeDebug">
          <div
            class="dlg dlg-debug"
            role="dialog"
            aria-modal="true"
            aria-label="书源调试"
            tabindex="-1"
            @keydown.esc="closeDebug"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">调试 · {{ debugSource?.bookSourceName }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="debugRunning" @click="closeDebug">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <div class="debug-actions">
              <button
                v-for="a in DEBUG_ACTIONS"
                :key="a.value"
                class="capsule debug-act"
                :class="{ active: debugAction === a.value }"
                type="button"
                :disabled="debugRunning"
                @click="debugAction = a.value"
              >
                {{ a.label }}
              </button>
            </div>
            <input
              v-model="debugInput"
              class="debug-input"
              type="text"
              :placeholder="debugPlaceholder"
              spellcheck="false"
              :disabled="debugRunning"
              @keydown.enter="runDebug"
            />
            <p class="field-tip">{{ debugTip }}</p>
            <div class="debug-log">
              <p v-for="(l, i) in debugLogs" :key="i" class="debug-line" :class="{ error: l.error }">
                {{ l.text }}
              </p>
              <p v-if="debugRunning" class="debug-line running">… 执行中（逐步输出）</p>
            </div>
            <p v-if="debugMsg" class="debug-msg" :class="{ error: debugMsgError }">{{ debugMsg }}</p>
            <div class="dlg-actions">
              <button v-if="debugRunning" class="ghost-btn" type="button" @click="stopDebug">停止</button>
              <template v-else>
                <button class="ghost-btn" type="button" @click="closeDebug">关闭</button>
                <button class="accent-btn" type="button" :disabled="!debugCanRun" @click="runDebug">开始调试</button>
              </template>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- 编辑书源弹窗（基本信息 + 规则字段 textarea JSON；留空 = 清除该规则，其余字段原样保留） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="editOpen" class="dlg-overlay" @click.self="closeEdit">
          <div
            class="dlg dlg-edit"
            role="dialog"
            aria-modal="true"
            aria-label="编辑书源"
            tabindex="-1"
            @keydown.esc="closeEdit"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">编辑书源 · {{ editSource?.bookSourceName }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="editBusy" @click="closeEdit">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmEdit">
              <label class="field">
                <span class="field-label">URL<em>*</em></span>
                <input
                  v-model="editForm.bookSourceUrl"
                  class="field-input"
                  type="text"
                  spellcheck="false"
                  :disabled="editBusy"
                />
                <span class="field-tip">修改 URL 将新增书源，原书源保留</span>
              </label>
              <label class="field">
                <span class="field-label">名称</span>
                <input v-model="editForm.bookSourceName" class="field-input" type="text" spellcheck="false" :disabled="editBusy" />
              </label>
              <label class="field">
                <span class="field-label">分组</span>
                <input
                  v-model="editForm.bookSourceGroup"
                  class="field-input"
                  type="text"
                  placeholder="多个分组用空格分隔"
                  spellcheck="false"
                  :disabled="editBusy"
                />
              </label>
              <label class="field">
                <span class="field-label">权重</span>
                <input
                  v-model.number="editWeight"
                  class="field-input"
                  type="number"
                  min="0"
                  step="1"
                  :disabled="editBusy"
                />
                <span class="field-tip">书源排序权重（数字越大越靠前，随 saveBookSource 提交 weight 字段）</span>
              </label>
              <!-- GAP 107：header / loginUrl / cookie 编辑 -->
              <label class="field">
                <span class="field-label">header（请求头 JSON）</span>
                <textarea
                  v-model="editHeader"
                  class="rule-textarea"
                  placeholder='{ "User-Agent": "Mozilla/5.0 …" }'
                  spellcheck="false"
                  :disabled="editBusy"
                ></textarea>
                <span class="field-tip">请求头 JSON 对象（留空 = 清除）；保存后随书源提交 header 字段</span>
              </label>
              <label class="field">
                <span class="field-label">loginUrl（登录地址）</span>
                <input
                  v-model="editLoginUrl"
                  class="field-input"
                  type="text"
                  placeholder="https://…/login"
                  spellcheck="false"
                  :disabled="editBusy"
                />
                <span class="field-tip">书源登录页地址（留空 = 清除）</span>
              </label>
              <label class="field">
                <span class="field-label">Cookie</span>
                <input
                  v-model="editCookie"
                  class="field-input"
                  type="text"
                  placeholder="粘贴 Cookie（保存时写入服务端）"
                  spellcheck="false"
                  :disabled="editBusy"
                />
                <span class="field-tip">非空时保存后调 setBookSourceCookie 写入（清除请用登录弹窗「清除 Cookie」）</span>
              </label>
              <div class="rules-head">
                <h3 class="rules-title">规则字段</h3>
                <span class="rules-sub">JSON 字段按对象编辑（单条规则）· 留空 = 清除该规则 · header/loginUrl/cookie 见上方</span>
              </div>
              <div class="rule-symbols" aria-label="规则符号快捷插入">
                <button
                  v-for="s in RULE_SYMBOLS"
                  :key="s"
                  class="rule-symbol-btn"
                  type="button"
                  :title="s === '\n' ? '插入换行' : `插入 ${s}`"
                  :disabled="editBusy"
                  @click="insertRuleSymbol(s)"
                >
                  {{ s === '\n' ? '换行' : s }}
                </button>
              </div>
              <label v-for="f in RULE_FIELDS" :key="f.key" class="field rule-field">
                <span class="field-label">{{ f.label }}</span>
                <textarea
                  v-model="editRules[f.key]"
                  class="rule-textarea"
                  :class="{ json: f.kind === 'json' }"
                  :placeholder="f.kind === 'json' ? '{ }' : 'https://…'"
                  spellcheck="false"
                  :disabled="editBusy"
                ></textarea>
                <span class="field-tip">{{ f.tip }}</span>
              </label>
              <p v-if="editMsg" class="field-tip" :class="{ error: editMsgError }">{{ editMsg }}</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="editBusy" @click="closeEdit">取消</button>
                <button class="accent-btn" type="submit" :disabled="editBusy || !editForm.bookSourceUrl.trim()">
                  {{ editBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- 书源登录弹窗（POST /reader3/loginBookSource：状态区 + 用户名/密码表单 + 图片验证码 + 手动 Cookie；登录态 localStorage 持久） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="loginOpen" class="dlg-overlay" @click.self="closeLogin">
          <div
            class="dlg dlg-login"
            role="dialog"
            aria-modal="true"
            aria-label="书源登录"
            tabindex="-1"
            @keydown.esc="closeLogin"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">登录 · {{ loginSource?.bookSourceName }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="loginBusy" @click="closeLogin">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>

            <!-- 状态区：已登录（本地缓存） / 未登录 / 登录态未知（getCaptcha 探测） -->
            <div class="login-status" :class="{ logged: loginState === 'logged', not: loginState === 'not' }">
              <span class="login-state-dot"></span>
              <span class="login-state-text">
                {{ loginState === 'logged' ? '已登录' : loginState === 'not' ? '未登录' : '登录态未知' }}
              </span>
              <span v-if="loginState === 'logged' && cookieSummary" class="login-cookie-sum" :title="cookieSummary + '…'">
                Cookie {{ cookieSummary }}…
              </span>
              <span v-else-if="loginState === 'unknown' && loginProbe" class="login-probe">{{ loginProbe }}</span>
            </div>

            <!-- 表单：用户名/密码 → 登录 -->
            <form class="dlg-form" @submit.prevent="doLogin">
              <label class="field">
                <span class="field-label">用户名</span>
                <input
                  v-model="loginForm.username"
                  class="field-input"
                  type="text"
                  autocomplete="username"
                  placeholder="书源登录账号"
                  spellcheck="false"
                  :disabled="loginBusy"
                />
              </label>
              <label class="field">
                <span class="field-label">密码</span>
                <input
                  v-model="loginForm.password"
                  class="field-input"
                  type="password"
                  autocomplete="current-password"
                  placeholder="书源登录密码"
                  :disabled="loginBusy"
                />
              </label>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="loginBusy" @click="closeLogin">关闭</button>
                <button class="accent-btn" type="submit" :disabled="loginBusy">
                  {{ loginBusy ? '登录中…' : '登录' }}
                </button>
              </div>
            </form>

            <!-- 图片验证码区：needCaptcha=true（或探测命中）→ 显示 captchaUrl 图片 + 输入 → submitCaptcha -->
            <div v-if="captcha" class="captcha-box">
              <div class="captcha-head">
                <span class="field-label">图片验证码</span>
                <button class="captcha-refresh" type="button" :disabled="loginBusy || probing" @click="probeCaptcha(true)">
                  {{ probing ? '刷新中…' : '刷新验证码' }}
                </button>
              </div>
              <img v-if="captcha.captchaUrl" class="captcha-img" :src="captcha.captchaUrl" alt="验证码图片" />
              <p class="field-tip">{{ captcha.message }}</p>
              <div class="captcha-row">
                <input
                  v-model="captchaText"
                  class="field-input"
                  type="text"
                  placeholder="输入验证码"
                  spellcheck="false"
                  :disabled="loginBusy"
                  @keydown.enter="doSubmitCaptcha"
                />
                <button
                  class="accent-outline-btn"
                  type="button"
                  :disabled="loginBusy || !captchaText.trim()"
                  @click="doSubmitCaptcha"
                >
                  {{ loginBusy ? '提交中…' : '提交验证码' }}
                </button>
              </div>
            </div>

            <!-- 手动 Cookie 区：needManualCaptcha=true → 提示 + 明文粘贴框 + 保存 -->
            <div v-if="showManual" class="manual-box">
              <p class="field-tip">需手动验证码：请在浏览器登录该书源后，在下方粘贴 Cookie（明文显示，便于核对）</p>
              <textarea
                v-model="manualCookie"
                class="cookie-textarea"
                placeholder="粘贴 Cookie，如 a=1; b=2"
                spellcheck="false"
                :disabled="loginBusy"
              ></textarea>
              <div class="dlg-actions">
                <button
                  class="accent-btn"
                  type="button"
                  :disabled="loginBusy || !manualCookie.trim()"
                  @click="saveManualCookie"
                >
                  {{ loginBusy ? '保存中…' : '保存 Cookie' }}
                </button>
              </div>
            </div>

            <p v-if="loginMsg" class="login-msg" :class="{ error: loginMsgError }">{{ loginMsg }}</p>

            <!-- 底部：清除 Cookie（空 cookie = 清除） -->
            <div class="dlg-actions dlg-foot">
              <button class="danger-btn" type="button" :disabled="loginBusy" @click="clearLoginCookie">清除 Cookie</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- 书源 Cookie 管理弹窗（GAP 196：服务端登录态 + 本地标记；摘要来自 getBookSourceCookie；清除走 setBookSourceCookie 空 cookie） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="cookieMgrOpen" class="dlg-overlay" @click.self="closeCookieMgr">
          <div
            class="dlg dlg-cookie"
            role="dialog"
            aria-modal="true"
            aria-label="Cookie 管理"
            tabindex="-1"
            @keydown.esc="closeCookieMgr"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">Cookie 管理</h2>
              <button class="dlg-close" type="button" title="关闭" @click="closeCookieMgr">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>

            <p class="cookie-mgr-note">
              已登录书源 {{ loggedSources.length }} 个。服务端保存 Cookie/UA/登录头，摘要来自 getBookSourceCookie；清除后该书源登录态失效。
              <span v-if="cookieRowsMsg" class="cookie-mgr-warn">{{ cookieRowsMsg }}</span>
            </p>

            <ul v-if="loggedSources.length" class="cookie-list">
              <li v-for="s in loggedSources" :key="s.bookSourceUrl" class="cookie-row">
                <span class="cookie-name" :title="s.bookSourceUrl">{{ s.bookSourceName }}</span>
                <span class="cookie-domain" :title="s.bookSourceUrl">{{ hostOf(s.bookSourceUrl) }}</span>
                <span
                  v-if="cookieRows.find((r) => r.sourceUrl === s.bookSourceUrl)"
                  class="cookie-summary"
                >
                  {{ cookiePreview(cookieRows.find((r) => r.sourceUrl === s.bookSourceUrl)!) }}
                  <span
                    v-if="cookieRows.find((r) => r.sourceUrl === s.bookSourceUrl)!.userAgent"
                    class="cookie-meta"
                  >
                    UA: {{ cookieRows.find((r) => r.sourceUrl === s.bookSourceUrl)!.userAgent }}
                  </span>
                  <span
                    v-if="cookieRows.find((r) => r.sourceUrl === s.bookSourceUrl)!.loginHeader"
                    class="cookie-meta"
                  >
                    Header: {{ cookieRows.find((r) => r.sourceUrl === s.bookSourceUrl)!.loginHeader }}
                  </span>
                </span>
                <span v-else class="cookie-summary local">本地标记（服务端无登录态）</span>
                <button
                  class="danger-btn"
                  type="button"
                  :disabled="cookieMgrBusy.has(s.bookSourceUrl)"
                  @click="clearSourceCookie(s)"
                >
                  {{ cookieMgrBusy.has(s.bookSourceUrl) ? '清除中…' : '清除' }}
                </button>
              </li>
            </ul>
            <div v-else class="state-row">
              <span class="state-text">暂无已登录书源——书源行「登录」成功或粘贴 Cookie 后会出现在这里</span>
            </div>

            <div class="dlg-actions dlg-foot">
              <button class="ghost-btn" type="button" @click="closeCookieMgr">关闭</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.sources-page {
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
  width: min(860px, 100%);
  margin: 0 auto;
  padding: 44px 32px 72px;
}
.section-head {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
  margin-bottom: 26px;
}
.page-title {
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
.head-actions {
  margin-left: auto;
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
  max-width: 100%;
  padding-bottom: 2px;
}
.head-actions > .ghost-btn,
.head-actions > .accent-outline-btn {
  flex: 0 0 auto;
}
.default-mode-note {
  margin: -12px 0 18px;
  padding: 8px 12px;
  border: 1px solid var(--accent);
  border-radius: var(--radius);
  background: var(--accent-soft);
  color: var(--accent-deep);
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
}
.local-file-input {
  display: none;
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
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.ghost-btn:hover:not(:disabled) {
  color: var(--text-1);
  border-color: var(--border-strong);
}
.accent-outline-btn {
  padding: 7px 16px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: none;
  color: var(--accent);
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
.accent-outline-btn:hover:not(:disabled) {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}
.ghost-btn:disabled,
.accent-outline-btn:disabled,
.accent-btn:disabled,
.danger-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

/* ================= 筛选行 ================= */
.filter-row {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
}
.group-capsules {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.capsule {
  padding: 4px 13px;
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
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.capsule:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.capsule.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
  font-weight: 400;
}
/* 分组胶囊旁「导出本组」按钮 */
.group-export-btn {
  flex-shrink: 0;
  padding: 4px 12px;
  border-radius: 999px;
  border: 1px solid var(--accent);
  background: none;
  color: var(--accent);
  font-family: inherit;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  white-space: nowrap;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.group-export-btn:hover:not(:disabled) {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}
.group-export-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* ================= GAP 28：分组胶囊右键/长按菜单 ================= */
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
  padding: 6px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.1);
}
.ctx-title {
  padding: 4px 10px 8px;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
.filter-box {
  position: relative;
  flex-shrink: 0;
  width: 200px;
}
.filter-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 13px;
  height: 13px;
  color: var(--text-3);
  pointer-events: none;
  transition: color 0.2s ease;
}
.filter-box:focus-within .filter-icon {
  color: var(--accent);
}
.filter-input {
  width: 100%;
  height: 34px;
  padding: 0 12px 0 30px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-1);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  outline: none;
  transition: border-color 0.2s ease;
}
.filter-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.filter-input:focus {
  border-color: var(--accent);
}

/* ================= 状态行 ================= */
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

/* ================= 书源列表 ================= */
.source-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}
.source-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 13px 6px;
  border-bottom: 1px solid var(--border);
}
.source-row:first-child {
  border-top: 1px solid var(--border);
}
.source-main {
  flex: 1;
  min-width: 0;
}
.source-name {
  margin: 0;
  font-size: 13.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-url {
  margin: 3px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-group {
  flex-shrink: 0;
  max-width: 140px;
  padding: 1px 8px;
  border-radius: 4px;
  border: 1px solid var(--border);
  color: var(--text-3);
  font-size: 11px;
  font-weight: 300;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-state {
  flex-shrink: 0;
  width: 30px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.source-state.on {
  color: var(--accent);
  font-weight: 400;
}

/* ================= 默认书源星标（探测 isDefault 字段后显示） ================= */
.default-btn {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.default-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.default-btn.on {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}
.default-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.default-btn svg {
  width: 12px;
  height: 12px;
}

/* 极简开关：细线圆角条 */
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

.delete-btn {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
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
.delete-btn:hover {
  color: #cf4444;
  background: rgba(207, 68, 68, 0.06);
}
.delete-btn svg {
  width: 13px;
  height: 13px;
}

/* 刷新订阅按钮（细字图标） */
.refresh-btn {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
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
.refresh-btn:hover:not(:disabled) {
  color: var(--accent);
  background: rgba(64, 158, 120, 0.06);
}
.refresh-btn:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
.refresh-btn svg {
  width: 13px;
  height: 13px;
}

/* ================= 失效检测 ================= */
.invalid-note {
  margin: 0 0 14px;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 0.5px;
  color: var(--text-2);
}
.invalid-note.error {
  color: #cf4444;
}
/* 失效书源：整行置灰 + 名称/徽标红色 */
.source-row.invalid {
  opacity: 0.62;
}
.source-row.invalid .source-name {
  color: #cf4444;
}

/* ================= 多选模式（GAP 27）：行勾选 + 底部批量操作栏 ================= */
.source-row.selected {
  background: rgba(64, 158, 120, 0.05);
}
.select-box {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  border: 1px solid var(--border-strong);
  background: none;
  color: #fff;
  cursor: pointer;
  padding: 0;
  transition: all 0.15s ease;
}
.select-box svg {
  width: 12px;
  height: 12px;
}
.select-box.on {
  background: var(--accent);
  border-color: var(--accent);
}
.batch-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 18px;
  padding: 12px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-float);
}
.batch-all {
  padding: 6px 12px;
}
.batch-count {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  white-space: nowrap;
}
.batch-sep {
  flex: 1;
}
.batch-btn {
  padding: 6px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}
.batch-btn:hover:not(:disabled) {
  color: var(--text-1);
  border-color: var(--border-strong);
}
.batch-btn.danger {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.4);
}
.batch-btn.danger:hover:not(:disabled) {
  background: rgba(207, 68, 68, 0.08);
  border-color: #cf4444;
}
.batch-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.ghost-btn.active {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 排序模式（书源行拖拽排序） ================= */
.source-drag {
  flex-shrink: 0;
  width: 20px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
  cursor: grab;
}
.source-drag:active {
  cursor: grabbing;
}
.source-drag svg {
  width: 14px;
  height: 14px;
}
.source-row.sorting {
  cursor: default;
}
.source-row.drag-over {
  box-shadow: inset 0 -2px 0 var(--accent);
}
.source-row.drag-over .source-name {
  color: var(--accent);
}
.sort-save-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-float);
}
.sort-save-tip {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.sort-save-idle {
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.source-badge {
  flex-shrink: 0;
  padding: 1px 7px;
  border-radius: 4px;
  font-size: 10.5px;
  font-weight: 400;
  letter-spacing: 1px;
}
.source-badge.invalid {
  color: #cf4444;
  border: 1px solid rgba(207, 68, 68, 0.5);
  background: rgba(207, 68, 68, 0.06);
}

.source-badge.logged {
  color: var(--accent);
  border: 1px solid rgba(64, 158, 120, 0.5);
  background: rgba(64, 158, 120, 0.06);
}

/* 测试按钮（细字描边） */
.test-btn {
  flex-shrink: 0;
  padding: 4px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
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
.test-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* ================= 书源调试弹窗 ================= */
.dlg-debug {
  width: min(480px, 100%);
}
.debug-actions {
  display: flex;
  gap: 6px;
  margin-bottom: 10px;
}
.debug-act {
  padding: 4px 16px;
}
.debug-input {
  width: 100%;
  height: 36px;
  margin-bottom: 10px;
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
.debug-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.debug-input:focus {
  border-color: var(--accent);
  background: var(--surface);
}
.debug-input:disabled {
  opacity: 0.55;
}
.debug-log {
  min-height: 96px;
  max-height: 200px;
  margin: 8px 0 0;
  padding: 10px 12px;
  overflow-y: auto;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 11.5px;
  line-height: 1.8;
}
.debug-line {
  margin: 0;
  color: var(--text-2);
  white-space: pre-wrap;
  word-break: break-all;
}
.debug-line.error {
  color: #cf4444;
}
.debug-line.running {
  color: var(--text-3);
}
.debug-msg {
  margin: 10px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.debug-msg.error {
  color: #cf4444;
}

/* ================= 订阅源区块 ================= */
.subs-section {
  margin-top: 40px;
  padding-top: 24px;
  border-top: 1px solid var(--border);
}
.subs-head {
  display: flex;
  align-items: baseline;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}
.subs-title {
  margin: 0;
  font-size: 14px;
  font-weight: 400;
  letter-spacing: 2px;
  color: var(--text-1);
}
.subs-sub {
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  flex: 1;
  min-width: 160px;
}
.subs-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
  flex-wrap: wrap;
}
.subs-all {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  cursor: pointer;
}
.subs-all input {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}
.subs-bulk-count {
  font-size: 12px;
  font-weight: 400;
  color: var(--accent-deep);
}
.subs-add {
  display: flex;
  gap: 8px;
}
.subs-input {
  flex: 1;
  min-width: 0;
  padding: 0 12px;
}
.subs-msg {
  margin: 10px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.subs-msg.error {
  color: #cf4444;
}
.subs-empty {
  margin: 16px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.subs-list {
  list-style: none;
  margin: 14px 0 0;
  padding: 0;
}
.subs-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 11px 6px;
  border-bottom: 1px solid var(--border);
}
.subs-row:first-child {
  border-top: 1px solid var(--border);
}
.subs-main {
  flex: 1;
  min-width: 0;
}
.subs-name {
  margin: 0;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.subs-url {
  margin: 3px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.sub-check {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}
.subs-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: transparent;
  cursor: pointer;
  color: var(--text-2);
}
.subs-toggle-track {
  display: inline-flex;
  width: 26px;
  height: 14px;
  border-radius: 999px;
  background: var(--accent);
  align-items: center;
  transition: background 0.15s ease;
}
.subs-toggle-thumb {
  display: block;
  width: 10px;
  height: 10px;
  margin: 0 2px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.15s ease;
  transform: translateX(10px);
}
.subs-toggle.off .subs-toggle-track {
  background: var(--border);
}
.subs-toggle.off .subs-toggle-thumb {
  transform: translateX(0);
}
.subs-toggle-text {
  font-size: 11px;
  font-weight: 300;
}

/* ================= 弹窗（极简，自写轻量） ================= */
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
  width: min(420px, 100%);
  padding: 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  outline: none;
}
.dlg-confirm {
  width: min(360px, 100%);
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
  gap: 14px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.field-label {
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-2);
}
.field-label em {
  font-style: normal;
  color: #cf4444;
  margin-left: 2px;
}
.field-input {
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
.field-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.field-input:focus {
  border-color: var(--accent);
  background: var(--surface);
}
.field-tip {
  margin: -4px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.field-tip.error {
  color: #cf4444;
}
.confirm-text {
  margin: 0 0 18px;
  font-size: 13px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-2);
}
.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
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
  background: rgba(207, 68, 68, 0.08);
}

/* ================= 书源导入预览弹窗 ================= */
.preview-dlg {
  width: min(720px, 100%);
  max-height: 86vh;
  display: flex;
  flex-direction: column;
}
.preview-tools {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  padding: 12px 18px;
  border-bottom: 1px solid var(--border);
}
.mini-btn {
  padding: 4px 10px;
  font-size: 12px;
  letter-spacing: 0.5px;
}
.preview-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-3);
}
.preview-list {
  overflow-y: auto;
  flex: 1;
  min-height: 160px;
  max-height: 54vh;
  padding: 6px 10px;
}
.preview-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 8px;
  border-bottom: 1px solid var(--border);
  border-radius: var(--radius);
}
.preview-row.checked {
  background: rgba(var(--accent-rgb, 80, 140, 220), 0.06);
}
.preview-check {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 54px;
  cursor: pointer;
}
.preview-check input {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
}
.preview-idx {
  font-size: 11px;
  color: var(--text-3);
  min-width: 18px;
}
.preview-meta {
  flex: 1;
  min-width: 0;
}
.preview-name {
  font-size: 13px;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.preview-name em {
  margin-left: 6px;
  padding: 1px 5px;
  border-radius: 4px;
  font-style: normal;
  font-size: 10px;
  color: var(--accent);
  border: 1px solid var(--accent);
}
.preview-url {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.preview-move {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.mini-icon {
  width: 24px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: none;
  color: var(--text-2);
  cursor: pointer;
}
.mini-icon svg {
  width: 12px;
  height: 12px;
}
.mini-icon:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.mini-icon:disabled {
  opacity: 0.35;
  cursor: default;
}

/* ================= 编辑书源弹窗（规则字段 textarea） ================= */
.dlg-edit {
  width: min(680px, 100%);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
}
.dlg-edit .dlg-form {
  overflow-y: auto;
  padding-right: 6px;
}
.rules-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-top: 2px;
  padding-top: 14px;
  border-top: 1px dashed var(--border);
}
.rules-title {
  margin: 0;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.rules-sub {
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
}
.rule-symbols {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 8px 0 2px;
}
.rule-symbol-btn {
  padding: 3px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
  color: var(--text-2);
  font: 11px/1.4 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  cursor: pointer;
}
.rule-symbol-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.rule-symbol-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.rule-field {
  gap: 4px;
}
.rule-textarea {
  width: 100%;
  min-height: 32px;
  padding: 7px 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  outline: none;
  resize: vertical;
  transition: border-color 0.2s ease;
}
.rule-textarea.json {
  min-height: 120px;
}
.rule-textarea::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.rule-textarea:focus {
  border-color: var(--accent);
  background: var(--surface);
}
.rule-textarea:disabled {
  opacity: 0.55;
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

/* ================= 书源登录弹窗 ================= */
.dlg-login {
  width: min(440px, 100%);
}
/* Cookie 管理弹窗（GAP 196） */
.dlg-cookie {
  width: min(560px, 100%);
}
.cookie-mgr-note {
  margin: 0 0 14px;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-3);
}
.cookie-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 46vh;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.cookie-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
}
.cookie-row:last-child {
  border-bottom: none;
}
.cookie-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.cookie-domain {
  flex-shrink: 0;
  max-width: 160px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.cookie-summary {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  background: var(--bg);
  border: 1px dashed var(--border);
  cursor: help;
}
/* 状态区 */
.login-status {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 10px;
  margin-bottom: 14px;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.login-status.logged {
  border-color: rgba(64, 158, 120, 0.45);
  background: rgba(64, 158, 120, 0.06);
}
.login-status.not {
  border-color: rgba(207, 68, 68, 0.45);
  background: rgba(207, 68, 68, 0.05);
}
.login-state-dot {
  flex-shrink: 0;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-3);
}
.login-status.logged .login-state-dot {
  background: var(--accent);
}
.login-status.not .login-state-dot {
  background: #cf4444;
}
.login-state-text {
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.login-cookie-sum {
  min-width: 0;
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.login-probe {
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
/* 图片验证码区 */
.captcha-box {
  margin-top: 14px;
  padding: 12px;
  border-radius: 6px;
  border: 1px dashed var(--border-strong);
  background: var(--bg);
}
.captcha-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.captcha-refresh {
  border: none;
  background: none;
  padding: 0;
  color: var(--accent);
  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.captcha-refresh:hover:not(:disabled) {
  color: var(--accent-deep);
}
.captcha-refresh:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.captcha-img {
  display: block;
  max-width: 100%;
  max-height: 120px;
  margin-bottom: 8px;
  border-radius: 4px;
  border: 1px solid var(--border);
  background: #fff;
}
.captcha-row {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}
.captcha-row .field-input {
  flex: 1;
  min-width: 0;
}
/* 手动 Cookie 区（明文显示，便于核对） */
.manual-box {
  margin-top: 14px;
  padding: 12px;
  border-radius: 6px;
  border: 1px dashed rgba(207, 68, 68, 0.4);
  background: var(--bg);
}
.cookie-textarea {
  width: 100%;
  min-height: 72px;
  margin-top: 8px;
  padding: 8px 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-1);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  outline: none;
  resize: vertical;
  transition: border-color 0.2s ease;
}
.cookie-textarea:focus {
  border-color: var(--accent);
}
.cookie-textarea:disabled {
  opacity: 0.55;
}
.login-msg {
  margin: 12px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.login-msg.error {
  color: #cf4444;
}
.dlg-foot {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    padding: 12px 16px;
  }
  .content {
    padding: 32px 16px 56px;
  }
  .filter-row {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  .filter-box {
    width: 100%;
  }
  .source-group {
    display: none;
  }
  .source-state {
    display: none;
  }
  .source-row {
    min-height: 44px;
    padding: 12px 6px;
  }
  .dlg-overlay {
    padding: 16px;
  }
}
</style>
