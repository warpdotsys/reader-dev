// package io.legado.app.model.webBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.BookSource
// import io.legado.app.data.entities.SearchBook
// import io.legado.app.exception.NoStackTraceException
// import io.legado.app.help.http.StrResponse
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.model.webBook.BookChapterList
// import io.legado.app.model.webBook.BookContent
// import io.legado.app.model.webBook.BookInfo
// import io.legado.app.model.webBook.BookList
// import io.legado.app.model.Debug
// import io.legado.app.model.DebugLog
// import mu.KotlinLogging
// import kotlinx.coroutines.CoroutineScope
// import kotlinx.coroutines.coroutineScope
// import kotlinx.coroutines.Dispatchers
// import kotlinx.coroutines.withContext
//
// private val logger = KotlinLogging.logger {}

pub struct WebBook {
    pub book_source: BookSource,
    pub debug_log: bool,
    pub debug_logger: Option<DebugLog>,
    pub user_name_space: Option<String>,
}

impl WebBook {

    pub fn new(
        book_source: BookSource,
        debug_log: bool,
        debug_logger: Option<DebugLog>,
        user_name_space: Option<String>
    ) -> Self {
        WebBook { book_source, debug_log, debug_logger, user_name_space }
    }

    // constructor(
    //     bookSourceString: String,
    //     debugLog: Boolean = true,
    //     debugLogger: DebugLog? = null,
    //     userNameSpace: String? = null
    // ) : this(BookSource.fromJson(bookSourceString).getOrNull() ?: BookSource(), debugLog, debugLogger, userNameSpace)
    pub fn from_json_string(
        book_source_string: &str,
        debug_log: bool,
        debug_logger: Option<DebugLog>,
        user_name_space: Option<String>
    ) -> Self {
        let book_source = BookSource::from_json(book_source_string).get_or_null().unwrap_or(BookSource::new());
        Self::new(book_source, debug_log, debug_logger, user_name_space)
    }

    pub fn source_url(&self) -> String {
        return self.book_source.book_source_url.clone();
    }

    pub fn user_ns(&self) -> String {
        return self.user_name_space.clone().unwrap_or("unknow".to_string());
    }

    fn prepare_source(&self) {
        self.book_source.set_user_name_space(&self.user_ns());
        self.book_source.set_logger(self.debugger());
    }

    pub fn debugger(&self) -> Option<&DebugLog> {
        if self.debug_logger.is_some() {
            return self.debug_logger.as_ref();
        }
        if self.debug_log {
            return Some(&Debug);
        }
        return None;
    }

    /**
     * 搜索
     */
    pub async fn search_book(
        &self,
        key: &str,
        page: Option<i32>
    ) -> Vec<SearchBook> {
        let variable_book = SearchBook::new().also(|it| it.set_user_name_space(&self.user_ns()));
        self.prepare_source();
        if let Some(search_url) = self.book_source.search_url.clone() {
            let analyze_url = AnalyzeUrl::new(
                &search_url,
                key,
                page,
                &self.book_source.book_source_url,
                &self.book_source,
                &variable_book,
                self.book_source.get_header_map(true),
                self.debugger()
            );
            let mut res = analyze_url.get_str_response_await();
            //检测书源是否已登录
            if let Some(check_js) = self.book_source.login_check_js.clone() {
                if check_js.is_not_blank() {
                    res = analyze_url.eval_js(&check_js, &res) as StrResponse;
                }
            }
            return BookList::analyze_book_list(
                res.body.as_deref(),
                &self.book_source,
                &analyze_url,
                &res.url,
                &variable_book,
                true,
                self.debugger()
            ).await.map(|mut it| {
                it.toc_html = String::new();
                it.info_html = String::new();
                it
            });
        }
        return Vec::new();
    }

    pub async fn precise_search(&self, name: &str, author: &str) -> Result<Book> {
        // runCatching {
        let book = self.search_book(name, Some(1)).await
            .into_iter()
            .find(|it| it.name == name && it.author == author)
            .map(|it| it.to_book())
            .unwrap_or_else(|| panic!("未搜索到 {}({}) 书籍", name, author));
        if book.toc_url.is_blank() {
            self.get_book_info(book, false).await;
        }
        Ok(book)
    }

    /**
     * 发现
     */
    pub async fn explore_book(
        &self,
        url: &str,
        page: Option<i32>
    ) -> Vec<SearchBook> {
        let variable_book = SearchBook::new().also(|it| it.set_user_name_space(&self.user_ns()));
        self.prepare_source();
        let analyze_url = AnalyzeUrl::new(
            url,
            None,
            page,
            &self.book_source.book_source_url,
            &self.book_source,
            &variable_book,
            self.book_source.get_header_map(true),
            self.debugger()
        );
        let mut res = analyze_url.get_str_response_await();
        //检测书源是否已登录
        if let Some(check_js) = self.book_source.login_check_js.clone() {
            if check_js.is_not_blank() {
                res = analyze_url.eval_js(&check_js, &res) as StrResponse;
            }
        }
        return BookList::analyze_book_list(
            res.body.as_deref(),
            &self.book_source,
            &analyze_url,
            &res.url,
            &variable_book,
            false,
            self.debugger()
        ).await;
    }

