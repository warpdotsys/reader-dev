// Execute the real adapter with generated transports/storage; no HTTP or private data.
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'
import vm from 'node:vm'
import ts from 'typescript'

const source = readFileSync(new URL('./httpTts.ts', import.meta.url), 'utf8')
const tree = ts.createSourceFile('httpTts.ts', source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
const withoutImports = tree.statements.filter(statement => !ts.isImportDeclaration(statement))
const script = withoutImports.map(statement => statement.getText(tree)).join('\n')
const executable = ts.transpileModule(script, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
}).outputText

function fixture(options = {}) {
  const storage = new Map()
  if (options.cached) storage.set('reader_http_tts_list', JSON.stringify(options.cached))
  const writes = []
  const backend = options.backend ?? [{ id: 1900000000001, name: '生成 HTTP 源', url: 'http://127.0.0.1/generated' }]
  const response = { isSuccess: true, errorMsg: '', data: backend, generatedEnvelopeField: 'unchanged' }
  const context = {
    exports: {}, Error,
    axios: { isAxiosError: error => error?.generatedTransportFailure === true },
    get: async route => {
      assert.equal(route, '/httpTTS/list')
      if (options.error) throw options.error
      return response
    },
    post: async (route, body) => { writes.push({ route, body: { ...body } }); return { isSuccess: true, errorMsg: '', data: '' } },
    onBackendReachable: () => {},
    localStorage: { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value) },
  }
  vm.runInNewContext(executable, context)
  return { api: context.exports, storage, writes, backend, response }
}

test('legacy HTTP source without type gets online display default on a successful read', async () => {
  const f = fixture()
  const result = await f.api.getHttpTtsList()
  assert.equal(result.data[0].type, 0)
  assert.equal(result.isSuccess, true)
  assert.equal(result.errorMsg, '')
  assert.equal(result.generatedEnvelopeField, 'unchanged')
  assert.equal(Object.hasOwn(f.backend[0], 'type'), false, 'Do not mutate the server response entity')
})

test('null display type also has the same legacy online default', async () => {
  const f = fixture({ backend: [{ id: 4, name: '生成空类型', url: 'http://127.0.0.1/generated', type: null }] })
  assert.equal((await f.api.getHttpTtsList()).data[0].type, 0)
  assert.equal(f.backend[0].type, null)
})

test('explicit numeric preview type is preserved rather than silently rewritten', async () => {
  const f = fixture({ backend: [{ id: 'preview', name: '生成预览', url: 'http://127.0.0.1/generated', type: 1 }] })
  assert.equal((await f.api.getHttpTtsList()).data[0].type, 1)
})

test('successful read mirrors the display default into only the existing browser cache', async () => {
  const f = fixture()
  await f.api.getHttpTtsList()
  assert.equal(JSON.parse(f.storage.get('reader_http_tts_list'))[0].type, 0)
  assert.equal(f.writes.length, 0)
})

test('generated offline cached legacy entity also has a display default', () => {
  const f = fixture({ cached: [{ id: 9, name: '生成缓存', url: 'http://127.0.0.1/generated' }] })
  assert.equal(f.api.loadHttpTtsList()[0].type, 0)
})

test('online display adaptation preserves complete legacy fields and unknown metadata', async () => {
  const item = { id: 1900000000002, name: '生成完整字段', url: 'http://127.0.0.1/generated',
    contentType: null, concurrentRate: '2', loginUrl: null, loginUi: '[]', header: '{"X-Generated":"one"}',
    jsLib: 'generated-not-executed', enabledCookieJar: false, loginCheckJs: null, lastUpdateTime: 7,
    generatedUnknown: { keep: true } }
  const f = fixture({ backend: [item] })
  const normalized = (await f.api.getHttpTtsList()).data[0]
  assert.deepEqual(JSON.parse(JSON.stringify(normalized)), { ...item, type: 0 })
  assert.equal(f.writes.length, 0)
})

test('a display default never becomes a backend persistence field', async () => {
  const f = fixture()
  const item = (await f.api.getHttpTtsList()).data[0]
  await f.api.saveHttpTts(item)
  assert.equal(f.writes.length, 1)
  assert.equal(f.writes[0].route, '/httpTTS/save')
  assert.equal(Object.hasOwn(f.writes[0].body, 'type'), false)
  assert.equal(f.writes[0].body.id, 1900000000001)
  assert.equal(f.writes[0].body.name, '生成 HTTP 源')
})

test('transport fallback remains explicitly unsuccessful, with cached display normalization', async () => {
  const error = Object.assign(new Error('Generated unavailable transport'), { generatedTransportFailure: true })
  const f = fixture({ error, cached: [{ id: 5, name: '生成离线', url: 'http://127.0.0.1/generated' }] })
  const result = await f.api.getHttpTtsList()
  assert.equal(result.isSuccess, false)
  assert.equal(result.data[0].type, 0)
  assert.match(result.errorMsg, /服务端暂不可用/)
})

test('business failure is not converted into successful display data or an offline write', async () => {
  const error = new Error('Generated business failure')
  const f = fixture({ error })
  await assert.rejects(f.api.getHttpTtsList(), thrown => thrown === error)
  assert.equal(f.writes.length, 0)
  assert.equal(f.storage.size, 0)
})
