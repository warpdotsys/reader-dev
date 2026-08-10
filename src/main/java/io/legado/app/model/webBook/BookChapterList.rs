// package io.legado.app.model.webBook
//
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.BookSource
// import io.legado.app.data.entities.rule.TocRule
// import io.legado.app.exception.TocEmptyException
// import io.legado.app.model.DebugLog
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.utils.isTrue
// import io.legado.app.utils.TextUtils
// import kotlinx.coroutines.CoroutineScope
// import kotlinx.coroutines.Dispatchers.IO
// import kotlinx.coroutines.async
// import kotlinx.coroutines.ensureActive
// import kotlinx.coroutines.withContext
// import kotlin.coroutines.coroutineContext

pub struct BookChapterList;

impl BookChapterList {

    pub async fn analyze_chapter_list(
        book: &mut Book,
        body: Option<&str>,
        book_source: &BookSource,
        base_url: &str,
        redirect_url: &str,
        debug_log: Option<&DebugLog>
    ) -> Vec<BookChapter> {
        if body.is_none() {
            panic!(
                //            App.INSTANCE.getString(R.string.error_get_web_content, baseUrl)
                //todo getString
                "error_get_web_content"
            );
        }
        let mut chapter_list = Vec::<BookChapter>::new();
        if let Some(dl) = debug_log {
            dl.log(&book_source.book_source_url, &format!("≡获取成功:{}", base_url));
        }
        // debugLog?.log(bookSource.bookSourceUrl, body)
        let toc_rule = book_source.get_toc_rule();
        let mut next_url_list = vec![redirect_url.to_string()];
        let mut reverse = false;
        let mut list_rule = toc_rule.chapter_list.clone().unwrap_or(String::new());
        if list_rule.starts_with("-") {
            reverse = true;
            list_rule = list_rule[1..].to_string();
        }
        if list_rule.starts_with("+") {
            list_rule = list_rule[1..].to_string();
        }
        let mut chapter_data = Self::analyze_chapter_list_private(
            book, base_url, redirect_url, body.unwrap(),
            &toc_rule, &list_rule, book_source, true, true, debug_log
        ).await;
        chapter_list.extend(chapter_data.0);
        match chapter_data.1.len() {
            0 => {}
            1 => {
                let mut next_url = chapter_data.1[0].clone();
                while !next_url.is_empty() && !next_url_list.contains(&next_url) {
                    next_url_list.push(next_url.clone());
                    let analyze_url = AnalyzeUrl::new(
                        &next_url,
                        book_source,
                        book,
                        book_source.get_header_map(),
                        debug_log
                    );
                    if let Some(next_body) = analyze_url.get_str_response_await().body {
                        chapter_data = Self::analyze_chapter_list_private(
                            book, &next_url, &next_url,
                            &next_body, &toc_rule, &list_rule, book_source, true, false, debug_log
                        ).await;
                        next_url = chapter_data.1.first().cloned().unwrap_or(String::new());
                        chapter_list.extend(chapter_data.0);
                    }
                }
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("◇目录总页数:{}", next_url_list.len()));
                }
            }
            _ => {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("◇并发解析目录,总页数:{}", chapter_data.1.len()));
                }
                // withContext(IO) {
                let mut futures = Vec::new();
                for it in 0..chapter_data.1.len() {
                    let url_str = chapter_data.1[it].clone();
                    let analyze_url = AnalyzeUrl::new(
                        &url_str,
                        book_source,
                        book,
                        book_source.get_header_map(),
                        debug_log
                    );
                    let res = analyze_url.get_str_response_await();
                    let body_ = res.body.unwrap();
                    futures.push(Self::analyze_chapter_list_private(
                        book, &url_str, &res.url,
                        &body_, &toc_rule, &list_rule, book_source, false, false, debug_log
                    ));
                }
                for fut in futures {
                    chapter_list.extend(fut.await.0);
                }
                // }
            }
        }
        if chapter_list.is_empty() {
            panic!("目录为空");
        }
        //去重
        if !reverse {
            chapter_list.reverse();
        }
        // coroutineContext.ensureActive()
        let mut list = Vec::<BookChapter>::new();
        // LinkedHashSet(chapterList) 去重
        let mut seen = std::collections::HashSet::new();
        for c in chapter_list {
            if seen.insert(c.clone()) {
                list.push(c);
            }
        }
        // if (!book.getReverseToc()) {
        list.reverse();
        // }
        if let Some(dl) = debug_log {
            dl.log(&book.origin, &format!("◇目录总数:{}", list.len()));
        }
        for (index, book_chapter) in list.iter_mut().enumerate() {
            // coroutineContext.ensureActive()
            book_chapter.index = index;
        }
        if list.len() > 0 {
            book.latest_chapter_title = Some(list.last().unwrap().title.clone());
        }
        //        book.durChapterTitle =
        //            list.getOrNull(book.durChapterIndex)?.title ?: book.latestChapterTitle
        if book.total_chapter_num < list.len() {
            book.last_check_count = list.len() - book.total_chapter_num;
            // book.latestChapterTime = System.currentTimeMillis()
            // book.lastCheckTime = System.currentTimeMillis()
        }
        book.total_chapter_num = list.len();
        // coroutineContext.ensureActive()
        return list;
    }

    async fn analyze_chapter_list_private(
        book: &mut Book,
        base_url: &str,
        redirect_url: &str,
        body: &str,
        toc_rule: &TocRule,
        list_rule: &str,
        book_source: &BookSource,
        get_next_url: bool,
        log: bool,
        debug_log: Option<&DebugLog>
    ) -> (Vec<BookChapter>, Vec<String>) {
        let mut analyze_rule = AnalyzeRule::new(book, book_source, debug_log);
        analyze_rule.set_content(body).set_base_url(base_url);
        analyze_rule.set_redirect_url(redirect_url);
        //获取目录列表
        let mut chapter_list = Vec::<BookChapter>::new();
        if log {
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, "┌获取目录列表");
            }
        }
        let elements = analyze_rule.get_elements(list_rule);
        if log {
            if let Some(dl) = debug_log {
                dl.log(&book_source.book_source_url, &format!("└列表大小:{}", elements.len()));
            }
        }
        //获取下一页链接
        let mut next_url_list = Vec::<String>::new();
        let next_toc_rule = toc_rule.next_toc_url.clone();
        if get_next_url && !next_toc_rule.is_empty() {
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌获取目录下一页列表");
                }
            }
            if let Some(list) = analyze_rule.get_string_list(&next_toc_rule, true) {
                for item in list {
                    if item != redirect_url {
                        next_url_list.push(item);
                    }
                }
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, &format!("└{}", join("，\n", &next_url_list)));
                }
            }
        }
        // coroutineContext.ensureActive()
        if !elements.is_empty() {
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "┌解析目录列表");
                }
            }
            let name_rule = analyze_rule.split_source_rule(&toc_rule.chapter_name);
            let url_rule = analyze_rule.split_source_rule(&toc_rule.chapter_url);
            let vip_rule = analyze_rule.split_source_rule(&toc_rule.is_vip);
            let up_time_rule = analyze_rule.split_source_rule(&toc_rule.update_time);
            let is_volume_rule = analyze_rule.split_source_rule(&toc_rule.is_volume);
            for (index, item) in elements.iter().enumerate() {
                // coroutineContext.ensureActive()
                analyze_rule.set_content(item);
                let mut book_chapter = BookChapter::new(book.book_url.clone(), redirect_url.to_string());
                analyze_rule.chapter = Some(book_chapter.clone());
                book_chapter.title = analyze_rule.get_string(&name_rule);
                book_chapter.url = analyze_rule.get_string(&url_rule);
                book_chapter.tag = analyze_rule.get_string(&up_time_rule);
                book_chapter.set_user_name_space(book.get_user_name_space());
                let is_volume = analyze_rule.get_string(&is_volume_rule);
                book_chapter.is_volume = false;
                if is_volume.is_true() {
                    book_chapter.is_volume = true;
                }
                if book_chapter.url.is_empty() {
                    if book_chapter.is_volume {
                        book_chapter.url = book_chapter.title.clone() + &index.to_string();
                        if log {
                            if let Some(dl) = debug_log {
                                dl.log(&book_source.book_source_url, &format!("⇒一级目录{}未获取到url,使用标题替代", index));
                            }
                        }
                    } else {
                        book_chapter.url = base_url.to_string();
                        if log {
                            if let Some(dl) = debug_log {
                                dl.log(&book_source.book_source_url, &format!("⇒目录{}未获取到url,使用baseUrl替代", index));
                            }
                        }
                    }
                }
                if !book_chapter.title.is_empty() {
                    let is_vip = analyze_rule.get_string(&vip_rule);
                    if is_vip.is_true() {
                        book_chapter.title = format!("\u{1F512}{}", book_chapter.title);
                    }
                    chapter_list.push(book_chapter);
                } else if log {
                    if let Some(dl) = debug_log {
                        dl.log(&book_source.book_source_url, "章节名为空");
                    }
                }
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(&book_source.book_source_url, "└目录列表解析完成");
                    if !chapter_list.is_empty() {
                        dl.log(&book_source.book_source_url, "≡首章信息");
                        dl.log(&book_source.book_source_url, &format!("◇章节名称:{}", chapter_list[0].title));
                        dl.log(&book_source.book_source_url, &format!("◇章节链接:{}", chapter_list[0].url));
                        dl.log(&book_source.book_source_url, &format!("◇章节信息:{}", chapter_list[0].tag));
                        dl.log(&book_source.book_source_url, &format!("◇是否卷名:{}", chapter_list[0].is_volume));
                    } else {
                        dl.log(&book_source.book_source_url, "章节列表为空");
                    }
                }
            }
        }
        return (chapter_list, next_url_list);
    }
}

fn join(sep: &str, list: &Vec<String>) -> String {
    let mut s = String::new();
    for (i, item) in list.iter().enumerate() {
        if i > 0 {
            s.push_str(sep);
        }
        s.push_str(item);
    }
    s
}
