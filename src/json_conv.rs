// 自动生成：实体 -> serde_json::Value（ReturnData JSON 序列化）
use serde_json::{Value, json};

pub fn search_result_to_json(v: &crate::io_legado_app_data_entities_searchresult::SearchResult) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("resultCount"), json!(v.result_count));
    m.insert(String::from("resultCountWithinChapter"), json!(v.result_count_within_chapter));
    m.insert(String::from("resultText"), json!(v.result_text));
    m.insert(String::from("chapterTitle"), json!(v.chapter_title));
    m.insert(String::from("query"), json!(v.query));
    m.insert(String::from("pageSize"), json!(v.page_size));
    m.insert(String::from("chapterIndex"), json!(v.chapter_index));
    m.insert(String::from("pageIndex"), json!(v.page_index));
    m.insert(String::from("queryIndexInResult"), json!(v.query_index_in_result));
    m.insert(String::from("queryIndexInChapter"), json!(v.query_index_in_chapter));
    Value::Object(m)
}

pub fn search_results_to_json(v: &[crate::io_legado_app_data_entities_searchresult::SearchResult]) -> Value {
    Value::Array(v.iter().map(|x| search_result_to_json(x)).collect())
}

pub fn rss_article_to_json(v: &crate::io_legado_app_data_entities_rssarticle::RssArticle) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("origin"), json!(v.origin));
    m.insert(String::from("sort"), json!(v.sort));
    m.insert(String::from("title"), json!(v.title));
    m.insert(String::from("order"), json!(v.order));
    m.insert(String::from("link"), json!(v.link));
    insert_opt(&mut m, "pubDate", v.pub_date.clone());
    insert_opt(&mut m, "description", v.description.clone());
    insert_opt(&mut m, "content", v.content.clone());
    insert_opt(&mut m, "image", v.image.clone());
    m.insert(String::from("read"), json!(v.read));
    insert_opt(&mut m, "variable", v.variable.clone());
    Value::Object(m)
}

pub fn rss_articles_to_json(v: &[crate::io_legado_app_data_entities_rssarticle::RssArticle]) -> Value {
    Value::Array(v.iter().map(|x| rss_article_to_json(x)).collect())
}

