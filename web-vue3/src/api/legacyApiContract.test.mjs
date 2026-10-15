import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'

const root = new URL('../../../', import.meta.url)

async function source(path) {
  return readFile(new URL(path, root), 'utf8')
}

test('HTTP TTS 前端请求与 legacy 已注册的路由及实体请求体一致', async () => {
  const [api, controller, client] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('src/main/java/com/htmake/reader/api/controller/HttpTTSController.kt'),
    source('web-vue3/src/api/httpTts.ts'),
  ])

  for (const route of ['list', 'save', 'saveMulti', 'delete', 'deleteMulti']) {
    assert.match(api, new RegExp(`router\\.(?:get|post)\\("/reader3/httpTTS/${route}"`))
    assert.match(client, new RegExp(`['"]/httpTTS/${route}['"]`))
  }
  assert.doesNotMatch(client, /getHttpTTSList|saveHttpTTS'|deleteHttpTTSs/)
  assert.match(client, /list\.map\(toLegacyHttpTts\)/)
  assert.match(client, /toLegacyHttpTts\(tts\)/)
  assert.match(client, /function toLegacyId\(id: string\)/)
  assert.match(client, /return 1_000_000_000_000 \+ \(hash >>> 0\)/)
  // legacy CURD 的实体匹配键是名称。客户端完整保留名称和 URL，
  // 同一项重发仍命中原记录，不同名称的项目不会因 Vue 预览 id 映射发生业务层碰撞。
  assert.match(controller, /return entity\.name == json\.getString\("name"\)/)
  assert.match(client, /name: tts\.name,/)
  assert.match(client, /url: tts\.url,/)
  assert.match(client, /axios\.isAxiosError\(error\) && !error\.response/)
  assert.match(client, /saveHttpTts\(tts: HttpTts\)[\s\S]*?catch \(error\) \{\s*if \(!isTransportFailure\(error\)\) throw error/)
})

test('用户管理调用 getUserList、deleteUsers 和 resetPassword 的实际契约', async () => {
  const [api, client] = await Promise.all([
    source('src/main/java/com/htmake/reader/api/YueduApi.kt'),
    source('web-vue3/src/api/users.ts'),
  ])

  for (const route of ['getUserList', 'deleteUsers', 'resetPassword']) {
    assert.match(api, new RegExp(`"/reader3/${route}"`))
    assert.match(client, new RegExp(`['"]/${route}['"]`))
  }
  assert.match(client, /post<ReaderUser\[\]>\('\/deleteUsers', usernames, managerOptions\(\)\)/)
  assert.match(client, /post\('\/resetPassword', \{ username, password: newPassword \}, managerOptions\(\)\)/)
  assert.doesNotMatch(client, /['"]\/getUsers['"]|['"]\/deleteUser['"]|['"]\/resetUserPassword['"]|\{ username, newPassword \}/)
})

test('管理密钥只走请求头，不拼入用户管理 URL', async () => {
  const [request, users] = await Promise.all([
    source('web-vue3/src/api/request.ts'),
    source('web-vue3/src/api/users.ts'),
  ])

  assert.match(request, /headers\?: Record<string, string>/)
  assert.match(request, /headers: opts\?\.headers/)
  assert.match(request, /'headers' in paramsOrOpts/)
  assert.match(users, /'X-Reader-Secure-Key': key/)
  assert.match(users, /fetch\(`\/reader3\/getUserList\?\$\{params\.toString\(\)\}`, \{[\s\S]*headers: key \? \{ 'X-Reader-Secure-Key': key \}/)
  assert.doesNotMatch(users, /params\.set\('secureKey'/)
})
