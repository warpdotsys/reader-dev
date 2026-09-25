<script setup lang="ts">
/**
 * 顶部导航共享组件（P3-A：各视图手抄顶栏收敛）
 *
 * 统一渲染：品牌字标 + 导航链接（router.push）+ 用户 chip / 退出（可选）。
 * 已替换视图：书架 / 搜索 / 探索 / 书源 / RSS / 文件 / 设置 / 用户。
 *
 * Props:
 * - variant: 'nav'（品牌 + 导航链接，默认）| 'minimal'（返回按钮 + 品牌）
 * - active: 当前路由路径（匹配的链接加 .active 高亮）
 * - links: 要显示的导航键（默认全量；'users' 仅 showUsersLink 时按管理员身份门控）
 * - showUser: 是否显示用户名 chip（默认 true）
 * - showLogout: 是否显示退出按钮（默认 false；点击 emit('logout')）
 * - showUsersLink: 是否显示「用户」入口（默认 false；仅管理员可见，不受 secure 模式限制）
 * - backLabel: minimal 变体返回按钮文案（默认空 = 仅图标）
 * - dense: 紧凑顶栏（探索页风格：小间距/细边框）
 *
 * Slots:
 * - leading: 覆盖最左侧内容（返回按钮/品牌；默认按 variant 渲染）
 * - default: 品牌与导航之间的内容（书架搜索框 / 探索页标题）
 * - extra: 导航行内的附加按钮（书架的书签/OPDS 等视图专属动作）
 * - trailing: minimal 变体下导航行位置的附加内容（探索页 top-actions）
 */
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { t } from '@/utils/i18n'

const props = withDefaults(
  defineProps<{
    variant?: 'nav' | 'minimal'
    active?: string
    links?: string[]
    showUser?: boolean
    showLogout?: boolean
    showUsersLink?: boolean
    backLabel?: string
    dense?: boolean
  }>(),
  {
    variant: 'nav',
    active: '',
    links: () => [
      'bookshelf',
      'search',
      'explore',
      'sources',
      'rules',
      'rss',
      'files',
      'store',
      'monitor',
      'users',
      'settings',
    ],
    showUser: true,
    showLogout: false,
    showUsersLink: false,
    backLabel: '',
    dense: false,
  },
)

const emit = defineEmits<{ logout: []; back: [] }>()

const router = useRouter()
const store = useUserStore()

/** 管理员手动进入/退出 default（系统配置层）身份 */
function toggleDefaultConfig() {
  store.toggleDefaultConfigMode()
}

/** 导航键 → 路由与 i18n 文案（键名与 i18n nav.* 对齐；缺失回退 zh/原文） */
const NAV_LINKS: Record<string, { to: string; i18n: string }> = {
  bookshelf: { to: '/', i18n: 'nav.bookshelf' },
  search: { to: '/search', i18n: 'nav.search' },
  explore: { to: '/explore', i18n: 'nav.explore' },
  sources: { to: '/sources', i18n: 'nav.sources' },
  rules: { to: '/rules', i18n: 'nav.rules' },
  rss: { to: '/rss', i18n: 'nav.rss' },
  files: { to: '/files', i18n: 'nav.files' },
  store: { to: '/store', i18n: 'nav.store' },
  monitor: { to: '/server-stats', i18n: 'nav.serverStats' },
  users: { to: '/users', i18n: 'nav.users' },
  settings: { to: '/settings', i18n: 'nav.settings' },
}

const visibleLinks = computed(() => {
  const out: { to: string; label: string }[] = []
  for (const key of props.links) {
    if (key === 'users' && !(props.showUsersLink && store.isAdmin)) continue
    const def = NAV_LINKS[key]
    if (!def) continue
    out.push({ to: def.to, label: t(def.i18n) })
  }
  return out
})
</script>

