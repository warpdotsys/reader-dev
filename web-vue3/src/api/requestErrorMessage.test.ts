import assert from 'node:assert/strict'
import { test } from 'node:test'
import { requestErrorMessage } from './requestErrorMessage.ts'

test('纯网络错误用中文连接提示，不改变原来的错误分类', () => {
  const error = Object.assign(new Error('Network Error'), { code: 'ERR_NETWORK' })
  assert.equal(requestErrorMessage(error), '网络连接失败，请检查连接后重试')
  assert.equal(error.message, 'Network Error')
  assert.equal(error.code, 'ERR_NETWORK')
})

test('旧无 code 网络错误仍翻译；普通业务错误保留原文', () => {
  assert.equal(requestErrorMessage(new Error('Network Error')), '网络连接失败，请检查连接后重试')
  assert.equal(requestErrorMessage(new Error('未开启本地书仓功能')), '未开启本地书仓功能')
})

test('超时信息用中文，不展示 axios 的英文内部提示', () => {
  for (const code of ['ECONNABORTED', 'ETIMEDOUT']) {
    assert.equal(requestErrorMessage({ code, message: 'timeout of 15000ms exceeded' }),
      '请求超时，请稍后重试')
  }
  assert.equal(requestErrorMessage({ message: 'timeout of 15000ms exceeded' }), '请求超时，请稍后重试')
})

test('没有业务信息的真实 HTTP 错误显示状态，不冒充离线', () => {
  for (const status of [401, 404, 503]) {
    assert.equal(requestErrorMessage({ response: { status }, message: 'Request failed with status code ' + status }),
      `请求失败（HTTP ${status}）`)
  }
})

test('服务端业务信息优先，NEED_SECURE_KEY 等判断保持原错误对象', () => {
  const error = { response: { status: 503, data: { errorMsg: '生成服务不可用', data: 'NEED_SECURE_KEY' } },
    code: 'ERR_NETWORK', message: 'Network Error' }
  assert.equal(requestErrorMessage(error), '生成服务不可用')
  assert.equal(error.response.data.data, 'NEED_SECURE_KEY')
})

test('有 HTTP 响应时不根据 Network Error 文本误报纯网络错误', () => {
  assert.equal(requestErrorMessage({ response: { status: 502 }, message: 'Network Error' }),
    '请求失败（HTTP 502）')
})

test('空值有中文兜底；非文本服务端字段不进入提示', () => {
  for (const value of [null, undefined, {}, { response: { data: { errorMsg: '   ' } } },
    { response: { data: { errorMsg: { generated: true } } } }]) {
    assert.equal(requestErrorMessage(value), '网络错误')
  }
})

test('展示层不修改冻结错误，也不额外读取请求头、URL 或凭据配置', () => {
  const error = Object.freeze({ code: 'ERR_NETWORK', message: 'Network Error',
    get config(): never { throw new Error('Do not read generated credential config') } })
  assert.equal(requestErrorMessage(error), '网络连接失败，请检查连接后重试')
})
