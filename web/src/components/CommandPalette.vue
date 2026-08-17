<script setup lang="ts">
/**
 * 全局命令面板（Ctrl+K）——任意页面可用：
 * - Ctrl+K / Ctrl+Shift+K 开关（Esc 关闭）
 * - ↑↓ 选择 / Enter 执行 / Esc 关闭（输入框内与面板级均生效）
 * - 输入即过滤命令；输入非空时追加「搜索：{kw}」动态命令（跳搜索页预填）
 * 挂载于 App.vue（全局单例），不依赖当前路由页面。
 */
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  filterCommands,
  searchCommandFor,
  type PaletteCommand,
} from '@/utils/commandPalette'
import { applyUiTheme } from '@/utils/uiTheme'
import { setLang } from '@/utils/i18n'

const router = useRouter()

const open = ref(false)
const query = ref('')
const activeIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLUListElement | null>(null)

/** 过滤后的命令列表；输入非空时末尾追加「搜索：{kw}」动态命令 */
const commands = computed<PaletteCommand[]>(() => {
  const kw = query.value.trim()
  const list = filterCommands(query.value)
  if (kw) list.push(searchCommandFor(kw))
  return list
})

watch(commands, () => {
  if (activeIndex.value >= commands.value.length) activeIndex.value = Math.max(0, commands.value.length - 1)
})

function openPalette() {
  open.value = true
  query.value = ''
  activeIndex.value = 0
  void nextTick(() => {
    inputRef.value?.focus()
    inputRef.value?.select()
  })
}

function closePalette() {
  open.value = false
}

/** 执行命令（按声明式 action 分发） */
function runCommand(cmd: PaletteCommand) {
  switch (cmd.action.kind) {
    case 'navigate':
      void router.push(cmd.action.path)
      break
    case 'search':
      void router.push(
        cmd.action.keyword ? { path: '/search', query: { key: cmd.action.keyword } } : '/search',
      )
      break
    case 'theme':
      applyUiTheme(cmd.action.theme)
      break
    case 'lang':
      setLang(cmd.action.lang)
      break
  }
  closePalette()
}

/** 键盘导航：↑↓ 选择 / Enter 执行 / Esc 关闭（Home/End 首尾） */
function onKeydown(e: KeyboardEvent) {
  const list = commands.value
  const n = list.length
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      if (n > 0) activeIndex.value = (activeIndex.value + 1) % n
      break
    case 'ArrowUp':
      e.preventDefault()
      if (n > 0) activeIndex.value = (activeIndex.value - 1 + n) % n
      break
    case 'Home':
      e.preventDefault()
      activeIndex.value = 0
      break
    case 'End':
      e.preventDefault()
      if (n > 0) activeIndex.value = n - 1
      break
    case 'Enter': {
      e.preventDefault()
      const cmd = list[activeIndex.value]
      if (cmd) runCommand(cmd)
      break
    }
    case 'Escape':
      e.preventDefault()
      closePalette()
      break
  }
}

/** 选中项滚动进可视区（列表超高时） */
watch(activeIndex, () => {
  void nextTick(() => {
    listRef.value?.querySelector('.pal-item.active')?.scrollIntoView({ block: 'nearest' })
  })
})

/** 全局 Ctrl+K（macOS 用 Cmd+K）开关 */
function onGlobalKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    e.stopPropagation()
    if (open.value) closePalette()
    else openPalette()
  }
}

onMounted(() => window.addEventListener('keydown', onGlobalKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onGlobalKeydown))
</script>

<template>
  <Teleport to="body">
    <Transition name="pal">
      <div v-if="open" class="pal-overlay" @mousedown.self="closePalette">
        <div
          class="palette"
          role="dialog"
          aria-modal="true"
          aria-label="命令面板"
          @keydown="onKeydown"
        >
          <div class="pal-input-row">
            <svg class="pal-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round">
              <circle cx="11" cy="11" r="6.5" />
              <path d="M20 20l-3.8-3.8" />
            </svg>
            <input
              ref="inputRef"
              v-model="query"
              class="pal-input"
              type="text"
              placeholder="输入命令 / 搜索书籍…"
              spellcheck="false"
              autocomplete="off"
            />
            <kbd class="pal-kbd">Esc</kbd>
          </div>
          <ul ref="listRef" class="pal-list">
            <li
              v-for="(cmd, i) in commands"
              :key="cmd.id"
              class="pal-item"
              :class="{ active: i === activeIndex }"
              @mousemove="activeIndex = i"
              @click="runCommand(cmd)"
            >
              <span class="pal-title" :title="cmd.title">{{ cmd.title }}</span>
              <span class="pal-group">{{ cmd.group }}</span>
            </li>
            <li v-if="commands.length === 0" class="pal-empty">无匹配命令</li>
          </ul>
          <div class="pal-foot">
            <span>↑↓ 选择</span>
            <span>Enter 执行</span>
            <span>Esc 关闭</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.pal-overlay {
  position: fixed;
  inset: 0;
  z-index: 400;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 14vh 16px 16px;
  background: rgba(10, 10, 14, 0.35);
  backdrop-filter: blur(2px);
}
.palette {
  width: min(560px, 100%);
  overflow: hidden;
  background: var(--surface);
  border: 1px solid var(--border-strong);
  border-radius: 14px;
  box-shadow: 0 18px 56px rgba(0, 0, 0, 0.24);
}
.pal-input-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 13px 16px;
  border-bottom: 1px solid var(--border);
}
.pal-search-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  color: var(--text-3);
}
.pal-input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: none;
  color: var(--text-1);
  font-family: inherit;
  font-size: 15px;
  letter-spacing: 0.5px;
}
.pal-input::placeholder {
  color: var(--text-3);
}
.pal-kbd {
  flex-shrink: 0;
  padding: 2px 7px;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: var(--hover);
  color: var(--text-3);
  font-family: inherit;
  font-size: 11px;
  line-height: 1.5;
}
.pal-list {
  max-height: 340px;
  overflow-y: auto;
  margin: 0;
  padding: 6px;
  list-style: none;
}
.pal-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 9px 12px;
  border-radius: 8px;
  cursor: pointer;
}
.pal-item.active {
  background: var(--accent-soft);
}
.pal-title {
  font-size: 13.5px;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.pal-group {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
  letter-spacing: 1px;
}
.pal-empty {
  padding: 18px 12px;
  text-align: center;
  font-size: 13px;
  color: var(--text-3);
}
.pal-foot {
  display: flex;
  gap: 16px;
  padding: 9px 16px;
  border-top: 1px solid var(--border);
  font-size: 11px;
  font-weight: 300;
  color: var(--text-3);
}
.pal-enter-active,
.pal-leave-active {
  transition: opacity 0.14s ease;
}
.pal-enter-active .palette,
.pal-leave-active .palette {
  transition: transform 0.14s ease;
}
.pal-enter-from,
.pal-leave-to {
  opacity: 0;
}
.pal-enter-from .palette,
.pal-leave-to .palette {
  transform: translateY(-10px) scale(0.99);
}
</style>
