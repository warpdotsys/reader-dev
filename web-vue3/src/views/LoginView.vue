<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { login as loginApi } from '@/api/auth'
import { useUserStore } from '@/stores/user'
import { t } from '@/utils/i18n'

const router = useRouter()
const route = useRoute()
const store = useUserStore()

const mode = ref<'login' | 'register'>('login')
const loading = ref(false)
const form = reactive({ username: '', password: '', code: '' })
/** GAP 150：记住我——不勾选时 token 存 sessionStorage（关闭标签页即登出） */
const remember = ref(localStorage.getItem('reader_remember') !== '0')

function switchMode(m: 'login' | 'register') {
  mode.value = m
}

async function submit() {
  const { username, password } = form
  if (!username.trim() || !password) {
    ElMessage.warning(t('login.needBoth'))
    return
  }
  if (mode.value === 'register' && username.trim().length < 5) {
    ElMessage.warning(t('login.usernameTooShort'))
    return
  }
  if (mode.value === 'register' && password.length < 8) {
    ElMessage.warning(t('login.passwordTooShort'))
    return
  }

  loading.value = true
  try {
    const res = await loginApi({
      username: username.trim(),
      password,
      isLogin: mode.value === 'login',
      // GAP 90：注册模式携带邀请码（后端 register 校验 code 参数）
      code: mode.value === 'register' && form.code.trim() ? form.code.trim() : undefined,
    })
    store.setSession(res.data.accessToken, res.data.username, remember.value)
    ElMessage.success(mode.value === 'login' ? t('login.welcomeBack') : t('login.registered'))
    // GAP 127：登录成功回跳 redirect query（仅限站内路径，防开放重定向）
    const q = route.query.redirect
    const redirect =
      typeof q === 'string' && q.startsWith('/') && !q.startsWith('//') ? q : '/'
    await router.replace(redirect)
  } catch {
    // 错误提示已由 axios 拦截器统一处理
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="login-page">
    <main class="login-panel">
      <!-- 徽标 + 字标 -->
      <div class="wordmark">
        <img class="login-logo" src="/logo.svg" :alt="t('brand.name')" />
        <h1 class="wordmark-text">{{ t('brand.name') }}<span class="wordmark-dot">.</span></h1>
        <p class="wordmark-sub">READER</p>
      </div>

      <!-- 登录 / 注册 切换（细字 + 下划线指示） -->
      <div class="mode-switch" role="tablist">
        <button
          type="button"
          :class="{ active: mode === 'login' }"
          @click="switchMode('login')"
        >
          {{ t('login.title') }}
        </button>
        <button
          type="button"
          :class="{ active: mode === 'register' }"
          @click="switchMode('register')"
        >
          {{ t('login.register') }}
        </button>
      </div>

      <!-- 下划线风格表单 -->
      <form class="login-form" @submit.prevent="submit">
        <label class="field">
          <span class="field-label">{{ t('login.username') }}</span>
          <input
            v-model="form.username"
            class="field-input"
            type="text"
            :placeholder="t('login.placeholder.username')"
            maxlength="32"
            autocomplete="username"
            spellcheck="false"
          />
        </label>

        <label class="field">
          <span class="field-label">{{ t('login.password') }}</span>
          <input
            v-model="form.password"
            class="field-input"
            type="password"
            :placeholder="t('login.placeholder.password')"
            maxlength="64"
            autocomplete="current-password"
          />
        </label>

        <!-- GAP 90：注册模式邀请码（后端开启邀请注册时必填；未开启可留空） -->
        <label v-if="mode === 'register'" class="field">
          <span class="field-label">{{ t('login.inviteCode') }}</span>
          <input
            v-model="form.code"
            class="field-input"
            type="text"
            :placeholder="t('login.placeholder.code')"
            maxlength="64"
            autocomplete="off"
            spellcheck="false"
          />
        </label>

        <!-- GAP 150：记住我（不勾选 → sessionStorage 存 token，关闭标签页即登出） -->
        <label class="remember-row">
          <input v-model="remember" class="remember-box" type="checkbox" />
          <span class="remember-label">{{ t('login.remember') }}</span>
        </label>

        <button class="submit-btn" type="submit" :disabled="loading">
          <span v-if="loading" class="btn-spinner" aria-hidden="true"></span>
          <span v-else>{{ mode === 'login' ? t('login.submit') : t('login.registerSubmit') }}</span>
        </button>
      </form>

      <p class="login-foot">
        {{ mode === 'login' ? t('login.noAccount') : t('login.hasAccount') }}
        <button
          type="button"
          class="link-btn"
          @click="switchMode(mode === 'login' ? 'register' : 'login')"
        >
          {{ mode === 'login' ? t('login.goRegister') : t('login.goLogin') }}
        </button>
        <a class="tg-foot" href="https://t.me/readerdev" target="_blank" rel="noopener">{{ t('login.tgGroup') }}</a>
      </p>
    </main>

    <footer class="login-footer">reader-dev · /reader3</footer>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  animation: fade-in 0.2s ease both;
}

.login-panel {
  width: min(340px, 100%);
  padding: 56px 8px 40px;
}

/* ---------- 字标 ---------- */
.wordmark {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 48px;
}
.wordmark-text {
  margin: 0;
  font-size: 34px;
  font-weight: 300;
  letter-spacing: 10px;
  text-indent: 10px; /* 抵消末字后的字距，视觉居中 */
  color: var(--text-1);
}
.wordmark-dot {
  color: var(--accent);
  font-weight: 400;
}
.wordmark-sub {
  margin: 10px 0 0;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 6px;
  text-indent: 6px;
  color: var(--text-3);
}

/* ---------- 模式切换（细字 + 下划线） ---------- */
.mode-switch {
  display: flex;
  justify-content: center;
  gap: 36px;
  margin-bottom: 44px;
}
.mode-switch button {
  padding: 2px 0 8px;
  border: none;
  border-bottom: 1px solid transparent;
  background: none;
  color: var(--text-3);
  font-size: 14px;
  font-weight: 400;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.mode-switch button:hover {
  color: var(--text-2);
}
.mode-switch button.active {
  color: var(--text-1);
  border-bottom-color: var(--accent);
}

/* ---------- 表单（下划线输入） ---------- */
.login-form {
  display: flex;
  flex-direction: column;
  gap: 30px;
}
.field {
  display: block;
}
.field-label {
  display: block;
  margin-bottom: 8px;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--text-2);
  letter-spacing: 1px;
}
.field-input {
  width: 100%;
  padding: 9px 2px;
  border: none;
  border-bottom: 1px solid var(--border);
  border-radius: 0;
  background: transparent;
  color: var(--text-1);
  font-family: inherit;
  font-size: 15px;
  font-weight: 400;
  outline: none;
  transition: border-color 0.2s ease;
}
.field-input::placeholder {
  color: var(--text-3);
  font-weight: 300;
}
.field-input:focus {
  border-bottom-color: var(--accent);
}

/* ---------- 记住我（GAP 150） ---------- */
.remember-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: -12px;
  cursor: pointer;
}
.remember-box {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}
.remember-label {
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 0.5px;
  color: var(--text-3);
  cursor: pointer;
  user-select: none;
}

