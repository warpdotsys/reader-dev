import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

const root = new URL('../../../', import.meta.url)

async function source(path) {
  return readFile(new URL(path, root), 'utf8')
}

test('发现页仅调用 Java/Kotlin 已注册的书源和探索接口', async () => {
  const [api, controller, client, view] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('src/main/java/com/htmake/reader/api/controller/BookController.kt'),
    source('web-vue3/src/api/explore.ts'),
    source('web-vue3/src/views/ExploreView.vue'),
  ])

  for (const route of ['getBookSources', 'exploreBook']) {
    assert.match(api, new RegExp(`"/reader3/${route}"`))
    assert.match(client, new RegExp(`['"]/${route}['"]`))
  }
  assert.doesNotMatch(api, /"\/reader3\/(?:getExploreSources|getExploreUrls)"/)
  assert.doesNotMatch(client, /['"]\/(?:getExploreSources|getExploreUrls)['"]|\bnew\s+Function\s*\(/)
  assert.match(client, /get<BookSource\[\]>\('\/getBookSources'\)/)
  assert.match(client, /post<SearchBook\[\]>\('\/exploreBook', \{ ruleFindUrl, bookSourceUrl, page \}\)/)
  assert.match(controller, /context\.bodyAsJson\.getString\("ruleFindUrl"\)/)
  assert.match(controller, /getBookSourceString\(context\)/)
  assert.match(view, /\.filter\(\(item\) => !!item\.exploreUrl\?\.trim\(\)\)/)
})

test('发现分类只解析静态格式，动态规则不会被伪装成空结果', async () => {
  const client = await source('web-vue3/src/api/explore.ts')

  assert.match(client, /JSON\.parse\(raw\)/)
  assert.match(client, /split\('\\n'\)/)
  assert.match(client, /raw\.startsWith\('@js:'\) \|\| raw\.includes\('<js>'\)/)
  assert.match(client, /throw unsupportedExploreUrl\(source\)/)
  assert.match(client, /当前 Web 端不能安全执行/)
})
