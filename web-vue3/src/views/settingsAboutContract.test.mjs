import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

const root = new URL('../../../', import.meta.url)

async function source(path) {
  return readFile(new URL(path, root), 'utf8')
}

test('候选前端、锁文件和 Java/Kotlin 工程使用同一待发布版本', async () => {
  const [backend, oldWeb, vueWeb, lock] = await Promise.all([
    source('build.gradle.kts'),
    source('web/package.json').then(JSON.parse),
    source('web-vue3/package.json').then(JSON.parse),
    source('web-vue3/package-lock.json').then(JSON.parse),
  ])
  const backendVersion = backend.match(/^version = "([^"]+)"$/m)?.[1]
  assert.ok(backendVersion)
  assert.equal(oldWeb.version, backendVersion)
  assert.equal(vueWeb.version, backendVersion)
  assert.equal(lock.version, backendVersion)
  assert.equal(lock.packages[''].version, backendVersion)
})

test('关于页不得继承 Rust 发布履历或显示后端未提供的统计字段', async () => {
  const [view, types, client] = await Promise.all([
    source('web-vue3/src/views/SettingsView.vue'),
    source('web-vue3/src/types/index.ts'),
    source('web-vue3/src/api/system.ts'),
  ])
  assert.match(view, /import \{ version as VERSION \} from '\.\.\/\.\.\/package\.json'/)
  assert.match(view, /Java\/Kotlin \+ Vue 3/)
  assert.match(view, /服务端版本[\s\S]*?当前接口未提供/)
  assert.match(view, /err\.message\.trim\(\) === '没有备份文件'[\s\S]*?prefMsgError\.value = false/)
  assert.doesNotMatch(view, /Rust \+ Vue 3|v5\.2\.|sysInfo\.(?:port|userCount|bookCount|bookSourceCount)/)
  const systemInfo = types.match(/export interface SystemInfo \{([\s\S]*?)\}/)?.[1]
  assert.ok(systemInfo)
  assert.match(systemInfo, /freeMemory\?: string/)
  assert.doesNotMatch(systemInfo, /port: number|bookCount: number/)
  assert.match(client, /不含版本号/)
})

test('设置页仅在用户主动复制时提供完整 OPDS 令牌地址', async () => {
  const view = await source('web-vue3/src/views/SettingsView.vue')
  assert.match(view, /const opdsDisplayUrl = computed\(\(\) => opdsUrl\.value\.replace\(/)
  assert.match(view, /<span class="row-value mono">\{\{ opdsDisplayUrl \}\}<\/span>/)
  assert.match(view, /async function copyOpdsUrl\(\) \{\s*if \(await copyText\(opdsUrl\.value\)\)/)
  assert.doesNotMatch(view, /<span class="row-value mono">\{\{ opdsUrl \}\}<\/span>/)
})