pub fn book_to_json(v: &crate::io_legado_app_data_entities_book::Book) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("bookUrl"), json!(v.book_url));
    m.insert(String::from("tocUrl"), json!(v.toc_url));
    m.insert(String::from("origin"), json!(v.origin));
    m.insert(String::from("originName"), json!(v.origin_name));
    m.insert(String::from("name"), json!(v.name));
    m.insert(String::from("author"), json!(v.author));
    insert_opt(&mut m, "kind", v.kind.clone());
    insert_opt(&mut m, "customTag", v.custom_tag.clone());
    insert_opt(&mut m, "coverUrl", v.cover_url.clone());
    insert_opt(&mut m, "customCoverUrl", v.custom_cover_url.clone());
    insert_opt(&mut m, "intro", v.intro.clone());
    insert_opt(&mut m, "customIntro", v.custom_intro.clone());
    insert_opt(&mut m, "charset", v.charset.clone());
    m.insert(String::from("group"), json!(v.group));
    insert_opt(&mut m, "latestChapterTitle", v.latest_chapter_title.clone());
    m.insert(String::from("latestChapterTime"), json!(v.latest_chapter_time));
    m.insert(String::from("lastCheckTime"), json!(v.last_check_time));
    m.insert(String::from("lastCheckCount"), json!(v.last_check_count as i64));
    m.insert(String::from("totalChapterNum"), json!(v.total_chapter_num as i64));
    insert_opt(&mut m, "durChapterTitle", v.dur_chapter_title.clone());
    m.insert(String::from("durChapterIndex"), json!(v.dur_chapter_index as i64));
    m.insert(String::from("durChapterPos"), json!(v.dur_chapter_pos as i64));
    m.insert(String::from("durChapterTime"), json!(v.dur_chapter_time));
    insert_opt(&mut m, "wordCount", v.word_count.clone());
    m.insert(String::from("canUpdate"), json!(v.can_update));
    m.insert(String::from("type"), json!(v.r#type));
    m.insert(String::from("readConfig"), match v.read_config.lock().unwrap().as_ref() {
        Some(c) => json!({
            "reverseToc": c.reverse_toc,
            "pageAnim": c.page_anim,
            "reSegment": c.re_segment,
            "imageStyle": c.image_style,
            "useReplaceRule": c.use_replace_rule,
            "delTag": c.del_tag,
            "pdfImageWidth": c.pdf_image_width,
        }),
        None => Value::Null,
    });
    m.insert(String::from("order"), json!(v.order as i64));
    m.insert(String::from("originOrder"), json!(v.origin_order as i64));
    m.insert(String::from("useReplaceRule"), json!(v.use_replace_rule));
    insert_opt(&mut m, "variable", v.variable.clone());
    m.insert(String::from("isInShelf"), json!(v.is_in_shelf));
    insert_opt(&mut m, "lastCheckError", v.last_check_error.clone());
    insert_opt(&mut m, "infoHtml", v.info_html.clone());
    insert_opt(&mut m, "tocHtml", v.toc_html.clone());
    m.insert(String::from("rootDir"), json!(v.root_dir));
    Value::Object(m)
}

pub fn books_to_json(v: &[crate::io_legado_app_data_entities_book::Book]) -> Value {
    Value::Array(v.iter().map(|x| book_to_json(x)).collect())
}

pub fn book_source_to_json(v: &crate::io_legado_app_data_entities_booksource::BookSource) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("bookSourceUrl"), json!(v.book_source_url));
    m.insert(String::from("bookSourceName"), json!(v.book_source_name));
    insert_opt(&mut m, "bookSourceGroup", v.book_source_group.clone());
    m.insert(String::from("bookSourceType"), json!(v.book_source_type as i64));
    insert_opt(&mut m, "bookUrlPattern", v.book_url_pattern.clone());
    m.insert(String::from("customOrder"), json!(v.custom_order as i64));
    m.insert(String::from("enabled"), json!(v.enabled));
    m.insert(String::from("enabledExplore"), json!(v.enabled_explore));
    insert_opt(&mut m, "enabledCookieJar", v.enabled_cookie_jar.clone());
    insert_opt(&mut m, "concurrentRate", v.concurrent_rate.clone());
    insert_opt(&mut m, "header", v.header.clone());
    insert_opt(&mut m, "loginUrl", v.login_url.clone());
    insert_opt(&mut m, "loginUi", v.login_ui.clone());
    insert_opt(&mut m, "loginCheckJs", v.login_check_js.clone());
    insert_opt(&mut m, "bookSourceComment", v.book_source_comment.clone());
    insert_opt(&mut m, "variableComment", v.variable_comment.clone());
    m.insert(String::from("lastUpdateTime"), json!(v.last_update_time));
    m.insert(String::from("respondTime"), json!(v.respond_time));
    m.insert(String::from("weight"), json!(v.weight as i64));
    insert_opt(&mut m, "exploreUrl", v.explore_url.clone());
    insert_opt(&mut m, "searchUrl", v.search_url.clone());
    if let Some(r) = &v.rule_search {
        m.insert(String::from("ruleSearch"), search_rule_to_json(r));
    }
    if let Some(r) = &v.rule_book_info {
        m.insert(String::from("ruleBookInfo"), book_info_rule_to_json(r));
    }
    if let Some(r) = &v.rule_toc {
        m.insert(String::from("ruleToc"), toc_rule_to_json(r));
    }
    if let Some(r) = &v.rule_content {
        m.insert(String::from("ruleContent"), content_rule_to_json(r));
    }
    if let Some(r) = &v.rule_explore {
        m.insert(String::from("ruleExplore"), explore_rule_to_json(r));
    }
    Value::Object(m)
}

pub fn book_sources_to_json(v: &[crate::io_legado_app_data_entities_booksource::BookSource]) -> Value {
    Value::Array(v.iter().map(|x| book_source_to_json(x)).collect())
}

