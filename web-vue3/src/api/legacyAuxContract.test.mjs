import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

const root = new URL('../../../', import.meta.url)

async function source(path) {
  return readFile(new URL(path, root), 'utf8')
}

test('OPDS、阅读统计和服务监控不调用恢复版不存在的路由', async () => {
  const [router, opds, stats, serverStats] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('web-vue3/src/api/opds.ts'),
    source('web-vue3/src/api/stats.ts'),
    source('web-vue3/src/api/serverStats.ts'),
  ])

  for (const route of ['getOpdsSettings', 'saveOpdsSettings', 'getReadingStats', 'getServerStats']) {
    assert.doesNotMatch(router, new RegExp(`"/reader3/${route}"`))
  }
  for (const client of [opds, stats, serverStats]) {
    assert.match(client, /Promise\.reject\(new Error\(/)
  }
})

test('TTS 使用 Java/Kotlin 已注册的 book/tts，并保留 legacy 字段语义', async () => {
  const [router, controller, client] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('src/main/java/com/htmake/reader/api/controller/BookController.kt'),
    source('web-vue3/src/api/tts.ts'),
  ])

  assert.match(router, /router\.post\("\/reader3\/book\/tts"\)/)
  assert.match(controller, /val type = value\("type"\)\.ifEmpty \{ "edge" \}/)
  assert.match(client, /fetch\(`\/reader3\/book\/tts/)
  assert.match(client, /type: useApi \? 'api' : undefined/)
  assert.match(client, /function toLegacyPitch\(value: string\)/)
  assert.doesNotMatch(client, /fetch\(`\/reader3\/tts/)
})

test('书源导出复用当前命名空间的 getBookSources，不请求不存在的下载路由', async () => {
  const [router, system] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('web-vue3/src/api/system.ts'),
  ])

  assert.match(router, /router\.get\("\/reader3\/getBookSources"\)/)
  assert.match(system, /get<unknown\[\]>\('\/getBookSources'\)/)
  assert.match(system, /new Blob\(\[JSON\.stringify\(response\.data \?\? \[\], null, 2\)\]/)
  assert.doesNotMatch(system, /['"]\/exportBookSources['"]|request\.get/)
})
