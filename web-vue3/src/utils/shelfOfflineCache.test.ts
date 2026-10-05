import assert from 'node:assert/strict'
import { test } from 'node:test'
import { canUseOfflineShelf, loadOfflineShelf, saveOfflineShelf } from './shelfOfflineCache.ts'
import { localChapterCacheScope } from './readerLocalCache.ts'

const user = { accessToken: 'generated-secret', username: 'generated-a', isAdmin: false, defaultConfigMode: false }
const deployment = 'http://127.0.0.1:18945/'
const scope = localChapterCacheScope(user, deployment)!
const shelf = { books: [{ bookUrl: 'generated-book', tocUrl: '', origin: 'loc_book', originName: 'generated', name: '生成书', author: '生成作者', type: 0, group: 0, latestChapterTime: 1 }], groups: [{ id: 1, name: '生成组' }], ts: 1 }
const prefix = 'reader_shelf_offline_v2:'

function storage() {
  const records = new Map<string, string>([['reader_shelf_offline', 'unknown legacy untouched']])
  return { records, getItem: (key: string) => records.get(key) ?? null, setItem: (key: string, value: string) => { records.set(key, value) } }
}

test('离线书架按部署/账号/default 空间隔离，不含凭据；同账号换 token 可续读', () => {
  const local = storage()
  assert.equal(saveOfflineShelf(scope, shelf, local), true)
  assert.deepEqual(loadOfflineShelf(scope, local), shelf)
  assert.deepEqual(loadOfflineShelf(localChapterCacheScope({ ...user, accessToken: 'new-generated-secret' }, deployment), local), shelf)
  for (const other of [
    localChapterCacheScope({ ...user, username: 'generated-b' }, deployment),
    localChapterCacheScope(user, deployment + 'reader/'),
    localChapterCacheScope({ ...user, isAdmin: true, defaultConfigMode: true }, deployment),
  ]) assert.equal(loadOfflineShelf(other, local), null)
  assert.equal(local.records.get('reader_shelf_offline'), 'unknown legacy untouched')
  assert.doesNotMatch([...local.records.keys()].join(''), /generated-secret/)
})

test('匿名、旧共享键和不一致封套不可读；不自动迁移或删除', () => {
  const local = storage()
  assert.equal(loadOfflineShelf(scope, local), null)
  assert.equal(loadOfflineShelf(null, local), null)
  assert.equal(saveOfflineShelf(null, shelf, local), false)
  for (const envelope of [shelf, { version: 1, scope, ...shelf }, { version: 2, scope: 'wrong-account', ...shelf }]) {
    local.records.set(prefix + scope, JSON.stringify(envelope))
    assert.equal(loadOfflineShelf(scope, local), null)
  }
  assert.equal(local.records.get('reader_shelf_offline'), 'unknown legacy untouched')
})

test('损坏或不完整元数据失败关闭，不补造字段；配额错误不报告保存成功', () => {
  const local = storage()
  for (const bad of [null, {}, { ...shelf, books: [null] }, { ...shelf, books: [{ name: '无链接' }] },
    { ...shelf, groups: [{ id: '1', name: '错类型' }] }, { ...shelf, ts: -1 }]) {
    local.records.set(prefix + scope, JSON.stringify({ version: 2, scope, ...(bad as object) }))
    assert.equal(loadOfflineShelf(scope, local), null)
  }
  local.records.set(prefix + scope, '{broken')
  assert.equal(loadOfflineShelf(scope, local), null)
  const broken = { getItem: () => { throw new Error('unavailable') }, setItem: () => { throw new Error('quota') } }
  assert.equal(loadOfflineShelf(scope, broken), null)
  assert.equal(saveOfflineShelf(scope, shelf, broken), false)
})

test('只允许明确网络失败、超时、502/503/504离线降级', () => {
  for (const code of ['ERR_NETWORK', 'ECONNABORTED', 'ETIMEDOUT']) assert.equal(canUseOfflineShelf({ isAxiosError: true, code }), true)
  for (const status of [502, 503, 504]) assert.equal(canUseOfflineShelf({ isAxiosError: true, response: { status } }), true)
  for (const error of [new Error('Network Error'), { code: 'READER_STALE_SESSION' },
    { isAxiosError: true, code: 'ERR_CANCELED' }, { isAxiosError: true, code: 'ERR_BAD_OPTION' },
    ...[200, 400, 401, 403, 404, 500, 501].map(status => ({ isAxiosError: true, response: { status } })),
    ...['NEED_LOGIN', 'NEED_SECURE_KEY'].map(data => ({ isAxiosError: true, response: { status: 503, data: { data } } })),
    { isAxiosError: true, response: { status: 503, data: { errorMsg: '请登录后使用' } } },
  ]) assert.equal(canUseOfflineShelf(error), false)
})