pub fn rss_source_to_json(v: &crate::io_legado_app_data_entities_rsssource::RssSource) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("sourceUrl"), json!(v.source_url));
    m.insert(String::from("sourceName"), json!(v.source_name));
    m.insert(String::from("sourceIcon"), json!(v.source_icon));
    insert_opt(&mut m, "sourceGroup", v.source_group.clone());
    insert_opt(&mut m, "sourceComment", v.source_comment.clone());
    m.insert(String::from("enabled"), json!(v.enabled));
    insert_opt(&mut m, "variableComment", v.variable_comment.clone());
    insert_opt(&mut m, "enabledCookieJar", v.enabled_cookie_jar.clone());
    insert_opt(&mut m, "concurrentRate", v.concurrent_rate.clone());
    insert_opt(&mut m, "header", v.header.clone());
    insert_opt(&mut m, "loginUrl", v.login_url.clone());
    insert_opt(&mut m, "loginUi", v.login_ui.clone());
    insert_opt(&mut m, "loginCheckJs", v.login_check_js.clone());
    insert_opt(&mut m, "sortUrl", v.sort_url.clone());
    m.insert(String::from("singleUrl"), json!(v.single_url));
    m.insert(String::from("articleStyle"), json!(v.article_style as i64));
    insert_opt(&mut m, "ruleArticles", v.rule_articles.clone());
    insert_opt(&mut m, "ruleNextPage", v.rule_next_page.clone());
    insert_opt(&mut m, "ruleTitle", v.rule_title.clone());
    insert_opt(&mut m, "rulePubDate", v.rule_pub_date.clone());
    insert_opt(&mut m, "ruleDescription", v.rule_description.clone());
    insert_opt(&mut m, "ruleImage", v.rule_image.clone());
    insert_opt(&mut m, "ruleLink", v.rule_link.clone());
    // fix: 缺失 ruleContent（Kotlin RssSource.ruleContent——保存一次后正文规则永久丢失，RSS 正文解析失效）
    insert_opt(&mut m, "ruleContent", v.rule_content.clone());
    insert_opt(&mut m, "style", v.style.clone());
    m.insert(String::from("enableJs"), json!(v.enable_js));
    m.insert(String::from("loadWithBaseUrl"), json!(v.load_with_base_url));
    m.insert(String::from("customOrder"), json!(v.custom_order as i64));
    Value::Object(m)
}

pub fn rss_sources_to_json(v: &[crate::io_legado_app_data_entities_rsssource::RssSource]) -> Value {
    Value::Array(v.iter().map(|x| rss_source_to_json(x)).collect())
}

pub fn replace_rule_to_json(v: &crate::io_legado_app_data_entities_replacerule::ReplaceRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("id"), json!(v.id));
    m.insert(String::from("name"), json!(v.name));
    insert_opt(&mut m, "group", v.group.clone());
    m.insert(String::from("pattern"), json!(v.pattern));
    m.insert(String::from("replacement"), json!(v.replacement));
    insert_opt(&mut m, "scope", v.scope.clone());
    m.insert(String::from("scopeTitle"), json!(v.scope_title));
    m.insert(String::from("scopeContent"), json!(v.scope_content));
    m.insert(String::from("isEnabled"), json!(v.is_enabled));
    m.insert(String::from("isRegex"), json!(v.is_regex));
    m.insert(String::from("timeoutMillisecond"), json!(v.timeout_millisecond));
    m.insert(String::from("order"), json!(v.order as i64));
    Value::Object(m)
}

pub fn replace_rules_to_json(v: &[crate::io_legado_app_data_entities_replacerule::ReplaceRule]) -> Value {
    Value::Array(v.iter().map(|x| replace_rule_to_json(x)).collect())
}

pub fn book_group_to_json(v: &crate::io_legado_app_data_entities_bookgroup::BookGroup) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("groupId"), json!(v.group_id));
    m.insert(String::from("groupName"), json!(v.group_name));
    insert_opt(&mut m, "cover", v.cover.clone());
    m.insert(String::from("order"), json!(v.order as i64));
    m.insert(String::from("show"), json!(v.show));
    Value::Object(m)
}

