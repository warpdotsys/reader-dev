<script setup lang="ts">
import { useRouter } from 'vue-router'
import { t } from '@/utils/i18n'

const router = useRouter()
</script>

<template>
  <div class="nf-page">
    <p class="nf-code">404</p>
    <p class="nf-text">{{ t('notFound.text') }}</p>
    <button class="nf-btn" type="button" @click="router.replace('/')">{{ t('notFound.back') }}</button>
  </div>
</template>

<style scoped>
/* GAP 128：路由兜底 404 页（极简：细字 + 返回书架） */
.nf-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 18px;
  padding: 24px;
  background: var(--bg, #fafafa);
  animation: fade-in 0.2s ease both;
}
.nf-code {
  margin: 0;
  font-size: 52px;
  font-weight: 200;
  letter-spacing: 6px;
  color: var(--text-3, #a1a1aa);
}
.nf-text {
  margin: 0;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 2px;
  color: var(--text-2, #52525b);
}
.nf-btn {
  margin-top: 10px;
  padding: 9px 34px;
  border: 1px solid var(--accent, #4f46e5);
  border-radius: var(--radius, 8px);
  background: none;
  color: var(--accent, #4f46e5);
  font-family: inherit;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 3px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    background-color 0.2s ease;
}
.nf-btn:hover {
  color: var(--accent-deep, #4338ca);
  background: var(--accent-soft, rgba(79, 70, 229, 0.08));
}
</style>
