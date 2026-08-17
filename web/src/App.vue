<script setup lang="ts">
import { getCurrentInstance, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { applyUiTheme, loadUiTheme } from '@/utils/uiTheme'
import { applyCustomCss } from '@/utils/customCss'
import { applyDocLang, lang, t } from '@/utils/i18n'
import ErrorBoundary from '@/components/ErrorBoundary.vue'
import CommandPalette from '@/components/CommandPalette.vue'

// GAP 69 全局错误处理：未被子组件捕获的渲染/生命周期/异步错误统一记录
// （子组件渲染错误由 ErrorBoundary 的 onErrorCaptured 拦截并展示重载页，
//   此处兜底记录 + 控制台可见，不打断用户操作）
const app = getCurrentInstance()?.appContext.app
if (app) {
  app.config.errorHandler = (err, instance, info) => {
    // eslint-disable-next-line no-console
    console.error('[global-error]', err, info, instance)
    // 渲染期错误：交给 ErrorBoundary 展示（errorCaptured 已 return false 时不会到这儿）
  }
}

onMounted(() => {
  // 界面主题（浅色/深色/跟随系统）：进入即恢复，并监听系统深色偏好（system 时自动切换）
  applyUiTheme(loadUiTheme())
  // GAP 5：自定义样式注入（reader_custom_css → 全局 <style>，阅读器/界面均可覆盖）
  applyCustomCss()
  // i18n：<html lang> + 当前路由标题（语言切换时由下方 watch 重算）
  applyDocLang()
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  const onSystemChange = () => {
    if (loadUiTheme() === 'system') applyUiTheme('system')
  }
  mq.addEventListener('change', onSystemChange)
})

// i18n：语言切换后重算 <html lang> 与当前路由标题
const route = useRoute()
watch(lang, () => {
  applyDocLang()
  const meta = route.meta
  const title = t(String(meta.titleKey ?? meta.title ?? ''))
  document.title = `${title} · ${t('brand.name')}`
})
</script>

<template>
  <ErrorBoundary>
    <router-view v-slot="{ Component }">
      <transition name="page" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </ErrorBoundary>
  <!-- GAP：全局命令面板（Ctrl+K——任意页可用） -->
  <CommandPalette />
</template>
