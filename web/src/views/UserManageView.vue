<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import {
  addUser,
  clearInactiveUsers,
  deleteUser,
  deleteUsers,
  getStoredSecureKey,
  getUsers,
  isNeedSecureKey,
  isNotImplemented,
  probeSecureMode,
  resetUserPassword,
  storeSecureKey,
  updateUser,
} from '@/api/users'
import { login } from '@/api/auth'
import { useUserStore } from '@/stores/user'
import TopNav from '@/components/TopNav.vue'
import type { ReaderUser, UserUpdatePayload } from '@/types'

const store = useUserStore()

type PermField = 'enableWebdav' | 'enableLocalStore' | 'enableBookSource' | 'enableRssSource' | 'isAdmin'

const PERM_LABEL: Record<PermField, string> = {
  enableWebdav: 'WebDAV',
  enableLocalStore: '本地书仓',
  enableBookSource: '书源',
  enableRssSource: 'RSS',
  isAdmin: '管理员',
}

/* ================= 列表 ================= */
const users = ref<ReaderUser[]>([])
const loading = ref(true)
const loadFailed = ref(false)
const secureMode = ref(false) // secure 模式（用户管理需 secureKey）——探测或 NEED_SECURE_KEY 时置 true

async function loadUsers() {
  loading.value = true
  loadFailed.value = false
  try {
    const res = await getUsers()
    users.value = Array.isArray(res.data) ? res.data : []
    // 已删除用户不再留在选中集合
    const alive = new Set(users.value.map((u) => u.username))
    selected.value = new Set([...selected.value].filter((name) => alive.has(name)))
  } catch (err) {
    if (isNeedSecureKey(err)) {
      // secure 模式缺/错 secureKey → 弹管理密码输入
      secureMode.value = true
      openKeyDialog(err instanceof Error ? err.message : '请输入管理密码')
    } else {
      loadFailed.value = true
      users.value = []
    }
  } finally {
    loading.value = false
  }
}

/* ================= secureKey（管理密码） ================= */
const keyDialogOpen = ref(false)
const keyInput = ref('')
const keyError = ref('')
let pendingOp: (() => Promise<void>) | null = null

/** 管理操作遇 NEED_SECURE_KEY：记录待重试操作并弹密码框（返回 true 表示已接管） */
function handleManageError(err: unknown, op: () => Promise<void>): boolean {
  if (!isNeedSecureKey(err)) return false
  secureMode.value = true
  pendingOp = op
  openKeyDialog(err instanceof Error ? err.message : '请输入管理密码')
  return true
}

