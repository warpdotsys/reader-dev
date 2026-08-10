// package io.legado.app.model
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import mu.KotlinLogging
// import okhttp3.logging.HttpLoggingInterceptor

// private val logger = KotlinLogging.logger {}

// interface DebugLog : HttpLoggingInterceptor.Logger

trait DebugLog {
    fn log(
        &self,
        source_url: Option<&str>,
        msg: Option<&str>,
        is_html: bool,
    ) {
        // logger.info("sourceUrl: {}, msg: {}", sourceUrl, msg)
    }

    fn log(&self, message: &str) {
        // logger.debug(message)
    }
}
