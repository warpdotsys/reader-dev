pub struct Action;

impl Action {
    pub const play: &'static str = "play";
    pub const stop: &'static str = "stop";
    pub const resume: &'static str = "resume";
    pub const pause: &'static str = "pause";
    pub const addTimer: &'static str = "addTimer";
    pub const setTimer: &'static str = "setTimer";
    pub const prevParagraph: &'static str = "prevParagraph";
    pub const nextParagraph: &'static str = "nextParagraph";
    pub const upTtsSpeechRate: &'static str = "upTtsSpeechRate";
    pub const adjustProgress: &'static str = "adjustProgress";
    pub const prev: &'static str = "prev";
    pub const next: &'static str = "next";
    pub const moveTo: &'static str = "moveTo";
    pub const init: &'static str = "init";
}
