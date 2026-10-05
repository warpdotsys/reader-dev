/**
 * C2: epubLoader unit tests (in-memory minimal EPUB via fflate.zipSync).
 */
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { zipSync, strToU8 } from 'fflate'
import { expandEpubArchive } from './epubArchive.ts'
import type { EpubArchiveWorker } from './epubWorker.ts'
import {
  parseEpubBytes,
  resolveHref,
  destroyEpubDoc,
  type EpubDoc,
  loadEpubDoc,
  epubFileLocation,
  epubChapterPath,
  epubChapterLocation,
  epubFragment,
  resolveEpubLink,
  epubNavigationIndex,
} from './epubLoader.ts'

/** Build a minimal EPUB byte stream (one nav-less OPF, two spine chapters) */
function buildMinimalEpub(): Uint8Array {
  const opf = `<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="id">urn:uuid:test</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="text/chapter1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href='text/chapter2.xhtml' media-type="application/xhtml+xml"/>
    <item id="css" href="style/main.css" media-type="text/css"/>
    <item id="img" href="images/cover.png" media-type="image/png"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2" linear="no"/>
  </spine>
</package>`
  return zipSync({
    mimetype: strToU8('application/epub+zip'),
    'META-INF/container.xml': strToU8(
      `<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>`,
    ),
    'OEBPS/content.opf': strToU8(opf),
    'OEBPS/text/chapter1.xhtml': strToU8('<html><body><p>Chapter one</p></body></html>'),
    'OEBPS/text/chapter2.xhtml': strToU8('<html><body><p>Chapter two</p></body></html>'),
    'OEBPS/style/main.css': strToU8('body{color:red}'),
    'OEBPS/images/cover.png': new Uint8Array([137, 80, 78, 71]),
  })
}

test('C2: parseEpubBytes resolves OPF via container.xml and extracts manifest/spine', () => {
  const doc = parseEpubBytes(buildMinimalEpub())
  // manifest entries resolved relative to OEBPS/
  assert.equal(doc.manifest.size, 4)
  assert.equal(doc.manifest.get('ch1')?.href, 'OEBPS/text/chapter1.xhtml')
  assert.equal(doc.manifest.get('css')?.mediaType, 'text/css')
  assert.equal(doc.manifest.get('img')?.href, 'OEBPS/images/cover.png')
  // spine order preserved; linear=no flagged but still listed (rendering decides)
  assert.deepEqual(
    doc.spine.map((s) => s.idref),
    ['ch1', 'ch2'],
  )
  assert.equal(doc.spine[0]?.linear, true)
  assert.equal(doc.spine[1]?.linear, false)
})

test('C2: parseEpubBytes tolerates single-quoted attributes and percent-encoded hrefs', () => {
  const bytes = zipSync({
    'META-INF/container.xml': strToU8(
      `<?xml version="1.0"?><rootfiles><rootfile full-path='book.opf'/></rootfiles>`,
    ),
    'book.opf': strToU8(
      `<package><manifest><item id="a" href='%E6%B5%8B%E8%AF%95.xhtml' media-type="application/xhtml+xml"/></manifest><spine><itemref idref="a"/></spine></package>`,
    ),
    '测试.xhtml': strToU8('<p/>'),
  })
  const doc = parseEpubBytes(bytes)
  assert.equal(doc.spine.length, 1)
  // decoded to the actual zip key
  assert.equal(doc.manifest.get('a')?.href, '测试.xhtml')
})

test('C2: resolveHref handles ./ ../ and nested base dirs', () => {
  assert.equal(resolveHref('', 'a.html'), 'a.html')
  assert.equal(resolveHref('OEBPS', './text/x.html'), 'OEBPS/text/x.html')
  assert.equal(resolveHref('OEBPS/text', '../style/main.css'), 'OEBPS/style/main.css')
  assert.equal(resolveHref('a/b', '../../top.html'), 'top.html')
})

test('C2: missing container.xml throws with clear message', () => {
  const bytes = zipSync({ 'x.html': strToU8('<p/>') })
  assert.throws(() => parseEpubBytes(bytes), /container\.xml/)
})

test('C2: destroyEpubDoc clears blob URL table (no-op without URLs)', () => {
  const doc: EpubDoc = parseEpubBytes(buildMinimalEpub())
  assert.equal(doc.blobUrls.size, 0)
  destroyEpubDoc(doc)
  assert.equal(doc.blobUrls.size, 0)
})

