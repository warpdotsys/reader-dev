package com.htmake.reader.lib.tts.model;

import com.htmake.reader.lib.tts.constant.OutputFormat;
import com.htmake.reader.lib.tts.util.Tools;

import java.io.Serializable;
import java.util.Optional;

public class SpeechConfig implements Serializable {

    public static final String CONFIG_PATTERN = "X-Timestamp:%s\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{\"context\":{\"synthesis\":{\"audio\":{\"metadataoptions\":{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"},\"outputFormat\":\"%s\"}}}}";

    private OutputFormat outputFormat;

    private SpeechConfig(OutputFormat outputFormat) {
        this.outputFormat = Optional.ofNullable(outputFormat).orElse(OutputFormat.audio_24khz_48kbitrate_mono_mp3);
    }

    public static SpeechConfig of(OutputFormat outputFormat) {
        return new SpeechConfig(outputFormat);
    }

    public OutputFormat getOutputFormat() {
        return outputFormat;
    }

    public void setOutputFormat(OutputFormat outputFormat) {
        this.outputFormat = outputFormat;
    }

    @Override
    public String toString() {
        return String.format(CONFIG_PATTERN, Tools.date(), outputFormat.getValue());
    }
}
