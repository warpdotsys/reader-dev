use crate::prelude::*;
// package io.legado.app.model
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import mu.KotlinLogging
// import okhttp3.logging.HttpLoggingInterceptor

// private val logger = KotlinLogging.logger {}

// interface DebugLog : HttpLoggingInterceptor.Logger

pub trait DebugLog {
    fn log(
        &self,
        source_url: Option<&str>,
        msg: Option<&str>,
        is_html: bool,
    ) {
        // logger.info("sourceUrl: {}, msg: {}", sourceUrl, msg)
    }

    // fix: Kotlin 重载 `log(message: String)`（HttpLoggingInterceptor.Logger 覆写）——Rust 不允许同名重载，
    //      消息版改固有方法 log_message（与 Debugger 转录约定一致）
    fn log_message(&self, message: &str) {
        // logger.debug(message)
    }

    // fix: trait object 克隆（AnalyzeRule::new 接收 &dyn DebugLog 但需 owned 存储；实现者覆写）
    fn clone_box(&self) -> Box<dyn DebugLog>;
}
