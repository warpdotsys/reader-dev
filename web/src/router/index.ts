import { createRouter, createWebHistory } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { t } from '@/utils/i18n'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { title: '登录', titleKey: 'route.login' },
    },
    {
      path: '/',
      name: 'bookshelf',
      component: () => import('@/views/BookshelfView.vue'),
      meta: { title: '书架', titleKey: 'route.bookshelf' },
    },
    {
      path: '/book/:url',
      name: 'book-detail',
      component: () => import('@/views/BookDetailView.vue'),
      meta: { title: '书籍详情', titleKey: 'route.bookDetail' },
    },
    {
      path: '/reader/:bookUrl',
      name: 'reader',
      component: () => import('@/views/ReaderView.vue'),
      meta: { title: '阅读', titleKey: 'route.reader' },
    },
    {
      path: '/search',
      name: 'search',
      component: () => import('@/views/SearchView.vue'),
      meta: { title: '搜索', titleKey: 'route.search' },
    },
    {
      path: '/explore',
      name: 'explore',
      component: () => import('@/views/ExploreView.vue'),
      meta: { title: '探索', titleKey: 'route.explore' },
    },
    {
      path: '/sources',
      name: 'sources',
      component: () => import('@/views/SourceManageView.vue'),
      meta: { title: '书源管理', titleKey: 'route.sources' },
    },
    {
      path: '/rules',
      name: 'rules',
      component: () => import('@/views/ReplaceRuleView.vue'),
      meta: { title: '替换规则', titleKey: 'route.rules' },
    },
    {
      path: '/rss',
      name: 'rss',
      component: () => import('@/views/RssView.vue'),
      meta: { title: 'RSS', titleKey: 'route.rss' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { title: '设置', titleKey: 'route.settings' },
    },
    {
      path: '/server-stats',
      name: 'server-stats',
      component: () => import('@/views/ServerStatsView.vue'),
      // P0-8 标注：服务监控为管理信息接口。登录守卫已有（下方 beforeEach）；
      // 非 secure 或非管理员场景由后端拒绝即可（管理接口走 checkManagerAuth：
      // 非 secure → 不支持；secure 缺/错 secureKey → NEED_SECURE_KEY），无需额外前端守卫
      meta: { title: '服务监控', titleKey: 'route.serverStats', requiresLogin: true, requiresManager: true },
    },
    {
      path: '/files',
      name: 'files',
      component: () => import('@/views/FileManageView.vue'),
      meta: { title: '文件', titleKey: 'route.files' },
    },
    {
      path: '/store',
      name: 'store',
      component: () => import('@/views/StoreView.vue'),
      meta: { title: '书仓', titleKey: 'route.store' },
    },
    {
      path: '/users',
      name: 'users',
      component: () => import('@/views/UserManageView.vue'),
      // P0-8 标注：用户管理为管理接口（getUsers 走 checkManagerAuth）。登录守卫已有；
      // 非 secure 或非管理员时后端拒绝已够——secure 缺/错 secureKey 返回 NEED_SECURE_KEY，
      // 由 UserManageView 引导输入（api/users.ts managerParams），无需额外前端路由守卫
      meta: { title: '用户管理', titleKey: 'route.users', requiresLogin: true, requiresManager: true },
    },
    // GAP 128：路由兜底——未匹配路径进简单 404 页（返回书架）
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: () => import('@/views/NotFoundView.vue'),
      meta: { title: '页面不存在', titleKey: 'route.notFound' },
    },
  ],
})

router.beforeEach((to) => {
  const store = useUserStore()
  if (to.path !== '/login' && !store.accessToken) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }
  if (to.path === '/login' && store.accessToken) {
    return { path: '/' }
  }
  return true
})

router.afterEach((to) => {
  const title = t(String(to.meta.titleKey ?? to.meta.title ?? ''))
  document.title = `${title} · ${t('brand.name')}`
})

export default router