pub fn book_groups_to_json(v: &[crate::io_legado_app_data_entities_bookgroup::BookGroup]) -> Value {
    Value::Array(v.iter().map(|x| book_group_to_json(x)).collect())
}

pub fn bookmark_to_json(v: &crate::io_legado_app_data_entities_bookmark::Bookmark) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("time"), json!(v.time));
    m.insert(String::from("bookName"), json!(v.book_name));
    m.insert(String::from("bookAuthor"), json!(v.book_author));
    m.insert(String::from("chapterIndex"), json!(v.chapter_index as i64));
    m.insert(String::from("chapterPos"), json!(v.chapter_pos as i64));
    m.insert(String::from("chapterName"), json!(v.chapter_name));
    m.insert(String::from("bookText"), json!(v.book_text));
    m.insert(String::from("content"), json!(v.content));
    Value::Object(m)
}

pub fn bookmarks_to_json(v: &[crate::io_legado_app_data_entities_bookmark::Bookmark]) -> Value {
    Value::Array(v.iter().map(|x| bookmark_to_json(x)).collect())
}

pub fn txt_toc_rule_to_json(v: &crate::io_legado_app_data_entities_txttocrule::TxtTocRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("id"), json!(v.id));
    m.insert(String::from("name"), json!(v.name));
    m.insert(String::from("rule"), json!(v.rule));
    m.insert(String::from("serialNumber"), json!(v.serial_number as i64));
    m.insert(String::from("enable"), json!(v.enable));
    Value::Object(m)
}
pub fn txt_toc_rules_to_json(v: &[crate::io_legado_app_data_entities_txttocrule::TxtTocRule]) -> Value {

    Value::Array(v.iter().map(|x| txt_toc_rule_to_json(x)).collect())
}

pub fn search_book_to_json(v: &crate::io_legado_app_data_entities_searchbook::SearchBook) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("bookUrl"), json!(v.book_url));
    m.insert(String::from("origin"), json!(v.origin));
    m.insert(String::from("originName"), json!(v.origin_name));
    m.insert(String::from("name"), json!(v.name));
    m.insert(String::from("author"), json!(v.author));
    insert_opt(&mut m, "kind", v.kind.clone());
    insert_opt(&mut m, "coverUrl", v.cover_url.clone());
    insert_opt(&mut m, "intro", v.intro.clone());
    insert_opt(&mut m, "wordCount", v.word_count.clone());
    insert_opt(&mut m, "latestChapterTitle", v.latest_chapter_title.clone());
    m.insert(String::from("tocUrl"), json!(v.toc_url));
    m.insert(String::from("time"), json!(v.time));
    insert_opt(&mut m, "variable", v.variable.clone());
    m.insert(String::from("originOrder"), json!(v.origin_order as i64));
    insert_opt(&mut m, "infoHtml", v.info_html.clone());
    insert_opt(&mut m, "tocHtml", v.toc_html.clone());
    // fix: 缺失 type（Kotlin SearchBook.type——音频书源加入书架后恒被当文本书）
    m.insert(String::from("type"), json!(v.r#type as i64));
    Value::Object(m)
}

pub fn search_books_to_json(v: &[crate::io_legado_app_data_entities_searchbook::SearchBook]) -> Value {
    Value::Array(v.iter().map(|x| search_book_to_json(x)).collect())
}

pub fn book_chapter_to_json(v: &crate::io_legado_app_data_entities_bookchapter::BookChapter) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("url"), json!(v.url));
    m.insert(String::from("title"), json!(v.title));
    m.insert(String::from("isVolume"), json!(v.is_volume));
    m.insert(String::from("baseUrl"), json!(v.base_url));
    m.insert(String::from("bookUrl"), json!(v.book_url));
    m.insert(String::from("index"), json!(v.index as i64));
    insert_opt(&mut m, "resourceUrl", v.resource_url.clone());
    insert_opt(&mut m, "tag", v.tag.clone());
    insert_opt(&mut m, "start", v.start.clone());
    insert_opt(&mut m, "end", v.end.clone());
    insert_opt(&mut m, "startFragmentId", v.start_fragment_id.clone());
    insert_opt(&mut m, "endFragmentId", v.end_fragment_id.clone());
    insert_opt(&mut m, "variable", v.variable.clone());
    Value::Object(m)
}

