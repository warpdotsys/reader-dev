import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 后端默认监听 8080（READER_SERVER_PORT 可覆盖），开发期代理 /reader3
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
        target: 'http://localhost:8084',
        changeOrigin: true,
      },
      '/assets': {
        target: 'http://localhost:8084',
        changeOrigin: true,
      },
    },
  },
  build: {
    chunkSizeWarningLimit: 1500,
    assetsDir: 'static',
  },
})
