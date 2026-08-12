use crate::prelude::*;
// package com.htmake.reader.lib.tts.model;

// import com.htmake.reader.lib.tts.constant.OutputFormat;
// import com.htmake.reader.lib.tts.util.Tools;

// import java.io.Serializable;
// import java.util.Optional;

// public class SpeechConfig implements Serializable {
pub struct SpeechConfig {
    // private OutputFormat outputFormat;
    pub output_format: Option<OutputFormat>,
}

impl SpeechConfig {
    // public static final String CONFIG_PATTERN = "X-Timestamp:%s\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{\"context\":{\"synthesis\":{\"audio\":{\"metadataoptions\":{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"},\"outputFormat\":\"%s\"}}}}";
    // fix: associated const moved from struct body into impl block (Rust 不允许结构体中定义关联常量)
    pub const CONFIG_PATTERN: &'static str = "X-Timestamp:%s\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{\"context\":{\"synthesis\":{\"audio\":{\"metadataoptions\":{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"},\"outputFormat\":\"%s\"}}}}";

    // private SpeechConfig(OutputFormat outputFormat) {
    //     this.outputFormat = Optional.ofNullable(outputFormat).orElse(OutputFormat.audio_24khz_48kbitrate_mono_mp3);
    // }
    fn new(output_format: Option<OutputFormat>) -> SpeechConfig {
        SpeechConfig {
            output_format: Some(output_format.unwrap_or(OutputFormat::audio_24khz_48kbitrate_mono_mp3)),
        }
    }

    // public static SpeechConfig of(OutputFormat outputFormat) {
    //     return new SpeechConfig(outputFormat);
    // }
    pub fn of(output_format: Option<OutputFormat>) -> SpeechConfig {
        return SpeechConfig::new(output_format);
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
    //     return String.format(CONFIG_PATTERN, Tools.date(), outputFormat.getValue());
    // }
    pub fn to_string(&self) -> String {
        // fix: format! 首参必须是字符串字面量，改以 Java String.format 的 %s 占位符逐次替换
        return Self::CONFIG_PATTERN
            .replacen("%s", &Tools::date(), 1)
            .replacen("%s", self.output_format.unwrap_or(OutputFormat::audio_24khz_48kbitrate_mono_mp3).get_value(), 1);
    }
}
