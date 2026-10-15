import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readFileSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const root=fileURLToPath(new URL('../../',import.meta.url))
const locked=JSON.parse(readFileSync(new URL('../../package-lock.json',import.meta.url)))
const required={axios:'1.20.0',vue:'3.5.42','@vue/server-renderer':'3.5.42',nanoid:'3.3.18','source-map-js':'1.2.2'}

function isolated(source) {
  // 涉及原型污染或旧版无限循环的样本只进入有限子进程，不污染其他测试。
  const result=spawnSync(process.execPath,['--max-old-space-size=128','--input-type=module','-e',source],
    {cwd:root,encoding:'utf8',timeout:5000,maxBuffer:65536})
  assert.equal(result.error,undefined,'安全样本必须正常结束，超时不是通过')
  assert.equal(result.status,0,result.stderr)
  assert.equal(result.signal,null)
  assert.equal(result.stdout.trim(),'PASS')
}

test('安全依赖：已审计补丁锁定且安装版本与锁一致',()=>{
  for(const[name,version]of Object.entries(required)) {
    const row=locked.packages['node_modules/'+name]
    assert.equal(row.version,version)
    assert.match(row.integrity,/^sha512-/)
    assert.equal(JSON.parse(readFileSync(new URL('../../node_modules/'+name+'/package.json',import.meta.url))).version,version)
  }
})

test('安全依赖：nanoid零尺寸生成器立即返回且不调用随机源',()=>isolated(`
  import assert from 'node:assert/strict';import {customRandom,customAlphabet} from 'nanoid';
  let calls=0;const random=n=>{calls++;return new Uint8Array(n)};
  assert.equal(customRandom('ab',0,random)(),'');assert.equal(calls,0);
  assert.equal(customRandom('ab',21,random)(0),'');assert.equal(calls,0);
  assert.equal(customAlphabet('ab',0)(),'');
  assert.equal(customRandom('ab',4,random)().length,4);console.log('PASS');
`))

test('安全依赖：source-map拒绝超大/非法/嵌套累计偏移，正常小映射不变',()=>isolated(`
  import assert from 'node:assert/strict';import maps from 'source-map-js';
  const {SourceMapConsumer}=maps;const flat={version:3,sources:['generated.js'],names:[],mappings:'AAAA'};
  const section=(line,column,map=flat)=>({version:3,sections:[{offset:{line,column},map}]});
  for(const n of [-1,1.5,Infinity,NaN,10000001,Number.MAX_SAFE_INTEGER])assert.throws(()=>new SourceMapConsumer(section(n,0)));
  assert.throws(()=>new SourceMapConsumer(section(1,0,section(10000000,0))));
  const mappings=[];new SourceMapConsumer(section(2,0)).eachMapping(value=>mappings.push(value));
  assert.equal(mappings.length,1);assert.equal(mappings[0].source,'generated.js');
  assert.equal(mappings[0].generatedLine,3);assert.equal(mappings[0].originalLine,1);console.log('PASS');
`))

test('安全依赖：Vue SSR不把回车属性名拼成事件和自动聚焦属性',()=>isolated(`
  import assert from 'node:assert/strict';import {ssrRenderAttrs} from '@vue/server-renderer';
  const prior=console.warn;console.warn=()=>{};let value;
  try{value=ssrRenderAttrs({'x\\rautofocus\\ronfocus':'generated-noop'});}finally{console.warn=prior;}
  assert.doesNotMatch(value,/onfocus|autofocus/);assert.equal(ssrRenderAttrs({title:'safe'}),' title="safe"');console.log('PASS');
`))

test('安全依赖：Axios fetch保留显式方法和生成鉴权头，不采用原型污染头',()=>isolated(`
  import assert from 'node:assert/strict';import http from 'node:http';import axios from 'axios';
  import {readerFetch} from './src/api/readerFetch.ts';
  const server=http.createServer((request,response)=>{response.setHeader('content-type','application/json');response.end(JSON.stringify({method:request.method,authorization:request.headers.authorization??null,good:request.headers['x-generated-good']??null}));});
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const before=Object.getOwnPropertyDescriptor(Object.prototype,'headers');assert.equal(before,undefined);
  Object.defineProperty(Object.prototype,'headers',{configurable:true,writable:true,value:{Authorization:'Generated-polluted'}});
  try{const result=await axios.get('http://127.0.0.1:'+server.address().port+'/',{adapter:'fetch',env:{fetch:readerFetch(fetch)},timeout:1000,headers:{'X-Generated-Good':'yes',Authorization:'Generated-clean'},fetchOptions:{}});
    assert.deepEqual(result.data,{method:'GET',authorization:'Generated-clean',good:'yes'});
  }finally{delete Object.prototype.headers;await new Promise(resolve=>server.close(resolve));}
  console.log('PASS');
`))

test('安全依赖：Reader fetch保持JSON、FormData、取消和禁止跟随重定向行为',()=>isolated(`
  import assert from 'node:assert/strict';import http from 'node:http';import axios from 'axios';
  import {readerFetch} from './src/api/readerFetch.ts';let redirected=0;
  const server=http.createServer(async(request,response)=>{
    if(request.url==='/redirect'){response.writeHead(302,{Location:'/target'});response.end();return;}
    if(request.url==='/target')redirected++;
    let body='';for await(const chunk of request)body+=chunk;
    response.setHeader('content-type','application/json');response.end(JSON.stringify({method:request.method,body,type:request.headers['content-type']??null}));
  });await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const base='http://127.0.0.1:'+server.address().port;
  const client=axios.create({adapter:'fetch',env:{fetch:readerFetch(fetch)},timeout:1000});
  try{
    const json=await client.post(base+'/',{generated:'中文'});
    assert.equal(json.data.method,'POST');assert.equal(JSON.parse(json.data.body).generated,'中文');
    const form=new FormData();form.set('generated','only-generated');
    const posted=await client.post(base+'/',form,{headers:{'Content-Type':'multipart/form-data'}});
    assert.match(posted.data.type,/^multipart\\/form-data; boundary=/);assert.ok(posted.data.body.includes('only-generated'));
    const redirect=await client.get(base+'/redirect',{maxRedirects:0,validateStatus:()=>true});
    assert.equal(redirect.status,302);assert.equal(redirected,0);
    const controller=new AbortController();controller.abort();
    await assert.rejects(client.get(base+'/',{signal:controller.signal}),error=>error.code==='ERR_CANCELED');
  }finally{await new Promise(resolve=>server.close(resolve));}console.log('PASS');
`))
