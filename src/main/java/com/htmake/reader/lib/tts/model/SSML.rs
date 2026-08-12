use crate::prelude::*;
// package com.htmake.reader.lib.tts.model;

// import com.htmake.reader.lib.tts.constant.OutputFormat;
// import com.htmake.reader.lib.tts.constant.TtsStyleEnum;
// import com.htmake.reader.lib.tts.constant.VoiceEnum;
// import com.htmake.reader.lib.tts.util.Tools;

// import java.io.Serializable;
// import java.util.Optional;

// public class SSML implements Serializable {
pub struct SSML {
    // private String synthesisText;
    // private VoiceEnum voice;
    // private String rate;
    // private String pitch;
    // private String volume;
    // private TtsStyleEnum style;
    // private OutputFormat outputFormat;
    pub synthesis_text: String,
    pub voice: Option<VoiceEnum>,
    pub rate: Option<String>,
    pub pitch: Option<String>,
    pub volume: Option<String>,
    pub style: Option<TtsStyleEnum>,
    pub output_format: Option<OutputFormat>,
}

impl SSML {
    // public static String SSML_PATTERN = "X-RequestId:%s\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:%sZ\r\nPath:ssml\r\n\r\n<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xmlns:mstts='https://www.w3.org/2001/mstts' xml:lang='%s'>\r\n<voice name='%s'>\r\n%s<prosody pitch='%s' rate='%s' volume='%s'>%s</prosody>%s</voice></speak>";
    // fix: associated const moved from struct body into impl block (Rust 不允许结构体中定义关联常量)
    pub const SSML_PATTERN: &'static str = "X-RequestId:%s\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:%sZ\r\nPath:ssml\r\n\r\n<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xmlns:mstts='https://www.w3.org/2001/mstts' xml:lang='%s'>\r\n<voice name='%s'>\r\n%s<prosody pitch='%s' rate='%s' volume='%s'>%s</prosody>%s</voice></speak>";

    // private SSML(String synthesisText, VoiceEnum voice, String rate, String pitch, String volume, TtsStyleEnum style, OutputFormat outputFormat) {
    //     this.synthesisText = synthesisText;
    //     this.voice = voice;
    //     this.rate = rate;
    //     this.pitch = pitch;
    //     this.volume = volume;
    //     this.style = style;
    //     this.outputFormat = outputFormat;
    // }
    fn new(synthesis_text: String, voice: Option<VoiceEnum>, rate: Option<String>, pitch: Option<String>, volume: Option<String>, style: Option<TtsStyleEnum>, output_format: Option<OutputFormat>) -> SSML {
        SSML {
            synthesis_text,
            voice,
            rate,
            pitch,
            volume,
            style,
            output_format,
        }
    }

    // public static SSMLBuilder builder() {
    //     return new SSMLBuilder();
    // }
    pub fn builder() -> SSMLBuilder {
        return SSMLBuilder::new();
    }

    // public String getSynthesisText() {
    //     return synthesisText;
    // }
    pub fn get_synthesis_text(&self) -> String {
        return self.synthesis_text.clone();
    }

    // public void setSynthesisText(String synthesisText) {
    //     this.synthesisText = synthesisText;
    // }
    pub fn set_synthesis_text(&mut self, synthesis_text: String) {
        self.synthesis_text = synthesis_text;
    }

    // public VoiceEnum getVoice() {
    //     return voice;
    // }
    pub fn get_voice(&self) -> Option<VoiceEnum> {
        return self.voice;
    }

    // public void setVoice(VoiceEnum voice) {
    //     this.voice = voice;
    // }
    pub fn set_voice(&mut self, voice: VoiceEnum) {
        self.voice = Some(voice);
    }

    // public String getRate() {
    //     return rate;
    // }
    pub fn get_rate(&self) -> Option<String> {
        return self.rate.clone();
    }

    // public void setRate(String rate) {
    //     this.rate = rate;
    // }
    pub fn set_rate(&mut self, rate: String) {
        self.rate = Some(rate);
    }

    // public String getPitch() {
    //     return pitch;
    // }
    pub fn get_pitch(&self) -> Option<String> {
        return self.pitch.clone();
    }

    // public void setPitch(String pitch) {
    //     this.pitch = pitch;
    // }
    pub fn set_pitch(&mut self, pitch: String) {
        self.pitch = Some(pitch);
    }

    // public String getVolume() {
    //     return volume;
    // }
    pub fn get_volume(&self) -> Option<String> {
        return self.volume.clone();
    }

    // public void setVolume(String volume) {
    //     this.volume = volume;
    // }
    pub fn set_volume(&mut self, volume: String) {
        self.volume = Some(volume);
    }

    // public TtsStyleEnum getStyle() {
    //     return style;
    // }
    pub fn get_style(&self) -> Option<TtsStyleEnum> {
        return self.style;
    }

    // public void setStyle(TtsStyleEnum style) {
    //     this.style = style;
    // }
    pub fn set_style(&mut self, style: TtsStyleEnum) {
        self.style = Some(style);
    }

    // public OutputFormat getOutputFormat() {
    //     return outputFormat;
    // }
    pub fn get_output_format(&self) -> Option<OutputFormat> {
        return self.output_format;
    }

    // public void setOutputFormat(OutputFormat outputFormat) {
    //     this.outputFormat = outputFormat;
    // }
    pub fn set_output_format(&mut self, output_format: OutputFormat) {
        self.output_format = Some(output_format);
    }

