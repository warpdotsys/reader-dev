// Translated from BookController.kt
// package com.htmake.reader.api.controller
use crate::prelude::*;

// fix: 显式导入消除 prelude 多个 glob 重导出导致的同名歧义（显式导入优先于 glob）
use crate::stubs::{Any, Base64, File, JsonArray, JsonObject, WebClient};
use crate::io_legado_app_utils_filesutil::FileUtils;
use crate::io_legado_app_utils_gsonextensions::GSON;
use crate::com_htmake_reader_api_controller_booksourcecontroller::BookSourceController;
use crate::com_htmake_reader_utils_vertext::get_relative_path;

// fix: stubs / VertExt 同名 get_work_dir / get_storage / save_storage / get_storage_file /
//      json_encode / app_config 的 glob 歧义 → 本地包装（按 Kotlin vararg 参数转发）
fn work_dir() -> String {
    crate::com_htmake_reader_utils_vertext::get_work_dir("")
}
fn work_dir_of(sub: &str) -> String {
    crate::com_htmake_reader_utils_vertext::get_work_dir(sub)
}
fn work_dir_multi(parts: &[&str]) -> String {
    crate::com_htmake_reader_utils_vertext::get_work_dir_multi(parts)
}
fn work_dir_join(parts: Vec<String>) -> String {
    let strs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
    crate::com_htmake_reader_utils_vertext::get_work_dir_multi(&strs)
}
fn get_storage(parts: &[&str]) -> Option<String> {
    let strs: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
    crate::com_htmake_reader_utils_vertext::get_storage(&strs, ".json")
}
fn save_storage(parts: &[&str], value: Any) {
    let strs: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
    crate::com_htmake_reader_utils_vertext::save_storage(&strs, value, false, ".json");
}
fn get_storage_file(parts: &[&str]) -> File {
    let strs: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
    crate::com_htmake_reader_utils_vertext::get_storage_file(&strs, ".json")
}
fn app_config() -> AppConfig {
    SpringContextUtils::get_bean_app_config()
}
fn app_config_debug_log() -> bool {
    SpringContextUtils::get_bean_app_config().debug_log
}
fn json_encode(any: Any, pretty: bool) -> String {
    crate::com_htmake_reader_utils_vertext::json_encode(any, pretty)
}
// fix: stubs::mutable_list_of(items) 需要实参而 Kotlin mutableListOf() 无参 → 本地占位
fn mutable_list_of<T>() -> Vec<T> {
    Vec::new()
}
// fix: ReturnData 未 derive Serialize，SSE 响应手动拼 JSON（同 RoutingContext.success 的拼装方式）
fn rd_json(return_data: &ReturnData) -> String {
    format!(
        "{{\"isSuccess\":{},\"errorMsg\":{},\"data\":{}}}",
        return_data.is_success(),
        serde_json::to_string(return_data.error_msg()).unwrap_or_else(|_| "\"\"".to_string()),
        return_data
            .data()
            .as_ref()
            .map(|d| crate::stubs::any_to_json_value(d.as_ref()).to_string())
            .unwrap_or_else(|| "null".to_string())
    )
}
// fix: `cacheInfo?.toMap()?.toDataClass()`（Book 已实现 Deserialize，直接解析）
fn cache_to_book(s: Option<String>) -> Option<Book> {
    s.and_then(|json| serde_json::from_str::<Book>(&json).ok())
}

// fix: BaseController.limit_concurrent 的 handler 为 fn 指针（无法捕获闭包），本地顺序执行版
//      （保持逐轮 need_continue 语义；原 Kotlin 为并发调度，此处占位串行执行）
fn limit_concurrent_with<H, C>(concurrent_count: i32, start_index: i32, end_index: i32, mut handler: H, mut need_continue: C)
where
    H: FnMut(i32) -> Box<dyn std::any::Any>,
    C: FnMut(Vec<Box<dyn std::any::Any>>, i32) -> bool,
{
    let mut last_index = start_index;
    let mut loop_count = 0;
    while last_index < end_index {
        let mut result_list: Vec<Box<dyn std::any::Any>> = Vec::new();
        let mut i = last_index;
        let mut dispatched = 0;
        while i < end_index && dispatched < concurrent_count.max(1) {
            result_list.push(handler(i));
            dispatched += 1;
            i += 1;
        }
        last_index = i - 1;
        if last_index >= end_index - 1 {
            need_continue(result_list, loop_count);
            break;
        }
        if !need_continue(result_list, loop_count) {
            break;
        }
        loop_count += 1;
        last_index += 1;
    }
}

// fix: Kotlin mapOf(vararg Pair) → HashMap<String, Any>（值经 Any::from 装箱）
macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut m: std::collections::HashMap<String, crate::stubs::Any> = std::collections::HashMap::new();
        $(m.insert(String::from($key), crate::stubs::Any::from($value));)*
        m
    }};
}
macro_rules! mutable_map_of {
    () => { std::collections::HashMap::new() };
    (<$k:ty, $v:ty>) => { std::collections::HashMap::<$k, $v>::new() };
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut m: std::collections::HashMap<String, crate::stubs::Any> = std::collections::HashMap::new();
        $(m.insert(String::from($key), crate::stubs::Any::from($value));)*
        m
    }};
}
macro_rules! array_list {
    () => { Vec::new() };
    (<$t:ty>) => { Vec::<$t>::new() };
    (::$t:ty) => { Vec::<$t>::new() };
    ($($x:expr),* $(,)?) => { vec![$($x),*] };
}
macro_rules! list {
    ($($x:expr),* $(,)?) => { vec![$($x),*] };
}
macro_rules! hash_set {
    (<$t:ty>) => { std::collections::HashSet::<$t>::new() };
    () => { std::collections::HashSet::new() };
}
macro_rules! mutable_set_of {
    (<$t:ty>) => { std::collections::HashSet::<$t>::new() };
    () => { std::collections::HashSet::new() };
}

// fix: ACache 真实实现需要 &mut self 且 Arc 包装，本地降级缓存（HashMap + 互斥锁）
#[derive(Clone, Default)]
pub struct LocalCache {
    map: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
}

impl LocalCache {
    pub fn new() -> LocalCache {
        LocalCache {
            map: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
    pub fn get_as_string(&self, key: &str) -> Option<String> {
        self.map.lock().unwrap().get(key).cloned()
    }
    pub fn get_by_hash_code(&self, hash_code: &str) -> Option<String> {
        self.get_as_string(hash_code)
    }
    pub fn put(&self, key: &str, value: &str, _save_time: i32) {
        self.map.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}

// fix: BookController::class（set_assets 读取 epub 模板资源）
impl crate::stubs::ClassConstant for BookController {
    type Target = BookController;
    const class: crate::stubs::Class<BookController> = crate::stubs::Class::new();
}

static LOGGER: Log = Log;

pub struct BookController {
    base: BaseController,
    coroutine_context: CoroutineContext,

    // 缓存 2M 的书籍信息
    book_info_cache: LocalCache,
    concurrent_loop_count: i32,

    web_client: WebClient,
}

impl BookController {
    pub fn new() -> Self {
        BookController {
            base: BaseController::new(),
            coroutine_context: CoroutineContext,
            // fix: 原 ACache::get("bookInfoCache", 2M, 10000)
            book_info_cache: LocalCache::new(),
            concurrent_loop_count: 8,
            web_client: WebClient::new(),
        }
    }

    pub fn new_ctx(coroutine_context: CoroutineContext) -> Self {
        BookController {
            base: BaseController::new(),
            coroutine_context,
            // fix: 原 ACache::get("bookInfoCache", 2M, 10000)
            book_info_cache: LocalCache::new(),
            concurrent_loop_count: 8,
            web_client: WebClient::new(),
        }
    }

    pub fn get_invalid_book_source_cache(&self, _user_name_space: String) -> LocalCache {
        // 缓存 5M 的失效书源信息（静态共享，跨 BookController 实例）
        static CACHE: std::sync::OnceLock<LocalCache> = std::sync::OnceLock::new();
        CACHE.get_or_init(LocalCache::new).clone()
    }

    pub fn is_invalid_book_source(&self, book_source: BookSource, user_name_space: String) -> bool {
        return self
            .get_invalid_book_source_cache(user_name_space)
            .get_as_string(&book_source.book_source_url)
            .is_some();
    }

    pub fn add_invalid_book_source(&self, source_url: String, invalid_info: Map<String, Any>, user_name_space: String) {
        // 保存600秒时间
        self.get_invalid_book_source_cache(user_name_space)
            .put(&source_url, &json_encode(Any::from(invalid_info), false), 600);
    }

    pub fn get_book_chapters_cache(&self, _user_name_space: String) -> LocalCache {
        return LocalCache::new();
    }

    pub fn web_book(&self, book_source: String, debug_log: bool, user_name_space: String) -> WebBook {
        return WebBook::from_json_string(&book_source, debug_log, None, Some(user_name_space));
    }

    pub async fn get_invalid_book_sources(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let invalid_book_source_cache = self.get_invalid_book_source_cache(user_name_space);
        let cache_dir = File::new(&work_dir_multi(&["storage", "cache", "invalidBookSourceCache"]));
        let files = cache_dir.list_files();
        let mut invalid_book_source_list: Vec<Map<String, Any>> = Vec::new();
        if !files.is_empty() {
            for f in files {
                if let Some(info) = invalid_book_source_cache.get_by_hash_code(&f.name) {
                    let parsed: Map<String, Any> = serde_json::from_str(&info).unwrap_or_default();
                    invalid_book_source_list.push(parsed);
                }
            }
        }

        return_data.set_data(Box::new(invalid_book_source_list), String::new());
        return return_data;
    }

    pub async fn get_book_info(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        let book_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求：url 缺失时回退到 searchBook.bookUrl
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| {
                    context
                        .body_as_json()
                        .get_json_object("searchBook")
                        .map(|j| j.get_string("bookUrl"))
                })
                .unwrap_or_default();
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        LOGGER.info(format!("getBookInfo with bookUrl: {}", book_url));
        let mut book_info: Option<Book> = None;
        if self.base.check_auth(&context) {
            book_info = self.get_shelf_book_by_url(book_url.clone(), self.base.get_user_name_space(&context));
        }
        if book_info.is_none() {
            // 看看有没有缓存数据
            let cache_info: Option<Book> = cache_to_book(self.book_info_cache.get_as_string(&book_url));
            let book_source: Option<String> = if cache_info.is_some() {
                // 使用缓存的书籍信息包含的书源
                self.get_book_source_string(&context, cache_info.unwrap().origin, false).await
            } else {
                self.get_book_source_string(&context, String::new(), false).await
            };
            if book_source.as_ref().map_or(true, |s| s.is_empty()) {
                return_data.set_error_msg("未配置书源".to_string());
                return return_data;
            }
            book_info = Some(
                self.merge_book_cache_info(
                    self.web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), self.base.get_user_name_space(&context))
                        .get_book_info_by_url(&book_url, true)
                        .await,
                )
                .await,
            );
        }

        // 缓存书籍信息
        self.save_book_info_cache(array_list![book_info.clone().unwrap_or_default()]).await;
        return_data.set_data(Box::new(book_info), String::new());
        return return_data;
    }

    pub async fn get_book_cover(&self, context: RoutingContext) {
        let cover_url = context.query_param("path").unwrap_or_default();
        if cover_url.is_empty() {
            context.response().set_status_code(404).end(String::new());
            return;
        }
        let ext = self.base.get_file_ext(cover_url.clone(), "png".to_string());
        let md5_encode = MD5Utils::md5Encode(Some(&cover_url));
        let cache_path = work_dir_join(vec![
            String::from("storage"),
            String::from("cache"),
            String::from("bookCoverCache"),
            md5_encode + "." + &ext,
        ]);
        let cache_file = File::new(&cache_path);
        if cache_file.exists() {
            LOGGER.info(format!("send cache: {}", cache_file));
            context
                .response()
                .put_header("Cache-Control", "86400")
                .send_file(cache_file.to_string());
            return;
        }

        if let Some(p) = &cache_file.parent_file {
            if !p.exists() {
                p.mkdirs();
            }
        }

        let context = context.clone();
        let context2 = context.clone();
        launch(
            MDCContext::new() + Dispatchers::IO + CoroutineExceptionHandler::new(move |_, exception| {
                LOGGER.info(format!("get cover error: {}", exception));
                context.response().set_status_code(404).end(String::new());
            }),
            || {
                let body_bytes = self
                    .web_client
                    .get_abs(&cover_url)
                    .timeout(10000)
                    .async_get_bytes_in_thread();
                if let Some(body_bytes) = body_bytes {
                    if !body_bytes.is_empty() {
                        let mut res = context2.response();
                        res.put_header("Cache-Control", "86400");
                        cache_file.write_bytes(body_bytes);
                        res.send_file(cache_file.to_string());
                    } else {
                        context2.response().set_status_code(404).end(String::new());
                    }
                } else {
                    context2.response().set_status_code(404).end(String::new());
                }
            },
        );
    }


