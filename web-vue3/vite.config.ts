import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 预览只代理本机 Java/Kotlin 服务；不接 Rust 的 8084 端口。
const backend = process.env.READER_BACKEND_URL || 'http://127.0.0.1:8080'
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/reader3': {
        target: backend,
        changeOrigin: true,
      },
      '/assets': {
        target: backend,
        changeOrigin: true,
      },
    },
  },
  build: {
    chunkSizeWarningLimit: 1500,
    assetsDir: 'static',
  },
})
