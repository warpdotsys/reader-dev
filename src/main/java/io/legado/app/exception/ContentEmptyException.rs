// package io.legado.app.exception

/**
 * 内容为空
 */
pub struct ContentEmptyException {
    pub msg: String,
}

impl ContentEmptyException {
    pub fn new(msg: String) -> Self {
        ContentEmptyException { msg }
    }
}

impl NoStackTraceException for ContentEmptyException {
    fn msg(&self) -> &str {
        &self.msg
    }
}
