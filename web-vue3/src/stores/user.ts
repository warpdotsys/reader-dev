import { defineStore } from 'pinia'
import { ref } from 'vue'
import { discardManagerCredential, managerCredentialScope, readManagerCredential, readerDeployment } from '../utils/managerCredential.ts'

const TOKEN_KEY = 'reader_access_token'
const USERNAME_KEY = 'reader_username'
const REMEMBER_KEY = 'reader_remember'
const ADMIN_KEY = 'reader_is_admin'
const DEFAULT_CONFIG_MODE_KEY = 'reader_default_config_mode'

/** 勾选「记住我」时 token 在 localStorage，否则仅存本标签页。 */
function readSession(): { token: string; username: string } {
  return {
    token: localStorage.getItem(TOKEN_KEY) || sessionStorage.getItem(TOKEN_KEY) || '',
    username: localStorage.getItem(USERNAME_KEY) || sessionStorage.getItem(USERNAME_KEY) || '',
  }
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
    (localStorage.getItem(DEFAULT_CONFIG_MODE_KEY) === '1' || sessionStorage.getItem(DEFAULT_CONFIG_MODE_KEY) === '1')
  const defaultConfigMode = ref(false)

  function clearSavedSession() {
    for (const storage of [localStorage, sessionStorage]) {
      for (const key of [TOKEN_KEY, USERNAME_KEY, ADMIN_KEY, DEFAULT_CONFIG_MODE_KEY]) storage.removeItem(key)
    }
  }

  function setSession(token: string, name: string, remember = true) {
    discardManagerCredential()
    restoreConfigMode = false
    sessionRevision.value++
    accessToken.value = token
    username.value = name
    isAdmin.value = false
    defaultConfigMode.value = false
    clearSavedSession()
    try {
      const storage = remember ? localStorage : sessionStorage
      storage.setItem(TOKEN_KEY, token)
      storage.setItem(USERNAME_KEY, name)
      localStorage.setItem(REMEMBER_KEY, remember ? '1' : '0')
    } catch { /* 存储不可用时仅内存会话 */ }
  }

  function clear() {
    discardManagerCredential()
    restoreConfigMode = false
    sessionRevision.value++
    accessToken.value = ''
    username.value = ''
    isAdmin.value = false
    defaultConfigMode.value = false
    clearSavedSession()
  }

  function saveConfigMode() {
    localStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    sessionStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    const storage = localStorage.getItem(TOKEN_KEY) ? localStorage : sessionStorage
    try { storage.setItem(DEFAULT_CONFIG_MODE_KEY, defaultConfigMode.value ? '1' : '0') }
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
    localStorage.removeItem(ADMIN_KEY)
    sessionStorage.removeItem(ADMIN_KEY)
    saveConfigMode()
  }

  return { accessToken, username, isAdmin, defaultConfigMode, sessionRevision,
    setSession, clear, toggleDefaultConfigMode, updateAdminStatus }
})
