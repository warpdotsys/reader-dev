/**
 * P1-4：RSS 正文 HTML 净化（纯函数，无外部依赖——不引入 DOMPurify CDN）。
 *
 * 在原有轻量清洗（去 script/style/嵌入标签/事件属性）基础上加固：
 * - 实体解码（数字实体 + 常见命名实体，最多 3 轮——覆盖 `&amp;#106;` 双重编码绕过）后校验；
 * - 移除 javascript:/data:/vbscript: 协议的 href/src/xlink:href 属性（含 `java\tscript:`、
 *   `&#106;avascript:` 等变体——协议内空白/控制字符剥离后前缀匹配）；
 * - xlink:href 与 href/src 同规则处理（SVG `<a xlink:href="javascript:...">` 是常见注入面）。
 */

/** 常见命名实体（仅 URL 协议判定相关子集；未知命名实体原样保留） */
const NAMED_ENTITIES: Record<string, string> = {
  amp: '&',
  lt: '<',
  gt: '>',
  quot: '"',
  apos: "'",
  nbsp: ' ',
  colon: ':',
  sol: '/',
  Tab: '\t',
  NewLine: '\n',
}

/** 单轮实体解码（数字实体 + 常见命名实体） */
function decodeEntitiesOnce(s: string): string {
  return s
    .replace(/&#x([0-9a-f]+);/gi, (_, h: string) => {
      const cp = parseInt(h, 16)
      return cp >= 0 && cp <= 0x10ffff ? String.fromCodePoint(cp) : ''
    })
    .replace(/&#(\d+);/g, (_, d: string) => {
      const cp = parseInt(d, 10)
      return cp >= 0 && cp <= 0x10ffff ? String.fromCodePoint(cp) : ''
    })
    .replace(/&([a-zA-Z][a-zA-Z0-9]*);/g, (m, name: string) => NAMED_ENTITIES[name] ?? m)
}

/**
 * 完整实体解码（最多 3 轮——覆盖 `&amp;#106;` 双重编码；无变化提前结束）。
 * 返回的是解码后字符串，仅用于协议判定；不改变原始 HTML 内容本身。
 */
export function decodeEntities(s: string): string {
  let cur = s
  for (let i = 0; i < 3; i++) {
    const next = decodeEntitiesOnce(cur)
    if (next === cur) return cur
    cur = next
  }
  return cur
}

/**
 * 危险 URL 协议判定：剥离空白/控制字符（浏览器 URL 解析对 scheme 内 tab/换行/C0 控制
 * 字符等同忽略，`java\tscript:` 与 `javascript:` 等价）后前缀匹配。
 */
export function isDangerousUrl(value: string): boolean {
  const stripped = value.replace(/[\u0000-\u0020\u007f]/g, '').toLowerCase()
  return (
    stripped.startsWith('javascript:') ||
    stripped.startsWith('data:') ||
    stripped.startsWith('vbscript:')
  )
}

/**
 * 轻量 HTML 净化：RSS 正文按 HTML 渲染前的安全清洗。
 * 1) 删除 script/style/iframe/object/embed/form 整块；
 * 2) 删除全部事件属性（on*）；
 * 3) href/src/xlink:href 值实体解码后做危险协议校验，命中则整个属性删除。
 */
export function sanitizeHtml(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/<(iframe|object|embed|form)[\s\S]*?<\/(?:iframe|object|embed|form)>/gi, '')
    .replace(/<(iframe|object|embed|form)\b[^>]*\/?>/gi, '')
    .replace(/\son\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, '')
    .replace(
      /\s(?:href|src|xlink:href)\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi,
      (m, raw: string) => {
        const unquoted = raw.replace(/^["']|["']$/g, '')
        return isDangerousUrl(decodeEntities(unquoted)) ? '' : m
      },
    )
}
