import { ElMessage } from 'element-plus'

/**
 * 触发浏览器下载：将 Blob 保存为 filename。
 *
 * legacy 后端错误体为 HTTP 200 + JSON（ReturnData），此处识别并提示，返回 false（未触发下载）。
 * 注意：Content-Type 为 application/json 时也可能是合法 JSON 文件内容（如书源导出
 * /reader3/exportBookSources 成功时即返回 application/json 数组），
 * 仅当解析结果为 ReturnData 错误形态（isSuccess=false 或非空 errorMsg）才视为错误。
 */
export async function downloadBlob(blob: Blob, filename: string): Promise<boolean> {
  if (blob.type.includes('application/json')) {
    try {
      const parsed = JSON.parse(await blob.text()) as {
        isSuccess?: boolean
        errorMsg?: string
      }
      if (
        parsed &&
        typeof parsed === 'object' &&
        !Array.isArray(parsed) &&
        (parsed.isSuccess === false ||
          (typeof parsed.errorMsg === 'string' && parsed.errorMsg.length > 0))
      ) {
        ElMessage.error(parsed.errorMsg || '下载失败')
        return false
      }
    } catch {
      // 内容非 JSON（Content-Type 误标）：按文件正常下载
    }
  }
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
  return true
}
