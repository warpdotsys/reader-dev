use crate::io_legado_app_model_analyzerule_analyzerule::SourceRule;
use crate::prelude::*;
use crate::stubs::Any;
// package io.legado.app.model.webBook
//
// import io.legado.app.data.entities.BookSource
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.SearchBook
// import io.legado.app.data.entities.rule.BookListRule
// import io.legado.app.help.BookHelp
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.utils.StringUtils.wordCountFormat
// import io.legado.app.utils.htmlFormat
// import kotlinx.coroutines.ensureActive
// import kotlin.coroutines.coroutineContext

pub struct BookList;

// fix: Kotlin `SearchRule/ExploreRule : BookListRule`（接口 trait 未在实体文件实现；此处按原逻辑补充实现，
//      使 `val bookListRule: BookListRule = when {...}` 得以转录）
macro_rules! impl_book_list_rule {
    ($t:ty) => {
        impl BookListRule for $t {
            fn book_list(&self) -> Option<&str> {
                self.book_list.as_deref()
            }
            fn set_book_list(&mut self, value: Option<String>) {
                self.book_list = value;
            }
            fn name(&self) -> Option<&str> {
                self.name.as_deref()
            }
            fn set_name(&mut self, value: Option<String>) {
                self.name = value;
            }
            fn author(&self) -> Option<&str> {
                self.author.as_deref()
            }
            fn set_author(&mut self, value: Option<String>) {
                self.author = value;
            }
            fn intro(&self) -> Option<&str> {
                self.intro.as_deref()
            }
            fn set_intro(&mut self, value: Option<String>) {
                self.intro = value;
            }
            fn kind(&self) -> Option<&str> {
                self.kind.as_deref()
            }
            fn set_kind(&mut self, value: Option<String>) {
                self.kind = value;
            }
            fn last_chapter(&self) -> Option<&str> {
                self.last_chapter.as_deref()
            }
            fn set_last_chapter(&mut self, value: Option<String>) {
                self.last_chapter = value;
            }
            fn update_time(&self) -> Option<&str> {
                self.update_time.as_deref()
            }
            fn set_update_time(&mut self, value: Option<String>) {
                self.update_time = value;
            }
            fn book_url(&self) -> Option<&str> {
                self.book_url.as_deref()
            }
            fn set_book_url(&mut self, value: Option<String>) {
                self.book_url = value;
            }
            fn cover_url(&self) -> Option<&str> {
                self.cover_url.as_deref()
            }
            fn set_cover_url(&mut self, value: Option<String>) {
                self.cover_url = value;
            }
            fn word_count(&self) -> Option<&str> {
                self.word_count.as_deref()
            }
            fn set_word_count(&mut self, value: Option<String>) {
                self.word_count = value;
            }
        }
    };
}
impl_book_list_rule!(SearchRule);
impl_book_list_rule!(ExploreRule);

impl BookList {