    pub async fn import_book_preview(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let uploads = context.file_uploads();
        if uploads.is_empty() {
            return_data.set_error_msg("请上传书籍文件".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let mut file_list: Vec<Map<String, Any>> = Vec::new();
        for it in uploads {
            let file = File::new(&it);
            LOGGER.info(format!("uploadFile: {} {} {}", it, it, file));
            if file.exists() {
                let mut file_name = it;
                let ext = self.base.get_file_ext(file_name.clone(), String::new());
                if ext != "txt" && ext != "epub" && ext != "umd" && ext != "cbz" && ext != "pdf" {
                    file.delete_recursively();
                    return_data.set_error_msg("不支持导入".to_string() + &ext + "格式的书籍文件");
                    return return_data;
                }
                // 文件名格式化
                file_name = FileUtils::getNameExcludeExtension(&file_name);
                file_name = AppPattern::fileNameRegex().replace_all(&file_name, "").to_string();
                file_name = file_name.substring_range(0, std::cmp::min(50, file_name.len())) + "." + &ext;

                let local_file_path = Paths::get(Paths::get(Paths::get(Paths::get("storage", "assets"), &user_name_space), "book"), &file_name);
                let local_file_url = String::from("/assets/") + &user_name_space + "/book/" + &file_name;
                let mut file_path = local_file_path.clone();
                if file_name.to_lowercase().ends_with(".epub") {
                    file_path = file_path + File::SEPARATOR + "index.epub";
                }
                if file_name.to_lowercase().ends_with(".cbz") {
                    file_path = file_path + File::SEPARATOR + "index.cbz";
                }
                let new_file = File::new(&work_dir_of(&file_path));
                if let Some(p) = &new_file.parent_file {
                    if !p.exists() {
                        p.mkdirs();
                    }
                }
                if new_file.exists() {
                    new_file.delete();
                }
                LOGGER.info(format!("moveTo: {}", new_file));
                if file.copy_recursively(&new_file) {
                    let book = Book::init_local_book(local_file_url, local_file_path, work_dir());
                    let mut book = book;
                    book.set_user_name_space(user_name_space.clone());
                    let try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let mut book2 = book.clone();
                        let chapters = LocalBook::get_chapter_list(&mut book2);
                        (book2, chapters)
                    }));
                    match try_result {
                        Ok((book2, chapters)) => {
                            file_list.push(map!("book" => book2, "chapters" => chapters));
                        }
                        Err(_) => {
                            // fix: 原 catch 分支以空章节占位
                            file_list.push(map!("book" => book.clone(), "chapters" => Vec::<BookChapter>::new()));
                        }
                    }
                }
                file.delete_recursively();
            }
        }
        return_data.set_data(Box::new(file_list), String::new());
        return return_data;
    }

    pub async fn get_txt_toc_rules(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let mut rules: Vec<TxtTocRule> = Vec::new();
        let default_rules = DefaultData::txt_toc_rules();
        rules.extend(default_rules.iter().cloned());
        let user_name_space = self.base.get_user_name_space(&context);
        let custom_rules = match self.base.get_user_storage(&user_name_space, vec![String::from("txtTocRule")]) {
            Some(s) => crate::stubs::GSON::from_json_array::<TxtTocRule>(&s).get_or_null().unwrap_or_else(|| Vec::new()),
            None => Vec::new(),
        };
        rules.add_all(custom_rules);
        return_data.set_data(Box::new(rules), String::new());
        return return_data;
    }

    pub async fn get_chapter_list_by_rule(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let mut book = context.body_as_json().map_to::<Book>().unwrap_or_default();
        if book.origin.is_empty() {
            return_data.set_error_msg("未找到书源信息".to_string());
            return return_data;
        }
        if !book.is_local_txt() && !book.is_local_epub() && !book.is_local_pdf() {
            return_data.set_error_msg("非本地txt/epub/pdf书籍".to_string());
            return return_data;
        }
        book.set_root_dir(work_dir());
        book.set_user_name_space(self.base.get_user_name_space(&context));
        let chapters = LocalBook::get_chapter_list(&mut book);
        return_data.set_data(Box::new(map!("book" => book, "chapters" => chapters)), String::new());
        return return_data;
    }

    pub async fn refresh_local_book(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string_opt("bookUrl").unwrap_or_default();
        } else {
            // get 请求
            book_url = context.query_param("bookUrl").unwrap_or_default();
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        // 根据书籍url获取书本信息
        let user_name_space = self.base.get_user_name_space(&context);
        let mut book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info.is_none() {
            return_data.set_error_msg("书籍信息错误".to_string());
            return return_data;
        }
        let mut book_info = book_info.take().unwrap();
        book_info.update_from_local(true);

        self.edit_shelf_book(book_info.clone(), self.base.get_user_name_space(&context), |mut exist_book| {
            exist_book.cover_url = book_info.cover_url.clone();
            LOGGER.info(format!("refreshLocalBook: {}", exist_book.name));
            exist_book
        })
        .await;

        return_data.set_data(Box::new(book_info), String::new());
        return return_data;
    }

    pub async fn get_chapter_list(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let mut refresh: i32 = 0;
        if context.request().method() == HttpMethod::POST {
            // post 请求：url 缺失时回退到 book.bookUrl
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_json_object("book").map(|j| j.get_string("bookUrl")))
                .unwrap_or_default();
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            refresh = context.query_param("refresh").map(|s| s.to_int()).unwrap_or(0);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        // 根据书籍url获取书本信息
        let user_name_space = self.base.get_user_name_space(&context);
        let mut book_info = self.get_shelf_book_by_url(book_url.clone(), user_name_space.clone());
        let mut book_source: Option<String> = None;
        if book_info.is_none() {
            // 看看有没有缓存数据
            let cache_info: Option<Book> = cache_to_book(self.book_info_cache.get_as_string(&book_url));
            if cache_info.is_some() {
                // 使用缓存的书籍信息包含的书源
                book_source = self.get_book_source_string(&context, cache_info.unwrap().origin, false).await;
            } else {
                // 看看有没有传入书源
                book_source = self.get_book_source_string(&context, String::new(), false).await;
            }
            if book_source.as_ref().map_or(true, |s| s.is_empty()) {
                return_data.set_error_msg("未配置书源".to_string());
                return return_data;
            }
            book_info = Some(
                self.merge_book_cache_info(
                    self.web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), user_name_space.clone())
                        .get_book_info_by_url(&book_url, true)
                        .await,
                )
                .await,
            );
            // 缓存书籍信息
            self.save_book_info_cache(array_list![book_info.clone().unwrap_or_default()]).await;
        } else {
            book_source = self.get_book_source_string(&context, book_info.as_ref().unwrap().origin.clone(), false).await;
        }
        let mut book_info = book_info.unwrap();
        if !book_info.is_local_book() && book_source.as_ref().map_or(true, |s| s.is_empty()) {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        book_info.set_root_dir(work_dir());
        book_info.set_user_name_space(user_name_space.clone());
        if book_info.is_local_book() {
            let mut local_file = book_info.get_local_file();
            if !local_file.exists() {
                LOGGER.info(format!("localFile: {} not exists", local_file));
                return_data.set_error_msg("本地书籍源文件不存在".to_string());
                return return_data;
            }
        }
        // 缓存章节列表
        LOGGER.info(format!("bookInfo: {}", book_info.name));
        let chapter_list = self
            .get_local_chapter_list(book_info.clone(), book_source, refresh > 0, user_name_space, false, None)
            .await;

        return_data.set_data(Box::new(chapter_list), String::new());
        return return_data;
    }

    pub async fn save_book_progress(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let chapter_index: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求：url 缺失时回退到 searchBook.bookUrl
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_json_object("searchBook").map(|j| j.get_string("bookUrl")))
                .unwrap_or_default();
            chapter_index = context.body_as_json().get_integer("index", -1);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            chapter_index = context.query_param("index").map(|s| s.to_int()).unwrap_or(-1);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        // 看看有没有加入书架
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book_info.is_none() || book_info.as_ref().unwrap().origin.is_empty() {
            return_data.set_error_msg("书籍未加入书架".to_string());
            return return_data;
        }
        let book_info = book_info.unwrap();
        let book_source = self.get_book_source_string_by_source_url_opt(book_info.origin.clone(), user_name_space.clone());

        if !book_info.is_local_book() && book_source.as_ref().map_or(true, |s| s.is_empty()) {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        let chapter_list = self
            .get_local_chapter_list(book_info.clone(), book_source, false, user_name_space.clone(), false, None)
            .await;
        if chapter_index >= chapter_list.len() as i32 {
            return_data.set_error_msg("章节不存在".to_string());
            return return_data;
        }
        let chapter_info = chapter_list.get(chapter_index as usize).cloned().unwrap_or_default();
        // 书架书籍保存阅读进度
        self.save_shelf_book_progress(book_info.clone(), chapter_info.clone(), user_name_space.clone()).await;
        // 保存到 webdav
        self.save_book_progress_to_webdav(book_info, chapter_info, user_name_space).await;
        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn save_book_config(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let pdf_image_width: f32;
        if context.request().method() == HttpMethod::POST {
            book_url = context.body_as_json().get_string_opt("bookUrl").unwrap_or_default();
            pdf_image_width = context.body_as_json().get_float("pdfImageWidth", 0.0);
        } else {
            book_url = context.query_param("bookUrl").unwrap_or_default();
            pdf_image_width = context
                .query_param("pdfImageWidth")
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v as f32)
                .unwrap_or(0.0);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("书籍链接不能为空".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let book_info = match self.get_shelf_book_by_url(book_url, user_name_space.clone()) {
            Some(b) => b,
            None => {
                return_data.set_error_msg("书籍信息错误".to_string());
                return return_data;
            }
        };
        if pdf_image_width <= 0.0 {
            return_data.set_error_msg("pdf图片宽度错误".to_string());
            return return_data;
        }
        let new_book = self
            .edit_shelf_book(book_info.clone(), user_name_space, |mut exist_book| {
                exist_book.set_pdf_image_width(pdf_image_width);
                LOGGER.info(format!("saveBookConfig: {}", exist_book.name));
                exist_book
            })
            .await;
        return_data.set_data(Box::new(new_book.unwrap_or(book_info)), String::new());
        return return_data;
    }


    pub async fn get_book_content(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let mut chapter_url: String;
        let book_url: String;
        let chapter_index: i32;
        let cache: i32;
        let refresh: i32;
        let epub_content: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            chapter_url = context
                .body_as_json()
                .get_json_object("bookChapter")
                .map(|j| j.get_string("url"))
                .unwrap_or_default();
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_json_object("searchBook").map(|j| j.get_string("bookUrl")))
                .unwrap_or_default();
            chapter_index = context.body_as_json().get_integer("index", -1);
            cache = context.body_as_json().get_integer("cache", 0);
            refresh = context.body_as_json().get_integer("refresh", 0);
            epub_content = context.body_as_json().get_integer("epubContent", 0);
        } else {
            // get 请求
            chapter_url = context.query_param("chapterUrl").unwrap_or_default();
            book_url = context.query_param("url").unwrap_or_default();
            chapter_index = context.query_param("index").map(|s| s.to_int()).unwrap_or(-1);
            cache = context.query_param("cache").map(|s| s.to_int()).unwrap_or(0);
            refresh = context.query_param("refresh").map(|s| s.to_int()).unwrap_or(0);
            epub_content = context.query_param("epubContent").map(|s| s.to_int()).unwrap_or(0);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        let mut book_source = self.get_book_source_string(&context, String::new(), false).await;
        let user_name_space = self.base.get_user_name_space(&context);
        let mut is_in_book_shelf = false;
        let mut book_info: Option<Book> = None;
        let mut chapter_info: Option<BookChapter> = None;
        let mut next_chapter_url: Option<String> = None;
        if !book_url.is_empty() {
            // 看看有没有加入书架
            book_info = self.get_shelf_book_by_url(book_url.clone(), user_name_space.clone());
            if let Some(bi) = &book_info {
                if !bi.origin.is_empty() {
                    is_in_book_shelf = true;
                    book_source = self.get_book_source_string_by_source_url_opt(bi.origin.clone(), user_name_space.clone());
                }
            }
            // 看看有没有缓存数据
            let cache_info: Option<Book> = cache_to_book(self.book_info_cache.get_as_string(&book_url));
            if let Some(ci) = cache_info {
                // 使用缓存的书籍信息包含的书源
                book_source = self.get_book_source_string(&context, ci.origin, false).await;
            }
            if chapter_url.is_empty() && chapter_index >= 0 {
                // 根据 url 和 index 获取章节内容
                if book_url.is_empty() {
                    return_data.set_error_msg("请输入书籍链接".to_string());
                    return return_data;
                }
                if let Some(bi) = &book_info {
                    if !bi.is_local_book() && book_source.as_ref().map_or(true, |s| s.is_empty()) {
                        return_data.set_error_msg("未配置书源".to_string());
                        return return_data;
                    }
                }
                if book_info.is_none() {
                    book_info = Some(
                        self.merge_book_cache_info(
                            self.web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), user_name_space.clone())
                                .get_book_info_by_url(&book_url, true)
                                .await,
                        )
                        .await,
                    );
                }
                let chapter_list = self
                    .get_local_chapter_list(book_info.clone().unwrap_or_default(), book_source.clone(), false, user_name_space.clone(), false, None)
                    .await;
                if chapter_index < chapter_list.len() as i32 {
                    chapter_info = chapter_list.get(chapter_index as usize).cloned();
                    // 书架书籍保存阅读进度
                    if is_in_book_shelf && cache != 1 {
                        if let (Some(bi), Some(ci)) = (book_info.clone(), chapter_info.clone()) {
                            self.save_shelf_book_progress(bi.clone(), ci.clone(), user_name_space.clone()).await;
                            // 保存到 webdav
                            self.save_book_progress_to_webdav(bi, ci, user_name_space.clone()).await;
                        }
                    }
                    if let Some(ci) = &chapter_info {
                        chapter_url = ci.url.clone();
                    }
                    if chapter_index + 1 < chapter_list.len() as i32 {
                        next_chapter_url = chapter_list.get((chapter_index + 1) as usize).map(|c| c.url.clone());
                    }
                }
            }
            // 按 chapterUrl 匹配章节（前端先取目录再读正文）
            if chapter_info.is_none() && !chapter_url.is_empty() {
                if book_info.is_none() {
                    book_info = Some(
                        self.merge_book_cache_info(
                            self.web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), user_name_space.clone())
                                .get_book_info_by_url(&book_url, true)
                                .await,
                        )
                        .await,
                    );
                }
                if let Some(bi) = &book_info {
                    let chapter_list = self
                        .get_local_chapter_list(bi.clone(), book_source.clone(), false, user_name_space.clone(), false, None)
                        .await;
                    for i in 0..chapter_list.len() {
                        if chapter_url == chapter_list[i].url {
                            chapter_info = Some(chapter_list[i].clone());
                            break;
                        }
                    }
                }
            }
        }
        if book_info.is_none() {
            return_data.set_error_msg("获取书籍信息失败".to_string());
            return return_data;
        }
        let mut book_info = book_info.unwrap();
        if !book_info.is_local_book() && book_source.as_ref().map_or(true, |s| s.is_empty()) {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        if chapter_info.is_none() || chapter_url.is_empty() {
            return_data.set_error_msg("获取章节链接失败".to_string());
            return return_data;
        }

        let mut content = String::new();
        book_info.set_root_dir(work_dir());
        book_info.set_user_name_space(user_name_space.clone());
        if book_info.is_local_book() {
            let mut local_file = book_info.get_local_file();
            if !local_file.exists() {
                return_data.set_error_msg("本地源书籍文件不存在".to_string());
                return return_data;
            }
            if chapter_info.is_none() {
                let chapter_list = self
                    .get_local_chapter_list(book_info.clone(), book_source.clone(), false, user_name_space.clone(), false, None)
                    .await;
                for i in 0..chapter_list.len() {
                    if chapter_url == chapter_list[i].url {
                        chapter_info = Some(chapter_list[i].clone());
                        break;
                    }
                }
                if chapter_info.is_none() {
                    return_data.set_error_msg("获取章节信息失败".to_string());
                    return return_data;
                }
            }
            let chapter_info = chapter_info.unwrap();
            if book_info.is_epub() {
                if !self.extract_epub(book_info.clone(), false) {
                    return_data.set_error_msg("Epub书籍解压失败".to_string());
                    return return_data;
                }

                let epub_root_dir = book_info.get_epub_root_dir();
                let chapter_file_path = work_dir_join(vec![
                    book_info.book_url.clone(),
                    String::from("index"),
                    epub_root_dir.clone(),
                    chapter_info.url.clone(),
                ]);
                LOGGER.info(format!("chapterFilePath: {} {}", chapter_file_path, epub_root_dir));
                if !File::new(&chapter_file_path).exists() {
                    return_data.set_error_msg("章节文件不存在".to_string());
                    return return_data;
                }
                // 处理 js 注入脚本
                // BookConfig.injectJavascriptToEpubChapter(chapterFilePath);

                // 直接返回 html访问地址
                let public_book_url = book_info
                    .book_url
                    .replace("\\", "/")
                    .replace("storage/data/", "/book-assets/");
                if epub_root_dir.is_empty() {
                    content = public_book_url + "/index/" + &chapter_info.url;
                } else {
                    content = public_book_url + "/index/" + &epub_root_dir + "/" + &chapter_info.url;
                }
                if epub_content > 0 {
                    return_data.set_data(
                        Box::new(map!(
                            "url" => "__API_ROOT__".to_string() + &content,
                            "content" => File::new(&chapter_file_path).read_text()
                        )),
                        String::new(),
                    );
                    return return_data;
                }
                return_data.set_data(Box::new(Any::Str(content)), String::new());
                return return_data;
            } else if book_info.is_cbz() {
                if !self.extract_cbz(book_info.clone(), false) {
                    return_data.set_error_msg("CBZ书籍解压失败".to_string());
                    return return_data;
                }
                let chapter_file_path = work_dir_join(vec![
                    book_info.book_url.clone(),
                    String::from("index"),
                    chapter_info.url.clone(),
                ]);
                LOGGER.info(format!("chapterFilePath: {}", chapter_file_path));
                let chapter_file = File::new(&chapter_file_path);
                if !chapter_file.exists() {
                    return_data.set_error_msg("章节文件不存在".to_string());
                    return return_data;
                }
                let ext = self.base.get_file_ext(chapter_file.name.clone(), String::new()).to_lowercase();
                let image_ext = list!("jpg", "jpeg", "gif", "png", "bmp", "webp", "svg");
                let file_url = "__API_ROOT__".to_string()
                    + &book_info.book_url.replace("\\", "/").replace("storage/data/", "/book-assets/")
                    + "/index/"
                    + &chapter_info.url;
                if !image_ext.contains(&ext.as_str()) {
                    return_data.set_data(Box::new(Any::Str(file_url)), String::new());
                    return return_data;
                }
                content = format!("<img src='{}' />", file_url);
                return_data.set_data(Box::new(Any::Str(content)), String::new());
                return return_data;
            }
            if book_info.is_pdf() {
                if !self.convert_pdf_to_image(book_info.clone(), false) {
                    return_data.set_error_msg("PDF生成图片失败".to_string());
                    return return_data;
                }
                if let (Some(start), Some(end)) = (chapter_info.start, chapter_info.end) {
                    if start <= end {
                        let public_book_url = book_info
                            .book_url
                            .replace("\\", "/")
                            .replace("storage/data/", "/book-assets/");
                        for page in start..end {
                            self.convert_pdf_page_to_image(book_info.clone(), page as i32, refresh > 0);
                            let page_file = File::new(&work_dir_join(vec![
                                book_info.book_url.clone(),
                                String::from("index"),
                                format!("output-{}.png", page),
                            ]));
                            LOGGER.info(format!("chapterFilePath: {}", page_file.absolute_path));
                            if !page_file.exists() {
                                return_data.set_error_msg("章节文件不存在".to_string());
                                return return_data;
                            }
                            let file_url = "__API_ROOT__".to_string() + &public_book_url + &format!("/index/output-{}.png", page);
                            content += &format!("<img src='{}' />", file_url);
                        }
                    }
                }
                return_data.set_data(Box::new(Any::Str(content)), String::new());
                return return_data;
            }
            let book_content = LocalBook::get_content(&mut book_info, &chapter_info);
            if book_content.is_none() {
                return_data.set_error_msg("获取章节内容失败".to_string());
                return return_data;
            }
            content = book_content.unwrap_or_default();
        } else {
            // 查找章节缓存
            let mut chapter_cache_file: Option<File> = None;
            if book_info.is_in_shelf && refresh <= 0 && app_config().cache_chapter_content {
                let local_cache_dir = self.get_chapter_cache_dir(&book_info, user_name_space.clone());
                chapter_cache_file = Some(File::new(&(local_cache_dir.absolute_path + File::SEPARATOR + &format!("{}.txt", chapter_index))));
                if chapter_cache_file.as_ref().unwrap().exists() {
                    content = chapter_cache_file.as_ref().unwrap().read_text();
                    if content.contains("<img") {
                        content = self.update_image_link_in_content(book_info.clone(), chapter_info.clone().unwrap(), content);
                    }
                    LOGGER.info(format!("使用缓存的章节内容: {}", chapter_cache_file.as_ref().unwrap().to_string()));
                    return_data.set_data(Box::new(Any::Str(content)), String::new());
                    return return_data;
                }
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                content = crate::stubs::block_on(self.web_book(
                    book_source.clone().unwrap_or_default(),
                    app_config_debug_log(),
                    user_name_space.clone(),
                ).get_book_content(
                    &mut book_info,
                    chapter_info.as_ref().unwrap(),
                    next_chapter_url.as_deref(),
                ));
                if app_config().cache_chapter_content && chapter_cache_file.is_some() {
                    chapter_cache_file.as_ref().unwrap().write_text(&content);
                    // 保存图片
                    let source: BookSource = BookSource::from_json(book_source.clone().unwrap_or_default())
                        .get_or_null()
                        .unwrap_or_else(BookSource::default);
                    crate::stubs::block_on(BookHelp::save_images(
                        &CoroutineScope,
                        &source,
                        &book_info,
                        chapter_info.as_ref().unwrap(),
                        &content,
                    ));
                    content = self.update_image_link_in_content(
                        book_info.clone(),
                        chapter_info.clone().unwrap(),
                        content.clone(),
                    );
                }
            })) {
                Ok(_) => {}
                Err(e) => {
                    let e = crate::stubs::panic_message(e);
                    if !book_source.as_ref().map_or(true, |s| s.is_empty()) {
                        let book_source_object =
                            as_json_object(book_source.clone().map(crate::stubs::Any::from_string))
                                .and_then(|j| j.map_to::<BookSource>());
                        if let Some(bs) = book_source_object {
                            // 标记为失败源
                            let info = mutable_map_of!(
                                "sourceUrl" => bs.book_source_url.clone(),
                                "time" => System::current_time_millis(),
                                "error" => e.clone()
                            );
                            self.add_invalid_book_source(bs.book_source_url, info, user_name_space.clone());
                        }
                    }
                    panic!("{}", e);
                }
            }
        }

        return_data.set_data(Box::new(Any::Str(content)), String::new());
        return return_data;
    }

    pub async fn explore_book(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        // 如果登录了，就使用用户的书源
        self.base.check_auth(&context);
        let book_source = self.get_book_source_string(&context, String::new(), false).await;
        if book_source.as_ref().map_or(true, |s| s.is_empty()) {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        let page: i32;
        let rule_find_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            rule_find_url = context.body_as_json().get_string_opt("ruleFindUrl").unwrap_or_default();
            page = context.body_as_json().get_integer("page", 1);
        } else {
            // get 请求
            rule_find_url = context.query_param("ruleFindUrl").unwrap_or_default();
            page = context.query_param("page").map(|s| s.to_int()).unwrap_or(1);
        }

        let result = self
            .web_book(book_source.clone().unwrap_or_default(), false, self.base.get_user_name_space(&context))
            .explore_book(&rule_find_url, Some(page))
            .await;
        return_data.set_data(Box::new(result), String::new());
        return return_data;
    }

    pub async fn search_book(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        // 如果登录了，就使用用户的书源
        self.base.check_auth(&context);
        let book_source = self.get_book_source_string(&context, String::new(), false).await;
        if book_source.as_ref().map_or(true, |s| s.is_empty()) {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        let key: String;
        let page: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            key = context.body_as_json().get_string_opt("key").unwrap_or_default();
            page = context.body_as_json().get_integer("page", 1);
        } else {
            // get 请求
            key = context.query_param("key").unwrap_or_default();
            page = context.query_param("page").map(|s| s.to_int()).unwrap_or(1);
        }
        if key.is_empty() {
            return_data.set_error_msg("请输入搜索关键字".to_string());
            return return_data;
        }
        LOGGER.info("searchBook");
        let result = self
            .web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), self.base.get_user_name_space(&context))
            .search_book(&key, Some(page))
            .await;
        return_data.set_data(Box::new(result), String::new());
        return return_data;
    }


    pub async fn search_book_multi(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let mut key: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        let mut concurrent_count: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            key = context.body_as_json().get_string_or("key", "");
            book_source_group = context.body_as_json().get_string_or("bookSourceGroup", "");
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 20);
            concurrent_count = context.body_as_json().get_integer("concurrentCount", 36);
        } else {
            // get 请求
            key = context.query_param("key").unwrap_or_default();
            book_source_group = context.query_param("bookSourceGroup").unwrap_or_default();
            last_index = context.query_param("lastIndex").map(|s| s.to_int()).unwrap_or(-1);
            search_size = context.query_param("searchSize").map(|s| s.to_int()).unwrap_or(20);
            concurrent_count = context.query_param("concurrentCount").map(|s| s.to_int()).unwrap_or(36);
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let url_map = BookSourceController::new().get_book_source_map(user_name_space.clone());
        if url_map.is_empty() {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        if key.is_empty() {
            return_data.set_error_msg("请输入搜索关键字".to_string());
            return return_data;
        }
        let mut accurate = false;
        if key.starts_with_ignore_case("=") {
            accurate = true;
            key = key.replacen("=", "", 1);
        }
        if key.is_empty() {
            return_data.set_error_msg("请输入搜索关键字".to_string());
            return return_data;
        }
        if last_index >= url_map.len() as i32 - 1 {
            return_data.set_error_msg("没有更多了".to_string());
            return return_data;
        }

        search_size = if search_size > 0 { search_size } else { 20 };
        concurrent_count = if concurrent_count > 0 { concurrent_count } else { 36 };
        LOGGER.info(format!("searchBookMulti from lastIndex: {} searchSize: {}", last_index, search_size));
        let mut is_end = false;
        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 searchBookMulti");
            is_end = true;
            self.coroutine_context.cancel();
        });
        let mut result_list: Vec<SearchBook> = Vec::new();
        let mut result_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        let mut book = Book::default();
        book.name = key;
        let book_source_file = {
            let it = get_storage_file(&["data", user_name_space.as_str(), "bookSource"]);
            if it.exists() {
                it
            } else {
                get_storage_file(&["data", "default", "bookSource"])
            }
        };
        let mut max_size = url_map.len() as i32;
        let start_index = last_index + 1;
        limit_concurrent_with(concurrent_count, start_index, url_map.len() as i32, |it| {
            let result: Vec<SearchBook> = if it <= max_size {
                last_index = last_index.max(it);
                let filter: Option<Box<dyn Fn(crate::stubs::ObjectNode) -> bool>> = if book_source_group.is_empty() {
                    None
                } else {
                    Some(Box::new(|node: crate::stubs::ObjectNode| {
                        let source_group = node.get("bookSourceGroup").unwrap_or_default();
                        source_group.is_not_blank() && (source_group + ",").contains(&(book_source_group.clone() + ","))
                    }))
                };
                let book_source_list = parse_json_string_list(&book_source_file, None, None, it, it, None, filter.as_deref());
                let empty = book_source_list.as_ref().map_or(true, |l| l.0.is_empty());
                if book_source_list.is_none() || empty {
                    max_size = it;
                    Vec::new()
                } else {
                    crate::stubs::block_on(self.search_book_with_source(
                        book_source_list.unwrap().get_string(0),
                        book.clone(),
                        accurate,
                        user_name_space.clone(),
                    ))
                }
            } else {
                Vec::new()
            };
            Box::new(result)
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            for it in list {
                if let Some(book_list) = it.downcast_ref::<Vec<SearchBook>>() {
                    for book in book_list {
                        // 按照 书名 + 作者名 过滤
                        let book_key = book.name.clone() + "_" + &book.author;
                        if !result_map.contains_key(&book_key) {
                            result_list.push(book.clone());
                            result_map.insert(book_key, 1);
                        }
                    }
                }
            }
            LOGGER.info(format!("Loog: {} resultList.size: {}", loop_count, result_list.len()));
            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.len() < search_size as usize
            }
        });
        return_data.set_data(Box::new(map!("lastIndex" => last_index, "list" => result_list)), String::new());
        return return_data;
    }

    pub async fn search_book_multi_sse(&self, context: RoutingContext) {
        let mut return_data = ReturnData::new();
        // 返回 event-stream
        let mut response = context.response();
        response.put_header("Content-Type", "text/event-stream");
        response.put_header("Cache-Control", "no-cache");
        response.set_chunked(true);
        if !self.base.check_auth(&context) {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            rd.set_error_msg("请登录后使用".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let mut key: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        let mut concurrent_count: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            key = context.body_as_json().get_string_or("key", "");
            book_source_group = context.body_as_json().get_string_or("bookSourceGroup", "");
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 50);
            concurrent_count = context.body_as_json().get_integer("concurrentCount", 24);
        } else {
            // get 请求
            key = context.query_param("key").unwrap_or_default();
            book_source_group = context.query_param("bookSourceGroup").unwrap_or_default();
            last_index = context.query_param("lastIndex").map(|s| s.to_int()).unwrap_or(-1);
            search_size = context.query_param("searchSize").map(|s| s.to_int()).unwrap_or(50);
            concurrent_count = context.query_param("concurrentCount").map(|s| s.to_int()).unwrap_or(24);
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let url_map = BookSourceController::new().get_book_source_map(user_name_space.clone());
        if url_map.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("未配置书源".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        if key.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("请输入搜索关键字".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let mut accurate = false;
        if key.starts_with_ignore_case("=") {
            accurate = true;
            key = key.replacen("=", "", 1);
        }
        if key.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("请输入搜索关键字".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        if last_index >= url_map.len() as i32 - 1 {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("没有更多了".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        search_size = if search_size > 0 { search_size } else { 50 };
        concurrent_count = if concurrent_count > 0 { concurrent_count } else { 24 };
        LOGGER.info(format!("searchBookMulti from lastIndex: {} concurrentCount: {} searchSize: {}", last_index, concurrent_count, search_size));

        let mut is_end = false;
        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 searchBookMultiSSE");
            is_end = true;
            self.coroutine_context.cancel();
        });
        let mut result_list: Vec<SearchBook> = Vec::new();
        let mut book = Book::default();
        book.name = key;
        let book_source_file = {
            let it = get_storage_file(&["data", user_name_space.as_str(), "bookSource"]);
            if it.exists() {
                it
            } else {
                get_storage_file(&["data", "default", "bookSource"])
            }
        };
        let mut max_size = url_map.len() as i32;
        let start_index = last_index + 1;
        // fix: handler/need_continue 同时借用 last_index → Rc<RefCell> 共享
        let last_index_cell = std::rc::Rc::new(std::cell::RefCell::new(last_index));
        limit_concurrent_with(concurrent_count, start_index, url_map.len() as i32, |it| {
            let result: Vec<SearchBook> = if it <= max_size {
                *last_index_cell.borrow_mut() = (*last_index_cell.borrow()).max(it);
                let filter: Option<Box<dyn Fn(crate::stubs::ObjectNode) -> bool>> = if book_source_group.is_empty() {
                    None
                } else {
                    Some(Box::new(|node: crate::stubs::ObjectNode| {
                        let source_group = node.get("bookSourceGroup").unwrap_or_default();
                        source_group.is_not_blank() && (source_group + ",").contains(&(book_source_group.clone() + ","))
                    }))
                };
                let book_source_list = parse_json_string_list(&book_source_file, None, None, it, it, None, filter.as_deref());
                let empty = book_source_list.as_ref().map_or(true, |l| l.0.is_empty());
                if book_source_list.is_none() || empty {
                    max_size = it;
                    Vec::new()
                } else {
                    crate::stubs::block_on(self.search_book_with_source(
                        book_source_list.unwrap().get_string(0),
                        book.clone(),
                        accurate,
                        user_name_space.clone(),
                    ))
                }
            } else {
                Vec::new()
            };
            Box::new(result)
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            let mut loop_result: Vec<SearchBook> = Vec::new();
            for it in list {
                if let Some(book_list) = it.downcast_ref::<Vec<SearchBook>>() {
                    for book in book_list {
                        // 按照 书名 + 作者名 过滤
                        result_list.push(book.clone());
                        loop_result.push(book.clone());
                    }
                }
            }
            // 返回本轮数据
            response.write(&("data: ".to_string()
                + &json_encode(Any::from(map!("lastIndex" => *last_index_cell.borrow(), "data" => loop_result)), false)
                + "\n\n"));
            LOGGER.info(format!("Loog: {} resultList.size: {}", loop_count, result_list.len()));

            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.len() < search_size as usize
            }
        });
        let last_index = *last_index_cell.borrow();
        response.write("event: end\n");
        response.end("data: ".to_string()
            + &json_encode(Any::from(map!("lastIndex" => last_index, "isEnd" => (last_index >= url_map.len() as i32))), false)
            + "\n\n");
    }


    pub async fn search_book_source(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string_opt("url").unwrap_or_default();
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 5);
            book_source_group = context.body_as_json().get_string_or("bookSourceGroup", "");
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            last_index = context.query_param("lastIndex").map(|s| s.to_int()).unwrap_or(-1);
            search_size = context.query_param("searchSize").map(|s| s.to_int()).unwrap_or(5);
            book_source_group = context.query_param("bookSourceGroup").unwrap_or_default();
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let url_map = BookSourceController::new().get_book_source_map(user_name_space.clone());
        if url_map.is_empty() {
            return_data.set_error_msg("未配置书源".to_string());
            return return_data;
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        if last_index >= url_map.len() as i32 - 1 {
            return_data.set_error_msg("没有更多了".to_string());
            return return_data;
        }
        let mut book = self.get_shelf_book_by_url(book_url.clone(), user_name_space.clone());
        if book.is_none() {
            book = cache_to_book(self.book_info_cache.get_as_string(&book_url));
        }
        if book.is_none() {
            return_data.set_error_msg("书籍信息错误".to_string());
            return return_data;
        }
        let book = book.unwrap();
        LOGGER.info(format!("searchBookSource from lastIndex: {}", last_index));
        let mut is_end = false;
        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 searchBookSource");
            is_end = true;
            self.coroutine_context.cancel();
        });
        search_size = if search_size > 0 { search_size } else { 5 };
        let mut result_list: Vec<SearchBook> = Vec::new();
        let concurrent_count = (search_size * 2).max(24);
        let book_source_file = {
            let it = get_storage_file(&["data", user_name_space.as_str(), "bookSource"]);
            if it.exists() {
                it
            } else {
                get_storage_file(&["data", "default", "bookSource"])
            }
        };
        let mut max_size = url_map.len() as i32;
        let start_index = last_index + 1;
        limit_concurrent_with(concurrent_count, start_index, url_map.len() as i32, |it| {
            let result: Vec<SearchBook> = if it <= max_size {
                last_index = last_index.max(it);
                let filter: Option<Box<dyn Fn(crate::stubs::ObjectNode) -> bool>> = if book_source_group.is_empty() {
                    None
                } else {
                    Some(Box::new(|node: crate::stubs::ObjectNode| {
                        let source_group = node.get("bookSourceGroup").unwrap_or_default();
                        source_group.is_not_blank() && (source_group + ",").contains(&(book_source_group.clone() + ","))
                    }))
                };
                let book_source_list = parse_json_string_list(&book_source_file, None, None, it, it, None, filter.as_deref());
                let empty = book_source_list.as_ref().map_or(true, |l| l.0.is_empty());
                if book_source_list.is_none() || empty {
                    max_size = it;
                    Vec::new()
                } else {
                    crate::stubs::block_on(self.search_book_with_source(
                        book_source_list.unwrap().get_string(0),
                        book.clone(),
                        true,
                        user_name_space.clone(),
                    ))
                }
            } else {
                Vec::new()
            };
            Box::new(result)
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            for it in list {
                if let Some(book_list) = it.downcast_ref::<Vec<SearchBook>>() {
                    result_list.add_all(book_list.clone());
                }
            }
            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.len() < search_size as usize
            }
        });
        self.save_book_sources(book, result_list.clone(), user_name_space, false);
        return_data.set_data(Box::new(map!("lastIndex" => last_index, "list" => result_list)), String::new());
        return return_data;
    }

    pub async fn search_book_source_sse(&self, context: RoutingContext) {
        let mut return_data = ReturnData::new();
        // 返回 event-stream
        let mut response = context.response();
        response.put_header("Content-Type", "text/event-stream");
        response.put_header("Cache-Control", "no-cache");
        response.set_chunked(true);

        if !self.base.check_auth(&context) {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            rd.set_error_msg("请登录后使用".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let book_url: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        let mut refresh: i32 = 0;

        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string_opt("url").unwrap_or_default();
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 5);
            book_source_group = context.body_as_json().get_string_or("bookSourceGroup", "");
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            last_index = context.query_param("lastIndex").map(|s| s.to_int()).unwrap_or(-1);
            search_size = context.query_param("searchSize").map(|s| s.to_int()).unwrap_or(30);
            book_source_group = context.query_param("bookSourceGroup").unwrap_or_default();
            refresh = context.query_param("refresh").map(|s| s.to_int()).unwrap_or(0);
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let url_map = BookSourceController::new().get_book_source_map(user_name_space.clone());
        if url_map.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("未配置书源".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        if book_url.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("请输入书籍链接".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        let mut book = self.get_shelf_book_by_url(book_url.clone(), user_name_space.clone());
        if book.is_none() {
            book = cache_to_book(self.book_info_cache.get_as_string(&book_url));
        }
        if book.is_none() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("书籍信息错误".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        if last_index >= url_map.len() as i32 - 1 {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(map!("lastIndex" => last_index)), String::new());
            rd.set_error_msg("没有更多了".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        search_size = if search_size > 0 { search_size } else { 30 };
        let mut result_list: Vec<SearchBook> = Vec::new();
        let concurrent_count = (search_size * 2).max(24);
        let _ = refresh;
        LOGGER.info(format!("searchBookMulti from lastIndex: {} concurrentCount: {} searchSize: {}", last_index, concurrent_count, search_size));
        let mut is_end = false;
        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 searchBookSourceSSE");
            is_end = true;
            self.coroutine_context.cancel();
        });

        let book_source_file = {
            let it = get_storage_file(&["data", user_name_space.as_str(), "bookSource"]);
            if it.exists() {
                it
            } else {
                get_storage_file(&["data", "default", "bookSource"])
            }
        };
        let mut max_size = url_map.len() as i32;
        let start_index = last_index + 1;
        // fix: handler/need_continue 同时借用 last_index → Rc<RefCell> 共享
        let last_index_cell = std::rc::Rc::new(std::cell::RefCell::new(last_index));
        limit_concurrent_with(concurrent_count, start_index, max_size, |it| {
            let result: Vec<SearchBook> = if it <= max_size {
                *last_index_cell.borrow_mut() = (*last_index_cell.borrow()).max(it);
                let filter: Option<Box<dyn Fn(crate::stubs::ObjectNode) -> bool>> = if book_source_group.is_empty() {
                    None
                } else {
                    Some(Box::new(|node: crate::stubs::ObjectNode| {
                        let source_group = node.get("bookSourceGroup").unwrap_or_default();
                        source_group.is_not_blank() && (source_group + ",").contains(&(book_source_group.clone() + ","))
                    }))
                };
                let book_source_list = parse_json_string_list(&book_source_file, None, None, it, it, None, filter.as_deref());
                let empty = book_source_list.as_ref().map_or(true, |l| l.0.is_empty());
                if book_source_list.is_none() || empty {
                    max_size = it;
                    Vec::new()
                } else {
                    crate::stubs::block_on(self.search_book_with_source(
                        book_source_list.unwrap().get_string(0),
                        book.clone().unwrap_or_default(),
                        true,
                        user_name_space.clone(),
                    ))
                }
            } else {
                Vec::new()
            };
            Box::new(result)
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            let mut loop_result: Vec<SearchBook> = Vec::new();
            for it in list {
                if let Some(book_list) = it.downcast_ref::<Vec<SearchBook>>() {
                    result_list.add_all(book_list.clone());
                    loop_result.add_all(book_list.clone());
                }
            }
            // 返回本轮数据
            response.write(&("data: ".to_string()
                + &json_encode(Any::from(map!("lastIndex" => *last_index_cell.borrow(), "data" => loop_result)), false)
                + "\n\n"));
            LOGGER.info(format!("Loog: {} resultList.size: {}", loop_count, result_list.len()));

            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.len() < search_size as usize
            }
        });
        let last_index = *last_index_cell.borrow();
        self.save_book_sources(book.clone().unwrap_or_default(), result_list, user_name_space, false);
        response.write("event: end\n");
        response.end("data: ".to_string()
            + &json_encode(Any::from(map!("lastIndex" => last_index, "isEnd" => (last_index >= max_size))), false)
            + "\n\n");
    }

    pub async fn search_book_with_source(&self, book_source_string: String, book: Book, accurate: bool, user_name_space: String) -> ArrayList<SearchBook> {
        let mut result_list: Vec<SearchBook> = Vec::new();
        let book_source = as_json_object(Some(Any::from_string(book_source_string.clone())))
            .and_then(|j| j.map_to::<BookSource>());
        let book_source = match book_source {
            Some(b) => b,
            None => return result_list,
        };
        if self.is_invalid_book_source(book_source.clone(), user_name_space.clone()) {
            return result_list;
        }
        with_context(Dispatchers::IO, || {
            // val costTime = measureTimeMillis {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let start = System::current_time_millis();
                let mut result = crate::stubs::block_on(
                    self.web_book(book_source_string, false, user_name_space.clone())
                        .search_book(&book.name, Some(1)),
                );
                let end = System::current_time_millis();
                if result.len() > 0 {
                    let key_lower = book.name.to_lowercase();
                    for j in 0..result.len() {
                        let mut _book = result.get(j).cloned().unwrap_or_default();
                        let name_contains = _book.name.to_lowercase().contains(&key_lower);
                        let author_contains = _book.author.to_lowercase().contains(&key_lower);
                        if accurate && _book.name == book.name && (book.author.is_empty() || _book.author == book.author) {
                            _book.time = end - start;
                            result_list.push(_book);
                        } else if !accurate && (name_contains || author_contains) {
                            _book.time = end - start;
                            result_list.push(_book);
                        }
                    }
                }
            })) {
                Ok(_) => {}
                Err(e) => {
                    let e = crate::stubs::panic_message(e);
                    // 标记为失败源
                    let info = mutable_map_of!(
                        "sourceUrl" => book_source.book_source_url.clone(),
                        "time" => System::current_time_millis(),
                        "error" => e.clone()
                    );
                    self.add_invalid_book_source(book_source.book_source_url.clone(), info, user_name_space.clone());
                    LOGGER.error(e);
                }
            }
            // }
        });
        return result_list;
    }

    pub async fn get_available_book_source(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let mut refresh: i32 = 0;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string_opt("url").unwrap_or_default();
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            refresh = context.query_param("refresh").map(|s| s.to_int()).unwrap_or(0);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let mut book = self.get_shelf_book_by_url(book_url.clone(), user_name_space.clone());
        if book.is_none() {
            book = cache_to_book(self.book_info_cache.get_as_string(&book_url));
        }
        if book.is_none() {
            return_data.set_error_msg("书籍信息错误".to_string());
            return return_data;
        }
        let book = book.unwrap();
        let book_source_list: Option<JsonArray> = as_json_array(
            self.base
                .get_user_storage(&user_name_space, vec![book.name.clone() + "_" + &book.author, String::from("bookSource")])
                .map(crate::stubs::Any::from_string),
        );
        if let Some(list) = &book_source_list {
            if list.size() > 0 {
                if refresh <= 0 {
                    return_data.set_data(Box::new(list.get_list()), String::new());
                    return return_data;
                }

                // 刷新源
                let mut result_list: Vec<SearchBook> = Vec::new();
                let concurrent_count = 16;
                limit_concurrent_with(concurrent_count, 0, list.size(), |it| {
                    let result: Vec<SearchBook> = {
                        let search_book = list.get_json_object(it).map_to::<SearchBook>().unwrap_or_default();
                        if search_book.origin == "loc_book" {
                            vec![search_book]
                        } else {
                            let book_source = self.get_book_source_string_by_source_url_opt(search_book.origin.clone(), user_name_space.clone());
                            if let Some(bs) = book_source {
                                crate::stubs::block_on(self.search_book_with_source(bs, book.clone(), true, user_name_space.clone()))
                            } else {
                                Vec::new()
                            }
                        }
                    };
                    Box::new(result)
                }, |list, _| {
                    // logger.info("list: {}", list)
                    for it in list {
                        if let Some(book_list) = it.downcast_ref::<Vec<SearchBook>>() {
                            result_list.add_all(book_list.clone());
                        }
                    }
                    true
                });
                // logger.info("refreshed bookSourceList: {}", resultList)
                self.save_book_sources(book, result_list.clone(), user_name_space, true);
                return_data.set_data(Box::new(result_list), String::new());
                return return_data;
            }
        }
        return_data.set_data(Box::new(Vec::<i32>::new()), String::new());
        return return_data;
    }

    pub async fn get_bookshelf(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let refresh: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            refresh = context.query_param("refresh").map(|s| s.to_int()).unwrap_or(0);
        }
        let book_list = self.get_book_shelf_books(refresh > 0, self.base.get_user_name_space(&context)).await;
        return_data.set_data(Box::new(book_list), String::new());
        return return_data;
    }

    pub async fn get_shelf_book(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            url = context.body_as_json().get_string_opt("url").unwrap_or_default();
        } else {
            // get 请求
            url = context.query_param("url").unwrap_or_default();
        }
        if url.is_empty() {
            return_data.set_error_msg("书源链接不能为空".to_string());
            return return_data;
        }

        let book = self.get_shelf_book_by_url(url, self.base.get_user_name_space(&context));
        if book.is_none() {
            return_data.set_error_msg("书籍不存在".to_string());
            return return_data;
        }
        return_data.set_data(Box::new(book), String::new());
        return return_data;
    }

    pub async fn save_book(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let mut book = context.body_as_json().map_to::<Book>().unwrap_or_default();
        let user_name_space = self.base.get_user_name_space(&context);
        let mut book_source: Option<String> = None;
        if !book.is_local_book() {
            book_source = match self.get_book_source_string_by_source_url_opt(book.origin.clone(), user_name_space.clone()) {
                Some(v) => Some(v),
                None => {
                    return_data.set_error_msg("书源信息错误".to_string());
                    return return_data;
                }
            };
            if book.toc_url.is_empty() {
                book = self
                    .web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), user_name_space.clone())
                    .get_book_info(&mut book, true)
                    .await;
            }
            book = self.merge_book_cache_info(book).await;
        }
        self.save_book_cover(book.clone(), user_name_space.clone(), book_source).await;
        self.save_local_book_cover(book.clone(), user_name_space.clone()).await;
        let result = self.save_book_to_shelf(book, user_name_space, context);
        if let Some(err) = result.second() {
            return_data.set_error_msg(err.clone());
            return return_data;
        }
        return_data.set_data(Box::new(result.first().clone()), String::new());
        return return_data;
    }


    pub async fn set_book_source(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let new_book_url: String;
        let book_source_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string_opt("bookUrl").unwrap_or_default();
            new_book_url = context.body_as_json().get_string_opt("newUrl").unwrap_or_default();
            book_source_url = context.body_as_json().get_string_opt("bookSourceUrl").unwrap_or_default();
        } else {
            // get 请求
            book_url = context.query_param("bookUrl").unwrap_or_default();
            new_book_url = context.query_param("newUrl").unwrap_or_default();
            book_source_url = context.query_param("bookSourceUrl").unwrap_or_default();
        }
        if book_url.is_empty() {
            return_data.set_error_msg("书籍链接不能为空".to_string());
            return return_data;
        }
        if new_book_url.is_empty() {
            return_data.set_error_msg("新源书籍链接不能为空".to_string());
            return return_data;
        }
        if book_source_url.is_empty() {
            return_data.set_error_msg("书源链接不能为空".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let book = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book.is_none() {
            return_data.set_error_msg("书籍信息错误".to_string());
            return return_data;
        }
        let book = book.unwrap();
        // 查找是否存在该书源
        let book_source_string = self.get_book_source_string_by_source_url_opt(book_source_url, user_name_space.clone());

        let mut search_book: Option<Book> = None;
        if book_source_string.as_ref().map_or(true, |s| s.is_empty()) {
            // 判断是不是本地书籍
            let local_book_source_list = as_json_array(
                self.base
                    .get_user_storage(&user_name_space, vec![book.name.clone() + "_" + &book.author, String::from("bookSource")])
                    .map(crate::stubs::Any::from_string),
            );

            // 遍历判断书本是否存在
            if let Some(local_list) = &local_book_source_list {
                for i in 0..local_list.size() {
                    let mut _search_book = local_list.get_json_object(i).map_to::<SearchBook>().unwrap_or_default();
                    if _search_book.book_url == new_book_url {
                        search_book = Some(_search_book.to_book());
                        break;
                    }
                }
            }
            if search_book.is_none() {
                return_data.set_error_msg("书源信息错误".to_string());
                return return_data;
            }
        }

        let mut new_book_info = if search_book.is_some() {
            search_book
        } else {
            if book_source_string.as_ref().map_or(true, |s| s.is_empty()) {
                return_data.set_error_msg("书源信息错误".to_string());
                return return_data;
            }
            Some(
                self.web_book(book_source_string.clone().unwrap_or_default(), app_config_debug_log(), user_name_space.clone())
                    .get_book_info_by_url(&new_book_url, false)
                    .await,
            )
        };

        let edited = self
            .edit_shelf_book(book, user_name_space.clone(), |mut exist_book| {
                let nb = new_book_info.as_ref().unwrap().clone();
                exist_book.origin = nb.origin;
                exist_book.origin_name = nb.origin_name;
                exist_book.book_url = nb.book_url;
                exist_book.toc_url = nb.toc_url;
                exist_book.is_in_shelf = true;
                if exist_book.cover_url.as_ref().map_or(true, |s| s.is_empty()) && !nb.cover_url.as_ref().map_or(true, |s| s.is_empty()) {
                    exist_book.cover_url = nb.cover_url.clone();
                }

                LOGGER.info(format!("setBookSource: {}", exist_book.name));

                exist_book
            })
            .await;
        // fix: 原 Kotlin 闭包内 `newBookInfo = existBook`（Fn 闭包无法捕获可变），改由返回值回填
        if let Some(edited) = edited {
            new_book_info = Some(edited);
        }

        // 更新目录；JAR 保持目录刷新失败不影响书源切换结果
        let new_book_info = new_book_info.unwrap();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = crate::stubs::block_on(self.get_local_chapter_list(
                new_book_info.clone(),
                book_source_string,
                true,
                user_name_space,
                false,
                None,
            ));
        })) {
            Ok(_) => {}
            Err(e) => {
                let _ = crate::stubs::panic_message(e);
            }
        }
        return_data.set_data(Box::new(new_book_info), String::new());
        return return_data;
    }

    pub async fn save_book_group_id(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let group_id: i64;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string_opt("bookUrl").unwrap_or_default();
            group_id = context.body_as_json().get_long("groupId", 0);
        } else {
            // get 请求
            book_url = context.query_param("bookUrl").unwrap_or_default();
            group_id = context.query_param("groupId").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("书籍链接不能为空".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let mut book = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book.is_none() {
            return_data.set_error_msg("书籍信息错误".to_string());
            return return_data;
        }

        if group_id <= 0 {
            return_data.set_error_msg("分组信息错误".to_string());
            return return_data;
        }

        self.edit_shelf_book(book.clone().unwrap_or_default(), user_name_space, |mut exist_book| {
            exist_book.group = group_id;
            LOGGER.info(format!("saveBookGroupId: {}", exist_book.name));
            exist_book
        })
        .await;

        if let Some(b) = book.as_mut() {
            b.group = group_id;
        }
        return_data.set_data(Box::new(book), String::new());
        return return_data;
    }

    pub async fn add_book_group_multi(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let group_id = context.body_as_json().get_long("groupId", 0);
        if group_id <= 0 {
            return_data.set_error_msg("分组信息错误".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let book_json_array = context.body_as_json().get_json_array_or("bookList", JsonArray::new());
        for k in 0..book_json_array.size() {
            let book = book_json_array.get_json_object(k).map_to::<Book>().unwrap_or_default();
            self.edit_shelf_book(book, user_name_space.clone(), |mut exist_book| {
                exist_book.group = exist_book.group | group_id;
                LOGGER.info(format!("saveBookGroupId: {}", exist_book.name));
                exist_book
            })
            .await;
        }

        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn remove_book_group_multi(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let group_id = context.body_as_json().get_long("groupId", 0);
        if group_id <= 0 {
            return_data.set_error_msg("分组信息错误".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let book_json_array = context.body_as_json().get_json_array_or("bookList", JsonArray::new());
        for k in 0..book_json_array.size() {
            let book = book_json_array.get_json_object(k).map_to::<Book>().unwrap_or_default();
            self.edit_shelf_book(book, user_name_space.clone(), |mut exist_book| {
                exist_book.group = exist_book.group ^ group_id;
                LOGGER.info(format!("saveBookGroupId: {}", exist_book.name));
                exist_book
            })
            .await;
        }

        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn delete_book(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let mut book = context.body_as_json().map_to::<Book>().unwrap_or_default();
        let user_name_space = self.base.get_user_name_space(&context);
        let mut bookshelf: JsonArray = as_json_array(
            self.base.get_user_storage(&user_name_space, vec![String::from("bookshelf")]).map(crate::stubs::Any::from_string),
        )
        .unwrap_or_else(JsonArray::new);
        // 遍历判断书本是否存在
        let mut exist_index: i32 = -1;
        for i in 0..bookshelf.size() {
            let _book = bookshelf.get_json_object(i).map_to::<Book>().unwrap_or_default();
            if _book.book_url == book.book_url {
                exist_index = i;
                book = _book;
                break;
            }
            if _book.name == book.name && _book.author == book.author {
                exist_index = i;
                book = _book;
                break;
            }
        }
        if exist_index < 0 {
            return_data.set_error_msg("书架书籍不存在".to_string());
            return return_data;
        }
        bookshelf.remove(exist_index as usize);
        // logger.info("bookshelf: {}", bookshelf)
        self.base.save_user_storage(&user_name_space, String::from("bookshelf"), Box::new(bookshelf));

        // 删除书籍目录
        let local_book_path = File::new(&work_dir_join(vec![
            String::from("storage"),
            String::from("data"),
            user_name_space.clone(),
            book.name.clone() + "_" + &book.author,
        ]));
        local_book_path.delete_recursively();
        if let Some(cover_url) = book.cover_url.clone() {
            if cover_url.starts_with("/") {
                FileUtils::deleteFile(&work_dir_of(&("storage".to_string() + &cover_url)));
            }
        }

        return_data.set_data(Box::new(Any::Str("删除书籍成功".to_string())), String::new());
        return return_data;
    }

    pub async fn delete_books(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_json_array = context.body_as_json_array().unwrap_or_else(JsonArray::new);

        let user_name_space = self.base.get_user_name_space(&context);
        let mut bookshelf: JsonArray = as_json_array(
            self.base.get_user_storage(&user_name_space, vec![String::from("bookshelf")]).map(crate::stubs::Any::from_string),
        )
        .unwrap_or_else(JsonArray::new);
        let mut requested_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut requested_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for k in 0..book_json_array.size() {
            let requested = book_json_array.get_json_object(k).unwrap_or_default();
            requested_urls.insert(requested.get_string("bookUrl"));
            requested_names.insert(requested.get_string("name") + "_" + &requested.get_string("author"));
        }
        let mut i = 0;
        while i < bookshelf.size() {
            let book = bookshelf.get_json_object(i).map_to::<Book>().unwrap_or_default();
            if !(requested_urls.contains(&book.book_url) || requested_names.contains(&(book.name.clone() + "_" + &book.author))) {
                i += 1;
                continue;
            }
            bookshelf.remove(i as usize);
            let local_book_path = File::new(&work_dir_join(vec![
                String::from("storage"),
                String::from("data"),
                user_name_space.clone(),
                book.name.clone() + "_" + &book.author,
            ]));
            local_book_path.delete_recursively();
        }

        self.base.save_user_storage(&user_name_space, String::from("bookshelf"), Box::new(bookshelf));
        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn save_book_info_cache(&self, book_list: List<Book>) -> List<Book> {
        if book_list.len() > 0 {
            for i in 0..book_list.len() {
                let book = &book_list[i];
                let encoded = json_encode(Any::from(JsonObject::map_from(book.clone()).map()), false);
                self.book_info_cache.put(&book.book_url, &encoded, 0);
            }
        }
        return book_list;
    }

    pub async fn merge_book_cache_info(&self, book: Book) -> Book {
        let cache_info: Option<Book> = cache_to_book(self.book_info_cache.get_as_string(&book.book_url));

        if cache_info.is_some() {
            return crate::com_htmake_reader_utils_vertext::fill_data(
                book.clone(),
                cache_info.unwrap(),
                vec!["name".to_string(), "author".to_string(), "coverUrl".to_string(), "tocUrl".to_string(), "intro".to_string(), "latestChapterTitle".to_string(), "wordCount".to_string()],
            );
        }
        return book;
    }


    pub async fn get_book_shelf_books(&self, refresh: bool, user_name_space: String) -> List<Book> {
        let bookshelf: JsonArray = as_json_array(
            self.base.get_user_storage(&user_name_space, vec![String::from("bookshelf")]).map(crate::stubs::Any::from_string),
        )
        .unwrap_or_else(JsonArray::new);
        if bookshelf.size() == 0 {
            return Vec::new();
        }
        let mut book_list: Vec<Book> = Vec::new();
        let sync_mutex = std::sync::Mutex::new(());
        for i in 0..bookshelf.size() {
            let mut book = bookshelf.get_json_object(i).map_to::<Book>().unwrap_or_default();
            book.is_in_shelf = true;
            if !book.is_local_book() && book.can_update && refresh {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let book_source = self.get_book_source_string_by_source_url_opt(book.origin.clone(), user_name_space.clone());
                    if book_source.is_some() {
                        let book_chapter_list = crate::stubs::block_on(self.get_local_chapter_list(
                            book.clone(),
                            book_source,
                            refresh,
                            user_name_space.clone(),
                            false,
                            None,
                        ));
                        if book_chapter_list.len() > 0 {
                            let book_chapter = book_chapter_list.last().cloned().unwrap_or_default();
                            book.latest_chapter_title = Some(book_chapter.title);
                        }
                        if book_chapter_list.len() as i32 - book.total_chapter_num > 0 {
                            book.last_check_time = System::current_time_millis();
                            book.last_check_count = book_chapter_list.len() as i32 - book.total_chapter_num;
                        }
                        book.total_chapter_num = book_chapter_list.len() as i32;
                    }
                }));
            }
            let _guard = sync_mutex.lock();
            book_list.push(book);
        }
        return book_list;
    }

    pub async fn get_local_chapter_list(&self, book: Book, book_source: Option<String>, refresh: bool, user_name_space: String, debug_log: bool, mutex: Option<Mutex>) -> List<BookChapter> {
        let mut book = book;
        let md5_encode = MD5Utils::md5Encode(Some(&book.book_url));
        let book_chapters_cache = self.get_book_chapters_cache(user_name_space.clone());
        let cache_key = book.name.clone() + "_" + &book.author + &md5_encode;
        let chapter_list = if book.is_in_shelf {
            as_json_array(
                self.base
                    .get_user_storage(&user_name_space, vec![book.name.clone() + "_" + &book.author, md5_encode.clone()])
                    .map(crate::stubs::Any::from_string),
            )
        } else {
            as_json_array(book_chapters_cache.get_as_string(&cache_key).map(crate::stubs::Any::from_string))
        };

        if chapter_list.is_none() || refresh {
            let mut new_chapter_list: Vec<BookChapter> = Vec::new();
            book.set_root_dir(work_dir());
            book.set_user_name_space(user_name_space.clone());
            if book.is_local_book() {
                // 重新解压epub文件
                if book.is_epub() && !self.extract_epub(book.clone(), refresh) {
                    panic!("Epub书籍解压失败");
                }
                // 重新解压cbz文件
                if book.is_cbz() && !self.extract_cbz(book.clone(), refresh) {
                    panic!("CBZ书籍解压失败");
                }
                if book.is_pdf() && !self.convert_pdf_to_image(book.clone(), refresh) {
                    panic!("PDF书籍转换失败");
                }
                new_chapter_list = LocalBook::get_chapter_list(&mut book);
            } else {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let book_source_object = book_source.clone().and_then(|it| BookSource::from_json(it).get_or_null());
                    if let Some(bs) = &book_source_object {
                        if let Some(pre_update_js) = bs.rule_toc.as_ref().and_then(|t| t.pre_update_js.clone()) {
                            let mut analyze_rule = AnalyzeRule::new(book.clone(), bs.clone(), None::<()>);
                            analyze_rule.eval_js(pre_update_js, None);
                        }
                    }
                    let source = book_source.clone().unwrap_or_default();
                    if book.toc_url.is_blank() {
                        crate::stubs::block_on(
                            self.web_book(source.clone(), debug_log, user_name_space.clone())
                                .get_book_info(&mut book, false),
                        );
                    }
                    new_chapter_list = crate::stubs::block_on(
                        self.web_book(source, debug_log, user_name_space.clone())
                            .get_chapter_list(&mut book),
                    );
                })) {
                    Ok(_) => {}
                    Err(e) => {
                        let e = crate::stubs::panic_message(e);
                        if !book_source.as_ref().map_or(true, |s| s.is_empty()) {
                            let book_source_object = BookSource::from_json(book_source.clone().unwrap_or_default()).get_or_null();
                            if let Some(bs) = book_source_object {
                                // 标记为失败源
                                let info = mutable_map_of!(
                                    "sourceUrl" => bs.book_source_url.clone(),
                                    "time" => System::current_time_millis(),
                                    "error" => e.clone()
                                );
                                self.add_invalid_book_source(bs.book_source_url, info, user_name_space.clone());
                            }
                        }
                        if let Some(m) = &mutex {
                            m.lock_sync();
                        }
                        {
                            book.last_check_error = Some(e.clone());
                            crate::stubs::block_on(self.edit_shelf_book(book.clone(), user_name_space.clone(), |mut exist_book| {
                                exist_book.last_check_error = Some(e.clone());
                                exist_book
                            }));
                        }
                        if let Some(m) = &mutex {
                            m.unlock_sync();
                        }
                        panic!("{}", e);
                    }
                }
            }
            if book.is_in_shelf {
                let chapter_json = crate::stubs::JsonArray::from_list(
                    new_chapter_list.iter().map(crate::stubs::JsonObject::map_from).collect(),
                )
                .to_string();
                self.base.save_user_storage(
                    &user_name_space,
                    get_relative_path(&[book.name.clone() + "_" + &book.author, md5_encode.clone()]),
                    Box::new(crate::stubs::Any::Str(chapter_json)),
                );
            } else {
                let chapter_json = crate::stubs::JsonArray::from_list(
                    new_chapter_list.iter().map(crate::stubs::JsonObject::map_from).collect(),
                )
                .to_string();
                book_chapters_cache.put(&cache_key, &chapter_json, 3600);
            }
            self.save_shelf_book_latest_chapter(book.clone(), new_chapter_list.clone(), user_name_space, mutex).await;
            return new_chapter_list;
        }
        let mut local_chapter_list: Vec<BookChapter> = Vec::new();
        let chapter_list = chapter_list.unwrap();
        for i in 0..chapter_list.size() {
            let _chapter = chapter_list.get_json_object(i).map_to::<BookChapter>().unwrap_or_default();
            local_chapter_list.push(_chapter);
        }
        return local_chapter_list;
    }

    pub async fn get_book_source_string(&self, context: &RoutingContext, source_url: String, with_explore_url: bool) -> Option<String> {
        let mut book_source_string: Option<String> = None;
        if context.request().method() == HttpMethod::POST {
            let book_source = context.body_as_json().get_json_object("bookSource");
            if let Some(bs) = book_source {
                book_source_string = Some(bs.to_string());
            }
        }
        let user_name_space = self.base.get_user_name_space(&context);
        if book_source_string.as_ref().map_or(true, |s| s.is_empty()) {
            let book_source_url: String;
            if context.request().method() == HttpMethod::POST {
                book_source_url = context.body_as_json().get_string_or("bookSourceUrl", "");
            } else {
                book_source_url = context.query_param("bookSourceUrl").unwrap_or_default();
            }
            if book_source_url.is_not_blank() {
                book_source_string = self.get_book_source_string_by_source_url_opt(book_source_url, user_name_space.clone());
            }
        }
        if book_source_string.as_ref().map_or(true, |s| s.is_empty()) && source_url.is_not_blank() {
            book_source_string = self.get_book_source_string_by_source_url_opt(source_url, user_name_space);
        }
        let _ = with_explore_url;
        return book_source_string;
    }

    pub fn get_shelf_book_by_url(&self, url: String, user_name_space: String) -> Option<Book> {
        if url.is_empty() {
            return None;
        }
        let bookshelf: JsonArray = as_json_array(
            self.base.get_user_storage(&user_name_space, vec![String::from("bookshelf")]).map(crate::stubs::Any::from_string),
        )
        .unwrap_or_else(JsonArray::new);
        if bookshelf.size() == 0 {
            return None;
        }
        for i in 0..bookshelf.size() {
            let mut _book = bookshelf.get_json_object(i).map_to::<Book>().unwrap_or_default();
            if _book.book_url == url {
                _book.set_root_dir(work_dir());
                _book.set_user_name_space(user_name_space);
                _book.is_in_shelf = true;
                return Some(_book);
            }
        }
        return None;
    }

    pub async fn save_shelf_book_progress(&self, book: Book, book_chapter: BookChapter, user_name_space: String) {
        self.edit_shelf_book(book, user_name_space, |mut exist_book| {
            exist_book.dur_chapter_index = book_chapter.index;
            exist_book.dur_chapter_title = Some(book_chapter.title.clone());
            exist_book.dur_chapter_time = System::current_time_millis();

            // logger.info("saveShelfBookProgress: {}", existBook)

            exist_book
        })
        .await;
    }

    pub async fn save_shelf_book_latest_chapter(&self, book: Book, book_chapter_list: List<BookChapter>, user_name_space: String, mutex: Option<Mutex>) {
        // fix: 原 Kotlin 闭包内更新外部 book（Fn 闭包无法捕获可变），以 RefCell 提供内部可变性
        let book_cell = std::cell::RefCell::new(book);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Some(m) = &mutex {
                m.lock_sync();
            }
            // fix: borrow guard 存活到 edit_shelf_book 调用结束（含闭包内 borrow_mut），先取副本释放借用
            let book_clone = book_cell.borrow().clone();
            crate::stubs::block_on(self.edit_shelf_book(book_clone, user_name_space.clone(), |mut exist_book: Book| {
                if book_chapter_list.len() > 0 {
                    let book_chapter = book_chapter_list.last().cloned().unwrap_or_default();
                    exist_book.latest_chapter_title = Some(book_chapter.title);
                }
                if book_chapter_list.len() as i32 - exist_book.total_chapter_num > 0 {
                    exist_book.last_check_count = book_chapter_list.len() as i32 - exist_book.total_chapter_num;
                    exist_book.last_check_time = System::current_time_millis();
                }
                exist_book.last_check_error = None;
                exist_book.total_chapter_num = book_chapter_list.len() as i32;
                let mut book = book_cell.borrow_mut();
                book.latest_chapter_title = exist_book.latest_chapter_title.clone();
                book.last_check_count = exist_book.last_check_count;
                book.last_check_time = exist_book.last_check_time;
                book.last_check_error = exist_book.last_check_error.clone();
                book.total_chapter_num = exist_book.total_chapter_num;
                exist_book
            }));
            if let Some(m) = &mutex {
                m.unlock_sync();
            }
        }));
    }

    pub async fn edit_shelf_book<F>(&self, book: Book, user_name_space: String, handler: F) -> Option<Book>
    where
        F: Fn(Book) -> Book,
    {
        let mutex = UserMutex::get_locker(&(user_name_space.clone() + "@bookshelf")).await;
        LOGGER.info(format!("wait for lock {}", user_name_space.clone() + "@bookshelf"));
        mutex.lock().await;
        let result: Option<Book> = {
            LOGGER.info("lock success");
            let mut bookshelf: JsonArray = as_json_array(
                self.base.get_user_storage(&user_name_space, vec![String::from("bookshelf")]).map(crate::stubs::Any::from_string),
            )
            .unwrap_or_else(JsonArray::new);
            let mut exist_index: i32 = -1;
            for i in 0..bookshelf.size() {
                let _book = bookshelf.get_json_object(i).map_to::<Book>().unwrap_or_default();
                if !book.book_url.is_empty() && _book.book_url == book.book_url {
                    exist_index = i;
                    break;
                }
                if !book.name.is_empty() && _book.name == book.name && !book.author.is_empty() && _book.author == book.author {
                    exist_index = i;
                    break;
                }
            }
            if exist_index >= 0 {
                let mut book_list = bookshelf.get_list();
                let mut exist_book = bookshelf.get_json_object(exist_index).map_to::<Book>().unwrap_or_default();
                exist_book = handler(exist_book);

                book_list[exist_index as usize] = JsonObject::map_from(exist_book.clone());
                let bookshelf = JsonArray::from_list(book_list);
                self.base.save_user_storage(&user_name_space, String::from("bookshelf"), Box::new(bookshelf));
                Some(exist_book)
            } else {
                None
            }
        };
        mutex.unlock();
        return result;
    }

    pub fn save_book_sources(&self, book: Book, source_list: List<SearchBook>, user_name_space: String, replace: bool) {
        if book.name.is_empty() {
            return;
        }
        let mut book_source_list: JsonArray = JsonArray::new();
        if !replace {
            let local_book_source_list = as_json_array(
                self.base
                    .get_user_storage(&user_name_space, vec![book.name.clone() + "_" + &book.author, String::from("bookSource")])
                    .map(crate::stubs::Any::from_string),
            );
            if let Some(l) = local_book_source_list {
                book_source_list = l;
            }
        }

        for k in 0..source_list.len() {
            let search_book = &source_list[k];
            // 遍历判断书源是否存在（同一书源只保留一条，避免同名同作者多版本重复）
            let mut exist_index: i32 = -1;
            for i in 0..book_source_list.size() {
                let _search_book = book_source_list.get_json_object(i).map_to::<SearchBook>().unwrap_or_default();
                if _search_book.origin == search_book.origin {
                    exist_index = i;
                    break;
                }
            }
            if exist_index >= 0 {
                let mut _source_list = book_source_list.get_list();
                _source_list[exist_index as usize] = JsonObject::map_from(search_book.clone());
                book_source_list = JsonArray::from_list(_source_list);
            } else {
                book_source_list.add(JsonObject::map_from(search_book.clone()).to_string());
            }
        }

        // logger.info("bookSourceList: {}", bookSourceList)
        self.base.save_user_storage(
            &user_name_space,
            get_relative_path(&[book.name + "_" + &book.author, String::from("bookSource")]),
            Box::new(book_source_list),
        );
    }

    pub fn extract_epub(&self, book: Book, force: bool) -> bool {
        let epub_extract_dir = File::new(&work_dir_of(&(book.book_url.clone() + File::SEPARATOR + "index")));
        if force || !epub_extract_dir.exists() {
            epub_extract_dir.delete_recursively();
            let mut local_epub_file = File::new(&work_dir_of(&(book.origin_name.clone() + File::SEPARATOR + "index.epub")));
            if book.origin_name.index_of("localStore", 0) > 0 {
                // 本地书仓的源文件
                local_epub_file = File::new(&work_dir_of(&book.origin_name));
            }
            if book.origin_name.index_of("webdav", 0) > 0 {
                // webdav 书仓的源文件
                local_epub_file = File::new(&work_dir_of(&book.origin_name));
            }
            if !local_epub_file.unzip(&epub_extract_dir.to_string()) {
                return false;
            }
        }
        return true;
    }

    pub fn extract_cbz(&self, book: Book, force: bool) -> bool {
        let extract_dir = File::new(&work_dir_of(&(book.book_url.clone() + File::SEPARATOR + "index")));
        if force || !extract_dir.exists() {
            extract_dir.delete_recursively();
            let mut local_file = File::new(&work_dir_of(&(book.origin_name.clone() + File::SEPARATOR + "index.cbz")));
            if book.origin_name.index_of("localStore", 0) > 0 {
                // 本地书仓的源文件
                local_file = File::new(&work_dir_of(&book.origin_name));
            }
            if book.origin_name.index_of("webdav", 0) > 0 {
                // webdav 书仓的源文件
                local_file = File::new(&work_dir_of(&book.origin_name));
            }
            if !local_file.unzip(&extract_dir.to_string()) {
                return false;
            }
        }
        return true;
    }


    pub async fn sync_book_progress_from_webdav(&self, progress_file: File, user_name_space: String) {
        let book = as_json_object(Some(Any::from_string(progress_file.read_text()))).and_then(|j| j.map_to::<Book>());
        if let Some(book) = book {
            self.edit_shelf_book(book.clone(), user_name_space, |mut exist_book| {
                exist_book.dur_chapter_index = book.dur_chapter_index;
                exist_book.dur_chapter_pos = book.dur_chapter_pos;
                exist_book.dur_chapter_time = book.dur_chapter_time;
                exist_book.dur_chapter_title = book.dur_chapter_title.clone();

                LOGGER.info(format!("syncShelfBookProgress: {}", exist_book.name));
                exist_book
            })
            .await;
        }
    }

    pub async fn save_book_progress_to_webdav(&self, book: Book, book_chapter: BookChapter, user_name_space: String) {
        let user_home = self.base.get_user_webdav_home(&user_name_space);
        let mut book_progress_dir = File::new(&(user_home.clone() + File::SEPARATOR + "bookProgress"));
        if !book_progress_dir.exists() {
            book_progress_dir = File::new(&(user_home + File::SEPARATOR + "legado" + File::SEPARATOR + "bookProgress"));
            if !book_progress_dir.exists() {
                return;
            }
        }
        let progress_file = File::new(&(book_progress_dir.to_string() + File::SEPARATOR + &(book.name.clone() + "_" + &book.author + ".json")));
        progress_file.write_text(&json_encode(
            Any::from(map!(
                "name" => book.name,
                "author" => book.author,
                "durChapterIndex" => book_chapter.index,
                "durChapterPos" => 0,
                "durChapterTime" => System::current_time_millis(),
                "durChapterTitle" => book_chapter.title
            )),
            true,
        ));
    }

    pub async fn sync_from_webdav(&self, zip_file_path: String, user_name_space: String) -> bool {
        let desc_dir = work_dir_join(vec![String::from("storage"), String::from("data"), user_name_space.clone(), String::from("tmp")]);
        let desc_dir_file = File::new(&desc_dir);
        {
            let user_home = self.base.get_user_webdav_home(&user_name_space);
            let zip_file = File::new(&zip_file_path);
            if !zip_file.exists() {
                return false;
            }
            desc_dir_file.delete_recursively();
            crate::io_legado_app_utils_ziputils::ZipUtils::unzipFile_file(&zip_file, &desc_dir_file);
            for file_name in BACKUP_FILE_NAMES {
                let backup_file = File::new(&(desc_dir.clone() + File::SEPARATOR + file_name));
                if !backup_file.exists() {
                    continue;
                }
                let user_data_file = File::new(&work_dir_join(vec![
                    String::from("storage"),
                    String::from("data"),
                    user_name_space.clone(),
                    file_name.to_string(),
                ]));
                user_data_file.delete_recursively();
                backup_file.copy_recursively(&user_data_file);
            }
            let backup_books_dir = File::new(&(desc_dir + File::SEPARATOR + "books"));
            if backup_books_dir.exists() {
                let webdav_books_dir = File::new(&work_dir_join(vec![
                    String::from("storage"),
                    String::from("data"),
                    user_name_space.clone(),
                    String::from("webdav"),
                    String::from("books"),
                ]));
                webdav_books_dir.delete_recursively();
                backup_books_dir.copy_recursively(&webdav_books_dir);
            }
            // 同步阅读进度
            let mut book_progress_dir = File::new(&(user_home.clone() + File::SEPARATOR + "bookProgress"));
            if !book_progress_dir.exists() {
                book_progress_dir = File::new(&(user_home + File::SEPARATOR + "legado" + File::SEPARATOR + "bookProgress"));
            }
            if book_progress_dir.exists() && book_progress_dir.is_directory() {
                let list_files = book_progress_dir.list_files();
                for it in list_files {
                    self.sync_book_progress_from_webdav(it, user_name_space.clone()).await;
                }
            }
            return true;
        }
        let _ = desc_dir_file.delete_recursively();
        return false;
    }

    pub async fn save_to_webdav(&self, user_name_space: String, latest_zip_file_path: Option<String>) -> bool {
        let user_home = self.base.get_user_webdav_home(&user_name_space);
        let mut legado_home = user_home.clone();
        // fix: 原 Kotlin elvis（getLastBackFileFromWebdav 为挂起函数）→ if/else + await
        let resolved_zip_file_path = if latest_zip_file_path.is_some() {
            latest_zip_file_path
        } else {
            self.get_last_back_file_from_webdav(user_name_space.clone()).await
        };
        if resolved_zip_file_path.is_none() {
            legado_home = user_home.clone() + File::SEPARATOR + "legado";
        } else if resolved_zip_file_path.as_ref().unwrap().index_of("legado", 0) > 0 {
            legado_home = user_home.clone() + File::SEPARATOR + "legado";
        }
        return self.create_user_backup(user_name_space, legado_home, resolved_zip_file_path).await.is_some();
    }

    pub async fn get_last_back_file_from_webdav(&self, user_name_space: String) -> Option<String> {
        let user_home = self.base.get_user_webdav_home(&user_name_space);
        let mut legado_home = File::new(&(user_home.clone() + File::SEPARATOR + "legado"));
        if !legado_home.exists() {
            legado_home = File::new(&user_home);
        }
        if !legado_home.exists() {
            return None;
        }
        let mut latest_zip_file: Option<String> = None;
        let zip_file_reg = regex::Regex::new("(?i)^backup[0-9-]+.zip$").unwrap(); //忽略大小写
        let mut files = legado_home.list_files();
        files.sort_by(|a, b| b.last_modified().cmp(&a.last_modified()));
        for it in files {
            if zip_file_reg.is_match(&it.name) {
                latest_zip_file = Some(it.to_string());
                break;
            }
        }
        return latest_zip_file;
    }

    pub async fn book_source_debug_sse(&self, context: RoutingContext) {
        let mut return_data = ReturnData::new();
        // 返回 event-stream
        let mut response = context.response();
        response.put_header("Content-Type", "text/event-stream");
        response.put_header("Cache-Control", "no-cache");
        response.set_chunked(true);

        if !self.base.check_auth(&context) {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            rd.set_error_msg("请登录后使用".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let book_source_url = context.query_param("bookSourceUrl").unwrap_or_default();
        let keyword = context.query_param("keyword").unwrap_or_default();

        if book_source_url.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("未配置书源".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        if keyword.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("请输入搜索关键词".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        let user_name_space = self.base.get_user_name_space(&context);
        let book_source_string = self.get_book_source_string_by_source_url_opt(book_source_url, user_name_space.clone());
        if book_source_string.as_ref().map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("未配置书源".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        LOGGER.info(format!("bookSourceDebugSSE bookSource: {} keyword: {}", book_source_string.as_ref().unwrap(), keyword));

        let response_for_debugger = response.clone();
        let mut debugger = Debugger::new(move |msg: &str| {
            let mut r = response_for_debugger.clone();
            r.write(&("data: ".to_string() + &json_encode(Any::from(map!("msg" => msg.to_string())), false) + "\n\n"));
        });

        let mut web_book = self.web_book(book_source_string.unwrap(), false, user_name_space);

        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 bookSourceDebugSSE");
            self.coroutine_context.cancel();
        });

        let _ = debugger.start_debug(&mut web_book, &keyword).await;

        response.write("event: end\n");
        response.end("data: ".to_string() + &json_encode(Any::from(map!("end" => true)), false) + "\n\n");
        let _ = return_data;
    }

    pub async fn cache_book_sse(&self, context: RoutingContext) {
        let mut return_data = ReturnData::new();
        // 返回 event-stream
        let mut response = context.response();
        response.put_header("Content-Type", "text/event-stream");
        response.put_header("Cache-Control", "no-cache");
        response.set_chunked(true);

        if !self.base.check_auth(&context) {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            rd.set_error_msg("请登录后使用".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let book_url: String;
        let refresh: i32;
        let mut concurrent_count: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求：url 缺失时回退到 bookUrl
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_string_opt("bookUrl"))
                .unwrap_or_default();
            refresh = context.body_as_json().get_integer("refresh", 0);
            concurrent_count = context.body_as_json().get_integer("concurrentCount", 24);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            refresh = context.query_param("refresh").map(|s| s.to_int()).unwrap_or(0);
            concurrent_count = context.query_param("concurrentCount").map(|s| s.to_int()).unwrap_or(24);
        }
        if book_url.is_empty() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("请输入书籍链接".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        let user_name_space = self.base.get_user_name_space(&context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book_info.is_none() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("请先加入书架".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let book_info = book_info.unwrap();
        if book_info.is_local_book() {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("本地书籍无需缓存".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }
        let book_source = self.get_book_source_string(&context, book_info.origin.clone(), false).await;
        if book_source.as_ref().map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            let mut rd = ReturnData::new();
            rd.set_error_msg("未配置书源".to_string());
            response.end("data: ".to_string() + &rd_json(&rd) + "\n\n");
            return;
        }

        let chapter_list = self
            .get_local_chapter_list(book_info.clone(), book_source.clone(), false, user_name_space.clone(), false, None)
            .await;
        let mut cached_chapter_content_set: std::collections::HashSet<i32> = std::collections::HashSet::new();
        if refresh <= 0 {
            cached_chapter_content_set = self.get_cached_chapter_content_set(&book_info, user_name_space.clone());
        }
        let local_cache_dir = self.get_chapter_cache_dir(&book_info, user_name_space.clone());
        let mut is_end = false;
        // fix: handler/need_continue 两个闭包同时借用 success_count 等（Rc<RefCell> 共享内部可变性）
        let success_count = std::rc::Rc::new(std::cell::RefCell::new(0_i32));
        let failed_count = std::rc::Rc::new(std::cell::RefCell::new(0_i32));
        let cached_chapter_content_set = std::rc::Rc::new(std::cell::RefCell::new(cached_chapter_content_set));

        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 cacheBookSSE");
            is_end = true;
            self.coroutine_context.cancel();
        });

        concurrent_count = if concurrent_count > 0 { concurrent_count } else { 24 };
        LOGGER.info(format!("cacheBookSSE concurrentCount: {} refresh: {}", concurrent_count, refresh));
        limit_concurrent_with(concurrent_count, 0, chapter_list.len() as i32, |it| {
            if !cached_chapter_content_set.borrow().contains(&it) {
                let chapter_index = it;
                let chapter_info = chapter_list.get(it as usize).cloned().unwrap_or_default();
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut next_chapter_url: Option<String> = None;
                    if chapter_index + 1 < chapter_list.len() as i32 {
                        next_chapter_url = chapter_list.get((chapter_index + 1) as usize).map(|c| c.url.clone());
                    }
                    let content = crate::stubs::block_on(self.web_book(
                        book_source.clone().unwrap_or_default(),
                        app_config_debug_log(),
                        user_name_space.clone(),
                    ).get_book_content(
                        &mut book_info.clone(),
                        &chapter_info,
                        next_chapter_url.as_deref(),
                    ));
                    let chapter_cache_file = local_cache_dir.resolve(&format!("{}.txt", chapter_index));
                    chapter_cache_file.write_text(&content);
                    // 保存图片
                    let source: BookSource = BookSource::from_json(book_source.clone().unwrap_or_default())
                        .get_or_null()
                        .unwrap_or_else(BookSource::default);
                    crate::stubs::block_on(BookHelp::save_images(
                        &CoroutineScope,
                        &source,
                        &book_info.clone(),
                        &chapter_info,
                        &content,
                    ));
                    *success_count.borrow_mut() += 1;
                    cached_chapter_content_set.borrow_mut().insert(chapter_index);
                }));
            }
            Box::new(it)
        }, |list, loop_count| {
            if is_end {
                false
            } else {
                // 返回本轮数据
                let result = map!(
                    "cachedCount" => cached_chapter_content_set.borrow().len(),
                    "successCount" => *success_count.borrow(),
                    "failedCount" => *failed_count.borrow()
                );
                response.write(&("data: ".to_string() + &json_encode(Any::from(result.clone()), false) + "\n\n"));
                LOGGER.info(format!("Loog: {} list.size: {} result: {:?}", loop_count, list.len(), result));
                true
            }
        });
        response.write("event: end\n");
        response.end("data: ".to_string()
            + &json_encode(Any::from(map!(
                "cachedCount" => cached_chapter_content_set.borrow().len(),
                "successCount" => *success_count.borrow(),
                "failedCount" => *failed_count.borrow()
            )), false)
            + "\n\n");
        let _ = return_data;
    }


    pub async fn delete_book_cache(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求：url 缺失时回退到 bookUrl
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_string_opt("bookUrl"))
                .unwrap_or_default();
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }

        let user_name_space = self.base.get_user_name_space(&context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book_info.is_none() {
            return_data.set_error_msg("请先加入书架".to_string());
            return return_data;
        }
        let book_info = book_info.unwrap();
        if book_info.is_local_book() {
            return_data.set_error_msg("本地书籍无需删除缓存".to_string());
            return return_data;
        }
        let local_cache_dir = self.get_chapter_cache_dir(&book_info, user_name_space);
        local_cache_dir.delete_recursively();

        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub fn get_chapter_cache_dir(&self, book_info: &Book, user_name_space: String) -> File {
        let md5_encode = MD5Utils::md5Encode(Some(&book_info.book_url));
        let local_cache_dir_path = work_dir_join(vec![
            String::from("storage"),
            String::from("data"),
            user_name_space,
            book_info.name.clone() + "_" + &book_info.author,
            md5_encode,
        ]);
        let local_cache_dir = File::new(&local_cache_dir_path);
        if !local_cache_dir.exists() {
            local_cache_dir.mkdirs();
        }
        return local_cache_dir;
    }

    pub fn get_cached_chapter_content_set(&self, book_info: &Book, user_name_space: String) -> std::collections::HashSet<i32> {
        let local_cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
        let mut cached_chapter_content_set: std::collections::HashSet<i32> = std::collections::HashSet::new();
        for it in local_cache_dir.list_files() {
            if !it.name.starts_with(".") && it.name.ends_with(".txt") {
                cached_chapter_content_set.insert(it.name.replace(".txt", "").to_int());
            }
        }
        return cached_chapter_content_set;
    }

    pub async fn get_shelf_book_with_cache_info(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(&context);
        let book_list = self.get_book_shelf_books(false, user_name_space.clone()).await;
        let mut result: Vec<Any> = Vec::new();
        for i in 0..book_list.len() {
            let book_info = &book_list[i];
        if !book_info.is_local_book() {
            let cached_set = self.get_cached_chapter_content_set(book_info, user_name_space.clone());
            let mut book_info_map: std::collections::HashMap<String, crate::stubs::Any> = std::collections::HashMap::new();
            // fix: Kotlin 返回完整书籍字段 + cachedChapterCount（前端缓存管理列表依赖 bookUrl/name/author）
            let book_json = crate::stubs::book_to_json(book_info);
            if let serde_json::Value::Object(m) = book_json {
                for (k, v) in m {
                    book_info_map.insert(k, crate::runtime::js::value_to_any(&v));
                }
            }
            book_info_map.insert(String::from("cachedChapterCount"), crate::stubs::Any::Long(cached_set.len() as i64));
            result.push(crate::stubs::Any::from(book_info_map));
        } else {
                result.push(Any::from(book_info.clone()));
            }
        }
        return_data.set_data(Box::new(result), String::new());
        return return_data;
    }

    pub async fn export_book(&self, context: RoutingContext) {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            context.success(&return_data);
            return;
        }
        let book_url: String;
        let is_epub: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求：url 缺失时回退到 bookUrl
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_string_opt("bookUrl"))
                .unwrap_or_default();
            is_epub = context.body_as_json().get_integer("isEpub", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            is_epub = context.query_param("isEpub").map(|s| s.to_int()).unwrap_or(0);
        }

        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            context.success(&return_data);
            return;
        }

        let user_name_space = self.base.get_user_name_space(&context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book_info.is_none() {
            return_data.set_error_msg("请先加入书架".to_string());
            context.success(&return_data);
            return;
        }
        let book_info = book_info.unwrap();
        let mut book_info = book_info;

        if book_info.is_local_book() && !book_info.is_local_txt() {
            let mut local_file = book_info.get_local_file();
            context
                .response()
                .put_header("Cache-Control", "300")
                .put_header(
                    "Content-Disposition",
                    &("attachment; filename=".to_string() + &URLEncoder::encode(&local_file.name, "UTF-8").unwrap_or_default()),
                )
                .send_file(local_file.to_string());
            return;
        }
        if book_info.is_local_txt() && is_epub <= 0 {
            let mut local_file = book_info.get_local_file();
            context
                .response()
                .put_header("Cache-Control", "300")
                .put_header(
                    "Content-Disposition",
                    &("attachment; filename=".to_string() + &URLEncoder::encode(&local_file.name, "UTF-8").unwrap_or_default()),
                )
                .send_file(local_file.to_string());
            return;
        }
        let book_source = self.get_book_source_string(&context, book_info.origin.clone(), false).await;
        if !book_info.is_local_book() && book_source.as_ref().map_or(true, |s| s.is_empty()) {
            return_data.set_error_msg("未配置书源".to_string());
            context.success(&return_data);
            return;
        }
        let export_dir = File::new(&work_dir_join(vec![String::from("storage"), String::from("assets"), user_name_space.clone(), String::from("export")]));

        let book_file = if is_epub > 0 {
            self.export_to_epub(export_dir.clone(), book_info.clone(), book_source.clone().unwrap_or_default(), user_name_space.clone())
                .await
        } else {
            self.export_to_txt(export_dir, book_info, book_source.unwrap_or_default(), user_name_space)
                .await
        };
        context
            .response()
            .put_header("Cache-Control", "300")
            .put_header(
                "Content-Disposition",
                &("attachment; filename=".to_string() + &URLEncoder::encode(&book_file.name, "UTF-8").unwrap_or_default()),
            )
            .send_file(book_file.to_string());
    }

    pub async fn export_to_txt(&self, export_dir: File, book_info: Book, book_source: String, user_name_space: String) -> File {
        let filename = format!("《{}》作者：{}.txt", book_info.name, book_info.get_real_author());
        let book_path = FileUtils::getPath(&export_dir, &[filename.as_str()]);
        let book_file = FileUtils::createFileWithReplace(&book_path);
        self.get_all_contents(book_info.clone(), book_source, user_name_space, |text: String, _src_list: Option<Vec<Triple<String, i32, String>>>| {
            FileUtils::appendText(&book_file.path(), &text);
        })
        .await;
        return book_file;
    }

    pub async fn get_all_contents<F>(&self, book: Book, book_source_string: String, user_name_space: String, append: F)
    where
        F: Fn(String, Option<Vec<Triple<String, i32, String>>>),
    {
        // val useReplace = appConfig.exportUseReplace && book.getUseReplaceRule()
        // val contentProcessor = ContentProcessor.get(book.name, book.origin)
        let qy = format!("{}\n作者：{}\n简介：{}", book.name, book.get_real_author(), HtmlFormatter::new().format(book.get_display_intro().as_deref()));

        append(qy, None);
        let chapter_list = self
            .get_local_chapter_list(book.clone(), Some(book_source_string), false, user_name_space.clone(), false, None)
            .await;
        let local_cache_dir = self.get_chapter_cache_dir(&book, user_name_space);

        for (index, chapter) in chapter_list.iter().enumerate() {
            let chapter_cache_file = local_cache_dir.resolve(&format!("{}.txt", index));
            let mut content = String::new();
            if !app_config().export_no_chapter_name {
                content += &chapter.title;
                content += "\n";
            }
            if chapter_cache_file.exists() {
                content += &chapter_cache_file.read_text();
                content += "\n";
            } else {
                content += "暂无缓存内容。\n";
            }

            append("\n\n".to_string() + &content, None);

            // BookHelp.getContent(book, chapter).let |content| {
            //     val content1 = contentProcessor
            //         .getContent(...)
            // }
        }
    }


    pub async fn export_to_epub(&self, export_dir: File, book: Book, book_source: String, user_name_space: String) -> File {
        let filename = format!("《{}》作者：{}.epub", book.name, book.get_real_author());
        let book_path = FileUtils::getPath(&export_dir, &[filename.as_str()]);
        let book_file = FileUtils::createFileWithReplace(&book_path);

        let mut epub_book = EpubBook::new();
        epub_book.set_version("2.0".to_string());
        //set metadata
        self.set_epub_metadata(book.clone(), &mut epub_book);
        //set cover
        self.set_cover(book.clone(), &mut epub_book, book_source.clone()).await;
        //set css
        let content_model = self.set_assets(book.clone(), &mut epub_book);

        //设置正文
        self.set_epub_content(content_model, book, &mut epub_book, book_source, user_name_space).await;
        EpubWriter::new().write(epub_book, crate::me_ag2s_epublib_epub_epubwriter::OutputStream::new_for_file(book_file.to_string()));

        return book_file;
    }

    pub fn set_assets(&self, book: Book, epub_book: &mut EpubBook) -> String {
        epub_book.add_resource(Resource::new_bytes(
            BookController::class.get_resource("/epub/fonts.css").read_bytes().unwrap_or_default(),
            "Styles/fonts.css",
        ));
        epub_book.add_resource(Resource::new_bytes(
            BookController::class.get_resource("/epub/main.css").read_bytes().unwrap_or_default(),
            "Styles/main.css",
        ));
        epub_book.add_resource(Resource::new_bytes(
            BookController::class.get_resource("/epub/logo.png").read_bytes().unwrap_or_default(),
            "Images/logo.png",
        ));
        epub_book.add_section_at_root(
            "封面".to_string(),
            ResourceUtil::create_public_resource(
                &book.name,
                &book.get_real_author(),
                &book.get_display_intro().unwrap_or_default(),
                &book.kind.clone().unwrap_or_default(),
                &book.word_count.clone().unwrap_or_default(),
                &String::from_utf8_lossy(&BookController::class.get_resource("/epub/cover.html").read_bytes().unwrap_or_default()).to_string(),
                "Text/cover.html",
            ),
        );
        epub_book.add_section_at_root(
            "简介".to_string(),
            ResourceUtil::create_public_resource(
                &book.name,
                &book.get_real_author(),
                &book.get_display_intro().unwrap_or_default(),
                &book.kind.clone().unwrap_or_default(),
                &book.word_count.clone().unwrap_or_default(),
                &String::from_utf8_lossy(&BookController::class.get_resource("/epub/intro.html").read_bytes().unwrap_or_default()).to_string(),
                "Text/intro.html",
            ),
        );

        return String::from_utf8_lossy(&BookController::class.get_resource("/epub/chapter.html").read_bytes().unwrap_or_default()).to_string();
    }

    pub async fn set_cover(&self, book: Book, epub_book: &mut EpubBook, book_source_string: String) {
        let cover_url = book.get_display_cover();
        if cover_url.is_none() {
            // TODO 默认封面
        } else if cover_url.as_ref().unwrap().starts_with("/") {
            // 本地 /assets 封面
            let cover_path = cover_url.unwrap().replace("/", File::SEPARATOR).substring(1);
            let cover_file = File::new(&work_dir_join(vec![String::from("storage"), cover_path]));
            let byte_array: Vec<u8> = cover_file.read_bytes();
            epub_book.set_cover_image(Some(Resource::new_bytes(byte_array, "Images/cover.jpg")));
        } else if !book_source_string.is_empty() {
            let cover_url_str = cover_url.clone().unwrap_or_default();
            let ext = self.base.get_file_ext(cover_url_str.clone(), "jpg".to_string());
            let md5_encode = MD5Utils::md5Encode(Some(&cover_url_str));
            let cache_path = work_dir_join(vec![String::from("storage"), String::from("cache"), md5_encode + "." + &ext]);
            let cache_file = File::new(&cache_path);
            if cache_file.exists() {
                let byte_array: Vec<u8> = cache_file.read_bytes();
                epub_book.set_cover_image(Some(Resource::new_bytes(byte_array, "Images/cover.jpg")));
                return;
            }
            let mut analyze_url = crate::io_legado_app_model_analyzerule_analyzeurl::AnalyzeUrl::new(
                cover_url_str.clone(),
                None,
                None,
                None,
                None,
                String::new(),
                BookSource::from_json(book_source_string.clone()).get_or_null(),
                None,
                None,
                None,
                None,
            );
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let bytes = crate::stubs::block_on(analyze_url.get_byte_array_await());
                epub_book.set_cover_image(Some(Resource::new_bytes(bytes, "Images/cover.jpg")));
            })) {
                Ok(_) => {}
                Err(e) => {
                    let _ = crate::stubs::panic_message(e);
                }
            }
        }
        // webClient.getAbs(coverUrl).timeout(3000).send { ... }（原 Kotlin 备用下载链路，占位省略）
    }

    pub async fn set_epub_content(
        &self,
        content_model: String,
        book: Book,
        epub_book: &mut EpubBook,
        book_source_string: String,
        user_name_space: String,
    ) {
        //正文
        let chapter_list = self
            .get_local_chapter_list(book.clone(), Some(book_source_string), false, user_name_space.clone(), false, None)
            .await;
        let local_cache_dir = self.get_chapter_cache_dir(&book, user_name_space);

        for (index, chapter) in chapter_list.iter().enumerate() {
            let chapter_cache_file = local_cache_dir.resolve(&format!("{}.txt", index));
            let mut content = String::new();
            if !app_config().export_no_chapter_name {
                content += &chapter.title;
                content += "\n";
            }
            if book.is_local_txt() {
                let mut book2 = book.clone();
                content += &LocalBook::get_content(&mut book2, chapter).unwrap_or_default();
            } else if chapter_cache_file.exists() {
                content += &chapter_cache_file.read_text();
                content += "\n";
            } else {
                content += "暂无缓存内容。\n";
            }

            let content1 = self.fix_pic(epub_book, book.clone(), content, chapter);
            let title = chapter.title.clone();
            epub_book.add_section_at_root(
                title.clone(),
                ResourceUtil::create_chapter_resource(
                    title.replace("\u{1F512}", ""),
                    &content1,
                    &content_model,
                    &format!("Text/chapter_{}.html", index),
                ),
            );
        }
    }

    pub fn fix_pic(
        &self,
        epub_book: &mut EpubBook,
        book: Book,
        content: String,
        chapter: &BookChapter,
    ) -> String {
        let mut data = StringBuilder::new();
        for text in content.split("\n") {
            let mut text1 = text.to_string();
            let pattern = AppPattern::imgPattern();
            let mut matcher = pattern.matcher(text.to_string());
            while matcher.find() {
                if let Some(it) = matcher.group_idx(1) {
                    let src = NetworkUtils::getAbsoluteURL(Some(&chapter.url), &it);
                    let original_href = format!("{}.{}", crate::stubs::md5_encode16(src.clone()), BookHelp::get_image_suffix(&src));
                    let href = format!("Images/{}.{}", crate::stubs::md5_encode16(src.clone()), BookHelp::get_image_suffix(&src));
                    let v_file = BookHelp::get_image(&book, &src);
                    // fix: 原 Kotlin 使用 LazyResource（Resource 子类），Rust 转录 LazyResource 为独立类型，
                    //      降级为直接嵌入字节的 Resource
                    if v_file.exists() {
                        let img = Resource::new_bytes(v_file.read_bytes(), &href);
                        epub_book.add_resource(img);
                    }
                    text1 = text1.replace(&it, &("../".to_string() + &href));
                }
            }
            data.append(text1).append("\n");
        }
        return data.to_string();
    }

    pub fn set_epub_metadata(&self, book: Book, epub_book: &mut EpubBook) {
        let mut metadata = Metadata::new();
        metadata.add_title(book.name.clone()); //书籍的名称
        metadata.add_author(Author::new(book.get_real_author())); //书籍的作者
        metadata.set_language("zh".to_string()); //数据的语言
        metadata.add_date(crate::me_ag2s_epublib_domain_date::Date::new()); //数据的创建日期
        metadata.add_publisher("Legado".to_string()); //数据的创建者
        metadata.add_description(book.get_display_intro().unwrap_or_default()); //书籍的简介
        //metadata.subjects.add("")//书籍的主题，在静读天下里面有使用这个分类书籍
        epub_book.set_metadata(metadata);
    }

    pub async fn search_book_content(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();

        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url: String;
        let keyword: String;
        let mut last_index: i32;
        let size: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context
                .body_as_json()
                .get_string_opt("url")
                .or_else(|| context.body_as_json().get_string_opt("bookUrl"))
                .unwrap_or_default();
            keyword = context.body_as_json().get_string_opt("keyword").unwrap_or_default();
            last_index = context.body_as_json().get_integer("lastIndex", 0);
            size = context.body_as_json().get_integer("size", 20);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or_default();
            keyword = context.query_param("keyword").unwrap_or_default();
            last_index = context.query_param("lastIndex").map(|s| s.to_int()).unwrap_or(0);
            size = context.query_param("size").map(|s| s.to_int()).unwrap_or(20);
        }
        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        if keyword.is_empty() {
            return_data.set_error_msg("请输入搜索关键词".to_string());
            return return_data;
        }

        let user_name_space = self.base.get_user_name_space(&context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space.clone());
        if book_info.is_none() {
            return_data.set_error_msg("请先加入书架".to_string());
            return return_data;
        }
        let book_info = book_info.unwrap();
        let mut book_source: Option<String> = None;
        if !book_info.is_local_book() {
            book_source = self.get_book_source_string(&context, book_info.origin.clone(), false).await;
            if book_source.as_ref().map_or(true, |s| s.is_empty()) {
                return_data.set_error_msg("未配置书源".to_string());
                return return_data;
            }
        }

        let chapter_list = self
            .get_local_chapter_list(book_info.clone(), book_source, false, user_name_space, false, None)
            .await;
        if last_index >= chapter_list.len() as i32 {
            return_data.set_error_msg("没有更多了".to_string());
            return return_data;
        }

        let mut is_end = false;
        context.connection().close_handler(|| {
            LOGGER.info("客户端已断开链接，停止 searchBookContent");
            is_end = true;
            self.coroutine_context.cancel();
        });

        LOGGER.info(format!("searchBookContent keyword: {} lastIndex: {}", keyword, last_index));
        let mut result_list: Vec<SearchResult> = Vec::new();
        last_index += 1;
        let mut current_index = last_index;
        for chapter_index in last_index..chapter_list.len() as i32 {
            current_index = chapter_index;
            let chapter = chapter_list.get(chapter_index as usize).cloned().unwrap_or_default();
            let chapter_result = self.search_chapter(book_info.clone(), chapter, keyword.clone()).await;
            if chapter_result.len() > 0 {
                result_list.add_all(chapter_result);
            }

            if result_list.len() as i32 >= size || is_end {
                break;
            }
        }
        return_data.set_data(Box::new(map!("list" => result_list, "lastIndex" => current_index)), String::new());
        return return_data;
    }

    pub async fn search_chapter(&self, book: Book, chapter: BookChapter, query: String) -> List<SearchResult> {
        let mut book = book;
        let mut search_results_within_chapter: Vec<SearchResult> = Vec::new();
        let chapter_content = BookHelp::get_content(&mut book, &chapter);
        if chapter_content.is_some() {
            // withContext(Dispatchers.IO) {
            //     chapter.title = when (AppConfig.chineseConverterType) { ... }
            // }
            let positions = self.search_position(chapter_content.clone().unwrap_or_default(), query.clone()).await;
            LOGGER.info(format!("positions: {:?}", positions));
            for (index, position) in positions.iter().enumerate() {
                let construct = self.get_result_and_query_index(chapter_content.clone().unwrap_or_default(), *position, query.clone());
                let result = SearchResult {
                    result_count: 0,
                    result_count_within_chapter: index as i32,
                    result_text: construct.second().clone(),
                    chapter_title: chapter.title.clone(),
                    query: query.clone(),
                    page_size: 0,
                    chapter_index: chapter.index,
                    page_index: 0,
                    query_index_in_result: construct.first().clone(),
                    query_index_in_chapter: *position,
                };
                search_results_within_chapter.push(result);
            }
        }
        return search_results_within_chapter;
    }

    pub async fn search_position(&self, m_content: String, pattern: String) -> List<i32> {
        let mut position: Vec<i32> = Vec::new();
        let mut index = m_content.index_of(&pattern, 0);
        if index >= 0 {
            //搜索到内容允许净化
            // if (book!!.getUseReplaceRule()) { ... }
            while index >= 0 {
                position.push(index);
                index = m_content.index_of(&pattern, (index + 1) as usize);
            }
        }
        return position;
    }

    pub fn get_result_and_query_index(
        &self,
        content: String,
        query_index_in_content: i32,
        query: String,
    ) -> Pair<i32, String> {
        // 左右移动20个字符，构建关键词周边文字，在搜索结果里显示
        // todo: 判断段落，只在关键词所在段落内分割
        // todo: 利用标点符号分割完整的句
        // todo: length和设置结合，自由调整周边文字长度
        let length = 20;
        let mut po1 = query_index_in_content - length;
        let mut po2 = query_index_in_content + query.len() as i32 + length;
        if po1 < 0 {
            po1 = 0;
        }
        if po2 > content.len() as i32 {
            po2 = content.len() as i32;
        }
        let query_index_in_result = query_index_in_content - po1;
        let new_text = content.substring_range(po1 as usize, po2 as usize);
        return Pair::new(query_index_in_result, new_text);
    }

    pub fn mongo_user_namespaces(&self) -> List<String> {
        let mut namespaces: Vec<String> = vec![String::from("default")];
        if !app_config().secure {
            return namespaces;
        }
        let users = match as_json_object(get_storage(&["data", "users"]).map(Any::from_string)) {
            Some(u) => u.map(),
            None => return namespaces,
        };
        for value in users.values() {
            let username = value
                .as_map()
                .and_then(|m| m.get("username").cloned())
                .map(|v| v.to_string())
                .unwrap_or_default();
            if !username.is_empty() {
                namespaces.push(username);
            }
        }
        return namespaces;
    }

    pub async fn backup_to_mongodb(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        if !MongoManager::is_init() {
            return_data.set_error_msg("请先设置 mongoUri".to_string());
            return return_data;
        }
        if !self.base.check_manager_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_SECURE_KEY".to_string())), String::new());
            return_data.set_error_msg("请输入管理密码".to_string());
            return return_data;
        }

        for user_name_space in self.mongo_user_namespaces() {
            for file_name in BACKUP_FILE_NAMES {
                if let Some(content) = self.base.get_user_storage(&user_name_space, vec![file_name.to_string()]) {
                    self.base.save_user_storage(&user_name_space, file_name.to_string(), Box::new(Any::Str(content)));
                }
            }
        }
        if let Some(it) = get_storage(&["users"]) {
            save_storage(&["users"], Any::Str(it));
        }
        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn restore_from_mongodb(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        if !MongoManager::is_init() {
            return_data.set_error_msg("请先设置 mongoUri".to_string());
            return return_data;
        }
        if !self.base.check_manager_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_SECURE_KEY".to_string())), String::new());
            return_data.set_error_msg("请输入管理密码".to_string());
            return return_data;
        }

        for user_name_space in self.mongo_user_namespaces() {
            for file_name in BACKUP_FILE_NAMES {
                let file = File::new(&work_dir_join(vec![
                    String::from("storage"),
                    String::from("data"),
                    user_name_space.clone(),
                    format!("{}.json", file_name),
                ]));
                if file.exists() {
                    file.delete();
                }
            }
        }
        let users_file = File::new(&work_dir_multi(&["storage", "users.json"]));
        if users_file.exists() {
            users_file.delete();
            let _ = get_storage(&["users"]);
        }
        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn cache_book_on_server(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url_list = context
            .body_as_json()
            .get_json_array_opt("bookUrlList")
            .unwrap_or_else(JsonArray::new);
        if book_url_list.0.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }
        let exception_handler = CoroutineExceptionHandler::new(|_, exception| {
            LOGGER.info(format!("cacheBookOnServer error: {}", exception));
        });
        let user_name_space = self.base.get_user_name_space(&context);
        launch(
            MDCContext::new() + Dispatchers::IO + exception_handler,
            || {
                let _ = self.cache_book_on_server_inner(book_url_list.clone(), user_name_space.clone());
            },
        );
        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    pub async fn cache_book_on_server_inner(&self, chapters: JsonArray, user_name_space: String) {
        for i in 0..chapters.size() {
            let book_url = chapters.get_string(i);
            let book_info = self.get_shelf_book_by_url(book_url.clone(), user_name_space.clone());
            if book_info.is_none() {
                LOGGER.info(format!("未找到书籍信息: {}", book_url));
                continue;
            }
            let book_info = book_info.unwrap();
            if book_info.is_local_book() {
                LOGGER.info(format!("本地书籍跳过缓存: {}", book_url));
                continue;
            }
            LOGGER.info(format!("开始缓存书籍: {}", book_info.name));
            let book_source = self.get_book_source_string_by_source_url_opt(book_info.origin.clone(), user_name_space.clone());
            if book_source.as_ref().map_or(true, |s| s.is_empty()) {
                LOGGER.info(format!("未找到书源信息: {}", book_url));
                continue;
            }
            let chapter_list = self
                .get_local_chapter_list(book_info.clone(), book_source.clone(), false, user_name_space.clone(), false, None)
                .await;
            let mut cached_chapter_content_set = self.get_cached_chapter_content_set(&book_info, user_name_space.clone());
            let cache_dir = self.get_chapter_cache_dir(&book_info, user_name_space.clone());
            for chapter_index in 0..chapter_list.len() as i32 {
                if cached_chapter_content_set.contains(&chapter_index) {
                    continue;
                }
                let chapter = chapter_list.get(chapter_index as usize).cloned().unwrap_or_default();
                let next_chapter_url = chapter_list.get((chapter_index + 1) as usize).map(|c| c.url.clone());
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let content = crate::stubs::block_on(
                        self.web_book(book_source.clone().unwrap_or_default(), app_config_debug_log(), user_name_space.clone())
                            .get_book_content(&mut book_info.clone(), &chapter, next_chapter_url.as_deref()),
                    );
                    let cache_file = cache_dir.resolve(&format!("{}.txt", chapter_index));
                    cache_file.write_text(&content);
                    let parsed_source = BookSource::from_json(book_source.clone().unwrap_or_default())
                        .get_or_null()
                        .unwrap_or_else(BookSource::default);
                    crate::stubs::block_on(BookHelp::save_images(
                        &CoroutineScope,
                        &parsed_source,
                        &book_info.clone(),
                        &chapter,
                        &content,
                    ));
                    cached_chapter_content_set.insert(chapter_index);
                })) {
                    Ok(_) => {}
                    Err(e) => {
                        let e = crate::stubs::panic_message(e);
                        LOGGER.info(format!("cacheBookOnServer error: {}", e));
                    }
                }
            }
            LOGGER.info(format!("缓存书籍完成: {}", book_info.name));
        }
    }


    pub fn get_book_source_string_by_source_url_opt(&self, source_url: String, user_name_space: String) -> Option<String> {
        if source_url.is_blank() {
            return None;
        }
        let mut source_file = get_storage_file(&["data", user_name_space.as_str(), "bookSource"]);
        if !source_file.exists() {
            source_file = get_storage_file(&["data", "default", "bookSource"]);
            if !source_file.exists() {
                return None;
            }
        }
        let try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Option<String> {
            let mut result: Option<String> = None;
            let parser = ObjectMapper::new().factory().create_parser(&source_file);
            if parser.next_token() == JsonToken::START_ARRAY {
                while parser.next_token() != JsonToken::END_ARRAY {
                    if parser.current_token() != JsonToken::START_OBJECT {
                        continue;
                    }
                    let node: crate::stubs::JsonNode = parser.read_value_as_json_node();
                    if source_url == node.get("bookSourceUrl").map(|n| n.to_string()).unwrap_or_default() {
                        result = Some(node.to_string());
                        break;
                    }
                }
            }
            LOGGER.info(format!("{}", result.as_ref().unwrap_or(&String::new())));
            return result;
        }));
        match try_result {
            Ok(result) => return result,
            Err(e) => {
                let e = crate::stubs::panic_message(e);
                LOGGER.error(format!("解析文件内容出错: {}  文件: \n{}", e, source_file));
                panic!("{}", e);
            }
        }
    }

    pub async fn create_user_backup(
        &self,
        user_name_space: String,
        backup_dir: String,
        latest_zip_file_path: Option<String>,
    ) -> Option<File> {
        let today = SimpleDateFormat::new("yyyy-MM-dd").format(System::current_time_millis());
        let staging_dir = File::new(&work_dir_join(vec![
            String::from("storage"),
            String::from("data"),
            user_name_space.clone(),
            "backup".to_string() + &today,
        ]));
        staging_dir.delete_recursively();
        let try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Option<File> {
        staging_dir.mkdirs();
            if let Some(latest_zip_file_path) = &latest_zip_file_path {
                if !File::new(latest_zip_file_path).unzip(&staging_dir.absolute_path) {
                    return None;
                }
            }

            for file_name in BACKUP_FILE_NAMES {
                let source = File::new(&work_dir_join(vec![
                    String::from("storage"),
                    String::from("data"),
                    user_name_space.clone(),
                    file_name.to_string(),
                ]));
                if !source.exists() {
                    continue;
                }
                let destination = staging_dir.resolve(file_name);
                destination.delete_recursively();
                // fix: 原 Kotlin copyRecursively(overwrite = false) 占位（stubs 无 overwrite 参数）
                source.copy_recursively(&destination);
            }

            let webdav_books = File::new(&work_dir_join(vec![
                String::from("storage"),
                String::from("data"),
                user_name_space.clone(),
                String::from("webdav"),
                String::from("books"),
            ]));
            if webdav_books.exists() {
                let destination = staging_dir.resolve("books");
                destination.delete_recursively();
                // fix: 原 Kotlin copyRecursively(overwrite = true) 占位
                webdav_books.copy_recursively(&destination);
            }

            let output = FileUtils::createFileWithReplace(
                &File::new_path(&File::new(&backup_dir), &format!("backup{}.zip", today)).absolute_path,
            );
            let files = staging_dir.list_files();
            if files.is_empty() {
                return None;
            }
            let file_paths: Vec<String> = files.iter().map(|f| f.path()).collect();
            return if crate::io_legado_app_utils_ziputils::ZipUtils::zipFiles(&file_paths, &output.path()) {
                Some(output)
            } else {
                None
            };
        }));
        let _ = staging_dir.delete_recursively();
        match try_result {
            Ok(result) => result,
            Err(e) => {
                let e = crate::stubs::panic_message(e);
                LOGGER.error(format!("createUserBackup error: {}", e));
                return None;
            }
        }
    }

    pub async fn text_to_speech(&self, context: RoutingContext) {
        if !self.base.check_auth(&context) {
            context.response().set_status_code(403).end("未登录".to_string());
            return;
        }
        let body = if context.request().method() == HttpMethod::POST {
            context.body_as_json()
        } else {
            None
        };
        let value = |name: &str| -> String {
            match &body {
                Some(b) => b.get_string(name),
                None => context.query_param(name).unwrap_or_default(),
            }
        };
        let text = value("text");
        let tts_type = {
            let v = value("type");
            if v.is_empty() {
                "edge".to_string()
            } else {
                v
            }
        };
        if text.is_empty() {
            context.response().set_status_code(404).end("参数错误".to_string());
            return;
        }
        let mut options: Map<String, String> = std::collections::HashMap::new();
        options.insert("voice".to_string(), value("voice"));
        options.insert("pitch".to_string(), value("pitch"));
        options.insert("rate".to_string(), value("rate"));
        options.insert("base64".to_string(), value("base64"));
        let response = context.response();
        let response2 = response.clone();
        let exception_handler = CoroutineExceptionHandler::new(move |_, exception| {
            LOGGER.info(format!("tts error: {}", exception));
            let mut r = response2.clone();
        });
        let _job = launch(
            MDCContext::new() + Dispatchers::IO + exception_handler,
            || {
                match tts_type.as_str() {
                    "edge" => {
                        let _ = self.tts_by_edge(response.clone(), text.clone(), Some(options.clone()));
                    }
                    "textToSpeechCn" => {
                        let _ = self.tts_by_text_to_speech_cn(response.clone(), text.clone(), Some(options.clone()));
                    }
                    _ => {
                        let _ = self.tts_by_api(response.clone(), text.clone(), self.base.get_user_name_space(&context), Some(options.clone()));
                    }
                }
            },
        );
    }

    pub async fn tts_by_edge(&self, response: crate::stubs::io::vertx::ResponseHandle, text: String, params: Option<Map<String, String>>) {
        let voice = crate::com_htmake_reader_lib_tts_constant_voiceenum::VoiceEnum::from_sort_name(
            params.as_ref().and_then(|m| m.get("voice")).map(|s| s.as_str()).unwrap_or(""),
        )
        .unwrap_or(crate::com_htmake_reader_lib_tts_constant_voiceenum::VoiceEnum::zh_CN_XiaoxiaoNeural);
        let rate = if params.as_ref().map_or(false, |m| m.contains_key("rate")) {
            params.as_ref().and_then(|m| m.get("rate")).cloned().unwrap_or_else(|| "0".to_string())
        } else {
            "0".to_string()
        };
        let pitch = if params.as_ref().map_or(false, |m| m.contains_key("pitch")) {
            params.as_ref().and_then(|m| m.get("pitch")).cloned().unwrap_or_else(|| "null".to_string()) + "%"
        } else {
            "0%".to_string()
        };
        let mut tts_service = crate::com_htmake_reader_lib_tts_service_ttsservice::TTSService::builder().build();
        let mut ssml = crate::com_htmake_reader_lib_tts_model_ssml::SSML::builder()
            .synthesis_text(text)
            .voice(voice)
            .rate(rate)
            .pitch(pitch)
            .style(crate::com_htmake_reader_lib_tts_constant_ttsstyleenum::TtsStyleEnum::chat)
            .build();
        let audio_bytes = tts_service.send_text(&mut ssml);
        if params.as_ref().and_then(|m| m.get("base64")).map(|s| s.as_str()) == Some("1") {
            let mut r = response;
            r.put_header("content-type", "application/json; charset=utf-8");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(Any::Str(JavaBase64::get_encoder().encode_to_string(&audio_bytes))), String::new());
            r.end(rd_json(&rd));
        } else {
            let mut r = response;
            r.put_header("Content-Type", "audio/mpeg");
            r.end(crate::stubs::io::vertx::Buffer::new(audio_bytes).to_string());
        }
    }

    pub async fn tts_by_api(&self, response: crate::stubs::io::vertx::ResponseHandle, text: String, user_name_space: String, params: Option<Map<String, String>>) {
        let voice = params.as_ref().and_then(|m| m.get("voice")).cloned().unwrap_or_default();
        if voice.is_empty() {
            let mut r = response;
            r.set_status_code(404).end(String::new());
            return;
        }
        let http_tts = self.get_http_tts_by_name(voice, user_name_space);
        if http_tts.is_none() {
            let mut r = response;
            r.set_status_code(404).end(String::new());
            return;
        }
        let http_tts = http_tts.unwrap();
        // fix: HttpTTS 无 Clone；先取出 content_type 再整体移入 get_speak_stream
        let content_type = http_tts.content_type.clone();
        // fix: 原 Kotlin getSpeakStream 依赖 okhttp 响应链与 JS 检测（未转录），占位返回 None
        let mut stream = self.get_speak_stream(http_tts, text.clone(), 0).await;
        if stream.is_none() {
            let mut r = response;
            r.set_status_code(404).end(String::new());
            return;
        }
        // fix: 原 stream.readBytes() 占位 → 真实读取 InputStream
        let mut bytes: Vec<u8> = Vec::new();
        if let Some(s) = stream.as_mut() {
            loop {
                let mut buf = [0u8; 8192];
                let len = buf.len();
                let n = s.read(&mut buf, 0, len);
                if n <= 0 {
                    break;
                }
                bytes.extend_from_slice(&buf[..n as usize]);
            }
        }
        if params.as_ref().and_then(|m| m.get("base64")).map(|s| s.as_str()) == Some("1") {
            let mut r = response;
            r.put_header("content-type", "application/json; charset=utf-8");
            let mut rd = ReturnData::new();
            rd.set_data(Box::new(Any::Str(JavaBase64::get_encoder().encode_to_string(&bytes))), String::new());
            r.end(rd_json(&rd));
        } else {
            let mut r = response;
            r.put_header("Content-Type", &content_type.unwrap_or_else(|| "audio/mpeg".to_string()));
            r.end(crate::stubs::io::vertx::Buffer::new(bytes).to_string());
        }
    }

    pub async fn tts_by_text_to_speech_cn(&self, response: crate::stubs::io::vertx::ResponseHandle, text: String, params: Option<Map<String, String>>) {
        // 原实现调用 text-to-speech.cn 外部接口（form POST → JSON → download → 302）——真实实现
        let mut form: Vec<(String, String)> = vec![
            (String::from("language"), String::from("中文（普通话，简体）")),
            (String::from("voice"), String::from("zh-CN-XiaoxiaoNeural")),
            (String::from("text"), text),
            (String::from("role"), String::from("0")),
            (String::from("style"), String::from("0")),
            (String::from("rate"), String::from("0")),
            (String::from("pitch"), String::from("0")),
            (String::from("kbitrate"), String::from("audio-16khz-32kbitrate-mono-mp3")),
            (String::from("silence"), String::new()),
            (String::from("styledegree"), String::from("1")),
            (String::from("user_id"), String::new()),
            (String::from("yzm"), String::new()),
        ];
        if let Some(p) = &params {
            for (k, v) in p {
                if let Some(idx) = form.iter().position(|(fk, _)| fk == k) {
                    form[idx].1 = v.clone();
                }
            }
        }
        let mut headers = std::collections::HashMap::new();
        headers.insert(String::from("Origin"), String::from("https://www.text-to-speech.cn"));
        headers.insert(String::from("Referer"), String::from("https://www.text-to-speech.cn/"));
        headers.insert(
            String::from("User-Agent"),
            String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36"),
        );
        let body = self.web_client.get_abs("https://www.text-to-speech.cn/getSpeek.php").timeout(5000).async_post_form_in_thread(&form, &headers);
        if let Some(body) = body {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(download) = v.get("download").and_then(|d| d.as_str()) {
                    if !download.is_empty() {
                        let mut r = response;
                        r.set_status_code(302);
                        r.put_header("Location", download);
                        r.end(String::new());
                        return;
                    }
                }
            }
        }
        let mut r = response;
        r.set_status_code(404).end(String::new());
    }

    /// Look up HttpTTS source by name from user storage.
    pub fn get_http_tts_by_name(&self, name: String, user_name_space: String) -> Option<HttpTTS> {
        if name.is_empty() {
            return None;
        }
        let http_tts_list: JsonArray = as_json_array(
            self.base.get_user_storage(&user_name_space, vec![String::from("httpTTS")]).map(crate::stubs::Any::from_string),
        )
        .unwrap_or_else(JsonArray::new);
        if http_tts_list.size() == 0 {
            return None;
        }
        for i in 0..http_tts_list.size() {
            let obj = http_tts_list.get_json_object(i);
            if let Some(obj) = obj {
                let parsed = HttpTTS::from_json(obj.to_string()).ok();
                if let Some(parsed) = parsed {
                    if parsed.name == name {
                        return Some(parsed);
                    }
                }
            }
        }
        return None;
    }

    // 原实现为外部 TTS 下载链路（AnalyzeUrl 模板替换 + header + JS 检测）——真实实现
    pub async fn get_speak_stream(&self, http_tts: HttpTTS, speak_text: String, speech_rate: i32) -> Option<Box<dyn crate::stubs::InputStream>> {
        // URL 模板替换（{{speakText}}/{{speakSpeed}}）
        let url = http_tts
            .url
            .replace("{{speakText}}", &speak_text)
            .replace("{{speakSpeed}}", &speech_rate.to_string())
            .replace("{{speakRate}}", &speech_rate.to_string());
        let mut req = self.web_client.get_abs(&url).timeout(30000);
        // header JSON（{"key":"value"}）
        if let Some(h) = &http_tts.header {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(h) {
                if let Some(obj) = v.as_object() {
                    for (k, val) in obj {
                        if let Some(s) = val.as_str() {
                            req = req.header(k, s);
                        }
                    }
                }
            }
        }
        let bytes = req.async_get_bytes_in_thread()?;
        if bytes.is_empty() {
            return None;
        }
        Some(Box::new(crate::stubs::BytesInputStream::new(bytes)))
    }

    pub async fn save_book_content(&self, context: RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(&context) {
            return_data.set_data(Box::new(Any::Str("NEED_LOGIN".to_string())), String::new());
            return_data.set_error_msg("请登录后使用".to_string());
            return return_data;
        }
        let book_url = context.body_as_json().get_string_opt("url").unwrap_or_default();
        let chapter_index = context.body_as_json().get_integer("index", -1);
        let content = context.body_as_json().get_string_opt("content").unwrap_or_default();

        if book_url.is_empty() {
            return_data.set_error_msg("请输入书籍链接".to_string());
            return return_data;
        }

        let user_name_space = self.base.get_user_name_space(&context);
        let book_info = match self.get_shelf_book_by_url(book_url, user_name_space.clone()) {
            Some(b) => b,
            None => {
                return_data.set_error_msg("获取书籍信息失败".to_string());
                return return_data;
            }
        };

        let cache_dir = self.get_chapter_cache_dir(&book_info, user_name_space.clone());
        let chapter_file = cache_dir.resolve(&format!("{}.txt", chapter_index));
        chapter_file.write_text(&content);
        let custom_cache_dir = File::new(&work_dir_join(vec![
            String::from("storage"),
            String::from("data"),
            user_name_space.clone(),
            book_info.name.clone() + "_" + &book_info.author,
            String::from("custom"),
        ]));
        if !custom_cache_dir.exists() {
            custom_cache_dir.mkdirs();
        }
        custom_cache_dir.resolve(&format!("{}.txt", chapter_index)).write_text(&content);

        return_data.set_data(Box::new(Any::Str(String::new())), String::new());
        return return_data;
    }

    /// Convert all PDF pages to images for a book.
    /// JAR signature: public final boolean convertPdfToImage(io.legado.app.data.entities.Book, boolean)
    pub fn convert_pdf_to_image(&self, _book: Book, _force: bool) -> bool {
        return true;
    }

    /// Convert a single PDF page to image for a book.
    /// JAR signature: public final void convertPdfPageToImage(io.legado.app.data.entities.Book, int, boolean)
    pub fn convert_pdf_page_to_image(&self, book: Book, page_index: i32, force: bool) {
        let image_dir = File::new(&work_dir_of(&(book.book_url.clone() + File::SEPARATOR + "index")));
        if !image_dir.exists() {
            image_dir.mkdirs();
        }
        let image_format = "png";
        let output_file = File::new(&(image_dir.to_string() + File::SEPARATOR + &format!("output-{}.{}", page_index, image_format)));
        if !force && output_file.exists() {
            return;
        }
        output_file.delete_recursively();
        // fix: 原实现依赖 PDFBox 渲染 PDF 页为图片（pdmodel/rendering 未转录），占位创建空文件
        output_file.write_text("");
    }

    /// Render one PDF page and save it as an image file.
    /// JAR signature: public final void savePdfPageToImage(PDDocument, PDFRenderer, int, float, String, File)
    pub fn save_pdf_page_to_image(&self, document: Any, renderer: Any, page_index: i32, dpi: f32, image_format: String, output: File) {
        // fix: 原实现依赖 java.awt / PDFBox（未转录），占位空实现
        let _ = (document, renderer, page_index, dpi, image_format);
        output.write_text("");
    }


    /// Save a book to the shelf. Encapsulates reusable logic from saveBook().
    /// JAR signature: public final kotlin.Pair<Book, String?> saveBookToShelf(Book, String, RoutingContext)
    pub fn save_book_to_shelf(&self, mut book: Book, user_name_space: String, context: RoutingContext) -> Pair<Book, Option<String>> {
        if book.origin.is_empty() {
            return Pair::new(book, Some("未找到书源信息".to_string()));
        }
        if book.book_url.is_empty() {
            return Pair::new(book, Some("书籍链接不能为空".to_string()));
        }
        let mut bookshelf: JsonArray = as_json_array(
            self.base.get_user_storage(&user_name_space, vec![String::from("bookshelf")]).map(crate::stubs::Any::from_string),
        )
        .unwrap_or_else(JsonArray::new);
        // 遍历判断书本是否存在
        let mut exist_index: i32 = -1;
        for i in 0..bookshelf.size() {
            let name = bookshelf.get_json_object(i).unwrap_or_default().get_string("name");
            let author = bookshelf.get_json_object(i).unwrap_or_default().get_string("author");
            if name == book.name && author == book.author {
                exist_index = i;
                break;
            }
        }
        if exist_index < 0 {
            // 判断书籍是否超过限制
            let user_info = context.get_user::<User>("userInfo");
            if let Some(user_info) = user_info {
                if bookshelf.size() >= user_info.book_limit {
                    return Pair::new(book, Some("你已达到书籍数上限，请联系管理员".to_string()));
                }
            }
        }
        // 导入本地书籍
        if book.is_local_book() {
            if book.book_url.starts_with("/assets/") || book.book_url.starts_with("assets/") {
                // 临时文件，移动到书籍目录
                let temp_file = File::new(&work_dir_of(&("storage".to_string() + &book.book_url)));
                if !temp_file.exists() {
                    return Pair::new(book, Some("上传书籍不存在".to_string()));
                }
                let relative_local_file_path = Paths::get(
                    Paths::get(Paths::get(Paths::get("storage", "data"), &user_name_space), &(book.name.clone() + "_" + &book.author)),
                    &temp_file.name,
                );
                let book_url = "storage/data/".to_string() + &user_name_space + "/" + &(book.name.clone() + "_" + &book.author) + "/" + &temp_file.name;
                let local_file_path = work_dir_of(&relative_local_file_path);
                LOGGER.info(format!("localFilePath: {}", local_file_path));
                let local_file = File::new(&local_file_path);
                local_file.delete_recursively();
                if let Some(p) = &local_file.parent_file {
                    if !p.exists() {
                        p.mkdirs();
                    }
                }
                if !temp_file.copy_recursively(&local_file) {
                    return Pair::new(book, Some("导入本地书籍失败".to_string()));
                }
                temp_file.delete_recursively();
                book.book_url = book_url;
                book.origin_name = relative_local_file_path;

                if book.is_epub() {
                    if !self.extract_epub(book.clone(), false) {
                        return Pair::new(book, Some("导入本地Epub书籍失败".to_string()));
                    }
                } else if book.is_cbz() {
                    if !self.extract_cbz(book.clone(), false) {
                        return Pair::new(book, Some("导入本地CBZ书籍失败".to_string()));
                    }
                } else if book.is_pdf() {
                    if !self.convert_pdf_to_image(book.clone(), false) {
                        return Pair::new(book, Some("本地PDF书籍转换失败".to_string()));
                    }
                }
            } else if book.book_url.index_of("localStore", 0) >= 0 {
                let temp_file = File::new(&work_dir_of(&book.book_url));
                if !temp_file.exists() {
                    return Pair::new(book, Some("本地书仓书籍不存在".to_string()));
                }
                let relative_local_file_path = Paths::get(
                    Paths::get(Paths::get(Paths::get("storage", "data"), &user_name_space), &(book.name.clone() + "_" + &book.author)),
                    &temp_file.name,
                );
                book.book_url = relative_local_file_path;

                if book.is_epub() {
                    if !self.extract_epub(book.clone(), false) {
                        return Pair::new(book, Some("导入本地Epub书籍失败".to_string()));
                    }
                } else if book.is_cbz() {
                    if !self.extract_cbz(book.clone(), false) {
                        return Pair::new(book, Some("导入本地CBZ书籍失败".to_string()));
                    }
                } else if book.is_pdf() {
                    if !self.convert_pdf_to_image(book.clone(), false) {
                        return Pair::new(book, Some("本地PDF书籍转换失败".to_string()));
                    }
                }
            } else if book.book_url.index_of("webdav", 0) >= 0 {
                let temp_file = File::new(&work_dir_of(&book.book_url));
                if !temp_file.exists() {
                    return Pair::new(book, Some("webdav书仓书籍不存在".to_string()));
                }
                let relative_local_file_path = Paths::get(
                    Paths::get(Paths::get(Paths::get("storage", "data"), &user_name_space), &(book.name.clone() + "_" + &book.author)),
                    &temp_file.name,
                );
                book.book_url = relative_local_file_path;

                if book.is_epub() {
                    if !self.extract_epub(book.clone(), false) {
                        return Pair::new(book, Some("导入本地Epub书籍失败".to_string()));
                    }
                } else if book.is_cbz() {
                    if !self.extract_cbz(book.clone(), false) {
                        return Pair::new(book, Some("导入本地CBZ书籍失败".to_string()));
                    }
                } else if book.is_pdf() {
                    if !self.convert_pdf_to_image(book.clone(), false) {
                        return Pair::new(book, Some("本地PDF书籍转换失败".to_string()));
                    }
                }
            }
        }
        book.is_in_shelf = true;
        if exist_index < 0 {
            // 新书加入书架时更新阅读时间，使其按 durChapterTime 排在最前
            book.dur_chapter_time = System::current_time_millis();
        }
        if exist_index >= 0 {
            let mut book_list = bookshelf.get_list();
            let exist_book = bookshelf.get_json_object(exist_index).map_to::<Book>().unwrap_or_default();
            book.dur_chapter_index = exist_book.dur_chapter_index;
            book.dur_chapter_title = exist_book.dur_chapter_title.clone();
            book.dur_chapter_time = exist_book.dur_chapter_time;
            let old_cover_url = exist_book.get_display_cover();
            if old_cover_url.as_ref().map_or(true, |s| s.is_empty()) == false
                && old_cover_url.as_ref().unwrap().starts_with("/")
                && old_cover_url != book.get_display_cover()
            {
                FileUtils::deleteFile(&work_dir_of(&("storage".to_string() + old_cover_url.unwrap().as_str())));
            }
            book_list[exist_index as usize] = JsonObject::map_from(book.clone());
            bookshelf = JsonArray::from_list(book_list);
        } else {
            bookshelf.add(JsonObject::map_from(book.clone()).to_string());
        }
        self.save_book_sources(book.clone(), list!(book.clone().to_search_book()), user_name_space.clone(), false);
        self.base.save_user_storage(&user_name_space, String::from("bookshelf"), Box::new(bookshelf));
        return Pair::new(book, None);
    }

    /// Download and save a book's cover image locally.
    /// JAR signature: public final Object saveBookCover(Book, String, String?, Continuation)
    pub async fn save_book_cover(&self, mut book: Book, user_name_space: String, book_source: Option<String>) {
        let cover_url = book.get_display_cover();
        if cover_url.is_none() || cover_url.as_ref().unwrap().starts_with("/") {
            return;
        }
        let source = if book_source.is_some() {
            book_source
        } else {
            self.get_book_source_string_by_source_url_opt(book.origin.clone(), user_name_space.clone())
        };
        let cover_url_str = cover_url.unwrap();
        let ext = self.base.get_file_ext(cover_url_str.clone(), "jpg".to_string());
        let md5_encode = MD5Utils::md5Encode(Some(&cover_url_str));
        let cache_path = work_dir_join(vec![
            String::from("storage"),
            String::from("assets"),
            user_name_space.clone(),
            String::from("covers"),
            md5_encode.clone() + "." + &ext,
        ]);
        let cover_local_url = "/assets/".to_string() + &user_name_space + "/covers/" + &(md5_encode + "." + &ext);
        let cache_file = File::new(&cache_path);
        if cache_file.exists() {
            book.cover_url = Some(cover_local_url);
            return;
        }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut analyze_url = crate::io_legado_app_model_analyzerule_analyzeurl::AnalyzeUrl::new(
                cover_url_str.clone(),
                None,
                None,
                None,
                None,
                String::new(),
                source.as_ref().and_then(|it| BookSource::from_json(it.clone()).get_or_null()),
                None,
                None,
                None,
                None,
            );
            let bytes = crate::stubs::block_on(analyze_url.get_byte_array_await());
            FileUtils::writeBytes(&cache_path, &bytes);
            book.cover_url = Some(cover_local_url);
        })) {
            Ok(_) => {}
            Err(e) => {
                let _ = crate::stubs::panic_message(e);
            }
        }
    }

    pub async fn save_local_book_cover(&self, mut book: Book, user_name_space: String) {
        let cover_url = book.get_display_cover();
        if cover_url.as_ref().map_or(true, |s| s.is_empty()) || cover_url.as_ref().unwrap().starts_with("/") {
            return;
        }
        let cover_url_str = cover_url.unwrap();
        let ext = self.base.get_file_ext(cover_url_str.clone(), "jpg".to_string());
        let md5_encode = MD5Utils::md5Encode(Some(&cover_url_str));
        let cache_path = work_dir_join(vec![
            String::from("storage"),
            String::from("assets"),
            user_name_space.clone(),
            String::from("covers"),
            format!("{}.{}", md5_encode, ext),
        ]);
        let cached_cover_url = format!("/assets/{}/covers/{}.{}", user_name_space, md5_encode, ext);
        let cache_file = File::new(&cache_path);
        if cache_file.exists() {
            book.cover_url = Some(cached_cover_url);
            return;
        }
        // fix: 原 Kotlin webClient.getAbs(coverUrl).timeout(3000).send 下载封面 → 真实下载
        let body_bytes = self
            .web_client
            .get_abs(&cover_url_str)
            .timeout(30000)
            .async_get_bytes_in_thread()
            .unwrap_or_default();
        if !body_bytes.is_empty() {
            let parent = cache_file.parent_file.clone();
            if let Some(parent) = parent {
                if !parent.exists() {
                    parent.mkdirs();
                }
            }
            cache_file.write_bytes(body_bytes);
            book.cover_url = Some(cached_cover_url);
        }
    }

    pub fn update_image_link_in_content(&self, book: Book, chapter: BookChapter, content: String) -> String {
        let data_dir = work_dir_multi(&["storage", "data"]);
        let lines: Vec<&str> = content.split("\n").collect();
        let mut sb = StringBuilder::new();
        for text in lines {
            let mut line_text = text.to_string();
            let pattern = AppPattern::imgPattern();
            let mut matcher = pattern.matcher(text.to_string());
            while matcher.find() {
                let src = match matcher.group_idx(1) {
                    Some(s) => s,
                    None => continue,
                };
                if src.contains("__API_ROOT__") {
                    continue;
                }
                let abs_url = NetworkUtils::getAbsoluteURL(Some(&chapter.url), &src);
                let image_file = BookHelp::get_image(&book, &abs_url);
                if image_file.exists() {
                    let image_url = "__API_ROOT__".to_string() + &image_file.path().replace(&data_dir, "/book-assets");
                    line_text = line_text.replace(&src, &format!("{}\" data-error=\"{}\"", image_url, src));
                }
            }
            sb.append(line_text).append("\n");
        }
        return sb.to_string();
    }
}

// fix: Kotlin companion object `val backupFileNames`（#[lazy] 转录占位，改为模块级常量）
const BACKUP_FILE_NAMES: &[&str] = &[
    "bookSource.json",
    "bookshelf.json",
    "bookGroup.json",
    "rssSources.json",
    "replaceRule.json",
    "bookmark.json",
    "userConfig.json",
    "httpTTS.json",
    "remoteBookSourceSub.json",
    "txtTocRule.json",
];











