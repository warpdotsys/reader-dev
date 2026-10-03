import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

const root = new URL('../../../', import.meta.url)

async function source(path) {
  return readFile(new URL(path, root), 'utf8')
}

test('RSS 客户端只调用 Java/Kotlin 已注册的文章列表和正文路由', async () => {
  const [api, controller, client] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('src/main/java/com/htmake/reader/api/controller/RssSourceController.kt'),
    source('web-vue3/src/api/rss.ts'),
  ])

  for (const route of ['getRssArticles', 'getRssContent']) {
    assert.match(api, new RegExp(`"/reader3/${route}"`))
    assert.match(client, new RegExp(`['"]/${route}['"]`))
  }
  assert.match(controller, /context\.bodyAsJson\.getString\("sourceUrl"\)/)
  assert.match(controller, /context\.queryParam\("sourceUrl"\)/)
  assert.match(controller, /context\.bodyAsJson\.getString\("sortName"/)
  assert.match(controller, /context\.bodyAsJson\.getString\("link"\)/)
  assert.match(controller, /context\.bodyAsJson\.getString\("origin"\)/)
  assert.match(client, /\{ sourceUrl, sortName, sortUrl, page \}/)
  assert.match(client, /\{ sourceUrl, link, origin \}/)
  assert.doesNotMatch(client, /['"]\/(?:getRssArticle|markRssArticleRead)['"]|rssSourceUrl/)
})

test('RSS 删除请求保留完整源实体，文章字段与 Kotlin RssArticle 一致', async () => {
  const [client, entity, view] = await Promise.all([
    source('web-vue3/src/api/rss.ts'),
    source('src/main/java/io/legado/app/data/entities/RssArticle.kt'),
    source('web-vue3/src/views/RssView.vue'),
  ])

  assert.match(client, /deleteRssSource\(source: RssSource\)/)
  assert.match(client, /post<string>\('\/deleteRssSource', source\)/)
  for (const field of ['origin', 'link', 'pubDate', 'image', 'read']) {
    assert.match(entity, new RegExp(`var ${field}:`))
  }
  assert.match(view, /getRssContent\(selectedUrl\.value, a\.link, a\.origin\)/)
  assert.match(view, /const READ_KEY = 'rss_read_articles'/)
  assert.match(view, /articleKey\(a: RssArticle\)/)
  assert.doesNotMatch(view, /(?:markRssArticleRead|hasRead)|getRssArticle\(a\.url\)/)
})
