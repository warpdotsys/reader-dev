<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  deleteReplaceRule,
  deleteReplaceRules,
  getReplaceRules,
  saveReplaceRule,
  saveReplaceRules,
} from '@/api/replaceRules'
import { deleteTxtTocRule, getTxtTocRules, importDefaultTxtTocRules, saveTxtTocRule } from '@/api/txtTocRules'
import { useUserStore } from '@/stores/user'
import { checkTestRegex } from '@/utils/regexGuard'
import type { ReplaceRule, TxtTocRule } from '@/types'

const router = useRouter()
const store = useUserStore()

/* ================= 列表（localStorage: reader_replace_rules，见 api/replaceRules.ts 契约注释） ================= */
const rules = ref<ReplaceRule[]>([])
const loading = ref(true)

async function load() {
  loading.value = true
  try {
    const res = await getReplaceRules()
    rules.value = res.data ?? []
  } catch {
    rules.value = []
  } finally {
    loading.value = false
  }
}

const enabledCount = computed(() => rules.value.filter((r) => r.enabled).length)

function newId(): string {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 8)}`
}

/* ================= 新增 / 编辑弹窗 ================= */
const editorOpen = ref(false)
const editorBusy = ref(false)
const editingId = ref<string | null>(null)
const form = ref({ name: '', find: '', replace: '', enabled: true })

function openAdd() {
  editingId.value = null
  form.value = { name: '', find: '', replace: '', enabled: true }
  editorOpen.value = true
  document.body.style.overflow = 'hidden'
}

function openEdit(r: ReplaceRule) {
  editingId.value = r.id
  form.value = {
    name: r.name ?? '',
    find: r.find ?? '',
    replace: r.replace ?? '',
    enabled: r.enabled,
  }
  editorOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeEditor() {
  if (editorBusy.value) return
  editorOpen.value = false
  document.body.style.overflow = ''
}

async function confirmSave() {
  if (editorBusy.value) return
  const find = form.value.find.trim()
  if (!find) {
    ElMessage.warning('「查找」内容不能为空')
    return
  }
  editorBusy.value = true
  const editing = editingId.value
  const rule: ReplaceRule = {
    id: editing ?? newId(),
    name: form.value.name.trim() || find,
    find,
    replace: form.value.replace,
    enabled: form.value.enabled,
    order: editing ? (rules.value.find((r) => r.id === editing)?.order ?? 0) : rules.value.length,
  }
  try {
    // 当前为 localStorage 占位；后端就绪后走 POST /reader3/saveReplaceRule（见 api/replaceRules.ts）
    const res = await saveReplaceRule(rule)
    // P1-C2：后端生效 id 与本地不一致（归属冲突改插新 id）→ 同步本地条目 id，避免重复保存
    if (res.data && typeof res.data === 'object' && res.data.id && res.data.id !== rule.id) {
      rule.id = res.data.id
      editingId.value = res.data.id
    }
    if (editing) {
      const i = rules.value.findIndex((r) => r.id === editing)
      if (i >= 0) rules.value[i] = rule
    } else {
      rules.value.push(rule)
    }
    closeEditor()
  } finally {
    editorBusy.value = false
  }
}

/* ================= 启用开关 ================= */
const toggling = ref<Set<string>>(new Set())

async function toggleRule(r: ReplaceRule) {
  if (toggling.value.has(r.id)) return
  toggling.value.add(r.id)
  const prev = r.enabled
  r.enabled = !prev // 乐观更新
  try {
    const res = await saveReplaceRule({ ...r, enabled: !prev })
    // P1-C2：后端生效 id 同步（归属冲突改插新 id 时避免后续保存重复建规则）
    if (res.data && typeof res.data === 'object' && res.data.id && res.data.id !== r.id) {
      r.id = res.data.id
    }
  } catch {
    r.enabled = prev // 失败回滚
  } finally {
    toggling.value.delete(r.id)
  }
}

/* ================= 删除（极简确认弹窗；替换规则 / TXT 目录规则共用） ================= */
const deleting = ref<{ kind: 'replace' | 'txt'; id: string; name: string } | null>(null)
const deletingMany = ref<{ kind: 'replace' | 'txt'; ids: string[] } | null>(null)
const deleteBusy = ref(false)

function askDelete(kind: 'replace' | 'txt', r: { id: string; name: string }) {
  deleting.value = { kind, id: r.id, name: r.name }
  deletingMany.value = null
  document.body.style.overflow = 'hidden'
}

function askDeleteMany(kind: 'replace' | 'txt', ids: string[]) {
  if (!ids.length) return
  deleting.value = null
  deletingMany.value = { kind, ids }
  document.body.style.overflow = 'hidden'
}

async function confirmDelete() {
  const many = deletingMany.value
  if (many && !deleteBusy.value) {
    deleteBusy.value = true
    try {
      if (many.kind === 'replace') {
        await deleteReplaceRules(many.ids)
        const removed = new Set(many.ids)
        rules.value = rules.value.filter((x) => !removed.has(x.id))
        selectedIds.value = new Set()
      } else {
        for (const id of many.ids) await deleteTxtTocRule(id)
        const removed = new Set(many.ids)
        txtRules.value = txtRules.value.filter((x) => !removed.has(x.id))
      }
      closeDelete()
    } catch {
      // 已提示
    } finally {
      deleteBusy.value = false
    }
    return
  }
  const t = deleting.value
  if (!t || deleteBusy.value) return
  deleteBusy.value = true
  try {
    if (t.kind === 'replace') {
      // 当前为 localStorage 占位；后端就绪后走 POST /reader3/deleteReplaceRule（见 api/replaceRules.ts）
      await deleteReplaceRule(t.id)
      rules.value = rules.value.filter((x) => x.id !== t.id)
    } else {
      await deleteTxtTocRule(t.id)
      txtRules.value = txtRules.value.filter((x) => x.id !== t.id)
    }
    closeDelete()
  } catch {
    // 已提示
  } finally {
    deleteBusy.value = false
  }
}

function closeDelete() {
  deleting.value = null
  deletingMany.value = null
  document.body.style.overflow = ''
}

/* ================= 批量选择 + JSON 导入/导出（替换规则） ================= */
const selectedIds = ref<Set<string>>(new Set())

const allSelected = computed(
  () => rules.value.length > 0 && rules.value.every((r) => selectedIds.value.has(r.id)),
)

function toggleSelected(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function toggleAll() {
  selectedIds.value = allSelected.value ? new Set() : new Set(rules.value.map((r) => r.id))
}

function exportJson() {
  const blob = new Blob([JSON.stringify(rules.value, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'replace-rules.json'
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}

const jsonOpen = ref(false)
const jsonText = ref('')
const jsonMsg = ref('')
const jsonBusy = ref(false)

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
    jsonMsg.value = 'JSON 必须是规则对象数组'
    return
  }
  const list = (arr as Array<Record<string, unknown>>)
    .filter((x) => x && typeof x === 'object')
    .map((x, i) => ({
      id: typeof x.id === 'string' && x.id ? x.id : newId(),
      name: typeof x.name === 'string' && x.name ? x.name : String(x.find ?? `规则 ${i + 1}`),
      find: typeof x.find === 'string' ? x.find : '',
      replace: typeof x.replace === 'string' ? x.replace : '',
      enabled: typeof x.enabled === 'boolean' ? x.enabled : true,
      order: typeof x.order === 'number' ? x.order : rules.value.length + i,
    }))
    .filter((r) => r.find)
  if (!list.length) {
    jsonMsg.value = '未找到有效的规则（需要非空 find）'
    return
  }
  jsonBusy.value = true
  try {
    const res = await saveReplaceRules(list)
    jsonMsg.value = `已导入 ${res.data?.count ?? list.length} 条规则（服务端不可用时仅保存在本机）`
    await load()
  } finally {
    jsonBusy.value = false
  }
}

/* ================= 选项卡：替换规则 / TXT 目录规则 ================= */
const activeTab = ref<'replace' | 'txt'>('replace')

/* ================= 规则测试（GAP 4 相关：弹窗输入样本文本 → 本地应用 → 前后对比） ================= */
const testOpen = ref(false)
const testingRule = ref<ReplaceRule | null>(null)
const sampleText = ref('')
const regexMode = ref(false)
const testResult = ref<{ after: string; count: number; error: string } | null>(null)
const testRunBusy = ref(false)

function openTest(r: ReplaceRule) {
  testingRule.value = r
  sampleText.value = ''
  regexMode.value = false
  testResult.value = null
  testOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeTest() {
  if (testRunBusy.value) return
  testOpen.value = false
  document.body.style.overflow = ''
}

/** 本地应用单条规则：默认与阅读页渲染同引擎（字面 replaceAll）；可切正则模式（简化版规则引擎）
 * P1-5：测试正则加长度限制（200 字符）——浏览器主线程同步匹配无法超时中断，超长恶意模式直接拒绝 */
function runTest() {
  const r = testingRule.value
  if (!r || testRunBusy.value) return
  testRunBusy.value = true
  testResult.value = null
  try {
    const input = sampleText.value
    const find = r.find || ''
    const replace = r.replace ?? ''
    let after: string
    let count = 0
    if (regexMode.value) {
      const guard = checkTestRegex(find)
      if (guard) {
        testResult.value = { after: input, count: 0, error: guard }
        return
      }
      const re = new RegExp(find, 'g')
      after = input.replace(re, replace)
      count = (input.match(re) ?? []).length
    } else {
      const parts = input.split(find)
      count = parts.length - 1
      after = parts.join(replace)
    }
    testResult.value = { after, count, error: '' }
  } catch (e) {
    testResult.value = { after: sampleText.value, count: 0, error: e instanceof Error ? e.message : String(e) }
  } finally {
    testRunBusy.value = false
  }
}

/* ================= TXT 目录规则（GAP 4 相关：管理 + 测试匹配章节标题） ================= */
const txtRules = ref<TxtTocRule[]>([])
const txtLoading = ref(true)
const txtError = ref('')
const txtBusy = ref(false)

const txtEnabledCount = computed(() => txtRules.value.filter((r) => r.enable).length)

async function loadTxtRules() {
  txtLoading.value = true
  txtError.value = ''
  try {
    const res = await getTxtTocRules()
    txtRules.value = res.data ?? []
  } catch {
    txtError.value = '目录规则加载失败'
  } finally {
    txtLoading.value = false
  }
}

/** 内置默认规则（后端固定 id default-N）：不可停用 / 删除 */
function isDefaultTxtRule(r: TxtTocRule): boolean {
  return (r.id || '').startsWith('default-')
}

async function toggleTxtRule(r: TxtTocRule) {
  if (txtBusy.value || isDefaultTxtRule(r)) return
  txtBusy.value = true
  const prev = r.enable
  r.enable = !prev // 乐观更新
  try {
    await saveTxtTocRule({ ...r, enable: !prev })
  } catch {
    r.enable = prev // 失败回滚
  } finally {
    txtBusy.value = false
  }
}

async function importTxtDefaults() {
  if (txtBusy.value) return
  txtBusy.value = true
  try {
    await importDefaultTxtTocRules()
    await loadTxtRules()
  } catch {
    // 已提示
  } finally {
    txtBusy.value = false
  }
}

/* TXT 规则新增弹窗 */
const txtEditorOpen = ref(false)
const txtForm = ref({ name: '', rule: '', enable: true })

function openTxtAdd() {
  txtForm.value = { name: '', rule: '', enable: true }
  txtEditorOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeTxtEditor() {
  if (txtBusy.value) return
  txtEditorOpen.value = false
  document.body.style.overflow = ''
}

async function confirmTxtSave() {
  if (txtBusy.value) return
  const name = txtForm.value.name.trim()
  const rule = txtForm.value.rule.trim()
  if (!name) {
    ElMessage.warning('「名称」不能为空')
    return
  }
  if (!rule) {
    ElMessage.warning('「规则」不能为空')
    return
  }
  // P1-5：保存前同样做长度校验（与测试弹窗一致，避免保存后无法测试）
  const guard = checkTestRegex(rule)
  if (guard) {
    ElMessage.warning(guard)
    return
  }
  try {
    new RegExp(rule, 'gm')
  } catch {
    ElMessage.warning('正则表达式无效')
    return
  }
  txtBusy.value = true
  try {
    await saveTxtTocRule({
      id: '',
      name,
      rule,
      enable: txtForm.value.enable,
      serialNumber: txtRules.value.length,
    })
    closeTxtEditor()
    await loadTxtRules()
  } finally {
    txtBusy.value = false
  }
}

/* TXT 规则测试弹窗：输入样本文本 → 正则按行匹配（MULTILINE，与后端分章同语义）→ 显示匹配章节标题 */
const txtTestOpen = ref(false)
const txtTestingRule = ref<TxtTocRule | null>(null)
const txtSample = ref('')
const txtTestResult = ref<{ titles: string[]; error: string } | null>(null)

function openTxtTest(r: TxtTocRule) {
  txtTestingRule.value = r
  txtSample.value = ''
  txtTestResult.value = null
  txtTestOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeTxtTest() {
  txtTestOpen.value = false
  document.body.style.overflow = ''
}

function runTxtTest() {
  const r = txtTestingRule.value
  if (!r) return
  txtTestResult.value = null
  // P1-5：测试正则长度限制（同替换规则测试）
  const guard = checkTestRegex(r.rule)
  if (guard) {
    txtTestResult.value = { titles: [], error: guard }
    return
  }
  try {
    const re = new RegExp(r.rule, 'gm')
    const titles: string[] = []
    for (const m of txtSample.value.matchAll(re)) {
      const t = (m[0] || '').trim()
      if (t) titles.push(t)
    }
    txtTestResult.value = { titles, error: '' }
  } catch (e) {
    txtTestResult.value = { titles: [], error: e instanceof Error ? e.message : String(e) }
  }
}

onMounted(() => {
  load()
  loadTxtRules()
})
</script>

<template>
  <div class="rules-page">
    <!-- 极简导航：字标 + 页面入口 -->
    <header class="topbar">
      <div class="brand">
        <img class="brand-logo" src="/logo.svg" alt="夜读" />
        <span class="brand-name">夜读<span class="brand-dot">.</span></span>
      </div>

      <div class="user-area">
        <button class="nav-link" type="button" @click="router.push('/')">书架</button>
        <button class="nav-link" type="button" @click="router.push('/search')">搜索</button>
        <button class="nav-link" type="button" @click="router.push('/sources')">书源</button>
        <button class="nav-link active" type="button" @click="router.push('/rules')">替换规则</button>
        <button class="nav-link" type="button" @click="router.push('/settings')">设置</button>
        <button
          v-if="store.isAdmin"
          class="default-config-btn"
          :class="{ active: store.defaultConfigMode }"
          type="button"
          :aria-pressed="store.defaultConfigMode"
          :title="store.defaultConfigMode ? '退出系统配置模式，回到本人账号' : '进入系统配置模式（default）：编辑对所有用户生效的公用数据'"
          @click="store.toggleDefaultConfigMode()"
        >
          {{ store.defaultConfigMode ? '退出系统配置' : '系统配置' }}
        </button>
        <span class="user-chip">{{ store.username || '未登录' }}</span>
      </div>
    </header>

    <main class="content">
      <div class="section-head">
        <h1 class="section-title">{{ activeTab === 'replace' ? '替换规则' : 'TXT 目录规则' }}</h1>
        <span class="count">{{ activeTab === 'replace' ? rules.length + ' 条 · ' + enabledCount + ' 启用' : txtRules.length + ' 条 · ' + txtEnabledCount + ' 启用' }}</span>
        <template v-if="activeTab === 'replace'">
          <button class="op-btn" type="button" @click="openJsonImport">导入 JSON</button>
          <button class="op-btn" type="button" :disabled="!rules.length" @click="exportJson">导出 JSON</button>
        </template>
        <button class="add-btn" type="button" @click="activeTab === 'replace' ? openAdd() : openTxtAdd()">新增规则</button>
      </div>
      <p v-if="store.isAdmin && store.defaultConfigMode" class="default-mode-note">
        正在编辑系统配置（default）：规则对所有用户生效
      </p>

      <!-- 选项卡 -->
      <div class="tabs">
        <button class="tab" :class="{ active: activeTab === 'replace' }" type="button" @click="activeTab = 'replace'; selectedIds = new Set()">替换规则</button>
        <button class="tab" :class="{ active: activeTab === 'txt' }" type="button" @click="activeTab = 'txt'; selectedIds = new Set()">TXT 目录规则</button>
      </div>

      <!-- ================= 替换规则 ================= -->
      <template v-if="activeTab === 'replace'">
        <div v-if="selectedIds.size" class="bulk-bar">
          <span class="bulk-count">已选 {{ selectedIds.size }} 条规则</span>
          <button class="ghost-btn" type="button" @click="selectedIds = new Set()">取消选择</button>
          <button class="danger-btn" type="button" @click="askDeleteMany('replace', Array.from(selectedIds))">
            删除选中
          </button>
        </div>
        <!-- 加载态 -->
        <div v-if="loading" class="state-row">
          <p class="state-text">加载中…</p>
        </div>

        <!-- 空状态 -->
        <div v-else-if="rules.length === 0" class="state-row">
          <p class="state-text">暂无规则，点击右上角新增</p>
        </div>

        <!-- 规则列表（极简表格） -->
        <div v-else class="table-wrap">
          <table class="rule-table">
            <thead>
              <tr>
                <th class="th-check">
                  <input
                    class="row-check"
                    type="checkbox"
                    :checked="allSelected"
                    :aria-label="allSelected ? '取消全选' : '全选'"
                    @change="toggleAll"
                  />
                </th>
                <th class="th-name">名称</th>
                <th class="th-find">查找</th>
                <th class="th-replace">替换</th>
                <th class="th-enabled">启用</th>
                <th class="th-ops">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="r in rules" :key="r.id">
                <td class="td-check">
                  <input
                    class="row-check"
                    type="checkbox"
                    :checked="selectedIds.has(r.id)"
                    :aria-label="`选择规则 ${r.name}`"
                    @change="toggleSelected(r.id)"
                  />
                </td>
                <td class="td-name" :title="r.name">{{ r.name }}</td>
                <td class="td-find mono" :title="r.find">{{ r.find }}</td>
                <td class="td-replace mono" :title="r.replace">{{ r.replace || '—' }}</td>
                <td class="td-enabled">
                  <button
                    class="switch"
                    :class="{ on: r.enabled }"
                    type="button"
                    role="switch"
                    :aria-checked="r.enabled"
                    :title="r.enabled ? '停用' : '启用'"
                    @click="toggleRule(r)"
                  >
                    <span class="switch-knob"></span>
                  </button>
                </td>
                <td class="td-ops">
                  <button class="op-btn" type="button" @click="openTest(r)">测试</button>
                  <button class="op-btn" type="button" @click="openEdit(r)">编辑</button>
                  <button class="op-btn danger" type="button" @click="askDelete('replace', r)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>

      <!-- ================= TXT 目录规则 ================= -->
      <template v-else>
        <div class="sub-head">
          <span class="sub-tip">上传 TXT 本地书时，按启用的规则正则匹配行作为章节标题</span>
          <button class="op-btn" type="button" :disabled="txtBusy" @click="importTxtDefaults">导入默认规则</button>
        </div>

        <!-- 加载态 -->
        <div v-if="txtLoading" class="state-row">
          <p class="state-text">加载中…</p>
        </div>
        <p v-else-if="txtError" class="state-error-line">{{ txtError }} <button class="retry-btn" type="button" @click="loadTxtRules">重试</button></p>

        <!-- 空状态 -->
        <div v-else-if="txtRules.length === 0" class="state-row">
          <p class="state-text">暂无目录规则，可导入内置默认规则或新增</p>
        </div>

        <!-- 规则列表 -->
        <div v-else class="table-wrap">
          <table class="rule-table">
            <thead>
              <tr>
                <th class="th-name">名称</th>
                <th class="th-find">规则（正则）</th>
                <th class="th-enabled">启用</th>
                <th class="th-ops">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="r in txtRules" :key="r.id">
                <td class="td-name" :title="r.name">{{ r.name }}</td>
                <td class="td-find mono" :title="r.rule">{{ r.rule }}</td>
                <td class="td-enabled">
                  <button
                    class="switch"
                    :class="{ on: r.enable }"
                    type="button"
                    role="switch"
                    :aria-checked="r.enable"
                    :disabled="isDefaultTxtRule(r)"
                    :title="isDefaultTxtRule(r) ? '内置默认规则' : (r.enable ? '停用' : '启用')"
                    @click="toggleTxtRule(r)"
                  >
                    <span class="switch-knob"></span>
                  </button>
                </td>
                <td class="td-ops">
                  <button class="op-btn" type="button" @click="openTxtTest(r)">测试</button>
                  <button v-if="!isDefaultTxtRule(r)" class="op-btn danger" type="button" @click="askDelete('txt', r)">删除</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>

      <p class="foot-tip">替换规则已同步到服务端（登录账号内多设备一致）；服务不可用时自动降级为本地浏览器存储。TXT 目录规则存储于服务端，上传 TXT 本地书时分章使用。</p>
    </main>

    <!-- 新增 / 编辑规则弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="editorOpen" class="dlg-overlay" @click.self="closeEditor">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="编辑替换规则" tabindex="-1" @keydown.esc="closeEditor">
            <div class="dlg-head">
              <h2 class="dlg-title">{{ editingId ? '编辑规则' : '新增规则' }}</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="editorBusy" @click="closeEditor">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmSave">
              <label class="field">
                <span class="field-label">名称</span>
                <input v-model="form.name" class="field-input" type="text" placeholder="留空则使用「查找」内容" maxlength="40" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">查找<em>*</em></span>
                <input v-model="form.find" class="field-input" type="text" placeholder="要被替换的文字（必填）" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">替换</span>
                <input v-model="form.replace" class="field-input" type="text" placeholder="替换为（可留空 = 删除匹配文字）" spellcheck="false" />
              </label>
              <div class="field">
                <span class="field-label">启用</span>
                <button
                  class="switch"
                  :class="{ on: form.enabled }"
                  type="button"
                  role="switch"
                  :aria-checked="form.enabled"
                  @click="form.enabled = !form.enabled"
                >
                  <span class="switch-knob"></span>
                </button>
              </div>
              <p class="field-tip">正文渲染时按顺序逐条 replaceAll（全文匹配，非正则）</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="editorBusy" @click="closeEditor">取消</button>
                <button class="accent-btn" type="submit" :disabled="editorBusy || !form.find.trim()">
                  {{ editorBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 删除确认弹窗（极简） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="deleting || deletingMany" class="dlg-overlay" @click.self="closeDelete">
          <div
            class="dlg dlg-confirm"
            role="alertdialog"
            aria-modal="true"
            aria-label="删除确认"
            tabindex="-1"
            @keydown.esc="closeDelete"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">
                删除{{ (deletingMany ?? deleting)?.kind === 'txt' ? '目录规则' : '规则' }}
              </h2>
            </div>
            <p class="confirm-text">
              <template v-if="deletingMany">
                确定删除选中的 {{ deletingMany.ids.length }} 条规则吗？此操作不可恢复。
              </template>
              <template v-else>确定删除「{{ deleting?.name }}」吗？此操作不可恢复。</template>
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

    <!-- JSON 导入弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="jsonOpen" class="dlg-overlay" @click.self="closeJsonImport">
          <div
            class="dlg dlg-test"
            role="dialog"
            aria-modal="true"
            aria-label="导入替换规则 JSON"
            tabindex="-1"
            @keydown.esc="closeJsonImport"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">导入替换规则 JSON</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="jsonBusy" @click="closeJsonImport">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <label class="field">
              <span class="field-label">规则数组</span>
              <textarea
                v-model="jsonText"
                class="field-textarea"
                rows="12"
                placeholder='[{"name":"示例","find":"旧文本","replace":"新文本","enabled":true}]'
                spellcheck="false"
              ></textarea>
            </label>
            <p class="field-tip">支持 id/name/find/replace/enabled/order；id 缺失时自动生成。</p>
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

    <!-- 替换规则测试弹窗：输入样本文本 → 本地应用 → 前后对比 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="testOpen" class="dlg-overlay" @click.self="closeTest">
          <div class="dlg dlg-test" role="dialog" aria-modal="true" aria-label="测试替换规则" tabindex="-1" @keydown.esc="closeTest">
            <div class="dlg-head">
              <h2 class="dlg-title">测试「{{ testingRule?.name }}」</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="testRunBusy" @click="closeTest">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <p class="test-rule-info mono">查找：{{ testingRule?.find }}</p>
            <p class="test-rule-info mono">替换：{{ testingRule?.replace || '（删除匹配）' }}</p>
            <label class="field">
              <span class="field-label">样本文本</span>
              <textarea v-model="sampleText" class="field-textarea" rows="5" placeholder="粘贴正文片段…" spellcheck="false"></textarea>
            </label>
            <div class="test-opts">
              <label class="check">
                <input v-model="regexMode" type="checkbox" />
                <span>按正则匹配（默认字面替换，与阅读页一致）</span>
              </label>
              <button class="accent-btn" type="button" :disabled="testRunBusy || !sampleText" @click="runTest">
                {{ testRunBusy ? '应用中…' : '应用规则' }}
              </button>
            </div>
            <div v-if="testResult" class="test-result">
              <p v-if="testResult.error" class="test-error">规则无效：{{ testResult.error }}</p>
              <template v-else>
                <p class="test-count">共替换 {{ testResult.count }} 处</p>
                <div class="cmp">
                  <span class="cmp-label">替换前</span>
                  <pre class="cmp-box before">{{ sampleText }}</pre>
                </div>
                <div class="cmp">
                  <span class="cmp-label">替换后</span>
                  <pre class="cmp-box after">{{ testResult.after }}</pre>
                </div>
              </template>
            </div>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" @click="closeTest">关闭</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- TXT 目录规则测试弹窗：输入样本文本 → 匹配章节标题 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="txtTestOpen" class="dlg-overlay" @click.self="closeTxtTest">
          <div class="dlg dlg-test" role="dialog" aria-modal="true" aria-label="测试 TXT 目录规则" tabindex="-1" @keydown.esc="closeTxtTest">
            <div class="dlg-head">
              <h2 class="dlg-title">测试「{{ txtTestingRule?.name }}」</h2>
              <button class="dlg-close" type="button" title="关闭" @click="closeTxtTest">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <p class="test-rule-info mono">{{ txtTestingRule?.rule }}</p>
            <label class="field">
              <span class="field-label">样本文本</span>
              <textarea v-model="txtSample" class="field-textarea" rows="6" placeholder="粘贴 TXT 正文片段（按行匹配章节标题）…" spellcheck="false"></textarea>
            </label>
            <div class="test-opts">
              <span class="sub-tip">规则按行匹配（MULTILINE），匹配的整行作为章节标题</span>
              <button class="accent-btn" type="button" :disabled="!txtSample" @click="runTxtTest">匹配章节</button>
            </div>
            <div v-if="txtTestResult" class="test-result">
              <p v-if="txtTestResult.error" class="test-error">规则无效：{{ txtTestResult.error }}</p>
              <template v-else>
                <p class="test-count">匹配到 {{ txtTestResult.titles.length }} 个章节标题</p>
                <ul v-if="txtTestResult.titles.length" class="toc-matches">
                  <li v-for="(t, i) in txtTestResult.titles" :key="i" class="toc-match mono">{{ t }}</li>
                </ul>
                <p v-else class="state-text">未匹配到章节标题</p>
              </template>
            </div>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" @click="closeTxtTest">关闭</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 新增 TXT 目录规则弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="txtEditorOpen" class="dlg-overlay" @click.self="closeTxtEditor">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="新增 TXT 目录规则" tabindex="-1" @keydown.esc="closeTxtEditor">
            <div class="dlg-head">
              <h2 class="dlg-title">新增 TXT 目录规则</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="txtBusy" @click="closeTxtEditor">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmTxtSave">
              <label class="field">
                <span class="field-label">名称<em>*</em></span>
                <input v-model="txtForm.name" class="field-input" type="text" placeholder="如：第X章" maxlength="40" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">规则（正则）<em>*</em></span>
                <textarea
                  v-model="txtForm.rule"
                  class="field-textarea"
                  rows="3"
                  placeholder="如：^\s*第\s*[0-9一二三四五六七八九十百千万零〇两]+\s*[章节卷回集部篇]"
                  spellcheck="false"
                ></textarea>
              </label>
              <div class="field">
                <span class="field-label">启用</span>
                <button
                  class="switch"
                  :class="{ on: txtForm.enable }"
                  type="button"
                  role="switch"
                  :aria-checked="txtForm.enable"
                  @click="txtForm.enable = !txtForm.enable"
                >
                  <span class="switch-knob"></span>
                </button>
              </div>
              <p class="field-tip">正则按行匹配（MULTILINE），匹配到的整行作为章节标题；上传 TXT 本地书时按启用的规则分章</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="txtBusy" @click="closeTxtEditor">取消</button>
                <button class="accent-btn" type="submit" :disabled="txtBusy || !txtForm.name.trim() || !txtForm.rule.trim()">
                  {{ txtBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.rules-page {
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
.default-config-btn {
  flex-shrink: 0;
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
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.default-config-btn:hover:not(:disabled) {
  color: var(--text-1);
  border-color: var(--border-strong);
}
.default-config-btn.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* ================= 内容区 ================= */
.content {
  width: min(860px, 100%);
  margin: 0 auto;
  padding: 44px 32px 72px;
}
.section-head {
  display: flex;
  align-items: baseline;
  gap: 14px;
  margin-bottom: 26px;
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
.add-btn {
  margin-left: auto;
  padding: 7px 18px;
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
.add-btn:hover {
  color: var(--accent-deep);
  border-color: var(--accent-deep);
  background: var(--accent-soft);
}

/* ================= 状态行 ================= */
.state-row {
  padding: 72px 0;
  text-align: center;
}
.state-text {
  margin: 0;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}

/* ================= 极简表格 ================= */
.table-wrap {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  background: var(--surface);
}
.rule-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.rule-table th {
  padding: 12px 16px;
  text-align: left;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-3);
  border-bottom: 1px solid var(--border);
  background: var(--bg);
}
.rule-table td {
  padding: 13px 16px;
  border-bottom: 1px solid var(--border);
  color: var(--text-2);
  vertical-align: middle;
}
.rule-table tr:last-child td {
  border-bottom: none;
}
.rule-table tbody tr {
  transition: background-color 0.15s ease;
}
.rule-table tbody tr:hover {
  background: var(--hover);
}
.th-name {
  width: 26%;
}
.th-enabled,
.th-ops {
  width: 90px;
}
.td-name {
  color: var(--text-1);
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 0;
}
.td-find,
.td-replace {
  max-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.mono {
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
}
.td-enabled,
.td-ops {
  text-align: center;
}

/* 极简开关 */
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

/* 操作按钮 */
.op-btn {
  padding: 3px 10px;
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
.op-btn + .op-btn {
  margin-left: 6px;
}
.op-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.op-btn.danger:hover {
  color: #cf4444;
  border-color: #cf4444;
}

.foot-tip {
  margin: 18px 2px 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= 批量选择 / JSON 导入 ================= */
.bulk-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
  padding: 8px 12px;
  border: 1px solid var(--accent);
  border-radius: var(--radius);
  background: var(--accent-soft);
}
.bulk-count {
  flex: 1;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--accent-deep);
}
.th-check,
.td-check {
  width: 40px;
  text-align: center;
}
.row-check {
  accent-color: var(--accent);
  cursor: pointer;
}
.json-msg {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--accent-deep);
}

/* ================= 选项卡 ================= */
.tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 20px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
}
.tab {
  padding: 6px 18px;
  border: 1px solid transparent;
  border-radius: 999px;
  background: none;
  color: var(--text-3);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.tab:hover {
  color: var(--accent);
}
.tab.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* ================= TXT 规则 ================= */
.sub-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}
.sub-tip {
  flex: 1;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.state-error-line {
  padding: 40px 0;
  text-align: center;
  font-size: 13px;
  color: #cf4444;
}
.retry-btn {
  margin-left: 8px;
  padding: 4px 12px;
  font-size: 12px;
  color: var(--accent);
  background: none;
  border: 1px solid var(--accent);
  border-radius: 999px;
  cursor: pointer;
}
.switch:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

/* ================= 测试弹窗 ================= */
.dlg-test {
  width: min(560px, 100%);
}
.test-rule-info {
  margin: 0 0 6px;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  word-break: break-all;
}
.field-textarea {
  width: 100%;
  min-height: 96px;
  padding: 10px 12px;
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
.test-opts {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 4px;
  flex-wrap: wrap;
}
.check {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
  cursor: pointer;
  user-select: none;
}
.check input {
  accent-color: var(--accent);
}
.test-result {
  margin-top: 14px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg);
}
.test-count {
  margin: 0 0 8px;
  font-size: 12px;
  font-weight: 400;
  color: var(--accent);
}
.test-error {
  margin: 0;
  font-size: 12.5px;
  color: #cf4444;
  word-break: break-all;
}
.cmp + .cmp {
  margin-top: 10px;
}
.cmp-label {
  display: block;
  margin-bottom: 4px;
  font-size: 11.5px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.cmp-box {
  margin: 0;
  max-height: 160px;
  overflow: auto;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--surface);
  border: 1px solid var(--border);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.7;
  color: var(--text-1);
  white-space: pre-wrap;
  word-break: break-all;
}
.cmp-box.after {
  border-color: var(--accent);
}
.toc-matches {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.toc-match {
  padding: 6px 10px;
  border-radius: 6px;
  background: var(--surface);
  border: 1px solid var(--border);
  font-size: 12px;
  color: var(--text-1);
  word-break: break-all;
}

/* ================= 弹窗 ================= */
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
.field-tip {
  margin: -4px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
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
.ghost-btn:disabled,
.accent-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
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
    flex-wrap: wrap;
    gap: 12px;
    padding: 12px 16px;
  }
  .content {
    padding: 32px 16px 56px;
  }
  .rule-table th,
  .rule-table td {
    padding: 10px 12px;
    min-height: 40px;
  }
  .th-replace {
    display: none;
  }
  .td-replace {
    display: none;
  }
  .dlg-overlay {
    padding: 16px;
  }
}
</style>
