// package io.legado.app.model.webBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookSource
// import io.legado.app.help.BookHelp
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.utils.StringUtils.wordCountFormat
// import io.legado.app.utils.htmlFormat
// import kotlinx.coroutines.ensureActive
// import kotlin.coroutines.coroutineContext

pub struct BookInfo;

impl BookInfo {

    // @Throws(Exception::class)
    pub async fn analyze_book_info(
        book: &mut Book,
        body: Option<&str>,
        book_source: &BookSource,
        base_url: &str,
        redirect_url: &str,
        can_re_name: bool,
        debug_log: Option<&DebugLog>
    ) {
        if body.is_none() {
            panic!("error_get_web_content: {}", base_url);
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("≡获取成功:{}", base_url));
        }
        let mut analyze_rule = AnalyzeRule::new(book, book_source, debug_log);
        analyze_rule.set_content(body.unwrap()).set_base_url(base_url);
        analyze_rule.set_redirect_url(redirect_url);
        Self::analyze_book_info_private(book, body, &mut analyze_rule, book_source, base_url, redirect_url, can_re_name, debug_log).await;
    }

    // @Throws(Exception::class)
    pub async fn analyze_book_info_private(
        book: &mut Book,
        body: Option<&str>,
        analyze_rule: &mut AnalyzeRule,
        book_source: &BookSource,
        base_url: &str,
        redirect_url: &str,
        can_re_name: bool,
        debug_log: Option<&DebugLog>
    ) {
        if body.is_none() {
            panic!("error_get_web_content: {}", base_url);
        }
        let info_rule = book_source.get_book_info_rule();
        if let Some(init) = info_rule.init.clone() {
            if !init.is_empty() {
                // coroutineContext.ensureActive()
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "≡执行详情页初始化规则");
                }
                analyze_rule.set_content(analyze_rule.get_element(&init));
            }
        }
        let m_can_re_name = can_re_name && !info_rule.can_re_name.is_blank();
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取书名");
        }
        let name = format_book_name(&analyze_rule.get_string(&info_rule.name));
        if !name.is_empty() && (m_can_re_name || book.name.is_empty()) {
            book.name = name.clone();
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", name));
        }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取作者");
        }
        let author = format_book_author(&analyze_rule.get_string(&info_rule.author));
        if !author.is_empty() && (m_can_re_name || book.author.is_empty()) {
            book.author = author.clone();
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", author));
        }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取分类");
        }
        // try {
        if let Some(kind_list) = analyze_rule.get_string_list(&info_rule.kind) {
            let joined = kind_list.join(",");
            if !joined.is_empty() {
                book.kind = joined;
            }
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", book.kind));
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取字数");
        }
        // try {
        let word_count = word_count_format(&analyze_rule.get_string(&info_rule.word_count));
        if !word_count.is_empty() {
            book.word_count = word_count;
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", book.word_count));
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取最新章节");
        }
        // try {
        let last_chapter = analyze_rule.get_string(&info_rule.last_chapter);
        if !last_chapter.is_empty() {
            book.latest_chapter_title = Some(last_chapter);
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", book.latest_chapter_title.clone().unwrap_or_default()));
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取简介");
        }
        // try {
        let intro = analyze_rule.get_string(&info_rule.intro);
        if !intro.is_empty() {
            book.intro = intro.html_format();
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", book.intro));
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取封面链接");
        }
        // try {
        let cover_url = analyze_rule.get_string(&info_rule.cover_url);
        if !cover_url.is_empty() {
            book.cover_url = get_absolute_url(redirect_url, &cover_url);
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", book.cover_url));
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取目录链接");
        }
        book.toc_url = analyze_rule.get_string(&info_rule.toc_url, true);
        if book.toc_url.is_empty() {
            book.toc_url = base_url.to_string();
        }
        if book.toc_url == base_url {
            book.toc_html = Some(body.unwrap().to_string());
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("└{}", book.toc_url));
        }
    }
}
