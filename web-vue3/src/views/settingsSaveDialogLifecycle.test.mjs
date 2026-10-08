// Execute four actual Settings save lifecycles with generated reactive-ref doubles.
// This is state-machine evidence, not rendered browser acceptance.
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'
import vm from 'node:vm'
import ts from 'typescript'

const vue = readFileSync(new URL('./SettingsView.vue', import.meta.url), 'utf8')
const script = vue.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)?.[1]
assert.ok(script, 'Settings script must be present')
const tree = ts.createSourceFile('SettingsView.ts', script, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
const functions = ['closeAddToc', 'confirmAddToc', 'closeAddTts', 'confirmAddTts',
  'closeEditTts', 'confirmEditTts', 'closeOpdsCfg', 'saveOpdsCfg'].map(name => {
  const node = tree.statements.find(item => ts.isFunctionDeclaration(item) && item.name?.text === name)
  assert.ok(node, `Actual component function ${name} must be present`)
  return node.getText(tree)
}).join('\n')
const executable = ts.transpileModule(functions, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText

function deferred() {
  let resolve, reject
  const promise = new Promise((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

function fixture(options = {}) {
  const calls = { save: [], load: 0, warning: [] }
  const context = {
    tocBusy: { value: false }, tocDialogOpen: { value: true },
    tocForm: { value: { name: '  生成规则  ', rule: '  ^第.+章$  ', enable: true, ...options.form } },
    customTocRules: { value: [{ id: 7 }] },
    document: { body: { style: { overflow: 'hidden' } } },
    newTocId: () => 1900000000001,
    ElMessage: { warning: value => calls.warning.push(value) },
    saveTxtTocRule: async value => { calls.save.push({ ...value }); return await options.save?.(value) },
    loadTxtTocRules: async () => { calls.load++; return await options.load?.() },
  }
  vm.runInNewContext(executable, context)
  return { context, calls, save: () => context.confirmAddToc(), close: () => context.closeAddToc() }
}

test('successful actual Settings save closes its dialog after releasing the busy guard', async () => {
  const f = fixture()
  await f.save()
  assert.equal(f.context.tocBusy.value, false)
  assert.equal(f.context.tocDialogOpen.value, false)
  assert.equal(f.context.document.body.style.overflow, '')
  assert.equal(f.calls.load, 1)
  assert.deepEqual(f.calls.save, [{ id: 1900000000001, name: '生成规则', rule: '^第.+章$', enable: true, serialNumber: 1 }])
})

test('pending save cannot be dismissed, then normal success closes it', async () => {
  const gate = deferred()
  const f = fixture({ save: () => gate.promise })
  const request = f.save()
  f.close()
  assert.equal(f.context.tocBusy.value, true)
  assert.equal(f.context.tocDialogOpen.value, true)
  assert.equal(f.context.document.body.style.overflow, 'hidden')
  assert.equal(f.calls.load, 0)
  gate.resolve()
  await request
  assert.equal(f.context.tocDialogOpen.value, false)
})

test('pending refresh also keeps the guard until all successful work settles', async () => {
  const gate = deferred()
  const entered = deferred()
  const f = fixture({ load: () => { entered.resolve(); return gate.promise } })
  const request = f.save()
  await entered.promise
  assert.equal(f.calls.load, 1)
  f.close()
  assert.equal(f.context.tocBusy.value, true)
  assert.equal(f.context.tocDialogOpen.value, true)
  gate.resolve()
  await request
  assert.equal(f.context.tocDialogOpen.value, false)
  assert.equal(f.context.document.body.style.overflow, '')
})

test('repeated submission during save performs exactly one backend write', async () => {
  const gate = deferred()
  const f = fixture({ save: () => gate.promise })
  const first = f.save()
  await f.save()
  assert.equal(f.calls.save.length, 1)
  assert.equal(f.context.tocBusy.value, true)
  gate.resolve()
  await first
  assert.equal(f.context.tocBusy.value, false)
  assert.equal(f.context.tocDialogOpen.value, false)
})

test('backend rejection releases the guard but preserves dialog and form for retry', async () => {
  const f = fixture({ save: () => { throw new Error('generated save failure') } })
  await f.save()
  assert.equal(f.context.tocBusy.value, false)
  assert.equal(f.context.tocDialogOpen.value, true)
  assert.equal(f.context.document.body.style.overflow, 'hidden')
  assert.equal(f.context.tocForm.value.name, '  生成规则  ')
  assert.equal(f.calls.load, 0)
})

test('refresh rejection does not escape the event handler or falsely close a completed form', async () => {
  const f = fixture({ load: () => { throw new Error('generated refresh failure') } })
  await f.save()
  assert.equal(f.context.tocBusy.value, false)
  assert.equal(f.context.tocDialogOpen.value, true)
  assert.equal(f.context.document.body.style.overflow, 'hidden')
})

test('empty rule makes no backend call and does not close the dialog', async () => {
  const f = fixture({ form: { rule: '  ' } })
  await f.save()
  assert.equal(f.calls.save.length, 0)
  assert.equal(f.calls.load, 0)
  assert.equal(f.context.tocBusy.value, false)
  assert.equal(f.context.tocDialogOpen.value, true)
  assert.deepEqual(f.calls.warning, ['规则正则不能为空'])
})

test('empty name falls back to the regex and an explicitly disabled rule remains disabled', async () => {
  const f = fixture({ form: { name: ' ', enable: false } })
  await f.save()
  assert.equal(f.calls.save[0].name, '^第.+章$')
  assert.equal(f.calls.save[0].enable, false)
  assert.equal(f.context.tocDialogOpen.value, false)
})

// OPDS uses generated transport doubles here. Its real backend remains unavailable.
const variants = [
  { name: 'TTS add', save: 'confirmAddTts', close: 'closeAddTts', busy: 'ttsBusy', open: 'ttsDialogOpen', throws: false },
  { name: 'TTS edit', save: 'confirmEditTts', close: 'closeEditTts', busy: 'ttsSaving', open: 'ttsEditing', throws: false },
  { name: 'OPDS generated lifecycle', save: 'saveOpdsCfg', close: 'closeOpdsCfg', busy: 'opdsCfgBusy', open: 'opdsCfgOpen', throws: false },
]

function otherFixture(kind, options = {}) {
  const calls = { save: [], load: 0, warning: [], success: [] }
  const context = {
    Error,
    ttsBusy: { value: false }, ttsDialogOpen: { value: true },
    ttsForm: { value: { name: '生成语音', url: 'https://generated.invalid/tts', type: 0 } },
    ttsSaving: { value: false },
    ttsEditing: { value: { id: 'generated-only', name: '生成语音', url: 'https://generated.invalid/tts' } },
    opdsCfgBusy: { value: false }, opdsCfgOpen: { value: true },
    opdsCfg: { value: { enabled: false, username: '', passwordSet: false } },
    opdsForm: { value: { username: 'generated-opds', password: 'GeneratedOnly-2026' } },
    opdsCfgMsg: { value: '' }, opdsCfgMsgError: { value: false }, opdsTestPassword: '',
    document: { body: { style: { overflow: 'hidden' } } },
    newTtsId: () => 'generated-only',
    ElMessage: { warning: value => calls.warning.push(value), success: value => calls.success.push(value) },
    saveHttpTts: async value => { calls.save.push({ ...value }); return await options.save?.() ?? { isSuccess: true, data: '' } },
    loadTtsList: async () => { calls.load++; return await options.load?.() },
    saveOpdsSettings: async (username, password) => {
      calls.save.push({ username, password })
      return await options.save?.() ?? { isSuccess: true, data: { enabled: true, username } }
    },
  }
  vm.runInNewContext(executable, context)
  return { calls, context, save: () => context[kind.save](), close: () => context[kind.close](),
    isOpen: () => !!context[kind.open].value, isBusy: () => context[kind.busy].value }
}

for (const kind of variants) {
  test(`${kind.name}: successful source function closes only after busy guard is released`, async () => {
    const f = otherFixture(kind)
    await f.save()
    assert.equal(f.isBusy(), false)
    assert.equal(f.isOpen(), false)
    assert.equal(f.calls.save.length, 1)
    assert.equal(f.context.document.body.style.overflow, '')
  })

  test(`${kind.name}: pending write rejects dismissal and duplicate submission`, async () => {
    const gate = deferred()
    const f = otherFixture(kind, { save: () => gate.promise })
    const first = f.save()
    f.close()
    await f.save()
    assert.equal(f.isBusy(), true)
    assert.equal(f.isOpen(), true)
    assert.equal(f.calls.save.length, 1)
    assert.equal(f.context.document.body.style.overflow, 'hidden')
    gate.resolve()
    await first
    assert.equal(f.isBusy(), false)
    assert.equal(f.isOpen(), false)
  })

  test(`${kind.name}: rejection retains the form and never falsely closes`, async () => {
    const f = otherFixture(kind, { save: () => { throw new Error('generated transport rejection') } })
    if (kind.throws) await assert.rejects(f.save(), /generated transport rejection/)
    else await f.save()
    assert.equal(f.isBusy(), false)
    assert.equal(f.isOpen(), true)
    assert.equal(f.calls.load, 0)
    assert.equal(f.context.document.body.style.overflow, 'hidden')
    if (kind.busy === 'opdsCfgBusy') {
      assert.equal(f.context.opdsCfgMsg.value, 'generated transport rejection')
      assert.equal(f.context.opdsCfgMsgError.value, true)
    }
  })

  test(`${kind.name}: invalid form performs no write and remains open`, async () => {
    const f = otherFixture(kind)
    f.context.ttsForm.value.url = ''
    f.context.ttsEditing.value.url = ''
    f.context.opdsForm.value.password = '123'
    await f.save()
    assert.equal(f.isBusy(), false)
    assert.equal(f.isOpen(), true)
    assert.equal(f.calls.save.length, 0)
  })
}

test('TTS add retains the explicit offline-cache warning and then closes normally', async () => {
  const f = otherFixture(variants[0], { save: async () => ({ isSuccess: false, data: '' }) })
  await f.save()
  assert.deepEqual(f.calls.warning, ['服务端不可用，听书源已暂存到当前浏览器'])
  assert.equal(f.isOpen(), false)
})

test('hosted browser workflow selects import and dialog journeys and rejects skipped reports', () => {
  const workflow = readFileSync(new URL('../../../.github/workflows/vue3-preview.yml', import.meta.url), 'utf8')
  const enforced = workflow.match(/for name in (.*); do/)[1].split(/\s+/)
  for (const name of ['BookshelfImport', 'TxtTocRule', 'SettingsDialog']) {
    assert.ok(workflow.includes(`--tests 'com.medwarp.reader.browserpoc.Vue3Preview${name}Test'`))
    assert.ok(enforced.includes(name))
  }
  assert.ok(workflow.includes(`grep -q 'tests="1" skipped="0" failures="0" errors="0"'`))
  assert.ok(workflow.includes('vue3-txt-rule-generated-*.png'))
  assert.ok(workflow.includes('vue3-settings-generated-*.png'))
})
