import { defineStore } from 'pinia'
import { ref } from 'vue'
import { discardManagerCredential, managerCredentialScope, readManagerCredential, readerDeployment } from '../utils/managerCredential.ts'

const TOKEN_KEY = 'reader_access_token'
const USERNAME_KEY = 'reader_username'
const REMEMBER_KEY = 'reader_remember'
const ADMIN_KEY = 'reader_is_admin'
const DEFAULT_CONFIG_MODE_KEY = 'reader_default_config_mode'
const TAB_STATE_KEY = 'reader_tab_session_v1'

function savedPair(storage: Storage): { token: string; username: string } {
  return { token: storage.getItem(TOKEN_KEY) || '', username: storage.getItem(USERNAME_KEY) || '' }
}

function ownsSavedPair(storage: Storage, token: string, username: string): boolean {
  const pair = savedPair(storage)
  return !!token && !!username && pair.token === token && pair.username === username
}

function readConfigMode(token: string, username: string): boolean {
  try {
    const tabMode = sessionStorage.getItem(DEFAULT_CONFIG_MODE_KEY)
    if (tabMode !== null) return tabMode === '1'
    return ownsSavedPair(localStorage, token, username) && localStorage.getItem(DEFAULT_CONFIG_MODE_KEY) === '1'
  } catch { return false }
}

/** 勾选「记住我」时 token 在 localStorage，否则仅存本标签页。 */
function readSession(): { token: string; username: string } {
  try {
    // 显式退出的标签刷新不能自动接管另一标签的“记住我”。
    if (sessionStorage.getItem(TAB_STATE_KEY) === 'signed-out') return { token: '', username: '' }
    const tab = savedPair(sessionStorage)
    if (tab.token && tab.username) return tab
    // 不拼接两个存储中的残缺身份，也不把损坏的本标签身份换成其他账号。
    if (tab.token || tab.username || sessionStorage.getItem(TAB_STATE_KEY) === 'active') return { token: '', username: '' }
    const saved = savedPair(localStorage)
    if (!saved.token || !saved.username) return { token: '', username: '' }
    // 记住我负责新标签初始化；运行中的标签固定完整身份，刷新不随共享存储换号。
    sessionStorage.setItem(TOKEN_KEY, saved.token)
    sessionStorage.setItem(USERNAME_KEY, saved.username)
    sessionStorage.setItem(TAB_STATE_KEY, 'active')
    return saved
  } catch { return { token: '', username: '' } }
}

export const useUserStore = defineStore('user', () => {
  const init = readSession()
  const accessToken = ref(init.token)
  const username = ref(init.username)
  // legacy 没有持久管理员角色：只有真实管理密码验证成功才授予本标签页管理能力。
  const isAdmin = ref(false)
  const sessionRevision = ref(0)
  let restoreConfigMode = !!readManagerCredential(managerCredentialScope(
    { accessToken: init.token, username: init.username }, readerDeployment())) &&
    readConfigMode(init.token, init.username)
  const defaultConfigMode = ref(false)

  function clearSavedSession() {
    // 只删除本标签实际持有的共享凭据；另一标签保存的新账号/新 token 保留。
    for (const storage of [sessionStorage, localStorage]) {
      try {
        if (storage === sessionStorage || ownsSavedPair(storage, accessToken.value, username.value)) {
          for (const key of [TOKEN_KEY, USERNAME_KEY, ADMIN_KEY, DEFAULT_CONFIG_MODE_KEY]) storage.removeItem(key)
        }
      } catch { /* 存储不可用时仍可清除内存身份 */ }
    }
  }

  function setSession(token: string, name: string, remember = true) {
    clearSavedSession()
    discardManagerCredential()
    restoreConfigMode = false
    sessionRevision.value++
    accessToken.value = token
    username.value = name
    isAdmin.value = false
    defaultConfigMode.value = false
    try {
      sessionStorage.setItem(TOKEN_KEY, token)
      sessionStorage.setItem(USERNAME_KEY, name)
      sessionStorage.setItem(TAB_STATE_KEY, 'active')
      if (remember) {
        localStorage.setItem(TOKEN_KEY, token)
        localStorage.setItem(USERNAME_KEY, name)
      }
      localStorage.setItem(REMEMBER_KEY, remember ? '1' : '0')
    } catch { /* 存储不可用时仅内存会话 */ }
  }

  function clear() {
    clearSavedSession()
    discardManagerCredential()
    restoreConfigMode = false
    sessionRevision.value++
    accessToken.value = ''
    username.value = ''
    isAdmin.value = false
    defaultConfigMode.value = false
    try { sessionStorage.setItem(TAB_STATE_KEY, 'signed-out') } catch { /* unavailable */ }
  }

  function saveConfigMode() {
    // 管理凭据和配置空间都是标签局部状态，不能覆盖另一账号的共享设置。
    try { sessionStorage.setItem(DEFAULT_CONFIG_MODE_KEY, defaultConfigMode.value ? '1' : '0') }
    catch { /* 存储不可用时仅内存会话 */ }
  }

  function toggleDefaultConfigMode() {
    if (!isAdmin.value) return
    sessionRevision.value++
    defaultConfigMode.value = !defaultConfigMode.value
    saveConfigMode()
  }

  /** 更新真实管理密码验证结果；撤销能力时同步退出系统配置空间。 */
  function updateAdminStatus(verified: boolean) {
    const wasDefault = isAdmin.value && defaultConfigMode.value
    defaultConfigMode.value = verified && (defaultConfigMode.value || restoreConfigMode)
    restoreConfigMode = false
    if (wasDefault !== defaultConfigMode.value) sessionRevision.value++
    isAdmin.value = verified
    if (ownsSavedPair(localStorage, accessToken.value, username.value)) localStorage.removeItem(ADMIN_KEY)
    sessionStorage.removeItem(ADMIN_KEY)
    saveConfigMode()
  }

  return { accessToken, username, isAdmin, defaultConfigMode, sessionRevision,
    setSession, clear, toggleDefaultConfigMode, updateAdminStatus }
})
