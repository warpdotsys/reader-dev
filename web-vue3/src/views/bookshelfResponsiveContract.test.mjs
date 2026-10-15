import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

const shelf = await readFile(new URL('./BookshelfView.vue', import.meta.url), 'utf8')

test('group labels own their overflow rather than painting over management controls', () => {
  const tabs = shelf.match(/\.group-tabs\s*\{([^}]+)\}/)?.[1]
  assert.ok(tabs)
  assert.match(tabs, /min-width:\s*0/)
  assert.match(tabs, /overflow-x:\s*auto/)
  assert.match(shelf, /class="group-tabs" role="tablist"/)
  assert.match(shelf, /class="group-manage"/)
})

test('phone shelf retains readable heading and all actions on separate grid and filter rows', () => {
  const phone = shelf.slice(shelf.indexOf('@media (max-width: 480px)'))
  assert.match(phone, /\.section-head\s*\{[^}]*display:\s*grid/)
  assert.match(phone, /grid-template-columns:\s*1fr auto auto auto/)
  assert.match(phone, /\.section-title\s*\{[^}]*grid-column:\s*1\s*\/\s*-1/)
  assert.match(phone, /\.section-head \.count,[\s\S]*?white-space:\s*nowrap/)
  assert.match(phone, /\.group-bar\s*\{[^}]*flex-wrap:\s*wrap/)
  assert.match(phone, /\.group-tabs\s*\{[^}]*flex-basis:\s*100%/)
})

test('OPDS shelf entry routes to independent account settings without promising progress sync', () => {
  assert.match(shelf, /OPDS 1\.2 提供本人书架中的本地 TXT、EPUB、PDF 下载/)
  assert.match(shelf, /配置 OPDS 独立账号/)
  assert.match(shelf, /不使用网页登录密码，不同步网页阅读进度/)
  assert.doesNotMatch(shelf, /const opdsUrl|function copyOpdsUrl|opdsCopied/)
  assert.doesNotMatch(shelf, /即可同步书架与阅读/)
  assert.doesNotMatch(shelf, /class="opds-url/)
})
