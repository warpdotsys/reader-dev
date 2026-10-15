<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useUserStore } from '@/stores/user'
import { claimTrialLicense, installLicense, loadLicense, sendTrialCode, type ReaderLicense } from '@/api/license'

const store = useUserStore()
const license = ref<ReaderLicense | null>(null)
const email = ref('')
const code = ref('')
const content = ref('')
const managerKey = ref('')
const busy = ref(false)
const message = ref('')
const failed = ref(false)
let generation = 0
let request: AbortController | undefined

function clearTransient() {
  generation++
  request?.abort()
  request = undefined
  busy.value = false
  email.value = ''; code.value = ''; content.value = ''; managerKey.value = ''
  message.value = ''; license.value = null
}

async function run(action: 'load' | 'send' | 'claim' | 'install') {
  if (busy.value) return
  const ticket = ++generation
  const revision = store.sessionRevision
  const controller = new AbortController()
  request = controller
  const timeout = window.setTimeout(() => controller.abort(), 15000)
  const current = () => ticket === generation && revision === store.sessionRevision
  busy.value = true; message.value = ''; failed.value = false
  try {
    if (action === 'load') {
      const result = await loadLicense(controller.signal)
      if (current()) license.value = result
    } else if (action === 'send') {
      await sendTrialCode(email.value, controller.signal)
      if (current()) message.value = '验证码已交给邮件服务，请检查邮箱；未收到时请联系管理员核查 SMTP。'
    } else if (action === 'claim') {
      const key = await claimTrialLicense(email.value, code.value, controller.signal)
      if (current()) { content.value = key; code.value = ''; message.value = '试用密钥已领取，尚未安装。请输入管理密码并点击导入。' }
    } else {
      const result = await installLicense(content.value, managerKey.value, controller.signal)
      if (current()) { license.value = result; content.value = ''; message.value = '许可证已导入，服务端已确认。' }
    }
  } catch (error) {
    if (current()) {
      failed.value = true
      message.value = controller.signal.aborted ? '请求超时或已取消；结果未确认，请刷新状态后再操作。'
        : error instanceof Error ? error.message : '许可证操作失败'
    }
  } finally {
    window.clearTimeout(timeout)
    if (current()) { busy.value = false; request = undefined; managerKey.value = '' }
  }
}

function expiry(value: number) { return value === 0 ? '不限期' : new Date(value).toLocaleString('zh-CN') }
watch(() => store.sessionRevision, () => { clearTransient(); void run('load') })
onMounted(() => { void run('load') })
onBeforeUnmount(clearTransient)
</script>

<template>
  <section class="license-panel" aria-labelledby="license-title" data-testid="license-panel">
    <h2 id="license-title">许可证与试用邮件</h2>
    <p>许可证影响整台 Reader，而非当前书架。导入需要管理密码；不会自动领取或安装。</p>
    <dl v-if="license">
      <dt>类型</dt><dd>{{ license.type === 'trial' ? '试用' : license.type === 'default' ? '默认' : license.type }}</dd>
      <dt>域名 / 用户上限</dt><dd>{{ license.host }} / {{ license.userMaxLimit }}</dd>
      <dt>许可证到期</dt><dd>{{ expiry(license.expiredAt) }}</dd>
      <dt>简洁网页授权到期</dt><dd>{{ expiry(license.simpleWebExpiredAt) }}</dd>
    </dl>
    <button type="button" :disabled="busy" @click="run('load')">刷新许可证状态</button>
    <form @submit.prevent="run('send')">
      <label>试用邮箱<input v-model="email" type="email" maxlength="254" autocomplete="off" :disabled="busy" required /></label>
      <button type="submit" :disabled="busy">发送验证码</button>
    </form>
    <form @submit.prevent="run('claim')">
      <label>邮箱验证码<input v-model="code" inputmode="numeric" maxlength="8" autocomplete="one-time-code" :disabled="busy" required /></label>
      <button type="submit" :disabled="busy">领取试用密钥</button>
    </form>
    <form @submit.prevent="run('install')">
      <label>许可证密钥<textarea v-model="content" maxlength="32768" rows="3" autocomplete="off" :disabled="busy" required /></label>
      <label>管理密码（仅本次请求）<input v-model="managerKey" type="password" maxlength="4096" autocomplete="off" :disabled="busy" required /></label>
      <button type="submit" :disabled="busy">导入许可证</button>
    </form>
    <p v-if="message" role="status" aria-live="polite" :class="{ error: failed }">{{ message }}</p>
    <p>邮箱、验证码和密钥仅保留在本页内存；离开页面或切换账号会清空。邮件发送成功不等于已送达收件箱。</p>
  </section>
</template>

<style scoped>
.license-panel { padding: 24px; margin-bottom: 20px; border: 1px solid var(--border); border-radius: 12px; background: var(--bg); }
h2 { font-size: 17px; margin: 0 0 14px; }
p, dl { font-size: 13px; line-height: 1.7; overflow-wrap: anywhere; }
dl { display: grid; grid-template-columns: minmax(110px, 1fr) 2fr; gap: 8px; }
dd { margin: 0; } form { display: flex; flex-wrap: wrap; gap: 10px; align-items: end; margin-top: 14px; }
label { display: flex; flex-direction: column; flex: 1 1 220px; gap: 6px; font-size: 13px; }
input, textarea { box-sizing: border-box; width: 100%; padding: 9px; border: 1px solid var(--border); border-radius: 6px; color: var(--text-1); background: var(--bg); font: inherit; }
button { padding: 9px 13px; border: 1px solid var(--border); border-radius: 6px; color: var(--text-1); background: var(--bg); cursor: pointer; }
button:disabled { opacity: .5; cursor: wait; }.error { color: #c44040; }
</style>
