use crate::prelude::*;
// fix: 显式导入消解 prelude glob 歧义 / 缺省名称（stubs 占位 + vertext 真实实现）
use crate::stubs::io::vertx::Router;
// fix: 真实控制器模块（BookController 等已实现真实逻辑）
use crate::com_htmake_reader_api_controller_bookcontroller::BookController;
use crate::com_htmake_reader_api_controller_bookgroupcontroller::BookGroupController;
use crate::com_htmake_reader_api_controller_bookmarkcontroller::BookmarkController;
use crate::com_htmake_reader_api_controller_booksourcecontroller::BookSourceController;
use crate::com_htmake_reader_api_controller_filecontroller::FileController;
use crate::com_htmake_reader_api_controller_httpttscontroller::HttpTTSController;
use crate::com_htmake_reader_api_controller_replacerulecontroller::ReplaceRuleController;
use crate::com_htmake_reader_api_controller_rsssourcecontroller::RssSourceController;
use crate::com_htmake_reader_api_controller_usercontroller::UserController;
use crate::com_htmake_reader_api_controller_webdavcontroller::WebdavController;
use crate::stubs::{
    File, MDC, RouteHandlerExt, RouterPostExt, Runtime, StaticHandler, uri_decode_component,
};
use crate::com_htmake_reader_utils_vertext::{get_work_dir, get_work_dir_multi};
// package com.htmake.reader.api

// private val logger = KotlinLogging.logger {}
#[allow(non_upper_case_globals)]
static logger: Log = Log;

// @Component
// class YueduApi : RestVerticle() {
pub struct YueduApi {
    // @Autowired
    // private lateinit var appConfig: AppConfig
    app_config: AppConfig,

    // @Autowired
    // private lateinit var env: Environment
    env: Environment,

    port: i32,
}

impl YueduApi {
    pub fn new() -> YueduApi {
        YueduApi {
            app_config: crate::com_htmake_reader_config_appconfig::AppConfig::default(),
            env: Environment,
            port: 8080,
        }
    }

    // override fun getContextPath(): String {
    //     return env.getProperty("reader.server.contextPath", "") ?: ""
    // }
    pub fn get_context_path(&self) -> String {
        return self.env.get_property_default("reader.server.contextPath", String::from(""));
    }

