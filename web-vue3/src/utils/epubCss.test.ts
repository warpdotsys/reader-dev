import { test } from 'node:test'
import assert from 'node:assert/strict'
import { strToU8 } from 'fflate'
import { destroyEpubDoc, type EpubDoc } from './epubLoader.ts'
import { epubStylesheetUrl, rewriteEpubCss, epubAssetUrl } from './epubCss.ts'

function fixture(files: Record<string, string>): EpubDoc {
  return { files: new Map(Object.entries(files).map(([name, value]) => [name, strToU8(value)])),
    manifest: new Map(), spine: [], opfDir: 'OEBPS', blobUrls: new Map() }
}
async function css(url: string) { return (await fetch(url)).text() }
function imports(text: string) { return [...text.matchAll(/@import\s+url\("([^"]+)"\)/g)].map(match => match[1]) }

test('EPUB CSS：多层引用以各自目录解析，保留 layer/supports/media，图片与字体字节不改动', async t => {
  const doc = fixture({
    'OEBPS/Styles/main.css': '@import /* generated */ "nested/mid.css" layer(book) supports(display: grid) screen;',
    'OEBPS/Styles/nested/mid.css': '@import url("../leaf/%E4%B8%AD%E6%96%87.css") screen;',
    'OEBPS/Styles/leaf/中文.css': 'p{background:url("../../Images/图.svg#mask");font:1em book} @font-face{font-family:book;src:url("../../Fonts/book.woff2")}',
    'OEBPS/Images/图.svg': '<svg>generated</svg>', 'OEBPS/Fonts/book.woff2': 'generated font URL bytes, not a shaping test',
  })
  t.after(() => destroyEpubDoc(doc))
  const root = epubStylesheetUrl(doc, 'OEBPS/Text/ch.xhtml', '../Styles/main.css')!
  const main = await css(root)
  assert.match(main, /layer\(book\) supports\(display: grid\) screen;/)
  const mid = await css(imports(main)[0])
  assert.match(mid, / screen;/)
  const leaf = await css(imports(mid)[0])
  assert.match(leaf, /blob:[^" ]+#mask/)
  assert.doesNotMatch(leaf, /\.\.\//)
  const font = [...leaf.matchAll(/url\("([^"]+)"\)/g)][1][1]
  assert.equal(await css(font), 'generated font URL bytes, not a shaping test')
  assert.equal(epubStylesheetUrl(doc, 'OEBPS/Text/ch.xhtml', '../Styles/main.css'), root)
})

test('EPUB CSS：循环只切递归边，分别从 A/B 进入不会丢掉另一张表规则', async t => {
  const doc = fixture({ 'a.css': '@import "b.css"; .a{color:red}', 'b.css': '@import "a.css"; .b{color:blue}' })
  t.after(() => destroyEpubDoc(doc))
  for (const root of ['a', 'b']) {
    const first = await css(epubStylesheetUrl(doc, 'ch.xhtml', root + '.css')!)
    const second = await css(imports(first)[0])
    assert.match(first, new RegExp(`\\.${root}\\{`))
    assert.match(second, new RegExp(`\\.${root === 'a' ? 'b' : 'a'}\\{`))
    assert.equal(await css(imports(second)[0]), '')
  }
  assert.equal(doc.blobUrls.size, 5)
})

test('EPUB CSS：普通字符串/注释不当引用，行内样式与 CSS 转义文件名可解析', t => {
  const doc = fixture({ 'OEBPS/图 片.svg': 'generated' })
  t.after(() => destroyEpubDoc(doc))
  const text = rewriteEpubCss(doc,
    '/* url("do-not-rewrite") */ p{content:"url(fake)";background:url("图\\20 片.svg");mask:url(#local)}', 'OEBPS/ch.xhtml')
  assert.match(text, /\/\* url\("do-not-rewrite"\) \*\//)
  assert.match(text, /content:"url\(fake\)"/)
  assert.match(text, /background:url\("blob:/)
  assert.match(text, /mask:url\("#local"\)/)
})

test('EPUB CSS：外网/编码协议与缺失依赖不能产生外网 URL；data:image 与本页 SVG 片段保留', t => {
  const doc = fixture({})
  t.after(() => destroyEpubDoc(doc))
  const text = rewriteEpubCss(doc,
    '@import "https://example.invalid/x.css" screen; p{background:url(//example.invalid/a);cursor:url(%6aavascript:evil)}', 'ch.xhtml')
  assert.doesNotMatch(text, /example\.invalid|javascript/i)
  assert.equal(imports(text)[0], '#')
  assert.equal(epubStylesheetUrl(doc, 'ch.xhtml', 'missing.css'), null)
  assert.equal(epubAssetUrl(doc, 'ch.xhtml', 'data:image/png;base64,AA=='), 'data:image/png;base64,AA==')
  assert.equal(epubAssetUrl(doc, 'ch.xhtml', '#mask'), '#mask')
})

test('EPUB CSS：深度、引用数、累计 blob 字节和资源 URL 数量超限均显式拒绝', t => {
  const deep = fixture(Object.fromEntries(Array.from({ length: 18 }, (_, index) =>
    [`${index}.css`, index === 17 ? '.generated{}' : `@import "${index + 1}.css";`])))
  t.after(() => destroyEpubDoc(deep))
  assert.throws(() => epubStylesheetUrl(deep, 'ch.xhtml', '0.css'), /嵌套深度/)
  const wide = fixture({ 'main.css': '@import "leaf.css";'.repeat(129), 'leaf.css': '.generated{}' })
  t.after(() => destroyEpubDoc(wide))
  assert.throws(() => epubStylesheetUrl(wide, 'ch.xhtml', 'main.css'), /引用数量/)
  const bytes = fixture({ 'main.css': '.generated{color:red}' })
  bytes.cssBytes = 32 * 1024 * 1024 - 1
  t.after(() => destroyEpubDoc(bytes))
  assert.throws(() => epubStylesheetUrl(bytes, 'ch.xhtml', 'main.css'), /资源总量/)
  const urls = fixture({ 'main.css': '.generated{}' })
  urls.blobUrls = new Map(Array.from({ length: 16384 }, (_, index) => [String(index), 'blob:generated-' + index]))
  t.after(() => destroyEpubDoc(urls))
  assert.throws(() => epubStylesheetUrl(urls, 'ch.xhtml', 'main.css'), /资源总量/)
})

test('EPUB CSS：回收所有 raw/重写/cycle URL 并重置预算，可重新创建', t => {
  const doc = fixture({ 'a.css': '@import "a.css"; .a{color:red}' })
  const revoked: string[] = []
  t.mock.method(URL, 'revokeObjectURL', (url: string) => revoked.push(url))
  epubStylesheetUrl(doc, 'ch.xhtml', 'a.css')
  const urls = [...doc.blobUrls.values()]
  destroyEpubDoc(doc)
  assert.deepEqual(revoked, urls)
  assert.equal(doc.cssBytes, 0)
  assert.equal(doc.blobUrls.size, 0)
  assert.ok(epubStylesheetUrl(doc, 'ch.xhtml', 'a.css'))
  destroyEpubDoc(doc)
})
