import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { parse } from 'vue/compiler-sfc'
import ts from 'typescript'

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
  const teardown = unmounts.filter(body => body.includes('releaseEpubDoc('))
  assert.equal(teardown.length, 1)
  const body = teardown[0]
  assert.match(body, /readerDisposed\s*=\s*true/)
  assert.match(body, /contentRequestGeneration\+\+/)
  const abort = body.indexOf('epubLoadController?.abort()')
  const detach = body.indexOf('epubDoc.value = null')
  const release = body.indexOf('releaseEpubDoc(released)')
  assert.ok(abort >= 0 && detach > abort && release > detach)
  assert.match(body, /epubTocEntries\.value\s*=\s*\[\]/)
})

test('EPUB终态接线：取消或旧身份加载结果必须释放，活动文档可复用清理契约保留', () => {
  assert.match(script, /if\s*\(controller\.signal\.aborted\s*\|\|\s*epubLoadController !== controller\s*\|\|\s*!requestContext\.isCurrent\(\)\)\s*\{\s*releaseEpubDoc\(loaded\);\s*return\s*\}/)
  assert.doesNotMatch(script, /destroyEpubDoc\(/)
  const modeWatcher = /watch\(\[epubRawActive, shelfBook\],[\s\S]*?\n\}\)/.exec(script)?.[0]
  assert.ok(modeWatcher)
  assert.doesNotMatch(modeWatcher, /releaseEpubDoc\(/)
})