    /**
     * 书籍信息
     */
    pub async fn get_book_info(&self, book: &mut Book, can_re_name: bool) -> Book {
        book.book_type = self.book_source.book_source_type.clone();
        book.set_user_name_space(&self.user_ns());
        self.prepare_source();
        if !book.info_html.is_empty() {
            BookInfo::analyze_book_info(
                book,
                Some(&book.info_html),
                &self.book_source,
                &book.book_url,
                &book.book_url,
                can_re_name,
                None
            ).await;
            return book.clone();
        } else {
            let analyze_url = AnalyzeUrl::new(
                &book.book_url,
                None,
                None,
                &self.book_source.book_source_url,
                &self.book_source,
                book,
                self.book_source.get_header_map(true),
                self.debugger()
            );
            let mut response = analyze_url.get_str_response_await();
            if let Some(check_js) = self.book_source.login_check_js.clone() {
                if check_js.is_not_blank() {
                    response = analyze_url.eval_js(&check_js, &response) as StrResponse;
                }
            }
            BookInfo::analyze_book_info(
                book,
                response.body.as_deref(),
                &self.book_source,
                &book.book_url,
                &response.url,
                can_re_name,
                None
            ).await;
            book.toc_html = None;
            return book.clone();
        }
    }

    /**
     * 书籍信息
     */
    pub async fn get_book_info_by_url(&self, book_url: &str, can_re_name: bool) -> Book {
        let mut book = Book::new();
        book.book_url = book_url.to_string();
        book.origin = self.book_source.book_source_url.clone();
        book.origin_name = self.book_source.book_source_name.clone();
        book.origin_order = self.book_source.custom_order;
        book.book_type = self.book_source.book_source_type.clone();
        book.set_user_name_space(&self.user_ns());
        return self.get_book_info(&mut book, can_re_name).await;
    }

    /**
     * 目录
     */
    pub async fn get_chapter_list(
        &self,
        book: &mut Book
    ) -> Vec<BookChapter> {
        book.book_type = self.book_source.book_source_type.clone();
        book.set_user_name_space(&self.user_ns());
        self.prepare_source();
        return if book.book_url == book.toc_url && !book.toc_html.is_empty() {
            BookChapterList::analyze_chapter_list(
                book,
                Some(book.toc_html.as_deref().unwrap()),
                &self.book_source,
                &book.toc_url,
                &book.toc_url,
                None
            ).await
        } else {
            let analyze_url = AnalyzeUrl::new(
                &book.toc_url,
                None,
                None,
                &book.book_url,
                &self.book_source,
                book,
                self.book_source.get_header_map(true),
                self.debugger()
            );
            let mut res = analyze_url.get_str_response_await();
            //检测书源是否已登录
            if let Some(check_js) = self.book_source.login_check_js.clone() {
                if check_js.is_not_blank() {
                    res = analyze_url.eval_js(&check_js, &res) as StrResponse;
                }
            }
            return BookChapterList::analyze_chapter_list(book, res.body.as_deref(), &self.book_source, &book.toc_url, &res.url, self.debugger()).await;
        };
    }

    /**
     * 章节内容
     */
    pub async fn get_book_content(
        &self,
        book: &mut Book,
        book_chapter: &BookChapter,
        // bookChapterUrl:String,
        next_chapter_url: Option<&str>
    ) -> String {
        book.set_user_name_space(&self.user_ns());
        self.prepare_source();
        if self.book_source.get_content_rule().content.is_empty() {
            if let Some(d) = self.debugger() {
                d.log(&self.book_source.book_source_url, &format!("⇒正文规则为空,使用章节链接: {}", book_chapter.url));
            }
            return book_chapter.url.clone();
        }
        if book_chapter.is_volume && book_chapter.url.starts_with(&book_chapter.title) {
            if let Some(d) = self.debugger() {
                d.log(&self.book_source.book_source_url, "⇒一级目录正文不解析规则");
            }
            return book_chapter.tag.clone().unwrap_or(String::new());
        }
        //        val body = if (book != null && bookChapter.url == book.bookUrl && !book.tocHtml.isNullOrEmpty()) {
        //            book.tocHtml
        //        } else {
        // logger.info("bookChapterUrl: {}", bookChapter.url, bookChapter.getAbsoluteURL())
        let analyze_url = AnalyzeUrl::new(
            &book_chapter.get_absolute_url(),
            None,
            None,
            &book.toc_url,
            &self.book_source,
            book,
            book_chapter,
            self.book_source.get_header_map(true),
            self.debugger()
        );
        let res = analyze_url.get_str_response_await(
            Some(&self.book_source.get_content_rule().web_js),
            Some(&self.book_source.get_content_rule().source_regex)
        );
        return BookContent::analyze_content(
            res.body.as_deref(),
            book,
            book_chapter,
            &self.book_source,
            &book_chapter.url,
            &res.url,
            next_chapter_url,
            self.debugger()
        ).await;
    }
}