test('legacy EPUB home mapping preserves owner and physical originName', () => {
  assert.deepEqual(epubFileLocation('storage/data/alice/书 EPUB.epub', 'alice'),
    { home: '__HOME__', path: '书 EPUB.epub' })
  assert.deepEqual(epubFileLocation('storage\\data\\alice\\webdav\\书.epub', 'alice'),
    { home: '__WEBDAV__', path: '书.epub' })
  assert.deepEqual(epubFileLocation('storage/localStore/书.epub', 'alice'),
    { home: '__LOCAL_STORE__', path: '书.epub' })
  for (const path of ['storage/data/bob/书.epub', 'storage/data/alice/../bob/a.epub',
    '/storage/data/alice/a.epub', 'https://other/a.epub', 'storage/data/alice//a.epub']) {
    assert.throws(() => epubFileLocation(path, 'alice'))
  }
})

for (const directory of [true, false]) {
  test(`legacy EPUB ${directory ? 'directory/index.epub' : 'direct file'} carries token and relative path`, async t => {
    const requests: URL[] = []
    // 此单元只验证 legacy 下载契约；实际打包 Worker 另由浏览器旅程执行。
    const workerDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'Worker')
    class FixtureWorker implements EpubArchiveWorker {
      onmessage: EpubArchiveWorker['onmessage'] = null
      onerror: EpubArchiveWorker['onerror'] = null
      postMessage(data: ArrayBuffer) {
        queueMicrotask(() => this.onmessage?.({ data: { files: [...expandEpubArchive(new Uint8Array(data))] } } as MessageEvent))
      }
      terminate() { this.onmessage = this.onerror = null }
    }
    Object.defineProperty(globalThis, 'Worker', { value: FixtureWorker, configurable: true })
    t.after(() => {
      if (workerDescriptor) Object.defineProperty(globalThis, 'Worker', workerDescriptor)
      else Reflect.deleteProperty(globalThis, 'Worker')
    })
    t.mock.method(globalThis, 'fetch', async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(String(input), 'http://localhost')
      requests.push(url)
      assert.equal(init?.credentials, 'same-origin')
      assert.equal(init?.cache, 'no-store')
      assert.equal(url.searchParams.get('home'), '__HOME__')
      assert.equal(url.searchParams.get('accessToken'), 'test-token-not-production')
      assert.equal(url.searchParams.get('ns'), null)
      assert.equal(url.searchParams.get('userNS'), 'default')
      assert.equal(new Headers(init?.headers).get('X-Reader-Secure-Key'), 'generated-manager-key')
      assert.equal(String(input).includes('generated-manager-key'), false)
      if (requests.length === 1) return Response.json(directory
        ? { isSuccess: true, data: [{ name: 'index.epub', isDirectory: false }] }
        : { isSuccess: false, errorMsg: '路径不是目录' }, { headers: { 'X-Reader-Namespace': 'default' } })
      assert.equal(url.searchParams.get('path'), `书 EPUB.epub${directory ? '/index.epub' : ''}`)
      assert.equal(url.searchParams.get('stream'), '1')
      return new Response(buildMinimalEpub() as BodyInit, { headers: { 'X-Reader-Namespace': 'default' } })
    })
    const doc = await loadEpubDoc('storage/data/default/书 EPUB.epub', {
      namespace: 'default', accessToken: 'test-token-not-production', systemNamespace: true, managerKey: 'generated-manager-key',
    })
    assert.equal(doc.spine.length, 2)
    assert.equal(requests.length, 2)
    assert.equal(requests[0].pathname, '/reader3/file/list')
    assert.equal(requests[1].pathname, '/reader3/file/download')
  })
}

test('EPUB loader never retries with privileged home or reads another user', async t => {
  let calls = 0
  t.mock.method(globalThis, 'fetch', async () => {
    calls++
    return Response.json({ isSuccess: false, errorMsg: '请登录后使用', data: 'NEED_LOGIN' })
  })
  await assert.rejects(loadEpubDoc('storage/data/bob/a.epub', { namespace: 'alice' }), /当前用户/)
  assert.equal(calls, 0)
  await assert.rejects(loadEpubDoc('storage/data/alice/a.epub', { namespace: 'alice' }), /请登录/)
  assert.equal(calls, 1)
})

