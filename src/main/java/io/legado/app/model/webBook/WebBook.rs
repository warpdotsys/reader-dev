use crate::prelude::*;
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
    // fix: DebugLog 为 trait，Option<DebugLog> 需 Box<dyn DebugLog>（同 BookSource.rs）
    pub debug_logger: Option<Box<dyn DebugLog>>,
    pub user_name_space: Option<String>,
}

impl WebBook {

    pub fn new(
        book_source: BookSource,
        debug_log: bool,
        debug_logger: Option<Box<dyn DebugLog>>,
        user_name_space: Option<String>
    ) -> Self {
        WebBook { book_source, debug_log, debug_logger, user_name_space }
    }

    // constructor(
    //     bookSourceString: String,
    //     debugLog: Boolean = true,
    //     debugLogger: DebugLog? = None,
    //     userNameSpace: String? = None
    // ) : this(BookSource.fromJson(bookSourceString).getOrNull() ?: BookSource(), debugLog, debugLogger, userNameSpace)
    pub fn from_json_string(
        book_source_string: &str,
        debug_log: bool,
        debug_logger: Option<Box<dyn DebugLog>>,
        user_name_space: Option<String>
    ) -> Self {
        let book_source = BookSource::from_json(book_source_string.to_string()).get_or_null().unwrap_or(BookSource::default());
        Self::new(book_source, debug_log, debug_logger, user_name_space)
    }

    pub fn source_url(&self) -> String {
        return self.book_source.book_source_url.clone();
    }

    pub fn user_ns(&self) -> String {
        return self.user_name_space.clone().unwrap_or("unknow".to_string());
    }

    // fix: Kotlin prepareSource() 调 set_user_name_space/set_logger（均需 &mut self）且 BookSource 无 Clone——
    //      BookSource::clone 由 stubs 提供，这里同步 user_name_space/debug_log 后返回副本供 AnalyzeUrl::new 使用
    fn prepare_source(&self) -> BookSource {
        let mut source = self.book_source.clone();
        source.user_name_space = self.user_ns();
        source.debug_log = if self.debug_log { Some(Box::new(Debug)) } else { None };
        source
    }

    // fix: debug_logger 为 Box<dyn DebugLog> 无法从 &self 移出/克隆（AnalyzeUrl::new 需所有权），
    //      debug_log 开启时用 Debug 占位（同 BookSource.rs set_logger 占位约定）
    fn debug_log_box(&self) -> Option<Box<dyn DebugLog>> {
        if self.debug_log {
            Some(Box::new(Debug) as Box<dyn DebugLog>)
        } else {
            None
        }
    }

