<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import TopNav from '@/components/TopNav.vue'
import {
  deleteHttpTts,
  deleteHttpTtsMany,
  getHttpTtsList,
  parseHttpTtsJson,
  saveHttpTts,
  saveHttpTtsMulti,
} from '@/api/httpTts'
import { uploadFile, mkdir } from '@/api/file'
import { loadCustomCss, saveCustomCss, applyCustomCss } from '@/utils/customCss'
import { imageProxyEnabled, setImageProxyEnabled } from '@/utils/imageProxy'
import {
  loadBgMode,
  loadBgImagePath,
  loadBgPreset,
  saveBgMode,
  saveBgImagePath,
  saveBgPreset,
  bgPresetUrl,
  BG_PRESETS,
  bgImageUrl as bgImageUrlOf,
  type BgMode,
} from '@/utils/readerBg'
import { clearCache, getCacheInfo } from '@/api/cache'
import { backupToWebdav, downloadBackupZip } from '@/api/backup'
import { getSystemInfo } from '@/api/system'
import { deleteTxtTocRule, getTxtTocRules, importDefaultTxtTocRules, saveTxtTocRule } from '@/api/txtTocRules'
import { getBookshelf } from '@/api/bookshelf'
import { login as loginApi } from '@/api/auth'
import { resetUserPassword } from '@/api/users'
import { getUserConfig, saveUserConfig } from '@/api/userConfig'
import { getReadingStats } from '@/api/stats'
import { getOpdsSettings, saveOpdsSettings } from '@/api/opds'
import { setGlobalHanMode, syncHanMode } from '@/utils/hanMode'
import {
  loadReaderConfig,
  applyReaderConfig,
  toServerConfig,
  fromServerConfig,
  READER_CONFIG_DEFAULTS,
  type ReaderConfig,
  type HanMode,
  type Theme,
  type TextAlign,
  type PageMode,
} from '@/utils/readerConfig'
import { applyUiTheme, loadUiTheme, uiThemeFromServer, uiThemeToServer, type UiTheme } from '@/utils/uiTheme'
import { isNotImplemented } from '@/utils/errors'
import { DAILY_STATS_KEY, last7Days, parseDailyStats } from '@/utils/dailyStats'
import { useUserStore } from '@/stores/user'
import { downloadBlob } from '@/utils/download'
import type { CacheClearType, CacheInfo, HttpTts, SystemInfo, TxtTocRule } from '@/types'

const router = useRouter()
const store = useUserStore()

/** 版本号与后端 Cargo.toml 保持一致（getSystemInfo 不可用时兜底显示） */
const VERSION = '5.2.4'

/** 系统信息（/reader3/getSystemInfo，设置页「关于」区展示） */
const sysInfo = ref<SystemInfo | null>(null)

async function loadSysInfo() {
  try {
    const res = await getSystemInfo()
    sysInfo.value = res.data ?? null
  } catch {
    sysInfo.value = null // 后端不可用时静默（版本仍显示前端常量）
  }
}

const showToken = ref(false)

function maskToken(t: string): string {
  if (!t) return '未登录'
  if (t.length <= 12) return `${t.slice(0, 4)}…`
  return `${t.slice(0, 8)}…${t.slice(-4)}`
}

async function logout() {
  try {
    await ElMessageBox.confirm('确定退出登录吗？', '退出登录', {
      confirmButtonText: '退出',
      cancelButtonText: '取消',
      type: 'warning',
    })
  } catch {
    return // 用户取消
  }
  store.clear() // 清空 localStorage（reader_access_token / reader_username）
  ElMessage.success('已退出登录')
  void router.replace('/login')
}

/* ================= GAP 87：修改密码（旧密码校验 → POST /reader3/resetUserPassword → 强制重新登录） ================= */

const pwdOpen = ref(false)
const pwdBusy = ref(false)
const pwdForm = ref({ oldPassword: '', newPassword: '', confirmPassword: '' })
const pwdMsg = ref('')
const pwdMsgError = ref(false)

function openPwd() {
  pwdForm.value = { oldPassword: '', newPassword: '', confirmPassword: '' }
  pwdMsg.value = ''
  pwdMsgError.value = false
  pwdOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closePwd() {
  if (pwdBusy.value) return
  pwdOpen.value = false
  document.body.style.overflow = ''
}

async function submitPwd() {
  if (pwdBusy.value) return
  const { oldPassword, newPassword, confirmPassword } = pwdForm.value
  if (!store.username) {
    pwdMsg.value = '未登录，无法修改密码'
    pwdMsgError.value = true
    return
  }
  if (!oldPassword || !newPassword) {
    pwdMsg.value = '请填写旧密码与新密码'
    pwdMsgError.value = true
    return
  }
  if (newPassword.length < 8) {
    pwdMsg.value = '新密码不能低于 8 位'
    pwdMsgError.value = true
    return
  }
  if (newPassword !== confirmPassword) {
    pwdMsg.value = '两次输入的新密码不一致'
    pwdMsgError.value = true
    return
  }
  pwdBusy.value = true
  pwdMsg.value = ''
  pwdMsgError.value = false
  try {
    // ① 旧密码校验：调登录接口（密码错则后端拒绝；登录会刷新 token，成功后同步本地会话）
    const loginRes = await loginApi({
      username: store.username,
      password: oldPassword,
      isLogin: true,
    })
    store.setSession(loginRes.data.accessToken, loginRes.data.username, true, loginRes.data.isAdmin === true)
    // ② 重置密码（后端重置后旧 token 失效）
    await resetUserPassword(store.username, newPassword)
    // ③ 强制重新登录
    ElMessage.success('密码已修改，请重新登录')
    store.clear()
    void router.replace('/login')
  } catch (err) {
    if ((err as { data?: unknown } | null)?.data === 'NEED_SECURE_KEY') {
      pwdMsg.value = '当前为 secure 模式，后端 resetUserPassword 需管理密码——个人改密接口后端待实现'
    } else if (err instanceof Error && err.message.includes('密码错误')) {
      pwdMsg.value = '旧密码错误'
    } else {
      pwdMsg.value = `修改失败：${err instanceof Error ? err.message : '请稍后重试'}`
    }
    pwdMsgError.value = true
  } finally {
    pwdBusy.value = false
  }
}
/* ================= 听书设置（HttpTTS，localStorage 占位，见 api/httpTts.ts 契约注释） ================= */

const TTS_TYPE_LABEL: Record<number, string> = { 0: '在线合成', 1: '本地引擎' }
const ttsList = ref<HttpTts[]>([])

async function loadTtsList() {
  try {
    const res = await getHttpTtsList()
    ttsList.value = res.data ?? []
  } catch {
    ttsList.value = []
  }
}

function ttsTypeLabel(t: number): string {
  return TTS_TYPE_LABEL[t] ?? `类型 ${t}`
}

function newTtsId(): string {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 8)}`
}

/* 新增弹窗 */
const ttsDialogOpen = ref(false)
const ttsBusy = ref(false)
const ttsForm = ref<{ name: string; url: string; type: number }>({ name: '', url: '', type: 0 })
/** 听书源多选集合（id/url） */
const ttsSelected = ref<Set<string>>(new Set())
/** 听书源编辑弹窗（完整字段 JSON 编辑） */
const ttsEditing = ref<HttpTts | null>(null)
/** 听书源 JSON 导入文件输入 */
const ttsImportRef = ref<HTMLInputElement | null>(null)

function openAddTts() {
  ttsForm.value = { name: '', url: '', type: 0 }
  ttsDialogOpen.value = true
  document.body.style.overflow = 'hidden'
}

function openEditTts(t: HttpTts) {
  ttsEditing.value = { ...t }
  document.body.style.overflow = 'hidden'
}

function closeEditTts() {
  if (ttsSaving.value) return
  ttsEditing.value = null
  document.body.style.overflow = ''
}

const ttsSaving = ref(false)

async function confirmEditTts() {
  const t = ttsEditing.value
  if (!t || ttsSaving.value) return
  if (!t.url.trim()) {
    ElMessage.warning('URL 不能为空')
    return
  }
  ttsSaving.value = true
  try {
    await saveHttpTts({
      ...t,
      name: t.name.trim() || t.url,
    })
    await loadTtsList()
    closeEditTts()
  } catch {
    // 已提示
  } finally {
    ttsSaving.value = false
  }
}

function closeAddTts() {
  if (ttsBusy.value) return
  ttsDialogOpen.value = false
  document.body.style.overflow = ''
}

async function confirmAddTts() {
  if (ttsBusy.value) return
  const url = ttsForm.value.url.trim()
  if (!url) {
    ElMessage.warning('URL 不能为空')
    return
  }
  ttsBusy.value = true
  try {
    // 当前为 localStorage 占位；后端就绪后走 POST /reader3/saveHttpTTS（见 api/httpTts.ts）
    await saveHttpTts({
      id: newTtsId(),
      name: ttsForm.value.name.trim() || url,
      url,
      type: ttsForm.value.type,
    })
    await loadTtsList()
    closeAddTts()
  } finally {
    ttsBusy.value = false
  }
}

/* 删除 */
const deletingTts = ref<HttpTts | null>(null)
const deleteTtsBusy = ref(false)

function askDeleteTts(t: HttpTts) {
  deletingTts.value = t
  document.body.style.overflow = 'hidden'
}

async function confirmDeleteTts() {
  const t = deletingTts.value
  if (!t || deleteTtsBusy.value) return
  deleteTtsBusy.value = true
  try {
    // 当前为 localStorage 占位；后端就绪后走 POST /reader3/deleteHttpTTS（见 api/httpTts.ts）
    await deleteHttpTts(t.id)
    ttsList.value = ttsList.value.filter((x) => x.id !== t.id)
    closeDeleteTts()
  } catch {
    // 已提示
  } finally {
    deleteTtsBusy.value = false
  }
}

function closeDeleteTts() {
  deletingTts.value = null
  document.body.style.overflow = ''
}

function toggleTtsSelect(id: string) {
  const next = new Set(ttsSelected.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  ttsSelected.value = next
}

async function removeSelectedTts() {
  if (ttsSelected.value.size === 0 || ttsBusy.value) return
  ttsBusy.value = true
  try {
    const ids = [...ttsSelected.value]
    const res = await deleteHttpTtsMany(ids)
    ElMessage.success(`已删除 ${res.data?.count ?? ids.length} 个听书源`)
    ttsSelected.value = new Set()
    await loadTtsList()
  } catch {
    // 已提示
  } finally {
    ttsBusy.value = false
  }
}

async function importTtsFile(file: File) {
  const text = await file.text()
  let parsed: HttpTts[]
  try {
    parsed = parseHttpTtsJson(text)
  } catch {
    ElMessage.error('听书源 JSON 解析失败')
    return
  }
  if (parsed.length === 0) {
    ElMessage.warning('未找到有效听书源数据')
    return
  }
  const res = await saveHttpTtsMulti(parsed)
  ElMessage.success(`已导入 ${res.data?.count ?? parsed.length} 个听书源`)
  await loadTtsList()
}

function onTtsImportChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) void importTtsFile(file)
  input.value = ''
}

function exportTtsJson() {
  if (ttsList.value.length === 0) {
    ElMessage.info('暂无听书源可导出')
    return
  }
  const blob = new Blob([JSON.stringify(ttsList.value, null, 2)], {
    type: 'application/json',
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `http-tts-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)
}

/* ================= txtTocRule（自定义 TXT 目录规则，后端 /reader3/getTxtTocRules 等） ================= */
const tocRules = ref<TxtTocRule[]>([])
const tocLoading = ref(true)

async function loadTxtTocRules() {
  tocLoading.value = true
  try {
    const res = await getTxtTocRules()
    tocRules.value = res.data ?? []
  } catch {
    tocRules.value = []
  } finally {
    tocLoading.value = false
  }
}

const customTocRules = computed(() => tocRules.value.filter((r) => !r.id.startsWith('default-')))

