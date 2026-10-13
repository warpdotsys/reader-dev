/** 保留 Axios 已解析 Request 的传输字段，避免运行时再次从 Object.prototype 取选项。 */
export function readerFetch(nativeFetch: typeof fetch): typeof fetch {
  return (input, init) => {
    const options = Object.assign(Object.create(null), init) as RequestInit & { duplex?: string }
    if (typeof Request !== 'undefined' && input instanceof Request) {
      // 显式自有字段也防御运行时先复制成普通对象、再补默认字段的行为。
      // 不改 URL、请求方法、Cookie 策略、取消信号、流或实际请求头。
      const headers: Record<string, string> = Object.create(null)
      input.headers.forEach((value, name) => { headers[name] = value })
      options.headers = headers
      options.method = input.method
      options.body = input.body
      options.signal = input.signal
      options.credentials = input.credentials
      options.mode = input.mode
      options.cache = input.cache
      options.integrity = input.integrity
      options.referrer = input.referrer
      options.referrerPolicy = input.referrerPolicy
      options.keepalive = input.keepalive
      // Axios 的 maxRedirects=0 在第二参数显式设置 manual，不能覆盖为 Request 原值。
      if (!Object.hasOwn(options, 'redirect')) options.redirect = input.redirect
      const duplex = (input as Request & { duplex?: string }).duplex
      if (duplex !== undefined) options.duplex = duplex
    }
    return nativeFetch(input, options)
  }
}