/* ---------- 提交按钮（纯色无渐变） ---------- */
.submit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  width: 100%;
  height: 44px;
  margin-top: 8px;
  border: none;
  border-radius: var(--radius);
  background: var(--accent);
  color: var(--on-accent);
  font-family: inherit;
  font-size: 14.5px;
  font-weight: 400;
  letter-spacing: 6px;
  cursor: pointer;
  transition: background 0.2s ease;
}
.submit-btn:hover:not(:disabled) {
  background: var(--accent-deep);
}
.submit-btn:active:not(:disabled) {
  background: var(--accent-deep);
}
.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-spinner {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid color-mix(in srgb, var(--on-accent) 35%, transparent);
  border-top-color: var(--on-accent);
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* ---------- 底部 ---------- */
.login-foot {
  margin: 36px 0 0;
  text-align: center;
  font-size: 13px;
  color: var(--text-3);
}
.link-btn {
  border: none;
  background: none;
  padding: 0;
  color: var(--accent);
  font-family: inherit;
  font-size: 13px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.link-btn:hover {
  color: var(--accent-deep);
}

.login-footer {
  position: fixed;
  bottom: 20px;
  left: 0;
  right: 0;
  text-align: center;
  font-size: 11px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-3);
}
.tg-foot {
  display: block;
  margin-top: 18px;
  text-align: center;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 0.5px;
  color: var(--text-3, #aaa);
  text-decoration: none;
  transition: color 0.2s ease;
}
.tg-foot:hover {
  color: var(--accent, #4f46e5);
}
.login-logo {
  width: 76px;
  height: 76px;
  border-radius: 20px;
  margin-bottom: 16px;
  box-shadow: 0 2px 14px rgba(30, 27, 75, 0.2);
}
</style>
