// Translated from BookController.kt
// package com.htmake.reader.api.controller
//
// import io.legado.app.constant.AppPattern
// import io.legado.app.data.entities.Book
// import io.legado.app.data.entities.BookChapter
// import io.legado.app.data.entities.TxtTocRule
// import io.legado.app.data.entities.SearchBook
// import io.legado.app.data.entities.BookSource
// import io.legado.app.data.entities.RssSource
// import io.legado.app.data.entities.RssArticle
// import io.legado.app.data.entities.SearchResult
// import io.legado.app.exception.TocEmptyException
// import io.legado.app.model.webBook.WebBook
// import io.legado.app.help.DefaultData
// import io.vertx.ext.web.Route
// import io.vertx.ext.web.Router
// import io.vertx.ext.web.RoutingContext
// import io.vertx.ext.web.handler.StaticHandler
// import mu.KotlinLogging
// import com.htmake.reader.config.AppConfig
// import com.htmake.reader.config.BookConfig
// import io.legado.app.constant.DeepinkBookSource
// import com.htmake.reader.utils.error
// import com.htmake.reader.utils.success
// import com.htmake.reader.utils.getStorage
// import com.htmake.reader.utils.saveStorage
// import com.htmake.reader.utils.asJsonArray
// import com.htmake.reader.utils.asJsonObject
// import com.htmake.reader.utils.toDataClass
// import com.htmake.reader.utils.toMap
// import com.htmake.reader.utils.fillData
// import com.htmake.reader.utils.getWorkDir
// import com.htmake.reader.utils.getStorageFile
// import com.htmake.reader.utils.parseJsonStringList
// import com.htmake.reader.utils.getRandomString
// import com.htmake.reader.utils.genEncryptedPassword
// import com.htmake.reader.utils.SpringContextUtils
// import com.htmake.reader.utils.deleteRecursively
// import com.htmake.reader.utils.unzip
// import com.htmake.reader.utils.zip
// import com.htmake.reader.utils.jsonEncode
// import com.htmake.reader.utils.getRelativePath
// import com.htmake.reader.utils.MongoManager
// import com.htmake.reader.utils.UserMutex
// import com.htmake.reader.verticle.RestVerticle
// import com.htmake.reader.SpringEvent
// import org.springframework.stereotype.Component
// import io.vertx.core.json.JsonObject
// import io.vertx.core.json.JsonArray
// import io.vertx.core.http.HttpMethod
// import io.vertx.core.http.HttpServerResponse
// import io.vertx.core.MultiMap
// import com.fasterxml.jackson.core.JsonToken
// import com.fasterxml.jackson.databind.JsonNode
// import com.fasterxml.jackson.databind.ObjectMapper
// import io.legado.app.data.entities.HttpTTS
// import com.htmake.reader.api.ReturnData
// import io.legado.app.utils.MD5Utils
// import io.legado.app.utils.FileUtils
// import java.net.URLDecoder
// import java.net.URLEncoder
// import java.net.URL
// import java.nio.charset.Charset
// import java.util.UUID
// import java.util.Base64
// import io.vertx.ext.web.client.WebClient
// import io.vertx.kotlin.coroutines.awaitResult
// import org.springframework.beans.factory.annotation.Autowired
// import org.springframework.core.env.Environment
// import java.io.File
// import java.io.FileOutputStream
// import java.lang.Runtime
// import kotlin.collections.mutableMapOf
// import kotlin.system.measureTimeMillis
// import kotlin.coroutines.CoroutineContext
// import kotlinx.coroutines.Dispatchers
// import kotlinx.coroutines.launch
// import kotlinx.coroutines.cancel
// import kotlinx.coroutines.CoroutineExceptionHandler
// import java.text.SimpleDateFormat
// import io.legado.app.utils.EncoderUtils
// import io.legado.app.utils.ACache
// import io.legado.app.utils.HtmlFormatter
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.utils.GSON
// import io.legado.app.utils.fromJsonObject
// import io.legado.app.utils.fromJsonArray
// import io.legado.app.model.rss.Rss
// import io.legado.app.model.Debug
// import io.legado.app.model.Debugger
// import io.legado.app.help.BookHelp
// import org.springframework.scheduling.annotation.Scheduled
// import io.legado.app.model.localBook.LocalBook
// import io.legado.app.model.analyzeRule.AnalyzeUrl
// import io.legado.app.model.analyzeRule.AnalyzeRule
// import io.legado.app.exception.NoStackTraceException
// import com.script.ScriptException
// import java.nio.file.Paths
// import java.io.InputStream
// import java.net.ConnectException
// import java.net.SocketTimeoutException
// import kotlinx.coroutines.withContext
// import kotlinx.coroutines.slf4j.MDCContext
// import kotlinx.coroutines.async
// import kotlinx.coroutines.sync.Mutex
// import kotlinx.coroutines.Deferred
// import kotlinx.coroutines.CoroutineScope
// import kotlinx.coroutines.ensureActive
// import me.ag2s.epublib.domain.*
// import me.ag2s.epublib.epub.EpubWriter
// import me.ag2s.epublib.util.ResourceUtil
// import org.mozilla.javascript.WrappedException
// import io.legado.app.help.coroutine.Coroutine

static LOGGER: Log = Log;

struct BookController {
    coroutine_context: CoroutineContext,

    // 缓存 2M 的书籍信息
    book_info_cache: ACache,
    concurrent_loop_count: i32 = 8,

    web_client: WebClient,
}

impl BookController {
    fn new(coroutine_context: CoroutineContext) -> Self {
        BookController {
            book_info_cache: ACache::get("bookInfoCache", 1000 * 1000 * 2, 10000), // 缓存 2M 的书籍信息
            concurrent_loop_count: 8,
            web_client: SpringContextUtils::get_bean("webClient", WebClient::class),
        }
    }

    fn get_invalid_book_source_cache(&self, user_name_space: String) -> ACache {
        let cache_dir = File::new(get_work_dir_multi(&["storage", "cache", "invalidBookSourceCache", user_name_space]));
        // 缓存 5M 的失效书源信息
        let invalid_book_source_cache = ACache::get(cache_dir, 1000 * 1000 * 5, 1000000);
        return invalid_book_source_cache;
    }

    fn is_invalid_book_source(&self, book_source: BookSource, user_name_space: String) -> bool {
        return self.get_invalid_book_source_cache(user_name_space).get_as_string(book_source.book_source_url) != None;
    }

    fn add_invalid_book_source(&self, source_url: String, invalid_info: Map<String, Any>, user_name_space: String) {
        // 保存600秒时间
        self.get_invalid_book_source_cache(user_name_space).put(source_url, json_encode(invalid_info), 600);
    }

    fn get_book_chapters_cache(&self, user_name_space: String) -> ACache {
        let cache_dir = File::new(get_work_dir_multi(&["storage", "cache", "bookChaptersCache", user_name_space]));
        return ACache::get(cache_dir, 1000 * 1000 * 5, 1000000);
    }

    fn web_book(&self, book_source: String, debug_log: bool, user_name_space: String) -> WebBook {
        return WebBook::new(book_source, debug_log, user_name_space = user_name_space);
    }

    async fn get_invalid_book_sources(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let invalid_book_source_cache = self.get_invalid_book_source_cache(user_name_space);
        let cache_dir = File::new(get_work_dir_multi(&["storage", "cache", "invalidBookSourceCache", user_name_space]));
        let files = cache_dir.list_files();
        let invalid_book_source_list = array_list!<Map<String, Any>>();
        if files != None {
            for f in files {
                if let Some(info) = invalid_book_source_cache.get_by_hash_code(f.name) {
                    invalid_book_source_list.add(info.to_map());
                }
            }
        }

        return return_data.set_data(invalid_book_source_list);
    }

    async fn get_book_info(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        let book_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_json_object("searchBook").get_string("bookUrl").unwrap_or_default();
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        LOGGER.info(format!("getBookInfo with bookUrl: {}", book_url));
        let book_info: Option<Book> = None;
        if self.base.check_auth(context) {
            book_info = self.get_shelf_book_by_url(book_url, self.base.get_user_name_space(context));
        }
        if book_info == None {
            // 看看有没有缓存数据
            let book_source: Option<String> = None;
            let cache_info: Option<Book> = book_info_cache.get_as_string(book_url)?.to_map()?.to_data_class();
            if cache_info != None {
                // 使用缓存的书籍信息包含的书源
                book_source = self.get_book_source_string(context, cache_info.origin);
            } else {
                book_source = self.get_book_source_string(context);
            }
            if book_source.map_or(true, |s| s.is_empty()) {
                return return_data.set_error_msg("未配置书源");
            }
            book_info = self.merge_book_cache_info(self.web_book(book_source, self.base.get_app_config_debug_log(), self.base.get_user_name_space(context)).get_book_info(book_url));
        }

        // 缓存书籍信息
        self.save_book_info_cache(array_list![book_info]);
        return return_data.set_data(book_info);
    }

    async fn get_book_cover(&self, context: RoutingContext) {
        let cover_url = context.query_param("path").unwrap_or("".to_string());
        if cover_url.map_or(true, |s| s.is_empty()) {
            context.response().set_status_code(404).end();
            return;
        }
        let ext = self.base.get_file_ext(cover_url, "png");
        let md5_encode = MD5Utils::md5_encode(cover_url).to_string();
        let cache_path = get_work_dir("storage", "cache", "bookCoverCache", md5_encode + "." + ext);
        let cache_file = File::new(cache_path);
        if cache_file.exists() {
            LOGGER.info(format!("send cache: {}", cache_file));
            context.response().put_header("Cache-Control", "86400").send_file(cache_file.to_string());
            return;
        }

        if !cache_file.parent_file.exists() {
            cache_file.parent_file.mkdirs();
        }

        launch(MDCContext() + Dispatchers::IO + CoroutineExceptionHandler |_, exception| {
            LOGGER.info(format!("get cover error: {}", exception.message));
            context.response().set_status_code(404).end();
        }) {
            web_client.get_abs(cover_url)
                .put_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .put_header("Referer", cover_url.substring_before_last("/"))
                .timeout(10000).send {
                let body_bytes = it.result()?.body_as_buffer()?.get_bytes();
                if body_bytes != None {
                    let res = context.response().put_header("Cache-Control", "86400");
                    cache_file.write_bytes(body_bytes);
                    res.send_file(cache_file.to_string());
                } else {
                    context.response().set_status_code(404).end();
                }
            }
        }
    }

