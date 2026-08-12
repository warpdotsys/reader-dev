use crate::prelude::*;
use crate::stubs::Any;
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
        debug_log: Option<&dyn DebugLog>
    ) {
        if body.is_none() {
            panic!("error_get_web_content: {}", base_url);
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("≡获取成功:{}", base_url)), false);
        }
        let mut analyze_rule = AnalyzeRule::new(&mut *book, book_source, debug_log);
        analyze_rule.set_content(Some(Box::new(Any::from(body.unwrap()))), None).set_base_url(Some(base_url.to_string()));
        analyze_rule.set_redirect_url(redirect_url.to_string());
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
        debug_log: Option<&dyn DebugLog>
    ) {
        if body.is_none() {
            panic!("error_get_web_content: {}", base_url);
        }
        let info_rule = book_source.get_book_info_rule();
        if let Some(init) = info_rule.init.clone() {
            if !init.is_empty() {
                // coroutineContext.ensureActive()
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("≡执行详情页初始化规则"), false);
                }
                let init_element = analyze_rule.get_element(init.clone());
                analyze_rule.set_content(init_element, None);
            }
        }
        let m_can_re_name = can_re_name && !info_rule.can_re_name.is_null_or_empty();
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取书名"), false);
        }
        let name = BookHelp::format_book_name(&analyze_rule.get_string(info_rule.name.clone(), None, false));
        if !name.is_empty() && (m_can_re_name || book.name.is_empty()) {
            book.name = name.clone();
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", name)), false);
        }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取作者"), false);
        }
        let author = BookHelp::format_book_author(&analyze_rule.get_string(info_rule.author.clone(), None, false));
        if !author.is_empty() && (m_can_re_name || book.author.is_empty()) {
            book.author = author.clone();
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", author)), false);
        }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取分类"), false);
        }
        // try {
        if let Some(kind_list) = analyze_rule.get_string_list(info_rule.kind.clone(), None, false) {
            let joined = kind_list.join(",");
            if !joined.is_empty() {
                book.kind = Some(joined);
            }
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", book.kind.clone().unwrap_or_default())), false);
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取字数"), false);
        }
        // try {
        let word_count = StringUtils::wordCountFormat(Some(&analyze_rule.get_string(info_rule.word_count.clone(), None, false)));
        if !word_count.is_empty() {
            book.word_count = Some(word_count);
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", book.word_count.clone().unwrap_or_default())), false);
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取最新章节"), false);
        }
        // try {
        let last_chapter = analyze_rule.get_string(info_rule.last_chapter.clone(), None, false);
        if !last_chapter.is_empty() {
            book.latest_chapter_title = Some(last_chapter);
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", book.latest_chapter_title.clone().unwrap_or_default())), false);
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取简介"), false);
        }
        // try {
        let intro = analyze_rule.get_string(info_rule.intro.clone(), None, false);
        if !intro.is_empty() {
            book.intro = Some(htmlFormat(Some(&intro)));
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", book.intro.clone().unwrap_or_default())), false);
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取封面链接"), false);
        }
        // try {
        let cover_url = analyze_rule.get_string(info_rule.cover_url.clone(), None, false);
        if !cover_url.is_empty() {
            book.cover_url = Some(get_absolute_url(None, cover_url.clone()));
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", book.cover_url.clone().unwrap_or_default())), false);
        }
        // } catch (e: Exception) {
        //     debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
        // }
        // coroutineContext.ensureActive()
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取目录链接"), false);
        }
        book.toc_url = analyze_rule.get_string(info_rule.toc_url.clone(), None, true);
        if book.toc_url.is_empty() {
            book.toc_url = base_url.to_string();
        }
        if book.toc_url == base_url {
            book.toc_html = Some(body.unwrap().to_string());
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", book.toc_url)), false);
        }
    }
}
