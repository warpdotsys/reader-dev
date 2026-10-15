import { resolveEpubLink, type EpubDoc, type EpubLocation } from './epubLoader'

export interface EpubTocEntry extends EpubLocation { title: string }

/** 只解析书内导航，不执行 XHTML、不联网；独立于 legacy 的资源级章节索引。 */
export function readEpubNavigation(doc: EpubDoc): EpubTocEntry[] {
  const entries = [...doc.manifest.values()]
  const navigation = entries.find(item => item.properties?.split(/\s+/).includes('nav'))
    ?? entries.find(item => item.mediaType === 'application/x-dtbncx+xml')
  if (!navigation) return []
  const bytes = doc.files.get(navigation.href)
  if (!bytes) throw new Error('EPUB 书内目录资源缺失')
  const xml = new DOMParser().parseFromString(new TextDecoder('utf-8').decode(bytes), 'application/xml')
  if (xml.getElementsByTagName('parsererror').length) throw new Error('EPUB 书内目录 XML 无法解析')
  const result: EpubTocEntry[] = []
  const seen = new Set<string>()
  const add = (href: string, title: string) => {
    const target = resolveEpubLink(doc, navigation.href, href)
    if (!target) return
    const key = JSON.stringify([target.path, target.fragment])
    if (seen.has(key)) return
    seen.add(key)
    result.push({ ...target, title: title.trim() || target.fragment || '未命名书内目录' })
  }
  if (navigation.properties?.split(/\s+/).includes('nav')) {
    const toc = [...xml.getElementsByTagNameNS('*', 'nav')].find(node =>
      (node.getAttributeNS('http://www.idpf.org/2007/ops', 'type') ?? node.getAttribute('epub:type') ?? '')
        .split(/\s+/).includes('toc'))
    if (!toc) return []
    for (const anchor of toc.getElementsByTagNameNS('*', 'a')) add(anchor.getAttribute('href') ?? '', anchor.textContent ?? '')
  } else {
    for (const point of xml.getElementsByTagNameNS('*', 'navPoint')) {
      const children = [...point.children]
      const content = children.find(node => node.localName === 'content')
      const label = children.find(node => node.localName === 'navLabel')
      if (content) add(content.getAttribute('src') ?? '', label?.textContent ?? '')
    }
  }
  return result
}
