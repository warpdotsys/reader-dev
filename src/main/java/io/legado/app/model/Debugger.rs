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

struct Debugger {
    log_msg: Box<dyn Fn(&str)>,
}

impl Debugger {
    // private val debugTimeFormat = SimpleDateFormat("[mm:ss.SSS]", Locale.getDefault())
    // private var startTime: Long = System.currentTimeMillis()

    fn log(
        &self,
        source_url: Option<&str>,
        msg: Option<&str>,
    ) {
        self.log(source_url, msg, false);
    }

    fn log(&self, message: &str) {
        // val time = debugTimeFormat.format(Date(System.currentTimeMillis() - startTime))
        let time = debug_time_format(format(System.current_time_millis() - self.start_time));
        (self.log_msg)(format!("{} {}", time, message));
    }

    fn log(
        &self,
        source_url: Option<&str>,
        msg: Option<&str>,
        is_html: bool,
    ) {
        if source_url.is_none() || msg.is_none() { return; }
        // logger.info("sourceUrl: {}, msg: {}", sourceUrl, msg)
        let mut print_msg = msg.unwrap();

        if is_html {
            print_msg = HtmlFormatter::format(print_msg);
        }
        let time = debug_time_format(format(System.current_time_millis() - self.start_time));
        print_msg = format!("{} {}", time, print_msg);
        (self.log_msg)(print_msg);
    }

    async fn start_debug(&mut self, web_book: &mut WebBook, key: &str) {
        let book_source = web_book.book_source;
        web_book.debug_logger = self;
        self.start_time = System::current_time_millis();
        if key.is_abs_url() {
            let book = Book::new();
            book.origin = book_source.book_source_url;
            book.book_url = key;
            self.log(book_source.book_source_url, "⇒开始访问详情页:{}".replace("{}", key));
            self.info_debug(web_book, book).await;
        } else if key.contains("::") {
            let url = key.substring_after("::");
            self.log(book_source.book_source_url, "⇒开始访问发现页:{}".replace("{}", url));
            self.explore_debug(web_book, url).await;
        } else if key.starts_with("++") {
            let url = key.substring(2);
            let book = Book::new();
            book.origin = book_source.book_source_url;
            book.toc_url = url;
            self.log(book_source.book_source_url, "⇒开始访目录页:{}".replace("{}", url));
            self.toc_debug(web_book, book).await;
        } else if key.starts_with("--") {
            let url = key.substring(2);
            let book = Book::new();
            book.origin = book_source.book_source_url;
            self.log(book_source.book_source_url, "⇒开始访正文页:{}".replace("{}", url));
            let chapter = BookChapter::new();
            chapter.title = "调试";
            chapter.url = url;
            self.content_debug(web_book, book, chapter, None).await;
        } else {
            self.log(book_source.book_source_url, "⇒开始搜索关键字:{}".replace("{}", key));
            self.search_debug(web_book, key).await;
        }
    }

    async fn explore_debug(&mut self, web_book: &mut WebBook, url: &str) {
        web_book.debug_logger = self;
        self.log(None, Some("︾开始解析发现页"));
        let result = run_catching(|| web_book.explore_book(url, 1));
        match result {
            Ok(explore_books) => {
                if !explore_books.is_empty() {
                    self.log(None, Some("┌发现结果列表"));
                    self.log(None, Some("└" + GSON::to_json(&explore_books)));
                    self.log(web_book.source_url, "︽发现页解析完成\n\n");
                    self.info_debug(web_book, explore_books[0].to_book()).await;
                } else {
                    self.log(web_book.source_url, "︽未获取到书籍");
                }
            }
            Err(e) => {
                self.log(web_book.source_url, "Error: " + e.localized_message());
                return Err(e);
            }
        }
    }

    async fn search_debug(&mut self, web_book: &mut WebBook, key: &str) {
        web_book.debug_logger = self;
        self.log(None, Some("︾开始解析搜索页"));
        let result = run_catching(|| web_book.search_book(key, 1));
        match result {
            Ok(search_books) => {
                if !search_books.is_empty() {
                    self.log(None, Some("┌搜索结果列表"));
                    self.log(None, Some("└" + GSON::to_json(&search_books)));
                    self.log(web_book.source_url, "︽搜索页解析完成\n\n");
                    self.info_debug(web_book, search_books[0].to_book()).await;
                } else {
                    self.log(web_book.source_url, "︽未获取到书籍");
                }
            }
            Err(e) => {
                self.log(web_book.source_url, "Error: " + e.localized_message());
                return Err(e);
            }
        }
    }

    async fn info_debug(&mut self, web_book: &mut WebBook, book: &Book) {
        web_book.debug_logger = self;
        self.log(None, Some("︾开始解析详情页"));
        let result = run_catching(|| web_book.get_book_info(book.book_url));
        match result {
            Ok(info) => {
                self.log(None, Some("┌书籍详情"));
                self.log(None, Some("└" + GSON::to_json(&info)));
                self.log(web_book.source_url, "︽详情页解析完成\n\n");
                self.toc_debug(web_book, info).await;
            }
            Err(e) => {
                self.log(web_book.source_url, "Error: " + e.localized_message());
                return Err(e);
            }
        }
    }

    async fn toc_debug(&mut self, web_book: &mut WebBook, book: &Book) {
        web_book.debug_logger = self;
        self.log(None, Some("︾开始解析目录页"));
        let result = run_catching(|| web_book.get_chapter_list(book));
        match result {
            Ok(chapter_list) => {
                if chapter_list.is_some() {
                    let it = chapter_list.unwrap();
                    if !it.is_empty() {
                        self.log(None, Some("┌目录列表"));
                        self.log(None, Some("└" + GSON::to_json(&it)));
                        self.log(web_book.source_url, "︽目录页解析完成\n\n");
                        let next_chapter_url = if it.size() > 1 { Some(it[1].url) } else { None };
                        self.content_debug(web_book, book, it[0], next_chapter_url).await;
                    } else {
                        self.log(web_book.source_url, "︽目录列表为空");
                    }
                }
            }
            Err(e) => {
                self.log(web_book.source_url, "Error: " + e.localized_message());
                return Err(e);
            }
        }
    }

    async fn content_debug(
        &mut self,
        web_book: &mut WebBook,
        book: &Book,
        book_chapter: &BookChapter,
        next_chapter_url: Option<&str>,
    ) {
        web_book.debug_logger = self;
        self.log(web_book.source_url, "︾开始解析正文页");
        let result = run_catching(|| web_book.get_book_content(book, book_chapter, next_chapter_url));
        match result {
            Ok(content) => {
                self.log(None, Some("┌正文内容"));
                self.log(None, Some("└" + GSON::to_json(&content)));
                self.log(web_book.source_url, "︽正文页解析完成");
            }
            Err(e) => {
                self.log(web_book.source_url, "Error: " + e.localized_message());
            }
        }
    }
}
