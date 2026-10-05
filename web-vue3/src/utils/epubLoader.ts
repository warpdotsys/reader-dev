/**
 * P0-1 EPUB 原版渲染：.epub 文件加载与结构化解析。
 *
 * 数据流：file/download stream=1 限流拉取 .epub → 有预算/超时的专用 Worker 解压
 * → META-INF/container.xml 定位 OPF → manifest/spine
 * 建立章节顺序 → 产出 EpubDoc（资源表 + spine 条目）供 EpubIframe 渲染。
 *
 * 资源引用策略：XHTML/CSS/图片全部以 blob URL 形式注入 iframe srcdoc，
 * 相对路径在生成 srcdoc 时重写为 blob URL；沙箱禁 script，无执行风险。
 */
import { strFromU8 } from 'fflate'
import { expandEpubArchive, readEpubResponse } from './epubArchive.ts'
import { expandEpubInWorker } from './epubWorker.ts'
import { assertLegacyNamespace, NamespaceProofError } from './legacyNamespace.ts'
import { tokenAuthenticationParams } from './tokenAuthentication.ts'

/** OPF manifest 单项 */
export interface EpubManifestItem {
  id: string
  href: string
  mediaType: string
  properties?: string
}

/** spine 单项（阅读顺序） */
export interface EpubSpineItem {
  idref: string
  linear: boolean
}

/** 解析后的 EPUB 文档 */
export interface EpubDoc {
  /** 全部资源：zip 内路径 → 字节 */
  files: Map<string, Uint8Array>
  /** manifest id → 条目 */
  manifest: Map<string, EpubManifestItem>
  /** 阅读顺序（仅 linear=yes 与无标记项） */
  spine: EpubSpineItem[]
  /** OPF 所在目录（相对路径基准） */
  opfDir: string
  /** 已创建的 blob URL（销毁时统一 revoke） */
  blobUrls: Map<string, string>
  /** 重写后的 CSS blob 总量，随文档回收；不是浏览器进程 RSS 上限。 */
  cssBytes?: number
}

