// 自动生成：实体 -> serde_json::Value（ReturnData JSON 序列化）
use serde_json::{Value, json};

pub fn book_to_json(v: &crate::io_legado_app_data_entities_book::Book) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("bookUrl"), json!(v.book_url));
    m.insert(String::from("tocUrl"), json!(v.toc_url));
    m.insert(String::from("origin"), json!(v.origin));
    m.insert(String::from("originName"), json!(v.origin_name));
    m.insert(String::from("name"), json!(v.name));
    m.insert(String::from("author"), json!(v.author));
    m.insert(String::from("kind"), map_opt(v.kind.clone()));
    m.insert(String::from("customTag"), map_opt(v.custom_tag.clone()));
    m.insert(String::from("coverUrl"), map_opt(v.cover_url.clone()));
    m.insert(String::from("customCoverUrl"), map_opt(v.custom_cover_url.clone()));
    m.insert(String::from("intro"), map_opt(v.intro.clone()));
    m.insert(String::from("customIntro"), map_opt(v.custom_intro.clone()));
    m.insert(String::from("charset"), map_opt(v.charset.clone()));
    m.insert(String::from("group"), json!(v.group));
    m.insert(String::from("latestChapterTitle"), map_opt(v.latest_chapter_title.clone()));
    m.insert(String::from("latestChapterTime"), json!(v.latest_chapter_time));
    m.insert(String::from("lastCheckTime"), json!(v.last_check_time));
    m.insert(String::from("lastCheckCount"), json!(v.last_check_count as i64));
    m.insert(String::from("totalChapterNum"), json!(v.total_chapter_num as i64));
    m.insert(String::from("durChapterTitle"), map_opt(v.dur_chapter_title.clone()));
    m.insert(String::from("durChapterIndex"), json!(v.dur_chapter_index as i64));
    m.insert(String::from("durChapterPos"), json!(v.dur_chapter_pos as i64));
    m.insert(String::from("durChapterTime"), json!(v.dur_chapter_time));
    m.insert(String::from("wordCount"), map_opt(v.word_count.clone()));
    m.insert(String::from("canUpdate"), json!(v.can_update));
    m.insert(String::from("order"), json!(v.order as i64));
    m.insert(String::from("originOrder"), json!(v.origin_order as i64));
    m.insert(String::from("useReplaceRule"), json!(v.use_replace_rule));
    m.insert(String::from("variable"), map_opt(v.variable.clone()));
    m.insert(String::from("isInShelf"), json!(v.is_in_shelf));
    m.insert(String::from("lastCheckError"), map_opt(v.last_check_error.clone()));
    m.insert(String::from("infoHtml"), map_opt(v.info_html.clone()));
    m.insert(String::from("tocHtml"), map_opt(v.toc_html.clone()));
    m.insert(String::from("rootDir"), json!(v.root_dir));
    m.insert(String::from("userNameSpace"), json!(v.user_name_space));
    Value::Object(m)
}

pub fn books_to_json(v: &[crate::io_legado_app_data_entities_book::Book]) -> Value {
    Value::Array(v.iter().map(|x| book_to_json(x)).collect())
}

pub fn book_source_to_json(v: &crate::io_legado_app_data_entities_booksource::BookSource) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("bookSourceUrl"), json!(v.book_source_url));
    m.insert(String::from("bookSourceName"), json!(v.book_source_name));
    m.insert(String::from("bookSourceGroup"), map_opt(v.book_source_group.clone()));
    m.insert(String::from("bookSourceType"), json!(v.book_source_type as i64));
    m.insert(String::from("bookUrlPattern"), map_opt(v.book_url_pattern.clone()));
    m.insert(String::from("customOrder"), json!(v.custom_order as i64));
    m.insert(String::from("enabled"), json!(v.enabled));
    m.insert(String::from("enabledExplore"), json!(v.enabled_explore));
    m.insert(String::from("enabledCookieJar"), map_opt(v.enabled_cookie_jar.clone()));
    m.insert(String::from("concurrentRate"), map_opt(v.concurrent_rate.clone()));
    m.insert(String::from("header"), map_opt(v.header.clone()));
    m.insert(String::from("loginUrl"), map_opt(v.login_url.clone()));
    m.insert(String::from("loginUi"), map_opt(v.login_ui.clone()));
    m.insert(String::from("loginCheckJs"), map_opt(v.login_check_js.clone()));
    m.insert(String::from("bookSourceComment"), map_opt(v.book_source_comment.clone()));
    m.insert(String::from("variableComment"), map_opt(v.variable_comment.clone()));
    m.insert(String::from("lastUpdateTime"), json!(v.last_update_time));
    m.insert(String::from("respondTime"), json!(v.respond_time));
    m.insert(String::from("weight"), json!(v.weight as i64));
    m.insert(String::from("exploreUrl"), map_opt(v.explore_url.clone()));
    m.insert(String::from("searchUrl"), map_opt(v.search_url.clone()));
    m.insert(String::from("userNameSpace"), json!(v.user_name_space));
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
    m.insert(String::from("sourceGroup"), map_opt(v.source_group.clone()));
    m.insert(String::from("sourceComment"), map_opt(v.source_comment.clone()));
    m.insert(String::from("enabled"), json!(v.enabled));
    m.insert(String::from("variableComment"), map_opt(v.variable_comment.clone()));
    m.insert(String::from("enabledCookieJar"), map_opt(v.enabled_cookie_jar.clone()));
    m.insert(String::from("concurrentRate"), map_opt(v.concurrent_rate.clone()));
    m.insert(String::from("header"), map_opt(v.header.clone()));
    m.insert(String::from("loginUrl"), map_opt(v.login_url.clone()));
    m.insert(String::from("loginUi"), map_opt(v.login_ui.clone()));
    m.insert(String::from("loginCheckJs"), map_opt(v.login_check_js.clone()));
    m.insert(String::from("sortUrl"), map_opt(v.sort_url.clone()));
    m.insert(String::from("singleUrl"), json!(v.single_url));
    m.insert(String::from("articleStyle"), json!(v.article_style as i64));
    m.insert(String::from("ruleArticles"), map_opt(v.rule_articles.clone()));
    m.insert(String::from("ruleNextPage"), map_opt(v.rule_next_page.clone()));
    m.insert(String::from("ruleTitle"), map_opt(v.rule_title.clone()));
    m.insert(String::from("rulePubDate"), map_opt(v.rule_pub_date.clone()));
    m.insert(String::from("ruleDescription"), map_opt(v.rule_description.clone()));
    m.insert(String::from("ruleImage"), map_opt(v.rule_image.clone()));
    m.insert(String::from("ruleLink"), map_opt(v.rule_link.clone()));
    m.insert(String::from("style"), map_opt(v.style.clone()));
    m.insert(String::from("enableJs"), json!(v.enable_js));
    m.insert(String::from("loadWithBaseUrl"), json!(v.load_with_base_url));
    m.insert(String::from("customOrder"), json!(v.custom_order as i64));
    m.insert(String::from("userNameSpace"), json!(v.user_name_space));
    Value::Object(m)
}

