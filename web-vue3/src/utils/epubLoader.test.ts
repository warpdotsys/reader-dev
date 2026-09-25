/**
 * C2: epubLoader unit tests (in-memory minimal EPUB via fflate.zipSync).
 */
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { zipSync, strToU8 } from 'fflate'
import {
  parseEpubBytes,
  resolveHref,
  destroyEpubDoc,
  type EpubDoc,
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
      `<?xml version="1.0"?><rootfiles><rootfile full-path="book.opf"/></rootfiles>`,
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
