import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'

import App from './App.vue'
import router from './router'
import { lazy } from './directives/lazy'
import './styles/main.css'

// 主题由阅读页顶部按钮切换（html[data-theme=light|dark|paper]，见 styles/main.css）
// 旧 html.dark hack（强制 dark class + main.css 反向重映射）已清理

// GAP 75：内置网络字体加载失败提示（@font-face 已有 font-display: swap 兜底——
// 失败时浏览器回退系统字体，此处仅 console.warn 标注，便于排查）
if (typeof document !== 'undefined' && 'fonts' in document) {
  const FONT_FAMILIES = ['LXGW WenKai', 'Source Han Serif CN']
  for (const family of FONT_FAMILIES) {
    document.fonts
      .load(`16px "${family}"`, '永州之野产异蛇，黑质而白章')
      .then(
        (loaded) => {
          if (!loaded || loaded.length === 0) {
            console.warn(`[fonts] "${family}" 加载失败（font-display: swap 已回退系统字体）`)
          }
        },
        () => {
          console.warn(`[fonts] "${family}" 加载失败（font-display: swap 已回退系统字体）`)
        },
      )
  }
}

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(ElementPlus)
app.directive('lazy', lazy)
app.mount('#app')

// PWA：Service Worker 注册——仅生产模式（开发期热更新会与 SW 缓存互相干扰）；
// sw.js 为 ES Module（导出纯函数供 node 单测，M5）——须以 { type: 'module' } 注册
if (import.meta.env.PROD && 'serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker
      .register('/sw.js', { type: 'module' })
      .then((reg) => {
        // legacy updateForce + SKIP_WAITING：新版本 SW 安装完成后立即接管并刷新页面
        reg.addEventListener('updatefound', () => {
          const worker = reg.installing
          if (!worker) return
          worker.addEventListener('statechange', () => {
            if (worker.state === 'installed' && navigator.serviceWorker.controller) {
              worker.postMessage({ type: 'SKIP_WAITING' })
            }
          })
        })
      })
      .catch((err) => {
        // 注册失败不阻断应用（如非 https/localhost 环境）
        console.warn('[sw] register failed:', err)
      })
  })
  let swReloading = false
  navigator.serviceWorker.addEventListener('controllerchange', () => {
    if (swReloading) return
    swReloading = true
    window.location.reload()
  })
}
