import { defineStore } from 'pinia'
import { ref } from 'vue'

const TOKEN_KEY = 'reader_access_token'
const USERNAME_KEY = 'reader_username'
const REMEMBER_KEY = 'reader_remember'
const ADMIN_KEY = 'reader_is_admin'
const DEFAULT_CONFIG_MODE_KEY = 'reader_default_config_mode'

/** 读取会话：勾选「记住我」时 token 在 localStorage（跨会话），否则在 sessionStorage（关标签页即登出） */
function readSession(): { token: string; username: string; isAdmin: boolean } {
  const token = localStorage.getItem(TOKEN_KEY) || sessionStorage.getItem(TOKEN_KEY) || ''
  const username = localStorage.getItem(USERNAME_KEY) || sessionStorage.getItem(USERNAME_KEY) || ''
  const isAdmin = localStorage.getItem(ADMIN_KEY) === '1' || sessionStorage.getItem(ADMIN_KEY) === '1'
  return { token, username, isAdmin }
}

export const useUserStore = defineStore('user', () => {
  const init = readSession()
  const accessToken = ref(init.token)
  const username = ref(init.username)
  const isAdmin = ref(init.isAdmin)
  /** 管理员手动进入 default（系统配置层）：请求统一带 ns=default */
  const defaultConfigMode = ref(
    localStorage.getItem(DEFAULT_CONFIG_MODE_KEY) === '1' ||
      sessionStorage.getItem(DEFAULT_CONFIG_MODE_KEY) === '1',
  )

  /** GAP 150：remember=false 时 token 只写 sessionStorage（关闭标签页即登出） */
  function setSession(token: string, name: string, remember = true, admin = false) {
    accessToken.value = token
    username.value = name
    isAdmin.value = admin
    defaultConfigMode.value = false
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(USERNAME_KEY)
    localStorage.removeItem(ADMIN_KEY)
    localStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    sessionStorage.removeItem(TOKEN_KEY)
    sessionStorage.removeItem(USERNAME_KEY)
    sessionStorage.removeItem(ADMIN_KEY)
    sessionStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    const store = remember ? localStorage : sessionStorage
    try {
      store.setItem(TOKEN_KEY, token)
      store.setItem(USERNAME_KEY, name)
      store.setItem(ADMIN_KEY, admin ? '1' : '0')
      localStorage.setItem(REMEMBER_KEY, remember ? '1' : '0')
    } catch {
      /* 存储不可用时仅内存会话 */
    }
  }

  function clear() {
    accessToken.value = ''
    username.value = ''
    defaultConfigMode.value = false
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(USERNAME_KEY)
    localStorage.removeItem(ADMIN_KEY)
    localStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    sessionStorage.removeItem(TOKEN_KEY)
    sessionStorage.removeItem(USERNAME_KEY)
    sessionStorage.removeItem(ADMIN_KEY)
    sessionStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
  }

  function toggleDefaultConfigMode() {
    defaultConfigMode.value = !defaultConfigMode.value
    localStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    sessionStorage.removeItem(DEFAULT_CONFIG_MODE_KEY)
    const store = localStorage.getItem(TOKEN_KEY) ? localStorage : sessionStorage
    try {
      store.setItem(DEFAULT_CONFIG_MODE_KEY, defaultConfigMode.value ? '1' : '0')
    } catch {
      /* 存储不可用时仅内存会话 */
    }
  }

  return { accessToken, username, isAdmin, defaultConfigMode, setSession, clear, toggleDefaultConfigMode }
})
