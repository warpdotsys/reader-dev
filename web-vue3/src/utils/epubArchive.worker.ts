import { expandEpubArchive } from './epubArchive'

const scope = globalThis as unknown as {
  onmessage: ((event: MessageEvent<ArrayBuffer>) => void) | null
  postMessage: (value: unknown, transfer: Transferable[]) => void
  close: () => void
}

scope.onmessage = event => {
  try {
    const files = [...expandEpubArchive(new Uint8Array(event.data))]
    scope.postMessage({ files }, files.map(([, data]) => data.buffer as ArrayBuffer))
  } catch (error) {
    const detail = error instanceof Error ? error.message : ''
    scope.postMessage({ error: detail.startsWith('EPUB ') ? detail : 'EPUB ZIP 无效或无法解压' }, [])
  } finally { scope.close() }
}
