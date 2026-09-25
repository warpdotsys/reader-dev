/**
 * P1-5 RSS article rich media playback (subset of Pro hls/dash/flv/webtorrent):
 * - Native video/audio (mp4/mp3/webm/ogg) plays directly by the browser
 * - HLS (.m3u8): not native in Chrome, attach via dynamic import of hls.js
 *   (lazy loaded to avoid bundle bloat)
 * - Others (flv/dash etc.) not supported yet
 *
 * Security: elements inside container are already sanitized by sanitizeHtml
 * (no script tags, no on-attributes, no dangerous protocols). This module only
 * enhances binding and never injects new content.
 */

/** Minimal hls.js instance interface */
interface HlsInstance {
  loadSource(src: string): void
  attachMedia(media: HTMLMediaElement): void
  on(evt: string, cb: (data: unknown) => void): void
  destroy(): void
}

/** Minimal hls.js constructor config */
interface HlsConfig {
  xhrSetup?: (xhr: XMLHttpRequest, url: string) => void
}

/** Cached hls.js module */
type HlsCtor = new (cfg?: HlsConfig) => HlsInstance
let HlsModule: HlsCtor | null = null

async function ensureHls(): Promise<HlsCtor | null> {
  if (HlsModule) return HlsModule
  try {
    const mod = (await import('hls.js')) as { default: HlsCtor }
    HlsModule = mod.default ?? null
    return HlsModule
  } catch {
    return null
  }
}

/** Check whether a media src is an HLS playlist */
export function isHlsSrc(src: string): boolean {
  return /\.m3u8(\?|#|$)/i.test(src)
}

/**
 * Scan video/audio elements inside container and enhance:
 * - HLS sources get attached to a dynamically imported hls.js instance;
 *   if hls.js fails to load the element is marked data-media-error.
 * - Returns a disposer (call on unmount to destroy hls instances).
 */
export function enhanceRssMedia(container: HTMLElement): () => void {
  const medias = Array.from(container.querySelectorAll<HTMLMediaElement>('video, audio'))
  const disposers: (() => void)[] = []

  for (const el of medias) {
    const src = el.getAttribute('src') ?? el.querySelector('source')?.getAttribute('src') ?? ''
    if (!src || !isHlsSrc(src)) continue // empty or natively-playable

    void (async () => {
      const Hls = await ensureHls()
      if (!Hls) {
        el.dataset.mediaError = 'hls-unavailable'
        return
      }
      const hls = new Hls({
        xhrSetup: (xhr) => {
          xhr.withCredentials = false
        },
      })
      hls.attachMedia(el)
      hls.loadSource(new URL(src, window.location.href).href)
      disposers.push(() => hls.destroy())
    })()
  }

  return () => {
    for (const d of disposers) d()
    disposers.length = 0
  }
}
