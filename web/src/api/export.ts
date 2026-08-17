import request from './request'

/**
 * 书籍导出 —— 后端契约（mobi/azw3 后端开发中，未就绪时返回错误 JSON 由调用方降级提示）
 *
 * GET /reader3/exportBook?url=<bookUrl>&format=txt|epub|html|mobi|azw3&encoding=utf-8|gbk
 *   → 成功：对应格式附件（blob 下载）
 *   → 失败：legacy ReturnData JSON 错误体（由调用方经 utils/download.ts downloadBlob 识别提示）
 * encoding 参数仅 txt 生效（后端并行实现中——未就绪时忽略，仍输出 UTF-8）。
 */

export type ExportFormat = 'txt' | 'epub' | 'html' | 'mobi' | 'azw3'

export type ExportEncoding = 'utf-8' | 'gbk'

/** 导出警告（X-Export-Warning 头，percent 编码 JSON）——非致命，随文件一同返回 */
export interface ExportWarning {
  /** 并发抓章失败的章节列表（P2：不再静默丢弃） */
  failedChapters?: { index: number; title: string; url: string; error: string }[]
  /** GBK 不可映射字符数（已转义为 &#x…; 保留原文，P2） */
  unmappableChars?: number
}

/** 导出结果：blob 文件 + 可选警告 */
export interface ExportResult {
  blob: Blob
  warning: ExportWarning | null
}

/** 解析 X-Export-Warning（percent 编码 JSON，HeaderValue 纯 ASCII 约束） */
function parseExportWarning(headers: Record<string, unknown>): ExportWarning | null {
  const raw = headers['x-export-warning']
  if (typeof raw !== 'string' || !raw) return null
  try {
    const parsed = JSON.parse(decodeURIComponent(raw)) as ExportWarning
    return parsed && typeof parsed === 'object' ? parsed : null
  } catch {
    return null // 头解析失败：不阻断下载
  }
}

/** GET /reader3/exportBook：导出本书为指定格式（blob + 警告，文件名由调用方拼 bookName.format） */
export function exportBook(
  url: string,
  format: ExportFormat,
  encoding: ExportEncoding = 'utf-8',
): Promise<ExportResult> {
  const params: Record<string, string> = { url, format }
  if (format === 'txt') params.encoding = encoding
  return request
    .get('/exportBook', {
      params,
      responseType: 'blob',
      timeout: 120_000,
      silent: true,
    })
    .then((r) => ({
      blob: r.data as Blob,
      warning: parseExportWarning(r.headers as Record<string, unknown>),
    }))
}
