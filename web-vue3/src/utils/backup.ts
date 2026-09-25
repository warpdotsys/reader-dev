export interface BackupFileCandidate {
  name: string
  path: string
  isDirectory: boolean
  lastModified: number | string
}

export interface LegacyWebdavBackup {
  name: string
  path: string
  lastModified: number
}

export const LEGACY_WEBDAV_BACKUP_DIR = 'webdav/legado'

/** Resolve the newest ZIP written by the legacy backup endpoint, rejecting paths outside its home folder. */
export function latestLegacyWebdavBackup(items: BackupFileCandidate[]): LegacyWebdavBackup | null {
  const prefix = `${LEGACY_WEBDAV_BACKUP_DIR}/`
  const candidates = items.flatMap((item) => {
    if (item.isDirectory || !/^backup\d{4}-\d{2}-\d{2}\.zip$/i.test(item.name)) return []

    const path = item.path.replace(/\\/g, '/').replace(/^\/+/, '')
    const parts = path.split('/')
    if (
      !path.startsWith(prefix) ||
      parts.includes('..') ||
      parts.length !== 3 ||
      parts[2] !== item.name
    ) return []

    const lastModified = typeof item.lastModified === 'number'
      ? item.lastModified
      : /^\d+$/.test(item.lastModified)
        ? Number(item.lastModified)
        : Date.parse(item.lastModified)

    return [{ name: item.name, path, lastModified: Number.isFinite(lastModified) ? lastModified : 0 }]
  })

  candidates.sort((a, b) => b.lastModified - a.lastModified || b.name.localeCompare(a.name))
  return candidates[0] ?? null
}
