import { test } from 'node:test'
import assert from 'node:assert/strict'
import { decodeEntities, isDangerousUrl, sanitizeHtml } from './sanitize.ts'

/* ================= P1-4：RSS 净化器加固（无外部依赖） ================= */

test('P1-4：危险协议判定——javascript:/data:/vbscript: 前缀', () => {
  assert.equal(isDangerousUrl('javascript:alert(1)'), true)
  assert.equal(isDangerousUrl('JaVaScRiPt:alert(1)'), true)
  assert.equal(isDangerousUrl('data:text/html;base64,PHNjcmlwdD4='), true)
  assert.equal(isDangerousUrl('vbscript:msgbox(1)'), true)
  assert.equal(isDangerousUrl('https://example.com/a'), false)
  assert.equal(isDangerousUrl('http://example.com'), false)
  assert.equal(isDangerousUrl('/relative/path'), false)
  assert.equal(isDangerousUrl(''), false)
})

test('P1-4：协议内空白/控制字符剥离（java\\tscript: 等价 javascript:）', () => {
  assert.equal(isDangerousUrl('java\tscript:alert(1)'), true)
  assert.equal(isDangerousUrl('java\nscript:alert(1)'), true)
  assert.equal(isDangerousUrl('jav\u0000ascript:alert(1)'), true)
  assert.equal(isDangerousUrl('java script:alert(1)'), true)
})

test('P1-4：实体解码——数字实体/命名实体/双重编码', () => {
  assert.equal(decodeEntities('&#106;avascript:'), 'javascript:')
  assert.equal(decodeEntities('&#x6A;avascript:'), 'javascript:')
  assert.equal(decodeEntities('&colon;&sol;&sol;x'), '://x')
  assert.equal(decodeEntities('&amp;#106;avascript:'), 'javascript:', '双重编码应解码到 javascript:')
  assert.equal(decodeEntities('&amp;amp;#106;'), 'j', '三重编码 3 轮内同样解到可执行协议（浏览器属性值递归解码同语义）')
  assert.equal(decodeEntities('a&unknown;b'), 'a&unknown;b', '未知命名实体原样保留')
})

test('P1-4：href/src 危险协议属性被移除（含实体编码变体）', () => {
  assert.equal(
    sanitizeHtml('<a href="javascript:alert(1)">x</a>'),
    '<a>x</a>',
  )
  assert.equal(
    sanitizeHtml('<a HREF=\'javascript:alert(1)\'>x</a>'),
    '<a>x</a>',
  )
  assert.equal(
    sanitizeHtml('<a href=javascript:alert(1)>x</a>'),
    '<a>x</a>',
    '无引号值同样移除',
  )
  assert.equal(
    sanitizeHtml('<img src="data:text/html;base64,PHNjcmlwdD4=">'),
    '<img>',
  )
  assert.equal(
    sanitizeHtml('<a href="&#106;avascript:alert(1)">x</a>'),
    '<a>x</a>',
    '数字实体编码变体',
  )
  assert.equal(
    sanitizeHtml('<a href="&amp;#106;avascript:alert(1)">x</a>'),
    '<a>x</a>',
    '双重编码变体',
  )
  assert.equal(
    sanitizeHtml('<a href="java&#x73;cript:alert(1)">x</a>'),
    '<a>x</a>',
    '十六进制实体变体',
  )
  assert.equal(
    sanitizeHtml('<a href="javascript&colon;alert(1)">x</a>'),
    '<a>x</a>',
    '命名实体变体',
  )
})

test('P1-4：xlink:href 处理（SVG 注入面）', () => {
  assert.equal(
    sanitizeHtml('<svg><a xlink:href="javascript:alert(1)"><text>x</text></a></svg>'),
    '<svg><a><text>x</text></a></svg>',
  )
  assert.equal(
    sanitizeHtml('<use xlink:href="data:image/svg+xml;base64,AAAA"/>'),
    '<use/>',
  )
})

test('P1-4：安全链接与正常属性不受影响', () => {
  const html =
    '<a href="https://example.com/a?b=1&amp;c=2" title="t">link</a><img src="/img/a.png" alt="图">'
  assert.equal(sanitizeHtml(html), html)
})

test('P1-4：script/style/iframe 与事件属性仍被移除', () => {
  const out = sanitizeHtml(
    '<script>alert(1)</script><p onclick="x()" style="color:red" onmouseover="y()">t</p><iframe src="https://e.com"></iframe>',
  )
  assert.ok(!out.includes('<script'))
  assert.ok(!out.includes('onclick'))
  assert.ok(!out.includes('onmouseover'))
  assert.ok(!out.includes('<iframe'))
  assert.ok(out.includes('<p style="color:red">t</p>'), 'style 属性保留（仅去 style 块/事件属性），段落文本完整')
})
