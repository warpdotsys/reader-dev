// package io.legado.app.exception

/**
 * 目录为空
 */
pub struct TocEmptyException {
    pub msg: String,
}

impl TocEmptyException {
    pub fn new(msg: String) -> Self {
        TocEmptyException { msg }
    }
}

impl NoStackTraceException for TocEmptyException {
    fn msg(&self) -> &str {
        &self.msg
    }
}
