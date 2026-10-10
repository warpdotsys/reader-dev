import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { test } from 'node:test'
import vm from 'node:vm'

const webRoot = new URL('../', import.meta.url)

function eventTarget() {
  const listeners = new Map()
  return {
    addEventListener(type, listener) {
      const handlers = listeners.get(type) || []
      handlers.push(listener)
      listeners.set(type, handlers)
    },
    dispatchEvent(event) {
      for (const listener of listeners.get(event.type) || []) listener(event)
      return true
    },
    emit(type, event = { type }) {
      for (const listener of listeners.get(type) || []) listener(event)
    },
  }
}

async function settle() {
  await Promise.resolve()
  await Promise.resolve()
}

/**
 * main.ts deliberately keeps the SW orchestration next to application startup.
 * Run that source with browser-shaped fakes so these tests cover the real event
 * wiring rather than a duplicate implementation in the test itself.
 */
async function bootMain({ controller, registration }) {
  let source = await readFile(new URL('./main.ts', import.meta.url), 'utf8')
  source = source
    .replace(/^import\s.+?\r?$/gm, '')
    .replaceAll('import.meta.env.PROD', 'true')
    .replaceAll('import.meta.env.BASE_URL', "'/'")

  const window = eventTarget()
  const serviceWorker = eventTarget()
  const updateEvents = []
  let reloads = 0
  window.addEventListener('reader:update-ready', (event) => updateEvents.push(event))
  window.location = { reload: () => { reloads += 1 } }

  Object.assign(serviceWorker, {
    controller,
    register: async () => registration,
    getRegistration: async () => registration,
  })

  const noopApp = { use: () => noopApp, directive: () => noopApp, mount: () => noopApp }
  vm.runInNewContext(source, {
    window,
    navigator: { serviceWorker },
    document: {},
    Event: class Event { constructor(type) { this.type = type } },
    console: { warn: () => {} },
    createApp: () => noopApp,
    createPinia: () => ({}),
    ElementPlus: {},
    App: {},
    router: {},
    lazy: {},
  }, { filename: 'main.ts' })

  window.emit('load')
  await settle()
  return { window, serviceWorker, updateEvents, reloads: () => reloads }
}

test('首次安装：不跳过 waiting，claim/controllerchange 也不自动重载正在输入的页面', async () => {
  const installing = eventTarget()
  installing.state = 'installing'
  const registration = Object.assign(eventTarget(), { installing, waiting: null })
  const page = await bootMain({ controller: null, registration })

  registration.emit('updatefound')
  installing.state = 'installed'
  installing.emit('statechange')
  page.serviceWorker.emit('controllerchange')

  assert.equal(page.updateEvents.length, 0)
  assert.equal(page.reloads(), 0)
})

test('后台更新：已有控制器时只通知，不会在登录表单编辑过程中自动重载', async () => {
  const installing = eventTarget()
  installing.state = 'installing'
  const registration = Object.assign(eventTarget(), { installing, waiting: null })
  const page = await bootMain({ controller: {}, registration })

  registration.emit('updatefound')
  installing.state = 'installed'
  installing.emit('statechange')
  page.serviceWorker.emit('controllerchange')

  assert.equal(page.updateEvents.length, 1)
  assert.equal(page.reloads(), 0)
})

test('用户显式应用更新：仅此时向 waiting worker 发 SKIP_WAITING，并在 controllerchange 后重载一次', async () => {
  const waiting = { messages: [], postMessage(message) { this.messages.push(message) } }
  const registration = Object.assign(eventTarget(), { installing: null, waiting })
  const page = await bootMain({ controller: {}, registration })

  page.window.emit('reader:apply-update')
  await settle()
  // 消息对象由 VM 上下文创建，不能直接用跨 realm 的 deepStrictEqual 比较原型。
  assert.equal(waiting.messages.length, 1)
  assert.equal(waiting.messages[0].type, 'SKIP_WAITING')
  assert.equal(page.reloads(), 0)

  page.serviceWorker.emit('controllerchange')
  page.serviceWorker.emit('controllerchange')
  assert.equal(page.reloads(), 1)
})

test('worker：install 不会自行 skipWaiting，只有显式消息才会激活 waiting worker', async () => {
  let source = await readFile(new URL('../public/sw.js', import.meta.url), 'utf8')
  source = source.replaceAll('export ', '')

  const listeners = new Map()
  let skipped = 0
  const self = {
    registration: { scope: 'https://reader.example/' },
    location: { origin: 'https://reader.example' },
    addEventListener(type, listener) { listeners.set(type, listener) },
    skipWaiting() { skipped += 1 },
    clients: { claim: async () => {} },
  }
  const cache = { addAll: async () => {} }
  vm.runInNewContext(source, {
    self,
    URL,
    caches: { open: async () => cache, keys: async () => [] },
    Response,
    fetch: async () => new Response('ok'),
  }, { filename: 'public/sw.js' })

  let installWork
  listeners.get('install')({ waitUntil: (promise) => { installWork = promise } })
  await installWork
  assert.equal(skipped, 0)

  listeners.get('message')({ data: { type: 'SKIP_WAITING' } })
  assert.equal(skipped, 1)
})
