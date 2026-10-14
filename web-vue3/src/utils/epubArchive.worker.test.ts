import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { runInNewContext } from 'node:vm'
import ts from 'typescript'

const source = readFileSync(new URL('./epubArchive.worker.ts', import.meta.url), 'utf8')
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText

function producer(decode: () => Map<string, Uint8Array>) {
  const messages: { value: any; transfer: Transferable[] }[] = []
  let closed = 0
  const scope = {
    onmessage: null as ((event: MessageEvent<ArrayBuffer>) => void) | null,
    postMessage(value: unknown, transfer: Transferable[]) { messages.push({ value, transfer }) },
    close() { closed++ },
    Uint8Array, Error, exports: {},
    require(name: string) {
      assert.equal(name, './epubArchive')
      return { expandEpubArchive: decode }
    },
  }
  runInNewContext(compiled, scope)
  return { scope, messages, get closed() { return closed } }
}

test('实际 Worker 源码在完成解码后才宣告结果传递，阶段不泄露资源且转移全部原字节', () => {
  const files = new Map([['generated/a.xhtml', new Uint8Array([1, 2])], ['generated/b.css', new Uint8Array([3])]])
  const fixture = producer(() => files)
  assert.equal(fixture.messages.length, 1)
  assert.equal(fixture.messages[0].value.phase, 'ready')
  fixture.scope.onmessage!({ data: new ArrayBuffer(2) } as MessageEvent<ArrayBuffer>)
  assert.deepEqual(fixture.messages.slice(0, 3).map(message => message.value.phase), ['ready', 'expanding', 'transferring'])
  for (const message of fixture.messages.slice(0, 3)) {
    assert.deepEqual(Object.keys(message.value), ['phase'])
    assert.equal(message.transfer.length, 0)
  }
  const result = fixture.messages[3]
  assert.deepEqual([...result.value.files].map(([name, bytes]) => [name, [...bytes]]),
    [...files].map(([name, bytes]) => [name, [...bytes]]))
  assert.equal(result.transfer.length, 2)
  assert.equal(result.transfer[0], files.get('generated/a.xhtml')!.buffer)
  assert.equal(result.transfer[1], files.get('generated/b.css')!.buffer)
  assert.equal(fixture.closed, 1)
})

test('实际 Worker 解码失败不宣告已完成或传递结果，仍关闭且过滤非 EPUB 私有错误', () => {
  for (const error of [new Error('EPUB generated limit'), new Error('PRIVATE_PATH_OR_CONTENT')]) {
    const fixture = producer(() => { throw error })
    fixture.scope.onmessage!({ data: new ArrayBuffer(2) } as MessageEvent<ArrayBuffer>)
    assert.deepEqual(fixture.messages.slice(0, 2).map(message => message.value.phase), ['ready', 'expanding'])
    assert.equal(fixture.messages.length, 3)
    assert.equal(fixture.messages[2].value.error,
      error.message.startsWith('EPUB ') ? error.message : 'EPUB ZIP 无效或无法解压')
    assert.equal(JSON.stringify(fixture.messages).includes('PRIVATE_'), false)
    assert.equal(fixture.closed, 1)
  }
})
