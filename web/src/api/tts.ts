import { get } from './request'
import { useUserStore } from '@/stores/user'
import type { ReturnData } from '@/types'

/**
 * 后端 TTS 语音合成（F-25）：
 * - GET  /reader3/getTTSVoices → ReturnData<{name,value,locale,gender}[]>
 * - GET/POST /reader3/tts      → 参数 text/voice/rate/pitch/volume/style/engine/url
 *   成功返回 audio/mpeg 字节流；失败返回 ReturnData JSON
 *
 * 注意：合成走 POST + JSON body 而非 GET query —— 整章文本放进 URL 会超过
 * 服务端请求头缓冲（hyper 默认 ~8KB）与代理限制，长章必失败。
 */

/** Edge TTS 语音（getTTSVoices 单项） */
export interface TtsVoice {
  name: string
  value: string
  locale: string
  gender: string
}

/** /reader3/tts 合成参数 */
export interface TtsSynthesizeParams {
  text: string
  voice: string
  /** Edge 百分比格式：+0% / +10% / -50% */
  rate: string
  /** Edge Hz 格式：+0Hz / -2Hz */
  pitch: string
  /** Edge 音量格式：+0% / +10% / -20% */
  volume?: string
  /** Edge express-as 风格（cheerful/sad 等，可选） */
  style?: string
  engine: 'edge' | 'http'
  /** engine=http 时的 HttpTTS 地址 */
  httpUrl?: string
}

/** GET /reader3/getTTSVoices：可用语音列表（静默失败，调用方降级） */
export function getTtsVoices(): Promise<ReturnData<TtsVoice[]>> {
  return get<TtsVoice[]>('/getTTSVoices', undefined, { silent: true })
}

/** POST /reader3/tts：合成整章音频 → Blob（业务失败抛 Error） */
export async function synthesizeTts(p: TtsSynthesizeParams): Promise<Blob> {
  const store = useUserStore()
  const params = new URLSearchParams()
  if (store.accessToken) params.set('accessToken', store.accessToken)
  const qs = params.toString()
  const res = await fetch(`/reader3/tts${qs ? `?${qs}` : ''}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      text: p.text,
      voice: p.voice,
      rate: p.rate,
      pitch: p.pitch,
      volume: p.volume ?? '+0%',
      style: p.style ?? '',
      engine: p.engine,
      url: p.engine === 'http' ? p.httpUrl : undefined,
    }),
  })
  const ct = res.headers.get('Content-Type') ?? ''
  // 成功：audio/mpeg 流；失败：ReturnData JSON（HTTP 200 + isSuccess=false）
  if (!res.ok || ct.includes('application/json')) {
    let msg = '语音合成失败'
    try {
      const j = (await res.json()) as { errorMsg?: string }
      if (j?.errorMsg) msg = j.errorMsg
    } catch {
      /* 非 JSON 错误体，保留默认文案 */
    }
    throw new Error(msg)
  }
  return res.blob()
}