function openKeyDialog(msg = '') {
  keyInput.value = getStoredSecureKey()
  keyError.value = msg
  keyDialogOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeKeyDialog() {
  keyDialogOpen.value = false
  pendingOp = null
  document.body.style.overflow = ''
}

async function confirmKey() {
  const key = keyInput.value.trim()
  if (!key) {
    ElMessage.warning('请输入管理密码')
    return
  }
  storeSecureKey(key)
  keyDialogOpen.value = false
  document.body.style.overflow = ''
  const op = pendingOp
  pendingOp = null
  keyError.value = ''
  if (op) await op()
  else await loadUsers()
}

/* ================= 搜索（GAP 42：前端过滤用户名） ================= */
const searchKey = ref('')

const filteredUsers = computed(() => {
  const kw = searchKey.value.trim().toLowerCase()
  if (!kw) return users.value
  return users.value.filter((u) => u.username.toLowerCase().includes(kw))
})

/* ================= 多选（自己不可选） ================= */
const selected = ref<Set<string>>(new Set())

const selectableUsers = computed(() =>
  filteredUsers.value.filter((u) => u.username !== store.username),
)

const allSelected = computed(
  () =>
    selectableUsers.value.length > 0 &&
    selectableUsers.value.every((u) => selected.value.has(u.username)),
)

const someSelected = computed(() =>
  selectableUsers.value.some((u) => selected.value.has(u.username)),
)

function toggleSelect(u: ReaderUser) {
  if (u.username === store.username) return
  const next = new Set(selected.value)
  if (next.has(u.username)) next.delete(u.username)
  else next.add(u.username)
  selected.value = next
}

function toggleSelectAll() {
  const next = new Set(selected.value)
  if (allSelected.value) {
    for (const u of selectableUsers.value) next.delete(u.username)
  } else {
    for (const u of selectableUsers.value) next.add(u.username)
  }
  selected.value = next
}

function selectedNames(max = 3): string {
  const names = [...selected.value]
  if (names.length === 0) return ''
  if (names.length <= max) return names.join('、')
  return `${names.slice(0, max).join('、')} 等 ${names.length} 个`
}

/* ================= 添加用户（GAP 43：addUser 未就绪时降级 register） ================= */
const adding = ref(false)
const addBusy = ref(false)
const addForm = ref<{
  username: string
  password: string
  enableWebdav: boolean
  enableLocalStore: boolean
  enableBookSource: boolean
  enableRssSource: boolean
  isAdmin: boolean
  bookSourceLimit: number
  bookLimit: number
}>({
  username: '',
  password: '',
  enableWebdav: true,
  enableLocalStore: true,
  enableBookSource: true,
  enableRssSource: true,
  isAdmin: false,
  bookSourceLimit: 80000,
  bookLimit: 5000,
})

function openAdd() {
  addForm.value = {
    username: '',
    password: '',
    enableWebdav: true,
    enableLocalStore: true,
    enableBookSource: true,
    enableRssSource: true,
    isAdmin: false,
    bookSourceLimit: 80000,
    bookLimit: 5000,
  }
  addBusy.value = false
  adding.value = true
  document.body.style.overflow = 'hidden'
}

function closeAdd() {
  if (addBusy.value) return
  adding.value = false
  document.body.style.overflow = ''
}

async function confirmAdd() {
  if (addBusy.value) return
  const username = addForm.value.username.trim()
  const password = addForm.value.password
  if (!username) {
    ElMessage.warning('请输入用户名')
    return
  }
  if (!password) {
    ElMessage.warning('请输入密码')
    return
  }
  addBusy.value = true
  try {
    // 1) 后端 addUser（silent：未实现时静默，业务错误手动提示）
    await addUser({
      username,
      password,
      enableWebdav: addForm.value.enableWebdav,
      enableLocalStore: addForm.value.enableLocalStore,
      enableBookSource: addForm.value.enableBookSource,
      enableRssSource: addForm.value.enableRssSource,
      isAdmin: addForm.value.isAdmin,
      bookSourceLimit: Math.max(0, Number(addForm.value.bookSourceLimit) || 0),
      bookLimit: Math.max(0, Number(addForm.value.bookLimit) || 0),
    })
    ElMessage.success('已创建用户')
    closeAdd()
    await loadUsers()
  } catch (err) {
    if (handleManageError(err, confirmAdd)) return // NEED_SECURE_KEY → 密码框 + 重试
    if (isNotImplemented(err)) {
      // 2) addUser 未就绪 → register 接口降级（isLogin=false；默认权限，注册错误由拦截器提示）
      try {
        await login({ username, password, isLogin: false })
        ElMessage.success('已创建用户（默认权限，可在编辑中调整）')
        closeAdd()
        await loadUsers()
      } catch {
        // 注册业务错误已由请求层提示
      }
    } else {
      ElMessage.error(err instanceof Error ? err.message : '创建失败')
    }
  } finally {
    addBusy.value = false
  }
}

/* ================= 权限开关（表格内直接切换） ================= */
const toggling = ref<Set<string>>(new Set())

function permPayload(u: ReaderUser, field: PermField, value: boolean): UserUpdatePayload {
  return {
    username: u.username,
    enableWebdav: u.enableWebdav,
    enableLocalStore: u.enableLocalStore,
    enableBookSource: u.enableBookSource,
    enableRssSource: u.enableRssSource,
    bookSourceLimit: u.bookSourceLimit,
    bookLimit: u.bookLimit,
    isAdmin: u.isAdmin,
    [field]: value,
  }
}

async function togglePerm(u: ReaderUser, field: PermField) {
  if (toggling.value.has(u.username)) return
  if (field === 'isAdmin' && u.isAdmin && u.username === store.username) {
    ElMessage.warning('不能撤销自己的管理员权限')
    return
  }
  toggling.value.add(u.username)
  const prev = Boolean(u[field])
  u[field] = !prev // 乐观切换，失败回滚
  try {
    await updateUser(permPayload(u, field, !prev))
  } catch (err) {
    u[field] = prev
    handleManageError(err, () => togglePerm(u, field)) // NEED_SECURE_KEY → 密码框 + 重试
  } finally {
    toggling.value.delete(u.username)
  }
}

/* ================= 编辑弹窗 ================= */
const editing = ref<ReaderUser | null>(null)
const editBusy = ref(false)
const editForm = ref<{
  enableWebdav: boolean
  enableLocalStore: boolean
  enableBookSource: boolean
  enableRssSource: boolean
  isAdmin: boolean
  bookSourceLimit: number
  bookLimit: number
}>({ enableWebdav: true, enableLocalStore: true, enableBookSource: true, enableRssSource: true, isAdmin: false, bookSourceLimit: 80000, bookLimit: 5000 })

function openEdit(u: ReaderUser) {
  editing.value = u
  editForm.value = {
    enableWebdav: u.enableWebdav,
    enableLocalStore: u.enableLocalStore,
    enableBookSource: u.enableBookSource,
    enableRssSource: u.enableRssSource,
    isAdmin: u.isAdmin ?? false,
    bookSourceLimit: u.bookSourceLimit ?? 0,
    bookLimit: u.bookLimit ?? 0,
  }
  editBusy.value = false
  document.body.style.overflow = 'hidden'
}

function closeEdit() {
  if (editBusy.value) return
  editing.value = null
  document.body.style.overflow = ''
}

async function saveEdit() {
  const target = editing.value
  if (!target || editBusy.value) return
  const f = editForm.value
  const payload: UserUpdatePayload = {
    username: target.username,
    enableWebdav: f.enableWebdav,
    enableLocalStore: f.enableLocalStore,
    enableBookSource: f.enableBookSource,
    enableRssSource: f.enableRssSource,
    isAdmin: f.isAdmin,
    bookSourceLimit: Math.max(0, Number(f.bookSourceLimit) || 0),
    bookLimit: Math.max(0, Number(f.bookLimit) || 0),
  }
  editBusy.value = true
  try {
    await updateUser(payload)
    Object.assign(target, payload)
    ElMessage.success('已保存')
    closeEdit()
  } catch (err) {
    handleManageError(err, saveEdit) // NEED_SECURE_KEY → 密码框，确认后重试保存
  } finally {
    editBusy.value = false
  }
}

/* ================= 删除（自己禁删） ================= */
const deleting = ref<ReaderUser | null>(null)
const deleteBusy = ref(false)

function askDelete(u: ReaderUser) {
  if (u.username === store.username) {
    ElMessage.warning('不能删除自己')
    return
  }
  deleting.value = u
  document.body.style.overflow = 'hidden'
}

function closeDelete() {
  if (deleteBusy.value) return
  deleting.value = null
  document.body.style.overflow = ''
}

async function confirmDelete() {
  const target = deleting.value
  if (!target || deleteBusy.value) return
  if (target.username === store.username) {
    ElMessage.warning('不能删除自己')
    closeDelete()
    return
  }
  deleteBusy.value = true
  try {
    await deleteUser(target.username)
    users.value = users.value.filter((x) => x.username !== target.username)
    ElMessage.success('已删除')
    closeDelete()
  } catch (err) {
    handleManageError(err, confirmDelete)
  } finally {
    deleteBusy.value = false
  }
}

/* ================= 批量删除 ================= */
const batchDeleting = ref(false)
const batchDeleteBusy = ref(false)

function askBatchDelete() {
  if (selected.value.size === 0) return
  batchDeleting.value = true
  document.body.style.overflow = 'hidden'
}

function closeBatchDelete() {
  if (batchDeleteBusy.value) return
  batchDeleting.value = false
  document.body.style.overflow = ''
}

async function confirmBatchDelete() {
  if (batchDeleteBusy.value) return
  const targets = [...selected.value]
  if (targets.length === 0) {
    closeBatchDelete()
    return
  }
  batchDeleteBusy.value = true
  try {
    const res = await deleteUsers(targets)
    const remaining = Array.isArray(res.data) ? (res.data as ReaderUser[]) : null
    users.value = remaining ?? users.value.filter((x) => !targets.includes(x.username))
    selected.value = new Set()
    ElMessage.success(`已删除 ${targets.length} 个用户`)
    closeBatchDelete()
  } catch (err) {
    handleManageError(err, confirmBatchDelete)
  } finally {
    batchDeleteBusy.value = false
  }
}

/* ================= 清理不活跃用户 ================= */
const cleanDialogOpen = ref(false)
const cleanBusy = ref(false)
const cleanDays = ref(31)

const cleanDaysValid = computed(() => {
  const n = Number(cleanDays.value)
  return Number.isFinite(n) && n >= 0
})

function openClean() {
  cleanDays.value = 31
  cleanBusy.value = false
  cleanDialogOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeClean() {
  if (cleanBusy.value) return
  cleanDialogOpen.value = false
  document.body.style.overflow = ''
}

async function confirmClean() {
  if (cleanBusy.value) return
  const days = Math.max(0, Math.floor(Number(cleanDays.value) || 31))
  cleanBusy.value = true
  try {
    const res = await clearInactiveUsers(days)
    const data = res.data as { deleted?: string[]; count?: number } | null
    const deleted = Array.isArray(data?.deleted) ? data.deleted : []
    users.value = users.value.filter((x) => !deleted.includes(x.username))
    selected.value = new Set([...selected.value].filter((name) => !deleted.includes(name)))
    ElMessage.success(`已清理 ${deleted.length} 个不活跃用户`)
    closeClean()
  } catch (err) {
    handleManageError(err, confirmClean)
  } finally {
    cleanBusy.value = false
  }
}

/* ================= 重置密码弹窗 ================= */
const resetting = ref<ReaderUser | null>(null)
const resetBusy = ref(false)
const resetForm = ref({ newPassword: '' })

function openReset(u: ReaderUser) {
  resetting.value = u
  resetForm.value = { newPassword: '' }
  resetBusy.value = false
  document.body.style.overflow = 'hidden'
}

function closeReset() {
  if (resetBusy.value) return
  resetting.value = null
  document.body.style.overflow = ''
}

async function confirmReset() {
  const target = resetting.value
  if (!target || resetBusy.value) return
  const pwd = resetForm.value.newPassword
  if (!pwd) {
    ElMessage.warning('请输入新密码')
    return
  }
  resetBusy.value = true
  try {
    await resetUserPassword(target.username, pwd)
    ElMessage.success('密码已重置')
    closeReset()
  } catch (err) {
    handleManageError(err, confirmReset)
  } finally {
    resetBusy.value = false
  }
}

/* ================= 展示 ================= */
function fmtTime(ts: number): string {
  if (!ts) return '从未'
  const ms = ts < 1e12 ? ts * 1000 : ts // 兼容秒级时间戳
  const d = new Date(ms)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

onMounted(() => {
  void loadUsers()
  // 探测 secure 模式（决定是否提示管理密码；书架导航入口由 BookshelfView 同样探测）
  void probeSecureMode().then((v) => {
    if (v) secureMode.value = true
  })
})

onBeforeUnmount(() => {
  document.body.style.overflow = ''
})
</script>

<template>
  <div class="users-page">
    <!-- 顶部导航（P3-A：共享 TopNav） -->
    <TopNav
      active="/users"
      :links="['bookshelf', 'search', 'sources', 'rules', 'files', 'users', 'settings']"
      show-users-link
    />

    <main class="content">
      <!-- 标题区 -->
      <div class="section-head">
        <h1 class="section-title">用户管理</h1>
        <span class="count">{{ searchKey.trim() ? `${filteredUsers.length} / ${users.length}` : users.length }} 个用户</span>
        <button class="tool-btn" type="button" :disabled="selected.size === 0" title="删除选中的用户" @click="askBatchDelete">
          批量删除{{ selected.size ? `（${selected.size}）` : '' }}
        </button>
        <button class="tool-btn" type="button" title="清理 N 天未登录的用户" @click="openClean">清理不活跃</button>
        <button class="add-btn" type="button" @click="openAdd">添加用户</button>
        <button class="refresh-btn" type="button" title="刷新" @click="loadUsers()">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12a9 9 0 1 1-2.64-6.36" />
            <path d="M21 3v6h-6" />
          </svg>
        </button>
      </div>

      <!-- 搜索（GAP 42：前端过滤用户名） -->
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
            placeholder="搜索用户名"
            spellcheck="false"
          />
          <button v-if="searchKey" class="search-clear" type="button" title="清空" @click="searchKey = ''">
            ✕
          </button>
        </div>
      </div>

      <!-- secure 模式提示（细字） -->
      <p v-if="secureMode" class="secure-tip">
        当前为安全模式（secure），管理操作需要管理密码（secureKey）；密码仅保存在本浏览器会话中。
      </p>

      <!-- 加载中 -->
      <div v-if="loading" class="state-line">加载中…</div>

      <!-- 加载失败（后端未就绪 / 非 secure 接口异常） -->
      <div v-else-if="loadFailed" class="state-line">
        <span>用户列表加载失败（后端接口可能未就绪）</span>
        <button class="retry-btn" type="button" @click="loadUsers()">重试</button>
      </div>

      <!-- 空状态 -->
      <div v-else-if="users.length === 0" class="state-line">暂无用户</div>

      <!-- 搜索无匹配 -->
      <div v-else-if="filteredUsers.length === 0" class="state-line">无匹配「{{ searchKey.trim() }}」的用户</div>

      <!-- 细字用户表格 -->
      <div v-else class="table-wrap">
        <table class="user-table">
          <thead>
            <tr>
              <th class="col-check">
                <button
                  class="row-check"
                  :class="{ checked: allSelected, partial: someSelected && !allSelected }"
                  type="button"
                  role="checkbox"
                  :aria-checked="allSelected ? 'true' : someSelected ? 'mixed' : 'false'"
                  :title="allSelected ? '取消全选' : '全选'"
                  @click="toggleSelectAll"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M5 12.5l4.2 4.2L19 7.2" />
                  </svg>
                </button>
              </th>
              <th class="col-user">用户名</th>
              <th class="col-perm">权限</th>
              <th class="col-num">书源上限</th>
              <th class="col-num">书籍上限</th>
              <th class="col-time">最后登录</th>
              <th class="col-time">注册时间</th>
              <th class="col-ops">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in filteredUsers" :key="u.username">
              <td class="col-check">
                <button
                  class="row-check"
                  :class="{ checked: selected.has(u.username) }"
                  :disabled="u.username === store.username"
                  type="button"
                  role="checkbox"
                  :aria-checked="selected.has(u.username)"
                  :title="u.username === store.username ? '不能选择自己' : selected.has(u.username) ? '取消选择' : '选择'"
                  @click="toggleSelect(u)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M5 12.5l4.2 4.2L19 7.2" />
                  </svg>
                </button>
              </td>
              <td class="col-user">
                <span class="uname" :title="u.username">{{ u.username }}</span>
                <span v-if="u.username === store.username" class="self-tag" title="当前登录账号">我</span>
                <span v-if="u.isAdmin" class="admin-tag" title="管理员（可操作系统 default 配置）">管理员</span>
              </td>
              <td class="col-perm">
                <div class="perm-cell">
                  <template v-for="(label, field) in PERM_LABEL" :key="field">
                    <button
                      class="switch"
                      :class="{ on: u[field as PermField] }"
                      :disabled="toggling.has(u.username)"
                      type="button"
                      role="switch"
                      :aria-checked="u[field as PermField]"
                      :title="`${label}：${u[field as PermField] ? '开' : '关'}`"
                      @click="togglePerm(u, field as PermField)"
                    >
                      <span class="switch-knob"></span>
                    </button>
                    <span class="perm-label">{{ label }}</span>
                  </template>
                </div>
              </td>
              <td class="col-num">{{ u.bookSourceLimit ?? 0 }}</td>
              <td class="col-num">{{ u.bookLimit ?? 0 }}</td>
              <td class="col-time">{{ fmtTime(u.lastLoginAt) }}</td>
              <td class="col-time">{{ u.createdAt ? fmtTime(u.createdAt) : '—' }}</td>
              <td class="col-ops">
                <button class="op-btn" type="button" @click="openEdit(u)">编辑</button>
                <button class="op-btn" type="button" @click="openReset(u)">重置密码</button>
                <button
                  class="op-btn danger"
                  type="button"
                  :disabled="u.username === store.username"
                  :title="u.username === store.username ? '不能删除自己' : '删除用户'"
                  @click="askDelete(u)"
                >
                  删除
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </main>

    <!-- 管理密码（secureKey）弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="keyDialogOpen" class="dlg-overlay" @click.self="closeKeyDialog">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="输入管理密码" tabindex="-1" @keydown.esc="closeKeyDialog">
            <div class="dlg-head">
              <h2 class="dlg-title">输入管理密码</h2>
              <button class="dlg-close" type="button" title="关闭" @click="closeKeyDialog">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmKey">
              <p class="field-tip">当前为安全模式（secure），管理操作需提供管理密码（secureKey）。密码仅保存在本浏览器会话中。</p>
              <label class="field">
                <span class="field-label">管理密码<em>*</em></span>
                <input v-model="keyInput" class="field-input mono" type="password" placeholder="secureKey" autocomplete="off" spellcheck="false" />
              </label>
              <p v-if="keyError" class="key-error">{{ keyError }}</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" @click="closeKeyDialog">取消</button>
                <button class="accent-btn" type="submit" :disabled="!keyInput.trim()">确认</button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 添加用户弹窗（GAP 43：addUser 未就绪时降级 register，默认权限） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="adding" class="dlg-overlay" @click.self="closeAdd">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="添加用户" tabindex="-1" @keydown.esc="closeAdd">
            <div class="dlg-head">
              <h2 class="dlg-title">添加用户</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="addBusy" @click="closeAdd">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmAdd">
              <label class="field">
                <span class="field-label">用户名<em>*</em></span>
                <input v-model="addForm.username" class="field-input" type="text" placeholder="字母或数字，至少 5 位" autocomplete="off" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">密码<em>*</em></span>
                <input v-model="addForm.password" class="field-input mono" type="password" placeholder="至少 8 位" autocomplete="new-password" spellcheck="false" />
              </label>
              <div class="field">
                <span class="field-label">权限</span>
                <div class="perm-rows">
                  <div v-for="(label, field) in PERM_LABEL" :key="field" class="perm-row">
                    <span class="perm-row-label">{{ label }}</span>
                    <button
                      class="switch"
                      :class="{ on: addForm[field as PermField] }"
                      type="button"
                      role="switch"
                      :aria-checked="addForm[field as PermField]"
                      @click="addForm[field as PermField] = !addForm[field as PermField]"
                    >
                      <span class="switch-knob"></span>
                    </button>
                  </div>
                </div>
              </div>
              <div class="field-row">
                <label class="field">
                  <span class="field-label">书源上限</span>
                  <input v-model.number="addForm.bookSourceLimit" class="field-input" type="number" min="0" step="1" />
                </label>
                <label class="field">
                  <span class="field-label">书籍上限</span>
                  <input v-model.number="addForm.bookLimit" class="field-input" type="number" min="0" step="1" />
                </label>
              </div>
              <p class="field-tip">新用户默认权限全开；管理员账号可操作系统 default 书源/订阅配置。</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="addBusy" @click="closeAdd">取消</button>
                <button class="accent-btn" type="submit" :disabled="addBusy || !addForm.username.trim() || !addForm.password">
                  {{ addBusy ? '创建中…' : '创建' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 编辑用户弹窗（权限开关 + 上限输入） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="editing" class="dlg-overlay" @click.self="closeEdit">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="编辑用户" tabindex="-1" @keydown.esc="closeEdit">
            <div class="dlg-head">
              <h2 class="dlg-title">编辑用户 · {{ editing.username }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="editBusy" @click="closeEdit">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="saveEdit">
              <div class="field">
                <span class="field-label">权限</span>
                <div class="perm-rows">
                  <div v-for="(label, field) in PERM_LABEL" :key="field" class="perm-row">
                    <span class="perm-row-label">{{ label }}</span>
                    <button
                      class="switch"
                      :class="{ on: editForm[field as PermField] }"
                      type="button"
                      role="switch"
                      :aria-checked="editForm[field as PermField]"
                      @click="editForm[field as PermField] = !editForm[field as PermField]"
                    >
                      <span class="switch-knob"></span>
                    </button>
                  </div>
                </div>
              </div>
              <div class="field-row">
                <label class="field">
                  <span class="field-label">书源上限</span>
                  <input v-model.number="editForm.bookSourceLimit" class="field-input" type="number" min="0" step="1" />
                </label>
                <label class="field">
                  <span class="field-label">书籍上限</span>
                  <input v-model.number="editForm.bookLimit" class="field-input" type="number" min="0" step="1" />
                </label>
              </div>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="editBusy" @click="closeEdit">取消</button>
                <button class="accent-btn" type="submit" :disabled="editBusy">
                  {{ editBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 批量删除确认弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="batchDeleting" class="dlg-overlay" @click.self="closeBatchDelete">
          <div class="dlg dlg-confirm" role="alertdialog" aria-modal="true" aria-label="批量删除用户" tabindex="-1" @keydown.esc="closeBatchDelete">
            <div class="dlg-head">
              <h2 class="dlg-title">批量删除用户</h2>
            </div>
            <p class="confirm-text">确定删除选中的 {{ selected.size }} 个用户吗？此操作不可恢复，其书源、书籍与缓存数据将一并清理。</p>
            <p v-if="selectedNames()" class="confirm-text confirm-list">{{ selectedNames() }}</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="batchDeleteBusy" @click="closeBatchDelete">取消</button>
              <button class="danger-btn" type="button" :disabled="batchDeleteBusy" @click="confirmBatchDelete">
                {{ batchDeleteBusy ? '删除中…' : '删除' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 清理不活跃用户弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="cleanDialogOpen" class="dlg-overlay" @click.self="closeClean">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="清理不活跃用户" tabindex="-1" @keydown.esc="closeClean">
            <div class="dlg-head">
              <h2 class="dlg-title">清理不活跃用户</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="cleanBusy" @click="closeClean">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmClean">
              <label class="field">
                <span class="field-label">未登录天数<em>*</em></span>
                <input v-model.number="cleanDays" class="field-input" type="number" min="0" step="1" />
              </label>
              <p class="field-tip">删除最近 N 天未登录的用户（0 表示全部非活跃判定，当前账号与最后一名管理员不受影响）。此操作不可恢复。</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="cleanBusy" @click="closeClean">取消</button>
                <button class="danger-btn" type="submit" :disabled="cleanBusy || !cleanDaysValid">
                  {{ cleanBusy ? '清理中…' : '确认清理' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 重置密码弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="resetting" class="dlg-overlay" @click.self="closeReset">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="重置密码" tabindex="-1" @keydown.esc="closeReset">
            <div class="dlg-head">
              <h2 class="dlg-title">重置密码 · {{ resetting.username }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="resetBusy" @click="closeReset">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmReset">
              <label class="field">
                <span class="field-label">新密码<em>*</em></span>
                <input v-model="resetForm.newPassword" class="field-input mono" type="password" placeholder="输入新密码" autocomplete="new-password" spellcheck="false" />
              </label>
              <p class="field-tip">重置后，该用户将使用新密码登录。</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="resetBusy" @click="closeReset">取消</button>
                <button class="accent-btn" type="submit" :disabled="resetBusy || !resetForm.newPassword">
                  {{ resetBusy ? '重置中…' : '重置密码' }}
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
          <div class="dlg dlg-confirm" role="alertdialog" aria-modal="true" aria-label="删除用户" tabindex="-1" @keydown.esc="closeDelete">
            <div class="dlg-head">
              <h2 class="dlg-title">删除用户</h2>
            </div>
            <p class="confirm-text">确定删除用户「{{ deleting.username }}」吗？此操作不可恢复。</p>
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
  </div>
</template>

<style scoped>
.users-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  animation: fade-in 0.2s ease both;
}

/* ================= 内容区 ================= */
.content {
  width: min(1080px, 100%);
  margin: 0 auto;
  padding: 48px 32px 72px;
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
.refresh-btn {
  margin-left: auto;
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
.refresh-btn svg {
  width: 14px;
  height: 14px;
}
.add-btn {
  flex-shrink: 0;
  padding: 5px 16px;
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
    background-color 0.2s ease;
}
.add-btn:hover {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}
.tool-btn {
  flex-shrink: 0;
  padding: 5px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 0.5px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.tool-btn:hover:not(:disabled) {
  color: var(--text-1);
  border-color: var(--border-strong);
  background: var(--hover);
}
.tool-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

/* 搜索框（GAP 42） */
.filter-bar {
  margin-bottom: 18px;
}
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 38px;
  max-width: 320px;
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

/* secure 模式提示 */
.secure-tip {
  margin: 0 0 20px;
  padding: 10px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--accent-soft);
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 0.5px;
  color: var(--accent-deep);
}

/* 加载 / 空 / 失败状态 */
.state-line {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 72px 0;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.retry-btn {
  padding: 4px 12px;
  border-radius: var(--radius);
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
.retry-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}

/* ================= 细字表格 ================= */
.table-wrap {
  overflow-x: auto;
}
.user-table {
  width: 100%;
  min-width: 980px;
  border-collapse: collapse;
}
.user-table th {
  padding: 10px 14px;
  text-align: left;
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
  border-bottom: 1px solid var(--border);
}
.user-table td {
  padding: 12px 14px;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-2);
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.user-table tbody tr {
  transition: background-color 0.15s ease;
}
.user-table tbody tr:hover {
  background: var(--hover);
}
.col-user {
  width: 20%;
}
.col-perm {
  width: 28%;
}
.col-num {
  width: 7%;
}
.col-time {
  width: 14%;
}
.col-ops {
  width: 18%;
  white-space: nowrap;
}
.col-check {
  width: 40px;
  padding-right: 4px;
  text-align: center;
}
.row-check {
  width: 18px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  border: 1px solid var(--border-strong);
  background: none;
  color: transparent;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    color 0.2s ease;
  vertical-align: middle;
}
.row-check svg {
  width: 11px;
  height: 11px;
}
.row-check:hover:not(:disabled) {
  border-color: var(--accent);
}
.row-check.checked {
  border-color: var(--accent);
  background: var(--accent);
  color: var(--on-accent);
}
.row-check.partial {
  border-color: var(--accent);
  color: var(--accent);
}
.row-check:disabled {
  cursor: not-allowed;
  opacity: 0.35;
}
.uname {
  font-weight: 400;
  color: var(--text-1);
}
.self-tag {
  margin-left: 8px;
  padding: 1px 7px;
  border-radius: 999px;
  border: 1px solid var(--accent);
  color: var(--accent);
  font-size: 10.5px;
  font-weight: 400;
  letter-spacing: 1px;
}
.admin-tag {
  margin-left: 8px;
  padding: 1px 7px;
  border-radius: 999px;
  border: 1px solid #b7791f;
  background: rgba(183, 121, 31, 0.08);
  color: #9a6417;
  font-size: 10.5px;
  font-weight: 400;
  letter-spacing: 1px;
}

/* 权限开关组：极简圆角开关 + 细字标签 */
.perm-cell {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.perm-label {
  margin-right: 6px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* 极简圆角开关（与设置页一致） */
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
  vertical-align: middle;
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
.switch:hover:not(:disabled) {
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
.switch:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

/* 操作按钮：细字文本 */
.op-btn {
  margin-right: 10px;
  padding: 3px 0;
  border: none;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  font-weight: 400;
  cursor: pointer;
  transition: color 0.2s ease;
}
.op-btn:hover:not(:disabled) {
  color: var(--accent);
}
.op-btn.danger {
  color: #cf4444;
}
.op-btn.danger:hover:not(:disabled) {
  color: #b33535;
}
.op-btn:disabled {
  cursor: not-allowed;
  opacity: 0.35;
}

/* ================= 弹窗（与设置页同款极简） ================= */
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
  font-weight: 300;
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
.mono {
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
}
.field-row {
  display: flex;
  gap: 12px;
}
.field-row .field {
  flex: 1;
}
.field-tip {
  margin: -4px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-3);
}
.key-error {
  margin: -4px 0 0;
  font-size: 12px;
  font-weight: 400;
  color: #cf4444;
}
.perm-rows {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
}
.perm-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.perm-row-label {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
}
.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 6px;
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
  background: rgba(207, 68, 68, 0.08);
}
.ghost-btn:disabled,
.accent-btn:disabled,
.danger-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.confirm-text {
  margin: 0 0 18px;
  font-size: 13px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-2);
}
.confirm-list {
  margin-top: -10px;
  padding: 10px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  font-size: 12px;
  word-break: break-all;
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

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    flex-wrap: wrap;
    gap: 12px;
    padding: 12px 16px;
  }
  .content {
    padding: 32px 16px 56px;
  }
  .section-head {
    flex-wrap: wrap;
    align-items: center;
    row-gap: 10px;
  }
  .section-head .tool-btn,
  .section-head .add-btn,
  .section-head .refresh-btn {
    flex-shrink: 0;
  }
  .dlg-overlay {
    padding: 16px;
  }
}
</style>
