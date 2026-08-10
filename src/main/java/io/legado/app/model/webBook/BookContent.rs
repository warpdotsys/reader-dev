// package io.legado.app.model.webBook
//
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.BookSource
// import io.legado.app.data.entities.rule.ContentRule
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.utils.HtmlFormatter
// import kotlinx.coroutines.CoroutineScope
// import kotlinx.coroutines.Dispatchers.IO
// import kotlinx.coroutines.async
// import kotlinx.coroutines.ensureActive
// import kotlinx.coroutines.withContext
// import kotlin.coroutines.coroutineContext

pub struct BookContent;

impl BookContent {

    pub async fn analyze_content(
        body: Option<&str>,
        book: &Book,
        book_chapter: &BookChapter,
        book_source: &BookSource,
        base_url: &str,
        redirect_url: &str,
        next_chapter_url: Option<&str>,
        debug_log: Option<&DebugLog>
    ) -> String {
        if body.is_none() {
            panic!("error_get_web_content");
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("≡获取成功:{}", base_url));
        }
        let m_next_chapter_url = if !next_chapter_url.is_none() && !next_chapter_url.unwrap().is_empty() {
            Some(next_chapter_url.unwrap().to_string())
        } else {
            // appDb.bookChapterDao.getChapter(book.bookUrl, bookChapter.index + 1)?.url
            None
        };
        let mut content = String::new();
        let mut next_url_list = vec![redirect_url.to_string()];
        let content_rule = book_source.get_content_rule();
        let analyze_rule = AnalyzeRule::new(book, book_source, debug_log).set_content(body.unwrap(), base_url);
        analyze_rule.set_redirect_url(redirect_url);
        analyze_rule.chapter = book_chapter;
        analyze_rule.next_chapter_url = m_next_chapter_url.clone();
        // coroutineContext.ensureActive()
        let mut content_data = Self::analyze_content_private(
            book, base_url, redirect_url, body.unwrap(), &content_rule, book_chapter, book_source,
            &m_next_chapter_url, true, debug_log
        );
        content.push_str(&content_data.0);
        if content_data.1.len() == 1 {
            let mut next_url = content_data.1[0].clone();
            while !next_url.is_empty() && !next_url_list.contains(&next_url) {
                if !m_next_chapter_url.is_none()
                    && get_absolute_url(redirect_url, &next_url)
                    == get_absolute_url(redirect_url, m_next_chapter_url.as_ref().unwrap())
                {
                    break;
                }
                next_url_list.push(next_url.clone());
                // coroutineContext.ensureActive()
                let res = AnalyzeUrl::new(
                    &next_url,
                    book_source,
                    book,
                    book_source.get_header_map(),
                    debug_log
                ).get_str_response_await();
                if let Some(next_body) = res.body {
                    content_data = Self::analyze_content_private(
                        book, &next_url, &res.url, &next_body, &content_rule,
                        book_chapter, book_source, &m_next_chapter_url, false, debug_log
                    );
                    next_url = if !content_data.1.is_empty() {
                        content_data.1[0].clone()
                    } else {
                        String::new()
                    };
                    content.push_str("\n");
                    content.push_str(&content_data.0);
                }
            }
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, &format!("◇本章总页数:{}", next_url_list.len()));
            }
        } else if content_data.1.len() > 1 {
            // coroutineContext.ensureActive()
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, &format!("◇并发解析正文,总页数:{}", content_data.1.len()));
            }
            // withContext(IO) {
            let mut futures = Vec::new();
            for it in 0..content_data.1.len() {
                let url_str = content_data.1[it].clone();
                let analyze_url = AnalyzeUrl::new(
                    &url_str,
                    book_source,
                    book,
                    book_source.get_header_map(),
                    debug_log
                );
                let res = analyze_url.get_str_response_await();
                let body_ = res.body.unwrap();
                futures.push(Self::analyze_content_private(
                    book, &url_str, &res.url, &body_, &content_rule,
                    book_chapter, book_source, &m_next_chapter_url, false, debug_log
                ));
            }
            for fut in futures {
                // coroutineContext.ensureActive()
                content.push_str("\n");
                content.push_str(&fut.0);
            }
            // }
        }
        let mut content_str = content;
        let replace_regex = content_rule.replace_regex.clone();
        if !replace_regex.is_none() && !replace_regex.clone().unwrap().is_empty() {
            content_str = analyze_rule.get_string(&replace_regex.unwrap(), &content_str);
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取章节名称");
            dl.log(&book_source.book_source_url, &format!("└{}", book_chapter.title));
            dl.log(&book_source.book_source_url, &format!("┌获取正文内容 (长度：{})", content_str.len()));
            if content_str.len() > 300 {
                dl.log(&book_source.book_source_url, &format!(
                    "└\n{} ... {}",
                    content_str[..150].to_string(),
                    content_str[content_str.len() - 150..].to_string()
                ));
            } else {
                dl.log(&book_source.book_source_url, &format!("└\n{}", content_str));
            }
        }
        return content_str;
    }

    // @Throws(Exception::class)
    fn analyze_content_private(
        book: &Book,
        base_url: &str,
        redirect_url: &str,
        body: &str,
        content_rule: &ContentRule,
        chapter: &BookChapter,
        book_source: &BookSource,
        next_chapter_url: &Option<String>,
        print_log: bool,
        debug_log: Option<&DebugLog>
    ) -> (String, Vec<String>) {
        let mut analyze_rule = AnalyzeRule::new(book, book_source, debug_log);
        analyze_rule.set_content(body, base_url);
        analyze_rule.chapter = chapter;
        let r_url = analyze_rule.set_redirect_url(redirect_url);
        analyze_rule.next_chapter_url = next_chapter_url.clone();
        let mut next_url_list = Vec::<String>::new();
        analyze_rule.chapter = chapter;
        //获取正文
        let mut content = analyze_rule.get_string(&content_rule.content);
        content = html_formatter_format_keep_img(&content, &r_url);
        //获取下一页链接
        let next_url_rule = content_rule.next_content_url.clone();
        if !next_url_rule.is_none() && !next_url_rule.clone().unwrap().is_empty() {
            if print_log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取正文下一页链接");
                }
            }
            if let Some(list) = analyze_rule.get_string_list(&next_url_rule.unwrap(), true) {
                next_url_list.extend(list);
            }
            if print_log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", next_url_list.join("，")));
                }
            }
        }
        return (content, next_url_list);
    }
}
