use crate::prelude::*;
// package io.legado.app.model
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.model.webBook.WebBook
// import io.legado.app.utils.isAbsUrl
// import io.legado.app.utils.HtmlFormatter
// import io.legado.app.utils.GSON
// import mu.KotlinLogging
// import java.text.SimpleDateFormat
// import java.util.Locale
// import java.util.Date

// private val logger = KotlinLogging.logger {}

// fix: WebBook.debug_logger 为 Box<dyn DebugLog>（'static），而各 debug_* 方法仅有 &mut self 无法移入 Box；
//      log_msg 改 Rc 使 Debugger 可 Clone，站点赋值用 Box::new(self.clone())（克隆体内 start_time 在赋值时点已就绪）
#[derive(Clone)]
pub struct Debugger {
    log_msg: Rc<dyn Fn(&str)>,
    start_time: i64,
}

impl DebugLog for Debugger {
    // Kotlin override fun log(sourceUrl: String?, msg: String?, isHtml: Boolean)
    fn log(
        &self,
        source_url: Option<&str>,
        msg: Option<&str>,
        is_html: bool,
    ) {
        if source_url.is_none() || msg.is_none() { return; }
        // logger.info("sourceUrl: {}, msg: {}", sourceUrl, msg)
        let mut print_msg = msg.unwrap().to_string();

        if is_html {
            print_msg = HtmlFormatter::new().format(Some(&print_msg));
        }
        let time = debug_time_format(System::current_time_millis() - self.start_time);
        print_msg = format!("{} {}", time, print_msg);
        (self.log_msg)(&print_msg);
    }
}

