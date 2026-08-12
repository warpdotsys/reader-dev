use crate::prelude::*;
// package io.legado.app.exception

/**
 * 不记录错误堆栈的报错
 */
pub trait NoStackTraceException {
    fn msg(&self) -> &str;

    fn fill_in_stack_trace(&self) -> &Self {
        self
    }
}

pub struct NoStackTraceExceptionImpl {
    pub msg: String,
}

impl NoStackTraceExceptionImpl {
    pub fn new(msg: String) -> Self {
        NoStackTraceExceptionImpl { msg }
    }
}

impl NoStackTraceException for NoStackTraceExceptionImpl {
    fn msg(&self) -> &str {
        &self.msg
    }
}
