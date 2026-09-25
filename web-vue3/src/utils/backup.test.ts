import { test } from 'node:test'
import assert from 'node:assert/strict'
import { latestLegacyWebdavBackup } from './backup.ts'

test('selects the newest legacy backup ZIP and normalizes the user-home path', () => {
  assert.deepEqual(latestLegacyWebdavBackup([
    { name: 'backup2026-09-23.zip', path: '/webdav/legado/backup2026-09-23.zip', isDirectory: false, lastModified: 100 },
    { name: 'backup2026-09-25.zip', path: '\\webdav\\legado\\backup2026-09-25.zip', isDirectory: false, lastModified: 200 },
    { name: 'notes.txt', path: '/webdav/legado/notes.txt', isDirectory: false, lastModified: 300 },
    { name: 'backup2026-09-26.zip', path: '/webdav/legado/backup2026-09-26.zip', isDirectory: true, lastModified: 400 },
  ]), {
    name: 'backup2026-09-25.zip',
    path: 'webdav/legado/backup2026-09-25.zip',
    lastModified: 200,
  })
})

test('does not download a ZIP whose path escapes the expected backup directory', () => {
  assert.equal(latestLegacyWebdavBackup([
    { name: 'backup2026-09-25.zip', path: '/webdav/legado/../private/backup2026-09-25.zip', isDirectory: false, lastModified: 200 },
    { name: 'backup2026-09-24.zip', path: '/other/backup2026-09-24.zip', isDirectory: false, lastModified: 100 },
  ]), null)
})

test('returns no backup when the legacy folder has no dated archive', () => {
  assert.equal(latestLegacyWebdavBackup([
    { name: 'backup.zip', path: '/webdav/legado/backup.zip', isDirectory: false, lastModified: 200 },
  ]), null)
})
