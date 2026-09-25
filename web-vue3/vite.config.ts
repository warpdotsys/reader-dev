import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 预览只代理本机 Java/Kotlin 服务；不接 Rust 的 8084 端口。
const backend = process.env.READER_BACKEND_URL || 'http://127.0.0.1:8080'
const uiBase = process.env.READER_UI_BASE || '/'
if (!uiBase.startsWith('/') || !uiBase.endsWith('/') || uiBase.includes('..')) {
  throw new Error('READER_UI_BASE must be an absolute path ending in /, such as / or /reader/')
}
const backendProxy = {
  '/reader3': { target: backend, changeOrigin: true },
  '/assets': { target: backend, changeOrigin: true },
}
export default defineConfig({
  base: uiBase,
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5173,
    proxy: backendProxy,
  },
  preview: {
    port: 4173,
    proxy: backendProxy,
  },
  build: {
    chunkSizeWarningLimit: 1500,
    assetsDir: 'static',
  },
})
