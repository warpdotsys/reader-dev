/**
 * P0-1 EPUB 原版渲染：.epub 文件加载与结构化解析。
 *
 * 数据流：file/download stream=1 拉取 .epub 字节 → fflate 解压（同步 unzip，
 * 书籍级一次性成本）→ META-INF/container.xml 定位 OPF → manifest/spine
 * 建立章节顺序 → 产出 EpubDoc（资源表 + spine 条目）供 EpubIframe 渲染。
 *
 * 资源引用策略：XHTML/CSS/图片全部以 blob URL 形式注入 iframe srcdoc，
 * 相对路径在生成 srcdoc 时重写为 blob URL；沙箱禁 script，无执行风险。
 */
import { unzipSync, strFromU8 } from 'fflate'

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
}

/** container.xml → OPF 路径 */
function findOpfPath(files: Map<string, Uint8Array>): string {
  const container = files.get('META-INF/container.xml')
  if (!container) throw new Error('EPUB 缺少 container.xml')
  const xml = strFromU8(container)
  const m = /full-path\s*=\s*"([^"]+)"/.exec(xml)
  if (!m) throw new Error('container.xml 无 rootfile')
  return m[1]
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
export async function loadEpubDoc(bookUrl: string): Promise<EpubDoc> {
  // file/download 需要 path 参数 + accessToken（request 层自动附带 token）
  const res = await fetch(
    `/reader3/file/download?path=${encodeURIComponent(bookUrl)}&stream=1`,
    { credentials: 'same-origin' },
  )
  if (!res.ok) throw new Error(`EPUB 文件获取失败（${res.status}）`)
  const buf = new Uint8Array(await res.arrayBuffer())
  return parseEpubBytes(buf)
}

/** 从字节解析 EPUB（测试可直接喂内存数据） */
export function parseEpubBytes(bytes: Uint8Array): EpubDoc {
  const raw = unzipSync(bytes)
  const files = new Map<string, Uint8Array>()
  for (const [name, data] of Object.entries(raw)) {
    if (name.endsWith('/')) continue
    files.set(normalizeZipName(name), data)
  }

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
}