<template>
  <header class="topbar" :class="{ dense }">
    <slot name="leading">
      <template v-if="variant === 'minimal'">
        <button class="back-btn" type="button" @click="emit('back')">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M19 12H5" />
            <path d="M11 18l-6-6 6-6" />
          </svg>
          <span v-if="backLabel">{{ backLabel }}</span>
        </button>
      </template>
      <div class="brand">
        <img class="brand-logo" src="/logo.svg" alt="夜读" />
        <span class="brand-name">{{ t('brand.name') }}<span class="brand-dot">.</span></span>
      </div>
    </slot>

    <slot />

    <div v-if="variant === 'nav'" class="user-area">
      <button
        v-for="link in visibleLinks"
        :key="link.to"
        class="nav-link"
        :class="{ active: link.to === active }"
        type="button"
        @click="router.push(link.to)"
      >
        {{ link.label }}
      </button>
      <slot name="extra" />
      <button
        v-if="store.isAdmin"
        class="default-config-btn"
        :class="{ active: store.defaultConfigMode }"
        type="button"
        :aria-pressed="store.defaultConfigMode"
        :title="store.defaultConfigMode ? '退出系统配置模式，回到本人账号' : '进入系统配置模式（default）：编辑对所有用户生效的公用数据'"
        @click="toggleDefaultConfig"
      >
        {{ store.defaultConfigMode ? '退出系统配置' : '系统配置' }}
      </button>
      <span v-if="showUser" class="user-chip">{{ store.username || '未登录' }}</span>
      <button v-if="showLogout" class="logout-btn" type="button" @click="emit('logout')">
        {{ t('nav.logout') }}
      </button>
    </div>
    <slot v-else name="trailing" />
    <button
      v-if="variant === 'minimal' && store.isAdmin"
      class="default-config-btn"
      :class="{ active: store.defaultConfigMode }"
      type="button"
      :aria-pressed="store.defaultConfigMode"
      :title="store.defaultConfigMode ? '退出系统配置模式，回到本人账号' : '进入系统配置模式（default）：编辑对所有用户生效的公用数据'"
      @click="toggleDefaultConfig"
    >
      {{ store.defaultConfigMode ? '退出系统配置' : '系统配置' }}
    </button>
  </header>
</template>

<style scoped>
.topbar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 14px 32px;
  background: var(--bg-float);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}
.topbar.dense {
  gap: 12px;
  padding: 12px 20px;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.brand-logo {
  width: 26px;
  height: 26px;
}
.brand-name {
  font-size: 17px;
  font-weight: 300;
  letter-spacing: 3px;
  color: var(--text-1);
}
.brand-dot {
  color: var(--accent);
  font-weight: 400;
}
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.back-btn:hover {
  color: var(--accent);
}
.back-btn svg {
  width: 14px;
  height: 14px;
}
.user-area {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-left: auto;
  flex-shrink: 0;
}
.nav-link {
  padding: 5px 2px;
  border: none;
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 13px;
  font-weight: 300;
  letter-spacing: 1px;
  cursor: pointer;
  transition: color 0.2s ease;
}
.nav-link:hover {
  color: var(--accent);
}
.nav-link.active {
  color: var(--accent);
  font-weight: 400;
}
.user-chip {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-2);
}
.logout-btn {
  padding: 6px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease;
}
.logout-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.default-config-btn {
  flex-shrink: 0;
  padding: 6px 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: none;
  color: var(--text-2);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 400;
  letter-spacing: 1px;
  cursor: pointer;
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease;
}
.default-config-btn:hover:not(:disabled) {
  color: var(--text-1);
  border-color: var(--border-strong);
}
.default-config-btn.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* 响应式（对齐各视图原媒体查询：RSS 760px / 其余 720px；书架移动端横向滚动导航） */
@media (max-width: 760px) {
  .topbar {
    padding: 12px 16px;
  }
}
@media (max-width: 720px) {
  .topbar {
    flex-wrap: wrap;
    gap: 12px;
  }
  .user-area {
    overflow-x: auto;
    max-width: 100%;
    scrollbar-width: none;
    -webkit-overflow-scrolling: touch;
  }
  .user-area::-webkit-scrollbar {
    display: none;
  }
  .user-area .nav-link,
  .user-area .user-chip,
  .user-area .logout-btn,
  .user-area .default-config-btn {
    flex-shrink: 0;
  }
}
</style>
