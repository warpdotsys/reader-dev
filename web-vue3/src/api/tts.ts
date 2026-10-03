import { useUserStore } from '@/stores/user'
import type { ReturnData } from '@/types'

/**
 * 后端 TTS 语音合成（F-25）：
 * - Java/Kotlin 已注册 GET/POST /reader3/book/tts；参数为 text/type/voice/rate/pitch/base64。
 *   成功返回 audio/mpeg 字节流；失败返回 ReturnData JSON
 * - type=api&voice={HttpTTS名称}&base64=1（legacy 契约）：按名分派听书源，
 *   成功返回 ReturnData JSON 包裹的音频 base64 字符串
 *
 * 注意：合成走 POST + JSON body 而非 GET query —— 整章文本放进 URL 会超过
 * 服务端请求头缓冲（hyper 默认 ~8KB）与代理限制，长章必失败。
 */

/** Edge TTS 语音项。服务端没有列举路由，默认音色仍可直接合成。 */
export interface TtsVoice {
  name: string
  value: string
  locale: string
  gender: string
}

/** /reader3/book/tts 合成参数 */
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

/**
 * 原 JAR 与当前服务端均没有 getTTSVoices。不要捏造一份可能与服务端不同步的
 * 列表；阅读器会保留默认音色并允许直接合成。
 */
export function getTtsVoices(): Promise<ReturnData<TtsVoice[]>> {
  return Promise.reject(new Error('当前 Java/Kotlin 服务端未实现 TTS 音色列表接口'))
}

/** base64 → audio Blob（type=api&base64=1 成功响应解码） */
function base64ToBlob(b64: string): Blob {
  const bin = atob(b64)
  const bytes = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
  return new Blob([bytes], { type: 'audio/mpeg' })
}

/**
 * BookController 会把 pitch 原样追加 "%"；Vue 的 UI 值是 Edge 的 "+0Hz"，
 * 故转换为旧端点要求的数值形式，避免发送无效的 "+0Hz%"。
 */
function toLegacyPitch(value: string): string {
  return value.trim().replace(/hz$/i, '') || '0'
}

/** POST /reader3/book/tts：合成整章音频 → Blob（业务失败抛 Error） */
export async function synthesizeTts(p: TtsSynthesizeParams): Promise<Blob> {
  const store = useUserStore()
  const params = new URLSearchParams()
  if (store.accessToken) params.set('accessToken', store.accessToken)
  const qs = params.toString()
  // engine=http → legacy type=api 契约：voice={HttpTTS名称} 按名分派 + base64=1 JSON 包裹响应
  const useApi = p.engine === 'http' && !!p.httpName
  const res = await fetch(`/reader3/book/tts${qs ? `?${qs}` : ''}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      text: p.text,
      voice: useApi ? p.httpName : p.voice,
      rate: p.rate,
      pitch: toLegacyPitch(p.pitch),
      // book/tts 的 legacy 分派字段是 type，而非 Rust 版的 engine。
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