test('系统 EPUB 不允许缺密钥或静默回退本人空间', async t => {
  let calls = 0
  t.mock.method(globalThis, 'fetch', async () => {
    calls++
    return Response.json({ isSuccess: true, data: [] }, { headers: { 'X-Reader-Namespace': 'generated-a' } })
  })
  const options = { namespace: 'default', systemNamespace: true }
  await assert.rejects(loadEpubDoc('storage/data/default/a.epub', options), /未确认系统配置空间/)
  assert.equal(calls, 0)
  await assert.rejects(loadEpubDoc('storage/data/default/a.epub', { ...options, managerKey: 'generated-key' }), /未确认系统配置空间/)
  assert.equal(calls, 1)
})

test('HTTP 200 business download failure is not parsed as ZIP', async t => {
  let calls = 0
  t.mock.method(globalThis, 'fetch', async () => Response.json(++calls === 1
    ? { isSuccess: false, errorMsg: '路径不是目录' }
    : { isSuccess: false, errorMsg: '路径不存在' }))
  await assert.rejects(loadEpubDoc('storage/data/alice/a.epub', { namespace: 'alice' }), /路径不存在/)
  assert.equal(calls, 2)
})

test('EPUB download deadline includes a hung path check and cancels the real request signal', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] })
  const external = new AbortController()
  let requestSignal: AbortSignal | undefined
  t.after(() => external.abort())
  t.mock.method(globalThis, 'fetch', (_input: unknown, init?: RequestInit) => {
    requestSignal = init?.signal ?? undefined
    return new Promise<Response>((_resolve, reject) => requestSignal?.addEventListener('abort',
      () => reject(new DOMException('Generated pending request aborted', 'AbortError')), { once: true }))
  })
  const pending = loadEpubDoc('storage/data/alice/generated.epub', { namespace: 'alice', signal: external.signal })
  void pending.catch(() => {})
  t.mock.timers.tick(60000)
  await new Promise<void>(resolve => setImmediate(resolve))
  assert.equal(requestSignal?.aborted, true, 'Hung listing must be aborted at the total download deadline')
  await assert.rejects(pending, /EPUB 下载超时/)
  assert.equal(external.signal.aborted, false, 'Loader must not mutate caller cancellation state')
})

for (const phase of ['headers', 'body', 'error-json'] as const) {
  test(`EPUB total download deadline aborts a stalled ${phase} after a successful listing`, async t => {
    t.mock.timers.enable({ apis: ['setTimeout'] })
    const external = new AbortController()
    t.after(() => external.abort())
    let calls = 0
    let requestSignal: AbortSignal | undefined
    let streamCancelled = false
    t.mock.method(globalThis, 'fetch', async (_input: unknown, init?: RequestInit) => {
      requestSignal = init?.signal ?? undefined
      if (++calls === 1) return Response.json({ isSuccess: false, errorMsg: '路径不是目录' })
      if (phase === 'headers') return new Promise<Response>((_resolve, reject) =>
        requestSignal?.addEventListener('abort', () => reject(new DOMException('Generated abort', 'AbortError')),
          { once: true }))
      return new Response(new ReadableStream<Uint8Array>({
        start(controller) {
          requestSignal?.addEventListener('abort', () => {
            streamCancelled = true
            controller.error(new DOMException('Generated body abort', 'AbortError'))
          }, { once: true })
          controller.enqueue(new Uint8Array(phase === 'body' ? [0x50, 0x4b] : [0x7b]))
        },
      }), { headers: phase === 'error-json' ? { 'content-type': 'application/json' } : {} })
    })
    const pending = loadEpubDoc('storage/data/alice/generated.epub', { namespace: 'alice', signal: external.signal })
    void pending.catch(() => {})
    await new Promise<void>(resolve => setImmediate(resolve))
    assert.equal(calls, 2)
    t.mock.timers.tick(59999)
    assert.equal(requestSignal?.aborted, false)
    t.mock.timers.tick(1)
    await assert.rejects(pending, /EPUB 下载超时（60 秒）/)
    assert.equal(requestSignal?.aborted, true)
    assert.equal(external.signal.aborted, false)
    if (phase !== 'headers') assert.equal(streamCancelled, true)
  })
}

