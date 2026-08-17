import request, { type RequestOptions } from './request'
import type { ReturnData, FileItem } from '@/types'

/** secure 模式书仓写操作的管理密码（FileManageView 弹窗确认后设置；仅保存在本标签页） */
let fileSecureKey = ''

export function setFileSecureKey(key: string): void {
  fileSecureKey = key
}

function secureBody(extra: Record<string, unknown>): Record<string, unknown> {
  return fileSecureKey ? { ...extra, secureKey: fileSecureKey } : extra
}

/**
 * GET /reader3/file/list：文件列表
 * @param path 当前目录（根目录传空串）
 * @param home 可选：__LOCAL_STORE__=书仓 / __HOME__=用户数据 / __WEBDAV__=WebDAV / 空=用户根
 */
export function listFiles(path: string, home = ''): Promise<ReturnData<FileItem[]>> {
  return request
    .get('/file/list', { params: { path, ...(home ? { home } : {}) } })
    .then((r) => r.data as ReturnData<FileItem[]>)
}

/** GET /reader3/file/get：读取文本文件内容（home 需与 list 一致，否则解析到不同根） */
export function getFile(path: string, home = ''): Promise<ReturnData<string>> {
  return request
    .get('/file/get', { params: { path, ...(home ? { home } : {}) } })
    .then((r) => r.data as ReturnData<string>)
}

/** POST /reader3/file/save：写入文本文件（body { path, content }） */
export function saveFile(path: string, content: string, home = ''): Promise<ReturnData<null>> {
  return request
    .post('/file/save', secureBody({ path, content, ...(home ? { home } : {}) }))
    .then((r) => r.data as ReturnData<null>)
}

/** POST /reader3/file/mkdir：新建文件夹（body { path: 父目录, name: 文件夹名 }） */
export function mkdir(
  parent: string,
  name: string,
  home = '',
  opts?: RequestOptions,
): Promise<ReturnData<null>> {
  return request
    .post('/file/mkdir', secureBody({ path: parent, name, ...(home ? { home } : {}) }), {
      silent: opts?.silent,
    })
    .then((r) => r.data as ReturnData<null>)
}

/** POST /reader3/file/rename：重命名文件/目录（body { path, name }；secure 模式书仓写需管理密码） */
export function renameFile(path: string, name: string, home = ''): Promise<ReturnData<null>> {
  return request
    .post('/file/rename', secureBody({ path, name, ...(home ? { home } : {}) }))
    .then((r) => r.data as ReturnData<null>)
}

/** GET /reader3/file/download：下载文件，返回 Blob（大文件放宽超时） */
export function downloadFile(path: string, home = ''): Promise<Blob> {
  return request
    .get('/file/download', {
      params: { path, ...(home ? { home } : {}) },
      responseType: 'blob',
      timeout: 120_000,
    })
    .then((r) => r.data as Blob)
}

/**
 * POST /reader3/file/upload：multipart 上传（字段 file + path + home，FormData 交 axios 设 Content-Type）
 * @param onProgress 上传进度回调（0-100）
 */
export function uploadFile(
  file: File,
  path: string,
  home = '',
  onProgress?: (percent: number) => void,
): Promise<ReturnData<FileItem[]>> {
  const form = new FormData()
  form.append('file', file)
  form.append('path', path)
  if (home) form.append('home', home)
  if (fileSecureKey) form.append('secureKey', fileSecureKey)
  return request
    .post('/file/upload', form, {
      timeout: 120_000,
      onUploadProgress: onProgress
        ? (e) => {
            if (e.total) onProgress(Math.min(100, Math.round((e.loaded / e.total) * 100)))
          }
        : undefined,
    })
    .then((r) => r.data as ReturnData<FileItem[]>)
}

/**
 * POST /reader3/scanLocalBookDir：直接读取书仓/用户目录/WebDAV 中已有的书籍文件
 * 导入书架（无需再上传）。path 为文件时导入单本，为目录时递归扫描。
 */
export function scanLocalBookDir(
  path: string,
  home = '',
  recursive = true,
): Promise<ReturnData<{ imported: number; failed: number; total: number; errors: { name: string; error: string }[] }>> {
  return request
    .post('/scanLocalBookDir', { path, ...(home ? { home } : {}), recursive }, { timeout: 300_000 })
    .then((r) => r.data as ReturnData<{ imported: number; failed: number; total: number; errors: { name: string; error: string }[] }>)
}

/** POST /reader3/file/delete：删除文件/目录（body { path }） */
export function deleteFile(path: string, home = ''): Promise<ReturnData<null>> {
  return request
    .post('/file/delete', secureBody({ path, ...(home ? { home } : {}) }))
    .then((r) => r.data as ReturnData<null>)
}
