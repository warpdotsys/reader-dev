/**
 * 屏幕常亮（Wake Lock API）——阅读页进入时请求，页面切后台/离开时释放；
 * 不支持或请求失败的环境静默跳过（不抛错、不影响阅读）。
 */

export interface WakeLockHandle {
  release: () => Promise<void>
  addEventListener?: (type: 'release', cb: () => void) => void
}

let lock: WakeLockHandle | null = null

function isSupported(): boolean {
  return typeof navigator !== 'undefined' && 'wakeLock' in navigator
}

/** 是否正持有屏幕常亮 */
export function isWakeLockHeld(): boolean {
  return lock !== null
}

/** 释放屏幕常亮（未持有时为空操作；失败静默） */
export async function releaseWakeLock(): Promise<void> {
  const l = lock
  lock = null
  if (l) {
    try {
      await l.release()
    } catch {
      /* 静默 */
    }
  }
}

/**
 * 请求屏幕常亮。返回是否成功持有。
 * - 不支持 Wake Lock 的环境直接返回 false；
 * - 已持有则先释放再重新请求（避免重复锁）；
 * - 浏览器在后台自动释放时（release 事件）内部置空，页面重新可见后再次请求即可。
 */
export async function requestWakeLock(): Promise<boolean> {
  if (!isSupported()) return false
  await releaseWakeLock()
  try {
    const nav = navigator as Navigator & {
      wakeLock: { request: (type: 'screen') => Promise<WakeLockHandle> }
    }
    const l = await nav.wakeLock.request('screen')
    l.addEventListener?.('release', () => {
      if (lock === l) lock = null
    })
    lock = l
    return true
  } catch {
    lock = null
    return false
  }
}
