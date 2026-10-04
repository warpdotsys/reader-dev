import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  clearLocalBook, getLocalChapter, listLocalChapterUrls, localChapterCacheScope,
  saveLocalChapter, saveLocalChapters,
} from './readerLocalCache.ts'

const user = { accessToken: 'generated-session-one', username: 'generated-a', isAdmin: false, defaultConfigMode: false }
const deployment = 'http://127.0.0.1:18895/'

test('章节缓存隔离账号、部署路径和实际命名空间，但不持久化 accessToken', () => {
  const scope = localChapterCacheScope(user, deployment)
  assert.ok(scope)
  assert.doesNotMatch(scope, /generated-session-one/)
  assert.equal(localChapterCacheScope({ ...user, accessToken: 'generated-session-two' }, deployment), scope)
  assert.notEqual(localChapterCacheScope({ ...user, username: 'generated-b' }, deployment), scope)
  assert.notEqual(localChapterCacheScope(user, deployment + 'reader/'), scope)
  assert.equal(localChapterCacheScope({ ...user, defaultConfigMode: true }, deployment), scope)
  const admin = { ...user, isAdmin: true }
  assert.notEqual(localChapterCacheScope({ ...admin, defaultConfigMode: true }, deployment), scope)
  assert.notEqual(
    localChapterCacheScope({ ...admin, defaultConfigMode: true }, deployment),
    localChapterCacheScope({ ...admin, username: 'generated-b', defaultConfigMode: true }, deployment),
  )
})

test('无认证身份时所有章节缓存操作关闭，不打开历史共享缓存', async () => {
  assert.equal(localChapterCacheScope({ ...user, accessToken: '' }, deployment), null)
  assert.equal(localChapterCacheScope({ ...user, username: '' }, deployment), null)
  assert.equal(localChapterCacheScope(user, ''), null)
  const chapter = { bookUrl: 'generated-book', chapterUrl: 'generated-chapter', title: '生成章节', index: 0, content: '生成正文' }
  assert.equal(await getLocalChapter(null, chapter.bookUrl, chapter.chapterUrl), null)
  assert.equal(await saveLocalChapter(null, chapter), false)
  assert.equal(await saveLocalChapters(null, [chapter]), 0)
  assert.equal(await clearLocalBook(null, chapter.bookUrl), 0)
  assert.deepEqual(await listLocalChapterUrls(null, chapter.bookUrl), [])
})
