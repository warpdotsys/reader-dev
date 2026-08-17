/**
 * 后端可达通知（P2：backendDown 永不重置修复）。
 *
 * httpTts / replaceRules / sourceSubs 等「后端优先、localStorage 降级」模块在后端失败后
 * 会置 backendDown 短路（避免每次操作都等 15s 超时）；本模块提供轻量注册表——
 * request.ts 拦截器在任一后端请求成功（或收到 HTTP 响应）时通知，各模块借此复位
 * 短路标志，网络恢复 / 重新登录后自动回到后端优先。
 */

type ResetFn = () => void

const resets = new Set<ResetFn>()

/** 注册复位回调（降级模块在模块加载时调用一次） */
export function onBackendReachable(fn: ResetFn): void {
  resets.add(fn)
}

/** 后端确认可达：触发全部已注册复位回调 */
export function notifyBackendReachable(): void {
  for (const fn of resets) fn()
}

/** 仅供测试/调试：清空注册表 */
export function clearBackendReachableHooks(): void {
  resets.clear()
}
