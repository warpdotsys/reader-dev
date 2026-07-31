package com.htmake.reader.lib.tts.constant;

public enum TtsStyleEnum {

    advertisement_upbeat("advertisement_upbeat"),
    affectionate("affectionate"),
    angry("angry"),
    assistant("assistant"),
    calm("calm"),
    chat("chat"),
    cheerful("cheerful"),
    customerservice("customerservice"),
    depressed("depressed"),
    disgruntled("disgruntled"),
    documentary_narration("documentary-narration"),
    embarrassed("embarrassed"),
    empathetic("empathetic"),
    envious("envious"),
    excited("excited"),
    fearful("fearful"),
    friendly("friendly"),
    gentle("gentle"),
    hopeful("hopeful"),
    lyrical("lyrical"),
    narration_professional("narration-professional"),
    narration_relaxed("narration-relaxed"),
    newscast("newscast"),
    newscast_casual("newscast-casual"),
    newscast_formal("newscast-formal"),
    poetry_reading("poetry-reading"),
    sad("sad"),
    serious("serious"),
    shouting("shouting"),
    sports_commentary("sports_commentary"),
    sports_commentary_excited("sports_commentary_excited"),
    whispering("whispering"),
    terrified("terrified"),
    unfriendly("unfriendly");

    private final String value;

    TtsStyleEnum(String value) {
        this.value = value;
    }

    public String getValue() {
        return value;
    }
}