    // @Throws(Exception::class)
    pub async fn analyze_book_list(
        body: Option<&str>,
        book_source: &BookSource,
        analyze_url: &AnalyzeUrl,
        base_url: &str,
        variable_book: &SearchBook,
        is_search: bool,
        debug_log: Option<&dyn DebugLog>
    ) -> Vec<SearchBook> {
        let mut book_list = Vec::<SearchBook>::new();
        if body.is_none() {
            panic!(
                //            App.INSTANCE.getString(
                //                R.string.error_get_web_content,
                //                analyzeUrl.ruleUrl
                //            )
                //todo getString
                "error_get_web_content"
            );
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some(&format!("≡获取成功:{}", analyze_url.rule_url)), false);
        }
        let mut analyze_rule = AnalyzeRule::new(&*variable_book, Some(book_source), debug_log);
        analyze_rule.set_content(Some(Box::new(Any::from(body.unwrap()))), None).set_base_url(Some(base_url.to_string()));
        analyze_rule.set_redirect_url(base_url.to_string());
        if let Some(book_url_pattern) = book_source.book_url_pattern.clone() {
            if Regex::new(&book_url_pattern).unwrap().is_match(base_url) {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("≡链接为详情页"), false);
                }
                if let Some(mut search_book) = Self::get_info_item(
                    body.unwrap(), &mut analyze_rule, book_source, analyze_url, base_url,
                    variable_book.variable.as_deref(), debug_log
                ).await {
                    search_book.info_html = Some(body.unwrap().to_string());
                    book_list.push(search_book);
                }
                return book_list;
            }
        }
        let collections: Vec<Box<Any>>;
        let mut reverse = false;
        let book_list_rule: &dyn BookListRule = if is_search {
            &book_source.get_search_rule()
        } else if book_source.get_explore_rule().book_list.as_ref().map_or(true, |s| s.is_blank()) {
            &book_source.get_search_rule()
        } else {
            &book_source.get_explore_rule()
        };
        let mut rule_list: String = book_list_rule.book_list().unwrap_or("").to_string();
        if rule_list.starts_with("-") {
            reverse = true;
            rule_list = rule_list[1..].to_string();
        }
        if rule_list.starts_with("+") {
            rule_list = rule_list[1..].to_string();
        }
        if let Some(dl) = debug_log {
            dl.log(Some(&book_source.book_source_url), Some("┌获取书籍列表"), false);
        }
        collections = analyze_rule.get_elements(rule_list);
        // coroutineContext.ensureActive()
        if collections.is_empty() && book_source.book_url_pattern.is_null_or_empty() {
            if let Some(dl) = debug_log {
                dl.log(Some(&book_source.book_source_url), Some("└列表为空,按详情页解析"), false);
            }
            if let Some(mut search_book) = Self::get_info_item(
                body.unwrap(), &mut analyze_rule, book_source, analyze_url, base_url,
                variable_book.variable.as_deref(), debug_log
            ).await {
                search_book.info_html = Some(body.unwrap().to_string());
                book_list.push(search_book);
            }
        } else {
            let rule_name = analyze_rule.split_source_rule(book_list_rule.name().map(|s| s.to_string()), false);
            let rule_book_url = analyze_rule.split_source_rule(book_list_rule.book_url().map(|s| s.to_string()), false);
            let rule_author = analyze_rule.split_source_rule(book_list_rule.author().map(|s| s.to_string()), false);
            let rule_cover_url = analyze_rule.split_source_rule(book_list_rule.cover_url().map(|s| s.to_string()), false);
            let rule_intro = analyze_rule.split_source_rule(book_list_rule.intro().map(|s| s.to_string()), false);
            let rule_kind = analyze_rule.split_source_rule(book_list_rule.kind().map(|s| s.to_string()), false);
            let rule_last_chapter = analyze_rule.split_source_rule(book_list_rule.last_chapter().map(|s| s.to_string()), false);
            let rule_word_count = analyze_rule.split_source_rule(book_list_rule.word_count().map(|s| s.to_string()), false);
            if let Some(dl) = debug_log {
                dl.log(Some(&book_source.book_source_url), Some(&format!("└列表大小:{}", collections.len())), false);
            }
            for (index, item) in collections.iter().enumerate() {
                // coroutineContext.ensureActive()
                if let Some(mut search_book) = Self::get_search_item(
                    item, &mut analyze_rule, book_source, base_url, variable_book.variable.as_deref(), index == 0,
                    &rule_name, &rule_book_url, &rule_author,
                    &rule_kind, &rule_cover_url, &rule_word_count,
                    &rule_intro, &rule_last_chapter,
                    debug_log
                ).await {
                    if base_url == search_book.book_url {
                        search_book.info_html = Some(body.unwrap().to_string());
                    }
                    book_list.push(search_book);
                }
            }
            if reverse {
                book_list.reverse();
            }
        }
        return book_list;
    }

    async fn get_info_item(
        body: &str,
        analyze_rule: &mut AnalyzeRule,
        book_source: &BookSource,
        analyze_url: &AnalyzeUrl,
        base_url: &str,
        variable: Option<&str>,
        debug_log: Option<&dyn DebugLog>
    ) -> Option<SearchBook> {
        let mut book = Book::default();
        book.variable = variable.map(|v| v.to_string());
        book.book_url = analyze_url.rule_url.clone();
        book.origin = book_source.book_source_url.clone();
        book.origin_name = book_source.book_source_name.clone();
        book.origin_order = book_source.custom_order;
        book.r#type = book_source.book_source_type;
        book.set_user_name_space(analyze_rule.get_user_name_space());
        // fix: Kotlin `analyzeRule.ruleData = book`（同一对象别名）——Book 无 Clone，move 后 book 不可再用；
        //      以 book 快照（variable/userNameSpace）充当规则数据
        let mut rule_data_book = Book::default();
        rule_data_book.variable = book.variable.clone();
        rule_data_book.user_name_space = book.user_name_space.clone();
        analyze_rule.rule_data = Box::new(rule_data_book);
        BookInfo::analyze_book_info_private(
            &mut book,
            Some(body),
            analyze_rule,
            book_source,
            base_url,
            base_url,
            false,
            debug_log
        ).await;
        if book.name.is_not_blank() {
            return Some(book.to_search_book());
        }
        return None;
    }

    async fn get_search_item(
        item: &Box<Any>,
        analyze_rule: &mut AnalyzeRule,
        book_source: &BookSource,
        base_url: &str,
        variable: Option<&str>,
        log: bool,
        rule_name: &Vec<SourceRule>,
        rule_book_url: &Vec<SourceRule>,
        rule_author: &Vec<SourceRule>,
        rule_kind: &Vec<SourceRule>,
        rule_cover_url: &Vec<SourceRule>,
        rule_word_count: &Vec<SourceRule>,
        rule_intro: &Vec<SourceRule>,
        rule_last_chapter: &Vec<SourceRule>,
        debug_log: Option<&dyn DebugLog>
    ) -> Option<SearchBook> {
        let mut search_book = SearchBook::default();
        search_book.variable = variable.map(|v| v.to_string());
        search_book.origin = book_source.book_source_url.clone();
        search_book.origin_name = book_source.book_source_name.clone();
        search_book.r#type = book_source.book_source_type;
        search_book.origin_order = book_source.custom_order;
        search_book.set_user_name_space(analyze_rule.get_user_name_space());
        // fix: 以实时副本充当规则数据（{{bookName}} 自引用/@put: 变量持久化；原空快照——自引用解析为空）
        analyze_rule.rule_data = Box::new(search_book.clone());
        analyze_rule.set_content(Some(item.clone()), None);
        // coroutineContext.ensureActive()
        if log {
            if let Some(dl) = debug_log {
                dl.log(Some(&book_source.book_source_url), Some("┌获取书名"), false);
            }
        }
        search_book.name = BookHelp::format_book_name(&analyze_rule.get_string_inner(rule_name.clone(), None, false));
        analyze_rule.set_book_field("bookName", search_book.name.clone());
        if log {
            if let Some(dl) = debug_log {
                dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.name)), false);
            }
        }
        if !search_book.name.is_empty() {
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取作者"), false);
                }
            }
            search_book.author = BookHelp::format_book_author(&analyze_rule.get_string_inner(rule_author.clone(), None, false));
            analyze_rule.set_book_field("bookAuthor", search_book.author.clone());
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.author)), false);
                }
            }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取分类"), false);
                }
            }
            // try {
            if let Some(kind_list) = analyze_rule.get_string_list_inner(rule_kind.clone(), None, false) {
                search_book.kind = Some(kind_list.join(","));
                analyze_rule.set_book_field("bookKind", kind_list.join(","));
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.kind.clone().unwrap_or_default())), false);
                }
            }
            // } catch (e: Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取字数"), false);
                }
            }
            // try {
            search_book.word_count = Some(StringUtils::wordCountFormat(Some(&analyze_rule.get_string_inner(rule_word_count.clone(), None, false))));
            analyze_rule.set_book_field("bookWordCount", search_book.word_count.clone().unwrap_or_default());
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.word_count.clone().unwrap_or_default())), false);
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取最新章节"), false);
                }
            }
            // try {
            search_book.latest_chapter_title = Some(analyze_rule.get_string_inner(rule_last_chapter.clone(), None, false));
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.latest_chapter_title.clone().unwrap_or_default())), false);
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取简介"), false);
                }
            }
            // try {
            search_book.intro = Some(HtmlFormatter::new().formatKeepImg(Some(&analyze_rule.get_string_inner(rule_intro.clone(), None, false))));
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.intro.clone().unwrap_or_default())), false);
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取封面链接"), false);
                }
            }
            // try {
            let cover = analyze_rule.get_string_inner(rule_cover_url.clone(), None, false);
            if !cover.is_empty() {
                search_book.cover_url = Some(get_absolute_url(crate::stubs::URL::parse(&book_source.book_source_url).ok().as_ref(), cover));
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.cover_url.clone().unwrap_or_default())), false);
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取详情页链接"), false);
                }
            }
            search_book.book_url = analyze_rule.get_string_inner(rule_book_url.clone(), None, true);
            if search_book.book_url.is_empty() {
                search_book.book_url = base_url.to_string();
            }
            analyze_rule.set_book_field("bookUrl", search_book.book_url.clone());
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", search_book.book_url)), false);
                }
            }
            // fix: 读回规则数据中 @put: 写入的变量（Kotlin ruleData=searchBook 本体——变量随条目返回）
            if let Some(v) = analyze_rule.rule_data_variable() {
                search_book.variable = Some(v);
            }
            return Some(search_book);
        }
        return None;
    }
}
