import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

const root = new URL('../../../', import.meta.url)

async function source(path) {
  return readFile(new URL(path, root), 'utf8')
}

test('缓存前端只调用 Java/Kotlin 已注册的整书缓存与单书缓存路由', async () => {
  const [api, controller, client, dialog, reader] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('src/main/java/com/htmake/reader/api/controller/BookController.kt'),
    source('web-vue3/src/api/cacheBook.ts'),
    source('web-vue3/src/components/ChapterCacheDialog.vue'),
    source('web-vue3/src/views/ReaderView.vue'),
  ])

  assert.match(api, /router\.get\("\/reader3\/cacheBookSSE"\)/)
  assert.match(api, /router\.post\("\/reader3\/cacheBookOnServer"\)/)
  assert.match(api, /router\.get\("\/reader3\/getShelfBookWithCacheInfo"\)/)
  assert.match(api, /router\.post\("\/reader3\/deleteBookCache"\)/)
  assert.match(controller, /getJsonArray\("bookUrlList"\)/)
  assert.match(controller, /"cachedCount" to cachedChapterContentSet\.size/)
  assert.match(client, /post<string>\('\/cacheBookOnServer', \{ bookUrlList \}/)
  assert.match(client, /new URLSearchParams\(\{ url: bookUrl \}\)/)
  assert.match(client, /cachedCount/)

  for (const missingRoute of [
    'cacheBookRangeOnServer',
    'cancelCacheBook',
    'getBookCacheChapters',
    'getCacheInfo',
    'clearCache',
  ]) {
    assert.doesNotMatch(api, new RegExp(`/reader3/${missingRoute}`))
    assert.doesNotMatch(client, new RegExp(`['"]/${missingRoute}`))
  }
  assert.doesNotMatch(dialog, /cacheBookRangeOnServer|cancelCacheBook/)
  assert.doesNotMatch(reader, /getBookCacheChapters/)
})

test('服务端缓存 UI 明示整书限制，全局缓存 UI 不伪造统计或清理成功', async () => {
  const [dialog, settings, cache] = await Promise.all([
    source('web-vue3/src/components/ChapterCacheDialog.vue'),
    source('web-vue3/src/views/SettingsView.vue'),
    source('web-vue3/src/api/cache.ts'),
  ])

  assert.match(dialog, /仅支持整书服务端缓存/)
  assert.match(dialog, /关闭此窗口会停止当前请求/)
  assert.match(settings, /全局缓存统计和批量清理尚无后端接口/)
  assert.doesNotMatch(settings, /getCacheInfo|clearCache|已清理\$\{clearTypeLabel/)
  assert.doesNotMatch(cache, /function getCacheInfo|function clearCache/)
})
