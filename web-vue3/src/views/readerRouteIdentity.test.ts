import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { test } from 'node:test'
import { createRenderer, defineComponent, h, nextTick, onMounted, onUnmounted } from 'vue'
import { createMemoryHistory, createRouter, RouterView, useRoute } from 'vue-router'

const appPath = fileURLToPath(new URL('../App.vue', import.meta.url))
const readerPath = fileURLToPath(new URL('./ReaderView.vue', import.meta.url))

/**
 * 使用 Vue 的实际 renderer 验证 key 的组件生命周期语义。宿主节点只记录树关系，
 * 不需要浏览器或真实 ReaderView 的网络/EPUB 依赖。
 */
interface HostNode {
  children: HostNode[]
  parent: HostNode | null
  text?: string
}

const renderer = createRenderer<HostNode, HostNode>({
  patchProp() {},
  insert(child, parent, anchor) {
    child.parent = parent
    const at = anchor ? parent.children.indexOf(anchor) : -1
    if (at < 0) parent.children.push(child)
    else parent.children.splice(at, 0, child)
  },
  remove(child) {
    const parent = child.parent
    if (!parent) return
    const at = parent.children.indexOf(child)
    if (at >= 0) parent.children.splice(at, 1)
    child.parent = null
  },
  createElement: () => ({ children: [], parent: null }),
  createText: (text) => ({ children: [], parent: null, text }),
  createComment: (text) => ({ children: [], parent: null, text }),
  setText(node, text) { node.text = text },
  setElementText(node, text) { node.text = text },
  parentNode: (node) => node.parent,
  nextSibling(node) {
    const parent = node.parent
    if (!parent) return null
    return parent.children[parent.children.indexOf(node) + 1] ?? null
  },
})

async function waitForRoute(router: ReturnType<typeof createRouter>, action: () => void): Promise<void> {
  const settled = new Promise<void>((resolve) => {
    const remove = router.afterEach(() => {
      remove()
      resolve()
    })
  })
  action()
  await settled
  await nextTick()
}

test('真实 Vue Router：同书 query 前进后退不重建，换书卸载 A 并挂载 B', async () => {
  const events: string[] = []
  const Reader = defineComponent({
    setup() {
      const route = useRoute()
      const instanceBookUrl = String(route.params.bookUrl)
      onMounted(() => events.push(`mount:${instanceBookUrl}`))
      onUnmounted(() => events.push(`unmount:${instanceBookUrl}`))
      return () => h('reader', `${route.params.bookUrl}:${route.query.chapter ?? ''}`)
    },
  })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/reader/:bookUrl', name: 'reader', component: Reader }],
  })
  const Root = defineComponent({
    setup() {
      const route = useRoute()
      return () => h(RouterView, null, {
        default: ({ Component }: { Component: ReturnType<typeof h> }) => h(Component, {
          key: route.name === 'reader' ? String(route.params.bookUrl ?? '') : undefined,
        }),
      })
    },
  })
  await router.push({ name: 'reader', params: { bookUrl: 'book-A' }, query: { chapter: '0' } })
  const app = renderer.createApp(Root)
  app.use(router)
  app.mount({ children: [], parent: null })
  await nextTick()

  assert.deepEqual(events, ['mount:book-A'])

  await router.push({ name: 'reader', params: { bookUrl: 'book-A' }, query: { chapter: '1' } })
  await nextTick()
  assert.deepEqual(events, ['mount:book-A'])

  await waitForRoute(router, () => router.back())
  assert.deepEqual(events, ['mount:book-A'])

  await waitForRoute(router, () => router.forward())
  assert.deepEqual(events, ['mount:book-A'])

  await router.push({ name: 'reader', params: { bookUrl: 'book-B' }, query: { chapter: '0' } })
  await nextTick()
  assert.deepEqual(events, ['mount:book-A', 'unmount:book-A', 'mount:book-B'])
  app.unmount()
})

test('实现契约：App 仅以 bookUrl 作 Reader key，Reader 冻结实例书籍身份', async () => {
  const [app, reader] = await Promise.all([readFile(appPath, 'utf8'), readFile(readerPath, 'utf8')])
  assert.match(app, /route\.name === 'reader' \? String\(route\.params\.bookUrl \?\? ''\) : undefined/)
  assert.match(app, /<component :is="Component" :key="routeViewKey\(\)" \/>/)
  assert.doesNotMatch(app, /fullPath/)
  assert.match(reader, /const initialBookUrl = String\(route\.params\.bookUrl \?\? ''\)/)
  assert.match(reader, /const bookUrl = computed\(\(\) => initialBookUrl\)/)
})
