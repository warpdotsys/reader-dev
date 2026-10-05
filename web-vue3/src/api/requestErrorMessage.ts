/** 展示错误不改变原错误对象，保留调用方的状态、取消与离线分类。 */
export function requestErrorMessage(error: unknown): string {
  const value = error as {
    response?: { status?: unknown; data?: { errorMsg?: unknown } }
    code?: unknown
    message?: unknown
  } | null | undefined
  const serverMessage = value?.response?.data?.errorMsg
  if (typeof serverMessage === 'string' && serverMessage.trim()) return serverMessage
  const status = value?.response?.status
  if (typeof status === 'number' && Number.isInteger(status) && status >= 400 && status <= 599) {
    return `请求失败（HTTP ${status}）`
  }
  const message = typeof value?.message === 'string' ? value.message : ''
  if (!value?.response) {
    if (value?.code === 'ERR_NETWORK' || message === 'Network Error') {
      return '网络连接失败，请检查连接后重试'
    }
    if (value?.code === 'ECONNABORTED' || value?.code === 'ETIMEDOUT' || /^timeout of \d+ms exceeded$/.test(message)) {
      return '请求超时，请稍后重试'
    }
  }
  return message || '网络错误'
}
