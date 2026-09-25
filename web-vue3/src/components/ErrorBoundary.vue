<script setup lang="ts">
/**
 * 子组件错误边界（GAP 69）：捕获子树渲染/生命周期错误（onErrorCaptured），
 * 出错时展示「页面出错了 + 重新加载」提示页，避免整站白屏。
 * 用法：<ErrorBoundary><router-view … /></ErrorBoundary>
 */
import { onErrorCaptured, ref } from 'vue'

const hasError = ref(false)
const errorMsg = ref('')

onErrorCaptured((err, instance, info) => {
  hasError.value = true
  errorMsg.value = err instanceof Error ? err.message : String(err)
  // 上报到全局 errorHandler（App.vue 注册）
  if (typeof window !== 'undefined' && window.console && window.console.error) {
    window.console.error('[ErrorBoundary]', err, info, instance)
  }
  // 阻止错误继续向上冒泡（避免全局 handler 重复提示）
  return false
})

function reload() {
  window.location.reload()
}

function reset() {
  hasError.value = false
  errorMsg.value = ''
}
</script>

<template>
  <!-- 捕获到渲染错误：显示重载提示页 -->
  <div v-if="hasError" class="error-boundary">
    <p class="eb-title">页面出错了</p>
    <p class="eb-msg">渲染时发生异常，请重新加载页面重试</p>
    <p v-if="errorMsg" class="eb-detail">{{ errorMsg }}</p>
    <div class="eb-actions">
      <button class="eb-btn primary" type="button" @click="reload">重新加载</button>
      <button class="eb-btn" type="button" @click="reset">重试渲染</button>
    </div>
  </div>
  <!-- 正常渲染插槽内容 -->
  <slot v-else />
</template>

<style scoped>
.error-boundary {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 24px;
  background: var(--bg, #fafafa);
}
.eb-title {
  margin: 0;
  font-size: 18px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--text-1, #18181b);
}
.eb-msg {
  margin: 0;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-2, #52525b);
}
.eb-detail {
  margin: 0;
  max-width: 480px;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3, #a1a1aa);
  word-break: break-all;
  text-align: center;
}
.eb-actions {
  display: flex;
  gap: 12px;
  margin-top: 14px;
}
.eb-btn {
  padding: 9px 26px;
  border: 1px solid var(--border-strong, #d4d4d8);
  border-radius: 8px;
  background: none;
  color: var(--text-2, #52525b);
  font-family: inherit;
  font-size: 13px;
  letter-spacing: 2px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.eb-btn:hover {
  color: var(--accent, #4f46e5);
  border-color: var(--accent, #4f46e5);
}
.eb-btn.primary {
  border-color: var(--accent, #4f46e5);
  background: var(--accent, #4f46e5);
  color: var(--on-accent, #fff);
}
.eb-btn.primary:hover {
  background: var(--accent-deep, #4338ca);
  border-color: var(--accent-deep, #4338ca);
}
</style>