function newTocId(): string {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 8)}`
}

/* 新增弹窗 */
const tocDialogOpen = ref(false)
const tocBusy = ref(false)
const tocForm = ref<{ name: string; rule: string; enable: boolean }>({ name: '', rule: '', enable: true })

function openAddToc() {
  tocForm.value = { name: '', rule: '', enable: true }
  tocDialogOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeAddToc() {
  if (tocBusy.value) return
  tocDialogOpen.value = false
  document.body.style.overflow = ''
}

async function confirmAddToc() {
  if (tocBusy.value) return
  const rule = tocForm.value.rule.trim()
  if (!rule) {
    ElMessage.warning('规则正则不能为空')
    return
  }
  tocBusy.value = true
  try {
    await saveTxtTocRule({
      id: newTocId(),
      name: tocForm.value.name.trim() || rule,
      rule,
      enable: tocForm.value.enable,
      serialNumber: customTocRules.value.length,
    })
    await loadTxtTocRules()
    closeAddToc()
  } finally {
    tocBusy.value = false
  }
}

/* 启用开关（默认规则只读，仅自定义规则可切换） */
const tocToggling = ref<Set<string>>(new Set())

async function toggleTocRule(r: TxtTocRule) {
  if (tocToggling.value.has(r.id) || r.id.startsWith('default-')) return
  tocToggling.value.add(r.id)
  const prev = r.enable
  r.enable = !prev
  try {
    await saveTxtTocRule({ ...r, enable: !prev })
  } catch {
    r.enable = prev
  } finally {
    tocToggling.value.delete(r.id)
  }
}

/* 删除（仅自定义规则） */
const deletingToc = ref<TxtTocRule | null>(null)
const deleteTocBusy = ref(false)

function askDeleteToc(r: TxtTocRule) {
  if (r.id.startsWith('default-')) return
  deletingToc.value = r
  document.body.style.overflow = 'hidden'
}

async function confirmDeleteToc() {
  const r = deletingToc.value
  if (!r || deleteTocBusy.value) return
  deleteTocBusy.value = true
  try {
    await deleteTxtTocRule(r.id)
    tocRules.value = tocRules.value.filter((x) => x.id !== r.id)
    closeDeleteToc()
  } catch {
    // 已提示
  } finally {
    deleteTocBusy.value = false
  }
}

function closeDeleteToc() {
  deletingToc.value = null
  document.body.style.overflow = ''
}

/* 导入默认规则 */
const tocImportBusy = ref(false)

async function runImportDefaultToc() {
  if (tocImportBusy.value) return
  tocImportBusy.value = true
  try {
    const res = await importDefaultTxtTocRules()
    ElMessage.success(`已导入 ${res.data?.count ?? 0} 条默认规则`)
    await loadTxtTocRules()
  } catch {
    // 已提示
  } finally {
    tocImportBusy.value = false
  }
}

onMounted(() => {
  loadTtsList()
  loadSysInfo()
  loadTxtTocRules()
  loadCacheInfo()
  loadServerPref()
  loadOpdsCfg()
})

/* ================= 阅读偏好（多端同步：GET/POST /reader3/getUserConfig|saveUserConfig，服务器优先） ================= */

const pref = ref<ReaderConfig>(loadReaderConfig())
const prefSaving = ref(false)
const prefMsg = ref('')
const prefMsgError = ref(false)

/** 简繁模式改动 → 全站响应（搜索/目录/书源/书海等共用 hanMode 共享状态，见 utils/hanMode.ts） */
watch(
  () => pref.value.hanMode,
  (m) => setGlobalHanMode(m),
)

const HAN_OPTIONS: { value: HanMode; label: string }[] = [
  { value: 'auto', label: '自动' },
  { value: 'simp', label: '简体' },
  { value: 'trad', label: '繁体' },
]
const THEME_OPTIONS: { value: Theme; label: string }[] = [
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
  { value: 'warm', label: '暖色' },
  { value: 'system', label: '跟随系统' },
]
const WIDTH_OPTIONS: { value: string; label: string }[] = [
  { value: '720px', label: '窄' },
  { value: '900px', label: '适中' },
  { value: '1080px', label: '宽' },
]
const FONT_OPTIONS: { value: string; label: string }[] = [
  { value: 'system', label: '系统' },
  { value: 'song', label: '宋体' },
  { value: 'hei', label: '黑体' },
  { value: 'kai', label: '楷体' },
  { value: 'fangsong', label: '仿宋' },
  { value: 'round', label: '圆体' },
  { value: 'lishu', label: '隶书' },
  { value: 'yahei', label: '雅黑' },
  { value: 'pingfang', label: '苹方' },
  { value: 'wenkai', label: '文楷' },
  { value: 'hanserif', label: '思源宋' },
  { value: 'serif', label: '衬线' },
]
const ALIGN_OPTIONS: { value: TextAlign; label: string }[] = [
  { value: 'left', label: '左对齐' },
  { value: 'justify', label: '两端对齐' },
]
const PAGE_MODE_OPTIONS: { value: PageMode; label: string }[] = [
  { value: 'scroll', label: '滚动' },
  { value: 'slide', label: '上下翻页' },
  { value: 'hslide', label: '左右翻章' },
  { value: 'flip', label: '仿真翻页' },
]

/** 界面主题（外观卡片）：浅色 / 深色 / 跟随系统——与阅读内容主题分离，切换即时生效并落 localStorage */
const UI_THEME_OPTIONS: { value: UiTheme; label: string }[] = [
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
  { value: 'system', label: '跟随系统' },
]
const uiTheme = ref<UiTheme>(loadUiTheme())

/** 界面主题即时预览：切换即应用到全局（html[data-theme]），保存时随 userConfig 上传 */
watch(uiTheme, (t) => {
  applyUiTheme(t)
})

/** 进入设置页：拉取服务器配置并与本地合并（服务器优先），应用后回写本地 */
async function loadServerPref() {
  prefMsg.value = ''
  prefMsgError.value = false
  try {
    const res = await getUserConfig()
    // 后端返回 { ns, config }（config 为配置 JSON 或 null）
    const cfg = res.data && typeof res.data === 'object' ? (res.data as Record<string, unknown>).config : undefined
    const server = fromServerConfig(cfg)
    const merged = { ...loadReaderConfig(), ...server }
    applyReaderConfig(merged)
    pref.value = merged
    // 服务器下发的简繁模式同步到全站共享状态
    syncHanMode()
    // 界面主题（ui_theme 键）同样服务器优先
    const ui = uiThemeFromServer(cfg ? (cfg as Record<string, unknown>).ui_theme : undefined)
    if (ui) {
      uiTheme.value = ui
      applyUiTheme(ui)
    }
    prefMsg.value = '已从服务器同步阅读偏好（服务器优先）'
  } catch (err) {
    prefMsg.value = isNotImplemented(err)
      ? '配置同步接口后端暂未提供（GET /reader3/getUserConfig）· 当前仅保存在本机'
      : `同步失败：${err instanceof Error ? err.message : '请稍后重试'}`
    prefMsgError.value = true
  }
}

/** 保存阅读偏好：本地立即生效（阅读页读取 localStorage）+ POST /reader3/saveUserConfig
 *  P2：先读云端现有配置再合并更新已知字段（整量覆盖会删除云端未知字段——多端/未来版本共存字段） */
async function savePref() {
  if (prefSaving.value) return
  prefSaving.value = true
  prefMsg.value = ''
  prefMsgError.value = false
  applyReaderConfig(pref.value)
  try {
    // ① 读现有云端配置（失败不影响保存——仅提交已知字段）
    let existing: Record<string, unknown> = {}
    try {
      const res = await getUserConfig()
      const cfg =
        res.data && typeof res.data === 'object'
          ? (res.data as Record<string, unknown>).config
          : undefined
      if (cfg && typeof cfg === 'object') existing = cfg as Record<string, unknown>
    } catch {
      /* 读取失败：仅保存已知字段 */
    }
    // ② 合并：云端未知字段保留，仅更新已知字段
    await saveUserConfig({
      ...existing,
      ...toServerConfig(pref.value),
      ...uiThemeToServer(uiTheme.value),
    })
    prefMsg.value = '已保存到服务器，多端一致'
  } catch (err) {
    prefMsg.value = isNotImplemented(err)
      ? '保存接口后端暂未提供（POST /reader3/saveUserConfig）· 已保存到本机'
      : `保存失败：${err instanceof Error ? err.message : '请稍后重试'}`
    prefMsgError.value = true
  } finally {
    prefSaving.value = false
  }
}

/* ================= 恢复默认设置（GAP 38：清空阅读偏好 localStorage + 重置为默认，确认弹窗） ================= */
/** 阅读偏好 localStorage 键（与 utils/readerConfig.ts 的 KEY_MAP 一致） */
const READER_PREF_KEYS = [
  'reader_han_mode',
  'reader_theme',
  'reader_font_size',
  'reader_line_height',
  'reader_para_spacing',
  'reader_font_weight',
  'reader_content_width',
  'reader_font_family',
  'reader_letter_spacing',
  'reader_text_indent',
  'reader_text_align',
  'reader_page_mode',
]

const resetPrefBusy = ref(false)

async function resetPref() {
  if (resetPrefBusy.value) return
  try {
    await ElMessageBox.confirm(
      '确定恢复默认设置吗？本机阅读偏好将清空并重置为默认值（服务器配置仍优先，可点「保存到云端」覆盖）。',
      '恢复默认',
      { confirmButtonText: '恢复默认', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return // 用户取消
  }
  resetPrefBusy.value = true
  try {
    for (const k of READER_PREF_KEYS) {
      try {
        localStorage.removeItem(k)
      } catch {
        /* 单个键失败继续 */
      }
    }
    const defaults = { ...READER_CONFIG_DEFAULTS }
    pref.value = defaults
    applyReaderConfig(defaults)
    prefMsg.value = '已恢复默认设置（本机）'
    prefMsgError.value = false
  } finally {
    resetPrefBusy.value = false
  }
}

/* ================= GAP 4：阅读背景（纯色/纸纹/图片——图片上传到服务器 assets/background/，本地只记路径） ================= */

const BG_OPTIONS: { value: BgMode; label: string }[] = [
  { value: 'color', label: '纯色' },
  { value: 'texture', label: '纸纹' },
  { value: 'preset', label: '内置图' },
  { value: 'image', label: '图片' },
]

const bgMode = ref<BgMode>(loadBgMode())
const bgImagePath = ref(loadBgImagePath())
const bgPreset = ref(loadBgPreset())
/** 预览/阅读展示 URL（file/download + accessToken） */
const bgImageUrl = computed(() => bgImageUrlOf(bgImagePath.value, store.accessToken))
const bgImageName = computed(() => {
  const p = bgImagePath.value
  return p ? p.split('/').pop() || p : ''
})
const bgUploadBusy = ref(false)
const bgPick = ref<HTMLInputElement | null>(null)
const imageProxy = ref(imageProxyEnabled())

function toggleImageProxy() {
  imageProxy.value = !imageProxy.value
  setImageProxyEnabled(imageProxy.value)
  ElMessage.success(imageProxy.value ? '已开启图片代理（远端图片经服务器回源）' : '已关闭图片代理')
}

/** 背景模式切换：纸纹同步 reader_texture（阅读页纸纹开关同源），纯色清除 */
function setBgMode(m: BgMode) {
  bgMode.value = m
  if (m === 'preset') {
    bgPreset.value = loadBgPreset()
    saveBgPreset(bgPreset.value)
  }
  saveBgMode(m)
  try {
    if (m === 'texture') localStorage.setItem('reader_texture', '1')
    else localStorage.removeItem('reader_texture')
  } catch {
    /* ignore */
  }
}

/** 选择内置背景图：写入名称并切到 preset 模式 */
function pickBgPreset(name: string) {
  bgPreset.value = name
  saveBgPreset(name)
  bgMode.value = 'preset'
  saveBgMode('preset')
}

/** 上传背景图：file/upload 到 assets/background/（用户根，后端 file API）；目标目录不存在时先 mkdir */
function ensureBgDir(): Promise<void> {
  // mkdir 幂等失败静默（目录已存在时后端可能报错，不影响上传）
  return mkdir('', 'assets', '', { silent: true })
    .catch(() => undefined)
    .then(() => mkdir('assets', 'background', '', { silent: true }).catch(() => undefined))
    .then(() => undefined)
}

async function onBgPick(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = '' // 允许重复选择同一文件
  if (!file) return
  if (!file.type.startsWith('image/')) {
    ElMessage.warning('请选择图片文件（png/jpg/webp 等）')
    return
  }
  bgUploadBusy.value = true
  try {
    await ensureBgDir()
    // 上传返回 FileItem[]（path 相对用户根，形如 /assets/background/x.jpg）
    const res = await uploadFile(file, 'assets/background', '')
    const items = (res.data ?? []) as { path?: string }[]
    const path = items[0]?.path ? items[0].path.replace(/^\/+/, '') : `assets/background/${file.name}`
    bgImagePath.value = path
    saveBgImagePath(path)
    bgMode.value = 'image'
    saveBgMode('image')
    ElMessage.success('背景图已上传（阅读页图片背景生效）')
  } catch {
    // 请求层已提示
  } finally {
    bgUploadBusy.value = false
  }
}

function removeBgImage() {
  bgImagePath.value = ''
  saveBgImagePath('')
  if (bgMode.value === 'image') {
    bgMode.value = 'color'
    saveBgMode('color')
  }
  ElMessage.success('已移除背景图')
}

/* ================= GAP 5：自定义样式（reader_custom_css → 注入全局 <style>；输入停顿自动保存并生效） ================= */

const customCss = ref(loadCustomCss())
let cssTimer: number | undefined
watch(customCss, (v) => {
  if (cssTimer !== undefined) window.clearTimeout(cssTimer)
  cssTimer = window.setTimeout(() => {
    saveCustomCss(v)
    applyCustomCss(v)
  }, 400)
})

function restoreCustomCss() {
  customCss.value = ''
  saveCustomCss('')
  applyCustomCss('')
  ElMessage.success('已恢复默认（自定义样式已清空）')
}

onBeforeUnmount(() => {
  if (cssTimer !== undefined) window.clearTimeout(cssTimer)
})

/* ================= 阅读统计（GET /reader3/getReadingStats；后端未就绪时本地进度降级） ================= */

/** 归一化后的展示模型（兼容后端秒数形态与契约对象形态） */
interface StatsView {
  today: { seconds: number; count: number }
  week: { seconds: number; count: number }
  total: { seconds: number; count: number }
  top: { name: string; bookUrl?: string; seconds?: number; chars?: number; count?: number }[]
}

const statsOpen = ref(false)
const statsLoading = ref(false)
const statsFromLocal = ref(false)
const stats = ref<StatsView | null>(null)
const statsMsg = ref('')
const statsMsgError = ref(false)

function numOr(v: unknown, fallback = 0): number {
  const n = Number(v)
  return Number.isFinite(n) ? n : fallback
}

/** 容错解析后端统计：today/week/total 为秒数（数字）或 {count,minutes,books} 对象 */
function normalizeStats(raw: unknown): StatsView | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  const cell = (x: unknown): { seconds: number; count: number } => {
    if (typeof x === 'number') return { seconds: Math.max(0, x), count: 0 }
    const it = (x && typeof x === 'object' ? x : {}) as Record<string, unknown>
    const minutes = numOr(it.minutes)
    return {
      seconds: typeof it.seconds === 'number' ? Math.max(0, it.seconds) : minutes > 0 ? Math.round(minutes * 60) : 0,
      count: numOr(it.count),
    }
  }
  const list = Array.isArray(o.books) ? o.books : Array.isArray(o.topBooks) ? o.topBooks : []
  const top: StatsView['top'] = []
  for (const t of list) {
    if (typeof t === 'string') {
      top.push({ name: t })
      continue
    }
    if (!t || typeof t !== 'object') continue
    const it = t as Record<string, unknown>
    const name = typeof it.name === 'string' ? it.name : typeof it.bookName === 'string' ? it.bookName : ''
    if (!name) continue
    top.push({
      name,
      bookUrl: typeof it.bookUrl === 'string' ? it.bookUrl : undefined,
      seconds: typeof it.seconds === 'number' ? it.seconds : undefined,
      chars: typeof it.chars === 'number' ? it.chars : undefined,
      count: typeof it.count === 'number' ? it.count : undefined,
    })
  }
  return { today: cell(o.today), week: cell(o.week), total: cell(o.total), top }
}

/** 秒 → 阅读时长文案 */
function fmtMinutes(seconds: number): string {
  if (seconds <= 0) return ''
  if (seconds < 60) return '不足 1 分钟'
  return `${Math.round(seconds / 60)} 分钟`
}

/** TOP 行右侧数值文案 */
function topValue(b: StatsView['top'][number]): string {
  if (typeof b.seconds === 'number' && b.seconds > 0) return fmtMinutes(b.seconds)
  if (typeof b.chars === 'number' && b.chars > 0) return `${(b.chars / 10000).toFixed(1)} 万字`
  if (typeof b.count === 'number') return `${b.count} 次`
  return '—'
}

function startOfDay(d: Date): number {
  const x = new Date(d)
  x.setHours(0, 0, 0, 0)
  return x.getTime()
}

function startOfWeek(d: Date): number {
  const day0 = startOfDay(d)
  const offset = (d.getDay() + 6) % 7 // 周一为起点
  return day0 - offset * 86400000
}

/** GAP 110：近 7 天每日时长柱状图数据（本机 reader_daily_stats 累计；纯 CSS div 高度，无图表库） */
interface DailyBar {
  label: string
  seconds: number
  heightPct: number
}

function readDailyStatsLocal(): Record<string, number> {
  try {
    return parseDailyStats(localStorage.getItem(DAILY_STATS_KEY))
  } catch {
    return {}
  }
}

const dailyBars = computed<DailyBar[]>(() => {
  const map = readDailyStatsLocal()
  const days = last7Days(map)
  const max = Math.max(...days.map((x) => x.seconds), 0)
  return days.map((x) => ({
    label: x.date.slice(5), // MM-DD
    seconds: x.seconds,
    // 有值日至少 8% 高度（可见）；零值日 0（CSS min-height 留 2px 占位）
    heightPct: max > 0 && x.seconds > 0 ? Math.max(8, Math.round((x.seconds / max) * 100)) : 0,
  }))
})
/** 近 7 天有实际时长才显示柱状图 */
const dailyHasData = computed(() => dailyBars.value.some((b) => b.seconds > 0))

/** 本地降级：从 reader-progress-{bookUrl} 汇总近似统计（name 经书架映射；GAP 110：今日/近 7 天秒数取自本机每日累计） */
function localStats(nameMap: Map<string, string>): StatsView {
  const day0 = startOfDay(new Date())
  const week0 = startOfWeek(new Date())
  const entries: { name: string; updatedAt: number }[] = []
  try {
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (!key || !key.startsWith('reader-progress-')) continue
      const raw = localStorage.getItem(key)
      if (!raw) continue
      const p = JSON.parse(raw) as { updatedAt?: unknown }
      if (typeof p.updatedAt !== 'number') continue
      const bookUrl = key.slice('reader-progress-'.length)
      entries.push({ name: nameMap.get(bookUrl) || bookUrl, updatedAt: p.updatedAt })
    }
  } catch {
    /* ignore */
  }
  const countSince = (t: number) => entries.filter((e) => e.updatedAt >= t).length
  const top = [...entries].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 5)
  // GAP 110：每日时长（本机累计）→ 今日/近 7 天秒数（与柱状图一致）
  const days = last7Days(readDailyStatsLocal())
  const weekSeconds = days.reduce((s, x) => s + x.seconds, 0)
  const todaySeconds = days[days.length - 1]?.seconds ?? 0
  return {
    today: { seconds: todaySeconds, count: countSince(day0) },
    week: { seconds: weekSeconds, count: countSince(week0) },
    total: { seconds: 0, count: entries.length },
    top: top.map((e) => ({ name: e.name, count: 1 })),
  }
}

async function loadStats() {
  statsOpen.value = true
  document.body.style.overflow = 'hidden'
  statsLoading.value = true
  statsMsg.value = ''
  statsMsgError.value = false
  statsFromLocal.value = false
  stats.value = null
  try {
    const res = await getReadingStats()
    stats.value = normalizeStats(res.data)
    if (!stats.value) throw new Error('empty')
  } catch {
    // 后端未就绪/数据为空：本地降级近似统计
    statsFromLocal.value = true
    try {
      const shelfRes = await getBookshelf().catch(() => null)
      const nameMap = new Map((shelfRes?.data ?? []).map((b) => [b.bookUrl, b.name]))
      stats.value = localStats(nameMap)
      statsMsg.value = '后端统计接口暂未提供（GET /reader3/getReadingStats）· 以下为本地近似统计'
    } catch {
      stats.value = null
      statsMsg.value = '统计加载失败'
      statsMsgError.value = true
    }
  } finally {
    statsLoading.value = false
  }
}

function closeStats() {
  statsOpen.value = false
  document.body.style.overflow = ''
}

/* ================= 缓存管理（契约 GET /reader3/getCacheInfo + POST /reader3/clearCache） ================= */

const cacheInfo = ref<CacheInfo | null>(null)
/** 后端契约是否可用（getCacheInfo 静默探测；未实现时置 false，界面显示「后端待实现」） */
const cacheReady = ref(false)
const cacheBusy = ref(false)

/** 清理类型（极简胶囊单选）：目录 / 章节 / 全部 */
const CLEAR_TYPES: { value: CacheClearType; label: string }[] = [
  { value: 'toc', label: '目录' },
  { value: 'chapters', label: '章节' },
  { value: 'all', label: '全部' },
]
const cacheType = ref<CacheClearType>('chapters')

function fmtSize(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return '0 B'
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}

async function loadCacheInfo() {
  try {
    const res = await getCacheInfo()
    cacheInfo.value = res.data ?? null
    cacheReady.value = true
  } catch {
    // 接口未实现（404）/网络失败：静默降级显示「后端待实现」
    cacheInfo.value = null
    cacheReady.value = false
  }
}

function clearTypeLabel(t: CacheClearType): string {
  return CLEAR_TYPES.find((x) => x.value === t)?.label ?? t
}

async function runClearCache() {
  if (!cacheReady.value) {
    ElMessage.info('清理缓存接口后端待实现（POST /reader3/clearCache）')
    return
  }
  if (cacheBusy.value) return
  cacheBusy.value = true
  try {
    await clearCache(cacheType.value)
    ElMessage.success(`已清理${clearTypeLabel(cacheType.value)}缓存`)
    await loadCacheInfo()
  } catch {
    // 已提示
  } finally {
    cacheBusy.value = false
  }
}

/* ================= OPDS 访问 ================= */
/** OPDS 地址 = 当前 host + /opds（secure 模式附带 accessToken=用户名:token，外部阅读器免输入密码） */
const opdsUrl = computed(() => {
  const base = `${window.location.origin}/opds`
  return store.accessToken ? `${base}?accessToken=${encodeURIComponent(store.accessToken)}` : base
})
const opdsCopied = ref(false)

/* GAP 53：OPDS 独立账号 + 测试连接（GET/POST /reader3/getOpdsSettings|saveOpdsSettings；fetch /opds 验证） */
const opdsCfg = ref<{ enabled: boolean; username: string; passwordSet: boolean }>({
  enabled: false,
  username: '',
  passwordSet: false,
})
const opdsCfgOpen = ref(false)
const opdsCfgBusy = ref(false)
const opdsCfgMsg = ref('')
const opdsCfgMsgError = ref(false)
const opdsForm = ref({ username: '', password: '' })
/** 配置时输入的密码留内存：测试连接用 Basic 认证（服务端不回传密码） */
let opdsTestPassword = ''
const opdsTesting = ref(false)
const opdsTestOk = ref<boolean | null>(null)
const opdsTestMsg = ref('')

async function loadOpdsCfg() {
  try {
    const res = await getOpdsSettings()
    const d = res.data
    if (d) {
      opdsCfg.value = { enabled: !!d.enabled, username: d.username || '', passwordSet: !!d.passwordSet }
    }
  } catch {
    /* 后端不可用：保持默认 */
  }
}

function openOpdsCfg() {
  opdsForm.value = { username: opdsCfg.value.username, password: '' }
  opdsCfgMsg.value = ''
  opdsCfgMsgError.value = false
  opdsCfgOpen.value = true
  document.body.style.overflow = 'hidden'
}

function closeOpdsCfg() {
  if (opdsCfgBusy.value) return
  opdsCfgOpen.value = false
  document.body.style.overflow = ''
}

/** 保存账号（username 空 = 禁用独立账号） */
async function saveOpdsCfg() {
  if (opdsCfgBusy.value) return
  const username = opdsForm.value.username.trim()
  const password = opdsForm.value.password
  if (username && password.length < 4) {
    opdsCfgMsg.value = '密码至少 4 位'
    opdsCfgMsgError.value = true
    return
  }
  if (username && !password && opdsCfg.value.username !== username) {
    opdsCfgMsg.value = '新账号需填写密码'
    opdsCfgMsgError.value = true
    return
  }
  opdsCfgBusy.value = true
  opdsCfgMsg.value = ''
  try {
    const res = await saveOpdsSettings(username, password)
    const d = res.data
    opdsCfg.value = {
      enabled: !!d?.enabled,
      username: d?.username ?? username,
      passwordSet: !!d?.enabled,
    }
    // 本次输入的密码留内存供「测试连接」Basic 认证使用（刷新后失效——服务端不回传密码）
    if (username && password) opdsTestPassword = password
    if (!username) opdsTestPassword = ''
    ElMessage.success(username ? 'OPDS 账号已保存' : '已禁用 OPDS 独立账号')
    closeOpdsCfg()
  } catch {
    // 错误提示已由拦截器处理
  } finally {
    opdsCfgBusy.value = false
  }
}

/** 测试连接：fetch /opds——配置了独立账号且密码在内存 → Basic 认证；否则带 accessToken */
async function testOpds() {
  if (opdsTesting.value) return
  opdsTesting.value = true
  opdsTestOk.value = null
  opdsTestMsg.value = '测试中…'
  try {
    const url = `${window.location.origin}/opds`
    const headers: Record<string, string> = { Accept: 'application/atom+xml, application/opds+json, */*' }
    const user = opdsCfg.value.username.trim()
    if (opdsCfg.value.enabled && user && opdsTestPassword) {
      headers.Authorization = `Basic ${btoa(`${user}:${opdsTestPassword}`)}`
    } else if (store.accessToken) {
      // 独立账号密码不可用（未在本会话配置）→ 回退 accessToken
      headers.Authorization = `Bearer ${store.accessToken}`
    }
    const resp = await fetch(url, { headers })
    if (resp.ok) {
      opdsTestOk.value = true
      opdsTestMsg.value = `连接成功（HTTP ${resp.status} · ${resp.headers.get('content-type')?.split(';')[0] || 'OPDS'}）`
    } else if (resp.status === 401 || resp.status === 403) {
      opdsTestOk.value = false
      opdsTestMsg.value = `连接失败：认证未通过（HTTP ${resp.status}）——请检查 OPDS 账号或 accessToken`
    } else {
      opdsTestOk.value = false
      opdsTestMsg.value = `连接失败（HTTP ${resp.status}）`
    }
  } catch (err) {
    opdsTestOk.value = false
    opdsTestMsg.value = `连接失败：${err instanceof Error ? err.message : '网络错误'}`
  } finally {
    opdsTesting.value = false
  }
}

/** 复制文本：优先剪贴板 API，不可用时 textarea 降级；返回是否成功 */
async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    try {
      const ta = document.createElement('textarea')
      ta.value = text
      ta.style.position = 'fixed'
      ta.style.opacity = '0'
      document.body.appendChild(ta)
      ta.select()
      document.execCommand('copy')
      document.body.removeChild(ta)
      return true
    } catch {
      return false
    }
  }
}

async function copyOpdsUrl() {
  if (await copyText(opdsUrl.value)) {
    opdsCopied.value = true
    window.setTimeout(() => (opdsCopied.value = false), 1600)
  }
}

/* ================= WebDAV 访问地址（GAP 40：http://host:port/reader3/webdav/，与 OPDS 卡片同风格） ================= */
const webdavUrl = computed(() => `${window.location.origin}/reader3/webdav/`)
const webdavCopied = ref(false)

async function copyWebdavUrl() {
  if (await copyText(webdavUrl.value)) {
    webdavCopied.value = true
    window.setTimeout(() => (webdavCopied.value = false), 1600)
  }
}

/* ================= 数据备份（WebDAV） ================= */
const backupBusy = ref(false)
const backupPath = ref('')
const backupDownloadBusy = ref(false)
/** GAP 151：备份目标子目录（默认 webdav/legado；localStorage 记忆） */
const BACKUP_PATH_KEY = 'reader_backup_path'
const backupDir = ref(localStorage.getItem(BACKUP_PATH_KEY) || 'webdav/legado')
watch(backupDir, (v) => {
  try {
    localStorage.setItem(BACKUP_PATH_KEY, v)
  } catch {
    /* ignore */
  }
})

async function runBackup() {
  if (backupBusy.value) return
  backupBusy.value = true
  backupPath.value = ''
  try {
    const res = await backupToWebdav(backupDir.value.trim() || undefined)
    backupPath.value = res.data?.path ?? ''
    if (!backupPath.value) {
      ElMessage.warning('备份完成，但未返回文件路径')
    } else {
      ElMessage.success('备份完成')
    }
  } catch {
    // 错误提示已由拦截器处理
  } finally {
    backupBusy.value = false
  }
}

/** 下载备份 zip：返回绝对路径 → 取其文件名拼 __HOME__/{备份路径} 相对路径 → file/download */
async function downloadBackup() {
  if (backupDownloadBusy.value) return
  const abs = backupPath.value
  if (!abs) return
  backupDownloadBusy.value = true
  try {
    const blob = await downloadBackupZip(abs, backupDir.value.trim() || 'webdav/legado')
    const name = abs.split(/[\\/]/).filter(Boolean).pop() || 'backup.zip'
    await downloadBlob(blob, name)
  } catch {
    // 请求层已提示
  } finally {
    backupDownloadBusy.value = false
  }
}

/* ================= 导出数据（GAP 39：备份为 zip 并直接下载，复用备份/下载逻辑） ================= */
const exportBusy = ref(false)

async function runExportData() {
  if (exportBusy.value) return
  exportBusy.value = true
  try {
    const res = await backupToWebdav()
    const abs = res.data?.path ?? ''
    if (!abs) {
      ElMessage.warning('备份完成但未返回文件路径，无法下载')
      return
    }
    const blob = await downloadBackupZip(abs)
    const name = abs.split(/[\\/]/).filter(Boolean).pop() || 'backup.zip'
    await downloadBlob(blob, name)
    ElMessage.success('已导出备份')
  } catch {
    // 请求层已提示
  } finally {
    exportBusy.value = false
  }
}
</script>

<template>
  <div class="settings-page">
    <!-- 顶部导航（P3-A：共享 TopNav） -->
    <TopNav active="/settings" :links="['bookshelf', 'search', 'sources', 'rules', 'users', 'settings']" show-users-link />

    <main class="content">
      <div class="section-head">
        <h1 class="section-title">设置</h1>
        <span class="count">v{{ VERSION }}</span>
      </div>

      <!-- 账号信息 -->
      <section class="card">
        <h2 class="card-title">账号信息</h2>
        <div class="row">
          <span class="row-label">用户名</span>
          <span class="row-value">{{ store.username || '未登录' }}</span>
        </div>
        <div class="row">
          <span class="row-label">Token</span>
          <span class="row-value mono">{{ showToken ? store.accessToken : maskToken(store.accessToken) }}</span>
          <button
            v-if="store.accessToken"
            class="row-action"
            type="button"
            @click="showToken = !showToken"
          >
            {{ showToken ? '隐藏' : '显示' }}
          </button>
        </div>
        <div class="card-foot">
          <button class="ghost-btn" type="button" @click="openPwd">修改密码</button>
          <button class="danger-btn" type="button" @click="logout">退出登录</button>
        </div>
      </section>

      <!-- 外观（界面主题：浅色/深色/跟随系统——与阅读内容主题分离） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">外观</h2>
          <span class="card-sub">界面主题 · 切换即时生效（本机），保存到云端后多端一致</span>
          <button class="row-action" type="button" :disabled="prefSaving" @click="savePref">
            {{ prefSaving ? '保存中…' : '保存到云端' }}
          </button>
        </div>
        <div class="row">
          <span class="row-label">界面主题</span>
          <div class="pref-pills">
            <button
              v-for="o in UI_THEME_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: uiTheme === o.value }"
              type="button"
              @click="uiTheme = o.value"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <div class="row">
          <span class="row-label">阅读内容主题</span>
          <span class="row-value hint">阅读页内独立设置（浅色/深色/暖色/跟随系统），与界面主题互不影响</span>
        </div>
      </section>

      <!-- 阅读偏好（多端同步：GET/POST /reader3/getUserConfig|saveUserConfig，服务器优先） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">阅读偏好</h2>
          <span class="card-sub">简繁 / 阅读主题 / 排版 · 服务器与本地合并，服务器优先</span>
          <button class="row-action" type="button" :disabled="prefSaving" @click="savePref">
            {{ prefSaving ? '保存中…' : '保存到云端' }}
          </button>
          <button class="row-action" type="button" :disabled="resetPrefBusy" @click="resetPref">
            恢复默认
          </button>
        </div>
        <div class="row">
          <span class="row-label">简繁</span>
          <div class="pref-pills">
            <button
              v-for="o in HAN_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: pref.hanMode === o.value }"
              type="button"
              @click="pref.hanMode = o.value"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <div class="row">
          <span class="row-label">阅读主题</span>
          <div class="pref-pills">
            <button
              v-for="o in THEME_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: pref.theme === o.value }"
              type="button"
              @click="pref.theme = o.value"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <div class="row">
          <span class="row-label">字号</span>
          <div class="pref-slider">
            <input v-model.number="pref.fontSize" class="pref-range" type="range" min="14" max="22" step="1" />
            <span class="pref-val">{{ pref.fontSize }}px</span>
          </div>
        </div>
        <div class="row">
          <span class="row-label">行距</span>
          <div class="pref-slider">
            <input v-model.number="pref.lineHeight" class="pref-range" type="range" min="1.5" max="2.5" step="0.1" />
            <span class="pref-val">{{ pref.lineHeight.toFixed(1) }}</span>
          </div>
        </div>
        <div class="row">
          <span class="row-label">段距</span>
          <div class="pref-slider">
            <input v-model.number="pref.paraSpacing" class="pref-range" type="range" min="0.5" max="2" step="0.1" />
            <span class="pref-val">{{ pref.paraSpacing.toFixed(1) }}</span>
          </div>
        </div>
        <div class="row">
          <span class="row-label">字重</span>
          <div class="pref-slider">
            <input v-model.number="pref.fontWeight" class="pref-range" type="range" min="300" max="500" step="50" />
            <span class="pref-val">{{ pref.fontWeight }}</span>
          </div>
        </div>
        <div class="row">
          <span class="row-label">宽度</span>
          <div class="pref-pills">
            <button
              v-for="o in WIDTH_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: pref.contentWidth === o.value }"
              type="button"
              @click="pref.contentWidth = o.value"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <div class="row">
          <span class="row-label">字体</span>
          <select v-model="pref.fontFamily" class="pref-select">
            <option v-for="o in FONT_OPTIONS" :key="o.value" :value="o.value">{{ o.label }}</option>
          </select>
        </div>
        <div class="row">
          <span class="row-label">字距</span>
          <div class="pref-slider">
            <input v-model.number="pref.letterSpacing" class="pref-range" type="range" min="0" max="2" step="0.5" />
            <span class="pref-val">{{ pref.letterSpacing.toFixed(1) }}</span>
          </div>
        </div>
        <div class="row">
          <span class="row-label">首行缩进</span>
          <button
            class="switch"
            :class="{ on: pref.textIndent }"
            type="button"
            role="switch"
            :aria-checked="pref.textIndent"
            @click="pref.textIndent = !pref.textIndent"
          >
            <span class="switch-knob"></span>
          </button>
        </div>
        <div class="row">
          <span class="row-label">对齐</span>
          <div class="pref-pills">
            <button
              v-for="o in ALIGN_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: pref.textAlign === o.value }"
              type="button"
              @click="pref.textAlign = o.value"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <div class="row">
          <span class="row-label">翻页</span>
          <div class="pref-pills">
            <button
              v-for="o in PAGE_MODE_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: pref.pageMode === o.value }"
              type="button"
              @click="pref.pageMode = o.value"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <p v-if="prefMsg" class="card-note" :class="{ error: prefMsgError }">{{ prefMsg }}</p>
        <p class="card-note">阅读页内的调整同样写入本机；「保存到云端」后多端一致（服务器优先）。</p>
      </section>

      <!-- 阅读背景（GAP 4：纯色 / 纸纹 / 自定义图片——图片上传到服务器 assets/background/） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">阅读背景</h2>
          <span class="card-sub">阅读页背景 · 纯色 / 纸纹 / 内置图 / 自定义图片</span>
        </div>
        <div class="row">
          <span class="row-label">背景</span>
          <div class="pref-pills">
            <button
              v-for="o in BG_OPTIONS"
              :key="o.value"
              class="capsule"
              :class="{ active: bgMode === o.value }"
              type="button"
              @click="setBgMode(o.value)"
            >
              {{ o.label }}
            </button>
          </div>
        </div>
        <div v-if="bgMode === 'preset'" class="row">
          <span class="row-label">内置图</span>
          <div class="bg-preset-grid">
            <button
              v-for="name in BG_PRESETS"
              :key="name"
              class="bg-preset-item"
              :class="{ active: bgPreset === name }"
              type="button"
              :style="{ backgroundImage: `url('${bgPresetUrl(name)}')` }"
              :title="name"
              @click="pickBgPreset(name)"
            >
              <span class="bg-preset-name">{{ name }}</span>
            </button>
          </div>
        </div>
        <div v-if="bgMode === 'image'" class="row">
          <span class="row-label">背景图</span>
          <span class="row-value mono" :title="bgImagePath">{{ bgImageName || '未上传' }}</span>
          <button class="row-action" type="button" :disabled="bgUploadBusy" @click="bgPick?.click()">
            {{ bgUploadBusy ? '上传中…' : '上传背景图' }}
          </button>
          <input ref="bgPick" class="visually-hidden" type="file" accept="image/*" @change="onBgPick" />
          <button v-if="bgImageName" class="row-action" type="button" :disabled="bgUploadBusy" @click="removeBgImage">
            移除
          </button>
        </div>
        <div v-if="bgMode === 'image' && bgImageUrl" class="bg-preview-wrap">
          <div class="bg-preview" :style="{ backgroundImage: `url('${bgImageUrl}')` }"></div>
          <span class="bg-preview-tip">预览（阅读页为固定铺满 + 遮罩）</span>
        </div>
        <div class="row">
          <span class="row-label">图片代理</span>
          <button
            class="switch"
            :class="{ on: imageProxy }"
            type="button"
            role="switch"
            :aria-checked="imageProxy"
            :title="imageProxy ? '关闭图片代理（图片直连书源）' : '开启图片代理（封面/正文/漫画经服务器回源）'"
            @click="toggleImageProxy"
          >
            <span class="switch-knob"></span>
          </button>
          <span class="row-value hint">封面 / 正文图片 / 漫画统一走 /assets/proxy 回源，防盗链且复用书源登录态</span>
        </div>
        <p class="card-note">内置图为随前端发布的 14 张经典阅读背景；自定义图片经 file/upload 保存到服务器 assets/background/（用户目录），本机仅记路径。图片背景在阅读页叠加半透明遮罩保证文字可读；secure 模式写文件需管理密码，上传失败时以页面提示为准。</p>
      </section>

      <!-- 自定义样式（GAP 5：reader_custom_css → 注入全局 <style>，阅读器/界面均可覆盖） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">自定义样式</h2>
          <span class="card-sub">自定义 CSS · 注入阅读器与全局界面（本机）</span>
          <button class="row-action" type="button" @click="restoreCustomCss">恢复默认</button>
        </div>
        <textarea
          v-model="customCss"
          class="css-editor"
          rows="8"
          spellcheck="false"
          placeholder=".reader-content { letter-spacing: 0.5px; }&#10;.reader-page { padding-top: 8px; }"
        ></textarea>
        <p class="card-note">输入停顿约 0.4s 后自动保存并注入（localStorage: reader_custom_css）；可覆盖阅读页 .reader-page/.reader-content 及全局样式。「恢复默认」清空。</p>
      </section>

      <!-- 阅读统计（GET /reader3/getReadingStats；后端未就绪时本地降级） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">阅读统计</h2>
          <span class="card-sub">今日 / 本周 / 总计 · 书籍 TOP</span>
          <button class="row-action" type="button" @click="loadStats">查看</button>
        </div>
        <p class="card-note">后端 GET /reader3/getReadingStats 未就绪时自动降级为本地近似统计。</p>
      </section>

      <!-- 听书设置 -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">听书设置</h2>
          <span class="card-sub">HttpTTS 朗读源 · 已接入服务端（账号内多设备一致；服务不可用时降级本地）</span>
          <div class="card-actions">
            <input
              ref="ttsImportRef"
              class="visually-hidden"
              type="file"
              accept="application/json,.json"
              @change="onTtsImportChange"
            />
            <button class="row-action" type="button" title="导入听书源 JSON" @click="ttsImportRef?.click()">导入</button>
            <button class="row-action" type="button" title="导出听书源 JSON" :disabled="ttsList.length === 0" @click="exportTtsJson">导出</button>
            <button
              v-if="ttsSelected.size > 0"
              class="row-action danger"
              type="button"
              :disabled="ttsBusy"
              @click="removeSelectedTts"
            >
              删除勾选 ({{ ttsSelected.size }})
            </button>
            <button
              v-else
              class="row-action"
              type="button"
              title="全选后批量删除"
              :disabled="ttsList.length === 0"
              @click="ttsSelected = new Set(ttsList.map((t) => t.id || t.url))"
            >
              全选
            </button>
            <button class="row-action" type="button" @click="openAddTts">新增听书源</button>
          </div>
        </div>
        <ul v-if="ttsList.length" class="tts-list">
          <li v-for="t in ttsList" :key="t.id" class="tts-row">
            <input
              class="tts-check"
              type="checkbox"
              :checked="ttsSelected.has(t.id) || ttsSelected.has(t.url)"
              :title="'勾选后批量删除'"
              @change="toggleTtsSelect(t.id || t.url)"
            />
            <span class="tts-name" :title="t.name">{{ t.name }}</span>
            <span class="tts-url mono" :title="t.url">{{ t.url }}</span>
            <span class="tts-type">{{ ttsTypeLabel(t.type) }}</span>
            <span v-if="t.contentType" class="tts-meta">{{ t.contentType }}</span>
            <button class="tts-edit" type="button" title="编辑听书源（完整字段）" @click="openEditTts(t)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                <path d="M12 20h9" />
                <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
              </svg>
            </button>
            <button class="tts-del" type="button" title="删除听书源" @click="askDeleteTts(t)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16" />
                <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                <path d="M6.5 7l.8 12a1.5 1.5 0 0 0 1.5 1.4h6.4a1.5 1.5 0 0 0 1.5-1.4l.8-12" />
              </svg>
            </button>
          </li>
        </ul>
        <p v-else class="tts-empty">暂无听书源。当前阅读页「听书」使用浏览器自带语音朗读。</p>
        <div class="row">
          <span class="row-label">阅读页听书</span>
          <span class="row-value">浏览器语音（SpeechSynthesis）</span>
          <span class="row-hint">阅读页顶栏「听书」按钮</span>
        </div>
      </section>

      <!-- OPDS 访问 -->
      <section class="card">
        <h2 class="card-title">OPDS 访问</h2>
        <div class="row">
          <span class="row-label">OPDS 地址</span>
          <span class="row-value mono">{{ opdsUrl }}</span>
          <button class="row-action" type="button" @click="copyOpdsUrl">
            {{ opdsCopied ? '已复制' : '复制' }}
          </button>
        </div>
        <!-- GAP 53：OPDS 独立账号（getOpdsSettings/saveOpdsSettings）+ 测试连接 -->
        <div class="row">
          <span class="row-label">OPDS 账号</span>
          <span class="row-value">
            {{ opdsCfg.enabled ? opdsCfg.username + (opdsCfg.passwordSet ? '（已设密码）' : '') : '未配置（使用登录账号）' }}
          </span>
          <button class="row-action" type="button" @click="openOpdsCfg">
            {{ opdsCfg.enabled ? '修改' : '配置' }}
          </button>
        </div>
        <div class="row">
          <span class="row-label">测试连接</span>
          <span class="row-value opds-test" :class="{ ok: opdsTestOk === true, fail: opdsTestOk === false }">
            {{ opdsTestMsg || '校验 OPDS 服务是否可访问（用当前配置账号）' }}
          </span>
          <button class="row-action" type="button" :disabled="opdsTesting" @click="testOpds">
            {{ opdsTesting ? '测试中…' : '测试连接' }}
          </button>
        </div>
        <p class="card-note">外部阅读器（如 legado、静读天下等）可通过此地址连接书架；已在地址中附带 accessToken，复制后粘贴到阅读器 OPDS 地址栏即可（未登录时不附带）。</p>
      </section>

      <!-- 数据备份 -->
      <section class="card">
        <h2 class="card-title">数据备份</h2>
        <div class="row">
          <span class="row-label">WebDAV 地址</span>
          <span class="row-value mono" :title="webdavUrl">{{ webdavUrl }}</span>
          <button class="row-action" type="button" @click="copyWebdavUrl">
            {{ webdavCopied ? '已复制' : '复制' }}
          </button>
        </div>
        <div class="row">
          <span class="row-label">备份路径</span>
          <input
            v-model="backupDir"
            class="path-input"
            type="text"
            placeholder="webdav/legado"
            maxlength="120"
            spellcheck="false"
            :title="backupDir"
          />
        </div>
        <div class="row">
          <span class="row-label">WebDAV 备份</span>
          <span class="row-value">{{ backupBusy ? '备份中…' : '备份到 WebDAV' }}</span>
          <button class="row-action" type="button" :disabled="backupBusy" @click="runBackup">
            {{ backupBusy ? '备份中…' : '立即备份' }}
          </button>
        </div>
        <div class="row">
          <span class="row-label">导出数据</span>
          <span class="row-value">备份为 zip 并直接下载（{{ backupDir.trim() || 'webdav/legado' }} 目录）</span>
          <button class="row-action" type="button" :disabled="exportBusy" @click="runExportData">
            {{ exportBusy ? '导出中…' : '导出数据' }}
          </button>
        </div>
        <div v-if="backupPath" class="row">
          <span class="row-label">下载备份</span>
          <span class="row-value mono backup-path" :title="backupPath">{{ backupPath }}</span>
          <button
            class="row-action"
            type="button"
            :disabled="backupDownloadBusy"
            @click="downloadBackup"
          >
            {{ backupDownloadBusy ? '下载中…' : '下载备份' }}
          </button>
        </div>
        <p class="card-note">WebDAV 地址供外部客户端（如 RaiDrive、文件管理器）挂载访问；备份/导出需要后端已配置 WebDAV。备份路径参数已随请求发送，后端当前固定写入 webdav/legado 目录（路径参数待后端支持）。</p>
      </section>

      <!-- 缓存（契约 GET /reader3/getCacheInfo + POST /reader3/clearCache） -->
      <section class="card">
        <h2 class="card-title">缓存</h2>
        <div class="row">
          <span class="row-label">缓存统计</span>
          <span v-if="cacheReady" class="row-value">
            章节 {{ cacheInfo?.chapterCount ?? 0 }} · 目录 {{ cacheInfo?.tocCacheCount ?? 0 }} · {{ fmtSize(cacheInfo?.totalSize ?? 0) }}
          </span>
          <span v-else class="row-value">后端待实现</span>
        </div>
        <div class="row">
          <span class="row-label">清理缓存</span>
          <div class="cache-types">
            <button
              v-for="t in CLEAR_TYPES"
              :key="t.value"
              class="capsule"
              :class="{ active: cacheType === t.value }"
              type="button"
              :disabled="!cacheReady || cacheBusy"
              @click="cacheType = t.value"
            >
              {{ t.label }}
            </button>
          </div>
          <button
            class="row-action cache-clear"
            type="button"
            :disabled="!cacheReady || cacheBusy"
            :title="cacheReady ? '清理所选类型缓存' : '清理接口后端待实现'"
            @click="runClearCache"
          >
            {{ cacheBusy ? '清理中…' : '清理' }}
          </button>
        </div>
        <p v-if="!cacheReady" class="card-note">
          缓存统计接口 GET /reader3/getCacheInfo 与清理接口 POST /reader3/clearCache 后端待实现。
        </p>
        <p v-else class="card-note">正文/目录缓存占用磁盘空间，清理后再次打开会重新拉取。</p>
      </section>

      <!-- txtTocRule（自定义 TXT 目录规则） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">txtTocRule</h2>
          <span class="card-sub">TXT 分章正则 · {{ customTocRules.length }} 条自定义</span>
          <button class="row-action" type="button" :disabled="tocImportBusy" @click="runImportDefaultToc">
            {{ tocImportBusy ? '导入中…' : '导入默认规则' }}
          </button>
          <button class="row-action" type="button" @click="openAddToc">新增规则</button>
        </div>
        <p class="card-note">上传 TXT 本地书时按启用的规则分章（无自定义规则时使用内置默认规则）。默认规则只读，可导入为自定义后编辑。</p>
        <p v-if="tocLoading" class="tts-empty">加载中…</p>
        <ul v-else-if="tocRules.length" class="tts-list toc-list">
          <li v-for="r in tocRules" :key="r.id" class="tts-row">
            <span class="tts-name" :title="r.name">{{ r.name }}</span>
            <span class="tts-url mono" :title="r.rule">{{ r.rule }}</span>
            <span class="tts-type">{{ r.id.startsWith('default-') ? '默认' : `#${r.serialNumber}` }}</span>
            <button
              class="switch"
              :class="{ on: r.enable }"
              type="button"
              role="switch"
              :aria-checked="r.enable"
              :title="r.id.startsWith('default-') ? '默认规则不可单独停用' : (r.enable ? '停用' : '启用')"
              @click="toggleTocRule(r)"
            >
              <span class="switch-knob"></span>
            </button>
            <button
              v-if="!r.id.startsWith('default-')"
              class="tts-del"
              type="button"
              title="删除规则"
              @click="askDeleteToc(r)"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 7h16" />
                <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                <path d="M6.5 7l.8 12a1.5 1.5 0 0 0 1.5 1.4h6.4a1.5 1.5 0 0 0 1.5-1.4l.8-12" />
              </svg>
            </button>
          </li>
        </ul>
        <p v-else class="tts-empty">暂无自定义规则。可「导入默认规则」或新增正则（匹配行作为章节标题）。</p>
      </section>

      <!-- 快捷键（GAP 198：全局 + 阅读器速查表，简单表格排版） -->
      <section class="card">
        <div class="card-head">
          <h2 class="card-title">快捷键</h2>
          <span class="card-sub">全局与阅读器按键速查</span>
        </div>
        <table class="keys-table">
          <thead>
            <tr>
              <th class="keys-scope">范围</th>
              <th class="keys-key">按键</th>
              <th>功能</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td rowspan="4">全局</td>
              <td><kbd class="kbd">G</kbd></td>
              <td>书架页：聚焦搜索框</td>
            </tr>
            <tr>
              <td><kbd class="kbd">S</kbd></td>
              <td>书架页：跳转设置</td>
            </tr>
            <tr>
              <td><kbd class="kbd">R</kbd></td>
              <td>书架页：刷新书架</td>
            </tr>
            <tr>
              <td><kbd class="kbd">Ctrl + K</kbd></td>
              <td>命令面板（全站可用）</td>
            </tr>
            <tr>
              <td rowspan="5">阅读器</td>
              <td><kbd class="kbd">←</kbd> / <kbd class="kbd">→</kbd></td>
              <td>上一章 / 下一章（目录抽屉打开时不触发）</td>
            </tr>
            <tr>
              <td><kbd class="kbd">PageUp</kbd> / <kbd class="kbd">PageDown</kbd></td>
              <td>向上 / 向下翻页（约 90% 视口高度）</td>
            </tr>
            <tr>
              <td><kbd class="kbd">Space</kbd></td>
              <td>音频/视频书：播放 / 暂停；文本书：自动阅读暂停 / 恢复</td>
            </tr>
            <tr>
              <td><kbd class="kbd">Esc</kbd></td>
              <td>关闭图片全屏查看</td>
            </tr>
            <tr>
              <td class="keys-none">—</td>
              <td>亮度 / 字号调节：阅读页顶栏按钮（无默认按键）</td>
            </tr>
          </tbody>
        </table>
        <p class="card-note">输入框 / 文本域聚焦时快捷键不触发；g / s / r 仅在书架页生效，Ctrl+K 命令面板全站可用（macOS 为 Cmd+K）。</p>
      </section>

      <!-- 关于 -->
      <section class="card">
        <h2 class="card-title">关于</h2>
        <div class="row">
          <span class="row-label">应用</span>
          <span class="row-value">Reader Dev（夜读）</span>
        </div>
        <div class="row">
          <span class="row-label">版本</span>
          <span class="row-value">v{{ sysInfo?.version || VERSION }}</span>
        </div>
        <div class="row">
          <span class="row-label">定位</span>
          <span class="row-value">自托管 Web 阅读服务 · 书源搜索 · 本地书仓 · OPDS · WebDAV</span>
        </div>
        <div class="row">
          <span class="row-label">技术栈</span>
          <span class="row-value">Rust + Vue 3 · legado 语义书源规则引擎</span>
        </div>
        <div class="row">
          <span class="row-label">v5.2.4</span>
          <span class="row-value">搜索并发提升（多源 24 / SSE 48）· 内置反检测浏览器增强（stealth 指纹补齐 + 反爬域名自动优先）· 失效书源检测超时修复（96 并发 + 900s 前端超时）· 书架密度按钮/悬浮简介层叠修复 · 书源管理/文件页/设置页移动端布局修复 · 正文无换行智能分句</span>
        </div>
        <div class="row">
          <span class="row-label">v5.2.3</span>
          <span class="row-value">书源导入预览选择/排序（全选/反选/新增/重复标记）· 按书源分组搜索 · 书仓目录直接扫描导入书架 · 书架已读章节与未读更新数 · 正文 script 泄漏清洗 · java.createSymmetricCrypto 对称解密 · 暂不加入可返回 · 移动端竖屏适配</span>
        </div>
        <div class="row">
          <span class="row-label">v5.2.2</span>
          <span class="row-value">KindleMOBI 尾部附加数据清理（trailing/multibyte flags）与 PalmDoc 重叠回引展开，修复 4KB 边界后中文乱码与残留 HTML</span>
        </div>
        <div class="row">
          <span class="row-label">v5.2.1</span>
          <span class="row-value">MOBI/AZW3 未知编码中文修复（PalmDoc/Huffman 原始字节解压 + chardetng 编码探测，样本正文验证通过）</span>
        </div>
        <div class="row">
          <span class="row-label">v5.2.0</span>
          <span class="row-value">阅读中换源（作者/最新章/当前章末尾预览）· 规则引擎修复（JS 搜索 URL、相对 URL、URL/URLSearchParams、jsLib/variable 全局注入）· 统计式编码探测 · 内置反检测浏览器兜底 · Docker 分层复用 · 移动端自适应 · quickKey/点击区域/切章动画/章节超时 · 离线书架缓存 · 图片代理 · 多分组 · 书源 Cookie 管理 · 自定义字体 · 文件编辑 · 精确搜书</span>
        </div>
        <div class="row">
          <span class="row-label">v5.1.0</span>
          <span class="row-value">legacy Web UI 批次 · simple-web 详情/换源/RSS 分类分页 · 内置背景图库 · 替换规则批量与 JSON · RSS 编辑与导入 · 订阅批量删除 · 阅读页详情与追更</span>
        </div>
        <div class="row">
          <span class="row-label">v5.0.9</span>
          <span class="row-value">legacy 全量对齐 · 默认 TXT 目录规则 · 本地文件名书名/作者解析 · CBZ ComicInfo 与封面</span>
        </div>
        <div class="row">
          <span class="row-label">v5.0.8</span>
          <span class="row-value">双向章节缓存 · 迁移 toc_url 回填 · 正文 HTML 清洗 · Android application 兼容</span>
        </div>
        <div class="row">
          <span class="row-label">权限模型</span>
          <span class="row-value">管理员管理系统 default 配置 · 普通用户私有覆盖仅对自己生效</span>
        </div>
        <template v-if="sysInfo">
          <div class="row">
            <span class="row-label">服务端口</span>
            <span class="row-value mono">{{ sysInfo.port }}</span>
          </div>
          <div class="row">
            <span class="row-label">用户数</span>
            <span class="row-value">{{ sysInfo.userCount }}</span>
          </div>
          <div class="row">
            <span class="row-label">书籍数</span>
            <span class="row-value">{{ sysInfo.bookCount }}</span>
          </div>
          <div class="row">
            <span class="row-label">书源数</span>
            <span class="row-value">{{ sysInfo.bookSourceCount }}</span>
          </div>
        </template>
        <div class="row">
          <span class="row-label">源码</span>
          <span class="row-value"><a class="tg-link" href="https://github.com/warpdotsys/reader-dev" target="_blank" rel="noopener">github.com/warpdotsys/reader-dev</a></span>
        </div>
        <div class="row">
          <span class="row-label">许可</span>
          <span class="row-value">GPL-3.0</span>
        </div>
        <div class="row">
          <span class="row-label">交流</span>
          <span class="row-value"><a class="tg-link" href="https://t.me/readerdev" target="_blank" rel="noopener">Telegram 群 t.me/readerdev</a></span>
        </div>
      </section>
    </main>
    <!-- GAP 53：OPDS 账号配置弹窗（saveOpdsSettings；username 空 = 禁用） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="opdsCfgOpen" class="dlg-overlay" @click.self="closeOpdsCfg">
          <div
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="OPDS 账号"
            tabindex="-1"
            @keydown.esc="closeOpdsCfg"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">OPDS 独立账号</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="opdsCfgBusy" @click="closeOpdsCfg">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="saveOpdsCfg">
              <label class="field">
                <span class="field-label">用户名</span>
                <input
                  v-model="opdsForm.username"
                  class="field-input"
                  type="text"
                  placeholder="留空 = 禁用独立账号（回退登录账号）"
                  spellcheck="false"
                  :disabled="opdsCfgBusy"
                />
              </label>
              <label class="field">
                <span class="field-label">密码</span>
                <input
                  v-model="opdsForm.password"
                  class="field-input"
                  type="password"
                  autocomplete="new-password"
                  placeholder="至少 4 位（已配置且不改则留空）"
                  :disabled="opdsCfgBusy"
                />
              </label>
              <p class="field-tip">外部阅读器用此账号 + 密码连接 OPDS（secure 模式优先于登录账号校验）</p>
              <p v-if="opdsCfgMsg" class="field-tip" :class="{ error: opdsCfgMsgError }">{{ opdsCfgMsg }}</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="opdsCfgBusy" @click="closeOpdsCfg">取消</button>
                <button class="accent-btn" type="submit" :disabled="opdsCfgBusy">
                  {{ opdsCfgBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 修改密码弹窗（GAP 87：旧密码校验 → resetUserPassword → 强制重新登录） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="pwdOpen" class="dlg-overlay" @click.self="closePwd">
          <div
            class="dlg"
            role="dialog"
            aria-modal="true"
            aria-label="修改密码"
            tabindex="-1"
            @keydown.esc="closePwd"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">修改密码</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="pwdBusy" @click="closePwd">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="submitPwd">
              <label class="field">
                <span class="field-label">旧密码<em>*</em></span>
                <input
                  v-model="pwdForm.oldPassword"
                  class="field-input"
                  type="password"
                  placeholder="当前密码"
                  maxlength="64"
                  autocomplete="current-password"
                />
              </label>
              <label class="field">
                <span class="field-label">新密码<em>*</em></span>
                <input
                  v-model="pwdForm.newPassword"
                  class="field-input"
                  type="password"
                  placeholder="至少 8 位"
                  maxlength="64"
                  autocomplete="new-password"
                />
              </label>
              <label class="field">
                <span class="field-label">确认新密码<em>*</em></span>
                <input
                  v-model="pwdForm.confirmPassword"
                  class="field-input"
                  type="password"
                  placeholder="再次输入新密码"
                  maxlength="64"
                  autocomplete="new-password"
                />
              </label>
              <p class="field-tip">校验旧密码后调用 POST /reader3/resetUserPassword；成功后需重新登录。</p>
              <p v-if="pwdMsg" class="pwd-msg" :class="{ error: pwdMsgError }">{{ pwdMsg }}</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="pwdBusy" @click="closePwd">取消</button>
                <button
                  class="accent-btn"
                  type="submit"
                  :disabled="pwdBusy || !pwdForm.oldPassword || !pwdForm.newPassword"
                >
                  {{ pwdBusy ? '提交中…' : '确认修改' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 新增听书源弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="ttsDialogOpen" class="dlg-overlay" @click.self="closeAddTts">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="新增听书源" tabindex="-1" @keydown.esc="closeAddTts">
            <div class="dlg-head">
              <h2 class="dlg-title">新增听书源</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="ttsBusy" @click="closeAddTts">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmAddTts">
              <label class="field">
                <span class="field-label">URL<em>*</em></span>
                <input v-model="ttsForm.url" class="field-input" type="text" placeholder="https://…/tts?text=" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">名称</span>
                <input v-model="ttsForm.name" class="field-input" type="text" placeholder="留空则使用 URL" maxlength="40" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">类型</span>
                <select v-model.number="ttsForm.type" class="field-input field-select">
                  <option :value="0">0 · 在线合成</option>
                  <option :value="1">1 · 本地引擎</option>
                </select>
              </label>
              <p class="field-tip">听书源已接入服务端（POST /reader3/saveHttpTTS）；离线时降级本地存储</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="ttsBusy" @click="closeAddTts">取消</button>
                <button class="accent-btn" type="submit" :disabled="ttsBusy || !ttsForm.url.trim()">
                  {{ ttsBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 编辑听书源弹窗（legacy HttpTTS 完整字段） -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="ttsEditing" class="dlg-overlay" @click.self="closeEditTts">
          <div class="dlg dlg-tts-edit" role="dialog" aria-modal="true" aria-label="编辑听书源" tabindex="-1" @keydown.esc="closeEditTts">
            <div class="dlg-head">
              <h2 class="dlg-title">编辑听书源</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="ttsSaving" @click="closeEditTts">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmEditTts">
              <label class="field">
                <span class="field-label">URL<em>*</em></span>
                <input v-model="ttsEditing.url" class="field-input" type="text" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">名称<em>*</em></span>
                <input v-model="ttsEditing.name" class="field-input" type="text" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">类型</span>
                <select v-model.number="ttsEditing.type" class="field-input field-select">
                  <option :value="0">0 · 在线合成</option>
                  <option :value="1">1 · 本地引擎</option>
                </select>
              </label>
              <label class="field">
                <span class="field-label">Content-Type</span>
                <input v-model="ttsEditing.contentType" class="field-input" type="text" placeholder="audio/mpeg（留空按音频流）" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">并发率</span>
                <input v-model="ttsEditing.concurrentRate" class="field-input" type="text" placeholder="0 = 不限制" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">登录页</span>
                <input v-model="ttsEditing.loginUrl" class="field-input" type="text" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">请求头 JSON</span>
                <textarea v-model="ttsEditing.header" class="field-input" rows="3" placeholder='{"X-Token":"…"}' spellcheck="false"></textarea>
              </label>
              <label class="field">
                <span class="field-label">JS 依赖库</span>
                <textarea v-model="ttsEditing.jsLib" class="field-input" rows="3" spellcheck="false"></textarea>
              </label>
              <label class="field">
                <span class="field-label">登录校验 JS</span>
                <textarea v-model="ttsEditing.loginCheckJs" class="field-input" rows="3" spellcheck="false"></textarea>
              </label>
              <label class="field">
                <span class="field-label">登录 UI 配置</span>
                <textarea v-model="ttsEditing.loginUi" class="field-input" rows="3" spellcheck="false"></textarea>
              </label>
              <label class="field-toggle">
                <input v-model="ttsEditing.enabledCookieJar" type="checkbox" />
                <span>启用 Cookie Jar</span>
              </label>
              <p class="field-tip">legacy HttpTTS 完整字段；保存走 POST /reader3/saveHttpTTS</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="ttsSaving" @click="closeEditTts">取消</button>
                <button class="accent-btn" type="submit" :disabled="ttsSaving || !ttsEditing.url.trim()">
                  {{ ttsSaving ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 删除听书源确认弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="deletingTts" class="dlg-overlay" @click.self="closeDeleteTts">
          <div class="dlg dlg-confirm" role="alertdialog" aria-modal="true" aria-label="删除听书源" tabindex="-1" @keydown.esc="closeDeleteTts">
            <div class="dlg-head">
              <h2 class="dlg-title">删除听书源</h2>
            </div>
            <p class="confirm-text">确定删除「{{ deletingTts.name }}」吗？此操作不可恢复。</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="deleteTtsBusy" @click="closeDeleteTts">取消</button>
              <button class="danger-btn" type="button" :disabled="deleteTtsBusy" @click="confirmDeleteTts">
                {{ deleteTtsBusy ? '删除中…' : '删除' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 新增 txtTocRule 弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="tocDialogOpen" class="dlg-overlay" @click.self="closeAddToc">
          <div class="dlg" role="dialog" aria-modal="true" aria-label="新增 txtTocRule" tabindex="-1" @keydown.esc="closeAddToc">
            <div class="dlg-head">
              <h2 class="dlg-title">新增 txtTocRule</h2>
              <button class="dlg-close" type="button" title="关闭" :disabled="tocBusy" @click="closeAddToc">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <form class="dlg-form" @submit.prevent="confirmAddToc">
              <label class="field">
                <span class="field-label">名称</span>
                <input v-model="tocForm.name" class="field-input" type="text" placeholder="留空则使用正则内容" maxlength="40" spellcheck="false" />
              </label>
              <label class="field">
                <span class="field-label">正则规则<em>*</em></span>
                <input v-model="tocForm.rule" class="field-input mono" type="text" placeholder="如 ^第.+章$" spellcheck="false" />
              </label>
              <div class="field">
                <span class="field-label">启用</span>
                <button
                  class="switch"
                  :class="{ on: tocForm.enable }"
                  type="button"
                  role="switch"
                  :aria-checked="tocForm.enable"
                  @click="tocForm.enable = !tocForm.enable"
                >
                  <span class="switch-knob"></span>
                </button>
              </div>
              <p class="field-tip">正则按行匹配（MULTILINE），匹配到的行作为章节标题；上传 TXT 时按启用的规则分章。</p>
              <div class="dlg-actions">
                <button class="ghost-btn" type="button" :disabled="tocBusy" @click="closeAddToc">取消</button>
                <button class="accent-btn" type="submit" :disabled="tocBusy || !tocForm.rule.trim()">
                  {{ tocBusy ? '保存中…' : '保存' }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 删除 txtTocRule 确认弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="deletingToc" class="dlg-overlay" @click.self="closeDeleteToc">
          <div class="dlg dlg-confirm" role="alertdialog" aria-modal="true" aria-label="删除 txtTocRule" tabindex="-1" @keydown.esc="closeDeleteToc">
            <div class="dlg-head">
              <h2 class="dlg-title">删除规则</h2>
            </div>
            <p class="confirm-text">确定删除「{{ deletingToc.name }}」吗？此操作不可恢复。</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" :disabled="deleteTocBusy" @click="closeDeleteToc">取消</button>
              <button class="danger-btn" type="button" :disabled="deleteTocBusy" @click="confirmDeleteToc">
                {{ deleteTocBusy ? '删除中…' : '删除' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
    <!-- 阅读统计弹窗 -->
    <Teleport to="body">
      <Transition name="dlg">
        <div v-if="statsOpen" class="dlg-overlay" @click.self="closeStats">
          <div
            class="dlg dlg-stats"
            role="dialog"
            aria-modal="true"
            aria-label="阅读统计"
            tabindex="-1"
            @keydown.esc="closeStats"
          >
            <div class="dlg-head">
              <h2 class="dlg-title">阅读统计</h2>
              <button class="dlg-close" type="button" title="关闭" @click="closeStats">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            </div>
            <p v-if="statsLoading" class="stats-loading">加载中…</p>
            <template v-else-if="stats">
              <p v-if="statsFromLocal" class="field-tip">{{ statsMsg }}</p>
              <div class="stats-grid">
                <div class="stats-cell">
                  <span class="stats-num">{{ stats.today.seconds > 0 ? fmtMinutes(stats.today.seconds) : stats.today.count > 0 ? stats.today.count : '0' }}</span>
                  <span class="stats-label">今日</span>
                  <span class="stats-sub">{{ stats.today.seconds > 0 ? '时长' : stats.today.count > 0 ? fmtMinutes(stats.today.seconds) || '阅读' : '阅读' }}</span>
                </div>
                <div class="stats-cell">
                  <span class="stats-num">{{ stats.week.seconds > 0 ? fmtMinutes(stats.week.seconds) : stats.week.count > 0 ? stats.week.count : '0' }}</span>
                  <span class="stats-label">本周</span>
                  <span class="stats-sub">{{ stats.week.seconds > 0 ? '时长' : stats.week.count > 0 ? fmtMinutes(stats.week.seconds) || '阅读' : '阅读' }}</span>
                </div>
                <div class="stats-cell">
                  <span class="stats-num">{{ stats.total.count > 0 ? stats.total.count : fmtMinutes(stats.total.seconds) || '0' }}</span>
                  <span class="stats-label">总计</span>
                  <span class="stats-sub">{{ stats.total.count > 0 ? fmtMinutes(stats.total.seconds) || '阅读' : '阅读' }}</span>
                </div>
              </div>
              <!-- GAP 110：近 7 天每日时长柱状图（本机累计——纯 CSS div 高度，无图表库） -->
              <div v-if="dailyHasData" class="stats-chart" role="img" aria-label="近 7 天每日阅读时长柱状图">
                <div
                  v-for="b in dailyBars"
                  :key="b.label"
                  class="chart-col"
                  :title="`${b.label} · ${fmtMinutes(b.seconds) || '0 分钟'}`"
                >
                  <div class="chart-bar" :class="{ zero: b.heightPct === 0 }" :style="{ height: b.heightPct + '%' }"></div>
                  <span class="chart-label">{{ b.label }}</span>
                </div>
              </div>
              <p v-if="dailyHasData" class="field-tip chart-tip">近 7 天每日阅读时长（本机统计——阅读页自动累计）</p>
              <template v-if="stats.top.length">
                <p class="stats-top-title">书籍 TOP{{ stats.top.length }}</p>
                <ul class="stats-top-list">
                  <li v-for="(b, i) in stats.top" :key="i" class="stats-top-row">
                    <span class="stats-rank">{{ i + 1 }}</span>
                    <span class="stats-book" :title="b.name">{{ b.name }}</span>
                    <span class="stats-val">{{ topValue(b) }}</span>
                  </li>
                </ul>
              </template>
              <p v-else class="field-tip">暂无书籍数据</p>
            </template>
            <p v-else class="stats-msg" :class="{ error: statsMsgError }">{{ statsMsg || '统计加载失败' }}</p>
            <div class="dlg-actions">
              <button class="ghost-btn" type="button" @click="closeStats">关闭</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.settings-page {
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

/* 用户区 */
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
  width: min(720px, 100%);
  margin: 0 auto;
  padding: 48px 32px 72px;
}
.section-head {
  display: flex;
  align-items: baseline;
  gap: 14px;
  margin-bottom: 32px;
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

/* ================= 卡片分区 ================= */
.card {
  margin-bottom: 28px;
  padding: 22px 24px 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}
.card-title {
  margin: 0 0 6px;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-3);
}
.card-head {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 4px;
}
.card-head .card-title {
  margin: 0;
}
.card-sub {
  flex: 1;
  min-width: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.row-action.danger {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.45);
}
.row-action.danger:hover {
  background: rgba(207, 68, 68, 0.08);
}

/* 听书源列表 */
.tts-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.tts-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.tts-name {
  flex-shrink: 0;
  max-width: 140px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tts-url {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tts-type {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.tts-meta {
  flex-shrink: 0;
  max-width: 140px;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tts-check {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}
.tts-edit {
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
.tts-edit:hover {
  color: var(--accent);
  background: var(--accent-soft);
}
.tts-edit svg {
  width: 12px;
  height: 12px;
}
.tts-del {
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
.tts-del:hover {
  color: #cf4444;
  background: rgba(207, 68, 68, 0.08);
}
.tts-del svg {
  width: 12px;
  height: 12px;
}

/* 听书源编辑弹窗 */
.dlg-tts-edit {
  width: min(520px, 100%);
  max-height: 90vh;
  overflow-y: auto;
}
.field-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 14px;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-2);
  cursor: pointer;
}
.field-toggle input {
  accent-color: var(--accent);
}

/* 极简开关（txtTocRule 启用切换） */
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
.toc-list .tts-name {
  max-width: 110px;
}
.toc-list .tts-url {
  font-size: 11.5px;
}

.tts-empty {
  margin: 12px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}

/* ================= 弹窗（新增 / 删除听书源） ================= */
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
.field-select {
  cursor: pointer;
}
.field-tip {
  margin: -4px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
/* GAP 87：修改密码结果提示 */
.pwd-msg {
  margin: 2px 0 0;
  font-size: 12px;
  font-weight: 300;
  color: var(--text-2);
}
.pwd-msg.error {
  color: #cf4444;
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
.row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 13px 0;
  border-bottom: 1px solid var(--border);
}
.row:last-of-type {
  border-bottom: none;
}
.row-label {
  flex-shrink: 0;
  width: 72px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-2);
}
.row-value {
  flex: 1;
  min-width: 0;
  font-size: 13.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.row-value.mono {
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
}
.mono {
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
}
.row-value.hint {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
/* GAP 151：备份路径输入（细字下划线风格） */
.path-input {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 2px;
  border: none;
  border-bottom: 1px solid var(--border);
  border-radius: 0;
  background: transparent;
  color: var(--text-1);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12.5px;
  font-weight: 400;
  outline: none;
  transition: border-color 0.2s ease;
}
.path-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.path-input:focus {
  border-bottom-color: var(--accent);
}
/* GAP 53：OPDS 测试连接结果着色 */
.row-value.opds-test {
  font-size: 12px;
  font-weight: 300;
  color: var(--text-3);
}
.row-value.opds-test.ok {
  color: #2e9e5b;
}
.row-value.opds-test.fail {
  color: #cf4444;
}
.row-hint {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
}
.row-action {
  flex-shrink: 0;
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
.row-action:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.card-foot {
  display: flex;
  justify-content: flex-end;
  padding-top: 16px;
}
/* GAP 198：快捷键速查表（简单表格排版） */
.keys-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12.5px;
}
.keys-table th {
  padding: 6px 10px;
  text-align: left;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-3);
  border-bottom: 1px solid var(--border);
}
.keys-table td {
  padding: 8px 10px;
  color: var(--text-2);
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.keys-table tbody tr:last-child td {
  border-bottom: none;
}
.keys-table .keys-scope {
  width: 64px;
  white-space: nowrap;
  font-weight: 400;
  color: var(--text-1);
}
.keys-table .keys-key {
  width: 190px;
  white-space: nowrap;
}
.keys-table .keys-none {
  color: var(--text-3);
}
.kbd {
  display: inline-block;
  padding: 1px 7px;
  border: 1px solid var(--border);
  border-bottom-width: 2px;
  border-radius: 5px;
  background: var(--bg);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--text-2);
}
.card-note {
  margin: 10px 0 0;
  font-size: 11.5px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--text-3);
}
.card-note.mono {
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
}
.backup-path {
  color: var(--accent);
}

/* 清理类型胶囊（极简：细字圆角条） */
.cache-types {
  display: flex;
  gap: 6px;
}
.capsule {
  padding: 3px 12px;
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
.capsule:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.capsule.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
  font-weight: 400;
}
.capsule:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.cache-clear {
  color: #cf4444;
  border-color: rgba(207, 68, 68, 0.45);
}
.cache-clear:hover:not(:disabled) {
  color: #cf4444;
  border-color: #cf4444;
}
.cache-clear:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.row-action:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.danger-btn {
  padding: 8px 20px;
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
    color 0.2s ease,
    background-color 0.2s ease,
    border-color 0.2s ease;
}
.danger-btn:hover {
  color: #ffffff;
  background: #cf4444;
  border-color: #cf4444;
}

/* ================= 阅读偏好控件 ================= */
.pref-pills {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.pref-slider {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}
.pref-range {
  flex: 1;
  min-width: 0;
  height: 18px;
  accent-color: var(--accent);
  cursor: pointer;
}
.pref-val {
  flex-shrink: 0;
  min-width: 46px;
  text-align: right;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.pref-select {
  height: 30px;
  padding: 0 8px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.pref-select:focus {
  border-color: var(--accent);
}
.card-note.error {
  color: #cf4444;
}

/* ================= 阅读背景（GAP 4） ================= */
.hidden-file {
  display: none;
}
.bg-preview-wrap {
  margin-top: 14px;
}
.bg-preview {
  height: 120px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background-color: var(--bg);
  background-size: cover;
  background-position: center;
}
.bg-preview-tip {
  display: block;
  margin-top: 6px;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.bg-preset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(92px, 1fr));
  gap: 8px;
  width: 100%;
}
.bg-preset-item {
  position: relative;
  aspect-ratio: 3 / 4;
  border-radius: var(--radius);
  border: 2px solid transparent;
  background-color: var(--bg);
  background-size: cover;
  background-position: center;
  cursor: pointer;
  overflow: hidden;
  transition:
    border-color 0.2s ease,
    transform 0.2s ease;
}
.bg-preset-item:hover {
  transform: translateY(-1px);
}
.bg-preset-item.active {
  border-color: var(--accent);
}
.bg-preset-name {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 4px 6px;
  background: rgba(0, 0, 0, 0.42);
  color: #fff;
  font-size: 11px;
  font-weight: 300;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ================= 自定义样式（GAP 5） ================= */
.css-editor {
  box-sizing: border-box;
  width: 100%;
  min-height: 140px;
  padding: 12px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: 'SF Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  font-weight: 300;
  line-height: 1.7;
  outline: none;
  resize: vertical;
  transition: border-color 0.2s ease;
}
.css-editor::placeholder {
  color: var(--text-3);
}
.css-editor:focus {
  border-color: var(--accent);
}

/* ================= 阅读统计弹窗 ================= */
.dlg-stats {
  width: min(400px, 100%);
}
.stats-loading {
  margin: 0;
  padding: 32px 0;
  text-align: center;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}
.stats-grid {
  display: flex;
  gap: 10px;
  margin: 12px 0 4px;
}
.stats-cell {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 16px 8px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
}
.stats-num {
  font-size: 24px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-1);
  font-variant-numeric: tabular-nums;
}
.stats-label {
  font-size: 11.5px;
  font-weight: 400;
  letter-spacing: 2px;
  color: var(--text-2);
}
.stats-sub {
  font-size: 10.5px;
  font-weight: 300;
  color: var(--text-3);
}
.stats-top-title {
  margin: 16px 0 8px;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-3);
}
/* GAP 110：近 7 天每日时长柱状图（纯 CSS div 高度） */
.stats-chart {
  display: flex;
  align-items: flex-end;
  gap: 10px;
  height: 110px;
  margin: 14px 0 2px;
  padding: 0 4px;
  border-bottom: 1px solid var(--border);
}
.chart-col {
  flex: 1;
  min-width: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}
.chart-bar {
  width: 100%;
  max-width: 24px;
  min-height: 2px;
  border-radius: 3px 3px 0 0;
  background: var(--accent);
  opacity: 0.85;
  transition: height 0.25s ease;
}
.chart-bar.zero {
  background: var(--border-strong);
  opacity: 0.55;
}
.chart-label {
  font-size: 10.5px;
  font-weight: 300;
  color: var(--text-3);
  letter-spacing: 0.5px;
  font-variant-numeric: tabular-nums;
}
.chart-tip {
  margin-top: 8px;
}
.stats-top-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.stats-top-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
}
.stats-rank {
  flex-shrink: 0;
  width: 18px;
  text-align: center;
  font-size: 11px;
  font-weight: 400;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.stats-book {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.stats-val {
  flex-shrink: 0;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.stats-msg {
  margin: 0;
  padding: 24px 0;
  text-align: center;
  font-size: 12.5px;
  font-weight: 300;
  color: var(--text-3);
}
.stats-msg.error {
  color: #cf4444;
}

/* ================= 响应式 ================= */
@media (max-width: 720px) {
  .topbar {
    flex-wrap: wrap;
    gap: 12px;
    padding: 12px 16px;
  }
  .user-area {
    margin-left: 0;
    overflow-x: auto;
    max-width: 100%;
    scrollbar-width: none;
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
  .card {
    padding: 18px 16px;
  }
  .keys-table {
    width: 100%;
    white-space: normal;
    table-layout: fixed;
  }
  .keys-table .keys-scope,
  .keys-table .keys-key {
    width: auto;
    white-space: normal;
  }
  .keys-table th,
  .keys-table td {
    word-break: break-word;
  }
  .row-hint {
    display: none;
  }
}
.tg-link {
  color: var(--text-2, #888);
  text-decoration: none;
  font-size: 12px;
  font-weight: 300;
  transition: color 0.2s ease;
}
.tg-link:hover {
  color: var(--accent, #4f46e5);
}
</style>
