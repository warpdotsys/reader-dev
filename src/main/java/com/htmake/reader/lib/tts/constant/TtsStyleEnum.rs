// package com.htmake.reader.lib.tts.constant;

// public enum TtsStyleEnum {
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TtsStyleEnum {
    advertisement_upbeat,
    affectionate,
    angry,
    assistant,
    calm,
    chat,
    cheerful,
    customerservice,
    depressed,
    disgruntled,
    documentary_narration,
    embarrassed,
    empathetic,
    envious,
    excited,
    fearful,
    friendly,
    gentle,
    hopeful,
    lyrical,
    narration_professional,
    narration_relaxed,
    newscast,
    newscast_casual,
    newscast_formal,
    poetry_reading,
    sad,
    serious,
    shouting,
    sports_commentary,
    sports_commentary_excited,
    whispering,
    terrified,
    unfriendly,
}

impl TtsStyleEnum {
    // public String getValue() {
    //     return value;
    // }
    pub fn get_value(&self) -> &'static str {
        match self {
            TtsStyleEnum::advertisement_upbeat => "advertisement_upbeat",
            TtsStyleEnum::affectionate => "affectionate",
            TtsStyleEnum::angry => "angry",
            TtsStyleEnum::assistant => "assistant",
            TtsStyleEnum::calm => "calm",
            TtsStyleEnum::chat => "chat",
            TtsStyleEnum::cheerful => "cheerful",
            TtsStyleEnum::customerservice => "customerservice",
            TtsStyleEnum::depressed => "depressed",
            TtsStyleEnum::disgruntled => "disgruntled",
            TtsStyleEnum::documentary_narration => "documentary-narration",
            TtsStyleEnum::embarrassed => "embarrassed",
            TtsStyleEnum::empathetic => "empathetic",
            TtsStyleEnum::envious => "envious",
            TtsStyleEnum::excited => "excited",
            TtsStyleEnum::fearful => "fearful",
            TtsStyleEnum::friendly => "friendly",
            TtsStyleEnum::gentle => "gentle",
            TtsStyleEnum::hopeful => "hopeful",
            TtsStyleEnum::lyrical => "lyrical",
            TtsStyleEnum::narration_professional => "narration-professional",
            TtsStyleEnum::narration_relaxed => "narration-relaxed",
            TtsStyleEnum::newscast => "newscast",
            TtsStyleEnum::newscast_casual => "newscast-casual",
            TtsStyleEnum::newscast_formal => "newscast-formal",
            TtsStyleEnum::poetry_reading => "poetry-reading",
            TtsStyleEnum::sad => "sad",
            TtsStyleEnum::serious => "serious",
            TtsStyleEnum::shouting => "shouting",
            TtsStyleEnum::sports_commentary => "sports_commentary",
            TtsStyleEnum::sports_commentary_excited => "sports_commentary_excited",
            TtsStyleEnum::whispering => "whispering",
            TtsStyleEnum::terrified => "terrified",
            TtsStyleEnum::unfriendly => "unfriendly",
        }
    }
}
