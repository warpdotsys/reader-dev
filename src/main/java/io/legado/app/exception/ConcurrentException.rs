use crate::prelude::*;
// @file:Suppress("unused")
//
// package io.legado.app.exception

/**
 * 并发限制
 */
pub struct ConcurrentException {
    pub msg: String,
    pub wait_time: i32,
}

impl ConcurrentException {
    pub fn new(msg: String, wait_time: i32) -> Self {
        ConcurrentException { msg, wait_time }
    }
}

impl NoStackTraceException for ConcurrentException {
    fn msg(&self) -> &str {
        &self.msg
    }
}