    async fn import_book_preview(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        if context.file_uploads() == None || context.file_uploads().is_empty() {
            return return_data.set_error_msg("请上传书籍文件");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let file_list = array_list!<Map<String, Any>>();
        for it in context.file_uploads() {
            let file = File::new(it.uploaded_file_name());
            LOGGER.info(format!("uploadFile: {} {} {}", it.uploaded_file_name()), it.file_name(), file);
            if file.exists() {
                let mut file_name = it.file_name();
                let ext = self.base.get_file_ext(file_name);
                if ext != "txt" && ext != "epub" && ext != "umd" && ext != "cbz" && ext != "pdf" {
                    file.delete_recursively();
                    return return_data.set_error_msg("不支持导入" + ext + "格式的书籍文件");
                }
                // 文件名格式化
                file_name = FileUtils::get_name_exclude_extension(file_name);
                file_name = file_name.replace(AppPattern::fileNameRegex, "");
                file_name = file_name.substring(0, Math::min(50, file_name.length)) + "." + ext;

                let local_file_path = Paths::get("storage", "assets", user_name_space, "book", file_name).to_string();
                let local_file_url = "/assets/" + user_name_space + "/book/" + file_name;
                let mut file_path = local_file_path;
                if file_name.ends_with(".epub", true) {
                    file_path = file_path + File::separator + "index.epub";
                }
                if file_name.ends_with(".cbz", true) {
                    file_path = file_path + File::separator + "index.cbz";
                }
                let new_file = File::new(get_work_dir(file_path));
                if !new_file.parent_file.exists() {
                    new_file.parent_file.mkdirs();
                }
                if new_file.exists() {
                    new_file.delete();
                }
                LOGGER.info(format!("moveTo: {}", new_file));
                if file.copy_recursively(new_file) {
                    let book = Book::init_local_book(local_file_url, local_file_path, get_work_dir());
                    book.set_user_name_space(user_name_space);
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let chapters = LocalBook::get_chapter_list(book);
                        file_list.add(map!("book" to book, "chapters" to chapters));
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                        file_list.add(map!("book" to book, "chapters" to array_list!<i32>()));
                    }
        }
                }
                file.delete_recursively();
            }
        }
        return return_data.set_data(file_list);
    }

    async fn get_txt_toc_rules(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let rules = ArrayList::<TxtTocRule>::new();
        rules.add_all(DefaultData::txtTocRules);
        let custom_rules = GSON.from_json_array::<TxtTocRule>(self.base.get_user_storage(self.base.get_user_name_space(context), "txtTocRule"))
            .get_or_null()
            ?: empty_list();
        rules.add_all(custom_rules);
        return return_data.set_data(rules);
    }

    async fn get_chapter_list_by_rule(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book = context.body_as_json().map_to::<Book>();
        if book.origin.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("未找到书源信息");
        }
        if !book.is_local_txt() && !book.is_local_epub() && !book.is_local_pdf() {
            return return_data.set_error_msg("非本地txt/epub/pdf书籍");
        }
        book.set_root_dir(get_work_dir());
        book.set_user_name_space(self.base.get_user_name_space(context));
        let chapters = LocalBook::get_chapter_list(book);
        return return_data.set_data(map!("book" to book, "chapters" to chapters));
    }

    async fn refresh_local_book(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("bookUrl");
        } else {
            // get 请求
            book_url = context.query_param("bookUrl").unwrap_or("".to_string());
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        // 根据书籍url获取书本信息
        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info == None {
            return return_data.set_error_msg("书籍信息错误");
        }
        book_info.update_from_local(true);

        self.edit_shelf_book(book_info, user_name_space, |exist_book| {
            exist_book.cover_url = book_info.cover_url;
            LOGGER.info(format!("refreshLocalBook: {}", exist_book));
            exist_book
        })

        return return_data.set_data(book_info);
    }

    async fn get_chapter_list(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let mut refresh: i32 = 0;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_json_object("book").get_string("bookUrl").unwrap_or_default();
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            refresh = context.query_param("refresh")?.to_int().unwrap_or(0);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        // 根据书籍url获取书本信息
        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        let book_source: Option<String> = None;
        if book_info == None {
            // 看看有没有缓存数据
            let cache_info: Option<Book> = book_info_cache.get_as_string(book_url)?.to_map()?.to_data_class();
            if cache_info != None {
                // 使用缓存的书籍信息包含的书源
                book_source = self.get_book_source_string(context, cache_info.origin);
            } else {
                // 看看有没有传入书源
                book_source = self.get_book_source_string(context);
            }
            if book_source.map_or(true, |s| s.is_empty()) {
                return return_data.set_error_msg("未配置书源");
            }
            book_info = self.merge_book_cache_info(self.web_book(book_source, self.base.get_app_config_debug_log(), user_name_space).get_book_info(book_url));
            // 缓存书籍信息
            self.save_book_info_cache(array_list![book_info]);
        } else {
            book_source = self.get_book_source_string(context, book_info.origin);
        }
        if !book_info.is_local_book() && book_source.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("未配置书源");
        }
        book_info.set_root_dir(get_work_dir());
        book_info.set_user_name_space(user_name_space);
        if book_info.is_local_book() {
            let local_file = book_info.get_local_file();
            if !local_file.exists() {
                LOGGER.info(format!("localFile: {} not exists", local_file));
                return return_data.set_error_msg("本地书籍源文件不存在");
            }
        }
        // 缓存章节列表
        LOGGER.info(format!("bookInfo: {}", book_info));
        let chapter_list = self.get_local_chapter_list(book_info, book_source.unwrap_or(""), refresh > 0, self.base.get_user_name_space(context), false);

        return return_data.set_data(chapter_list);
    }

    async fn save_book_progress(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let chapter_index: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_json_object("searchBook").get_string("bookUrl").unwrap_or_default();
            chapter_index = context.body_as_json().get_integer("index", -1);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            chapter_index = context.query_param("index")?.to_int().unwrap_or(-1);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        let user_name_space = self.base.get_user_name_space(context);
        // 看看有没有加入书架
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info == None || book_info.origin.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("书籍未加入书架");
        }
        let book_source = self.get_book_source_string_by_source_url_opt(book_info.origin, user_name_space);

        if !book_info.is_local_book() && book_source.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("未配置书源");
        }
        let chapter_list = self.get_local_chapter_list(book_info, book_source.unwrap_or(""), false, user_name_space, false);
        if chapter_index >= chapter_list.size {
            return return_data.set_error_msg("章节不存在");
        }
        let chapter_info = chapter_list.get(chapter_index);
        // 书架书籍保存阅读进度
        self.save_shelf_book_progress(book_info, chapter_info, user_name_space);
        // 保存到 webdav
        self.save_book_progress_to_webdav(book_info, chapter_info, user_name_space);
        return return_data.set_data("");
    }

    async fn save_book_config(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let pdf_image_width: f32;
        if context.request().method() == HttpMethod::POST {
            book_url = context.body_as_json().get_string("bookUrl").unwrap_or_default();
            pdf_image_width = context.body_as_json().get_float("pdfImageWidth", 0.0);
        } else {
            book_url = context.query_param("bookUrl").unwrap_or("".to_string());
            pdf_image_width = context.query_param("pdfImageWidth")?.to_float_or_null().unwrap_or(0.0);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("书籍链接不能为空");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space)
            ?: return return_data.set_error_msg("书籍信息错误");
        if pdf_image_width <= 0.0 {
            return return_data.set_error_msg("pdf图片宽度错误");
        }
        let new_book = self.edit_shelf_book(book_info, user_name_space, |exist_book| {
            exist_book.set_pdf_image_width(pdf_image_width);
            LOGGER.info(format!("saveBookConfig: {}", exist_book));
            exist_book
        })
        return return_data.set_data(new_book.unwrap_or(book_info));
    }

    async fn get_book_content(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let chapter_url: String;
        let book_url: String;
        let chapter_index: i32;
        let cache: i32;
        let refresh: i32;
        let epub_content: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            chapter_url = context.body_as_json().get_string("chapterUrl").unwrap_or(context.body_as_json)().get_json_object("bookChapter")?.get_string("url").unwrap_or_default();
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_json_object("searchBook")?.get_string("bookUrl").unwrap_or_default();
            chapter_index = context.body_as_json().get_integer("index", -1);
            cache = context.body_as_json().get_integer("cache", 0);
            refresh = context.body_as_json().get_integer("refresh", 0);
            epub_content = context.body_as_json().get_integer("epubContent", 0);
        } else {
            // get 请求
            chapter_url = context.query_param("chapterUrl").unwrap_or("".to_string());
            book_url = context.query_param("url").unwrap_or("".to_string());
            chapter_index = context.query_param("index")?.to_int().unwrap_or(-1);
            cache = context.query_param("cache")?.to_int().unwrap_or(0);
            refresh = context.query_param("refresh")?.to_int().unwrap_or(0);
            epub_content = context.query_param("epubContent")?.to_int().unwrap_or(0);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        let mut book_source = self.get_book_source_string(context);
        let user_name_space = self.base.get_user_name_space(context);
        let mut is_in_book_shelf = false;
        let mut book_info: Option<Book> = None;
        let mut chapter_info: Option<BookChapter> = None;
        let mut next_chapter_url: Option<String> = None;
        if !book_url.map_or(true, |s| s.is_empty()) {
            // 看看有没有加入书架
            book_info = self.get_shelf_book_by_url(book_url, user_name_space);
            if book_info != None && !book_info.origin.map_or(true, |s| s.is_empty()) {
                is_in_book_shelf = true;
                book_source = self.get_book_source_string_by_source_url_opt(book_info.origin, user_name_space);
            }
            // 看看有没有缓存数据
            let cache_info: Option<Book> = book_info_cache.get_as_string(book_url)?.to_map()?.to_data_class();
            if cache_info != None {
                // 使用缓存的书籍信息包含的书源
                book_source = self.get_book_source_string(context, cache_info.origin);
            }
            if chapter_url.map_or(true, |s| s.is_empty()) && chapter_index >= 0 {
                // 根据 url 和 index 获取章节内容
                if book_url.map_or(true, |s| s.is_empty()) {
                    return return_data.set_error_msg("请输入书籍链接");
                }
                if book_info != None && !book_info.is_local_book() && book_source.map_or(true, |s| s.is_empty()) {
                    return return_data.set_error_msg("未配置书源");
                }
                book_info = book_info.unwrap_or(self.merge_book_cache_info(self.web_book(book_source.unwrap_or_default()), self.base.get_app_config_debug_log(), user_name_space).get_book_info(book_url));
                let chapter_list = self.get_local_chapter_list(book_info, book_source.unwrap_or(""), false, user_name_space, false);
                if chapter_index < chapter_list.size {
                    chapter_info = chapter_list.get(chapter_index);
                    // 书架书籍保存阅读进度
                    if is_in_book_shelf && cache != 1 {
                        self.save_shelf_book_progress(book_info, chapter_info, user_name_space);
                        // 保存到 webdav
                        self.save_book_progress_to_webdav(book_info, chapter_info, user_name_space);
                    }
                    chapter_url = chapter_info.url;
                    if chapter_index + 1 < chapter_list.size {
                        let next_chapter_info = chapter_list.get(chapter_index + 1);
                        next_chapter_url = next_chapter_info.url;
                    }
                }
            }
        }
        if book_info == None {
            return return_data.set_error_msg("获取书籍信息失败");
        }
        if !book_info.is_local_book() && book_source.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("未配置书源");
        }
        if chapter_info == None || chapter_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("获取章节链接失败");
        }

        let mut content = "";
        book_info.set_root_dir(get_work_dir());
        book_info.set_user_name_space(user_name_space);
        if book_info.is_local_book() {
            let local_file = book_info.get_local_file();
            if !local_file.exists() {
                return return_data.set_error_msg("本地源书籍文件不存在");
            }
            if chapter_info == None {
                let chapter_list = self.get_local_chapter_list(book_info, book_source.unwrap_or(""), false, user_name_space, false);
                for i in 0..chapter_list.size {
                    if chapter_url == chapter_list.get(i).url {
                        chapter_info = chapter_list.get(i);
                        break;
                    }
                }
                if chapter_info == None {
                    return return_data.set_error_msg("获取章节信息失败");
                }
            }
            if book_info.is_epub() {
                if !self.extract_epub(book_info) {
                    return return_data.set_error_msg("Epub书籍解压失败");
                }

                let epub_root_dir = book_info.get_epub_root_dir();
                let chapter_file_path = get_work_dir(book_info.book_url, "index", epub_root_dir, chapter_info.url);
                LOGGER.info(format!("chapterFilePath: {} {}", chapter_file_path, epub_root_dir));
                if !File::new(chapter_file_path).exists() {
                    return return_data.set_error_msg("章节文件不存在");
                }
                // 处理 js 注入脚本
                // BookConfig.injectJavascriptToEpubChapter(chapterFilePath);

                // 直接返回 html访问地址
                let public_book_url = book_info.book_url.replace("\\", "/").replace("storage/data/", "/book-assets/");
                if epub_root_dir.is_empty() {
                    content = public_book_url + "/index/" + chapter_info.url;
                } else {
                    content = public_book_url + "/index/" + epub_root_dir + "/" + chapter_info.url;
                }
                if epub_content > 0 {
                    return return_data.set_data(
                        map!(
                            "url" to "__API_ROOT__" + content,
                            "content" to File::new(chapter_file_path).read_text()
                        )
                    );
                }
                return return_data.set_data(content);
            } else if book_info.is_cbz() {
                if !self.extract_cbz(book_info) {
                    return return_data.set_error_msg("CBZ书籍解压失败");
                }
                let chapter_file_path = get_work_dir(book_info.book_url, "index", chapter_info.url);
                LOGGER.info(format!("chapterFilePath: {}", chapter_file_path));
                let chapter_file = File::new(chapter_file_path);
                if !chapter_file.exists() {
                    return return_data.set_error_msg("章节文件不存在");
                }
                let ext = self.base.get_file_ext(chapter_file.name).to_lowercase();
                let image_ext = list!("jpg", "jpeg", "gif", "png", "bmp", "webp", "svg");
                let file_url = "__API_ROOT__" + book_info.book_url.replace("\\", "/").replace("storage/data/", "/book-assets/") + "/index/" + chapter_info.url;
                if !image_ext.contains(ext) {
                    return return_data.set_data(file_url);
                }
                content = "<img src='" + file_url + "' />";
                return return_data.set_data(content);
            }
            if book_info.is_pdf() {
                if !self.convert_pdf_to_image(book_info) {
                    return return_data.set_error_msg("PDF生成图片失败");
                }
                let start = chapter_info.start;
                let end = chapter_info.end;
                if start != None && end != None && start <= end {
                    let public_book_url = book_info.book_url.replace("\\", "/").replace("storage/data/", "/book-assets/");
                    for page in start..end {
                        self.convert_pdf_page_to_image(book_info, page.to_int(), refresh > 0);
                        let page_file = File::new(get_work_dir(book_info.book_url, "index", "output-$page.png"));
                        LOGGER.info(format!("chapterFilePath: {}", page_file.absolute_path));
                        if !page_file.exists() {
                            return return_data.set_error_msg("章节文件不存在");
                        }
                        let file_url = "__API_ROOT__" + public_book_url + "/index/output-$page.png";
                        content += "<img src='" + file_url + "' />";
                    }
                }
                return return_data.set_data(content);
            }
            let book_content = LocalBook::get_content(book_info, chapter_info);
            if book_content == None {
                return return_data.set_error_msg("获取章节内容失败");
            }
            content = book_content;
        } else {
            // 查找章节缓存
            let mut chapter_cache_file: Option<File> = None;
            if book_info.is_in_shelf && refresh <= 0 && app_config.cache_chapter_content {
                let local_cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
                chapter_cache_file = File::new(local_cache_dir.absolute_path + File::separator + chapter_index + ".txt");
                if chapter_cache_file.exists() {
                    content = chapter_cache_file.read_text();
                    if content.contains("<img") {
                        content = self.update_image_link_in_content(book_info, chapter_info, content);
                    }
                    LOGGER.info(format!("使用缓存的章节内容: {}", chapter_cache_file.to_string()));
                    return return_data.set_data(content);
                }
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                content = self.web_book(book_source.unwrap_or(""), self.base.get_app_config_debug_log(), user_name_space).get_book_content(book_info, chapter_info, next_chapter_url);
                if app_config.cache_chapter_content && chapter_cache_file != None {
                    chapter_cache_file.write_text(content);
                    // 保存图片
                    BookHelp::save_images(
                        self,
                        BookSource::from_json(book_source.unwrap_or("")).get_or_null().unwrap_or(BookSource)::new(),
                        book_info,
                        chapter_info,
                        content
                    );
                    content = self.update_image_link_in_content(book_info, chapter_info, content);
                }
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                if !book_source.map_or(true, |s| s.is_empty()) {
                    let book_source_object = as_json_object(book_source)?.map_to::<BookSource>();
                    if book_source_object != None {
                        // 标记为失败源
                        let info = mutable_map_of!("sourceUrl" to book_source_object.book_source_url, "time" to System::current_time_millis(), "error" to e.to_string());
                        self.add_invalid_book_source(book_source_object.book_source_url, info, user_name_space);
                    }
                }
                throw e;
            }
        }
        }

        return return_data.set_data(content);
    }

    async fn explore_book(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        // 如果登录了，就使用用户的书源
        self.base.check_auth(context);
        let book_source = self.get_book_source_string(context);
        if book_source.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("未配置书源");
        }
        let page: i32;
        let rule_find_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            rule_find_url = context.body_as_json().get_string("ruleFindUrl");
            page = context.body_as_json().get_integer("page", 1);
        } else {
            // get 请求
            rule_find_url = context.query_param("ruleFindUrl").unwrap_or("".to_string());
            page = context.query_param("page")?.to_int().unwrap_or(1);
        }

        let result = self.web_book(book_source, false, self.base.get_user_name_space(context)).explore_book(rule_find_url, page);
        return return_data.set_data(result);
    }

    async fn search_book(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        // 如果登录了，就使用用户的书源
        self.base.check_auth(context);
        let book_source = self.get_book_source_string(context);
        if book_source.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("未配置书源");
        }
        let key: String;
        let page: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            key = context.body_as_json().get_string("key");
            page = context.body_as_json().get_integer("page", 1);
        } else {
            // get 请求
            key = context.query_param("key").unwrap_or("".to_string());
            page = context.query_param("page")?.to_int().unwrap_or(1);
        }
        if key.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入搜索关键字");
        }
        LOGGER.info { "searchBook" };
        let result = self.web_book(book_source, self.base.get_app_config_debug_log(), self.base.get_user_name_space(context)).search_book(key, page);
        return return_data.set_data(result);
    }

    async fn search_book_multi(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let mut key: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        let mut concurrent_count: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            key = context.body_as_json().get_string("key", "");
            book_source_group = context.body_as_json().get_string("bookSourceGroup", "");
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 20);
            concurrent_count = context.body_as_json().get_integer("concurrentCount", 36);
        } else {
            // get 请求
            key = context.query_param("key").unwrap_or("".to_string());
            book_source_group = context.query_param("bookSourceGroup").unwrap_or("".to_string());
            last_index = context.query_param("lastIndex")?.to_int().unwrap_or(-1);
            search_size = context.query_param("searchSize")?.to_int().unwrap_or(20);
            concurrent_count = context.query_param("concurrentCount")?.to_int().unwrap_or(36);
        }
        let user_name_space = self.base.get_user_name_space(context);
        let url_map = BookSourceController::new(self.coroutine_context).get_book_source_map(user_name_space);
        if url_map.is_empty() {
            return return_data.set_error_msg("未配置书源");
        }
        if key.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入搜索关键字");
        }
        let mut accurate = false;
        if key.starts_with("=", ignore_case = true) {
            accurate = true;
            key = key.replace_first("=", "");
        }
        if key.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入搜索关键字");
        }
        if last_index >= url_map.size - 1 {
            return return_data.set_error_msg("没有更多了");
        }

        search_size = if search_size > 0 { search_size } else { 20 };
        concurrent_count = if concurrent_count > 0 { concurrent_count } else { 36 };
        LOGGER.info(format!("searchBookMulti from lastIndex: {} searchSize: {}", last_index, search_size));
        let mut is_end = false;
        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 searchBookMulti"));
            is_end = true;
            self.coroutine_context.cancel();
        }
        let mut result_list = array_list!<SearchBook>();
        let mut result_map = mutable_map_of!<String, i32>();
        let book = Book::new();
        book.name = key;
        let book_source_file = get_storage_file("data", user_name_space, "bookSource").let {
            if it.exists() { it } else { get_storage_file("data", "default", "bookSource") }
        };
        let mut max_size = url_map.size;
        self.base.limit_concurrent(concurrent_count, last_index + 1, url_map.size, |it| {
            if it <= max_size {
                last_index = Math::max(last_index, it);
                let book_source_list = parse_json_string_list(
                    book_source_file,
                    start_index = it,
                    end_index = it,
                    filter = if book_source_group.is_empty() { None } else { |node| {
                        let source_group = node.get("bookSourceGroup")?.as_text().unwrap_or("");
                        source_group.is_not_empty() && (source_group + ",").contains(book_source_group + ",")
                    } }
                );
                if book_source_list == None || book_source_list.is_empty {
                    max_size = it;
                    empty_list::<SearchBook>()
                } else {
                    self.search_book_with_source(book_source_list.get_string(0), book, accurate, user_name_space)
                }
            } else {
                empty_list::<SearchBook>()
            }
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            for it in list {
                let book_list = it as? Collection<SearchBook>;
                if let Some(book_list) = book_list {
                    for book in book_list {
                        // 按照 书名 + 作者名 过滤
                        let book_key = book.name + '_' + book.author;
                        if !result_map.contains_key(book_key) {
                            result_list.add(book);
                            result_map.put(book_key, 1);
                        }
                    }
                }
            }
            LOGGER.info(format!("Loog: {} resultList.size: {}", loop_count, result_list.size));
            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.size < search_size
            }
        });
        return return_data.set_data(map!("lastIndex" to last_index, "list" to result_list));
    }

    async fn search_book_multi_sse(&self, context: RoutingContext) {
        let return_data = ReturnData::new();
        // 返回 event-stream
        let response = context.response().put_header("Content-Type", "text/event-stream")
            .put_header("Cache-Control", "no-cache")
            .set_chunked(true);
        if !self.base.check_auth(context) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用"), false) + "\n\n");
            return;
        }
        let mut key: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        let mut concurrent_count: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            key = context.body_as_json().get_string("key", "");
            book_source_group = context.body_as_json().get_string("bookSourceGroup", "");
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 50);
            concurrent_count = context.body_as_json().get_integer("concurrentCount", 24);
        } else {
            // get 请求
            key = context.query_param("key").unwrap_or("".to_string());
            book_source_group = context.query_param("bookSourceGroup").unwrap_or("".to_string());
            last_index = context.query_param("lastIndex")?.to_int().unwrap_or(-1);
            search_size = context.query_param("searchSize")?.to_int().unwrap_or(50);
            concurrent_count = context.query_param("concurrentCount")?.to_int().unwrap_or(24);
        }
        let user_name_space = self.base.get_user_name_space(context);
        let url_map = BookSourceController::new(self.coroutine_context).get_book_source_map(user_name_space);
        if url_map.is_empty() {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("未配置书源"), false) + "\n\n");
            return;
        }
        if key.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("请输入搜索关键字"), false) + "\n\n");
            return;
        }
        let mut accurate = false;
        if key.starts_with("=", ignore_case = true) {
            accurate = true;
            key = key.replace_first("=", "");
        }
        if key.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("请输入搜索关键字"), false) + "\n\n");
            return;
        }
        if last_index >= url_map.size - 1 {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("没有更多了"), false) + "\n\n");
            return;
        }

        search_size = if search_size > 0 { search_size } else { 50 };
        concurrent_count = if concurrent_count > 0 { concurrent_count } else { 24 };
        LOGGER.info(format!("searchBookMulti from lastIndex: {} concurrentCount: {} searchSize: {}", last_index, concurrent_count, search_size));

        let mut is_end = false;
        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 searchBookMultiSSE"));
            is_end = true;
            self.coroutine_context.cancel();
        }
        let mut result_list = array_list!<SearchBook>();
        let book = Book::new();
        book.name = key;
        let book_source_file = get_storage_file("data", user_name_space, "bookSource").let {
            if it.exists() { it } else { get_storage_file("data", "default", "bookSource") }
        };
        let mut max_size = url_map.size;
        self.base.limit_concurrent(concurrent_count, last_index + 1, url_map.size, |it| {
            if it <= max_size {
                last_index = Math::max(last_index, it);
                let book_source_list = parse_json_string_list(
                    book_source_file,
                    start_index = it,
                    end_index = it,
                    filter = if book_source_group.is_empty() { None } else { |node| {
                        let source_group = node.get("bookSourceGroup")?.as_text().unwrap_or("");
                        source_group.is_not_empty() && (source_group + ",").contains(book_source_group + ",")
                    } }
                );
                if book_source_list == None || book_source_list.is_empty {
                    max_size = it;
                    empty_list::<SearchBook>()
                } else {
                    self.search_book_with_source(book_source_list.get_string(0), book, accurate, user_name_space)
                }
            } else {
                empty_list::<SearchBook>()
            }
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            let loop_result = array_list!<SearchBook>();
            for it in list {
                let book_list = it as? Collection<SearchBook>;
                if let Some(book_list) = book_list {
                    for book in book_list {
                        // 按照 书名 + 作者名 过滤
                        result_list.add(book);
                        loop_result.add(book);
                    }
                }
            }
            // 返回本轮数据
            response.write("data: " + json_encode(map!("lastIndex" to last_index, "data" to loop_result), false) + "\n\n");
            LOGGER.info(format!("Loog: {} resultList.size: {}", loop_count, result_list.size));

            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.size < search_size
            }
        });
        response.write("event: end\n");
        response.end("data: " + json_encode(map!("lastIndex" to last_index, "isEnd" to (last_index >= url_map.size)), false) + "\n\n");
    }

    async fn search_book_source(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url");
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 5);
            book_source_group = context.body_as_json().get_string("bookSourceGroup", "");
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            last_index = context.query_param("lastIndex")?.to_int().unwrap_or(-1);
            search_size = context.query_param("searchSize")?.to_int().unwrap_or(5);
            book_source_group = context.query_param("bookSourceGroup").unwrap_or("".to_string());
        }
        let user_name_space = self.base.get_user_name_space(context);
        let url_map = BookSourceController::new(self.coroutine_context).get_book_source_map(user_name_space);
        if url_map.is_empty() {
            return return_data.set_error_msg("未配置书源");
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        if last_index >= url_map.size - 1 {
            return return_data.set_error_msg("没有更多了");
        }
        let mut book = self.get_shelf_book_by_url(book_url, user_name_space);
        if book == None {
            book = book_info_cache.get_as_string(book_url)?.to_map()?.to_data_class();
        }
        if book == None {
            return return_data.set_error_msg("书籍信息错误");
        }
        LOGGER.info(format!("searchBookSource from lastIndex: {}", last_index));
        let mut is_end = false;
        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 searchBookSource"));
            is_end = true;
            self.coroutine_context.cancel();
        }
        search_size = if search_size > 0 { search_size } else { 5 };
        let mut result_list = array_list!<SearchBook>();
        let concurrent_count = Math::max(search_size * 2, 24);
        let book_source_file = get_storage_file("data", user_name_space, "bookSource").let {
            if it.exists() { it } else { get_storage_file("data", "default", "bookSource") }
        };
        let mut max_size = url_map.size;
        self.base.limit_concurrent(concurrent_count, last_index + 1, url_map.size, |it| {
            if it <= max_size {
                last_index = Math::max(last_index, it);
                let book_source_list = parse_json_string_list(
                    book_source_file,
                    start_index = it,
                    end_index = it,
                    filter = if book_source_group.is_empty() { None } else { |node| {
                        let source_group = node.get("bookSourceGroup")?.as_text().unwrap_or("");
                        source_group.is_not_empty() && (source_group + ",").contains(book_source_group + ",")
                    } }
                );
                if book_source_list == None || book_source_list.is_empty {
                    max_size = it;
                    empty_list::<SearchBook>()
                } else {
                    self.search_book_with_source(book_source_list.get_string(0), book, accurate = true, user_name_space = user_name_space)
                }
            } else {
                empty_list::<SearchBook>()
            }
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            for it in list {
                let book_list = it as? Collection<SearchBook>;
                if let Some(book_list) = book_list {
                    result_list.add_all(book_list);
                }
            }
            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.size < search_size
            }
        });
        self.save_book_sources(book, result_list, user_name_space);
        return return_data.set_data(map!("lastIndex" to last_index, "list" to result_list));
    }
    async fn search_book_source_sse(&self, context: RoutingContext) {
        let return_data = ReturnData::new();
        // 返回 event-stream
        let response = context.response().put_header("Content-Type", "text/event-stream")
            .put_header("Cache-Control", "no-cache")
            .set_chunked(true);

        if !self.base.check_auth(context) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用"), false) + "\n\n");
            return;
        }
        let book_url: String;
        let mut last_index: i32;
        let mut search_size: i32;
        let book_source_group: String;
        let mut refresh: i32 = 0;

        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url");
            last_index = context.body_as_json().get_integer("lastIndex", -1);
            search_size = context.body_as_json().get_integer("searchSize", 30);
            book_source_group = context.body_as_json().get_string("bookSourceGroup", "");
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            last_index = context.query_param("lastIndex")?.to_int().unwrap_or(-1);
            search_size = context.query_param("searchSize")?.to_int().unwrap_or(30);
            book_source_group = context.query_param("bookSourceGroup").unwrap_or("".to_string());
            refresh = context.query_param("refresh")?.to_int().unwrap_or(0);
        }
        let user_name_space = self.base.get_user_name_space(context);
        let url_map = BookSourceController::new(self.coroutine_context).get_book_source_map(user_name_space);
        if url_map.is_empty() {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("未配置书源"), false) + "\n\n");
            return;
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("请输入书籍链接"), false) + "\n\n");
            return;
        }

        let mut book = self.get_shelf_book_by_url(book_url, user_name_space);
        if book == None {
            book = book_info_cache.get_as_string(book_url)?.to_map()?.to_data_class();
        }
        if book == None {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("书籍信息错误"), false) + "\n\n");
            return;
        }
        if last_index >= url_map.size - 1 {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_data(map!("lastIndex" to last_index)).set_error_msg("没有更多了"), false) + "\n\n");
            return;
        }

        search_size = if search_size > 0 { search_size } else { 30 };
        let mut result_list = array_list!<SearchBook>();
        let concurrent_count = Math::max(search_size * 2, 24);
        LOGGER.info(format!("searchBookMulti from lastIndex: {} concurrentCount: {} searchSize: {}", last_index, concurrent_count, search_size));
        let mut is_end = false;
        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 searchBookSourceSSE"));
            is_end = true;
            self.coroutine_context.cancel();
        }

        let book_source_file = get_storage_file("data", user_name_space, "bookSource").let {
            if it.exists() { it } else { get_storage_file("data", "default", "bookSource") }
        };
        let mut max_size = url_map.size;
        self.base.limit_concurrent(concurrent_count, last_index + 1, max_size, |it| {
            if it <= max_size {
                last_index = Math::max(last_index, it);
                let book_source_list = parse_json_string_list(
                    book_source_file,
                    start_index = it,
                    end_index = it,
                    filter = if book_source_group.is_empty() { None } else { |node| {
                        let source_group = node.get("bookSourceGroup")?.as_text().unwrap_or("");
                        source_group.is_not_empty() && (source_group + ",").contains(book_source_group + ",")
                    } }
                );
                if book_source_list == None || book_source_list.is_empty {
                    max_size = it;
                    empty_list::<SearchBook>()
                } else {
                    self.search_book_with_source(book_source_list.get_string(0), book, accurate = true, user_name_space = user_name_space)
                }
            } else {
                empty_list::<SearchBook>()
            }
        }, |list, loop_count| {
            // logger.info("list: {}", list)
            let loop_result = array_list!<SearchBook>();
            for it in list {
                let book_list = it as? Collection<SearchBook>;
                if let Some(book_list) = book_list {
                    result_list.add_all(book_list);
                    loop_result.add_all(book_list);
                }
            }
            // 返回本轮数据
            response.write("data: " + json_encode(map!("lastIndex" to last_index, "data" to loop_result), false) + "\n\n");
            LOGGER.info(format!("Loog: {} resultList.size: {}", loop_count, result_list.size));

            if is_end || loop_count >= self.concurrent_loop_count {
                // 超过最大轮次，终止执行
                false
            } else {
                result_list.size < search_size
            }
        });
        self.save_book_sources(book, result_list, user_name_space);
        response.write("event: end\n");
        response.end("data: " + json_encode(map!("lastIndex" to last_index, "isEnd" to (last_index >= max_size)), false) + "\n\n");
    }

    async fn search_book_with_source(&self, book_source_string: String, book: Book, accurate: bool = true, user_name_space: String = "default") -> ArrayList<SearchBook> {
        let mut result_list = array_list!<SearchBook>();
        let book_source = as_json_object(book_source_string)?.map_to::<BookSource>();
        if book_source == None {
            return result_list;
        }
        if self.is_invalid_book_source(book_source, user_name_space) {
            return result_list;
        }
        with_context(Dispatchers::IO, || {
            // val costTime = measureTimeMillis {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let start = System::current_time_millis();
                let mut result = self.web_book(book_source_string, false, user_name_space).search_book(book.name, 1);
                let end = System::current_time_millis();
                if result.size > 0 {
                    for j in 0..result.size {
                        let mut _book = result.get(j);
                        if accurate && _book.name.equals(book.name) &&
                            (book.author.map_or(true, |s| s.is_empty()) || _book.author.equals(book.author)) {
                            _book.time = end - start;
                            result_list.add(_book);
                        } else if !accurate && (_book.name.index_of(book.name, ignore_case = true) >= 0 || _book.author.index_of(book.name, ignore_case = true) >= 0) {
                            _book.time = end - start;
                            result_list.add(_book);
                        }
                    }
                }
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                // 标记为失败源
                let info = mutable_map_of!("sourceUrl" to book_source.book_source_url, "time" to System::current_time_millis(), "error" to e.to_string());
                self.add_invalid_book_source(book_source.book_source_url, info, user_name_space);

                e.print_stack_trace();
            }
        }
            // }
        });
        return result_list;
    }

    async fn get_available_book_source(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let refresh: i32 = 0;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url");
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            refresh = context.query_param("refresh")?.to_int().unwrap_or(0);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let mut book = self.get_shelf_book_by_url(book_url, user_name_space);
        if book == None {
            book = book_info_cache.get_as_string(book_url)?.to_map()?.to_data_class();
        }
        if book == None {
            return return_data.set_error_msg("书籍信息错误");
        }
        let mut book_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, book.name + "_" + book.author, "bookSource"));
        if book_source_list != None && book_source_list.size() > 0 {
            if refresh <= 0 {
                return return_data.set_data(book_source_list.get_list());
            }

            // 刷新源
            let mut result_list = array_list!<SearchBook>();
            let concurrent_count = 16;
            self.base.limit_concurrent(concurrent_count, 0, book_source_list.size(), |it| {
                let search_book = book_source_list.get_json_object(it).map_to::<SearchBook>();
                if search_book.origin.equals("loc_book") {
                    array_list!(search_book)
                } else {
                    let book_source = self.get_book_source_string_by_source_url_opt(search_book.origin, user_name_space);
                    if book_source != None {
                        self.search_book_with_source(book_source, book, accurate = true, user_name_space = user_name_space)
                    } else {
                        array_list!::<SearchBook>()
                    }
                }
            }, |list, _| {
                // logger.info("list: {}", list)
                for it in list {
                    let book_list = it as? Collection<SearchBook>;
                    if let Some(book_list) = book_list {
                        result_list.add_all(book_list);
                    }
                }
                true
            });
            // logger.info("refreshed bookSourceList: {}", resultList)
            self.save_book_sources(book, result_list, user_name_space, true);
            return return_data.set_data(result_list);
        }
        return return_data.set_data(array_list!::<i32>());
    }

    async fn get_bookshelf(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let refresh: i32 = 0;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            refresh = context.body_as_json().get_integer("refresh", 0);
        } else {
            // get 请求
            refresh = context.query_param("refresh")?.to_int().unwrap_or(0);
        }
        let book_list = self.get_book_shelf_books(refresh > 0, self.base.get_user_name_space(context));
        return return_data.set_data(book_list);
    }

    async fn get_shelf_book(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            url = context.body_as_json().get_string("url");
        } else {
            // get 请求
            url = context.query_param("url").unwrap_or("".to_string());
        }
        if url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("书源链接不能为空");
        }

        let book = self.get_shelf_book_by_url(url, self.base.get_user_name_space(context));
        if book == None {
            return return_data.set_error_msg("书籍不存在");
        }
        return return_data.set_data(book);
    }

    async fn save_book(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let mut book = context.body_as_json().map_to::<Book>();
        let user_name_space = self.base.get_user_name_space(context);
        let mut book_source: Option<String> = None;
        if !book.is_local_book() {
            book_source = self.get_book_source_string_by_source_url_opt(book.origin, user_name_space)
                ?: return return_data.set_error_msg("书源信息错误");
            if book.toc_url.map_or(true, |s| s.is_empty()) {
                self.web_book(book_source, self.base.get_app_config_debug_log(), user_name_space).get_book_info(book);
            }
            book = self.merge_book_cache_info(book);
        }
        self.save_book_cover(book, user_name_space, book_source);
        self.save_local_book_cover(book, user_name_space);
        let result = self.save_book_to_shelf(book, user_name_space, context);
        if result.second != None {
            return return_data.set_error_msg(result.second.unwrap_or(""));
        }
        return return_data.set_data(result.first);
    }

    async fn set_book_source(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let new_book_url: String;
        let book_source_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("bookUrl");
            new_book_url = context.body_as_json().get_string("newUrl");
            book_source_url = context.body_as_json().get_string("bookSourceUrl");
        } else {
            // get 请求
            book_url = context.query_param("bookUrl").unwrap_or("".to_string());
            new_book_url = context.query_param("newUrl").unwrap_or("".to_string());
            book_source_url = context.query_param("bookSourceUrl").unwrap_or("".to_string());
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("书籍链接不能为空");
        }
        if new_book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("新源书籍链接不能为空");
        }
        if book_source_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("书源链接不能为空");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book = self.get_shelf_book_by_url(book_url, user_name_space);
        if book == None {
            return return_data.set_error_msg("书籍信息错误");
        }
        // 查找是否存在该书源
        let book_source_string = self.get_book_source_string_by_source_url_opt(book_source_url, user_name_space);

        let mut search_book: Option<Book> = None;
        if book_source_string.map_or(true, |s| s.is_empty()) {
            // 判断是不是本地书籍
            let local_book_source_list = as_json_array(self.base.get_user_storage(user_name_space, book.name + "_" + book.author, "bookSource"));

            // 遍历判断书本是否存在
            if local_book_source_list != None {
                for i in 0..local_book_source_list.size() {
                    let _search_book = local_book_source_list.get_json_object(i).map_to::<SearchBook>();
                    if _search_book.book_url.equals(new_book_url) {
                        search_book = _search_book.to_book();
                        break;
                    }
                }
            }
            if search_book == None {
                return return_data.set_error_msg("书源信息错误");
            }
        }

        let mut new_book_info = if search_book != None {
            search_book
        } else {
            if book_source_string.map_or(true, |s| s.is_empty()) {
                return return_data.set_error_msg("书源信息错误");
            }
            self.web_book(book_source_string, self.base.get_app_config_debug_log(), user_name_space).get_book_info(new_book_url, false)
        };

        self.edit_shelf_book(book, user_name_space, |exist_book| {
            exist_book.origin = new_book_info.origin;
            exist_book.origin_name = new_book_info.origin_name;
            exist_book.book_url = new_book_info.book_url;
            exist_book.toc_url = new_book_info.toc_url;
            exist_book.is_in_shelf = true;
            if exist_book.cover_url.map_or(true, |s| s.is_empty()) && !new_book_info.cover_url.map_or(true, |s| s.is_empty()) {
                exist_book.cover_url = new_book_info.cover_url;
            }

            LOGGER.info(format!("setBookSource: {}", exist_book));

            new_book_info = exist_book;

            exist_book
        });

        // 更新目录；JAR 保持目录刷新失败不影响书源切换结果
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.get_local_chapter_list(new_book_info, book_source_string.unwrap_or(""), true, user_name_space, false);
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
        }
        }
        return return_data.set_data(new_book_info);
    }
    async fn save_book_group_id(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let group_id: i64;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("bookUrl");
            group_id = context.body_as_json().get_long("groupId", 0);
        } else {
            // get 请求
            book_url = context.query_param("bookUrl").unwrap_or("".to_string());
            group_id = context.query_param("groupId")?.to_long_or_null().unwrap_or(0);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("书籍链接不能为空");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let mut book = self.get_shelf_book_by_url(book_url, user_name_space);
        if book == None {
            return return_data.set_error_msg("书籍信息错误");
        }

        if group_id <= 0 {
            return return_data.set_error_msg("分组信息错误");
        }

        self.edit_shelf_book(book, user_name_space, |exist_book| {
            exist_book.group = group_id;
            LOGGER.info(format!("saveBookGroupId: {}", exist_book));
            exist_book
        });

        book.group = group_id;
        return return_data.set_data(book);
    }

    async fn add_book_group_multi(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let group_id = context.body_as_json().get_long("groupId", 0);
        if group_id <= 0 {
            return return_data.set_error_msg("分组信息错误");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book_json_array = context.body_as_json().get_json_array("bookList", JsonArray::new());
        for k in 0..book_json_array.size() {
            let book = book_json_array.get_json_object(k).map_to::<Book>();
            self.edit_shelf_book(book, user_name_space, |exist_book| {
                exist_book.group = exist_book.group or group_id;
                LOGGER.info(format!("saveBookGroupId: {}", exist_book));
                exist_book
            });
        }

        return return_data.set_data("");
    }

    async fn remove_book_group_multi(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let group_id = context.body_as_json().get_long("groupId", 0);
        if group_id <= 0 {
            return return_data.set_error_msg("分组信息错误");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book_json_array = context.body_as_json().get_json_array("bookList", JsonArray::new());
        for k in 0..book_json_array.size() {
            let book = book_json_array.get_json_object(k).map_to::<Book>();
            self.edit_shelf_book(book, user_name_space, |exist_book| {
                exist_book.group = exist_book.group xor group_id;
                LOGGER.info(format!("saveBookGroupId: {}", exist_book));
                exist_book
            });
        }

        return return_data.set_data("");
    }

    async fn delete_book(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let mut book = context.body_as_json().map_to::<Book>();
        let user_name_space = self.base.get_user_name_space(context);
        let mut bookshelf: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "bookshelf"));
        if bookshelf == None {
            bookshelf = JsonArray::new();
        }
        // 遍历判断书本是否存在
        let mut exist_index: i32 = -1;
        for i in 0..bookshelf.size() {
            let _book = bookshelf.get_json_object(i).map_to::<Book>();
            if _book.book_url.equals(book.book_url) {
                exist_index = i;
                book = _book;
                break;
            }
            if _book.name.equals(book.name) && _book.author.equals(book.author) {
                exist_index = i;
                book = _book;
                break;
            }
        }
        if exist_index < 0 {
            return return_data.set_error_msg("书架书籍不存在");
        }
        bookshelf.remove(exist_index);
        // logger.info("bookshelf: {}", bookshelf)
        self.base.save_user_storage(user_name_space, "bookshelf", bookshelf);

        // 删除书籍目录
        let local_book_path = File::new(get_work_dir("storage", "data", user_name_space, book.name + "_" + book.author));
        local_book_path.delete_recursively();
        if book.cover_url?.starts_with("/") == true {
            FileUtils::delete_file(get_work_dir("storage" + book.cover_url));
        }

        return return_data.set_data("删除书籍成功");
    }

    async fn delete_books(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_json_array = context.body_as_json_array();

        let user_name_space = self.base.get_user_name_space(context);
        let mut bookshelf: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "bookshelf"));
        if bookshelf == None {
            bookshelf = JsonArray::new();
        }
        let requested_urls = hash_set!<String>();
        let requested_names = hash_set!<String>();
        for k in 0..book_json_array.size() {
            let requested = book_json_array.get_json_object(k);
            requested_urls.add(requested.get_string("bookUrl", ""));
            requested_names.add(requested.get_string("name", "") + "_" + requested.get_string("author", ""));
        }
        let iterator = bookshelf.iterator();
        while iterator.has_next() {
            let book = (iterator.next() as JsonObject).map_to::<Book>();
            if book.book_url !in requested_urls && (book.name + "_" + book.author) !in requested_names {
                continue;
            }
            iterator.remove();
            let local_book_path = File::new(get_work_dir("storage", "data", user_name_space, book.name + "_" + book.author));
            local_book_path.delete_recursively();
        }

        self.base.save_user_storage(user_name_space, "bookshelf", bookshelf);
        return return_data.set_data("");
    }

    async fn save_book_info_cache(&self, book_list: List<Book>) -> List<Book> {
        if book_list.size > 0 {
            for i in 0..book_list.size {
                let book = book_list.get(i);
                book_info_cache.put(book.book_url, json_encode(JsonObject::map_from(book).map));
            }
        }
        return book_list;
    }

    async fn merge_book_cache_info(&self, book: Book) -> Book {
        let cache_info: Option<Book> = book_info_cache.get_as_string(book.book_url)?.to_map()?.to_data_class();

        if cache_info != None {
            return book.fill_data(cache_info, list!("name", "author", "coverUrl", "tocUrl", "intro", "latestChapterTitle", "wordCount"));
        }
        return book;
    }

    async fn get_book_shelf_books(&self, refresh: bool = false, user_name_space: String) -> List<Book> {
        let bookshelf: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "bookshelf"));
        if bookshelf == None {
            return array_list!::<Book>();
        }
        if bookshelf.size() == 0 {
            return array_list!::<Book>();
        }
        let mut book_list = array_list!<Book>();
        let concurrent_count = 16;
        let mutex = Mutex::new();
        let sync_mutex = Mutex::new();
        self.base.limit_concurrent(concurrent_count, 0, bookshelf.size(), |it| {
            let mut book = bookshelf.get_json_object(it).map_to::<Book>();
            book.is_in_shelf = true;
            if !book.is_local_book() && book.can_update && refresh {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let book_source = self.get_book_source_string_by_source_url_opt(book.origin, user_name_space);
                    if book_source != None {
                        with_context(Dispatchers::IO, || {
                            let book_chapter_list = self.get_local_chapter_list(book, book_source, refresh, user_name_space, false, mutex);
                            if book_chapter_list.size > 0 {
                                let book_chapter = book_chapter_list.last();
                                book.latest_chapter_title = book_chapter.title;
                            }
                            if book_chapter_list.size - book.total_chapter_num > 0 {
                                book.last_check_time = System::current_time_millis();
                                book.last_check_count = book_chapter_list.size - book.total_chapter_num;
                            }
                            book.total_chapter_num = book_chapter_list.size;
                        });
                    }
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                    e.print_stack_trace();
                }
        }
            }
            sync_mutex.lock();
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                book_list.add(book);
            }));
            sync_mutex.unlock();
        });
        return book_list;
    }

    async fn get_local_chapter_list(&self, book: Book, book_source: Option<String>, refresh: bool = false, user_name_space: String, debug_log: bool = true, mutex: Option<Mutex> = None) -> List<BookChapter> {
        let md5_encode = MD5Utils::md5_encode(book.book_url).to_string();
        let book_chapters_cache = self.get_book_chapters_cache(user_name_space);
        let cache_key = book.name + "_" + book.author + md5_encode;
        let chapter_list = if book.is_in_shelf {
            as_json_array(self.base.get_user_storage(user_name_space, book.name + "_" + book.author, md5_encode))
        } else {
            as_json_array(book_chapters_cache.get_as_string(cache_key))
        };

        if chapter_list == None || refresh {
            let mut new_chapter_list: List<BookChapter>;
            book.set_root_dir(get_work_dir());
            book.set_user_name_space(user_name_space);
            if book.is_local_book() {
                // 重新解压epub文件
                if book.is_epub() && !self.extract_epub(book, refresh) {
                    throw Exception::new("Epub书籍解压失败");
                }
                // 重新解压cbz文件
                if book.is_cbz() && !self.extract_cbz(book, refresh) {
                    throw Exception::new("CBZ书籍解压失败");
                }
                if book.is_pdf() && !self.convert_pdf_to_image(book, refresh) {
                    throw Exception::new("PDF书籍转换失败");
                }
                new_chapter_list = LocalBook::get_chapter_list(book);
            } else {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let book_source_object = book_source.and_then(|it| BookSource::from_json(it).get_or_null());
                    if let Some(book_source_object) = book_source_object {
                        if let Some(pre_update_js) = book_source_object.rule_toc.pre_update_js {
                            AnalyzeRule::new(book, book_source_object).eval_js(pre_update_js);
                        }
                    }
                    let source = require_not_null(book_source);
                    if book.toc_url.is_null_or_blank() {
                        self.web_book(source, debug_log, user_name_space).get_book_info(book, false);
                    }
                    new_chapter_list = self.web_book(source, debug_log, user_name_space).get_chapter_list(book);
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                    if !book_source.map_or(true, |s| s.is_empty()) {
                        let book_source_object = BookSource::from_json(book_source).get_or_null();
                        if book_source_object != None {
                            // 标记为失败源
                            let info = mutable_map_of!("sourceUrl" to book_source_object.book_source_url, "time" to System::current_time_millis(), "error" to e.to_string());
                            self.add_invalid_book_source(book_source_object.book_source_url, info, user_name_space);
                        }
                    }
                    if mutex != None { mutex.lock() }
                    {
                        book.last_check_error = e.to_string();
                        self.edit_shelf_book(book, user_name_space, |exist_book| {
                            exist_book.last_check_error = e.to_string();
                            exist_book
                        });
                    }
                    panic!("{}", e);
            }
        }
            if book.is_in_shelf {
                self.base.save_user_storage(user_name_space, get_relative_path(book.name + "_" + book.author, md5_encode), new_chapter_list);
            } else {
                book_chapters_cache.put(cache_key, json_encode(new_chapter_list), 3600);
            }
            self.save_shelf_book_latest_chapter(book, new_chapter_list, user_name_space, mutex);
            return new_chapter_list;
        }
        let mut local_chapter_list = array_list!<BookChapter>();
        for i in 0..chapter_list.size() {
            let _chapter = chapter_list.get_json_object(i).map_to::<BookChapter>();
            local_chapter_list.add(_chapter);
        }
        return local_chapter_list;
    }
    }
    async fn get_book_source_string(
        &self,
        context: RoutingContext,
        source_url: String = "",
        with_explore_url: bool = false
    ) -> Option<String> {
        let mut book_source_string: Option<String> = None;
        if context.request().method() == HttpMethod::POST {
            let book_source = context.body_as_json().get_json_object("bookSource");
            if book_source != None {
                book_source_string = book_source.to_string();
            }
        }
        let user_name_space = self.base.get_user_name_space(context);
        if book_source_string.map_or(true, |s| s.is_empty()) {
            let book_source_url: String;
            if context.request().method() == HttpMethod::POST {
                book_source_url = context.body_as_json().get_string("bookSourceUrl", "");
            } else {
                book_source_url = context.query_param("bookSourceUrl").unwrap_or("".to_string());
            }
            if book_source_url.is_not_blank() {
                book_source_string = self.get_book_source_string_by_source_url_opt(book_source_url, user_name_space);
            }
        }
        if book_source_string.map_or(true, |s| s.is_empty()) && source_url.is_not_blank() {
            book_source_string = self.get_book_source_string_by_source_url_opt(source_url, user_name_space);
        }
        return book_source_string;
    }

    fn get_shelf_book_by_url(&self, url: String, user_name_space: String) -> Option<Book> {
        if url.is_empty() {
            return None;
        }
        let bookshelf: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "bookshelf"));
        if bookshelf == None {
            return None;
        }
        for i in 0..bookshelf.size() {
            let _book = bookshelf.get_json_object(i).map_to::<Book>();
            if _book.book_url.equals(url) {
                _book.set_root_dir(get_work_dir());
                _book.set_user_name_space(user_name_space);
                _book.is_in_shelf = true;
                return _book;
            }
        }
        return None;
    }

    async fn save_shelf_book_progress(&self, book: Book, book_chapter: BookChapter, user_name_space: String) {
        self.edit_shelf_book(book, user_name_space, |exist_book| {
            exist_book.dur_chapter_index = book_chapter.index;
            exist_book.dur_chapter_title = book_chapter.title;
            exist_book.dur_chapter_time = System::current_time_millis();

            // logger.info("saveShelfBookProgress: {}", existBook)

            exist_book
        });
    }

    async fn save_shelf_book_latest_chapter(&self, book: Book, book_chapter_list: List<BookChapter>, user_name_space: String, mutex: Option<Mutex> = None) {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Some(mutex) = mutex { mutex.lock() }
            self.edit_shelf_book(book, user_name_space, |exist_book| {
                if book_chapter_list.size > 0 {
                    let book_chapter = book_chapter_list.last();
                    exist_book.latest_chapter_title = book_chapter.title;
                }
                if book_chapter_list.size - exist_book.total_chapter_num > 0 {
                    exist_book.last_check_count = book_chapter_list.size - exist_book.total_chapter_num;
                    exist_book.last_check_time = System::current_time_millis();
                }
                exist_book.last_check_error = None;
                exist_book.total_chapter_num = book_chapter_list.size;
                book.latest_chapter_title = exist_book.latest_chapter_title;
                book.last_check_count = exist_book.last_check_count;
                book.last_check_time = exist_book.last_check_time;
                book.last_check_error = exist_book.last_check_error;
                book.total_chapter_num = exist_book.total_chapter_num;
                exist_book
            });
        }));
            if let Some(mutex) = mutex { mutex.unlock() }
    }

    async fn edit_shelf_book(&self, book: Book, user_name_space: String, handler: (Book)->Book) -> Option<Book> {
        let mutex = UserMutex::get_locker(user_name_space + "@bookshelf");
        LOGGER.info(format!("wait for lock {}", user_name_space + "@bookshelf"));
        mutex.lock();
        {
            LOGGER.info(format!("lock success"));
            let mut bookshelf: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "bookshelf"));
            if bookshelf == None {
                bookshelf = JsonArray::new();
            }
            let mut exist_index: i32 = -1;
            for i in 0..bookshelf.size() {
                let _book = bookshelf.get_json_object(i).map_to::<Book>();
                if book.book_url.is_not_empty() && _book.book_url.equals(book.book_url) {
                    exist_index = i;
                    break;
                }
                if book.name.is_not_empty() && _book.name.equals(book.name) && book.author.is_not_empty() && _book.author.equals(book.author) {
                    exist_index = i;
                    break;
                }
            }
            if exist_index >= 0 {
                let mut book_list = bookshelf.get_list();
                let mut exist_book = bookshelf.get_json_object(exist_index).map_to::<Book>();
                exist_book = handler(exist_book);

                book_list.set(exist_index, JsonObject::map_from(exist_book));
                bookshelf = JsonArray::new(book_list);
                self.base.save_user_storage(user_name_space, "bookshelf", bookshelf);
                return exist_book;
            }
            return None;
            mutex.unlock();
        }
    }

    fn save_book_sources(&self, book: Book, source_list: List<SearchBook>, user_name_space: String, replace: bool = false) {
        if book.name.is_empty() {
            return;
        }
        let mut book_source_list = JsonArray::new();
        if !replace {
            let local_book_source_list = as_json_array(self.base.get_user_storage(user_name_space, book.name + "_" + book.author, "bookSource"));
            book_source_list = local_book_source_list;
        }

        for k in 0..source_list.size {
            let search_book = source_list.get(k);
            // 遍历判断书源是否存在（同一书源只保留一条，避免同名同作者多版本重复）
            let mut exist_index: i32 = -1;
            for i in 0..book_source_list.size() {
                let _search_book = book_source_list.get_json_object(i).map_to::<SearchBook>();
                if _search_book.origin.equals(search_book.origin) {
                    exist_index = i;
                    break;
                }
            }
            if exist_index >= 0 {
                let mut _source_list = book_source_list.get_list();
                _source_list.set(exist_index, JsonObject::map_from(search_book));
                book_source_list = JsonArray::new(_source_list);
            } else {
                book_source_list.add(JsonObject::map_from(search_book));
            }
        }

        // logger.info("bookSourceList: {}", bookSourceList)
        self.base.save_user_storage(user_name_space, get_relative_path(book.name + "_" + book.author, "bookSource"), book_source_list);
    }

    fn extract_epub(&self, book: Book, force: bool = false) -> bool {
        let epub_extract_dir = File::new(get_work_dir(book.book_url + File::separator + "index"));
        if force || !epub_extract_dir.exists() {
            epub_extract_dir.delete_recursively();
            let mut local_epub_file = File::new(get_work_dir(book.origin_name + File::separator + "index.epub"));
            if book.origin_name.index_of("localStore") > 0 {
                // 本地书仓的源文件
                local_epub_file = File::new(get_work_dir(book.origin_name));
            }
            if book.origin_name.index_of("webdav") > 0 {
                // webdav 书仓的源文件
                local_epub_file = File::new(get_work_dir(book.origin_name));
            }
            if !local_epub_file.unzip(epub_extract_dir.to_string()) {
                return false;
            }
        }
        return true;
    }

    fn extract_cbz(&self, book: Book, force: bool = false) -> bool {
        let extract_dir = File::new(get_work_dir(book.book_url + File::separator + "index"));
        if force || !extract_dir.exists() {
            extract_dir.delete_recursively();
            let mut local_file = File::new(get_work_dir(book.origin_name + File::separator + "index.cbz"));
            if book.origin_name.index_of("localStore") > 0 {
                // 本地书仓的源文件
                local_file = File::new(get_work_dir(book.origin_name));
            }
            if book.origin_name.index_of("webdav") > 0 {
                // webdav 书仓的源文件
                local_file = File::new(get_work_dir(book.origin_name));
            }
            if !local_file.unzip(extract_dir.to_string()) {
                return false;
            }
        }
        return true;
    }

    async fn sync_book_progress_from_webdav(&self, progress_file_path: Any, user_name_space: String) {
        let mut progress_file: Option<File> = None;
        match progress_file_path {
            File => progress_file = progress_file_path,
            String => progress_file = File::new(progress_file_path),
        }
        if progress_file == None {
            return;
        }
        let book = as_json_object(progress_file.read_text())?.map_to::<Book>();
        if book != None {
            self.edit_shelf_book(book, user_name_space, |exist_book| {
                exist_book.dur_chapter_index = book.dur_chapter_index;
                exist_book.dur_chapter_pos = book.dur_chapter_pos;
                exist_book.dur_chapter_time = book.dur_chapter_time;
                exist_book.dur_chapter_title = book.dur_chapter_title;

                LOGGER.info(format!("syncShelfBookProgress: {}", exist_book));
                exist_book
            });
        }
    }

    async fn save_book_progress_to_webdav(&self, book: Book, book_chapter: BookChapter, user_name_space: String) {
        let user_home = self.base.get_user_webdav_home(user_name_space);
        let mut book_progress_dir = File::new(user_home + File::separator + "bookProgress");
        if !book_progress_dir.exists() {
            book_progress_dir = File::new(user_home + File::separator + "legado" + File::separator + "bookProgress");
            if !book_progress_dir.exists() {
                return;
            }
        }
        let progress_file = File::new(book_progress_dir.to_string() + File::separator + book.name + "_" + book.author + ".json");
        progress_file.write_text(json_encode(map!(
            "name" to book.name,
            "author" to book.author,
            "durChapterIndex" to book_chapter.index,
            "durChapterPos" to 0,
            "durChapterTime" to System::current_time_millis(),
            "durChapterTitle" to book_chapter.title
        ), true));
    }

    async fn sync_from_webdav(&self, zip_file_path: String, user_name_space: String) -> bool {
        let desc_dir = get_work_dir("storage", "data", user_name_space, "tmp");
        let desc_dir_file = File::new(desc_dir);
        {
            let user_home = self.base.get_user_webdav_home(user_name_space);
            let zip_file = File::new(zip_file_path);
            if !zip_file.exists() {
                return false;
            }
            desc_dir_file.delete_recursively();
            io.legado.app.utils.ZipUtils.unzip_file(zip_file, desc_dir_file);
            for file_name in self.backup_file_names {
                let backup_file = File::new(desc_dir + File::separator + file_name);
                if !backup_file.exists() { continue }
                let user_data_file = File::new(get_work_dir("storage", "data", user_name_space, file_name));
                user_data_file.delete_recursively();
                backup_file.copy_recursively(user_data_file);
            }
            let backup_books_dir = File::new(desc_dir + File::separator + "books");
            if backup_books_dir.exists() {
                let webdav_books_dir = File::new(get_work_dir("storage", "data", user_name_space, "webdav", "books"));
                webdav_books_dir.delete_recursively();
                backup_books_dir.copy_recursively(webdav_books_dir);
            }
            // 同步阅读进度
            let mut book_progress_dir = File::new(user_home + File::separator + "bookProgress");
            if !book_progress_dir.exists() {
                book_progress_dir = File::new(user_home + File::separator + "legado" + File::separator + "bookProgress");
            }
            if book_progress_dir.exists() && book_progress_dir.is_directory() {
                if let Some(list_files) = book_progress_dir.list_files() {
                    for it in list_files {
                        self.sync_book_progress_from_webdav(it, user_name_space);
                    }
                }
            }
            return true;
        }
            desc_dir_file.delete_recursively();
        return false;
    }

    async fn save_to_webdav(&self, user_name_space: String, latest_zip_file_path: Option<String> = None) -> bool {
        let user_home = self.base.get_user_webdav_home(user_name_space);
        let mut legado_home = user_home;
        let resolved_zip_file_path = latest_zip_file_path.unwrap_or(self.get_last_back_file_from_webdav(user_name_space));
        if resolved_zip_file_path == None {
            legado_home = user_home + File::separator + "legado";
        } else if resolved_zip_file_path.index_of("legado") > 0 {
            legado_home = user_home + File::separator + "legado";
        }
        return self.create_user_backup(user_name_space, legado_home, resolved_zip_file_path) != None;
    }

    async fn get_last_back_file_from_webdav(&self, user_name_space: String) -> Option<String> {
        let user_home = self.base.get_user_webdav_home(user_name_space);
        let mut legado_home = File::new(user_home + File::separator + "legado");
        if !legado_home.exists() {
            legado_home = File::new(user_home);
        }
        if !legado_home.exists() {
            return None;
        }
        let mut latest_zip_file: Option<String> = None;
        let zip_file_reg = Regex::new("^backup[0-9-]+.zip$", RegexOption::IGNORE_CASE);    //忽略大小写
        let mut files = legado_home.list_files().unwrap();
        files.sort_by_descending {
            it.last_modified()
        };
        for it in files {
            if zip_file_reg.matches(it.name) {
                latest_zip_file = it.to_string();
                break;
            }
        }
        return latest_zip_file;
    }

    async fn book_source_debug_sse(&self, context: RoutingContext) {
        let return_data = ReturnData::new();
        // 返回 event-stream
        let response = context.response().put_header("Content-Type", "text/event-stream")
            .put_header("Cache-Control", "no-cache")
            .set_chunked(true);

        if !self.base.check_auth(context) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用"), false) + "\n\n");
            return;
        }
        let book_source_url = context.query_param("bookSourceUrl").unwrap_or("".to_string());
        let keyword = context.query_param("keyword").unwrap_or("".to_string());

        if book_source_url.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("未配置书源"), false) + "\n\n");
            return;
        }
        if keyword.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("请输入搜索关键词"), false) + "\n\n");
            return;
        }

        let user_name_space = self.base.get_user_name_space(context);
        let book_source_string = self.get_book_source_string_by_source_url_opt(book_source_url, user_name_space);
        if book_source_string.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("未配置书源"), false) + "\n\n");
            return;
        }

        LOGGER.info(format!("bookSourceDebugSSE bookSource: {} keyword: {}", book_source_string, keyword));

        let debugger = Debugger::new(|msg| {
            response.write("data: " + json_encode(map!("msg" to msg), false) + "\n\n");
        });

        let web_book = self.web_book(book_source_string, false, user_name_space);

        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 bookSourceDebugSSE"));
            self.coroutine_context.cancel();
        }

        debugger.start_debug(web_book, keyword);

        response.write("event: end\n");
        response.end("data: " + json_encode(map!("end" to true), false) + "\n\n");
    }
    async fn cache_book_sse(&self, context: RoutingContext) {
        let return_data = ReturnData::new();
        // 返回 event-stream
        let response = context.response().put_header("Content-Type", "text/event-stream")
            .put_header("Cache-Control", "no-cache")
            .set_chunked(true);

        if !self.base.check_auth(context) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用"), false) + "\n\n");
            return;
        }
        let book_url: String;
        let refresh: i32;
        let mut concurrent_count: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_string("bookUrl").unwrap_or_default();
            refresh = context.body_as_json().get_integer("refresh", 0);
            concurrent_count = context.body_as_json().get_integer("concurrentCount", 24);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            refresh = context.query_param("refresh")?.to_int().unwrap_or(0);
            concurrent_count = context.query_param("concurrentCount")?.to_int().unwrap_or(24);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("请输入书籍链接"), false) + "\n\n");
            return;
        }

        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info == None {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("请先加入书架"), false) + "\n\n");
            return;
        }
        if book_info.is_local_book() {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("本地书籍无需缓存"), false) + "\n\n");
            return;
        }
        let book_source = self.get_book_source_string(context, book_info.origin);
        if book_source.map_or(true, |s| s.is_empty()) {
            response.write("event: error\n");
            response.end("data: " + json_encode(return_data.set_error_msg("未配置书源"), false) + "\n\n");
            return;
        }

        let chapter_list = self.get_local_chapter_list(book_info, book_source, false, user_name_space, false);
        let mut cached_chapter_content_set = mutable_set_of!<i32>();
        if refresh <= 0 {
            cached_chapter_content_set = self.get_cached_chapter_content_set(book_info, user_name_space);
        }
        let local_cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
        let mut is_end = false;
        let mut success_count = 0;
        let mut failed_count = 0;

        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 cacheBookSSE"));
            is_end = true;
            self.coroutine_context.cancel();
        }

        concurrent_count = if concurrent_count > 0 { concurrent_count } else { 24 };
        LOGGER.info(format!("cacheBookSSE concurrentCount: {} refresh: {}", concurrent_count, refresh));
        self.base.limit_concurrent(concurrent_count, 0, chapter_list.size, |it| {
            if !cached_chapter_content_set.contains(it) {
                let chapter_index = it;
                let chapter_info = chapter_list.get(it);
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut next_chapter_url: Option<String> = None;
                    if chapter_index + 1 < chapter_list.size {
                        let next_chapter_info = chapter_list.get(chapter_index + 1);
                        next_chapter_url = next_chapter_info.url;
                    }
                    let content = self.web_book(book_source, self.base.get_app_config_debug_log(), user_name_space).get_book_content(book_info, chapter_info, next_chapter_url);
                    let chapter_cache_file = File::new(local_cache_dir.absolute_path + File::separator + chapter_index + ".txt");
                    chapter_cache_file.write_text(content);
                    // 保存图片
                    BookHelp::save_images(
                        self,
                        BookSource::from_json(book_source).get_or_null().unwrap_or(BookSource)::new(),
                        book_info,
                        chapter_info,
                        content
                    );
                    success_count++;
                    cached_chapter_content_set.add(chapter_index);
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                    is_end = true;
                    failed_count++;
                }
        }
            }
            it
        }, |list, loop_count| {
            if is_end {
                false
            } else {
                // 返回本轮数据
                let result = map!(
                    "cachedCount" to cached_chapter_content_set.size,
                    "successCount" to success_count,
                    "failedCount" to failed_count
                );
                response.write("data: " + json_encode(result, false) + "\n\n");
                LOGGER.info(format!("Loog: {} list.size: {} result: {}", loop_count, list.size, result));
                true
            }
        });
        response.write("event: end\n");
        response.end("data: " + json_encode(map!(
            "cachedCount" to cached_chapter_content_set.size,
            "successCount" to success_count,
            "failedCount" to failed_count
        ), false) + "\n\n");
    }

    async fn delete_book_cache(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_string("bookUrl").unwrap_or_default();
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }

        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info == None {
            return return_data.set_error_msg("请先加入书架");
        }
        if book_info.is_local_book() {
            return return_data.set_error_msg("本地书籍无需删除缓存");
        }
        let local_cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
        local_cache_dir.delete_recursively();

        return return_data.set_data("");
    }

    fn get_chapter_cache_dir(&self, book_info: Book, user_name_space: String) -> File {
        let md5_encode = MD5Utils::md5_encode(book_info.book_url).to_string();
        let local_cache_dir_path = get_work_dir("storage", "data", user_name_space, book_info.name + "_" + book_info.author, md5_encode);
        let local_cache_dir = File::new(local_cache_dir_path);
        if !local_cache_dir.exists() {
            local_cache_dir.mkdirs();
        }
        return local_cache_dir;
    }

    fn get_cached_chapter_content_set(&self, book_info: Book, user_name_space: String) -> MutableSet<i32> {
        let local_cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
        let cached_chapter_content_set = mutable_set_of!<i32>();
        for it in local_cache_dir.list_files().unwrap() {
            if !it.name.starts_with(".") && it.name.ends_with(".txt") {
                cached_chapter_content_set.add(it.name.replace(".txt", "").to_int());
            }
        }
        return cached_chapter_content_set;
    }

    async fn get_shelf_book_with_cache_info(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book_list = self.get_book_shelf_books(false, user_name_space);
        let mut result = mutable_list_of::<Any>();
        for i in 0..book_list.size {
            let book_info = book_list.get(i);
            if !book_info.is_local_book() {
                let cached_set = self.get_cached_chapter_content_set(book_info, user_name_space);
                let mut book_info_map = book_info.to_map() as MutableMap<String, Any>;
                book_info_map.put("cachedChapterCount", cached_set.size);
                result.add(book_info_map);
            } else {
                result.add(book_info);
            }
        }
        return return_data.set_data(result);
    }

    async fn export_book(&self, context: RoutingContext) {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            context.success(return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用"));
            return;
        }
        let book_url: String;
        let is_epub: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_string("bookUrl").unwrap_or_default();
            is_epub = context.body_as_json().get_integer("isEpub", 0);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            is_epub = context.query_param("isEpub")?.to_int().unwrap_or(0);
        }

        if book_url.map_or(true, |s| s.is_empty()) {
            context.success(return_data.set_error_msg("请输入书籍链接"));
            return;
        }

        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info == None {
            context.success(return_data.set_error_msg("请先加入书架"));
            return;
        }

        if book_info.is_local_book() && !book_info.is_local_txt() {
            let local_file = book_info.get_local_file();
            context.response().put_header("Cache-Control", "300")
                            .put_header("Content-Disposition", "attachment; filename=" + URLEncoder::encode(local_file.name, "UTF-8"))
                            .send_file(local_file.to_string());
            return;
        }
        if book_info.is_local_txt() && is_epub <= 0 {
            let local_file = book_info.get_local_file();
            context.response().put_header("Cache-Control", "300")
                            .put_header("Content-Disposition", "attachment; filename=" + URLEncoder::encode(local_file.name, "UTF-8"))
                            .send_file(local_file.to_string());
            return;
        }
        let book_source = self.get_book_source_string(context, book_info.origin);
        if !book_info.is_local_book() && book_source.map_or(true, |s| s.is_empty()) {
            context.success(return_data.set_error_msg("未配置书源"));
            return;
        }
        let export_dir = File::new(get_work_dir("storage", "assets", user_name_space, "export"));

        let book_file = if is_epub > 0 {
            self.export_to_epub(export_dir, book_info, book_source.unwrap_or(""), user_name_space)
        } else {
            self.export_to_txt(export_dir, book_info, book_source.unwrap_or(""), user_name_space)
        };
        context.response().put_header("Cache-Control", "300")
                        .put_header("Content-Disposition", "attachment; filename=" + URLEncoder::encode(book_file.name, "UTF-8"))
                        .send_file(book_file.to_string());
    }

    async fn export_to_txt(&self, export_dir: File, book_info: Book, book_source: String, user_name_space: String) -> File {
        let filename = "《${book_info.name}》作者：${book_info.get_real_author()}.txt";
        let book_path = FileUtils::get_path(export_dir, filename);
        let book_file = FileUtils::create_file_with_replace(book_path);
        // val stringBuilder = StringBuilder()
        self.get_all_contents(book_info, book_source, user_name_space, |text, src_list| {
            book_file.append_text(text, Charset::for_name(app_config.export_charset));
            // stringBuilder.append(text)
            // srcList?.forEach {
            //     val vFile = BookHelp.getImage(bookInfo, it.third)
            //     if (vFile.exists()) {
            //         FileUtils.createFileIfNotExist(
            //             exportDir,
            //             "${book.name}_${book.author}",
            //             "images",
            //             it.first,
            //             "${it.second}-${MD5Utils.md5Encode16(it.third)}.jpg"
            //         ).writeBytes(vFile.readBytes())
            //     }
            // }
        });
        return book_file;
    }

    async fn get_all_contents(
        &self,
        book: Book,
        book_source_string: String,
        user_name_space: String,
        append: (text: String, src_list: Option<ArrayList<Triple<String, i32, String>>>) -> Unit
    ) {
        // val useReplace = appConfig.exportUseReplace && book.getUseReplaceRule()
        // val contentProcessor = ContentProcessor.get(book.name, book.origin)
        let qy = "${book.name}\n作者：${book.get_real_author()}\n简介：${HtmlFormatter::format(book.get_display_intro())}";

        append(qy, None);
        let chapter_list = self.get_local_chapter_list(book, book_source_string, false, user_name_space, false);
        let local_cache_dir = self.get_chapter_cache_dir(book, user_name_space);

        for (index, chapter) in chapter_list.enumerate() {
            let mut chapter_cache_file = File::new(local_cache_dir.absolute_path + File::separator + index + ".txt");
            let mut content = "";
            if !app_config.export_no_chapter_name {
                content += chapter.title + "\n";
            }
            if chapter_cache_file.exists() {
                content += chapter_cache_file.read_text() + "\n";
            } else {
                content += "暂无缓存内容。\n";
            }

            append.invoke("\n\n" + content, None);

            // BookHelp.getContent(book, chapter).let |content| {
            //     val content1 = contentProcessor
            //         .getContent(
            //             book,
            //             chapter,
            //             content.unwrap_or("null"),
            //             includeTitle = !appConfig.exportNoChapterName,
            //             useReplace = useReplace,
            //             chineseConvert = false,
            //             reSegment = false
            //         ).joinToString("\n")
            //     if (appConfig.exportPictureFile) {
            //         //txt导出图片文件
            //         val srcList = arrayListOf<Triple<String, Int, String>>()
            //         content?.split("\n")?.forEachIndexed { index, text ->
            //             val matcher = AppPattern.imgPattern.matcher(text)
            //                 matcher.group(1)?.let {
            //                     val src = NetworkUtils.getAbsoluteURL(chapter.url, it)
            //                     srcList.add(Triple(chapter.title, index, src))
            //                 }
            //             }
            //         }
            //         append.invoke("\n\n$content1", srcList)
            //     } else {
            //         append.invoke("\n\n$content1", None)
            //     }
            // }
        }
    }
    async fn export_to_epub(&self, export_dir: File, book: Book, book_source: String, user_name_space: String) -> File {
        let filename = "《${book.name}》作者：${book.get_real_author()}.epub";
        let book_path = FileUtils::get_path(export_dir, filename);
        let book_file = FileUtils::create_file_with_replace(book_path);

        let epub_book = EpubBook::new();
        epub_book.version = "2.0";
        //set metadata
        self.set_epub_metadata(book, epub_book);
        //set cover
        self.set_cover(book, epub_book, book_source);
        //set css
        let content_model = self.set_assets(book, epub_book);

        //设置正文
        self.set_epub_content(content_model, book, epub_book, book_source, user_name_space);
        EpubWriter::new().write(epub_book, FileOutputStream::new(book_file));

        return book_file;
    }

    fn set_assets(&self, book: Book, epub_book: EpubBook) -> String {
        epub_book.resources.add(
            Resource::new(
                BookController::class.get_resource("/epub/fonts.css").read_bytes(),
                "Styles/fonts.css"
            )
        );
        epub_book.resources.add(
            Resource::new(
                BookController::class.get_resource("/epub/main.css").read_bytes(),
                "Styles/main.css"
            )
        );
        epub_book.resources.add(
            Resource::new(
                BookController::class.get_resource("/epub/logo.png").read_bytes(),
                "Images/logo.png"
            )
        );
        epub_book.add_section(
            "封面",
            ResourceUtil::create_public_resource(
                book.name,
                book.get_real_author(),
                book.get_display_intro(),
                book.kind,
                book.word_count,
                String::new(BookController::class.get_resource("/epub/cover.html").read_bytes()),
                "Text/cover.html"
            )
        );
        epub_book.add_section(
            "简介",
            ResourceUtil::create_public_resource(
                book.name,
                book.get_real_author(),
                book.get_display_intro(),
                book.kind,
                book.word_count,
                String::new(BookController::class.get_resource("/epub/intro.html").read_bytes()),
                "Text/intro.html"
            )
        );

        return String::new(BookController::class.get_resource("/epub/chapter.html").read_bytes());
    }

    async fn set_cover(&self, book: Book, epub_book: EpubBook, book_source_string: String) {
        let cover_url = book.get_display_cover();
        if cover_url == None {
            // TODO 默认封面

        } else if cover_url.starts_with("/") {
            // 本地 /assets 封面
            let cover_path = cover_url.replace("/", File::separator).substring(1);
            let cover_file = File::new(get_work_dir("storage", cover_path));
            let byte_array: ByteArray = cover_file.read_bytes();
            epub_book.cover_image = Resource::new(byte_array, "Images/cover.jpg");
        } else if !book_source_string.map_or(true, |s| s.is_empty()) {
            let ext = self.base.get_file_ext(cover_url, "jpg");
            let md5_encode = MD5Utils::md5_encode(cover_url).to_string();
            let cache_path = get_work_dir("storage", "cache", md5_encode + "." + ext);
            let cache_file = File::new(cache_path);
            if cache_file.exists() {
                let byte_array: ByteArray = cache_file.read_bytes();
                epub_book.cover_image = Resource::new(byte_array, "Images/cover.jpg");
                return;
            }
            let analyze_url = AnalyzeUrl::new(cover_url, source = BookSource::from_json(book_source_string).get_or_null());
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                analyze_url.get_byte_array_await().let {
                    epub_book.cover_image = Resource::new(it, "Images/cover.jpg");
                }
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                e.print_stack_trace();
            } finally {

            }
        }
            // webClient.getAbs(coverUrl).timeout(3000).send
            // webClient.getAbs(coverUrl).timeout(3000).send {
            //     var bodyBytes = it.result()?.bodyAsBuffer()?.getBytes()
            //     if (bodyBytes != None) {
            //         epubBook.coverImage = Resource(bodyBytes, "Images/cover.jpg")
            //     }
            // }
        }
    }

    async fn set_epub_content(
        &self,
        content_model: String,
        book: Book,
        epub_book: EpubBook,
        book_source_string: String,
        user_name_space: String
    ) {
        //正文
        let chapter_list = self.get_local_chapter_list(book, book_source_string, false, user_name_space, false);
        let local_cache_dir = self.get_chapter_cache_dir(book, user_name_space);

        for (index, chapter) in chapter_list.enumerate() {
            let mut chapter_cache_file = File::new(local_cache_dir.absolute_path + File::separator + index + ".txt");
            let mut content = "";
            if !app_config.export_no_chapter_name {
                content += chapter.title + "\n";
            }
            if book.is_local_txt() {
                content += LocalBook::get_content(book, chapter).unwrap_or("");
            } else if chapter_cache_file.exists() {
                content += chapter_cache_file.read_text() + "\n";
            } else {
                content += "暂无缓存内容。\n";
            }

            let mut content1 = self.fix_pic(epub_book, book, content, chapter);
            // content1 = contentProcessor
            //     .getContent(
            //         book,
            //         chapter,
            //         content1,
            //         includeTitle = false,
            //         useReplace = useReplace,
            //         chineseConvert = false,
            //         reSegment = false
            //     )
            //     .joinToString("\n")
            let title = chapter.title;
            epub_book.add_section(
                title,
                ResourceUtil::create_chapter_resource(
                    title.replace("\u{1F512}", ""),
                    content1,
                    content_model,
                    "Text/chapter_${index}.html"
                )
            );
        }
    }

    fn fix_pic(
        &self,
        epub_book: EpubBook,
        book: Book,
        content: String,
        chapter: BookChapter
    ) -> String {
        let data = StringBuilder::new("");
        for text in content.split("\n") {
            let mut text1 = text;
            let matcher = AppPattern::imgPattern.matcher(text);
            while matcher.find() {
                if let Some(it) = matcher.group(1) {
                    let src = NetworkUtils::get_absolute_url(chapter.url, it);
                    let original_href = "${MD5Utils::md5_encode16(src)}.${BookHelp::get_image_suffix(src)}";
                    let href = "Images/${MD5Utils::md5_encode16(src)}.${BookHelp::get_image_suffix(src)}";
                    let v_file = BookHelp::get_image(book, src);
                    let fp = FileResourceProvider::new(v_file.parent);
                    if v_file.exists() {
                        let img = LazyResource::new(fp, href, original_href);
                        epub_book.resources.add(img);
                    }
                    text1 = text1.replace(it, "../" + href);
                }
            }
            data.append(text1).append("\n");
        }
        return data.to_string();
    }

    fn set_epub_metadata(&self, book: Book, epub_book: EpubBook) {
        let metadata = Metadata::new();
        metadata.titles.add(book.name);//书籍的名称
        metadata.authors.add(Author::new(book.get_real_author()));//书籍的作者
        metadata.language = "zh";//数据的语言
        metadata.dates.add(Date::new());//数据的创建日期
        metadata.publishers.add("Legado");//数据的创建者
        metadata.descriptions.add(book.get_display_intro());//书籍的简介
        //metadata.subjects.add("")//书籍的主题，在静读天下里面有使用这个分类书籍
        epub_book.metadata = metadata;
    }

    async fn search_book_content(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();

        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url: String;
        let keyword: String;
        let mut last_index: i32;
        let size: i32;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_url = context.body_as_json().get_string("url").unwrap_or(context.body_as_json)().get_string("bookUrl").unwrap_or_default();
            keyword = context.body_as_json().get_string("keyword").unwrap_or_default();
            last_index = context.body_as_json().get_integer("lastIndex", 0);
            size = context.body_as_json().get_integer("size", 20);
        } else {
            // get 请求
            book_url = context.query_param("url").unwrap_or("".to_string());
            keyword = context.query_param("keyword").unwrap_or("".to_string());
            last_index = context.query_param("lastIndex")?.to_int().unwrap_or(0);
            size = context.query_param("size")?.to_int().unwrap_or(20);
        }
        if book_url.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入书籍链接");
        }
        if keyword.map_or(true, |s| s.is_empty()) {
            return return_data.set_error_msg("请输入搜索关键词");
        }

        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
        if book_info == None {
            return return_data.set_error_msg("请先加入书架");
        }
        let mut book_source: Option<String> = None;
        if !book_info.is_local_book() {
            book_source = self.get_book_source_string(context, book_info.origin);
            if book_source.map_or(true, |s| s.is_empty()) {
                return return_data.set_error_msg("未配置书源");
            }
        }

        let chapter_list = self.get_local_chapter_list(book_info, book_source.unwrap_or(""), false, user_name_space);
        if last_index >= chapter_list.size {
            return return_data.set_error_msg("没有更多了");
        }

        let mut is_end = false;
        context.request().connection().close_handler {
            LOGGER.info(format!("客户端已断开链接，停止 searchBookContent"));
            is_end = true;
            self.coroutine_context.cancel();
        }

        LOGGER.info(format!("searchBookContent keyword: {} lastIndex: {}", keyword, last_index));
        let mut result_list = mutable_list_of::<SearchResult>();
        last_index += 1;
        let mut current_index = last_index;
        for chapter_index in last_index..chapter_list.size {
            current_index = chapter_index;
            let chapter = chapter_list.get(chapter_index);
            let chapter_result = self.search_chapter(book_info, chapter, keyword);
            if chapter_result.size > 0 {
                result_list.add_all(chapter_result);
            }

            if result_list.size >= size || is_end {
                break;
            }
        }
        return return_data.set_data(map!("list" to result_list, "lastIndex" to current_index));
    }

    async fn search_chapter(&self, book: Book, chapter: BookChapter, query: String) -> List<SearchResult> {
        let search_results_within_chapter: MutableList<SearchResult> = mutable_list_of();
        let chapter_content = BookHelp::get_content(book, chapter);
        if chapter_content != None {
            // withContext(Dispatchers.IO) {
            //     chapter.title = when (AppConfig.chineseConverterType) {
            //         1 -> ChineseUtils.t2s(chapter.title)
            //         2 -> ChineseUtils.s2t(chapter.title)
            //         else -> chapter.title
            //     }
            //     mContent = contentProcessor!!.getContent(
            //         book, chapter, chapterContent,
            //         chineseConvert = true,
            //         reSegment = false,
            //         useReplace = false
            //     ).joinToString("")
            // }
            let positions = self.search_position(chapter_content, query);
            LOGGER.info(format!("positions: {}", positions));
            for (index, position) in positions.enumerate() {
                let construct = self.get_result_and_query_index(chapter_content, position, query);
                let result = SearchResult::new(
                    result_count_within_chapter = index,
                    result_text = construct.second,
                    chapter_title = chapter.title,
                    query = query,
                    chapter_index = chapter.index,
                    query_index_in_result = construct.first,
                    query_index_in_chapter = position
                );
                search_results_within_chapter.add(result);
            }
        }
        return search_results_within_chapter;
    }

    async fn search_position(&self, m_content: String, pattern: String) -> List<i32> {
        let position: MutableList<i32> = mutable_list_of();
        let mut index = m_content.index_of(pattern);
        if index >= 0 {
            //搜索到内容允许净化
            // if (book!!.getUseReplaceRule()) {
            //     mContent = contentProcessor!!.replaceContent(mContent)
            //     index = mContent.indexOf(pattern)
            // }
            while index >= 0 {
                position.add(index);
                index = m_content.index_of(pattern, index + 1);
            }
        }
        return position;
    }

    fn get_result_and_query_index(
        &self,
        content: String,
        query_index_in_content: i32,
        query: String
    ) -> Pair<i32, String> {
        // 左右移动20个字符，构建关键词周边文字，在搜索结果里显示
        // todo: 判断段落，只在关键词所在段落内分割
        // todo: 利用标点符号分割完整的句
        // todo: length和设置结合，自由调整周边文字长度
        let length = 20;
        let mut po1 = query_index_in_content - length;
        let mut po2 = query_index_in_content + query.length + length;
        if po1 < 0 {
            po1 = 0;
        }
        if po2 > content.length {
            po2 = content.length;
        }
        let query_index_in_result = query_index_in_content - po1;
        let new_text = content.substring(po1, po2);
        return Pair::new(query_index_in_result, new_text);
    }
    #[lazy]
    static BACKUP_FILE_NAMES: &[&str] = &[
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

    fn mongo_user_namespaces(&self) -> List<String> {
        let mut namespaces = array_list!("default");
        if !app_config.secure { return namespaces }
        let users = as_json_object(get_storage("data", "users"))?.map.unwrap_or(return) namespaces;
        for value in users.values {
            let username = (value as? Map<*, *>)?.get("username") as? String.unwrap_or("");
            if username.is_not_empty() { namespaces += username }
        }
        return namespaces;
    }

    async fn backup_to_mongodb(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        if !MongoManager::is_init() {
            return return_data.set_error_msg("请先设置 mongoUri");
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data("NEED_SECURE_KEY").set_error_msg("请输入管理密码");
        }

        for user_name_space in self.mongo_user_namespaces() {
            for file_name in self.backup_file_names {
                if let Some(content) = self.base.get_user_storage(user_name_space, file_name) {
                    self.base.save_user_storage(user_name_space, file_name, content);
                }
            }
        }
        if let Some(it) = get_storage("users") { save_storage("users", value = it) }
        return return_data.set_data("");
    }

    async fn restore_from_mongodb(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        if !MongoManager::is_init() {
            return return_data.set_error_msg("请先设置 mongoUri");
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data("NEED_SECURE_KEY").set_error_msg("请输入管理密码");
        }

        for user_name_space in self.mongo_user_namespaces() {
            for file_name in self.backup_file_names {
                let file = File::new(get_work_dir("storage", "data", user_name_space, "${file_name}.json"));
                if file.exists() { file.delete() }
            }
        }
        let users_file = File::new(get_work_dir_multi(&["storage", "users.json"]));
        if users_file.exists() {
            users_file.delete();
            get_storage("users");
        }
        return return_data.set_data("");
    }

    async fn cache_book_on_server(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url_list = context.body_as_json().get_json_array("bookUrlList") ?: JsonArray::new();
        if book_url_list.is_empty {
            return return_data.set_error_msg("请输入书籍链接");
        }
        let exception_handler = CoroutineExceptionHandler::new(|_, exception| {
            LOGGER.info(format!("cacheBookOnServer error: {}", exception.message));
        });
        let user_name_space = self.base.get_user_name_space(context);
        launch(MDCContext() + Dispatchers::IO + exception_handler) {
            self.cache_book_on_server(book_url_list, user_name_space);
        };
        return return_data.set_data("");
    }

    async fn cache_book_on_server(&self, chapters: JsonArray, user_name_space: String) {
        for i in 0..chapters.size() {
            let book_url = chapters.get_string(i);
            let book_info = self.get_shelf_book_by_url(book_url, user_name_space);
            if book_info == None {
                LOGGER.info(format!("未找到书籍信息: {}", book_url));
                continue;
            }
            if book_info.is_local_book() {
                LOGGER.info(format!("本地书籍跳过缓存: {}", book_url));
                continue;
            }
            LOGGER.info(format!("开始缓存书籍: {}", book_info));
            let book_source = self.get_book_source_string_by_source_url_opt(book_info.origin, user_name_space);
            if book_source.map_or(true, |s| s.is_empty()) {
                LOGGER.info(format!("未找到书源信息: {}", book_url));
                continue;
            }
            let chapter_list = self.get_local_chapter_list(book_info, book_source, false, user_name_space, false);
            let mut cached_chapter_content_set = self.get_cached_chapter_content_set(book_info, user_name_space);
            let cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
            for chapter_index in chapter_list.indices {
                if cached_chapter_content_set.contains(chapter_index) {
                    continue;
                }
                let chapter = chapter_list[chapter_index];
                let next_chapter_url = chapter_list.get_or_null(chapter_index + 1)?.url;
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let content = self.web_book(book_source, self.base.get_app_config_debug_log(), user_name_space)
                        .get_book_content(book_info, chapter, next_chapter_url);
                    let cache_file = File::new(cache_dir, "${chapter_index}.txt");
                    cache_file.write_text(content);
                    let parsed_source = BookSource::from_json(book_source).get_or_null().unwrap_or(BookSource)::new();
                    BookHelp::save_images(self, parsed_source, book_info, chapter, content);
                    cached_chapter_content_set.add(chapter_index);
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                    LOGGER.info(format!("cacheBookOnServer error: {}", e.message));
                }
        }
            }
            LOGGER.info(format!("缓存书籍完成: {}", book_info));
        }
    }

    fn get_book_source_string_by_source_url_opt(&self, source_url: String, user_name_space: String) -> Option<String> {
        if source_url.is_blank() {
            return None;
        }
        let mut source_file = get_storage_file("data", user_name_space, "bookSource");
        if !source_file.exists() {
            source_file = get_storage_file("data", "default", "bookSource");
            if !source_file.exists() {
                return None;
            }
        }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut result: Option<String> = None;
            let parser = ObjectMapper::new().factory.create_parser(source_file);
            if parser.next_token() == JsonToken::START_ARRAY {
                while parser.next_token() != JsonToken::END_ARRAY {
                    if parser.current_token() != JsonToken::START_OBJECT {
                        continue;
                    }
                    let node: JsonNode = parser.read_value_as_tree();
                    if source_url == node.get("bookSourceUrl")?.as_text() {
                        result = node.to_string();
                        break;
                    }
                }
            }
            LOGGER.info(format!("{}", result));
            return result;
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
            LOGGER.error(format!("解析文件内容出错: {}  文件: \n{}", e, source_file));
            throw e;
        }
        }
    }

    async fn create_user_backup(
        &self,
        user_name_space: String,
        backup_dir: String,
        latest_zip_file_path: Option<String> = None
    ) -> Option<File> {
        let today = SimpleDateFormat::new("yyyy-MM-dd").format(System::current_time_millis());
        let staging_dir = File::new(get_work_dir("storage", "data", user_name_space, "backup" + today));
        staging_dir.delete_recursively();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if latest_zip_file_path != None && !File::new(latest_zip_file_path).unzip(staging_dir.absolute_path) {
                return None;
            }

            for file_name in self.backup_file_names {
                let source = File::new(get_work_dir("storage", "data", user_name_space, file_name));
                if !source.exists() { continue }
                let destination = File::new(staging_dir, file_name);
                destination.delete_recursively();
                source.copy_recursively(destination, overwrite = false);
            }

            let webdav_books = File::new(get_work_dir("storage", "data", user_name_space, "webdav", "books"));
            if webdav_books.exists() {
                let destination = File::new(staging_dir, "books");
                destination.delete_recursively();
                webdav_books.copy_recursively(destination, overwrite = true);
            }

            let output = FileUtils::create_file_with_replace(
                File::new(backup_dir, "backup" + today + ".zip").absolute_path
            );
            let files = staging_dir.list_files()?.to_list().unwrap_or(return) None;
            return if io.legado.app.utils.ZipUtils.zip_files(files, output) { output } else { None };
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
            LOGGER.error(format!("createUserBackup error: {}", e.message));
            return None;
        } finally {
            staging_dir.delete_recursively();
        }
        }
    }

    async fn text_to_speech(&self, context: RoutingContext) {
        if !self.base.check_auth(context) {
            context.response().set_status_code(403).end("未登录");
            return;
        }
        let body = if context.request().method() == HttpMethod::POST { context.body_as_json() } else { None };
        fn value(name: String) -> String = if body != None { body.get_string(name).unwrap_or("") } else { context.query_param(name).unwrap_or("") };
        let text = value("text");
        let type = value("type").if_empty { "edge" };
        if text.is_empty() {
            context.response().set_status_code(404).end("参数错误");
            return;
        }
        let options = map!(
            "voice" to value("voice"),
            "pitch" to value("pitch"),
            "rate" to value("rate"),
            "base64" to value("base64")
        );
        let response = context.response();
        let exception_handler = CoroutineExceptionHandler::new(|_, exception| {
            LOGGER.info(format!("tts error: {}", exception.message));
            response.set_status_code(404).end();
        });
        launch(MDCContext() + Dispatchers::IO + exception_handler) {
            match type {
                "edge" => self.tts_by_edge(response, text, options),
                "textToSpeechCn" => self.tts_by_text_to_speech_cn(response, text, options),
                _ => self.tts_by_api(response, text, self.base.get_user_name_space(context), options),
            }
        };
    }

    async fn tts_by_edge(&self, response: HttpServerResponse, text: String, params: Option<Map<String, String>> = None) {
        let voice = com.htmake.reader.lib.tts.constant.VoiceEnum::from_sort_name(params?.get("voice"))
            ?: com.htmake.reader.lib.tts.constant.VoiceEnum::zh_CN_XiaoxiaoNeural;
        let rate = if params?.contains_key("rate") == true { params["rate"].unwrap_or("0") } else { "0" };
        let pitch = if params?.contains_key("pitch") == true {
            (params["pitch"].unwrap_or("null")) + "%"
        } else {
            "0%"
        };
        let tts_service = com.htmake.reader.lib.tts.service.TTSService::builder()
            .build();
        let ssml = com.htmake.reader.lib.tts.model.SSML::builder()
            .synthesis_text(text)
            .voice(voice)
            .rate(rate)
            .pitch(pitch)
            .style(com.htmake.reader.lib.tts.constant.TtsStyleEnum::chat)
            .build();
        let audio_bytes = tts_service.send_text(ssml);
        if params?.get("base64") == "1" {
            response.put_header("content-type", "application/json; charset=utf-8")
                .end(json_encode(ReturnData::new().set_data(Base64::get_encoder().encode_to_string(audio_bytes))));
        } else {
            response.put_header("Content-Type", "audio/mpeg").end(io.vertx.core.buffer.Buffer::buffer(audio_bytes));
        }
    }

    async fn tts_by_api(&self, response: HttpServerResponse, text: String, user_name_space: String, params: Option<Map<String, String>> = None) {
        let voice = params?.get("voice");
        if voice.map_or(true, |s| s.is_empty()) {
            response.set_status_code(404).end();
            return;
        }
        let http_tts = self.get_http_tts_by_name(voice, user_name_space);
        if http_tts == None {
            response.set_status_code(404).end();
            return;
        }
        let speech_rate = (5 + ((params?.get("rate")?.to_double().unwrap_or(1.0)) - 0.5) * 30).to_int();
        let stream = self.get_speak_stream(http_tts, text, speech_rate);
        if stream == None {
            response.set_status_code(404).end();
            return;
        }
        let bytes = stream.read_bytes();
        if params?.get("base64") == "1" {
            response.put_header("content-type", "application/json; charset=utf-8")
                .end(json_encode(ReturnData::new().set_data(Base64::get_encoder().encode_to_string(bytes))));
        } else {
            response.put_header("Content-Type", http_tts.content_type.unwrap_or("audio/mpeg"))
                .end(io.vertx.core.buffer.Buffer::buffer(bytes));
        }
    }

    async fn tts_by_text_to_speech_cn(&self, response: HttpServerResponse, text: String, params: Option<Map<String, String>> = None) {
        let form = MultiMap::case_insensitive_multi_map();
        form.add("language", "中文（普通话，简体）");
        form.add("voice", "zh-CN-XiaoxiaoNeural");
        form.add("text", text);
        form.add("role", "0");
        form.add("style", "0");
        form.add("rate", "0");
        form.add("pitch", "0");
        form.add("kbitrate", "audio-16khz-32kbitrate-mono-mp3");
        form.add("silence", "");
        form.add("styledegree", "1");
        form.add("user_id", "");
        form.add("yzm", "");
        if let Some(params) = params {
            for (key, value) in params {
                form.set(key, value);
            }
        }
        let result = await_result::<io.vertx.ext.web.client.HttpResponse<io.vertx.core.buffer.Buffer>>(|handler| {
            web_client.post_abs("https://www.text-to-speech.cn/getSpeek.php")
                .timeout(5000)
                .put_header("Origin", "https://www.text-to-speech.cn")
                .put_header("Referer", "https://www.text-to-speech.cn/")
                .put_header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
                .send_form(form, |ar| { handler.handle(ar) });
        });
        LOGGER.info(format!("res: {}", result));
        let json = result.body_as_json_object();
        LOGGER.info(format!("jsonRes: {}", json));
        if json != None {
            let download = json.get_string("download");
            if download != None {
                response.set_status_code(302).put_header("Location", download).end();
            } else {
                response.set_status_code(404).end();
            }
        } else {
            response.set_status_code(404).end();
        }
    }

    /// Look up HttpTTS source by name from user storage.
    fn get_http_tts_by_name(&self, name: String, user_name_space: String) -> Option<HttpTTS> {
        if name.is_empty() {
            return None;
        }
        let http_tts_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "httpTTS"));
        if http_tts_list == None {
            return None;
        }
        for i in 0..http_tts_list.size() {
            let obj = http_tts_list.get_json_object(i);
            if obj != None {
                let parsed = HttpTTS::from_json(obj.to_string()).get_or_null();
                if parsed?.name == name { return parsed }
            }
        }
        return None;
    }
    async fn get_speak_stream(
        &self,
        http_tts: HttpTTS,
        speak_text: String,
        speech_rate: i32
    ) -> Option<InputStream> {
        let mut download_error_no = 0;
        loop {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let analyze_url = AnalyzeUrl::new(
                    m_url = http_tts.url,
                    speak_text = speak_text,
                    speak_speed = speech_rate,
                    source = http_tts,
                    header_map_f = http_tts.get_header_map(true),
                    debug_log = Debug
                );
                let mut response = analyze_url.get_response_await();
                if let Some(check_js) = http_tts.login_check_js?.take_if { it.is_not_blank() } {
                    response = analyze_url.eval_js(check_js, response) as okhttp3::Response;
                }

                self.coroutine_context.ensure_active();
                if let Some(content_type) = response.header("Content-Type") {
                    if content_type == "application/json" {
                        throw NoStackTraceException::new(response.body!!.string());
                    }
                    if let Some(expected_content_type) = http_tts.content_type?.take_if { it.is_not_blank() } {
                        if !Regex::new(expected_content_type).matches(content_type) {
                            throw NoStackTraceException::new(
                                "TTS服务器返回错误：" + response.body!!.string()
                            );
                        }
                    }
                }
                self.coroutine_context.ensure_active();
                let body = response.body!!;
                download_error_no = 0;
                return body.byte_stream();
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
                if e is kotlinx.coroutines.CancellationException { throw e }
                if e is ScriptException || e is WrappedException {
                    LOGGER.error(format!("js错误\n{}", e.localized_message, e));
                    throw e;
                }
                download_error_no++;
                if e is SocketTimeoutException || e is ConnectException {
                    if download_error_no <= 5 { continue }
                    LOGGER.error(format!("tts超时或连接错误超过5次\n{}", e.localized_message, e));
                    throw e;
                }
                LOGGER.error(format!("tts下载错误\n{}", e.localized_message, e));
                if download_error_no > 5 {
                    LOGGER.error(format!("TTS服务器连续5次错误，已暂停阅读。", e));
                    throw e;
                }
                LOGGER.error(format!("TTS下载音频出错，使用无声音频代替。\n朗读文本：{}", speak_text));
                return None;
            }
        }
        }
    }

    async fn save_book_content(&self, context: RoutingContext) -> ReturnData {
        let return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data("NEED_LOGIN").set_error_msg("请登录后使用");
        }
        let book_url = context.body_as_json().get_string("url").unwrap_or_default();
        let chapter_index = context.body_as_json().get_integer("index", -1);
        let content = context.body_as_json().get_string("content").unwrap_or_default();

        if book_url.is_empty() {
            return return_data.set_error_msg("请输入书籍链接");
        }

        let user_name_space = self.base.get_user_name_space(context);
        let book_info = self.get_shelf_book_by_url(book_url, user_name_space)
            ?: return return_data.set_error_msg("获取书籍信息失败");

        let cache_dir = self.get_chapter_cache_dir(book_info, user_name_space);
        let chapter_file = File::new(cache_dir, "${chapter_index}.txt");
        chapter_file.write_text(content);
        let custom_cache_dir = File::new(
            get_work_dir("storage", "data", user_name_space, book_info.name + "_" + book_info.author, "custom")
        );
        if !custom_cache_dir.exists() {
            custom_cache_dir.mkdirs();
        }
        File::new(custom_cache_dir, "${chapter_index}.txt").write_text(content);

        return return_data.set_data("");
    }

    /// Convert all PDF pages to images for a book.
    /// JAR signature: public final boolean convertPdfToImage(io.legado.app.data.entities.Book, boolean)
    fn convert_pdf_to_image(&self, book: Book, force: bool = false) -> bool {
        return true;
    }

    /// Convert a single PDF page to image for a book.
    /// JAR signature: public final void convertPdfPageToImage(io.legado.app.data.entities.Book, int, boolean)
    fn convert_pdf_page_to_image(&self, book: Book, page_index: i32, force: bool = false) {
        let image_dir = File::new(get_work_dir(book.book_url + File::separator + "index"));
        if !image_dir.exists() {
            image_dir.mkdirs();
        }
        let image_format = "png";
        let output_file = File::new(image_dir.to_string() + File::separator + "output-" + page_index + "." + image_format);
        if !force && output_file.exists() {
            return;
        }
        output_file.delete_recursively();
        let mut local_file = File::new(get_work_dir(book.origin_name + File::separator + "index.pdf"));
        if book.origin_name.index_of("localStore") > 0 {
            local_file = File::new(get_work_dir(book.origin_name));
        }
        if book.origin_name.index_of("webdav") > 0 {
            local_file = File::new(get_work_dir(book.origin_name));
        }
        let doc = org.apache.pdfbox.pdmodel.PDDocument::load(local_file);
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let renderer = org.apache.pdfbox.rendering.PDFRenderer::new(doc);
            let target_width = book.get_pdf_image_width();
            self.save_pdf_page_to_image(doc, renderer, page_index, target_width, image_format, output_file);
        }));
            doc.close();
    }

    /// Render one PDF page and save it as an image file.
    /// JAR signature: public final void savePdfPageToImage(PDDocument, PDFRenderer, int, float, String, File)
    fn save_pdf_page_to_image(&self, document: org.apache.pdfbox.pdmodel.PDDocument, renderer: org.apache.pdfbox.rendering.PDFRenderer, page_index: i32, dpi: f32, image_format: String, output: File) {
        let render_dpi = 300.0;
        let page = document.get_page(page_index);
        let crop_box = page.crop_box;
        let target_height: f32 = 0.0;
        let scale = dpi / crop_box.width;
        let scaled_height = crop_box.height * scale;
        let height = if target_height == 0.0 { scaled_height.to_int() } else { target_height.to_int() };
        let dimension = java.awt.Dimension::new(dpi.to_int(), height);
        let image = renderer.render_image_with_dpi(page_index, render_dpi, org.apache.pdfbox.rendering.ImageType::RGB);
        let scaled_image = image.get_scaled_instance(dimension.width, dimension.height, java.awt.Image::SCALE_SMOOTH);
        let buffered_image = java.awt.image.BufferedImage::new(dimension.width, dimension.height, java.awt.image.BufferedImage::TYPE_INT_RGB);
        let g2d = buffered_image.create_graphics();
        g2d.draw_image(scaled_image, 0, 0, None);
        g2d.dispose();
        javax.imageio.ImageIO::write(buffered_image, image_format, output);
    }

    /// Save a book to the shelf. Encapsulates reusable logic from saveBook().
    /// JAR signature: public final kotlin.Pair<Book, String?> saveBookToShelf(Book, String, RoutingContext)
    fn save_book_to_shelf(&self, _book: Book, user_name_space: String, context: RoutingContext) -> Pair<Book, Option<String>> {
        let mut book = _book;
        if book.origin.map_or(true, |s| s.is_empty()) {
            return Pair::new(book, "未找到书源信息");
        }
        if book.book_url.map_or(true, |s| s.is_empty()) {
            return Pair::new(book, "书籍链接不能为空");
        }
        let mut bookshelf: Option<JsonArray> = as_json_array(self.base.get_user_storage(user_name_space, "bookshelf"));
        if bookshelf == None {
            bookshelf = JsonArray::new();
        }
        // 遍历判断书本是否存在
        let mut exist_index: i32 = -1;
        for i in 0..bookshelf.size() {
            let name = bookshelf.get_json_object(i).get_string("name", "");
            let author = bookshelf.get_json_object(i).get_string("author", "");
            if name.equals(book.name) && author.equals(book.author) {
                exist_index = i;
                break;
            }
        }
        if exist_index < 0 {
            // 判断书籍是否超过限制
            let user_info = context.get("userInfo") as com.htmake.reader.entity.User?;
            if user_info != None && bookshelf.size() >= user_info.book_limit {
                return Pair::new(book, "你已达到书籍数上限，请联系管理员");
            }
        }
        // 导入本地书籍
        if book.is_local_book() {
            if book.book_url.starts_with("/assets/") || book.book_url.starts_with("assets/") {
                // 临时文件，移动到书籍目录
                let temp_file = File::new(get_work_dir("storage" + book.book_url));
                if !temp_file.exists() {
                    return Pair::new(book, "上传书籍不存在");
                }
                let relative_local_file_path = Paths::get("storage", "data", user_name_space, book.name + "_" + book.author, temp_file.name).to_string();
                let book_url = "storage/data/" + user_name_space + "/" + book.name + "_" + book.author + "/" + temp_file.name;
                let local_file_path = get_work_dir(relative_local_file_path);
                LOGGER.info(format!("localFilePath: {}", local_file_path));
                let local_file = File::new(local_file_path);
                local_file.delete_recursively();
                if !local_file.parent_file.exists() {
                    local_file.parent_file.mkdirs();
                }
                if !temp_file.copy_recursively(local_file) {
                    return Pair::new(book, "导入本地书籍失败");
                }
                temp_file.delete_recursively();
                book.book_url = book_url;
                book.origin_name = relative_local_file_path;

                if book.is_epub() {
                    if !self.extract_epub(book) {
                        return Pair::new(book, "导入本地Epub书籍失败");
                    }
                } else if book.is_cbz() {
                    if !self.extract_cbz(book) {
                        return Pair::new(book, "导入本地CBZ书籍失败");
                    }
                } else if book.is_pdf() {
                    if !self.convert_pdf_to_image(book) {
                        return Pair::new(book, "本地PDF书籍转换失败");
                    }
                }
            } else if book.book_url.index_of("localStore") >= 0 {
                let temp_file = File::new(get_work_dir(book.book_url));
                if !temp_file.exists() {
                    return Pair::new(book, "本地书仓书籍不存在");
                }
                let relative_local_file_path = Paths::get("storage", "data", user_name_space, book.name + "_" + book.author, temp_file.name).to_string();
                book.book_url = relative_local_file_path;

                if book.is_epub() {
                    if !self.extract_epub(book) {
                        return Pair::new(book, "导入本地Epub书籍失败");
                    }
                } else if book.is_cbz() {
                    if !self.extract_cbz(book) {
                        return Pair::new(book, "导入本地CBZ书籍失败");
                    }
                } else if book.is_pdf() {
                    if !self.convert_pdf_to_image(book) {
                        return Pair::new(book, "本地PDF书籍转换失败");
                    }
                }
            } else if book.book_url.index_of("webdav") >= 0 {
                let temp_file = File::new(get_work_dir(book.book_url));
                if !temp_file.exists() {
                    return Pair::new(book, "webdav书仓书籍不存在");
                }
                let relative_local_file_path = Paths::get("storage", "data", user_name_space, book.name + "_" + book.author, temp_file.name).to_string();
                book.book_url = relative_local_file_path;

                if book.is_epub() {
                    if !self.extract_epub(book) {
                        return Pair::new(book, "导入本地Epub书籍失败");
                    }
                } else if book.is_cbz() {
                    if !self.extract_cbz(book) {
                        return Pair::new(book, "导入本地CBZ书籍失败");
                    }
                } else if book.is_pdf() {
                    if !self.convert_pdf_to_image(book) {
                        return Pair::new(book, "本地PDF书籍转换失败");
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
            let exist_book = bookshelf.get_json_object(exist_index).map_to::<Book>();
            book.dur_chapter_index = exist_book.dur_chapter_index;
            book.dur_chapter_title = exist_book.dur_chapter_title;
            book.dur_chapter_time = exist_book.dur_chapter_time;
            let old_cover_url = exist_book.get_display_cover();
            if !old_cover_url.map_or(true, |s| s.is_empty()) && old_cover_url.starts_with("/") && old_cover_url != book.get_display_cover() {
                FileUtils::delete_file(get_work_dir("storage" + old_cover_url));
            }
            book_list.set(exist_index, JsonObject::map_from(book));
            bookshelf = JsonArray::new(book_list);
        } else {
            bookshelf.add(JsonObject::map_from(book));
        }
        self.save_book_sources(book, list!(book.to_search_book()), user_name_space);
        self.base.save_user_storage(user_name_space, "bookshelf", bookshelf);
        return Pair::new(book, None);
    }

    /// Download and save a book's cover image locally.
    /// JAR signature: public final Object saveBookCover(Book, String, String?, Continuation)
    async fn save_book_cover(&self, book: Book, user_name_space: String, book_source: Option<String> = None) {
        let cover_url = book.get_display_cover();
        if cover_url == None || cover_url.starts_with("/") {
            return;
        }
        let source = if book_source != None {
            book_source
        } else {
            self.get_book_source_string_by_source_url_opt(book.origin, user_name_space)
        };
        let ext = self.base.get_file_ext(cover_url, "jpg");
        let md5_encode = MD5Utils::md5_encode(cover_url).to_string();
        let cache_path = get_work_dir("storage", "assets", user_name_space, "covers", md5_encode + "." + ext);
        let cover_local_url = "/assets/" + user_name_space + "/covers/" + md5_encode + "." + ext;
        let cache_file = File::new(cache_path);
        if cache_file.exists() {
            book.cover_url = cover_local_url;
            return;
        }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let analyze_url = io.legado.app.model.analyzeRule.AnalyzeUrl::new(
                cover_url,
                source = source.and_then(|it| BookSource::from_json(it).get_or_null())
            );
            let bytes = analyze_url.get_byte_array_await();
            FileUtils::write_bytes(cache_path, bytes);
            book.cover_url = cover_local_url;
})) { Ok(_) => {}, Err(e) => { let e = crate::stubs::panic_message(e);
            e.print_stack_trace();
        }
        }
    }

    async fn save_local_book_cover(&self, book: Book, user_name_space: String) {
        let cover_url = book.get_display_cover();
        if cover_url.map_or(true, |s| s.is_empty()) || cover_url.starts_with("/") { return }
        let ext = self.base.get_file_ext(cover_url, "jpg");
        let md5_encode = MD5Utils::md5_encode(cover_url).to_string();
        let cache_path = get_work_dir("storage", "assets", user_name_space, "covers", "${md5_encode}.${ext}");
        let cached_cover_url = "/assets/${user_name_space}/covers/${md5_encode}.${ext}";
        let cache_file = File::new(cache_path);
        if cache_file.exists() {
            book.cover_url = cached_cover_url;
            return;
        }
        let response = await_result::<io.vertx.ext.web.client.HttpResponse<io.vertx.core.buffer.Buffer>>(|handler| {
            web_client.get_abs(cover_url).timeout(3000).send(handler);
        });
        let body_bytes = response.body_as_buffer()?.bytes;
        if body_bytes != None {
            cache_file.write_bytes(body_bytes);
            book.cover_url = cached_cover_url;
        }
    }

    fn update_image_link_in_content(&self, book: Book, chapter: BookChapter, content: String) -> String {
        let data_dir = get_work_dir_multi(&["storage", "data"]);
        let lines = content.split("\n");
        let sb = StringBuilder::new();
        for text in lines {
            let mut line_text = text;
            let matcher = io.legado.app.constant.AppPattern::imgPattern.matcher(text);
            while matcher.find() {
                let src = matcher.group(1).unwrap_or(continue);
                if src.contains("__API_ROOT__") { continue }
                let abs_url = io.legado.app.utils.NetworkUtils::get_absolute_url(chapter.url, src);
                let image_file = io.legado.app.help.BookHelp::get_image(book, abs_url);
                if image_file.exists() {
                    let image_url = "__API_ROOT__" + image_file.path.replace(data_dir, "/book-assets");
                    line_text = line_text.replace(src, "${image_url}\" data-error=\"${src}");
                }
            }
            sb.append(line_text).append("\n");
        }
        return sb.to_string();
    }
}
