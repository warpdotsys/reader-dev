// Execute actual component functions with generated API/ref/timer doubles.
// This is lifecycle evidence, not real-browser memory or EPUB acceptance.
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'
import vm from 'node:vm'
import { parse } from 'vue/compiler-sfc'
import ts from 'typescript'

const script = parse(readFileSync(new URL('./BookshelfView.vue', import.meta.url), 'utf8')).descriptor.scriptSetup.content
const tree = ts.createSourceFile('BookshelfView.ts', script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
const functions = ['isCurrentImportSession', 'clearImportCloseTimer', 'scheduleImportClose',
  'disposeImportSession', 'checkPreview', 'isSupported', 'openImport', 'closeImport', 'addFiles',
  'discardUnconfirmedPreview', 'discardPreview', 'removeItem', 'checkImportDuplicates', 'startUpload']
const executable = ts.transpileModule(functions.map(name => {
  const node = tree.statements.find(value => ts.isFunctionDeclaration(value) && value.name?.text === name)
  assert.ok(node, `Actual component function ${name} must exist`)
  return node.getText(tree)
}).join('\n'), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None } }).outputText

function deferred() {
  let resolve, reject
  const promise = new Promise((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

function generatedItem(status = 'pending', id = 'generated') {
  return { file: { name: `${id}.epub`, size: 123 }, status, progress: 0,
    preview: { book: { name: id, origin: 'local', bookUrl: `/assets/generated/${id}.epub` }, chapters: [] } }
}

function fixture(options = {}) {
  const calls = { discard: [], save: [], refresh: 0, confirm: 0, warning: [], info: [] }
  const timers = new Map()
  let sequence = 0
  const context = {
    Error, shelfDisposed: false, importSession: 1, importCloseTimer: undefined,
    importOpen: { value: true }, uploadBusy: { value: false }, importDone: { value: false },
    importSummary: { value: '' }, acceptTip: { value: '' }, uploadIndex: { value: 0 },
    importItems: { value: options.items ?? [generatedItem()] }, books: { value: [] },
    dialogRef: { value: null }, reactive: value => value,
    nextTick: async callback => callback(), document: { body: { style: { overflow: 'hidden' } } },
    window: {
      setTimeout: callback => { const id = ++sequence; timers.set(id, callback); return id },
      clearTimeout: id => timers.delete(id),
    },
    ElMessage: { warning: value => calls.warning.push(value), info: value => calls.info.push(value) },
    ElMessageBox: { confirm: async () => { calls.confirm++; return await options.confirm?.() } },
    getBookshelf: async () => ({ data: options.shelf ?? [] }),
    importBookPreview: async file => await options.preview?.(file) ?? { data: [generatedItem().preview] },
    discardImportPreview: async preview => { calls.discard.push(preview); return await options.discard?.(preview) },
    saveBook: async book => { calls.save.push(book); return await options.save?.(book) },
    load: async () => { calls.refresh++; return await options.load?.() },
  }
  context.previewChecking = { get value() { return context.importItems.value.some(item => item.status === 'preparing') } }
  context.hasPending = { get value() { return context.importItems.value.some(item => item.status === 'pending') } }
  vm.runInNewContext(executable, context)
  return { context, calls, timers, dispose() { context.shelfDisposed = true; context.disposeImportSession() } }
}

test('close detaches File/preview list and cleans only never-submitted assets', async () => {
  const pending = generatedItem(), done = generatedItem('done', 'saved')
  const uncertain = generatedItem('error', 'uncertain'); uncertain.saveAttempted = true
  const f = fixture({ items: [pending, done, uncertain] })
  f.context.closeImport()
  await Promise.resolve()
  assert.equal(f.context.importOpen.value, false)
  assert.equal(f.context.importItems.value.length, 0)
  assert.equal(f.context.document.body.style.overflow, '')
  assert.equal(f.calls.discard.length, 1)
  assert.equal(f.calls.discard[0].book.name, 'generated')
  for (const item of [pending, done, uncertain]) assert.equal(item.preview, undefined)
})

test('late preview after unmount is discarded without reattaching old state', async () => {
  const gate = deferred(), item = generatedItem('preparing')
  delete item.preview
  const f = fixture({ items: [item], preview: () => gate.promise })
  const request = f.context.checkPreview(item, 1)
  f.dispose()
  gate.resolve({ data: [generatedItem().preview] })
  await request
  assert.equal(item.preview, undefined)
  assert.equal(item.status, 'preparing')
  assert.equal(f.context.importItems.value.length, 0)
  assert.equal(f.calls.discard.length, 1)
})

test('removed preparing item cannot reappear when its preview completes', async () => {
  const gate = deferred(), item = generatedItem('preparing'); delete item.preview
  const f = fixture({ items: [item], preview: () => gate.promise })
  const request = f.context.checkPreview(item, 1)
  f.context.removeItem(0)
  gate.resolve({ data: [generatedItem().preview] })
  await request
  assert.equal(f.context.importItems.value.length, 0)
  assert.equal(item.preview, undefined)
  assert.equal(f.calls.discard.length, 1)
})

test('old queued close cannot dismiss a reopened dialog or erase its new timer', () => {
  const f = fixture()
  f.context.scheduleImportClose(1)
  const obsolete = f.timers.get(f.context.importCloseTimer)
  f.context.closeImport(); f.context.openImport()
  f.context.scheduleImportClose(f.context.importSession)
  const currentTimer = f.context.importCloseTimer
  obsolete()
  assert.equal(f.context.importOpen.value, true)
  assert.equal(f.context.importCloseTimer, currentTimer)
  f.timers.get(currentTimer)()
  assert.equal(f.context.importOpen.value, false)
  assert.equal(f.context.importItems.value.length, 0)
})

test('selecting another file cancels even an already queued automatic close', async () => {
  const gate = deferred(), f = fixture({ preview: () => gate.promise })
  f.context.scheduleImportClose(1)
  const obsolete = f.timers.get(f.context.importCloseTimer)
  f.context.addFiles([{ name: 'second.epub', size: 123 }])
  obsolete()
  assert.equal(f.context.importOpen.value, true)
  assert.equal(f.context.importItems.value.length, 2)
  gate.resolve({ data: [generatedItem('pending', 'second').preview] })
  await Promise.resolve(); await Promise.resolve()
  f.context.closeImport()
})

test('duplicate confirmation locks repeated submission and close until it settles', async () => {
  const gate = deferred(), f = fixture({ shelf: [{ name: 'generated' }], confirm: () => gate.promise })
  const request = f.context.startUpload()
  await Promise.resolve(); await Promise.resolve()
  await f.context.startUpload(); f.context.closeImport()
  assert.equal(f.context.uploadBusy.value, true)
  assert.equal(f.context.importOpen.value, true)
  assert.equal(f.calls.confirm, 1)
  assert.equal(f.calls.save.length, 0)
  gate.resolve()
  await request
  assert.equal(f.calls.save.length, 1)
  assert.equal(f.context.uploadBusy.value, false)
  assert.equal(f.calls.refresh, 1)
})

test('unmount while confirming stops all saves and disposes unsubmitted assets', async () => {
  const gate = deferred(), f = fixture({ shelf: [{ name: 'generated' }], confirm: () => gate.promise })
  const request = f.context.startUpload()
  await Promise.resolve(); await Promise.resolve()
  f.dispose(); gate.resolve()
  await request
  assert.equal(f.calls.save.length, 0)
  assert.equal(f.calls.discard.length, 1)
  assert.equal(f.calls.refresh, 0)
  assert.equal(f.timers.size, 0)
})

for (const outcome of ['success', 'lost-response']) {
  test(`unmount during ${outcome} save preserves that asset and cancels remaining saves`, async () => {
    const gate = deferred(), entered = deferred()
    const first = generatedItem(), second = generatedItem('pending', 'second')
    const f = fixture({ items: [first, second], save: () => { entered.resolve(); return gate.promise } })
    const request = f.context.startUpload()
    await entered.promise
    assert.equal(first.saveAttempted, true)
    f.dispose()
    if (outcome === 'success') gate.resolve()
    else gate.reject(new Error('generated lost response'))
    await request
    assert.equal(f.calls.save.length, 1)
    assert.deepEqual(f.calls.discard.map(value => value.book.name), ['second'])
    assert.equal(f.calls.refresh, 0)
    assert.equal(f.context.importItems.value.length, 0)
    assert.equal(f.context.importDone.value, false)
    assert.equal(f.timers.size, 0)
  })
}

test('retry does not re-save previously successful rows; final close preserves both assets', async () => {
  const f = fixture({ items: [generatedItem('done', 'already-saved'), generatedItem()] })
  await f.context.startUpload()
  assert.deepEqual(f.calls.save.map(book => book.name), ['generated'])
  assert.equal(f.context.importSummary.value, '导入完成，共 2 本')
  f.timers.get(f.context.importCloseTimer)()
  assert.equal(f.calls.discard.length, 0)
  assert.equal(f.context.importItems.value.length, 0)
})

test('uncertain failed save retains the file for retry and never auto-deletes its asset', async () => {
  const item = generatedItem(), f = fixture({ items: [item], save: () => { throw new Error('generated loss') } })
  await f.context.startUpload()
  assert.equal(item.status, 'error')
  assert.equal(item.saveAttempted, true)
  assert.equal(item.preview.book.name, 'generated')
  assert.equal(f.context.uploadBusy.value, false)
  assert.equal(f.timers.size, 0)
  f.context.closeImport()
  assert.equal(f.calls.discard.length, 0)
})

test('refresh failure releases the submit guard without falsely scheduling a close', async () => {
  const f = fixture({ load: () => { throw new Error('generated refresh error') } })
  await assert.rejects(f.context.startUpload(), /generated refresh error/)
  assert.equal(f.context.uploadBusy.value, false)
  assert.equal(f.timers.size, 0)
  f.context.closeImport()
  assert.equal(f.calls.discard.length, 0)
})

test('preview cleanup rejection is handled and reported without an unhandled promise', async () => {
  const f = fixture({ discard: () => { throw new Error('generated delete error') } })
  f.context.closeImport()
  await new Promise(resolve => setImmediate(resolve))
  assert.equal(f.calls.warning.length, 1)
  assert.equal(f.calls.discard.length, 1)
})

test('unmount wiring disposes the import session after invalidating the shelf', () => {
  const bodies = []
  function visit(node) {
    if (ts.isCallExpression(node) && node.expression.getText(tree) === 'onBeforeUnmount') {
      bodies.push(node.arguments[0].getText(tree))
    }
    ts.forEachChild(node, visit)
  }
  visit(tree)
  const cleanup = bodies.find(body => body.includes('disposeImportSession()'))
  assert.ok(cleanup)
  assert.ok(cleanup.indexOf('shelfDisposed = true') < cleanup.indexOf('disposeImportSession()'))
})
