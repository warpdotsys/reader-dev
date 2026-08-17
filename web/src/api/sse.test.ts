import { test } from 'node:test'
import assert from 'node:assert/strict'
import { parseSSEBlock, dispatchSSEBlock, consumeSSEStream, consumeSSEStreamBlocks } from './sse.ts'
import type { SSEStreamCallbacks } from './sse.ts'

test('GAP 81：parseSSEBlock 解析 event:/data: 块（兼容 CRLF 与 data 前导空格）', () => {
  const evt = parseSSEBlock('event: book\ndata: {"lastIndex":0,"data":[]}\n\n')
  assert.equal(evt?.event, 'book')
  assert.equal(evt?.data, '{"lastIndex":0,"data":[]}')
  // CRLF 兼容
  const cr = parseSSEBlock('event: end\r\ndata: {"lastIndex":1,"isEnd":true}\r\n\r\n')
  assert.equal(cr?.event, 'end')
  assert.equal(cr?.data, '{"lastIndex":1,"isEnd":true}')
  // 无 data 行 → null
  assert.equal(parseSSEBlock('event: book\n\n'), null)
  // 多行 data 以换行拼接（SSE 语义）
  const multi = parseSSEBlock('event: end\ndata: {"a":1}\ndata: ,"b":2}\n\n')
  assert.equal(multi?.data, '{"a":1}\n,"b":2}')
})

test('GAP 81：dispatchSSEBlock 分发 book/end/error 事件', () => {
  const calls: string[] = []
  const cbs: SSEStreamCallbacks = {
    onBooks: (_i, books) => calls.push(`book:${books.length}`),
    onEnd: (i, isEnd) => calls.push(`end:${i}:${isEnd}`),
    onErrorEvent: (ret) => calls.push(`err:${ret.errorMsg}`),
  }
  dispatchSSEBlock('event: book\ndata: {"lastIndex":3,"data":[{"bookUrl":"u","origin":"o"}]}\n\n', cbs)
  dispatchSSEBlock('event: end\ndata: {"lastIndex":3,"isEnd":true}\n\n', cbs)
  dispatchSSEBlock('event: error\ndata: {"isSuccess":false,"errorMsg":"书源不存在","data":null}\n\n', cbs)
  assert.deepEqual(calls, ['book:1', 'end:3:true', 'err:书源不存在'])
})

test('GAP 81：dispatchSSEBlock 容错——坏 JSON 忽略、end 坏 JSON 兜底 onEnd(-1,false)、error 无 JSON 结构兜底文案', () => {
  const calls: string[] = []
  const cbs: SSEStreamCallbacks = {
    onBooks: () => calls.push('book'),
    onEnd: (i, isEnd) => calls.push(`end:${i}:${isEnd}`),
    onErrorEvent: (ret) => calls.push(`err:${String(ret.errorMsg)}`),
  }
  dispatchSSEBlock('event: book\ndata: not-json\n\n', cbs)
  dispatchSSEBlock('event: end\ndata: garbage\n\n', cbs)
  dispatchSSEBlock('event: error\ndata: 服务异常\n\n', cbs)
  assert.deepEqual(calls, ['end:-1:false', 'err:服务异常'])
})

test('GAP 81：consumeSSEStream 按空行切块并跨 chunk 拼接', async () => {
  const blocks: string[] = []
  const cbs: SSEStreamCallbacks = {
    onBooks: (_i, books) => blocks.push(`book:${books.length}`),
    onEnd: () => blocks.push('end'),
    onErrorEvent: () => blocks.push('err'),
  }
  // 两个 chunk 把「book 块 + end 块」切碎传输（模拟 TCP 分包）
  const raw =
    'event: book\ndata: {"lastIndex":0,"data":[{"bookUrl":"u","origin":"o"}]}\n\nevent: end\ndata: {"lastIndex":0,"isEnd":true}\n\n'
  const cut = Math.floor(raw.length / 2)
  const chunks = [raw.slice(0, cut), raw.slice(cut)]
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      for (const c of chunks) controller.enqueue(new TextEncoder().encode(c))
      controller.close()
    },
  })
  await consumeSSEStream(stream, cbs, () => false)
  assert.deepEqual(blocks, ['book:1', 'end'])
})

test('GAP 81：consumeSSEStream 用户取消不触发 onStreamError', async () => {
  let streamErr = ''
  const cbs: SSEStreamCallbacks = {
    onBooks: () => {},
    onEnd: () => {},
    onErrorEvent: () => {},
    onStreamError: (msg) => {
      streamErr = msg
    },
  }
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(new TextEncoder().encode('event: book\ndata: {"lastIndex":0,"data":[]}\n\n'))
      controller.error(new Error('aborted'))
    },
  })
  await consumeSSEStream(stream, cbs, () => true)
  assert.equal(streamErr, '')
})

test('P2 SSE 统一：consumeSSEStreamBlocks 按块回调 + 跨 chunk 拼接（cacheBook/sourceDebug 共用）', async () => {
  const blocks: string[] = []
  const raw =
    'data: {"type":"step","message":"1"}\n\ndata: {"type":"step","message":"2"}\n\ndata: {"type":"result","data":1}\n\n'
  const cut = Math.floor(raw.length / 2)
  const chunks = [raw.slice(0, cut), raw.slice(cut)]
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      for (const c of chunks) controller.enqueue(new TextEncoder().encode(c))
      controller.close()
    },
  })
  await consumeSSEStreamBlocks(stream, (b) => blocks.push(b), () => false)
  assert.equal(blocks.length, 3)
  assert.match(blocks[0], /message.:.1/)
  assert.match(blocks[2], /result/)
})

test('P2 SSE 统一：consumeSSEStreamBlocks 连接中断回调 onStreamError；用户取消静默', async () => {
  let errMsg = ''
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(new TextEncoder().encode('data: {"a":1}\n\n'))
      controller.error(new Error('boom'))
    },
  })
  await consumeSSEStreamBlocks(
    stream,
    () => {},
    () => false,
    (msg) => {
      errMsg = msg
    },
  )
  assert.ok(errMsg.includes('连接中断'))

  // 用户取消：不回调 onStreamError
  errMsg = ''
  const stream2 = new ReadableStream<Uint8Array>({
    start(controller) {
      controller.error(new Error('aborted'))
    },
  })
  await consumeSSEStreamBlocks(
    stream2,
    () => {},
    () => true,
    (msg) => {
      errMsg = msg
    },
  )
  assert.equal(errMsg, '')
})
