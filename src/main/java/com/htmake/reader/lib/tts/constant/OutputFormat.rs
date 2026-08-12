use crate::prelude::*;
// package com.htmake.reader.lib.tts.constant;

// public enum OutputFormat {
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputFormat {
    amr_wb_16000hz,
    audio_16khz_16bit_32kbps_mono_opus,
    audio_16khz_32kbitrate_mono_mp3,
    audio_16khz_64kbitrate_mono_mp3,
    audio_16khz_128kbitrate_mono_mp3,
    audio_24khz_16bit_24kbps_mono_opus,
    audio_24khz_16bit_48kbps_mono_opus,
    audio_24khz_48kbitrate_mono_mp3,
    audio_24khz_96kbitrate_mono_mp3,
    audio_24khz_160kbitrate_mono_mp3,
    audio_48khz_96kbitrate_mono_mp3,
    audio_48khz_192kbitrate_mono_mp3,
    ogg_16khz_16bit_mono_opus,
    ogg_24khz_16bit_mono_opus,
    ogg_48khz_16bit_mono_opus,
    raw_8khz_8bit_mono_alaw,
    raw_8khz_8bit_mono_mulaw,
    raw_8khz_16bit_mono_pcm,
    raw_16khz_16bit_mono_pcm,
    raw_16khz_16bit_mono_truesilk,
    raw_22050hz_16bit_mono_pcm,
    raw_24khz_16bit_mono_pcm,
    raw_24khz_16bit_mono_truesilk,
    raw_44100hz_16bit_mono_pcm,
    raw_48khz_16bit_mono_pcm,
    webm_16khz_16bit_mono_opus,
    webm_24khz_16bit_24kbps_mono_opus,
    webm_24khz_16bit_mono_opus,
}

impl OutputFormat {
    // public String getValue() {
    //     return value;
    // }
    pub fn get_value(&self) -> &'static str {
        match self {
            OutputFormat::amr_wb_16000hz => "amr-wb-16000hz",
            OutputFormat::audio_16khz_16bit_32kbps_mono_opus => "audio-16khz-16bit-32kbps-mono-opus",
            OutputFormat::audio_16khz_32kbitrate_mono_mp3 => "audio-16khz-32kbitrate-mono-mp3",
            OutputFormat::audio_16khz_64kbitrate_mono_mp3 => "audio-16khz-64kbitrate-mono-mp3",
            OutputFormat::audio_16khz_128kbitrate_mono_mp3 => "audio-16khz-128kbitrate-mono-mp3",
            OutputFormat::audio_24khz_16bit_24kbps_mono_opus => "audio-24khz-16bit-24kbps-mono-opus",
            OutputFormat::audio_24khz_16bit_48kbps_mono_opus => "audio-24khz-16bit-48kbps-mono-opus",
            OutputFormat::audio_24khz_48kbitrate_mono_mp3 => "audio-24khz-48kbitrate-mono-mp3",
            OutputFormat::audio_24khz_96kbitrate_mono_mp3 => "audio-24khz-96kbitrate-mono-mp3",
            OutputFormat::audio_24khz_160kbitrate_mono_mp3 => "audio-24khz-160kbitrate-mono-mp3",
            OutputFormat::audio_48khz_96kbitrate_mono_mp3 => "audio-48khz-96kbitrate-mono-mp3",
            OutputFormat::audio_48khz_192kbitrate_mono_mp3 => "audio-48khz-192kbitrate-mono-mp3",
            OutputFormat::ogg_16khz_16bit_mono_opus => "ogg-16khz-16bit-mono-opus",
            OutputFormat::ogg_24khz_16bit_mono_opus => "ogg-24khz-16bit-mono-opus",
            OutputFormat::ogg_48khz_16bit_mono_opus => "ogg-48khz-16bit-mono-opus",
            OutputFormat::raw_8khz_8bit_mono_alaw => "raw-8khz-8bit-mono-alaw",
            OutputFormat::raw_8khz_8bit_mono_mulaw => "raw-8khz-8bit-mono-mulaw",
            OutputFormat::raw_8khz_16bit_mono_pcm => "raw-8khz-16bit-mono-pcm",
            OutputFormat::raw_16khz_16bit_mono_pcm => "raw-16khz-16bit-mono-pcm",
            OutputFormat::raw_16khz_16bit_mono_truesilk => "raw-16khz-16bit-mono-truesilk",
            OutputFormat::raw_22050hz_16bit_mono_pcm => "raw-22050hz-16bit-mono-pcm",
            OutputFormat::raw_24khz_16bit_mono_pcm => "raw-24khz-16bit-mono-pcm",
            OutputFormat::raw_24khz_16bit_mono_truesilk => "raw-24khz-16bit-mono-truesilk",
            OutputFormat::raw_44100hz_16bit_mono_pcm => "raw-44100hz-16bit-mono-pcm",
            OutputFormat::raw_48khz_16bit_mono_pcm => "raw-48khz-16bit-mono-pcm",
            OutputFormat::webm_16khz_16bit_mono_opus => "webm-16khz-16bit-mono-opus",
            OutputFormat::webm_24khz_16bit_24kbps_mono_opus => "webm-24khz-16bit-24kbps-mono-opus",
            OutputFormat::webm_24khz_16bit_mono_opus => "webm-24khz-16bit-mono-opus",
        }
    }
}
