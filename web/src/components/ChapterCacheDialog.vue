<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { getBookContent } from '@/api/books'
import { cacheBookRangeOnServer, cacheBookSSE, cancelCacheBook } from '@/api/cacheBook'
import { saveLocalChapter } from '@/utils/readerLocalCache'
import type { BookChapter } from '@/types'

interface Props {
  modelValue: boolean
  bookUrl: string
  bookName?: string
  /** 目录实章（已过滤卷标题行；顺序与后端 range 0 基一致） */
  chapters: BookChapter[]
  /** 书源 origin（拉取到本机时逐章 getBookContent 需要） */
  origin?: string
  /** 默认起始章（1 基；目录页单章缓存传入） */
  defaultFrom?: number
  /** 默认范围：目录单章 → chapter；阅读页/详情页 → all */
  defaultScope?: 'chapter' | 'rest' | 'all' | 'range'
  /** 服务端方向是否可用（服务端缓存要求书已加入书架） */
  allowServer?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  bookName: '',
  origin: '',
  defaultFrom: 1,
  defaultScope: 'all',
  allowServer: true,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'done', payload: { direction: 'server' | 'local'; from: number; to: number; saved: number }): void
}>()

type CacheDirection = 'server' | 'local'
type CacheScope = 'chapter' | 'rest' | 'all' | 'range'

const direction = ref<CacheDirection>('server')
const scope = ref<CacheScope>('all')
const rangeFrom = ref(1)
const rangeTo = ref(1)
const busy = ref(false)
const finished = ref(false)
const cached = ref(0)
const total = ref(0)
const msg = ref('')
const msgError = ref(false)
let sseHandle: { close: () => void } | null = null
let cancelLocal = false

const chapterCount = computed(() => props.chapters.length)
const title = computed(() => props.bookName || '本书')

const from = computed(() => {
  if (scope.value === 'range') return Math.max(1, Math.min(rangeFrom.value, chapterCount.value || 1))
  if (scope.value === 'chapter' || scope.value === 'rest') {
    return Math.max(1, Math.min(props.defaultFrom, chapterCount.value || 1))
  }
  return 1
})

const to = computed(() => {
  const n = chapterCount.value || 1
  if (scope.value === 'range') return Math.max(from.value, Math.min(rangeTo.value, n))
  if (scope.value === 'chapter') return from.value
  return n
})

const percent = computed(() => {
  if (total.value <= 0) return finished.value ? 100 : 0
  return Math.min(100, Math.round((cached.value / total.value) * 100))
})

const serverDisabled = computed(() => direction.value === 'server' && !props.allowServer)

function reset() {
  direction.value = 'server'
  scope.value = props.defaultScope
  rangeFrom.value = props.defaultFrom
  rangeTo.value = chapterCount.value || 1
  busy.value = false
  finished.value = false
  cached.value = 0
  total.value = 0
  msg.value = ''
  msgError.value = false
  cancelLocal = false
}

function close() {
  if (busy.value) cancel()
  emit('update:modelValue', false)
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      reset()
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = ''
    }
  },
)

onBeforeUnmount(() => {
  if (sseHandle) sseHandle.close()
  document.body.style.overflow = ''
})

function fail(text: string) {
  busy.value = false
  msg.value = text
  msgError.value = true
}

function start() {
  if (busy.value || chapterCount.value === 0) return
  if (serverDisabled.value) {
    ElMessage.warning('服务端缓存需要先把书加入书架')
    return
  }
  busy.value = true
  finished.value = false
  msg.value = ''
  msgError.value = false
  const f = from.value
  const t = to.value
  if (direction.value === 'server') void startServer(f, t)
  else void startLocal(f, t)
}

async function startServer(f: number, t: number) {
  try {
    const res = await cacheBookRangeOnServer(props.bookUrl, f - 1, t - 1)
    if (!res.isSuccess) throw new Error(res.errorMsg || '缓存启动失败')
    const start = res.data
    cached.value = start?.cached ?? 0
    total.value = start?.total ?? t - f + 1
    const taskId = start?.taskId
    const handle = await cacheBookSSE(taskId || props.bookUrl, {
      onProgress: (p) => {
        if (typeof p.cached === 'number') cached.value = p.cached
        if (typeof p.total === 'number') total.value = p.total
        if (p.cancelled) {
          busy.value = false
          msg.value = '缓存已取消'
        } else if (p.error) {
          fail(`缓存失败：${p.error}`)
        } else if (p.finished) {
          busy.value = false
          finished.value = true
          msg.value = `已缓存到服务器（${cached.value}/${total.value} 章）`
          emit('done', { direction: 'server', from: f, to: t, saved: cached.value })
        }
      },
      onEnd: () => {
        if (busy.value && !msg.value) {
          busy.value = false
          finished.value = true
          msg.value = `已缓存到服务器（${cached.value}/${total.value} 章）`
          emit('done', { direction: 'server', from: f, to: t, saved: cached.value })
        }
      },
      onStreamError: (m) => {
        if (busy.value) fail(`缓存进度中断：${m}`)
      },
    }, !!taskId)
    sseHandle = handle
  } catch (err) {
    fail(`缓存失败：${err instanceof Error ? err.message : '请稍后重试'}`)
  }
}