    // @Override
    // public String toString() {
    //     return String.format(SSML_PATTERN,
    //             Tools.getRandomId(),
    //             Tools.date(),
    //             Optional.ofNullable(voice).orElse(VoiceEnum.zh_CN_XiaoxiaoNeural).getLocale(),
    //             Optional.ofNullable(voice).orElse(VoiceEnum.zh_CN_XiaoxiaoNeural).getShortName(),
    //             Optional.ofNullable(style).map(s -> String.format("<mstts:express-as style='%s'>\r\n", s.getValue())).orElse(""),
    //             Optional.ofNullable(pitch).orElse("+0Hz"),
    //             Optional.ofNullable(rate).orElse("+0%"),
    //             Optional.ofNullable(volume).orElse("+0%"),
    //             synthesisText,
    //             Optional.ofNullable(style).map(s -> "</mstts:express-as>").orElse("")
    //     );
    // }
    pub fn to_string(&self) -> String {
        let voice = self.voice.unwrap_or(VoiceEnum::zh_CN_XiaoxiaoNeural);
        let style_str = self
            .style
            .map(|s| format!("<mstts:express-as style='{}'>\r\n", s.get_value()))
            .unwrap_or_default();
        let style_end = self.style.map(|_| "</mstts:express-as>".to_string()).unwrap_or_default();
        // fix: format! 首参必须是字符串字面量，改以 Java String.format 的 %s 占位符逐次替换
        let args = [
            Tools::get_random_id(),
            Tools::date(),
            voice.get_locale().to_string(),
            voice.get_short_name().to_string(),
            style_str,
            self.pitch.clone().unwrap_or_else(|| "+0Hz".to_string()),
            self.rate.clone().unwrap_or_else(|| "+0%".to_string()),
            self.volume.clone().unwrap_or_else(|| "+0%".to_string()),
            self.synthesis_text.clone(),
            style_end,
        ];
        let mut result = Self::SSML_PATTERN.to_string();
        for arg in args {
            result = result.replacen("%s", &arg, 1);
        }
        return result;
    }
}

// public static class SSMLBuilder {
pub struct SSMLBuilder {
    // private String synthesisText;
    // private VoiceEnum voice;
    // private String rate;
    // private String pitch;
    // private String volume;
    // private TtsStyleEnum style;
    // private OutputFormat outputFormat;
    pub synthesis_text: Option<String>,
    pub voice: Option<VoiceEnum>,
    pub rate: Option<String>,
    pub pitch: Option<String>,
    pub volume: Option<String>,
    pub style: Option<TtsStyleEnum>,
    pub output_format: Option<OutputFormat>,
}

impl SSMLBuilder {
    // public SSMLBuilder() {
    // }
    pub fn new() -> SSMLBuilder {
        SSMLBuilder {
            synthesis_text: None,
            voice: None,
            rate: None,
            pitch: None,
            volume: None,
            style: None,
            output_format: None,
        }
    }

    // public SSMLBuilder synthesisText(String synthesisText) {
    //     this.synthesisText = synthesisText;
    //     return this;
    // }
    pub fn synthesis_text(&mut self, synthesis_text: String) -> &mut SSMLBuilder {
        self.synthesis_text = Some(synthesis_text);
        return self;
    }

    // public SSMLBuilder voice(VoiceEnum voice) {
    //     this.voice = voice;
    //     return this;
    // }
    pub fn voice(&mut self, voice: VoiceEnum) -> &mut SSMLBuilder {
        self.voice = Some(voice);
        return self;
    }

    // public SSMLBuilder rate(String rate) {
    //     this.rate = rate;
    //     return this;
    // }
    pub fn rate(&mut self, rate: String) -> &mut SSMLBuilder {
        self.rate = Some(rate);
        return self;
    }

    // public SSMLBuilder pitch(String pitch) {
    //     this.pitch = pitch;
    //     return this;
    // }
    pub fn pitch(&mut self, pitch: String) -> &mut SSMLBuilder {
        self.pitch = Some(pitch);
        return self;
    }

    // public SSMLBuilder volume(String volume) {
    //     this.volume = volume;
    //     return this;
    // }
    pub fn volume(&mut self, volume: String) -> &mut SSMLBuilder {
        self.volume = Some(volume);
        return self;
    }

    // public SSMLBuilder style(TtsStyleEnum style) {
    //     this.style = style;
    //     return this;
    // }
    pub fn style(&mut self, style: TtsStyleEnum) -> &mut SSMLBuilder {
        self.style = Some(style);
        return self;
    }

    // public SSMLBuilder outputFormat(OutputFormat outputFormat) {
    //     this.outputFormat = outputFormat;
    //     return this;
    // }
    pub fn output_format(&mut self, output_format: OutputFormat) -> &mut SSMLBuilder {
        self.output_format = Some(output_format);
        return self;
    }

    // public SSML build() {
    //     return new SSML(synthesisText, voice, rate, pitch, volume, style, outputFormat);
    // }
    pub fn build(&mut self) -> SSML {
        return SSML::new(
            self.synthesis_text.clone().unwrap_or_default(),
            self.voice,
            self.rate.clone(),
            self.pitch.clone(),
            self.volume.clone(),
            self.style,
            self.output_format,
        );
    }
}
