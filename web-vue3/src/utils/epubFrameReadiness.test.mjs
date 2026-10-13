import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import vm from 'node:vm'

const root = new URL('../../../', import.meta.url)
const code = readFileSync(new URL('browser-poc/src/test/resources/epub-frame-readiness.js', root), 'utf8')

function fixture() {
  let parsed = 0
  const current = { body: {}, documentElement: { outerHTML: '<html>generated-current</html>' },
    readyState: 'complete', fonts: { status: 'loaded' } }
  const expected = { documentElement: { outerHTML: current.documentElement.outerHTML } }
  const frame = { contentDocument: current, srcdoc: '<!doctype html><html>generated-current</html>', busy: 'false',
    getAttribute(name) { assert.equal(name, 'aria-busy'); return this.busy } }
  let selected = frame
  class Parser {
    parseFromString(value, type) {
      parsed++; assert.equal(value, frame.srcdoc); assert.equal(type, 'text/html'); return expected
    }
  }
  const ready = vm.runInNewContext('(' + code + ')', { document: {
    querySelector(selector) { assert.equal(selector, 'iframe.epub-frame'); return selected },
  }, DOMParser: Parser })
  return { current, expected, frame, ready, get parsed() { return parsed }, set selected(value) { selected = value } }
}

test('EPUB 就绪：完整当前文档、已加载字体、恢复完成和规范化 srcdoc 同时满足', () => {
  const f = fixture(), before = JSON.stringify(f.current)
  assert.equal(f.ready(), true); assert.equal(f.parsed, 1)
  assert.equal(JSON.stringify(f.current), before)
})

test('EPUB 就绪：旧章即使已有足量正文、URL 已变化，DOM 未提交仍拒绝', () => {
  const f = fixture(); f.current.body.innerText = '生成旧章正文'.repeat(1000)
  f.expected.documentElement.outerHTML = '<html>generated-new-chapter</html>'
  assert.equal(f.ready(), false)
})

test('EPUB 就绪：忙碌、缺失或无效属性不会提前解析文档', () => {
  for (const value of ['true', null, undefined, '', 'False']) {
    const f = fixture(); f.frame.busy = value
    assert.equal(f.ready(), false); assert.equal(f.parsed, 0)
  }
})

test('EPUB 就绪：加载中、字体未完成和空 srcdoc 均拒绝', () => {
  for (const change of [f => f.current.readyState = 'loading', f => f.current.readyState = 'interactive',
    f => f.current.fonts.status = 'loading', f => f.current.fonts = undefined,
    f => f.frame.srcdoc = '', f => f.frame.srcdoc = undefined]) {
    const f = fixture(); change(f); assert.equal(f.ready(), false); assert.equal(f.parsed, 0)
  }
})

test('EPUB 就绪：iframe、body 或文档缺失不等于完成', () => {
  for (const change of [f => f.selected = null, f => f.frame.contentDocument = null,
    f => f.current.body = null, f => f.current.documentElement = null,
    f => f.expected.documentElement = null]) {
    const f = fixture(); change(f); assert.equal(f.ready(), false)
  }
})

test('EPUB 就绪：同源访问或解析异常只返回未就绪，不改变安全设置', () => {
  const f = fixture(); Object.defineProperty(f.frame, 'contentDocument', { get() { throw Error('generated cross-origin') } })
  assert.equal(f.ready(), false)
})

test('EPUB 就绪：判据接入完整镜像与本地原版、锚点和滚动验收，不延长超时', () => {
  const helper = readFileSync(new URL('browser-poc/src/test/java/com/medwarp/reader/browserpoc/RawEpubReadiness.java', root), 'utf8')
  const native = readFileSync(new URL('browser-poc/src/test/java/com/medwarp/reader/browserpoc/NativeImageDefaultUiTest.java', root), 'utf8')
  const local = readFileSync(new URL('browser-poc/src/test/java/com/medwarp/reader/browserpoc/Vue3PreviewLocalReadingTest.java', root), 'utf8')
  assert.match(helper, /getResourceAsStream\("\/epub-frame-readiness\.js"\)/)
  assert.match(helper, /page\.waitForFunction\(new String\(bytes, StandardCharsets\.UTF_8\)\)/)
  assert.doesNotMatch(helper, /setDefaultTimeout|setTimeout|WaitForFunctionOptions/)
  assert.ok((native.match(/RawEpubReadiness\.waitFor\(page\)/g) || []).length >= 3)
  assert.match(local, /private static void waitRawReady\(Page page\) \{\s*RawEpubReadiness\.waitFor\(page\);\s*\}/)
  assert.ok((local.match(/RawEpubReadiness\.waitFor\(page\)/g) || []).length >= 2)
})
