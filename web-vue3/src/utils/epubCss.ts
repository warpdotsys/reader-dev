/** 重写 CSS 递归依赖；保持 @import 后的 media/layer/supports 由浏览器解释。 */
import { type EpubDoc, epubResourceUrl, resolveEpubLink } from './epubLoader.ts'

const MAX_CSS_BYTES = 32 * 1024 * 1024
const MAX_CSS_REFERENCES = 128
const MAX_CSS_DEPTH = 16

function unescapeCss(value: string): string {
  return value.replace(/\\(?:([\da-f]{1,6})(?:\r\n|[\t\n\f\r ])?|([\s\S]))/gi, (_, hex, escaped) => {
    if (!hex) return /[\r\n\f]/.test(escaped) ? '' : escaped
    const code = Number.parseInt(hex, 16)
    return code > 0 && code <= 0x10ffff && !(code >= 0xd800 && code <= 0xdfff)
      ? String.fromCodePoint(code) : '\ufffd'
  })
}

export function epubAssetUrl(doc: EpubDoc, currentPath: string, raw: string): string {
  const value = raw.trim()
  if (/^data:image\//i.test(value) || value.startsWith('#')) return value
  const target = resolveEpubLink(doc, currentPath, value)
  if (!target) return '#'
  const url = epubResourceUrl(doc, target.path)
  return url ? url + (target.fragment ? `#${encodeURIComponent(target.fragment)}` : '') : '#'
}

interface CssTraversal { references: number; root: string }

function createCssBlob(doc: EpubDoc, key: string, css: string): string {
  const data = new TextEncoder().encode(css)
  if ((doc.cssBytes ?? 0) + data.length > MAX_CSS_BYTES || doc.blobUrls.size >= 16384) {
    throw new Error('EPUB 原版样式资源总量超过上限')
  }
  const url = URL.createObjectURL(new Blob([data], { type: 'text/css;charset=utf-8' }))
  doc.cssBytes = (doc.cssBytes ?? 0) + data.length
  doc.blobUrls.set(key, url)
  return url
}

function stylesheet(doc: EpubDoc, path: string, stack: string[], traversal: CssTraversal): string {
  if (stack.includes(path)) {
    // CSS 循环只忽略递归边，保留每张表自己的规则，不把整张表替换为空实现。
    const key = 'reader-css:cycle'
    return doc.blobUrls.get(key) ?? createCssBlob(doc, key, '')
  }
  if (stack.length >= MAX_CSS_DEPTH || ++traversal.references > MAX_CSS_REFERENCES) {
    throw new Error('EPUB 原版样式嵌套深度或引用数量超过上限')
  }
  // 循环图的结果依赖入口，不能把 A 入口的 B 缓存用于 B 入口而丢掉 A 的规则。
  const key = `reader-css:${traversal.root}:${path}`
  const cached = doc.blobUrls.get(key)
  if (cached) return cached
  const data = doc.files.get(path)
  if (!data) return '#'
  if (data.length > 4 * 1024 * 1024) throw new Error('EPUB 单个样式资源超过原版排版上限')
  const css = rewrite(doc, new TextDecoder().decode(data), path, [...stack, path], traversal)
  return createCssBlob(doc, key, css)
}

// 注释和普通字符串整体跳过；@import URL 前缀优先匹配，条件后缀完全保留。
const CSS_REFS = /\/\*[\s\S]*?\*\/|"(?:\\[\s\S]|[^"\\])*"|'(?:\\[\s\S]|[^'\\])*'|@import(?:\s|\/\*[\s\S]*?\*\/)+(?:url\(\s*(?:"((?:\\[\s\S]|[^"\\])*)"|'((?:\\[\s\S]|[^'\\])*)'|([^)]*?))\s*\)|"((?:\\[\s\S]|[^"\\])*)"|'((?:\\[\s\S]|[^'\\])*)')|url\(\s*(?:"((?:\\[\s\S]|[^"\\])*)"|'((?:\\[\s\S]|[^'\\])*)'|([^)]*?))\s*\)/gi

function rewrite(doc: EpubDoc, css: string, currentPath: string, stack: string[], traversal: CssTraversal): string {
  return css.replace(CSS_REFS, (token, ...groups: unknown[]) => {
    if (token.startsWith('/*') || /^["']/.test(token)) return token
    const imported = /^@import/i.test(token)
    const captures = groups.slice(imported ? 0 : 5, imported ? 5 : 8)
    const raw = unescapeCss(String(captures.find(value => typeof value === 'string') ?? '').trim())
    let url: string
    if (imported) {
      const target = resolveEpubLink(doc, currentPath, raw)
      url = target ? stylesheet(doc, target.path, stack, traversal) : '#'
    } else url = epubAssetUrl(doc, currentPath, raw)
    return `${imported ? '@import ' : ''}url(${JSON.stringify(url)})`
  })
}

export function epubStylesheetUrl(doc: EpubDoc, currentPath: string, href: string): string | null {
  const target = resolveEpubLink(doc, currentPath, href)
  return target ? stylesheet(doc, target.path, [], { references: 0, root: target.path }) : null
}

export function rewriteEpubCss(doc: EpubDoc, css: string, currentPath: string): string {
  return rewrite(doc, css, currentPath, [], { references: 0, root: currentPath })
}