pub fn book_chapters_to_json(v: &[crate::io_legado_app_data_entities_bookchapter::BookChapter]) -> Value {
    Value::Array(v.iter().map(|x| book_chapter_to_json(x)).collect())
}

pub fn http_tts_to_json(v: &crate::io_legado_app_data_entities_httptts::HttpTTS) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("id"), json!(v.id));
    m.insert(String::from("name"), json!(v.name));
    m.insert(String::from("url"), json!(v.url));
    insert_opt(&mut m, "contentType", v.content_type.clone());
    insert_opt(&mut m, "concurrentRate", v.concurrent_rate.clone());
    insert_opt(&mut m, "loginUrl", v.login_url.clone());
    insert_opt(&mut m, "loginUi", v.login_ui.clone());
    insert_opt(&mut m, "header", v.header.clone());
    insert_opt(&mut m, "jsLib", v.js_lib.clone());
    insert_opt(&mut m, "enabledCookieJar", v.enabled_cookie_jar.clone());
    insert_opt(&mut m, "loginCheckJs", v.login_check_js.clone());
    m.insert(String::from("lastUpdateTime"), json!(v.last_update_time));
    Value::Object(m)
}

pub fn http_tts_list_to_json(v: &[crate::io_legado_app_data_entities_httptts::HttpTTS]) -> Value {
    Value::Array(v.iter().map(|x| http_tts_to_json(x)).collect())
}

pub fn user_to_json(v: &crate::com_htmake_reader_entity_user::User) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("username"), json!(v.username));
    m.insert(String::from("password"), json!(v.password));
    m.insert(String::from("salt"), json!(v.salt));
    m.insert(String::from("token"), json!(v.token));
    m.insert(String::from("lastLoginAt"), json!(v.last_login_at));
    m.insert(String::from("createdAt"), json!(v.created_at));
    m.insert(String::from("enableWebdav"), json!(v.enable_webdav));
    m.insert(String::from("enableLocalStore"), json!(v.enable_local_store));
    m.insert(String::from("enableBookSource"), json!(v.enable_book_source));
    m.insert(String::from("enableRssSource"), json!(v.enable_rss_source));
    m.insert(String::from("bookSourceLimit"), json!(v.book_source_limit as i64));
    m.insert(String::from("bookLimit"), json!(v.book_limit as i64));
    Value::Object(m)
}

pub fn users_to_json(v: &[crate::com_htmake_reader_entity_user::User]) -> Value {
    Value::Array(v.iter().map(|x| user_to_json(x)).collect())
}

pub fn vec_string_to_value(v: &[String]) -> Value { Value::Array(v.iter().map(|s| json!(s)).collect()) }
pub fn map_opt<T: ToString>(v: Option<T>) -> Value {
    match v {
        Some(x) => {
            let s = x.to_string();
            if s == "true" {
                Value::Bool(true)
            } else if s == "false" {
                Value::Bool(false)
            } else if let Ok(i) = s.parse::<i64>() {
                Value::Number(i.into())
            } else if let Ok(f) = s.parse::<f64>() {
                Value::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
            } else {
                Value::String(s)
            }
        }
        None => Value::Null,
    }
}

/// fix: 对齐 Kotlin Gson NON_NULL——null 字段省略不写入（原显式 null 污染落盘/响应 JSON）
pub fn insert_opt<T: ToString>(m: &mut serde_json::Map<String, Value>, key: &str, v: Option<T>) {
    if let Some(x) = v {
        m.insert(key.to_string(), map_opt(Some(x)));
    }
}


// ================= 规则实体 → JSON（书源持久化） =================
use crate::io_legado_app_data_entities_rule_searchrule::SearchRule;
use crate::io_legado_app_data_entities_rule_bookinforule::BookInfoRule;
use crate::io_legado_app_data_entities_rule_tocrule::TocRule;
use crate::io_legado_app_data_entities_rule_contentrule::ContentRule;
use crate::io_legado_app_data_entities_rule_explorerule::ExploreRule;