/** container.xml → OPF 路径 */
function findOpfPath(files: Map<string, Uint8Array>): string {
  const container = files.get('META-INF/container.xml')
  if (!container) throw new Error('EPUB 缺少 container.xml')
  const xml = strFromU8(container)
  const m = /full-path\s*=\s*(["'])(.*?)\1/.exec(xml)
  if (!m) throw new Error('container.xml 无 rootfile')
  return normalizeZipName(m[2])
}

/** 规范化 zip 路径：解 %XX、反斜杠、去 ./ */
function normalizeZipName(name: string): string {
  let n = name.replace(/\\/g, '/')
  try {
    n = decodeURIComponent(n)
  } catch {
    /* 已是明文 */
  }
  while (n.startsWith('./')) n = n.slice(2)
  return n
}

/** 相对路径解析：base 目录 + href → zip 内路径（含 ../ 处理） */
export function resolveHref(opfDir: string, href: string): string {
  const parts = (opfDir ? opfDir.split('/') : []).concat(href.split('/'))
  const out: string[] = []
  for (const p of parts) {
    if (!p || p === '.') continue
    if (p === '..') out.pop()
    else out.push(p)
  }
  return out.join('/')
}

/**
 * 加载并解析 EPUB：
 * - bookUrl 为本地书路径 → file/download?path=...&stream=1 直出字节
 * - 返回 EpubDoc；调用方持有并在切换/卸载时调 destroyEpubDoc 回收 blob URL
 */
export interface EpubLoadOptions {
  namespace: string
  accessToken?: string
  systemNamespace?: boolean
  managerKey?: string
  signal?: AbortSignal
}

/** 只映射现有文件 home，不使用管理级 __STORAGE__ 或跨用户回退。 */
export function epubFileLocation(source: string, namespace: string): { home: string; path: string } {
  const path = source.replace(/\\/g, '/')
  if (!namespace || /[\\/]/.test(namespace) || namespace === '.' || namespace === '..'
    || path.split('/').some(p => !p || p === '.' || p === '..' || p.includes(':'))) {
    throw new Error('EPUB 文件路径无效')
  }
  const userPrefix = `storage/data/${namespace}/`
  const roots = [
    [`${userPrefix}webdav/`, '__WEBDAV__'],
    [userPrefix, '__HOME__'],
    ['storage/localStore/', '__LOCAL_STORE__'],
  ]
  for (const [prefix, home] of roots) {
    if (path.startsWith(prefix) && path.length > prefix.length) {
      return { home, path: path.slice(prefix.length) }
    }
  }
  throw new Error('EPUB 不在当前用户可下载目录内；请将该书导入当前用户')
}

export async function loadEpubDoc(source: string, options: EpubLoadOptions): Promise<EpubDoc> {
  const location = epubFileLocation(source, options.namespace)
  const params = new URLSearchParams(location)
  for (const [name, value] of Object.entries(tokenAuthenticationParams(options.accessToken))) params.set(name, value)
  if (options.systemNamespace) {
    if (!options.managerKey) throw new NamespaceProofError()
    params.set('userNS', 'default')
  }
  if (options.signal?.aborted) throw new DOMException('EPUB 加载已取消', 'AbortError')
  const controller = new AbortController()
  const cancel = () => controller.abort(options.signal?.reason)
  options.signal?.addEventListener('abort', cancel, { once: true })
  let timedOut = false
  // One deadline covers directory checking, headers, error JSON and the entire streamed body.
  // Clear it before starting the separately bounded 15-second decompression Worker.
  const timer = setTimeout(() => { timedOut = true; controller.abort() }, 60000)
  let buf: Uint8Array
  try {
    buf = await downloadEpubBytes(params, location.path, controller.signal, options)
  } catch (error) {
    controller.abort()
    if (timedOut && !options.signal?.aborted) {
      throw new Error('EPUB 下载超时（60 秒），已终止原版排版加载')
    }
    throw error
  } finally {
    clearTimeout(timer)
    options.signal?.removeEventListener('abort', cancel)
  }
  return parseEpubFiles(await expandEpubInWorker(buf, options.signal))
}

async function downloadEpubBytes(params: URLSearchParams, path: string, signal: AbortSignal, options: EpubLoadOptions): Promise<Uint8Array> {
  const init: RequestInit = { credentials: 'same-origin', cache: 'no-store', signal,
    headers: options.systemNamespace ? { 'X-Reader-Secure-Key': options.managerKey! } : undefined }
  // 只查询选中书自身：legacy 导入布局是 xxx.epub/index.epub，书仓则可为单文件。
  const listing = await fetch(`/reader3/file/list?${params}`, init)
  assertLegacyNamespace({ systemNamespace: options.systemNamespace === true }, '/file/list', listing.headers.get('X-Reader-Namespace'))
  if (!listing.ok) throw new Error(`EPUB 路径检查失败（${listing.status}）`)
  const info = await listing.json() as {
    isSuccess: boolean; errorMsg?: string; data?: { name: string; isDirectory: boolean }[]
  }
  if (info.isSuccess) {
    if (!Array.isArray(info.data) || !info.data.some(f => f.name === 'index.epub' && !f.isDirectory)) {
      throw new Error('EPUB 目录缺少 index.epub')
    }
    params.set('path', `${path}/index.epub`)
  } else if (info.errorMsg !== '路径不是目录') {
    throw new Error(info.errorMsg || 'EPUB 路径检查失败')
  }
  params.set('stream', '1')
  const res = await fetch(`/reader3/file/download?${params}`, init)
  assertLegacyNamespace({ systemNamespace: options.systemNamespace === true }, '/file/download', res.headers.get('X-Reader-Namespace'))
  if (!res.ok) throw new Error(`EPUB 文件获取失败（${res.status}）`)
  // legacy 的业务错误也可能 HTTP 200，不能交给 unzip 后误报 invalid zip data。
  if (res.headers.get('content-type')?.includes('json')) {
    const error = await res.json() as { errorMsg?: string }
    throw new Error(error.errorMsg || 'EPUB 文件获取失败')
  }
  const buf = await readEpubResponse(res)
  if (buf[0] !== 0x50 || buf[1] !== 0x4b) throw new Error('返回内容不是 EPUB ZIP 文件')
  return buf
}

/** legacy TOC 是 OPF 相对 href，不保证其下标等于 spine（卷名/封面/非线性页）。 */
export function epubChapterPath(doc: EpubDoc, chapterUrl: string): string {
  return resolveHref(doc.opfDir, normalizeZipName(chapterUrl.split('#')[0]))
}

export interface EpubLocation { path: string; fragment: string }

/** 在分割后仅解码一次；编码的 # 可属于文件名，不能当作锚点分隔符。 */
export function epubFragment(href: string): string {
  const hash = href.indexOf('#')
  if (hash < 0) return ''
  const fragment = href.slice(hash + 1)
  try { return decodeURIComponent(fragment) } catch { return fragment }
}

export function epubChapterLocation(doc: EpubDoc, chapterUrl: string): EpubLocation {
  return { path: epubChapterPath(doc, chapterUrl), fragment: epubFragment(chapterUrl) }
}

/** 书内链接以当前 XHTML 为基准；不允许外连协议或不存在的 ZIP 资源。 */
export function resolveEpubLink(doc: EpubDoc, currentPath: string, href: string): EpubLocation | null {
  const raw = href.trim()
  if (!raw || /^[a-z][a-z0-9+.-]*:|^\/\//i.test(raw)) return null
  const pathPart = normalizeZipName(raw.split('#')[0]!.split('?')[0]!)
  if (/^[a-z][a-z0-9+.-]*:|^\/\//i.test(pathPart)) return null
  const dir = currentPath.includes('/') ? currentPath.slice(0, currentPath.lastIndexOf('/')) : ''
  const path = !pathPart ? currentPath : pathPart.startsWith('/')
    ? resolveHref('', pathPart.slice(1)) : resolveHref(dir, pathPart)
  return doc.files.has(path) ? { path, fragment: epubFragment(raw) } : null
}

/** 优先精确文件+锚点；未列入 TOC 的同页锚点保留当前章节，而非跳回第一项。 */
export function epubNavigationIndex(doc: EpubDoc, chapterUrls: readonly string[], target: EpubLocation,
  preferredIndex = -1): number {
  const locations = chapterUrls.map(url => epubChapterLocation(doc, url))
  const exact = locations.findIndex(location => location.path === target.path && location.fragment === target.fragment)
  if (exact >= 0) return exact
  if (target.fragment && locations[preferredIndex]?.path === target.path) return preferredIndex
  return locations.findIndex(location => location.path === target.path)
}

/** 从字节解析 EPUB（测试可直接喂内存数据） */
export function parseEpubBytes(bytes: Uint8Array): EpubDoc {
  return parseEpubFiles(expandEpubArchive(bytes))
}

function parseEpubFiles(files: Map<string, Uint8Array>): EpubDoc {

  const opfPath = findOpfPath(files)
  const opfDir = opfPath.includes('/') ? opfPath.slice(0, opfPath.lastIndexOf('/')) : ''
  const opfXml = strFromU8(files.get(opfPath) ?? new Uint8Array())

  // manifest 解析：属性顺序无关，逐 <item> 提取
  const manifest = new Map<string, EpubManifestItem>()
  for (const m of opfXml.matchAll(/<item\b[^>]*>/g)) {
    const tag = m[0]
    const attr = (n: string): string => {
      const r = new RegExp(`${n}\\s*=\\s*"([^"]*)"`).exec(tag)?.[1]
      ?? new RegExp(`${n}\\s*=\\s*'([^']*)'`).exec(tag)?.[1]
      ?? ''
      try {
        return decodeURIComponent(r)
      } catch {
        return r
      }
    }
    const id = attr('id')
    if (!id) continue
    manifest.set(id, {
      id,
      href: resolveHref(opfDir, attr('href')),
      mediaType: attr('media-type'),
      properties: attr('properties') || undefined,
    })
  }

  // spine 解析
  const spine: EpubSpineItem[] = []
  const spineBlock = /<spine\b[^>]*>([\s\S]*?)<\/spine>/i.exec(opfXml)?.[1] ?? ''
  for (const m of spineBlock.matchAll(/<itemref\b[^>]*>/g)) {
    const tag = m[0]
    const idref = /\bidref\s*=\s*"([^"]*)"/.exec(tag)?.[1]
      ?? /\bidref\s*=\s*'([^']*)'/.exec(tag)?.[1]
    if (!idref) continue
    const linear = !/\blinear\s*=\s*["']no["']/i.test(tag)
    spine.push({ idref, linear })
  }

  return { files, manifest, spine, opfDir, blobUrls: new Map() }
}

/** 取资源为 blob URL（缓存复用） */
export function epubResourceUrl(doc: EpubDoc, zipPath: string): string | null {
  const cached = doc.blobUrls.get(zipPath)
  if (cached) return cached
  const data = doc.files.get(zipPath)
  if (!data) return null
  if (doc.blobUrls.size >= 16384) throw new Error('EPUB 原版排版资源 URL 数量超过上限')
  const item = [...doc.manifest.values()].find((i) => i.href === zipPath)
  const type = mimeOf(zipPath, item?.mediaType)
  const url = URL.createObjectURL(new Blob([data as BlobPart], { type }))
  doc.blobUrls.set(zipPath, url)
  return url
}

function mimeOf(path: string, declared?: string): string {
  if (declared && !declared.includes('/')) return `application/${declared}`
  if (declared) return declared
  if (path.endsWith('.xhtml') || path.endsWith('.html')) return 'application/xhtml+xml'
  if (path.endsWith('.css')) return 'text/css'
  if (path.endsWith('.png')) return 'image/png'
  if (path.endsWith('.jpg') || path.endsWith('.jpeg')) return 'image/jpeg'
  if (path.endsWith('.gif')) return 'image/gif'
  if (path.endsWith('.svg')) return 'image/svg+xml'
  if (path.endsWith('.ttf')) return 'font/ttf'
  if (path.endsWith('.otf')) return 'font/otf'
  if (path.endsWith('.woff')) return 'font/woff'
  if (path.endsWith('.woff2')) return 'font/woff2'
  return 'application/octet-stream'
}

/** 销毁：revoke 全部 blob URL（切书/卸载防泄漏） */
export function destroyEpubDoc(doc: EpubDoc): void {
  for (const url of doc.blobUrls.values()) URL.revokeObjectURL(url)
  doc.blobUrls.clear()
  doc.cssBytes = 0
}
