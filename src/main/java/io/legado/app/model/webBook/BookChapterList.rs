use crate::prelude::*;
use crate::stubs::Any;
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
        debug_log: Option<&dyn DebugLog>
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
            dl.log(Some(&book_source.book_source_url), Some(&format!("≡获取成功:{}", base_url)), false);
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
                    // fix: Kotlin `AnalyzeUrl(mUrl=nextUrl, source=bookSource, ruleData=book,
                    //      headerMapF=bookSource.getHeaderMap(), debugLog=debugLog)`——AnalyzeUrl::new 收所有权,
                    //      source/ruleData/debugLog 为引用无法转移, 由 stubs 占位构造（同 AnalyzeRule::new 占位约定）
                    let mut analyze_url = analyze_url_new_placeholder(next_url.clone());
                    let res = analyze_url.get_str_response_await(None, None, false).await;
                    if let Some(next_body) = res.body() {
                        let res_url = res.url();
                        chapter_data = Self::analyze_chapter_list_private(
                            book, &next_url, &res_url,
                            next_body, &toc_rule, &list_rule, book_source, true, false, debug_log
                        ).await;
                        next_url = chapter_data.1.first().cloned().unwrap_or(String::new());
                        chapter_list.extend(chapter_data.0);
                    }
                }
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("◇目录总页数:{}", next_url_list.len())), false);
                }
            }
            _ => {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("◇并发解析目录,总页数:{}", chapter_data.1.len())), false);
                }
                // withContext(IO) {
                // fix: future 不能借用循环局部变量, 先收集(url, 响应url, body)再构造 future（逻辑等价 Kotlin asyncArray）
                let mut url_data: Vec<(String, String, String)> = Vec::new();
                for it in 0..chapter_data.1.len() {
                    let url_str = chapter_data.1[it].clone();
                    let mut analyze_url = analyze_url_new_placeholder(url_str.clone());
                    let res = analyze_url.get_str_response_await(None, None, false).await;
                    let body_ = res.body().cloned().unwrap();
                    url_data.push((url_str, res.url(), body_));
                }
                for (url_str, res_url, body_) in url_data {
                    chapter_list.extend(Self::analyze_chapter_list_private(
                        book, &url_str, &res_url,
                        &body_, &toc_rule, &list_rule, book_source, false, false, debug_log
                    ).await.0);
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
        // LinkedHashSet(chapterList) 去重（BookChapter 未实现 Clone, 按 url 去重, 等价 Kotlin equals/hashCode 仅比较 url）
        let mut seen = std::collections::HashSet::new();
        for c in chapter_list {
            if seen.insert(c.url.clone()) {
                list.push(c);
            }
        }
        // if (!book.getReverseToc()) {
        list.reverse();
        // }
        if let Some(dl) = debug_log {
            dl.log(Some(&book.origin), Some(&format!("◇目录总数:{}", list.len())), false);
        }
        for (index, book_chapter) in list.iter_mut().enumerate() {
            // coroutineContext.ensureActive()
            book_chapter.index = index as i32;
        }
        if list.len() > 0 {
            book.latest_chapter_title = Some(list.last().unwrap().title.clone());
        }
        //        book.durChapterTitle =
        //            list.getOrNull(book.durChapterIndex)?.title ?: book.latestChapterTitle
        if (book.total_chapter_num as usize) < list.len() {
            book.last_check_count = (list.len() as i32) - book.total_chapter_num;
            // book.latestChapterTime = System.currentTimeMillis()
            // book.lastCheckTime = System.currentTimeMillis()
        }
        book.total_chapter_num = list.len() as i32;
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
        debug_log: Option<&dyn DebugLog>
    ) -> (Vec<BookChapter>, Vec<String>) {
        let mut analyze_rule = AnalyzeRule::new(&mut *book, book_source, debug_log);
        analyze_rule.set_content(Some(Box::new(Any::from(body))), None).set_base_url(Some(base_url.to_string()));
        analyze_rule.set_redirect_url(redirect_url.to_string());
        //获取目录列表
        let mut chapter_list = Vec::<BookChapter>::new();
        if log {
            if let Some(dl) = debug_log {
                dl.log(Some(&book_source.book_source_url), Some("┌获取目录列表"), false);
            }
        }
        let elements = analyze_rule.get_elements(list_rule.to_string());
        if log {
            if let Some(dl) = debug_log {
                dl.log(Some(&book_source.book_source_url), Some(&format!("└列表大小:{}", elements.len())), false);
            }
        }
        //获取下一页链接
        let mut next_url_list = Vec::<String>::new();
        let next_toc_rule = toc_rule.next_toc_url.clone();
        if get_next_url && next_toc_rule.as_deref().map_or(true, |s| !s.is_empty()) {
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌获取目录下一页列表"), false);
                }
            }
            if let Some(list) = analyze_rule.get_string_list(next_toc_rule.clone(), None, true) {
                for item in list {
                    if item != redirect_url {
                        next_url_list.push(item);
                    }
                }
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some(&format!("└{}", join("，\n", &next_url_list))), false);
                }
            }
        }
        // coroutineContext.ensureActive()
        if !elements.is_empty() {
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("┌解析目录列表"), false);
                }
            }
            for (index, item) in elements.iter().enumerate() {
                // coroutineContext.ensureActive()
                analyze_rule.set_content(Some(item.clone()), None);
                // fix: BookChapter 未实现 Clone, analyzeRule.chapter 借道 Option 往返安装（Kotlin analyzeRule.chapter = bookChapter）
                let mut book_chapter = BookChapter {
                    book_url: book.book_url.clone(),
                    base_url: redirect_url.to_string(),
                    ..Default::default()
                };
                analyze_rule.chapter = Some(book_chapter);
                book_chapter = analyze_rule.chapter.take().unwrap();
                book_chapter.title = analyze_rule.get_string(toc_rule.chapter_name.clone(), None, false);
                analyze_rule.chapter = Some(book_chapter);
                book_chapter = analyze_rule.chapter.take().unwrap();
                book_chapter.url = analyze_rule.get_string(toc_rule.chapter_url.clone(), None, false);
                analyze_rule.chapter = Some(book_chapter);
                book_chapter = analyze_rule.chapter.take().unwrap();
                book_chapter.tag = Some(analyze_rule.get_string(toc_rule.update_time.clone(), None, false));
                analyze_rule.chapter = Some(book_chapter);
                book_chapter = analyze_rule.chapter.take().unwrap();
                book_chapter.set_user_name_space(book.get_user_name_space());
                analyze_rule.chapter = Some(book_chapter);
                book_chapter = analyze_rule.chapter.take().unwrap();
                let is_volume = analyze_rule.get_string(toc_rule.is_volume.clone(), None, false);
                book_chapter.is_volume = false;
                if is_volume.is_true() {
                    book_chapter.is_volume = true;
                }
                if book_chapter.url.is_empty() {
                    if book_chapter.is_volume {
                        book_chapter.url = book_chapter.title.clone() + &index.to_string();
                        if log {
                            if let Some(dl) = debug_log {
                                dl.log(Some(&book_source.book_source_url), Some(&format!("⇒一级目录{}未获取到url,使用标题替代", index)), false);
                            }
                        }
                    } else {
                        book_chapter.url = base_url.to_string();
                        if log {
                            if let Some(dl) = debug_log {
                                dl.log(Some(&book_source.book_source_url), Some(&format!("⇒目录{}未获取到url,使用baseUrl替代", index)), false);
                            }
                        }
                    }
                }
                if !book_chapter.title.is_empty() {
                    let is_vip = analyze_rule.get_string(toc_rule.is_vip.clone(), None, false);
                    if is_vip.is_true() {
                        book_chapter.title = format!("\u{1F512}{}", book_chapter.title);
                    }
                    chapter_list.push(book_chapter);
                } else if log {
                    if let Some(dl) = debug_log {
                        dl.log(Some(&book_source.book_source_url), Some("章节名为空"), false);
                    }
                }
            }
            if log {
                if let Some(dl) = debug_log {
                    dl.log(Some(&book_source.book_source_url), Some("└目录列表解析完成"), false);
                    if !chapter_list.is_empty() {
                        dl.log(Some(&book_source.book_source_url), Some("≡首章信息"), false);
                        dl.log(Some(&book_source.book_source_url), Some(&format!("◇章节名称:{}", chapter_list[0].title)), false);
                        dl.log(Some(&book_source.book_source_url), Some(&format!("◇章节链接:{}", chapter_list[0].url)), false);
                        dl.log(Some(&book_source.book_source_url), Some(&format!("◇章节信息:{}", chapter_list[0].tag.as_deref().unwrap_or(""))), false);
                        dl.log(Some(&book_source.book_source_url), Some(&format!("◇是否卷名:{}", chapter_list[0].is_volume)), false);
                    } else {
                        dl.log(Some(&book_source.book_source_url), Some("章节列表为空"), false);
                    }
                }
            }
        }
        return (chapter_list, next_url_list);
    }
}

pub fn join(sep: &str, list: &Vec<String>) -> String {
    let mut s = String::new();
    for (i, item) in list.iter().enumerate() {
        if i > 0 {
            s.push_str(sep);
        }
        s.push_str(item);
    }
    s
}
