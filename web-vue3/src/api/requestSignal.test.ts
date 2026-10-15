import assert from 'node:assert/strict'
import { createServer as createLoopbackServer } from 'node:http'
import type { Socket } from 'node:net'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'
import { after, afterEach, test } from 'node:test'
import axios, { type InternalAxiosRequestConfig } from 'axios'
import { createServer } from 'vite'

const sourceRoot = resolve(fileURLToPath(new URL('../', import.meta.url)))
const stubs: Record<string, string> = {
  '@/router': 'export default { currentRoute: { value: { fullPath: \'/\' } }, replace() {} }',
  '@/stores/user': 'const store={accessToken:\'\',username:\'\',isAdmin:false,defaultConfigMode:false,sessionRevision:0,clear(){}}; export const useUserStore=()=>store',
  './requestContext': 'export const readerRequestContext=()=>({params:{},headers:{"X-Reader-Secure-Key":"generated-context-key"},isCurrent:()=>true})',
  '@/utils/legacyNamespace': 'export const assertLegacyNamespace=()=>{}',
  '@/utils/message': 'export const ElMessage={error(){}}',
}
const virtualPrefix = 'virtual:request-signal/'

const vite = await createServer({
  configFile: false,
  root: resolve(sourceRoot, '..'),
  appType: 'custom',
  logLevel: 'silent',
  server: { middlewareMode: true, hmr: false, watch: null },
  plugins: [{
    name: 'request-signal-unit-stubs',
    resolveId(id) { return id.startsWith(virtualPrefix) ? `\0${id}` : null },
    load(id) { return id.startsWith(`\0${virtualPrefix}`) ? stubs[id.slice(virtualPrefix.length + 1)] : null },
    transform(code, id) {
      if (!id.replace(/\\/g, '/').endsWith('/src/api/request.ts')) return null
      return { code: Object.keys(stubs).reduce((text, source) => text.split(source).join(`${virtualPrefix}${source}`), code), map: null }
    },
  }],
})
let requestModule: typeof import('./request.ts')
try {
  requestModule = await vite.ssrLoadModule('/src/api/request.ts') as typeof import('./request.ts')
} catch (error) {
  await vite.close()
  throw error
}
const { default: request, get, post } = requestModule
const originalAdapter = request.defaults.adapter
const originalProxy = request.defaults.proxy

after(async () => { await vite.close() })

function success(config: InternalAxiosRequestConfig) {
  return { data: { isSuccess: true, data: 'generated' }, status: 200, statusText: 'OK', headers: {}, config }
}

afterEach(() => {
  request.defaults.adapter = originalAdapter
  request.defaults.proxy = originalProxy
})

async function within<T>(promise: Promise<T>, label: string, timeoutMs = 1000): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => { timer = setTimeout(() => reject(new Error(`${label} 超时`)), timeoutMs) }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}

test('GET 原样透传 signal，且保留 params、headers、timeout 语义', async () => {
  const controller = new AbortController()
  let seen: InternalAxiosRequestConfig | undefined
  request.defaults.adapter = async (config: InternalAxiosRequestConfig) => {
    seen = config
    return success(config)
  }

  const result = await get<string>('/generated-get', { retained: 'value' }, {
    signal: controller.signal, headers: { 'X-Generated': 'yes' }, timeout: 4321,
  })

  assert.equal(result.data, 'generated')
  assert.ok(seen)
  assert.equal(seen.signal, controller.signal)
  assert.equal(seen.params.retained, 'value')
  assert.equal(seen.headers.get('X-Generated'), 'yes')
  assert.equal(seen.timeout, 4321)
})

test('GET 未提供 signal 时不制造取消信号', async () => {
  let seen: InternalAxiosRequestConfig | undefined
  request.defaults.adapter = async (config: InternalAxiosRequestConfig) => {
    seen = config
    return success(config)
  }

  await get<string>('/generated-no-signal')
  assert.ok(seen)
  assert.equal(seen.signal, undefined)
})

test('显式单次管理密码不被已存管理密码覆盖，大小写不敏感', async () => {
  const observed: InternalAxiosRequestConfig[] = []
  request.defaults.adapter = async (config: InternalAxiosRequestConfig) => {
    observed.push(config)
    return success(config)
  }
  await post('/importLicense', { content: 'generated-key' }, {
    silent: true, headers: { 'x-reader-secure-key': 'generated-once-key' },
  })
  await get('/getUserList')
  assert.equal(observed[0].headers.get('X-Reader-Secure-Key'), 'generated-once-key')
  assert.equal(observed[1].headers.get('X-Reader-Secure-Key'), 'generated-context-key')
  assert.ok(!JSON.stringify(observed[0].params).includes('generated-once-key'))
})

test('POST 既有 signal 透传保持不变', async () => {
  const controller = new AbortController()
  let seen: InternalAxiosRequestConfig | undefined
  request.defaults.adapter = async (config: InternalAxiosRequestConfig) => {
    seen = config
    return success(config)
  }

  await post<string>('/generated-post', { retained: true }, { signal: controller.signal })
  assert.ok(seen)
  assert.equal(seen.signal, controller.signal)
})

test('GET 使用真实 Axios Node HTTP adapter 响应外部 AbortSignal 取消', async () => {
  const controller = new AbortController()
  let requestArrived!: () => void
  let clientClosed!: () => void
  const arrived = new Promise<void>(resolve => { requestArrived = resolve })
  const closed = new Promise<void>(resolve => { clientClosed = resolve })
  let responseTimer: ReturnType<typeof setTimeout> | undefined
  const sockets = new Set<Socket>()
  const server = createLoopbackServer((_req, res) => {
    requestArrived()
    res.once('close', clientClosed)
    responseTimer = setTimeout(() => {
      if (!res.destroyed) res.writeHead(200, { 'content-type': 'application/json' }).end('{"isSuccess":true,"data":"late"}')
    }, 250)
  })
  server.on('connection', socket => {
    sockets.add(socket)
    socket.once('close', () => sockets.delete(socket))
  })

  try {
    await within(new Promise<void>((resolve, reject) => {
      server.once('error', reject)
      server.listen(0, '127.0.0.1', () => resolve())
    }), 'loopback server listen')
    const address = server.address()
    assert.ok(address && typeof address !== 'string')
    request.defaults.adapter = axios.getAdapter('http')
    request.defaults.proxy = false
    const pending = get<string>(`http://127.0.0.1:${address.port}/generated-cancel`, undefined,
      { signal: controller.signal, silent: true })
    const settled = pending.then(
      value => ({ ok: true as const, value }),
      error => ({ ok: false as const, error }),
    )
    await within(arrived, 'loopback request arrival')
    controller.abort()

    const outcome = await within(settled, 'Axios cancellation')
    assert.equal(outcome.ok, false)
    if (outcome.ok) throw new Error('取消后的请求不应成功')
    assert.equal(axios.isCancel(outcome.error) && outcome.error.code === 'ERR_CANCELED', true)
    await within(closed, 'loopback client close')
  } finally {
    if (responseTimer !== undefined) clearTimeout(responseTimer)
    for (const socket of sockets) socket.destroy()
    await within(new Promise<void>((resolve, reject) => server.close(error => {
      if (!error || (error as NodeJS.ErrnoException).code === 'ERR_SERVER_NOT_RUNNING') resolve()
      else reject(error)
    })), 'loopback server close')
  }
})
