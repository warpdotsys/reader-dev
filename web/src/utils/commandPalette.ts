/**
 * 全局命令面板（Ctrl+K）——命令注册表 + 过滤逻辑（纯函数，可单测）。
 *
 * 组件 components/CommandPalette.vue 负责 UI / 键盘导航 / 快捷键监听；
 * 本模块只定义命令数据与匹配逻辑：动作以声明式 action 描述，
 * 组件统一执行（跳转 / 搜索 / 主题 / 语言），便于测试与后续扩展。
 */

import type { UiTheme } from './uiTheme'
import type { Lang } from './i18n'

/** 命令动作（声明式：组件按 kind 分发执行） */
export type PaletteAction =
  | { kind: 'navigate'; path: string }
  | { kind: 'search'; keyword?: string }
  | { kind: 'theme'; theme: UiTheme }
  | { kind: 'lang'; lang: Lang }

export interface PaletteCommand {
  id: string
  /** 分组名（右侧标签，如「跳转页面」「打开设置」「搜索」） */
  group: string
  title: string
  /** 匹配关键词（中英文别名，输入任一词即命中） */
  keywords: string[]
  action: PaletteAction
}

/** 页面跳转命令（书架/书源/设置/探索/RSS/文件/用户 + 替换规则/书仓） */
const NAV_PAGES: { path: string; title: string; keywords: string[] }[] = [
  { path: '/', title: '书架', keywords: ['bookshelf', 'shelf', '首页', 'home'] },
  { path: '/search', title: '搜索', keywords: ['search', '查找'] },
  { path: '/explore', title: '探索', keywords: ['explore', '发现', 'discover'] },
  { path: '/sources', title: '书源管理', keywords: ['source', '书源', 'booksource'] },
  { path: '/rules', title: '替换规则', keywords: ['rule', 'replace', '规则'] },
  { path: '/rss', title: 'RSS', keywords: ['rss', '订阅', 'feed'] },
  { path: '/settings', title: '设置', keywords: ['settings', 'config', '配置', '偏好'] },
  { path: '/files', title: '文件', keywords: ['file', '文件', '本地'] },
  { path: '/users', title: '用户管理', keywords: ['user', '用户', '账号'] },
  { path: '/store', title: '书仓', keywords: ['store', '书仓', '共享'] },
]

/** 设置项快捷命令（深色 / 浅色 / 跟随系统 / 语言） */
const SETTING_COMMANDS: Omit<PaletteCommand, 'group'>[] = [
  {
    id: 'theme-dark',
    title: '深色模式',
    keywords: ['dark', '深色', '夜间', 'night', 'theme'],
    action: { kind: 'theme', theme: 'dark' },
  },
  {
    id: 'theme-light',
    title: '浅色模式',
    keywords: ['light', '浅色', '白天', 'theme'],
    action: { kind: 'theme', theme: 'light' },
  },
  {
    id: 'theme-system',
    title: '跟随系统',
    keywords: ['system', '跟随系统', '自动', 'theme'],
    action: { kind: 'theme', theme: 'system' },
  },
  {
    id: 'lang-zh',
    title: '界面语言：中文',
    keywords: ['language', '语言', '中文', 'zh'],
    action: { kind: 'lang', lang: 'zh' },
  },
  {
    id: 'lang-en',
    title: '界面语言：English',
    keywords: ['language', '语言', 'english', '英文', 'en'],
    action: { kind: 'lang', lang: 'en' },
  },
]

/** 搜索书籍命令（跳搜索页；组件会按面板输入词追加「搜索：{kw}」动态命令） */
export const SEARCH_COMMAND: PaletteCommand = {
  id: 'search-books',
  group: '搜索',
  title: '搜索书籍',
  keywords: ['search', '搜索', '找书', '书籍'],
  action: { kind: 'search' },
}

/** 完整命令表（按分组顺序排列，过滤后保持此顺序） */
export function paletteCommands(): PaletteCommand[] {
  return [
    SEARCH_COMMAND,
    ...NAV_PAGES.map(
      (p): PaletteCommand => ({
        id: `nav-${p.path}`,
        group: '跳转页面',
        title: `跳转：${p.title}`,
        keywords: p.keywords,
        action: { kind: 'navigate', path: p.path },
      }),
    ),
    ...SETTING_COMMANDS.map(
      (c): PaletteCommand => ({
        ...c,
        group: '打开设置',
      }),
    ),
  ]
}

/** 按输入词过滤命令：空格分词 AND 匹配（标题 + 关键词，忽略大小写）；空输入返回全部 */
export function filterCommands(query: string, commands: PaletteCommand[] = paletteCommands()): PaletteCommand[] {
  const terms = query
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
  if (terms.length === 0) return commands
  return commands.filter((c) => {
    const hay = `${c.title} ${c.keywords.join(' ')}`.toLowerCase()
    return terms.every((term) => hay.includes(term))
  })
}

/** 按面板输入词生成「搜索：{kw}」动态命令（跳搜索页并预填关键词） */
export function searchCommandFor(keyword: string): PaletteCommand {
  const kw = keyword.trim()
  return {
    id: `search-${kw}`,
    group: '搜索',
    title: `搜索：${kw}`,
    keywords: ['search', '搜索', kw],
    action: { kind: 'search', keyword: kw },
  }
}
