import request from './request'

/** The restored Java/Kotlin endpoint supports TXT and EPUB through `isEpub`. */
export type ExportFormat = 'txt' | 'epub'

/** GET /reader3/exportBook returns the generated file or a JSON ReturnData error. */
export function exportBook(url: string, format: ExportFormat): Promise<Blob> {
  return request
    .get('/exportBook', {
      params: { url, isEpub: format === 'epub' ? 1 : 0 },
      responseType: 'blob',
      timeout: 120_000,
      silent: true,
    })
    .then((response) => response.data as Blob)
}