fn opt_str_to_value(v: &Option<String>) -> Value {
    match v {
        Some(s) => Value::String(s.clone()),
        None => Value::Null,
    }
}

pub fn search_rule_to_json(v: &SearchRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("bookList".into(), opt_str_to_value(&v.book_list));
    m.insert("name".into(), opt_str_to_value(&v.name));
    m.insert("author".into(), opt_str_to_value(&v.author));
    m.insert("intro".into(), opt_str_to_value(&v.intro));
    m.insert("kind".into(), opt_str_to_value(&v.kind));
    m.insert("lastChapter".into(), opt_str_to_value(&v.last_chapter));
    m.insert("updateTime".into(), opt_str_to_value(&v.update_time));
    m.insert("bookUrl".into(), opt_str_to_value(&v.book_url));
    m.insert("coverUrl".into(), opt_str_to_value(&v.cover_url));
    m.insert("wordCount".into(), opt_str_to_value(&v.word_count));
    Value::Object(m)
}

pub fn book_info_rule_to_json(v: &BookInfoRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("init".into(), opt_str_to_value(&v.init));
    m.insert("name".into(), opt_str_to_value(&v.name));
    m.insert("author".into(), opt_str_to_value(&v.author));
    m.insert("intro".into(), opt_str_to_value(&v.intro));
    m.insert("kind".into(), opt_str_to_value(&v.kind));
    m.insert("lastChapter".into(), opt_str_to_value(&v.last_chapter));
    m.insert("updateTime".into(), opt_str_to_value(&v.update_time));
    m.insert("coverUrl".into(), opt_str_to_value(&v.cover_url));
    m.insert("tocUrl".into(), opt_str_to_value(&v.toc_url));
    m.insert("wordCount".into(), opt_str_to_value(&v.word_count));
    m.insert("canReName".into(), opt_str_to_value(&v.can_re_name));
    Value::Object(m)
}

pub fn toc_rule_to_json(v: &TocRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("preUpdateJs".into(), opt_str_to_value(&v.pre_update_js));
    m.insert("chapterList".into(), opt_str_to_value(&v.chapter_list));
    m.insert("chapterName".into(), opt_str_to_value(&v.chapter_name));
    m.insert("chapterUrl".into(), opt_str_to_value(&v.chapter_url));
    m.insert("isVolume".into(), opt_str_to_value(&v.is_volume));
    m.insert("isVip".into(), opt_str_to_value(&v.is_vip));
    m.insert("updateTime".into(), opt_str_to_value(&v.update_time));
    m.insert("nextTocUrl".into(), opt_str_to_value(&v.next_toc_url));
    Value::Object(m)
}

pub fn content_rule_to_json(v: &ContentRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("content".into(), opt_str_to_value(&v.content));
    m.insert("nextContentUrl".into(), opt_str_to_value(&v.next_content_url));
    m.insert("webJs".into(), opt_str_to_value(&v.web_js));
    m.insert("sourceRegex".into(), opt_str_to_value(&v.source_regex));
    m.insert("replaceRegex".into(), opt_str_to_value(&v.replace_regex));
    m.insert("imageStyle".into(), opt_str_to_value(&v.image_style));
    Value::Object(m)
}

pub fn explore_rule_to_json(v: &ExploreRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("bookList".into(), opt_str_to_value(&v.book_list));
    m.insert("name".into(), opt_str_to_value(&v.name));
    m.insert("author".into(), opt_str_to_value(&v.author));
    m.insert("intro".into(), opt_str_to_value(&v.intro));
    m.insert("kind".into(), opt_str_to_value(&v.kind));
    m.insert("lastChapter".into(), opt_str_to_value(&v.last_chapter));
    m.insert("updateTime".into(), opt_str_to_value(&v.update_time));
    m.insert("bookUrl".into(), opt_str_to_value(&v.book_url));
    m.insert("coverUrl".into(), opt_str_to_value(&v.cover_url));
    m.insert("wordCount".into(), opt_str_to_value(&v.word_count));
    Value::Object(m)
}
