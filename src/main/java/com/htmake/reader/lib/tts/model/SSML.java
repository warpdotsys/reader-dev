package com.htmake.reader.lib.tts.model;

import com.htmake.reader.lib.tts.constant.OutputFormat;
import com.htmake.reader.lib.tts.constant.TtsStyleEnum;
import com.htmake.reader.lib.tts.constant.VoiceEnum;
import com.htmake.reader.lib.tts.util.Tools;

import java.io.Serializable;
import java.util.Optional;

public class SSML implements Serializable {

    public static String SSML_PATTERN = "X-RequestId:%s\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:%sZ\r\nPath:ssml\r\n\r\n<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xmlns:mstts='https://www.w3.org/2001/mstts' xml:lang='%s'>\r\n<voice name='%s'>\r\n%s<prosody pitch='%s' rate='%s' volume='%s'>%s</prosody>%s</voice></speak>";

    private String synthesisText;
    private VoiceEnum voice;
    private String rate;
    private String pitch;
    private String volume;
    private TtsStyleEnum style;
    private OutputFormat outputFormat;

    private SSML(String synthesisText, VoiceEnum voice, String rate, String pitch, String volume, TtsStyleEnum style, OutputFormat outputFormat) {
        this.synthesisText = synthesisText;
        this.voice = voice;
        this.rate = rate;
        this.pitch = pitch;
        this.volume = volume;
        this.style = style;
        this.outputFormat = outputFormat;
    }

    public static SSMLBuilder builder() {
        return new SSMLBuilder();
    }

    public String getSynthesisText() {
        return synthesisText;
    }

    public void setSynthesisText(String synthesisText) {
        this.synthesisText = synthesisText;
    }

    public VoiceEnum getVoice() {
        return voice;
    }

    public void setVoice(VoiceEnum voice) {
        this.voice = voice;
    }

    public String getRate() {
        return rate;
    }

    public void setRate(String rate) {
        this.rate = rate;
    }

    public String getPitch() {
        return pitch;
    }

    public void setPitch(String pitch) {
        this.pitch = pitch;
    }

    public String getVolume() {
        return volume;
    }

    public void setVolume(String volume) {
        this.volume = volume;
    }

    public TtsStyleEnum getStyle() {
        return style;
    }

    public void setStyle(TtsStyleEnum style) {
        this.style = style;
    }

    public OutputFormat getOutputFormat() {
        return outputFormat;
    }

    public void setOutputFormat(OutputFormat outputFormat) {
        this.outputFormat = outputFormat;
    }

    @Override
    public String toString() {
        return String.format(SSML_PATTERN,
                Tools.getRandomId(),
                Tools.date(),
                Optional.ofNullable(voice).orElse(VoiceEnum.zh_CN_XiaoxiaoNeural).getLocale(),
                Optional.ofNullable(voice).orElse(VoiceEnum.zh_CN_XiaoxiaoNeural).getShortName(),
                Optional.ofNullable(style).map(s -> String.format("<mstts:express-as style='%s'>\r\n", s.getValue())).orElse(""),
                Optional.ofNullable(pitch).orElse("+0Hz"),
                Optional.ofNullable(rate).orElse("+0%"),
                Optional.ofNullable(volume).orElse("+0%"),
                synthesisText,
                Optional.ofNullable(style).map(s -> "</mstts:express-as>").orElse("")
        );
    }

    // Synthetic access constructor for builder
    SSML(String synthesisText, VoiceEnum voice, String rate, String pitch, String volume, TtsStyleEnum style, OutputFormat outputFormat, SSML dummy) {
        this(synthesisText, voice, rate, pitch, volume, style, outputFormat);
    }

    public static class SSMLBuilder {

        private String synthesisText;
        private VoiceEnum voice;
        private String rate;
        private String pitch;
        private String volume;
        private TtsStyleEnum style;
        private OutputFormat outputFormat;

        public SSMLBuilder() {
        }

        public SSMLBuilder synthesisText(String synthesisText) {
            this.synthesisText = synthesisText;
            return this;
        }

        public SSMLBuilder voice(VoiceEnum voice) {
            this.voice = voice;
            return this;
        }

        public SSMLBuilder rate(String rate) {
            this.rate = rate;
            return this;
        }

        public SSMLBuilder pitch(String pitch) {
            this.pitch = pitch;
            return this;
        }

        public SSMLBuilder volume(String volume) {
            this.volume = volume;
            return this;
        }

        public SSMLBuilder style(TtsStyleEnum style) {
            this.style = style;
            return this;
        }

        public SSMLBuilder outputFormat(OutputFormat outputFormat) {
            this.outputFormat = outputFormat;
            return this;
        }

        public SSML build() {
            return new SSML(synthesisText, voice, rate, pitch, volume, style, outputFormat);
        }
    }
}