async function startLocal(f: number, t: number) {
  if (!props.origin) {
    fail('缺少书源信息，无法拉取正文')
    return
  }
  cancelLocal = false
  const selected = props.chapters.slice(f - 1, t).map((ch, i) => ({
    ch,
    index: f - 1 + i,
  }))
  total.value = selected.length
  cached.value = 0
  let cursor = 0
  let saved = 0
  const worker = async () => {
    while (!cancelLocal) {
      const item = selected[cursor++]
      if (!item) return
      try {
        const res = await getBookContent(item.ch.url, props.origin || '')
        const text = res.data?.content ?? ''
        if (text) {
          await saveLocalChapter({
            bookUrl: props.bookUrl,
            chapterUrl: item.ch.url,
            title: item.ch.title,
            index: item.index,
            content: text,
          })
          saved++
        }
      } catch {
        // 单章失败不中断整批（与服务器任务口径一致）
      }
      cached.value = saved
    }
  }
  await Promise.all(Array.from({ length: Math.min(3, selected.length) }, () => worker()))
  if (cancelLocal) {
    busy.value = false
    msg.value = '已取消'
    return
  }
  busy.value = false
  finished.value = true
  msg.value = `已保存到本机缓存 ${saved} 章`
  emit('done', { direction: 'local', from: f, to: t, saved })
}

function cancel() {
  if (!busy.value) return
  if (sseHandle) {
    sseHandle.close()
    sseHandle = null
    void cancelCacheBook(props.bookUrl, false).catch(() => {})
  }
  cancelLocal = true
  busy.value = false
  msg.value = '已取消'
  msgError.value = false
}
</script>