    pub fn debugger(&self) -> Option<&dyn DebugLog> {
        if self.debug_logger.is_some() {
            return self.debug_logger.as_deref();
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
        // fix: SearchBook 无 new()/also()（Kotlin also 占位仅实现于 i32），改为先 default 再 set_user_name_space
        let mut variable_book = SearchBook::default();
        variable_book.set_user_name_space(self.user_ns());
        let prepared_source = self.prepare_source();
        if let Some(search_url) = self.book_source.search_url.clone() {
            // fix: AnalyzeUrl::new 为全量参数构造（Kotlin 默认参数展开）；headerMapF 置 None 由 new 内按 source 回填 UA
            let mut analyze_url = AnalyzeUrl::new(
                search_url,
                Some(key.to_string()),
                page,
                None,
                None,
                self.book_source.book_source_url.clone(),
                Some(prepared_source),
                Some(Box::new(variable_book.clone())),
                None,
                None,
                self.debug_log_box()
            );
            let mut res = analyze_url.get_str_response_await(None, None, true).await;
            //检测书源是否已登录
            if let Some(check_js) = self.book_source.login_check_js.clone() {
                if check_js.is_not_blank() {
                    // fix: Kotlin `res = analyzeUrl.evalJS(checkJs, res) as StrResponse` —— eval_js 返回 Option<JsValue>
                    //      非 StrResponse，仅当 JS 返回字符串时重建 res（保留 url）
                    if let Some(js_value) = analyze_url.eval_js(check_js, res.body()) {
                        if let Some(js_str) = js_value.as_string() {
                            let res_url = res.url();
                            res = StrResponse::new_url(&res_url, Some(js_str));
                        }
                    }
                }
            }
            return BookList::analyze_book_list(
                res.body().map(|b| b.as_str()),
                &self.book_source,
                &analyze_url,
                &res.url(),
                &variable_book,
                true,
                self.debugger()
            ).await.into_iter().map(|mut it| {
                it.toc_html = Some(String::new());
                it.info_html = Some(String::new());
                it
            }).collect();
        }
        return Vec::new();
    }

    pub async fn precise_search(&self, name: &str, author: &str) -> Result<Book, StubError> {
        // runCatching {
        let mut book = self.search_book(name, Some(1)).await
            .into_iter()
            .find(|it| it.name == name && it.author == author)
            .map(|mut it| it.to_book())
            .unwrap_or_else(|| panic!("未搜索到 {}({}) 书籍", name, author));
        if book.toc_url.is_blank() {
            self.get_book_info(&mut book, false).await;
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
        let mut variable_book = SearchBook::default();
        variable_book.set_user_name_space(self.user_ns());
        let prepared_source = self.prepare_source();
        let mut analyze_url = AnalyzeUrl::new(
            url.to_string(),
            None,
            page,
            None,
            None,
            self.book_source.book_source_url.clone(),
            Some(prepared_source),
            Some(Box::new(variable_book.clone())),
            None,
            None,
            self.debug_log_box()
        );
        let mut res = analyze_url.get_str_response_await(None, None, true).await;
        //检测书源是否已登录
        if let Some(check_js) = self.book_source.login_check_js.clone() {
            if check_js.is_not_blank() {
                // fix: 同 search_book，eval_js 返回 Option<JsValue>，仅字符串结果重建 res
                if let Some(js_value) = analyze_url.eval_js(check_js, res.body()) {
                    if let Some(js_str) = js_value.as_string() {
                        let res_url = res.url();
                        res = StrResponse::new_url(&res_url, Some(js_str));
                    }
                }
            }
        }
        return BookList::analyze_book_list(
            res.body().map(|b| b.as_str()),
            &self.book_source,
            &analyze_url,
            &res.url(),
            &variable_book,
            false,
            self.debugger()
        ).await;
    }

    /**
     * 书籍信息
     */
    pub async fn get_book_info(&self, book: &mut Book, can_re_name: bool) -> Book {
        book.r#type = self.book_source.book_source_type;
        book.set_user_name_space(self.user_ns());
        let prepared_source = self.prepare_source();
        if !book.info_html.is_null_or_empty() {
            // fix: &book.book_url 与 book(&mut) 同参借用冲突，url 先克隆
            let book_url = book.book_url.clone();
            let info_html = book.info_html.clone();
            BookInfo::analyze_book_info(
                book,
                info_html.as_deref(),
                &self.book_source,
                &book_url,
                &book_url,
                can_re_name,
                None
            ).await;
            return book.clone();
        } else {
            // fix: Kotlin `AnalyzeUrl(book.bookUrl, baseUrl=bookSource.bookSourceUrl, ruleData=book)`——
            //      ruleData 传 Box<dyn RuleDataInterface> 需所有权，book 以 stubs 提供的 Clone 副本传递
            let mut analyze_url = AnalyzeUrl::new(
                book.book_url.clone(),
                None,
                None,
                None,
                None,
                self.book_source.book_source_url.clone(),
                Some(prepared_source),
                Some(Box::new(book.clone())),
                None,
                None,
                self.debug_log_box()
            );
            let mut response = analyze_url.get_str_response_await(None, None, true).await;
            if let Some(check_js) = self.book_source.login_check_js.clone() {
                if check_js.is_not_blank() {
                    if let Some(js_value) = analyze_url.eval_js(check_js, response.body()) {
                        if let Some(js_str) = js_value.as_string() {
                            let res_url = response.url();
                            response = StrResponse::new_url(&res_url, Some(js_str));
                        }
                    }
                }
            }
            let book_url = book.book_url.clone();
            BookInfo::analyze_book_info(
                book,
                response.body().map(|b| b.as_str()),
                &self.book_source,
                &book_url,
                &response.url(),
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
        let mut book = Book::default();
        book.book_url = book_url.to_string();
        book.origin = self.book_source.book_source_url.clone();
        book.origin_name = self.book_source.book_source_name.clone();
        book.origin_order = self.book_source.custom_order;
        book.r#type = self.book_source.book_source_type;
        book.set_user_name_space(self.user_ns());
        return self.get_book_info(&mut book, can_re_name).await;
    }

    /**
     * 目录
     */
    pub async fn get_chapter_list(
        &self,
        book: &mut Book
    ) -> Vec<BookChapter> {
        book.r#type = self.book_source.book_source_type;
        book.set_user_name_space(self.user_ns());
        let prepared_source = self.prepare_source();
        return if book.book_url == book.toc_url && !book.toc_html.is_null_or_empty() {
            // fix: toc_html/url 由 &mut book 借用传入同函数冲突，先克隆
            let toc_url = book.toc_url.clone();
            let toc_html = book.toc_html.clone();
            BookChapterList::analyze_chapter_list(
                book,
                Some(toc_html.as_deref().unwrap()),
                &self.book_source,
                &toc_url,
                &toc_url,
                None
            ).await
        } else {
            let toc_url = book.toc_url.clone();
            let book_url = book.book_url.clone();
            let mut analyze_url = AnalyzeUrl::new(
                toc_url.clone(),
                None,
                None,
                None,
                None,
                book_url,
                Some(prepared_source),
                Some(Box::new(book.clone())),
                None,
                None,
                self.debug_log_box()
            );
            let mut res = analyze_url.get_str_response_await(None, None, true).await;
            //检测书源是否已登录
            if let Some(check_js) = self.book_source.login_check_js.clone() {
                if check_js.is_not_blank() {
                    if let Some(js_value) = analyze_url.eval_js(check_js, res.body()) {
                        if let Some(js_str) = js_value.as_string() {
                            let res_url = res.url();
                            res = StrResponse::new_url(&res_url, Some(js_str));
                        }
                    }
                }
            }
            return BookChapterList::analyze_chapter_list(book, res.body().map(|b| b.as_str()), &self.book_source, &toc_url, &res.url(), self.debugger()).await;
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
        book.set_user_name_space(self.user_ns());
        let prepared_source = self.prepare_source();
        if prepared_source.get_content_rule().content.map_or(true, |c| c.is_empty()) {
            if let Some(d) = self.debugger() {
                d.log(Some(&self.book_source.book_source_url), Some(&format!("⇒正文规则为空,使用章节链接: {}", book_chapter.url)), false);
            }
            return book_chapter.url.clone();
        }
        if book_chapter.is_volume && book_chapter.url.starts_with(&book_chapter.title) {
            if let Some(d) = self.debugger() {
                d.log(Some(&self.book_source.book_source_url), Some("⇒一级目录正文不解析规则"), false);
            }
            return book_chapter.tag.clone().unwrap_or(String::new());
        }
        //        val body = if (book != None && bookChapter.url == book.bookUrl && !book.tocHtml.isNullOrEmpty()) {
        //            book.tocHtml
        //        } else {
        // logger.info("bookChapterUrl: {}", bookChapter.url, bookChapter.getAbsoluteURL())
        // fix: source/ruleData/chapter 均需所有权，以 stubs 提供的 Clone 副本传递；headerMapF 置 None 由 new 内按 source 回填
        let mut analyze_url = AnalyzeUrl::new(
            book_chapter.get_absolute_url(),
            None,
            None,
            None,
            None,
            book.toc_url.clone(),
            Some(prepared_source),
            Some(Box::new(book.clone())),
            Some(book_chapter.clone()),
            None,
            self.debug_log_box()
        );
        let content_rule = self.book_source.get_content_rule();
        let res = analyze_url.get_str_response_await(
            content_rule.web_js.clone(),
            content_rule.source_regex.clone(),
            true
        ).await;
        return BookContent::analyze_content(
            res.body().map(|b| b.as_str()),
            book,
            book_chapter,
            &self.book_source,
            &book_chapter.url,
            &res.url(),
            next_chapter_url,
            self.debugger()
        ).await;
    }
}
