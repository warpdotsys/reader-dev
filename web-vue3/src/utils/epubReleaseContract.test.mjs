import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { parse } from 'vue/compiler-sfc'
import ts from 'typescript'
import vm from 'node:vm'
import { releaseEpubDoc } from './epubLoader.ts'

const view = readFileSync(new URL('../views/ReaderView.vue', import.meta.url), 'utf8')
const script = parse(view).descriptor.scriptSetup.content
const source = ts.createSourceFile('ReaderView.ts', script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
const unmounts = []
function visit(node) {
  if (ts.isCallExpression(node) && node.expression.getText(source) === 'onBeforeUnmount') {
    const callback = node.arguments[0]
    if (callback && ts.isArrowFunction(callback)) unmounts.push(callback.body.getText(source))
  }
  ts.forEachChild(node, visit)
}
visit(source)

test('EPUB终态接线：先取消加载并清持有引用，再最终释放已加载文档', () => {
  const teardown = unmounts.filter(body => body.includes('releaseEpubReadResources()'))
  assert.equal(teardown.length, 1)
  const body = teardown[0]
  assert.match(body, /readerDisposed\s*=\s*true/)
  assert.match(body, /contentRequestGeneration\+\+/)
  const cleanup = source.statements.find(node => ts.isFunctionDeclaration(node) && node.name?.text === 'releaseEpubReadResources')?.body.getText(source)
  assert.ok(cleanup)
  const abort = cleanup.indexOf('epubLoadController?.abort()')
  const detach = cleanup.indexOf('epubDoc.value = null')
  const release = cleanup.indexOf('releaseEpubDoc(released)')
  assert.ok(abort >= 0 && detach > abort && release > detach)
  assert.match(cleanup, /epubTocEntries\.value\s*=\s*\[\]/)
  assert.match(body, /removeEventListener\('pagehide', onReaderPageHide\)/)
})

function pageLifecycle(t) {
  const doc = { files: new Map([['generated', new Uint8Array(1024)]]),
    manifest: new Map([['generated', {}]]), spine: [{}], opfDir: 'generated',
    blobUrls: new Map([['generated', 'blob:generated']]), cssBytes: 8 }
  const controller = new AbortController()
  const revoked = []
  t.mock.method(URL, 'revokeObjectURL', url => revoked.push(url))
  const context = vm.createContext({ epubDocumentClosing: false, epubLoadController: controller,
    epubDocLoading: { value: true }, epubDoc: { value: doc }, epubTocEntries: { value: [1] }, releaseEpubDoc })
  const definitions = source.statements.filter(node => ts.isFunctionDeclaration(node)
    && ['releaseEpubReadResources', 'onReaderPageHide'].includes(node.name?.text)).map(node => node.getText(source))
  assert.equal(definitions.length, 2)
  vm.runInContext(ts.transpileModule(definitions.join('\n'), {
    compilerOptions: { target: ts.ScriptTarget.ES2022 },
  }).outputText, context)
  return { context, doc, controller, revoked }
}

test('整页离开：真实清空资源表和blob，取消旧加载，拒绝后续重新加载', t => {
  const { context, doc, controller, revoked } = pageLifecycle(t)
  context.onReaderPageHide({ persisted: false })
  assert.equal(controller.signal.aborted, true)
  assert.equal(context.epubLoadController, null)
  assert.equal(context.epubDoc.value, null)
  assert.equal(context.epubDocLoading.value, false)
  assert.equal(context.epubTocEntries.value.length, 0)
  assert.equal(context.epubDocumentClosing, true)
  assert.equal(doc.files.size, 0)
  assert.equal(doc.manifest.size, 0)
  assert.equal(doc.spine.length, 0)
  assert.deepEqual(revoked, ['blob:generated'])
  assert.match(script, /if \(readerDisposed \|\| epubDocumentClosing \|\|/)
})

test('后退缓存：保留文档和加载，不撤销仍需恢复的资源', t => {
  const { context, doc, controller, revoked } = pageLifecycle(t)
  context.onReaderPageHide({ persisted: true })
  assert.equal(context.epubDoc.value, doc)
  assert.equal(doc.files.size, 1)
  assert.equal(controller.signal.aborted, false)
  assert.equal(context.epubDocumentClosing, false)
  assert.deepEqual(revoked, [])
})

test('整页退出后再次卸载：重复收尾不会二次撤销或保留旧字节', t => {
  const { context, doc, revoked } = pageLifecycle(t)
  context.onReaderPageHide({ persisted: false })
  context.releaseEpubReadResources()
  assert.equal(doc.files.size, 0)
  assert.deepEqual(revoked, ['blob:generated'])
  assert.match(script, /addEventListener\('pagehide', onReaderPageHide\)/)
})

test('EPUB终态接线：取消或旧身份加载结果必须释放，活动文档可复用清理契约保留', () => {
  assert.match(script, /if\s*\(controller\.signal\.aborted\s*\|\|\s*epubLoadController !== controller\s*\|\|\s*!requestContext\.isCurrent\(\)\)\s*\{\s*releaseEpubDoc\(loaded\);\s*return\s*\}/)
  assert.doesNotMatch(script, /destroyEpubDoc\(/)
  const modeWatcher = /watch\(\[epubRawActive, shelfBook\],[\s\S]*?\n\}\)/.exec(script)?.[0]
  assert.ok(modeWatcher)
  assert.doesNotMatch(modeWatcher, /releaseEpubDoc\(/)
})
