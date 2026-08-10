// package io.legado.app.exception

pub struct RegexTimeoutException {
    pub msg: String,
}

impl RegexTimeoutException {
    pub fn new(msg: String) -> Self {
        RegexTimeoutException { msg }
    }
}

impl NoStackTraceException for RegexTimeoutException {
    fn msg(&self) -> &str {
        &self.msg
    }
}