    // override suspend fun initRouter(router: Router) {
    //     setupPort()
    //
    //     if (appConfig.mongoUri.isNotEmpty()) {
    //         com.htmake.reader.utils.MongoManager.connect(appConfig.mongoUri)
    //     }
    //
    //     if (appConfig.remoteWebviewApi.isNotEmpty()) {
    //         RemoteWebview.setRemoteApi(appConfig.remoteWebviewApi)
    //     }
    //     ReaderAdapterHelper.setAdapter(ReaderAdapter)
    //
    //     // 旧版数据迁移
    //     migration()
    //
    //     // web界面
    //     router.route("/*").handler(StaticHandler.create("web").setDefaultContentEncoding("UTF-8"));
    //
    //     // assets
    //     var assetsDir = getWorkDir("storage", "assets");
    //     var assetsDirFile = File(assetsDir);
    //     if (!assetsDirFile.exists()) {
    //         assetsDirFile.mkdirs();
    //     }
    //     var assetsCss = getWorkDir("storage", "assets", "reader.css");
    //     var assetsCssFile = File(assetsCss);
    //     if (!assetsCssFile.exists()) {
    //         assetsCssFile.writeText("/* 在此处可以编写CSS样式来自定义页面 */");
    //     }
    //     router.route("/assets/*").handler(StaticHandler.create().setAllowRootFileSystemAccess(true).setWebRoot(assetsDir).setDefaultContentEncoding("UTF-8"));
    //
    //     // 书籍资源
    //     var dataDir = getWorkDir("storage", "data");
    //     router.route("/book-assets/*").handler {
    //         var path = it.request().path().replace("/book-assets/", "/", true)
    //         path = URIDecoder.decodeURIComponent(path, false)
    //         if ((path.endsWith("html", true) || path.endsWith("htm", true))) {
    //             val filePath = File(dataDir + path)
    //             if (filePath.exists()) {
    //                 BookConfig.injectJavascriptToEpubChapter(filePath.toString())
    //             }
    //         }
    //         it.next()
    //     }
    //     router.route("/book-assets/*").handler(StaticHandler.create().setAllowRootFileSystemAccess(true).setWebRoot(dataDir).setDefaultContentEncoding("UTF-8"))
    //
    //     // epub资源
    //     router.route("/epub/*").handler {
    //         var path = it.request().path().replace("/epub/", "/", true)
    //         path = URLDecoder.decode(path, "UTF-8")
    //         if (path.endsWith("html", true)) {
    //             var filePath = File(dataDir + path)
    //             if (filePath.exists()) {
    //                 // 处理 js 注入脚本
    //                 BookConfig.injectJavascriptToEpubChapter(filePath.toString())
    //             }
    //         }
    //         it.next()
    //     }
    //     router.route("/epub/*").handler(StaticHandler.create().setAllowRootFileSystemAccess(true).setWebRoot(dataDir).setDefaultContentEncoding("UTF-8"));
    //
    //     // simple-web界面
    //     router.route("/simple-web").handler {
    //         if (it.request().path().endsWith("/simple-web")) {
    //             val location = URLDecoder.decode(it.request().absoluteURI(), "UTF-8")
    //                 .replace("/simple-web", "/simple-web/", false)
    //             it.response().putHeader("Location", location).setStatusCode(302).end()
    //         } else {
    //             it.next()
    //         }
    //     }
    //     router.route("/simple-web/*").handler {
    //         it.next()
    //     }
    //     router.route("/simple-web/*").handler(StaticHandler.create("simple-web").setDefaultContentEncoding("UTF-8"))
    //
    //     // 获取系统信息
    //     router.get("/reader3/getSystemInfo").coroutineHandler { getSystemInfo(it) }
    //
    //     ////////// 接口部分
    //     val bookController = BookController(coroutineContext)
    //     val bookSourceController = BookSourceController(coroutineContext)
    //     val rssSourceController = RssSourceController(coroutineContext)
    //     val userController = UserController(coroutineContext)
    //     val webdavController = WebdavController(coroutineContext, router) { ctx, error ->
    //         onHandlerError(ctx, error)
    //     }
    //     val replaceRuleController = ReplaceRuleController(coroutineContext)
    //     val bookmarkController = BookmarkController(coroutineContext)
    //     val bookGroupController = BookGroupController(coroutineContext)
    //     val fileController = FileController(coroutineContext)
    //     val httpTTSController = HttpTTSController(coroutineContext)
    //
    //     /** 书源模块 */
    //     router.post("/reader3/saveBookSource").coroutineHandler { bookSourceController.saveBookSource(it) }
    //     ...
    // }
    pub fn init_router(&mut self, router: &mut Router) {
        self.setup_port();

        if !self.app_config.mongo_uri.is_empty() {
            MongoManager::connect(&self.app_config.mongo_uri);
        }

        if !self.app_config.remote_webview_api.is_empty() {
            RemoteWebview::set_remote_api(&self.app_config.remote_webview_api);
        }
        ReaderAdapterHelper::set_adapter(Box::new(ReaderAdapter));

        // 旧版数据迁移
        self.migration();

        // web界面
        router.route_with_path("/*").handler_static(StaticHandler::create("web").set_default_content_encoding("UTF-8"));

        // assets
        // fix: 目录经 Box::leak 转 'static（handler 闭包需 'static 且多个闭包共享）
        let assets_dir: &'static String = Box::leak(Box::new(get_work_dir_multi(&["storage", "assets"])));
        let assets_dir_file = File::new(&assets_dir);
        if !assets_dir_file.exists() {
            assets_dir_file.mkdirs();
        }
        let assets_css = get_work_dir_multi(&["storage", "assets", "reader.css"]);
        let assets_css_file = File::new(&assets_css);
        if !assets_css_file.exists() {
            assets_css_file.write_text("/* 在此处可以编写CSS样式来自定义页面 */");
        }
        router.route_with_path("/assets/*").handler_static(StaticHandler::create("").set_allow_root_file_system_access(true).set_web_root(assets_dir).set_default_content_encoding("UTF-8"));

        // 书籍资源
        let data_dir: &'static String = Box::leak(Box::new(get_work_dir_multi(&["storage", "data"])));
        router.route_with_path("/book-assets/*").handler(move |it| {
            let mut path = it.request().path().replace("/book-assets/", "/");
            path = uri_decode_component(&path, false);
            if path.to_lowercase().ends_with("html") || path.to_lowercase().ends_with("htm") {
                let file_path = File::new(&(data_dir.clone() + &path));
                if file_path.exists() {
                    BookConfig::inject_javascript_to_epub_chapter(file_path.to_string().as_str());
                }
            }
            it.next();
        });
        router.route_with_path("/book-assets/*").handler_static(StaticHandler::create("").set_allow_root_file_system_access(true).set_web_root(data_dir.clone()).set_default_content_encoding("UTF-8"));

        // epub资源
        router.route_with_path("/epub/*").handler(move |it| {
            let mut path = it.request().path().replace("/epub/", "/");
            path = url_decode(&path, "UTF-8");
            if path.to_lowercase().ends_with("html") {
                let file_path = File::new(&(data_dir.clone() + &path));
                if file_path.exists() {
                    // 处理 js 注入脚本
                    BookConfig::inject_javascript_to_epub_chapter(file_path.to_string().as_str());
                }
            }
            it.next();
        });
        router.route_with_path("/epub/*").handler_static(StaticHandler::create("").set_allow_root_file_system_access(true).set_web_root(data_dir).set_default_content_encoding("UTF-8"));

        // simple-web界面
        router.route_with_path("/simple-web").handler(move |it| {
            if it.request().path().ends_with("/simple-web") {
                let location = url_decode(&it.request().absolute_uri(), "UTF-8")
                    .replace("/simple-web", "/simple-web/");
                it.response().put_header("Location", location.as_str()).set_status_code(302).end(String::new());
            } else {
                it.next();
            }
        });
        router.route_with_path("/simple-web/*").handler(move |it| {
            it.next();
        });
        router.route_with_path("/simple-web/*").handler_static(StaticHandler::create("simple-web").set_default_content_encoding("UTF-8"));

        // fix: handler 闭包需 'static，self 为 &mut 借用——init_router 中 self 的可变使用（setup_port/migration）均在闭包定义前，之后仅只读，转 &'static 引用
        let self_ref: &'static YueduApi = unsafe { &*(self as *const YueduApi) };
        // 获取系统信息
        router.get("/reader3/getSystemInfo").coroutine_handler(move |it| self_ref.get_system_info(it));

        ////////// 接口部分
        // fix: 控制器经 Box::leak 转 'static（handler 闭包需 'static，控制器生命周期=程序运行期）
        let book_controller: &'static BookController = Box::leak(Box::new(BookController::new()));
        let book_source_controller: &'static BookSourceController = Box::leak(Box::new(BookSourceController::new()));
        let rss_source_controller: &'static RssSourceController = Box::leak(Box::new(RssSourceController::new()));
        let user_controller: &'static UserController = Box::leak(Box::new(UserController::new()));
        let webdav_controller: &'static WebdavController = Box::leak(Box::new(WebdavController::new(BaseController::new(), router, Box::new(move |ctx, error| {
            self_ref.on_handler_error(ctx, error);
        }))));
        let replace_rule_controller: &'static ReplaceRuleController = Box::leak(Box::new(ReplaceRuleController::new()));
        let bookmark_controller: &'static BookmarkController = Box::leak(Box::new(BookmarkController::new()));
        let book_group_controller: &'static BookGroupController = Box::leak(Box::new(BookGroupController::new()));
        let file_controller: &'static FileController = Box::leak(Box::new(FileController::new()));
        let http_tts_controller: &'static HttpTTSController = Box::leak(Box::new(HttpTTSController::new()));

        /** 书源模块 */
        router.post("/reader3/saveBookSource").coroutine_handler(move |it| book_source_controller.save_book_source(it));
        router.post("/reader3/saveBookSources").coroutine_handler(move |it| book_source_controller.save_book_sources_ctx(it));

        router.get("/reader3/getBookSource").coroutine_handler(move |it| book_source_controller.get_book_source(it));
        router.post("/reader3/getBookSource").coroutine_handler(move |it| book_source_controller.get_book_source(it));
        router.get("/reader3/getBookSources").coroutine_handler(move |it| book_source_controller.get_book_sources(it));
        router.post("/reader3/getBookSources").coroutine_handler(move |it| book_source_controller.get_book_sources(it));

        router.post("/reader3/deleteAllBookSources").coroutine_handler(move |it| book_source_controller.delete_all_book_sources(it));
        router.post("/reader3/deleteBookSource").coroutine_handler(move |it| book_source_controller.delete_book_source(it));
        router.post("/reader3/deleteBookSources").coroutine_handler(move |it| book_source_controller.delete_book_sources(it));

        // 上传书源文件
        router.post("/reader3/readSourceFile").coroutine_handler(move |it| book_source_controller.read_source_file(it));

        router.post("/reader3/saveFromRemoteSource").coroutine_handler_without_res(move |it| book_source_controller.save_from_remote_source(it));

        // 设置默认书源
        router.post("/reader3/setAsDefaultBookSources").coroutine_handler(move |it| book_source_controller.set_as_default_book_sources(it));
        router.post("/reader3/deleteUserBookSource").coroutine_handler(move |it| book_source_controller.delete_user_book_source(it));
        router.post("/reader3/deleteBookSourcesFile").coroutine_handler(move |it| book_source_controller.delete_book_sources_file(it));

        /** 书籍模块 */
        // 书架
        router.get("/reader3/getBookshelf").coroutine_handler(move |it| book_controller.get_bookshelf(it));
        router.get("/reader3/getShelfBook").coroutine_handler(move |it| book_controller.get_shelf_book(it));
        router.post("/reader3/saveBook").coroutine_handler(move |it| book_controller.save_book(it));
        router.post("/reader3/deleteBook").coroutine_handler(move |it| book_controller.delete_book(it));
        router.post("/reader3/deleteBooks").coroutine_handler(move |it| book_controller.delete_books(it));

        // 失效书源
        router.post("/reader3/getInvalidBookSources").coroutine_handler(move |it| book_controller.get_invalid_book_sources(it));

        // 探索
        router.post("/reader3/exploreBook").coroutine_handler(move |it| book_controller.explore_book(it));
        router.get("/reader3/exploreBook").coroutine_handler(move |it| book_controller.explore_book(it));

        // 搜索
        router.get("/reader3/searchBook").coroutine_handler(move |it| book_controller.search_book(it));
        router.post("/reader3/searchBook").coroutine_handler(move |it| book_controller.search_book(it));
        router.get("/reader3/searchBookMulti").coroutine_handler(move |it| book_controller.search_book_multi(it));
        router.post("/reader3/searchBookMulti").coroutine_handler(move |it| book_controller.search_book_multi(it));
        router.get("/reader3/searchBookMultiSSE").coroutine_handler_without_res(move |it| book_controller.search_book_multi_sse(it));

        // 书籍详情
        router.get("/reader3/getBookInfo").coroutine_handler(move |it| book_controller.get_book_info(it));
        router.post("/reader3/getBookInfo").coroutine_handler(move |it| book_controller.get_book_info(it));

        // 章节列表
        router.get("/reader3/getChapterList").coroutine_handler(move |it| book_controller.get_chapter_list(it));
        router.post("/reader3/getChapterList").coroutine_handler(move |it| book_controller.get_chapter_list(it));

        // 内容
        router.get("/reader3/getBookContent").coroutine_handler(move |it| book_controller.get_book_content(it));
        router.post("/reader3/getBookContent").coroutine_handler(move |it| book_controller.get_book_content(it));

        // 保存阅读进度
        router.post("/reader3/saveBookProgress").coroutine_handler(move |it| book_controller.save_book_progress(it));

        // 封面
        router.get("/reader3/cover").coroutine_handler_without_res(move |it| book_controller.get_book_cover(it));

        // 搜索其它来源
        router.get("/reader3/searchBookSource").coroutine_handler(move |it| book_controller.search_book_source(it));
        router.post("/reader3/searchBookSource").coroutine_handler(move |it| book_controller.search_book_source(it));
        router.get("/reader3/getAvailableBookSource").coroutine_handler(move |it| book_controller.get_available_book_source(it));
        router.post("/reader3/getAvailableBookSource").coroutine_handler(move |it| book_controller.get_available_book_source(it));
        router.get("/reader3/searchBookSourceSSE").coroutine_handler_without_res(move |it| book_controller.search_book_source_sse(it));

        // 换源
        router.get("/reader3/setBookSource").coroutine_handler(move |it| book_controller.set_book_source(it));
        router.post("/reader3/setBookSource").coroutine_handler(move |it| book_controller.set_book_source(it));

        // 修改分组
        router.post("/reader3/saveBookGroupId").coroutine_handler(move |it| book_controller.save_book_group_id(it));
        router.post("/reader3/addBookGroupMulti").coroutine_handler(move |it| book_controller.add_book_group_multi(it));
        router.post("/reader3/removeBookGroupMulti").coroutine_handler(move |it| book_controller.remove_book_group_multi(it));

        // 导入本地文件
        router.post("/reader3/importBookPreview").coroutine_handler(move |it| book_controller.import_book_preview(it));
        router.post("/reader3/refreshLocalBook").coroutine_handler(move |it| book_controller.refresh_local_book(it));

        // 获取txt章节规则
        router.get("/reader3/getTxtTocRules").coroutine_handler(move |it| book_controller.get_txt_toc_rules(it));
        router.post("/reader3/getChapterListByRule").coroutine_handler(move |it| book_controller.get_chapter_list_by_rule(it));

        // 书籍分组
        router.get("/reader3/getBookGroups").coroutine_handler(move |it| book_group_controller.get_book_groups(it));
        router.post("/reader3/saveBookGroup").coroutine_handler(move |it| book_group_controller.save_book_group(it));
        router.post("/reader3/deleteBookGroup").coroutine_handler(move |it| book_group_controller.delete_book_group(it));
        router.post("/reader3/saveBookGroupOrder").coroutine_handler(move |it| book_group_controller.save_book_group_order(it));

        // 调试书源
        router.get("/reader3/bookSourceDebugSSE").coroutine_handler_without_res(move |it| book_controller.book_source_debug_sse(it));

        // 缓存书籍章节
        router.get("/reader3/cacheBookSSE").coroutine_handler_without_res(move |it| book_controller.cache_book_sse(it));
        // 获取书籍缓存信息
        router.get("/reader3/getShelfBookWithCacheInfo").coroutine_handler(move |it| book_controller.get_shelf_book_with_cache_info(it));
        // 删除书籍章节缓存
        router.post("/reader3/deleteBookCache").coroutine_handler(move |it| book_controller.delete_book_cache(it));

        // 导出书籍
        router.post("/reader3/exportBook").coroutine_handler_without_res(move |it| book_controller.export_book(it));
        router.get("/reader3/exportBook").coroutine_handler_without_res(move |it| book_controller.export_book(it));

        // 全文搜索
        router.get("/reader3/searchBookContent").coroutine_handler(move |it| book_controller.search_book_content(it));
        router.post("/reader3/searchBookContent").coroutine_handler(move |it| book_controller.search_book_content(it));

        /** 用户模块 */
        // 上传文件
        router.post("/reader3/uploadFile").coroutine_handler(move |it| user_controller.upload_file(it));

        // 删除文件
        router.post("/reader3/deleteFile").coroutine_handler(move |it| user_controller.delete_file(it));

        // 登录
        router.post("/reader3/login").coroutine_handler(move |it| user_controller.login(it));
        // 注销登录
        router.post("/reader3/logout").coroutine_handler(move |it| user_controller.logout(it));

        // 获取用户信息
        router.get("/reader3/getUserInfo").coroutine_handler(move |it| user_controller.get_user_info(it));

        // 用户备份本地配置
        router.post("/reader3/saveUserConfig").coroutine_handler(move |it| user_controller.save_user_config(it));

        // 用户恢复本地配置
        router.get("/reader3/getUserConfig").coroutine_handler(move |it| user_controller.get_user_config(it));

        // 获取用户列表
        router.get("/reader3/getUserList").coroutine_handler(move |it| user_controller.get_user_list(it));

        // 删除用户
        router.post("/reader3/deleteUsers").coroutine_handler(move |it| user_controller.delete_users(it));

        // 添加用户
        router.post("/reader3/addUser").coroutine_handler(move |it| user_controller.add_user(it));

        // 重置用户密码
        router.post("/reader3/resetPassword").coroutine_handler(move |it| user_controller.reset_password(it));

        // 更新用户
        router.post("/reader3/updateUser").coroutine_handler(move |it| user_controller.update_user(it));


        /** rss模块 */
        // rss
        router.get("/reader3/getRssSources").coroutine_handler(move |it| rss_source_controller.get_rss_sources(it));
        router.post("/reader3/saveRssSource").coroutine_handler(move |it| rss_source_controller.save_rss_source(it));
        router.post("/reader3/saveRssSources").coroutine_handler(move |it| rss_source_controller.save_rss_sources(it));
        router.post("/reader3/deleteRssSource").coroutine_handler(move |it| rss_source_controller.delete_rss_source(it));
        // rss 列表
        router.get("/reader3/getRssArticles").coroutine_handler(move |it| rss_source_controller.get_rss_articles(it));
        router.post("/reader3/getRssArticles").coroutine_handler(move |it| rss_source_controller.get_rss_articles(it));
        // rss 内容
        router.get("/reader3/getRssContent").coroutine_handler(move |it| rss_source_controller.get_rss_content(it));
        router.post("/reader3/getRssContent").coroutine_handler(move |it| rss_source_controller.get_rss_content(it));

        /** 替换规则模块 */
        router.get("/reader3/getReplaceRules").coroutine_handler(move |it| replace_rule_controller.get_replace_rules(it));
        router.post("/reader3/saveReplaceRule").coroutine_handler(move |it| replace_rule_controller.save_replace_rule(it));
        router.post("/reader3/saveReplaceRules").coroutine_handler(move |it| replace_rule_controller.save_replace_rules(it));
        router.post("/reader3/deleteReplaceRule").coroutine_handler(move |it| replace_rule_controller.delete_replace_rule(it));
        router.post("/reader3/deleteReplaceRules").coroutine_handler(move |it| replace_rule_controller.delete_replace_rules(it));

        /** 书签模块 */
        router.get("/reader3/getBookmarks").coroutine_handler(move |it| bookmark_controller.get_bookmarks(it));
        router.post("/reader3/saveBookmark").coroutine_handler(move |it| bookmark_controller.save_bookmark(it));
        router.post("/reader3/saveBookmarks").coroutine_handler(move |it| bookmark_controller.save_bookmarks(it));
        router.post("/reader3/deleteBookmark").coroutine_handler(move |it| bookmark_controller.delete_bookmark(it));
        router.post("/reader3/deleteBookmarks").coroutine_handler(move |it| bookmark_controller.delete_bookmarks(it));

        router.post("/reader3/book/saveBookConfig").coroutine_handler(move |it| book_controller.save_book_config(it));
        router.get("/reader3/user/downloadBackupFile").coroutine_handler_without_res(move |it| user_controller.download_backup_file(it));

        router.get("/reader3/book/tts").coroutine_handler_without_res(move |it| book_controller.text_to_speech(it));
        router.post("/reader3/book/tts").coroutine_handler_without_res(move |it| book_controller.text_to_speech(it));
        // 保存书籍章节内容到缓存
        router.post("/reader3/saveBookContent").coroutine_handler(move |it| book_controller.save_book_content(it));

        /** MongoDB备份恢复 */
        router.post("/reader3/backupToMongodb").coroutine_handler(move |it| book_controller.backup_to_mongodb(it));
        router.post("/reader3/restoreFromMongodb").coroutine_handler(move |it| book_controller.restore_from_mongodb(it));

        /** 缓存书籍到服务器 */
        router.post("/reader3/cacheBookOnServer").coroutine_handler(move |it| book_controller.cache_book_on_server(it));

        /** 清理不活跃用户 */
        router.post("/reader3/clearInactiveUsers").coroutine_handler(move |it| user_controller.clear_inactive_users_ctx(it));

        /** webdav备份 */
        router.post("/reader3/backupToWebdav").coroutine_handler(move |it| webdav_controller.backup_to_webdav(it));

        /** 文件管理模块 */
        router.get("/reader3/file/list").coroutine_handler(move |it| file_controller.list(it));
        router.get("/reader3/file/get").coroutine_handler(move |it| file_controller.get(it));
        router.post("/reader3/file/save").coroutine_handler(move |it| file_controller.save(it));
        router.post("/reader3/file/mkdir").coroutine_handler(move |it| file_controller.mkdir(it));
        router.get("/reader3/file/download").coroutine_handler_without_res(move |it| file_controller.download(it));
        router.post("/reader3/file/upload").coroutine_handler(move |it| file_controller.upload(it));
        router.post("/reader3/file/delete").coroutine_handler(move |it| file_controller.delete(it));
        router.post("/reader3/file/deleteMulti").coroutine_handler(move |it| file_controller.delete_multi(it));
        router.post("/reader3/file/importPreview").coroutine_handler(move |it| file_controller.import_preview(it));
        router.post("/reader3/file/restore").coroutine_handler(move |it| file_controller.restore(it));
        router.get("/reader3/file/parse").coroutine_handler(move |it| file_controller.parse(it));
        router.post("/reader3/file/parse").coroutine_handler(move |it| file_controller.parse(it));

        /** HttpTTS模块 */
        router.get("/reader3/httpTTS/list").coroutine_handler(move |it| http_tts_controller.get_http_tts_list(it));
        router.post("/reader3/httpTTS/save").coroutine_handler(move |it| http_tts_controller.save_http_tts(it));
        router.post("/reader3/httpTTS/saveMulti").coroutine_handler(move |it| http_tts_controller.save_http_tts_list(it));
        router.post("/reader3/httpTTS/delete").coroutine_handler(move |it| http_tts_controller.delete_http_tts(it));
        router.post("/reader3/httpTTS/deleteMulti").coroutine_handler(move |it| http_tts_controller.delete_http_tts(it));
    }

    // suspend fun setupPort() {
    //     logger.info("port: {}", port)
    //     var serverPort = env.getProperty("reader.server.port", Int::class.java)
    //     logger.info("serverPort: {}", serverPort)
    //     if (serverPort != None && serverPort > 0) {
    //         port = serverPort;
    //     }
    // }
    pub fn setup_port(&mut self) {
        logger.info(format!("port: {}", self.port));
        let server_port = self.env.get_property_int("reader.server.port");
        logger.info(format!("serverPort: {}", server_port.map(|v| v.to_string()).unwrap_or_else(|| String::from("null"))));
        if let Some(server_port) = server_port {
            if server_port > 0 {
                self.port = server_port;
            }
        }
    }

    // suspend fun migration() {
    //     try {
    //         var storageDir = File(getWorkDir("storage"))
    //         var dataDir = File(getWorkDir("storage", "data", "default"))
    //         if (!storageDir.exists()) {
    //             // 直接使用新版本，则创建 default 目录，防止重启之后被迁移
    //             dataDir.mkdirs()
    //         } else if (!dataDir.exists()) {
    //             // 旧版本不管了
    //             dataDir.mkdirs()
    //         }
    //     } catch(e: Exception) {
    //         e.printStackTrace()
    //     }
    // }
    pub fn migration(&self) {
        let result = std::panic::catch_unwind(|| {
            let storage_dir = File::new(&get_work_dir("storage"));
            let data_dir = File::new(&get_work_dir_multi(&["storage", "data", "default"]));
            if !storage_dir.exists() {
                // 直接使用新版本，则创建 default 目录，防止重启之后被迁移
                data_dir.mkdirs();
            } else if !data_dir.exists() {
                // 旧版本不管了
                data_dir.mkdirs();
            }
        });
        if let Err(e) = result {
            eprintln!("{:?}", e);
        }
    }

    // override fun started() {
    //     SpringContextUtils.getApplicationContext().publishEvent(SpringEvent(this as java.lang.Object, "READY", ""));
    // }
    pub fn started(&self) {
        SpringContextUtils::get_application_context().map(|ctx| {
            ctx.publish_event(SpringEvent::new(Object, String::from("READY"), String::from("")));
        });
    }

    // override fun onStartError() {
    //     logger.error("应用启动失败，请检查" + port + "端口是否被占用")
    //     SpringContextUtils.getApplicationContext().publishEvent(SpringEvent(this as java.lang.Object, "START_ERROR", "应用启动失败，请检查" + port + "端口是否被占用"));
    // }
    pub fn on_start_error(&self) {
        logger.error(format!("应用启动失败，请检查{}端口是否被占用", self.port));
        SpringContextUtils::get_application_context().map(|ctx| {
            ctx.publish_event(SpringEvent::new(Object, String::from("START_ERROR"), format!("应用启动失败，请检查{}端口是否被占用", self.port)));
        });
    }

    // override fun onHandlerError(ctx: RoutingContext, error: Exception) {
    //     val returnData = ReturnData()
    //     logger.error("onHandlerError: ", error)
    //     if (!ctx.response().headWritten()) {
    //         ctx.success(returnData.setErrorMsg(error.toString()))
    //     } else {
    //         ctx.response().end(error.toString())
    //     }
    // }
    pub fn on_handler_error(&self, ctx: &RoutingContext, error: &Exception) {
        let mut return_data = ReturnData::new();
        logger.error(format!("onHandlerError: {}", error.to_string()));
        if !ctx.response().head_written() {
            ctx.success(return_data.set_error_msg(error.to_string()));
        } else {
            ctx.response().end(error.to_string());
        }
    }

    // private suspend fun getSystemInfo(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     var systemFont = System.getProperty("reader.system.fonts")
    //     var freeMemory = "" + (Runtime.getRuntime().freeMemory() / 1024 / 1024) + "M"
    //     var totalMemory = "" + (Runtime.getRuntime().totalMemory() / 1024 / 1024) + "M"
    //     var maxMemory = "" + (Runtime.getRuntime().maxMemory() / 1024 / 1024) + "M"
    //     val userController = UserController(coroutineContext)
    //     var dayLoginUser = 0
    //     var sevenDayLoginUser = 0
    //     var monthLoginUser = 0
    //     var keepUser = 0
    //     var dayRegisterUser = 0
    //     var sevenDayRegisterUser = 0
    //     var monthRegisterUser = 0
    //     val calendar = Calendar.getInstance().apply {
    //         set(Calendar.DAY_OF_MONTH, 1)
    //         set(Calendar.HOUR_OF_DAY, 0)
    //         set(Calendar.MINUTE, 0)
    //         set(Calendar.SECOND, 0)
    //         set(Calendar.MILLISECOND, 0)
    //     }
    //     userController.forEachUser { user ->
    //         if (user.last_login_at >= System.currentTimeMillis() - 86400000_i64) dayLoginUser++
    //         if (user.last_login_at >= System.currentTimeMillis() - 604800000_i64) sevenDayLoginUser++
    //         if (user.last_login_at >= calendar.timeInMillis) monthLoginUser++
    //         if (user.created_at >= System.currentTimeMillis() - 86400000_i64) dayRegisterUser++
    //         if (user.created_at >= System.currentTimeMillis() - 604800000_i64) sevenDayRegisterUser++
    //         if (user.created_at >= calendar.timeInMillis) monthRegisterUser++
    //         if (user.last_login_at >= user.created_at + 604800000_i64 &&
    //             user.last_login_at >= System.currentTimeMillis() - 604800000_i64) keepUser++
    //         false
    //     }
    //     return returnData.setData(mapOf(
    //         "fonts" to systemFont,
    //         "freeMemory" to freeMemory,
    //         "totalMemory" to totalMemory,
    //         "maxMemory" to maxMemory,
    //         "dayRegisterUser" to dayRegisterUser,
    //         "dayLoginUser" to dayLoginUser,
    //         "sevenDayRegisterUser" to sevenDayRegisterUser,
    //         "sevenDayLoginUser" to sevenDayLoginUser,
    //         "monthRegisterUser" to monthRegisterUser,
    //         "monthLoginUser" to monthLoginUser,
    //         "keepUser" to keepUser
    //     ))
    // }
    fn get_system_info(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        let system_font = System::get_property("reader.system.fonts");
        let free_memory = format!("{}M", Runtime::free_memory() / 1024 / 1024);
        let total_memory = format!("{}M", Runtime::total_memory() / 1024 / 1024);
        let max_memory = format!("{}M", Runtime::max_memory() / 1024 / 1024);
        let user_controller = UserController::new();
        let mut day_login_user = 0;
        let mut seven_day_login_user = 0;
        let mut month_login_user = 0;
        let mut keep_user = 0;
        let mut day_register_user = 0;
        let mut seven_day_register_user = 0;
        let mut month_register_user = 0;
        let mut calendar = Calendar::get_instance();
        calendar.set(Calendar::DAY_OF_MONTH, 1);
        calendar.set(Calendar::HOUR_OF_DAY, 0);
        calendar.set(Calendar::MINUTE, 0);
        calendar.set(Calendar::SECOND, 0);
        calendar.set(Calendar::MILLISECOND, 0);
        user_controller.for_each_user(&mut |user: &mut User| {
            if user.last_login_at >= System::current_time_millis() - 86400000_i64 {
                day_login_user += 1;
            }
            if user.last_login_at >= System::current_time_millis() - 604800000_i64 {
                seven_day_login_user += 1;
            }
            if user.last_login_at >= calendar.time_in_millis() {
                month_login_user += 1;
            }
            if user.created_at >= System::current_time_millis() - 86400000_i64 {
                day_register_user += 1;
            }
            if user.created_at >= System::current_time_millis() - 604800000_i64 {
                seven_day_register_user += 1;
            }
            if user.created_at >= calendar.time_in_millis() {
                month_register_user += 1;
            }
            if user.last_login_at >= user.created_at + 604800000_i64
                && user.last_login_at >= System::current_time_millis() - 604800000_i64 {
                keep_user += 1;
            }
            false
        });
        let mut data: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
        data.insert(String::from("fonts"), Box::new(system_font));
        data.insert(String::from("freeMemory"), Box::new(free_memory));
        data.insert(String::from("totalMemory"), Box::new(total_memory));
        data.insert(String::from("maxMemory"), Box::new(max_memory));
        data.insert(String::from("dayRegisterUser"), Box::new(day_register_user));
        data.insert(String::from("dayLoginUser"), Box::new(day_login_user));
        data.insert(String::from("sevenDayRegisterUser"), Box::new(seven_day_register_user));
        data.insert(String::from("sevenDayLoginUser"), Box::new(seven_day_login_user));
        data.insert(String::from("monthRegisterUser"), Box::new(month_register_user));
        data.insert(String::from("monthLoginUser"), Box::new(month_login_user));
        data.insert(String::from("keepUser"), Box::new(keep_user));
        return_data.set_data(Box::new(data), String::from(""));
        return return_data;
    }

    /**
     * 定时任务
     */

    /**
     * 定期检查书架书籍更新 (使用配置的间隔时间)
     */
    // @Scheduled(cron = "0 0/10 * * * ?")
    pub fn shelf_update_job(&self) {
        if self.app_config.shelf_update_inteval <= 0 {
            return;
        }
        let now = Calendar::get_instance();
        let minute_from_today = now.get(Calendar::HOUR_OF_DAY) * 60 + now.get(Calendar::MINUTE);
        if minute_from_today % self.app_config.shelf_update_inteval != 0 {
            return;
        }
        MDC::put("traceId", get_trace_id());
        // launch(MDCContext() + Dispatchers.IO) {
        let result = std::panic::catch_unwind(|| {
            let book_controller = BookController::new();

            logger.info("开始检查书架书籍更新");
            // 刷新系统默认书架
            book_controller.get_book_shelf_books(true, "default");

            // 刷新用户书架
            let user_controller = UserController::new();
            user_controller.for_each_user(&mut |user: &mut User| {
                if user.last_login_at >= System::current_time_millis() - 259200000_i64 {
                    book_controller.get_book_shelf_books(true, user.username.clone());
                }
                false
            });
            logger.info("书架书籍更新检查结束");
        });
        if let Err(e) = result {
            eprintln!("{:?}", e);
        }
        // }
    }

    /**
     * 每天清理不活跃用户
     */
    // @Scheduled(cron = "0 59 23 * * ?")
    pub fn clear_user(&self) {
        if self.app_config.auto_clear_inactive_user <= 0 || !self.app_config.secure {
            return;
        }
        MDC::put("traceId", get_trace_id());
        // launch(MDCContext() + Dispatchers.IO) {
        let result = std::panic::catch_unwind(|| {
            logger.info(format!("开始清理 {} 天未登录用户", self.app_config.auto_clear_inactive_user));
            UserController::new().clear_inactive_users(self.app_config.auto_clear_inactive_user);
            logger.info("不活跃用户自动清理结束");
        });
        if let Err(e) = result {
            eprintln!("{:?}", e);
        }
        // }
    }

    /**
     * 自动备份用户数据 (每天凌晨2点)
     */
    // @Scheduled(cron = "0 50 23 * * ?")
    pub fn auto_backup(&self) {
        if !self.app_config.auto_backup_user_data {
            return;
        }
        MDC::put("traceId", get_trace_id());
        // launch(MDCContext() + Dispatchers.IO) {
        let result = std::panic::catch_unwind(|| {
            logger.info("开始备份用户数据");
            let book_controller = BookController::new();

            // 备份默认用户
            book_controller.save_to_webdav(&String::from("default"), None);

            // 备份其他用户
            let user_controller = UserController::new();
            user_controller.for_each_user(&mut |user: &mut User| {
                if user.last_login_at >= System::current_time_millis() - 259200000_i64 {
                    book_controller.save_to_webdav(&user.username, None);
                }
                false
            });
            logger.info("备份用户数据结束");
        });
        if let Err(e) = result {
            eprintln!("{:?}", e);
        }
        // }
    }

    /**
     * 定期执行垃圾回收
     */
    // @Scheduled(cron = "0 0 2 * * ?")
    pub fn auto_gc(&self) {
        System::gc();
    }

    /**
     * 远程书源订阅更新
     */
    // @Scheduled(cron = "0 0/10 * * * ?")
    pub fn remote_book_source_sub_update_job(&self) {
        if self.app_config.remote_book_source_update_interval <= 0 {
            return;
        }
        let now = Calendar::get_instance();
        let minute_from_today = now.get(Calendar::HOUR_OF_DAY) * 60 + now.get(Calendar::MINUTE);
        if minute_from_today % self.app_config.remote_book_source_update_interval != 0 {
            return;
        }
        MDC::put("traceId", get_trace_id());
        // launch(MDCContext() + Dispatchers.IO) {
        let result = std::panic::catch_unwind(|| {
            logger.info("开始检查远程书源更新");
            let book_source_controller = BookSourceController::new();
            // Update for default namespace
            book_source_controller.update_remote_source_sub(String::from("default"), None);

            // Update for all users
            let user_controller = UserController::new();
            user_controller.for_each_user(&mut |user: &mut User| {
                if user.last_login_at >= System::current_time_millis() - 259200000_i64 {
                    book_source_controller.update_remote_source_sub(user.username.clone(), None);
                }
                false
            });
            logger.info("远程书源更新检查结束");
        });
        if let Err(e) = result {
            eprintln!("{:?}", e);
        }
        // }
    }
}
