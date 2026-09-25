import { get } from './request'
import { useUserStore } from '@/stores/user'
import type { ReturnData } from '@/types'

/**
 * 后端 TTS 语音合成（F-25）：
 * - GET  /reader3/getTTSVoices → ReturnData<{name,value,locale,gender}[]>
 * - GET/POST /reader3/tts      → 参数 text/voice/rate/pitch/volume/style/engine/url
 *   成功返回 audio/mpeg 字节流；失败返回 ReturnData JSON
 * - type=api&voice={HttpTTS名称}&base64=1（legacy 契约）：按名分派听书源，
 *   成功返回 ReturnData JSON 包裹的音频 base64 字符串
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
  /**
   * 语速：engine=edge 为 Edge 百分比格式（+0% / +10% / -50%）；
   * engine=http 为纯数字字符串（legacy 语义，后端按 speechRate=(5+(rate-0.5)*30) 映射）
   */
  rate: string
  /** Edge Hz 格式：+0Hz / -2Hz（api 分派时忽略） */
  pitch: string
  /** Edge 音量格式：+0% / +10% / -20%（api 分派时忽略） */
  volume?: string
  /** Edge express-as 风格（cheerful/sad 等，可选；api 分派时忽略） */
  style?: string
  engine: 'edge' | 'http'
  /** engine=http：HttpTTS 源名称（type=api&voice={名称} 按名分派，必填） */
  httpName?: string
}

/** GET /reader3/getTTSVoices：可用语音列表（静默失败，调用方降级） */
export function getTtsVoices(): Promise<ReturnData<TtsVoice[]>> {
  return get<TtsVoice[]>('/getTTSVoices', undefined, { silent: true })
}

/** base64 → audio Blob（type=api&base64=1 成功响应解码） */
function base64ToBlob(b64: string): Blob {
  const bin = atob(b64)
  const bytes = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
  return new Blob([bytes], { type: 'audio/mpeg' })
}

/** POST /reader3/tts：合成整章音频 → Blob（业务失败抛 Error） */
export async function synthesizeTts(p: TtsSynthesizeParams): Promise<Blob> {
  const store = useUserStore()
  const params = new URLSearchParams()
  if (store.accessToken) params.set('accessToken', store.accessToken)
  const qs = params.toString()
  // engine=http → legacy type=api 契约：voice={HttpTTS名称} 按名分派 + base64=1 JSON 包裹响应
  const useApi = p.engine === 'http' && !!p.httpName
  const res = await fetch(`/reader3/tts${qs ? `?${qs}` : ''}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      text: p.text,
      voice: useApi ? p.httpName : p.voice,
      rate: p.rate,
      pitch: p.pitch,
      volume: p.volume ?? '+0%',
      style: p.style ?? '',
      engine: useApi ? undefined : p.engine,
      type: useApi ? 'api' : undefined,
      base64: useApi ? '1' : undefined,
    }),
  })
  const ct = res.headers.get('Content-Type') ?? ''
  // JSON 响应：base64=1 的成功结果（ReturnData 包 base64 音频），或失败（ReturnData.errorMsg）
  if (ct.includes('application/json')) {
    let j: ReturnData<string> | null = null
    try {
      j = (await res.json()) as ReturnData<string>
    } catch {
      /* 非 JSON 错误体，保留默认文案 */
    }
    if (res.ok && j && j.isSuccess && typeof j.data === 'string' && j.data) {
      return base64ToBlob(j.data)
    }
    throw new Error(j?.errorMsg || '语音合成失败')
  }
  if (!res.ok) throw new Error('语音合成失败')
  return res.blob()
}
