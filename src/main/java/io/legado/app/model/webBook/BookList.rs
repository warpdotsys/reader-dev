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

impl BookList {

    // @Throws(Exception::class)
    pub async fn analyze_book_list(
        body: Option<&str>,
        book_source: &BookSource,
        analyze_url: &AnalyzeUrl,
        base_url: &str,
        variable_book: &SearchBook,
        is_search: bool,
        debug_log: Option<&DebugLog>
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
            dl.log(&book_source.book_source_url, &format!("≡获取成功:{}", analyze_url.rule_url));
        }
        let mut analyze_rule = AnalyzeRule::new(variable_book, book_source, debug_log);
        analyze_rule.set_content(body.unwrap()).set_base_url(base_url);
        analyze_rule.set_redirect_url(base_url);
        if let Some(book_url_pattern) = book_source.book_url_pattern.clone() {
            if Regex::new(&book_url_pattern).unwrap().is_match(base_url) {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "≡链接为详情页");
                }
                if let Some(mut search_book) = Self::get_info_item(
                    body.unwrap(), &mut analyze_rule, book_source, analyze_url, base_url,
                    &variable_book.variable, debug_log
                ).await {
                    search_book.info_html = Some(body.unwrap().to_string());
                    book_list.push(search_book);
                }
                return book_list;
            }
        }
        let collections: Vec<Box<dyn Any>>;
        let mut reverse = false;
        let book_list_rule = if is_search {
            book_source.get_search_rule()
        } else if book_source.get_explore_rule().book_list.is_blank() {
            book_source.get_search_rule()
        } else {
            book_source.get_explore_rule()
        };
        let mut rule_list: String = book_list_rule.book_list.clone().unwrap_or(String::new());
        if rule_list.starts_with("-") {
            reverse = true;
            rule_list = rule_list[1..].to_string();
        }
        if rule_list.starts_with("+") {
            rule_list = rule_list[1..].to_string();
        }
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, "┌获取书籍列表");
        }
        collections = analyze_rule.get_elements(&rule_list);
        // coroutineContext.ensureActive()
        if collections.is_empty() && book_source.book_url_pattern.is_empty() {
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, "└列表为空,按详情页解析");
            }
            if let Some(mut search_book) = Self::get_info_item(
                body.unwrap(), &mut analyze_rule, book_source, analyze_url, base_url,
                &variable_book.variable, debug_log
            ).await {
                search_book.info_html = Some(body.unwrap().to_string());
                book_list.push(search_book);
            }
        } else {
            let rule_name = analyze_rule.split_source_rule(&book_list_rule.name);
            let rule_book_url = analyze_rule.split_source_rule(&book_list_rule.book_url);
            let rule_author = analyze_rule.split_source_rule(&book_list_rule.author);
            let rule_cover_url = analyze_rule.split_source_rule(&book_list_rule.cover_url);
            let rule_intro = analyze_rule.split_source_rule(&book_list_rule.intro);
            let rule_kind = analyze_rule.split_source_rule(&book_list_rule.kind);
            let rule_last_chapter = analyze_rule.split_source_rule(&book_list_rule.last_chapter);
            let rule_word_count = analyze_rule.split_source_rule(&book_list_rule.word_count);
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, &format!("└列表大小:{}", collections.len()));
            }
            for (index, item) in collections.iter().enumerate() {
                // coroutineContext.ensureActive()
                if let Some(mut search_book) = Self::get_search_item(
                    item, &mut analyze_rule, book_source, base_url, &variable_book.variable, index == 0,
                    &rule_name, &rule_book_url, &rule_author,
                    &rule_cover_url, &rule_intro, &rule_kind,
                    &rule_last_chapter, &rule_word_count,
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
        variable: &str,
        debug_log: Option<&DebugLog>
    ) -> Option<SearchBook> {
        let mut book = Book::new(variable);
        book.book_url = analyze_url.rule_url.clone();
        book.origin = book_source.book_source_url.clone();
        book.origin_name = book_source.book_source_name.clone();
        book.origin_order = book_source.custom_order;
        book.book_type = book_source.book_source_type.clone();
        book.set_user_name_space(analyze_rule.get_user_name_space());
        analyze_rule.rule_data = book;
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
        item: &Box<dyn Any>,
        analyze_rule: &mut AnalyzeRule,
        book_source: &BookSource,
        base_url: &str,
        variable: &str,
        log: bool,
        rule_name: &Vec<AnalyzeRule::SourceRule>,
        rule_book_url: &Vec<AnalyzeRule::SourceRule>,
        rule_author: &Vec<AnalyzeRule::SourceRule>,
        rule_kind: &Vec<AnalyzeRule::SourceRule>,
        rule_cover_url: &Vec<AnalyzeRule::SourceRule>,
        rule_word_count: &Vec<AnalyzeRule::SourceRule>,
        rule_intro: &Vec<AnalyzeRule::SourceRule>,
        rule_last_chapter: &Vec<AnalyzeRule::SourceRule>,
        debug_log: Option<&DebugLog>
    ) -> Option<SearchBook> {
        let mut search_book = SearchBook::new(variable);
        search_book.origin = book_source.book_source_url.clone();
        search_book.origin_name = book_source.book_source_name.clone();
        search_book.book_type = book_source.book_source_type.clone();
        search_book.origin_order = book_source.custom_order;
        search_book.set_user_name_space(analyze_rule.get_user_name_space());
        analyze_rule.rule_data = search_book;
        analyze_rule.set_content(item);
        // coroutineContext.ensureActive()
        if log {
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, "┌获取书名");
            }
        }
        search_book.name = format_book_name(&analyze_rule.get_string(rule_name));
        if log {
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, &format!("└{}", search_book.name));
            }
        }
        if !search_book.name.is_empty() {
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取作者");
                }
            }
            search_book.author = format_book_author(&analyze_rule.get_string(rule_author));
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.author));
                }
            }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取分类");
                }
            }
            // try {
            if let Some(kind_list) = analyze_rule.get_string_list(rule_kind) {
                search_book.kind = Some(kind_list.join(","));
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.kind.clone().unwrap_or_default()));
                }
            }
            // } catch (e: Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取字数");
                }
            }
            // try {
            search_book.word_count = word_count_format(&analyze_rule.get_string(rule_word_count));
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.word_count));
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取最新章节");
                }
            }
            // try {
            search_book.latest_chapter_title = analyze_rule.get_string(rule_last_chapter);
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.latest_chapter_title));
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取简介");
                }
            }
            // try {
            search_book.intro = analyze_rule.get_string(rule_intro).html_format();
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.intro));
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取封面链接");
                }
            }
            // try {
            let cover = analyze_rule.get_string(rule_cover_url);
            if !cover.is_empty() {
                search_book.cover_url = get_absolute_url(base_url, &cover);
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.cover_url));
                }
            }
            // } catch (e: java.lang.Exception) {
            //     if (log) debugLog?.log(bookSource.bookSourceUrl, "└${e.localizedMessage}")
            // }
            // coroutineContext.ensureActive()
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取详情页链接");
                }
            }
            search_book.book_url = analyze_rule.get_string(rule_book_url, true);
            if search_book.book_url.is_empty() {
                search_book.book_url = base_url.to_string();
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", search_book.book_url));
                }
            }
            return Some(search_book);
        }
        return None;
    }
}