test('EPUB caller cancellation stays cancellation rather than a timeout', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] })
  const external = new AbortController()
  let requestSignal: AbortSignal | undefined
  t.mock.method(globalThis, 'fetch', (_input: unknown, init?: RequestInit) => {
    requestSignal = init?.signal ?? undefined
    return new Promise<Response>((_resolve, reject) => requestSignal?.addEventListener('abort',
      () => reject(new DOMException('Generated user cancel', 'AbortError')), { once: true }))
  })
  const pending = loadEpubDoc('storage/data/alice/generated.epub', { namespace: 'alice', signal: external.signal })
  external.abort()
  await assert.rejects(pending, { name: 'AbortError' })
  t.mock.timers.tick(120000)
  assert.equal(requestSignal?.aborted, true)
})

test('Already cancelled EPUB load issues no listing or download request', async t => {
  const external = new AbortController()
  external.abort()
  let calls = 0
  t.mock.method(globalThis, 'fetch', async () => { calls++; throw new Error('Must not fetch') })
  await assert.rejects(loadEpubDoc('storage/data/alice/generated.epub',
    { namespace: 'alice', signal: external.signal }), { name: 'AbortError' })
  assert.equal(calls, 0)
})

test('TOC paths resolve by href, not spine index; fragments and encoded Chinese are normalized', () => {
  const doc = parseEpubBytes(buildMinimalEpub())
  assert.equal(epubChapterPath(doc, 'text/chapter2.xhtml#last'), 'OEBPS/text/chapter2.xhtml')
  assert.equal(epubChapterPath(doc, '%E6%B5%8B%E8%AF%95.xhtml'), 'OEBPS/测试.xhtml')
})

test('EPUB locations preserve decoded fragments separately from encoded hashes in file names', () => {
  const doc = parseEpubBytes(buildMinimalEpub())
  assert.deepEqual(epubChapterLocation(doc, 'text/chapter%231.xhtml#%E4%B8%AD%E6%96%87'),
    { path: 'OEBPS/text/chapter#1.xhtml', fragment: '中文' })
  assert.equal(epubFragment('chapter.xhtml#%broken'), '%broken')
  assert.equal(epubFragment('chapter.xhtml'), '')
})

test('fragment-only and file-qualified EPUB links resolve against the current document', () => {
  const doc = parseEpubBytes(buildMinimalEpub())
  const current = 'OEBPS/text/chapter1.xhtml'
  assert.deepEqual(resolveEpubLink(doc, current, '#%E4%B8%AD%E6%96%87'), { path: current, fragment: '中文' })
  assert.deepEqual(resolveEpubLink(doc, current, './chapter2.xhtml#end'),
    { path: 'OEBPS/text/chapter2.xhtml', fragment: 'end' })
  assert.deepEqual(resolveEpubLink(doc, current, '/OEBPS/text/chapter2.xhtml#end'),
    { path: 'OEBPS/text/chapter2.xhtml', fragment: 'end' })
})

test('EPUB navigation selects the exact TOC fragment, not the first chapter sharing the file', () => {
  const doc = parseEpubBytes(buildMinimalEpub())
  const urls = ['text/chapter1.xhtml#intro', 'text/chapter1.xhtml#%E4%B8%AD%E6%96%87', 'text/chapter2.xhtml']
  const path = 'OEBPS/text/chapter1.xhtml'
  assert.equal(epubNavigationIndex(doc, urls, { path, fragment: '中文' }, 0), 1)
  assert.equal(epubNavigationIndex(doc, urls, { path, fragment: 'note' }, 1), 1)
  assert.equal(epubNavigationIndex(doc, urls, { path, fragment: 'note' }, 2), 0)
  assert.equal(epubNavigationIndex(doc, urls, { path, fragment: '' }, 1), 0)
  assert.equal(epubNavigationIndex(doc, urls, { path: 'missing', fragment: 'intro' }, 0), -1)
})

test('EPUB links cannot escape to external protocols or nonexistent resources', () => {
  const doc = parseEpubBytes(buildMinimalEpub())
  for (const href of ['https://example.invalid/a.xhtml', '//example.invalid/a.xhtml', 'javascript:alert(1)',
    'data:text/html,hello', '%6Aavascript:alert(1)', 'missing.xhtml#intro', '']) {
    assert.equal(resolveEpubLink(doc, 'OEBPS/text/chapter1.xhtml', href), null)
  }
})