impl Debugger {
    // Kotlin class Debugger(val logMsg: (String) -> Unit) : DebugLog
    pub fn new<F: Fn(&str) + 'static>(log_msg: F) -> Self {
        // private val debugTimeFormat = SimpleDateFormat("[mm:ss.SSS]", Locale.getDefault())
        // private var startTime: Long = System.currentTimeMillis()
        Debugger {
            log_msg: Rc::new(log_msg),
            start_time: System::current_time_millis(),
        }
    }

    // Kotlin fun log(sourceUrl: String?, msg: String?)（DebugLog trait 的 log(message) 因 trait 内 E0428 被丢弃，消息版改固有方法 log_message）
    fn log(
        &self,
        source_url: Option<&str>,
        msg: Option<&str>,
    ) {
        DebugLog::log(self, source_url, msg, false);
    }

    // Kotlin override fun log(message: String)
    fn log_message(&self, message: &str) {
        // val time = debugTimeFormat.format(Date(System.currentTimeMillis() - startTime))
        let time = debug_time_format(System::current_time_millis() - self.start_time);
        (self.log_msg)(&format!("{} {}", time, message));
    }

    pub async fn start_debug(&mut self, web_book: &mut WebBook, key: &str) -> Result<(), StubError> {
        let book_source = &web_book.book_source;
        web_book.debug_logger = Some(Box::new(self.clone()));
        self.start_time = System::current_time_millis();
        if key.is_abs_url() {
            let mut book = Book::default();
            book.origin = book_source.book_source_url.clone();
            book.book_url = key.to_string();
            self.log(Some(&book_source.book_source_url), Some(&format!("⇒开始访问详情页:{}", key)));
            self.info_debug(web_book, book).await
        } else if key.contains("::") {
            let url = key.substring_after("::");
            self.log(Some(&book_source.book_source_url), Some(&format!("⇒开始访问发现页:{}", url)));
            self.explore_debug(web_book, &url).await
        } else if key.starts_with("++") {
            let url = key.substring(2);
            let mut book = Book::default();
            book.origin = book_source.book_source_url.clone();
            book.toc_url = url.clone();
            self.log(Some(&book_source.book_source_url), Some(&format!("⇒开始访目录页:{}", url)));
            self.toc_debug(web_book, book).await
        } else if key.starts_with("--") {
            let url = key.substring(2);
            let mut book = Book::default();
            book.origin = book_source.book_source_url.clone();
            self.log(Some(&book_source.book_source_url), Some(&format!("⇒开始访正文页:{}", url)));
            let mut chapter = BookChapter::default();
            chapter.title = "调试".to_string();
            chapter.url = url.clone();
            self.content_debug(web_book, book, &chapter, None).await;
            Ok(())
        } else {
            self.log(Some(&book_source.book_source_url), Some(&format!("⇒开始搜索关键字:{}", key)));
            self.search_debug(web_book, key).await
        }
    }

    async fn explore_debug(&mut self, web_book: &mut WebBook, url: &str) -> Result<(), StubError> {
        web_book.debug_logger = Some(Box::new(self.clone()));
        self.log_message("︾开始解析发现页");
        // fix: E0502 run_catching 闭包借用 web_book 存于 result 直至析构，改临时表达式使借用随语句结束
        let mut explore_books = match run_catching(|| web_book.explore_book(url, Some(1))) {
            Ok(f) => f.await,
            Err(e) => {
                self.log(Some(&web_book.source_url()), Some(&format!("Error: {}", e.localized_message())));
                return Err(e);
            }
        };
        if !explore_books.is_empty() {
            self.log_message("┌发现结果列表");
            // fix: Kotlin "└" + GSON.toJson(exploreBooks)（实体无 serde derive，占位输出）
            self.log_message(&format!("└{}", gson_to_json_placeholder(&explore_books)));
            self.log(Some(&web_book.source_url()), Some("︽发现页解析完成\n\n"));
            self.info_debug(web_book, explore_books[0].to_book()).await
        } else {
            self.log(Some(&web_book.source_url()), Some("︽未获取到书籍"));
            Ok(())
        }
    }

    async fn search_debug(&mut self, web_book: &mut WebBook, key: &str) -> Result<(), StubError> {
        web_book.debug_logger = Some(Box::new(self.clone()));
        self.log_message("︾开始解析搜索页");
        // fix: E0502 同 explore_debug
        let mut search_books = match run_catching(|| web_book.search_book(key, Some(1))) {
            Ok(f) => f.await,
            Err(e) => {
                self.log(Some(&web_book.source_url()), Some(&format!("Error: {}", e.localized_message())));
                return Err(e);
            }
        };
        if !search_books.is_empty() {
            self.log_message("┌搜索结果列表");
            // fix: Kotlin "└" + GSON.toJson(searchBooks)（实体无 serde derive，占位输出）
            self.log_message(&format!("└{}", gson_to_json_placeholder(&search_books)));
            self.log(Some(&web_book.source_url()), Some("︽搜索页解析完成\n\n"));
            self.info_debug(web_book, search_books[0].to_book()).await
        } else {
            self.log(Some(&web_book.source_url()), Some("︽未获取到书籍"));
            Ok(())
        }
    }

    async fn info_debug(&mut self, web_book: &mut WebBook, book: Book) -> Result<(), StubError> {
        web_book.debug_logger = Some(Box::new(self.clone()));
        self.log_message("︾开始解析详情页");
        let mut book = book;
        // fix: E0502 同 explore_debug
        let info = match run_catching(|| web_book.get_book_info(&mut book, false)) {
            Ok(f) => f.await,
            Err(e) => {
                self.log(Some(&web_book.source_url()), Some(&format!("Error: {}", e.localized_message())));
                return Err(e);
            }
        };
        self.log_message("┌书籍详情");
        // fix: Kotlin "└" + GSON.toJson(it)（Book 无 serde derive，占位输出）
        self.log_message(&format!("└{}", gson_to_json_placeholder(&info)));
        self.log(Some(&web_book.source_url()), Some("︽详情页解析完成\n\n"));
        self.toc_debug(web_book, info).await
    }

    async fn toc_debug(&mut self, web_book: &mut WebBook, book: Book) -> Result<(), StubError> {
        web_book.debug_logger = Some(Box::new(self.clone()));
        self.log_message("︾开始解析目录页");
        let mut book = book;
        // fix: E0502/E0505 同 explore_debug（book 由闭包可变借用，await 后释放方可 move 出）
        let chapter_list = match run_catching(|| web_book.get_chapter_list(&mut book)) {
            Ok(f) => f.await,
            Err(e) => {
                self.log(Some(&web_book.source_url()), Some(&format!("Error: {}", e.localized_message())));
                return Err(e);
            }
        };
        if !chapter_list.is_empty() {
            self.log_message("┌目录列表");
            // fix: Kotlin "└" + GSON.toJson(it)（BookChapter 无 serde derive，占位输出）
            self.log_message(&format!("└{}", gson_to_json_placeholder(&chapter_list)));
            self.log(Some(&web_book.source_url()), Some("︽目录页解析完成\n\n"));
            let next_chapter_url = if chapter_list.len() > 1 { Some(chapter_list[1].url.as_str()) } else { None };
            self.content_debug(web_book, book, &chapter_list[0], next_chapter_url).await;
            Ok(())
        } else {
            self.log(Some(&web_book.source_url()), Some("︽目录列表为空"));
            Ok(())
        }
    }

    async fn content_debug(
        &mut self,
        web_book: &mut WebBook,
        book: Book,
        book_chapter: &BookChapter,
        next_chapter_url: Option<&str>,
    ) {
        web_book.debug_logger = Some(Box::new(self.clone()));
        self.log(Some(&web_book.source_url()), Some("︾开始解析正文页"));
        let mut book = book;
        let result = run_catching(|| web_book.get_book_content(&mut book, book_chapter, next_chapter_url));
        match result {
            Ok(f) => {
                let content = f.await;
                self.log_message("┌正文内容");
                // fix: Kotlin "└" + GSON.toJson(it)（正文为 String，占位输出）
                self.log_message(&format!("└{}", gson_to_json_placeholder(&content)));
                self.log(Some(&web_book.source_url()), Some("︽正文页解析完成"));
            }
            Err(e) => {
                self.log(Some(&web_book.source_url()), Some(&format!("Error: {}", e.localized_message())));
            }
        }
    }
}
