use crate::prelude::*;
// @file:JvmName("BookKt")

// package io.legado.app.data.entities

// import mu.KotlinLogging

// val logger = KotlinLogging.logger {}
// fix: E0015——static 不能调用非 const 关联函数；KotlinLoggingLogger 为单元结构体，直接构造
pub static LOGGER: KotlinLoggingLogger = KotlinLoggingLogger;
