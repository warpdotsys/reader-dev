import { get, post } from './request'

export interface ReaderLicense {
  host: string
  userMaxLimit: number
  expiredAt: number
  simpleWebExpiredAt: number
  type: string
}

function licenseValue(value: unknown): ReaderLicense {
  const item = value as Partial<ReaderLicense> | null
  if (!item || typeof item.host !== 'string' || typeof item.type !== 'string'
      || !Number.isSafeInteger(item.userMaxLimit) || item.userMaxLimit! < 0
      || !Number.isSafeInteger(item.expiredAt) || item.expiredAt! < 0
      || !Number.isSafeInteger(item.simpleWebExpiredAt) || item.simpleWebExpiredAt! < 0) {
    throw new Error('许可证响应格式无效')
  }
  return item as ReaderLicense
}

function emailValue(value: string): string {
  const email = value.trim()
  if (email.length > 254 || !/^[^\s@\x00-\x1f\x7f]+@[^\s@\x00-\x1f\x7f]+\.[^\s@\x00-\x1f\x7f]+$/.test(email)) {
    throw new Error('请填写有效邮箱')
  }
  return email
}

export async function loadLicense(signal: AbortSignal): Promise<ReaderLicense> {
  const response = await get<{ license: unknown }>('/getLicense', undefined, { silent: true, signal })
  if (response.isSuccess !== true) throw new Error(response.errorMsg || '读取许可证失败')
  return licenseValue(response.data?.license)
}

export async function sendTrialCode(email: string, signal: AbortSignal): Promise<void> {
  const response = await post<unknown>('/sendCodeToEmail', { email: emailValue(email) }, { silent: true, signal })
  if (response.isSuccess !== true || response.data !== '') throw new Error(response.errorMsg || '未确认验证码发送成功')
}

export async function claimTrialLicense(email: string, code: string, signal: AbortSignal): Promise<string> {
  if (!/^\d{8}$/.test(code.trim())) throw new Error('请输入 8 位邮箱验证码')
  const response = await post<{ key?: unknown }>('/supplyLicense', { email: emailValue(email), code: code.trim() }, { silent: true, signal })
  const key = response.data?.key
  if (response.isSuccess !== true || typeof key !== 'string' || !key || key.length > 32768) {
    throw new Error(response.errorMsg || '未收到有效试用密钥')
  }
  return key
}

export async function installLicense(content: string, managerKey: string, signal: AbortSignal): Promise<ReaderLicense> {
  if (!content.trim() || content.length > 32768) throw new Error('请输入有效密钥（最多 32768 字符）')
  if (!managerKey || managerKey.length > 4096 || /[\x00-\x1f\x7f]/.test(managerKey)) throw new Error('请输入有效管理密码')
  const response = await post<{ license: unknown }>('/importLicense', { content: content.trim() }, {
    silent: true, signal, headers: { 'X-Reader-Secure-Key': managerKey },
  })
  if (response.isSuccess !== true) throw new Error(response.errorMsg || '许可证导入未成功')
  return licenseValue(response.data?.license)
}
