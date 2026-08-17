import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  filterCommands,
  paletteCommands,
  searchCommandFor,
  SEARCH_COMMAND,
} from './commandPalette.ts'

test('命令表：搜索 + 跳转页面 + 设置项齐全（书架/书源/设置/探索/RSS/文件/用户/深色/语言）', () => {
  const cmds = paletteCommands()
  const ids = cmds.map((c) => c.id)
  // 搜索书籍
  assert.ok(ids.includes('search-books'))
  // 跳转页面
  for (const path of ['/', '/sources', '/settings', '/explore', '/rss', '/files', '/users']) {
    assert.ok(ids.includes(`nav-${path}`), `缺少导航命令 ${path}`)
  }
  // 设置项：深色 / 语言
  assert.ok(ids.includes('theme-dark'))
  assert.ok(ids.includes('theme-light'))
  assert.ok(ids.includes('theme-system'))
  assert.ok(ids.includes('lang-zh'))
  assert.ok(ids.includes('lang-en'))
  // 分组顺序：搜索在前，页面组在设置组前
  assert.equal(cmds[0].id, 'search-books')
  const groups = cmds.map((c) => c.group)
  assert.ok(groups.indexOf('跳转页面') < groups.indexOf('打开设置'))
})

test('空输入返回全部命令', () => {
  assert.equal(filterCommands('').length, paletteCommands().length)
  assert.equal(filterCommands('   ').length, paletteCommands().length)
})

test('按标题/关键词过滤（忽略大小写，空格分词 AND）', () => {
  const dark = filterCommands('深色')
  assert.ok(dark.length > 0)
  assert.ok(dark.some((c) => c.id === 'theme-dark'))

  const en = filterCommands('english')
  assert.ok(en.some((c) => c.id === 'lang-en'))

  // AND：两个词都命中才返回（'书源' 同时命中页面与设置词不在此列）
  const both = filterCommands('书源 管理')
  assert.ok(both.length > 0)
  assert.ok(both.every((c) => `${c.title} ${c.keywords.join(' ')}`.toLowerCase().includes('书源')))

  assert.equal(filterCommands('不存在的词xyz').length, 0)
})

test('搜索命令 action 声明正确', () => {
  assert.deepEqual(SEARCH_COMMAND.action, { kind: 'search' })
  const dyn = searchCommandFor('  斗破苍穹  ')
  assert.equal(dyn.title, '搜索：斗破苍穹')
  assert.deepEqual(dyn.action, { kind: 'search', keyword: '斗破苍穹' })
})

test('导航命令 path 正确', () => {
  const cmds = paletteCommands()
  const nav = cmds.find((c) => c.id === 'nav-/sources')
  assert.deepEqual(nav?.action, { kind: 'navigate', path: '/sources' })
})
