/**
 * 共享错误判断工具（P3-A：isNotImplemented 六处手抄收敛为单一实现）
 *
 * 原实现散落于 users.ts / BookDetailView / BookshelfView / SettingsView /
 * SourceManageView 五处（字节级重复）；现统一在此，各视图与 API 层从此处导入。
 */

/** 判断接口是否未实现（404/501/网络失败）——用于 addUser 未就绪时降级 register、
 * 老后端无某接口时前端降级等场景 */
export function isNotImplemented(err: unknown): boolean {
  const e = err as { response?: { status?: number }; message?: string } | null | undefined
  const status = e?.response?.status
  if (status === 404 || status === 501) return true
  const msg = e?.message ?? ''
  return !e?.response && (msg.includes('404') || msg.includes('Network Error'))
}
