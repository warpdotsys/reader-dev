import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readerFetch } from './readerFetch.ts'
import { readFileSync } from 'node:fs'

test('Reader fetch：已解析字段为自有值，输入与原选项保持不变', async () => {
  const input=new Request('http://127.0.0.1/generated',{method:'POST',body:'generated-body',headers:{'X-Generated':'yes'},credentials:'include',cache:'no-store',referrerPolicy:'no-referrer'})
  const init=Object.freeze({redirect:'manual' as const})
  let called=false
  const native=(async (request,options)=>{
    called=true
    assert.equal(request,input)
    assert.equal(Object.getPrototypeOf(options),null)
    for(const key of ['headers','method','body','signal','credentials','cache','mode','redirect','referrer','referrerPolicy','integrity','keepalive'])assert.ok(Object.hasOwn(options!,key))
    assert.deepEqual({...options!.headers},{'content-type':'text/plain;charset=UTF-8','x-generated':'yes'})
    assert.equal(options!.method,'POST');assert.equal(options!.credentials,'include')
    assert.equal(options!.redirect,'manual');assert.equal(options!.signal,input.signal)
    return new Response('generated')
  }) as typeof fetch
  const response=await readerFetch(native)(input,init)
  assert.equal(await response.text(),'generated');assert.ok(called)
  assert.deepEqual(init,{redirect:'manual'});assert.equal(input.bodyUsed,false)
})

test('Reader fetch：已有Request取消信号和native错误原样传播', async () => {
  const controller=new AbortController();controller.abort()
  const input=new Request('http://127.0.0.1/generated',{signal:controller.signal})
  const primary=new Error('generated primary')
  const native=(async (_input,options)=>{assert.ok(options!.signal!.aborted);throw primary}) as typeof fetch
  await assert.rejects(readerFetch(native)(input),error=>error===primary)
})

test('Reader fetch：URL输入只复制自有选项，不使用继承的鉴权头', async () => {
  const init=Object.assign(Object.create({headers:{Authorization:'Generated-inherited'}}),{cache:'no-store'})
  const native=(async (_input,options)=>{
    assert.equal(Object.getPrototypeOf(options),null)
    assert.equal(options!.headers,undefined);assert.equal(options!.cache,'no-store')
    return new Response('generated')
  }) as typeof fetch
  await readerFetch(native)('http://127.0.0.1/generated',init)
  assert.ok(!Object.hasOwn(init,'headers'))
})

test('Reader fetch：实际请求实例接入守卫，不强制替换XHR默认适配器或15秒期限',()=>{
  const source=readFileSync(new URL('./request.ts',import.meta.url),'utf8')
  assert.match(source,/env:\s*\{\s*fetch:\s*readerFetch\(globalThis\.fetch\.bind\(globalThis\)\)/)
  assert.match(source,/timeout:\s*15000/)
  assert.doesNotMatch(source,/adapter:\s*['"]fetch['"]|Object\.prototype\s*=/)
})
