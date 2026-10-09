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
    post: async (route, body) => {
      writes.push({ route, body: JSON.parse(JSON.stringify(body)) })
      if (options.postError) throw options.postError
      return options.postResponse ?? { isSuccess: true, errorMsg: '', data: '' }
    },
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

test('editing sends the complete original snapshot and no display field to one additive endpoint', async () => {
  const item = { id: 1900000000002, name: '生成旧名称', url: 'http://127.0.0.1/old', lastUpdateTime: 7,
    generatedUnknown: { keep: true }, enabledCookieJar: true, jsLib: 'generated-not-executed' }
  const f = fixture({ backend: [item] })
  const original = (await f.api.getHttpTtsList()).data[0]
  const edited = { ...original, name: '生成新名称', url: 'http://127.0.0.1/new' }
  const cacheBefore = f.storage.get('reader_http_tts_list')
  const result = await f.api.updateHttpTts(original, edited)
  assert.equal(result.isSuccess, true)
  assert.equal(f.writes.length, 1, 'Never use save then delete for a rename')
  assert.equal(f.writes[0].route, '/httpTTS/update')
  assert.deepEqual(f.writes[0].body.original, item)
  assert.equal(f.writes[0].body.updated.name, '生成新名称')
  assert.equal(f.writes[0].body.updated.id, item.id)
  assert.equal(f.writes[0].body.updated.enabledCookieJar, true)
  assert.equal(f.writes[0].body.updated.jsLib, 'generated-not-executed')
  assert.equal(Object.hasOwn(f.writes[0].body.updated, 'type'), false)
  assert.equal(Object.hasOwn(f.writes[0].body.updated, 'lastUpdateTime'), false)
  assert.equal(Object.hasOwn(f.writes[0].body.updated, 'generatedUnknown'), false)
  assert.equal(f.storage.get('reader_http_tts_list'), cacheBefore, 'Only a new /list may supply the confirmed timestamp')
  assert.equal(original.name, item.name)
})

test('editing preserves a negative numeric legacy id rather than applying the add adapter mapping', async () => {
  const f = fixture({ backend: [{ id: -4, name: '生成负ID', url: 'http://127.0.0.1/generated' }] })
  const original = (await f.api.getHttpTtsList()).data[0]
  await f.api.updateHttpTts(original, { ...original, id: 'different-preview-id', name: '生成新名称' })
  assert.equal(f.writes[0].body.original.id, -4)
  assert.equal(f.writes[0].body.updated.id, -4)
})

test('edit transport failure retains the old cache and reports no confirmed save', async () => {
  const postError = Object.assign(new Error('Generated unavailable'), { generatedTransportFailure: true })
  const f = fixture({ postError })
  const original = (await f.api.getHttpTtsList()).data[0]
  const before = f.storage.get('reader_http_tts_list')
  const result = await f.api.updateHttpTts(original, { ...original, name: '未确认的名称' })
  assert.equal(result.isSuccess, false)
  assert.match(result.errorMsg, /未确认保存/)
  assert.equal(f.storage.get('reader_http_tts_list'), before)
  assert.equal((await f.api.updateHttpTts(original, original)).isSuccess, false)
  assert.equal(f.writes.length, 1, 'Do not repeat an acknowledged unreachable write')
})

test('a conflicting edit or unsupported old backend does not mutate cache or fall back to legacy save', async () => {
  for (const postError of [new Error('生成重名拒绝'), Object.assign(new Error('Generated old backend 404'),
    { generatedTransportFailure: true, response: { status: 404 } })]) {
    const f = fixture({ postError })
    const original = (await f.api.getHttpTtsList()).data[0]
    const before = f.storage.get('reader_http_tts_list')
    await assert.rejects(f.api.updateHttpTts(original, { ...original, name: '冲突的名称' }), error => error === postError)
    assert.equal(f.storage.get('reader_http_tts_list'), before)
    assert.equal(f.writes.length, 1)
    assert.equal(f.writes[0].route, '/httpTTS/update')
  }
})

test('an unsuccessful edit envelope is preserved and never guessed into local success', async () => {
  const f = fixture({ postResponse: { isSuccess: false, errorMsg: '生成过期编辑', data: '' } })
  const original = (await f.api.getHttpTtsList()).data[0]
  const before = f.storage.get('reader_http_tts_list')
  const result = await f.api.updateHttpTts(original, original)
  assert.equal(result.isSuccess, false)
  assert.equal(result.errorMsg, '生成过期编辑')
  assert.equal(f.storage.get('reader_http_tts_list'), before)
})
