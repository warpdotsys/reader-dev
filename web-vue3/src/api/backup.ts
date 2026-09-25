import { post } from './request'
import { downloadFile, listFiles } from './file'
import type { ReturnData } from '@/types'
import { latestLegacyWebdavBackup, LEGACY_WEBDAV_BACKUP_DIR } from '@/utils/backup'

/**
 * POST /reader3/backupToWebdav：备份数据到 WebDAV。
 * Legacy 合约不接收路径参数，成功时 data 为空字符串；文件固定写入当前用户 home 下
 * webdav/legado/backupYYYY-MM-DD.zip。
 */
export function backupToWebdav(): Promise<ReturnData<string>> {
  return post<string>('/backupToWebdav')
}

/** List only the caller's legacy backup folder and return the newest dated archive path. */
export async function getLatestWebdavBackup() {
  const listing = await listFiles(LEGACY_WEBDAV_BACKUP_DIR, '__HOME__')
  const backup = latestLegacyWebdavBackup(listing.data ?? [])
  if (!backup) throw new Error('备份接口未在当前用户的 webdav/legado 目录生成 ZIP')
  return backup
}

/** Download a dated backup only when its path is inside the caller's legacy backup folder. */
export async function downloadWebdavBackup(path: string): Promise<Blob> {
  const name = path.replace(/\\/g, '/').split('/').pop() ?? ''
  const backup = latestLegacyWebdavBackup([{ name, path, isDirectory: false, lastModified: 0 }])
  if (!backup) throw new Error('备份路径无效')
  return downloadFile(backup.path, '__HOME__')
}
