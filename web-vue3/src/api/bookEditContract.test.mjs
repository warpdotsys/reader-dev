import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

test('编辑正文发送 legacy 的 url/index/content，不发送 bookUrl/chapterUrl 伪契约', async () => {
  const root = new URL('../../../', import.meta.url)
  const [client, controller, reader] = await Promise.all([
    readFile(new URL('web-vue3/src/api/books.ts', root), 'utf8'),
    readFile(new URL('src/main/java/com/htmake/reader/api/controller/BookController.kt', root), 'utf8'),
    readFile(new URL('web-vue3/src/views/ReaderView.vue', root), 'utf8'),
  ])
  assert.match(controller, /suspend fun saveBookContent/)
  assert.match(client, /post<string>\('\/saveBookContent', \{ url: bookUrl, index, content \}\)/)
  assert.match(reader, /await saveBookContent\(url, index, newContent\)/)
  assert.doesNotMatch(reader, /post\('\/saveBookContent'/)
})