pub fn rss_sources_to_json(v: &[crate::io_legado_app_data_entities_rsssource::RssSource]) -> Value {
    Value::Array(v.iter().map(|x| rss_source_to_json(x)).collect())
}

pub fn replace_rule_to_json(v: &crate::io_legado_app_data_entities_replacerule::ReplaceRule) -> Value {
    let mut m = serde_json::Map::new();
    m.insert(String::from("id"), json!(v.id));
    m.insert(String::from("name"), json!(v.name));
    m.insert(String::from("group"), map_opt(v.group.clone()));
    m.insert(String::from("pattern"), json!(v.pattern));
    m.insert(String::from("replacement"), json!(v.replacement));
    m.insert(String::from("scope"), map_opt(v.scope.clone()));
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
    m.insert(String::from("cover"), map_opt(v.cover.clone()));
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
    m.insert(String::from("kind"), map_opt(v.kind.clone()));
    m.insert(String::from("coverUrl"), map_opt(v.cover_url.clone()));
    m.insert(String::from("intro"), map_opt(v.intro.clone()));
    m.insert(String::from("wordCount"), map_opt(v.word_count.clone()));
    m.insert(String::from("latestChapterTitle"), map_opt(v.latest_chapter_title.clone()));
    m.insert(String::from("tocUrl"), json!(v.toc_url));
    m.insert(String::from("time"), json!(v.time));
    m.insert(String::from("variable"), map_opt(v.variable.clone()));
    m.insert(String::from("originOrder"), json!(v.origin_order as i64));
    m.insert(String::from("userNameSpace"), json!(v.user_name_space));
    m.insert(String::from("infoHtml"), map_opt(v.info_html.clone()));
    m.insert(String::from("tocHtml"), map_opt(v.toc_html.clone()));
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
    m.insert(String::from("resourceUrl"), map_opt(v.resource_url.clone()));
    m.insert(String::from("tag"), map_opt(v.tag.clone()));
    m.insert(String::from("start"), map_opt(v.start.clone()));
    m.insert(String::from("end"), map_opt(v.end.clone()));
    m.insert(String::from("startFragmentId"), map_opt(v.start_fragment_id.clone()));
    m.insert(String::from("endFragmentId"), map_opt(v.end_fragment_id.clone()));
    m.insert(String::from("variable"), map_opt(v.variable.clone()));
    m.insert(String::from("userNameSpace"), json!(v.user_name_space));
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
    m.insert(String::from("contentType"), map_opt(v.content_type.clone()));
    m.insert(String::from("concurrentRate"), map_opt(v.concurrent_rate.clone()));
    m.insert(String::from("loginUrl"), map_opt(v.login_url.clone()));
    m.insert(String::from("loginUi"), map_opt(v.login_ui.clone()));
    m.insert(String::from("header"), map_opt(v.header.clone()));
    m.insert(String::from("jsLib"), map_opt(v.js_lib.clone()));
    m.insert(String::from("enabledCookieJar"), map_opt(v.enabled_cookie_jar.clone()));
    m.insert(String::from("loginCheckJs"), map_opt(v.login_check_js.clone()));
    m.insert(String::from("lastUpdateTime"), json!(v.last_update_time));
    m.insert(String::from("userNameSpace"), json!(v.user_name_space));
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
    match v { Some(x) => Value::String(x.to_string()), None => Value::Null }
}