<template>
  <Teleport to="body">
    <Transition name="dlg">
      <div v-if="modelValue" class="dlg-overlay" @click.self="close">
        <div class="dlg dlg-cache" role="dialog" aria-modal="true" aria-label="章节缓存" tabindex="-1">
          <div class="dlg-head">
            <h2 class="dlg-title">缓存 · {{ title }}</h2>
            <button class="dlg-close" type="button" title="关闭" @click="close">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                <path d="M6 6l12 12M18 6L6 18" />
              </svg>
            </button>
          </div>

          <div class="cache-field">
            <span class="field-label">方向</span>
            <div class="seg">
              <button
                class="seg-btn"
                :class="{ active: direction === 'server' }"
                type="button"
                :disabled="busy || !allowServer"
                :title="allowServer ? '' : '服务端缓存需要先把书加入书架'"
                @click="direction = 'server'"
              >
                缓存到服务器
              </button>
              <button
                class="seg-btn"
                :class="{ active: direction === 'local' }"
                type="button"
                :disabled="busy || !origin"
                :title="origin ? '' : '缺少书源信息，无法拉取正文'"
                @click="direction = 'local'"
              >
                拉取到本机
              </button>
            </div>
          </div>

          <div class="cache-field">
            <span class="field-label">范围</span>
            <div class="seg">
              <button
                class="seg-btn"
                :class="{ active: scope === 'chapter' }"
                type="button"
                :disabled="busy"
                @click="scope = 'chapter'"
              >
                当前章
              </button>
              <button
                class="seg-btn"
                :class="{ active: scope === 'rest' }"
                type="button"
                :disabled="busy"
                @click="scope = 'rest'"
              >
                至末尾
              </button>
              <button
                class="seg-btn"
                :class="{ active: scope === 'all' }"
                type="button"
                :disabled="busy"
                @click="scope = 'all'"
              >
                全本
              </button>
              <button
                class="seg-btn"
                :class="{ active: scope === 'range' }"
                type="button"
                :disabled="busy"
                @click="scope = 'range'"
              >
                指定范围
              </button>
            </div>
          </div>

          <div v-if="scope === 'range'" class="range-row">
            <input
              v-model.number="rangeFrom"
              class="range-input"
              type="number"
              min="1"
              :max="chapterCount || 1"
              :disabled="busy"
              aria-label="起始章"
            />
            <span class="range-sep">至</span>
            <input
              v-model.number="rangeTo"
              class="range-input"
              type="number"
              min="1"
              :max="chapterCount || 1"
              :disabled="busy"
              aria-label="结束章"
            />
            <span class="range-total">共 {{ chapterCount }} 章</span>
          </div>

          <div v-if="busy || finished" class="cache-progress">
            <div class="cache-bar">
              <div class="cache-fill" :style="{ width: percent + '%' }"></div>
            </div>
            <span class="cache-percent">
              {{ busy ? `${cached} / ${total}` : finished ? '完成' : `${percent}%` }}
            </span>
          </div>
          <p v-if="msg" class="search-msg" :class="{ error: msgError }">{{ msg }}</p>

          <div class="dlg-actions">
            <button v-if="busy" class="ghost-btn" type="button" @click="cancel">取消</button>
            <template v-else>
              <button class="ghost-btn" type="button" @click="close">关闭</button>
              <button
                class="accent-btn"
                type="button"
                :disabled="serverDisabled || chapterCount === 0"
                @click="start"
              >
                {{ direction === 'server' ? '缓存到服务器' : '拉取到本机' }}
              </button>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dlg-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(24, 24, 27, 0.35);
}
.dlg {
  width: min(440px, 100%);
  max-height: min(560px, 86vh);
  display: flex;
  flex-direction: column;
  padding: 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  outline: none;
}
.dlg-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.dlg-title {
  margin: 0;
  font-size: 15px;
  font-weight: 400;
  letter-spacing: 1px;
  color: var(--text-1);
}
.dlg-close {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-3);
  cursor: pointer;
  transition:
    color 0.2s ease,
    background-color 0.2s ease;
}
.dlg-close:hover {
  color: var(--text-1);
  background: var(--hover);
}
.dlg-close svg {
  width: 13px;
  height: 13px;
}
.cache-field {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}
.field-label {
  flex-shrink: 0;
  min-width: 34px;
  font-size: 12px;
  font-weight: 300;
  letter-spacing: 1px;
  color: var(--text-3);
}
.seg {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.seg-btn {
  padding: 5px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12px;
  letter-spacing: 0.5px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.seg-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.seg-btn.active {
  color: var(--on-accent);
  border-color: var(--accent);
  background: var(--accent);
}
.seg-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.range-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: -2px 0 12px 44px;
}
.range-input {
  width: 76px;
  height: 30px;
  padding: 0 8px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-1);
  font-family: inherit;
  font-size: 12.5px;
  outline: none;
}
.range-input:focus {
  border-color: var(--accent);
}
.range-sep {
  font-size: 12px;
  color: var(--text-3);
}
.range-total {
  font-size: 11px;
  color: var(--text-3);
}
.cache-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 12px 0 2px;
}
.cache-bar {
  flex: 1;
  height: 5px;
  border-radius: 999px;
  background: var(--hover);
  overflow: hidden;
}
.cache-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
  transition: width 0.3s ease;
}
.cache-percent {
  flex-shrink: 0;
  min-width: 56px;
  text-align: right;
  font-size: 11.5px;
  font-weight: 300;
  color: var(--text-3);
  font-variant-numeric: tabular-nums;
}
.search-msg {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--text-2);
}
.search-msg.error {
  color: var(--danger, #d33);
}
.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
.ghost-btn {
  padding: 7px 18px;
  border-radius: var(--radius);
  border: 1px solid var(--border-strong);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.ghost-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
}
.ghost-btn:disabled,
.accent-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.accent-btn {
  padding: 7px 18px;
  border-radius: var(--radius);
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--on-accent);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease;
}
.accent-btn:hover:not(:disabled) {
  background: var(--accent-deep);
  border-color: var(--accent-deep);
}
.dlg-enter-active,
.dlg-leave-active {
  transition: opacity 0.18s ease;
}
.dlg-enter-active .dlg,
.dlg-leave-active .dlg {
  transition: transform 0.18s ease;
}
.dlg-enter-from,
.dlg-leave-to {
  opacity: 0;
}
.dlg-enter-from .dlg,
.dlg-leave-to .dlg {
  transform: translateY(8px) scale(0.98);
}
</style>
