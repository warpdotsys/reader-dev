//! 路由：/health + /reader3/*（兼容 legacy API）

use std::collections::HashMap;

use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use futures::StreamExt;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::model::User;
use crate::storage::Storage;

/// 统一返回结构（兼容 legacy ReturnData：isSuccess/errorMsg/data——camelCase）
#[derive(Debug, serde::Serialize)]
pub struct ReturnData {
    #[serde(rename = "isSuccess")]
    pub is_success: bool,
    #[serde(rename = "errorMsg")]
    pub error_msg: String,
    pub data: Value,
}

impl ReturnData {
    pub fn ok(data: Value) -> Self {
        Self {
            is_success: true,
            error_msg: String::new(),
            data,
        }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            is_success: false,
            error_msg: msg.into(),
            data: Value::Null,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    /// 图片代理磁盘缓存（GAP：storage/cache/images，LRU 容量控制 + 并发去重）
    pub image_cache: std::sync::Arc<crate::service::image_cache::ImageCache>,
}

/// F-10：目录缓存 TTL（5 分钟）
const TOC_CACHE_TTL_MS: i64 = 5 * 60 * 1000;

/// 构建路由
pub fn router(config: crate::AppConfig, storage: Storage) -> axum::Router {
    // 书源 cookie 存取注册（crawler 抓取/登录按用户命名空间读表）
    crate::service::crawler::register_cookie_storage(storage.clone());
    // 图片代理磁盘缓存（GAP：storage/cache/images；容量 env READER_IMAGE_CACHE_MB 默认 512MB）
    let image_cache = crate::service::image_cache::ImageCache::new(&config);
    let state = AppState {
        storage,
        image_cache,
    };

    // /assets 静态资源（封面等：storage/assets/**，legacy 兼容）
    let assets_dir = config.storage_dir().join("assets");
    let assets_service = tower_http::services::ServeDir::new(assets_dir);

    // Kindle 轻量页（web-simple/，/simple/* 与 legacy 别名 /simple-web/*——独立于
    // web-ui SPA，无 fallback，目录可经 READER_APP_SIMPLE_WEB_ROOT 覆盖，
    // 默认相对进程工作目录的 web-simple/）
    let simple_dir =
        std::env::var("READER_APP_SIMPLE_WEB_ROOT").unwrap_or_else(|_| "web-simple".to_string());
    let simple_service = tower_http::services::ServeDir::new(&simple_dir);

    // GAP 62：multipart 上传上限（env READER_UPLOAD_MAX_MB，默认 100MB；超限 → 413 →
    // 最外层 UploadLimitLayer 替换为明确 JSON 错误）。所有上传路由统一走此配置。
    let upload_limit = config.upload_max_bytes();

    axum::Router::new()
        .nest_service("/assets", assets_service)
        // GAP #88/125：封面/正文图片防盗链代理（精确路由优先于 /assets 静态目录）
        .route("/assets/proxy", get(assets_proxy))
        // legacy 静态路由：/book-assets/* 与 /epub/*（YueduApi.kt:136-162——均以
        // storage/data/ 为 Web 根：EPUB 解压资源/章节 HTML 直读；HTML 响应在 </body>
        // 前注入 __API_ROOT__ 脚本，见 serve_data_file）
        .route("/book-assets/*rest", get(book_assets))
        .route("/epub/*rest", get(epub_asset))
        .route("/health", get(health))
        // Kindle 轻量页（/simple/*：web-simple/ 纯静态——目录请求自动 index.html；
        // /simple-web/* 为 legacy 路由别名，同一 handler）
        .nest_service("/simple", simple_service.clone())
        .nest_service("/simple-web", simple_service)
        // 弱网优化：响应压缩（gzip/brotli）
        .layer(tower_http::compression::CompressionLayer::new())
        .route("/opds", get(opds_dispatch))
        .route("/opds-save", post(opds_save_post).get(opds_save_post))
        .route("/opds/*rest", get(opds_dispatch))
        // OPDS 独立账号设置（secure 模式外亦可配置，作用于 OPDS Basic 认证）
        .route(
            "/reader3/getOpdsSettings",
            get(get_opds_settings).post(get_opds_settings),
        )
        .route("/reader3/saveOpdsSettings", post(save_opds_settings))
        .route(
            "/reader3/uploadLocalBook",
            post(upload_local_book).layer(axum::extract::DefaultBodyLimit::max(upload_limit)),
        )
        // F-4 远程书源订阅导入
        .route(
            "/reader3/saveFromRemoteSource",
            post(save_from_remote_source),
        )
        // F-13 书架单书
        .route(
            "/reader3/getShelfBook",
            get(get_shelf_book).post(get_shelf_book),
        )
        // F-25 退出登录
        .route("/reader3/logout", post(logout))
        // F-34 不活跃用户清理（secure + secureKey）
        .route("/reader3/clearInactiveUsers", post(clear_inactive_users))
        // F-32 用户管理（secure + secureKey）
        .route("/reader3/getUsers", get(get_users).post(get_users))
        .route("/reader3/addUser", post(add_user))
        .route("/reader3/updateUser", post(update_user))
        .route("/reader3/deleteUser", post(delete_user))
        .route("/reader3/deleteUsers", post(delete_users))
        .route("/reader3/resetUserPassword", post(reset_user_password))
        // F-25 TTS：Edge 语音 + HttpTTS + 语音列表
        .route(
            "/reader3/getTTSVoices",
            get(get_tts_voices).post(get_tts_voices),
        )
        .route("/reader3/tts", get(tts_synthesize).post(tts_synthesize))
        // legacy 听书主入口路径别名（YueduApi.kt:374-375）
        .route(
            "/reader3/book/tts",
            get(tts_synthesize).post(tts_synthesize),
        )
        // F-39 手动备份到 WebDAV（书架数据 zip）
        .route(
            "/reader3/user/downloadBackupFile",
            get(download_backup_file),
        )
        .route("/reader3/backupToWebdav", post(backup_to_webdav))
        // MongoDB 备份/恢复（legacy 接口；uri 可走 body 或 READER_MONGODB_URI）
        .route("/reader3/backupToMongodb", post(backup_to_mongodb))
        .route("/reader3/restoreFromMongodb", post(restore_from_mongodb))
        // F-55 备份恢复（zip 上传 / webdav 目录内 zip）
        .route(
            "/reader3/restoreFromZip",
            post(restore_from_zip).layer(axum::extract::DefaultBodyLimit::max(upload_limit)),
        )
        .route("/reader3/restoreFromWebdav", post(restore_from_webdav))
        // F-38 文件管理（home 语义对齐 legacy FileController）
        .route("/reader3/file/list", get(crate::api::files::list))
        // legacy file/parse：目录扫描书籍导入（GET+POST，P0 路由补齐）
        .route(
            "/reader3/file/parse",
            get(crate::api::files::parse).post(crate::api::files::parse),
        )
        .route("/reader3/file/get", get(crate::api::files::get))
        .route("/reader3/file/save", post(crate::api::files::save))
        .route("/reader3/file/mkdir", post(crate::api::files::mkdir))
        .route("/reader3/file/rename", post(crate::api::files::rename))
        .route("/reader3/file/download", get(crate::api::files::download))
        .route(
            "/reader3/file/upload",
            post(crate::api::files::upload)
                .layer(axum::extract::DefaultBodyLimit::max(upload_limit)),
        )
        .route("/reader3/file/delete", post(crate::api::files::delete))
        .route(
            "/reader3/file/deleteMulti",
            post(crate::api::files::delete_multi),
        )
        .route("/reader3/deleteBook", post(delete_book))
        .route("/reader3/saveBook", post(save_book))
        .route("/reader3/saveBookProgress", post(save_book_progress))
        .route(
            "/reader3/getExploreSources",
            get(get_explore_sources).post(get_explore_sources),
        )
        .route(
            "/reader3/getExploreUrls",
            get(get_explore_urls).post(get_explore_urls),
        )
        .route("/reader3/exploreBook", get(explore_book).post(explore_book))
        .route(
            "/reader3/searchBookMultiSSE",
            get(search_book_multi_sse).post(search_book_multi_sse),
        )
        .route("/reader3/saveBookmark", post(save_bookmark))
        .route(
            "/reader3/getBookmarks",
            get(get_bookmarks).post(get_bookmarks),
        )
        .route("/reader3/deleteBookmark", post(delete_bookmark))
        .route(
            "/reader3/getBookGroups",
            get(get_book_groups).post(get_book_groups),
        )
        .route("/reader3/saveBookGroup", post(save_book_group))
        .route("/reader3/updateBookGroupId", post(update_book_group_id))
        .route("/reader3/setBookGroups", post(set_book_groups))
        .route("/reader3/addBookGroup", post(add_book_group))
        .route("/reader3/removeBookGroup", post(remove_book_group))
        .route("/reader3/deleteBookGroup", post(delete_book_group))
        // 命名兼容批（legacy 别名路由——外部客户端兼容）
        .route(
            "/reader3/getChapterList",
            get(get_book_toc).post(get_book_toc),
        )
        .route(
            "/reader3/getRssContent",
            get(get_rss_article).post(get_rss_article),
        )
        .route("/reader3/getUserList", get(get_users).post(get_users))
        .route(
            "/reader3/getBookGroupList",
            get(get_book_groups).post(get_book_groups),
        )
        .route("/reader3/saveBookGroupName", post(save_book_group))
        .route("/reader3/updateBookGroup", post(save_book_group))
        // F-28 替换规则
        .route(
            "/reader3/getReplaceRules",
            get(get_replace_rules).post(get_replace_rules),
        )
        .route("/reader3/saveReplaceRule", post(save_replace_rule))
        .route("/reader3/saveReplaceRules", post(save_replace_rules))
        .route(
            "/reader3/replaceRule/saveMulti",
            post(save_replace_rule_multi),
        )
        .route("/reader3/deleteReplaceRule", post(delete_replace_rule))
        .route("/reader3/deleteReplaceRules", post(delete_replace_rules))
        // F-26 HttpTTS 听书源管理
        .route(
            "/reader3/getHttpTTSList",
            get(get_http_tts_list).post(get_http_tts_list),
        )
        .route("/reader3/saveHttpTTS", post(save_http_tts))
        .route("/reader3/httpTTS/saveMulti", post(save_http_tts_multi))
        .route("/reader3/deleteHttpTTS", post(delete_http_tts))
        .route("/reader3/deleteHttpTTSs", post(delete_http_tts_multi))
        // legacy httpTTS/* 路径别名（YueduApi.kt:407-411——旧客户端主路径）
        .route(
            "/reader3/httpTTS/list",
            get(get_http_tts_list).post(get_http_tts_list),
        )
        .route("/reader3/httpTTS/save", post(save_http_tts))
        .route("/reader3/httpTTS/delete", post(delete_http_tts))
        .route("/reader3/httpTTS/deleteMulti", post(delete_http_tts_multi))
        // 自定义 TXT 目录规则（对齐 legado TxtTocRule）
        .route(
            "/reader3/getTxtTocRules",
            get(get_txt_toc_rules).post(get_txt_toc_rules),
        )
        .route("/reader3/saveTxtTocRule", post(save_txt_toc_rule))
        .route("/reader3/deleteTxtTocRule", post(delete_txt_toc_rule))
        .route(
            "/reader3/importDefaultTxtTocRules",
            post(import_default_txt_toc_rules),
        )
        // 系统信息 + 书源导出
        .route("/reader3/getSystemInfo", get(get_system_info))
        .route("/reader3/getServerStats", get(get_server_stats))
        .route("/reader3/exportBookSources", get(export_book_sources))
        // SPA fallback：未匹配路由 → webdav 分流 / API 404 / 前端
        .fallback(fallback_handler)
        .route("/reader3/getBookshelf", get(get_bookshelf))
        .route(
            "/reader3/getBookSources",
            get(get_book_sources).post(get_book_sources),
        )
        .route(
            "/reader3/getBookSource",
            get(get_book_source).post(get_book_source),
        )
        .route("/reader3/saveBookSource", post(save_book_source))
        .route("/reader3/saveBookSources", post(save_book_sources))
        .route("/reader3/deleteBookSource", post(delete_book_source))
        .route("/reader3/deleteBookSources", post(delete_book_sources))
        .route(
            "/reader3/deleteAllBookSources",
            post(delete_all_book_sources),
        )
        // 书源登录态（cookie 按用户隔离）
        .route(
            "/reader3/loginBookSource",
            get(login_book_source).post(login_book_source),
        )
        .route("/reader3/setBookSourceCookie", post(set_book_source_cookie))
        .route(
            "/reader3/getBookSourceCookie",
            get(get_book_source_cookie).post(get_book_source_cookie),
        )
        .route("/reader3/getCaptcha", post(get_captcha))
        .route("/reader3/submitCaptcha", post(submit_captcha))
        // 缓存管理 + 全书搜索 + 书源订阅
        .route(
            "/reader3/getCacheInfo",
            get(get_cache_info).post(get_cache_info),
        )
        .route("/reader3/clearCache", post(clear_cache))
        .route(
            "/reader3/searchBookContent",
            get(search_book_content).post(search_book_content),
        )
        .route(
            "/reader3/getSourceSubs",
            get(get_source_subs).post(get_source_subs),
        )
        .route("/reader3/saveSourceSub", post(save_source_sub))
        .route("/reader3/previewSourceSub", post(preview_source_sub))
        .route("/reader3/deleteSourceSub", post(delete_source_sub))
        .route("/reader3/deleteSourceSubs", post(delete_source_subs))
        .route("/reader3/refreshSourceSub", post(refresh_source_sub))
        .route("/reader3/setSourceSubEnabled", post(set_source_sub_enabled))
        // RSS 模块（兼容 legacy rss 路由）
        .route(
            "/reader3/getRssSources",
            get(get_rss_sources).post(get_rss_sources),
        )
        .route("/reader3/saveRssSource", post(save_rss_source))
        .route("/reader3/deleteRssSource", post(delete_rss_source))
        .route(
            "/reader3/getRssArticles",
            get(get_rss_articles).post(get_rss_articles),
        )
        .route("/reader3/markRssArticleRead", post(mark_rss_article_read))
        .route(
            "/reader3/getRssArticle",
            get(get_rss_article).post(get_rss_article),
        )
        .route("/reader3/searchBook", get(search_book).post(search_book))
        .route(
            "/reader3/searchBookMulti",
            get(search_book_multi).post(search_book_multi),
        )
        // 换源搜索：同书其他书源列表（url + bookSource）
        .route(
            "/reader3/searchBookSource",
            get(search_book_source).post(search_book_source),
        )
        // 换源持久化（legacy setBookSource 对齐）
        .route(
            "/reader3/setBookSource",
            get(set_book_source).post(set_book_source),
        )
        // legacy 兼容补齐：当前用户信息 / 封面代理 / 用户资产删除
        .route("/reader3/getUserInfo", get(get_user_info))
        .route("/reader3/cover", get(book_cover_legacy))
        .route("/reader3/deleteFile", get(delete_file).post(delete_file))
        // 文件模型遗留端点：删除用户书源文件=清空用户书源回退 default（SQLite 模型下等价语义）
        .route(
            "/reader3/deleteBookSourcesFile",
            post(delete_all_book_sources),
        )
        .route(
            "/reader3/getBookInfo",
            get(get_book_info).post(get_book_info),
        )
        .route("/reader3/getBookToc", get(get_book_toc).post(get_book_toc))
        .route(
            "/reader3/getBookContent",
            get(get_book_content).post(get_book_content),
        )
        // 差距补全批：多格式导出 / 书源调试 / 整书缓存 / 用户配置 / 本地书刷新 / 批量接口 / 书源健康 / 阅读统计
        .route("/reader3/exportBook", get(export_book).post(export_book))
        // legacy 书籍级阅读配置持久化（YueduApi.kt:371）
        .route("/reader3/book/saveBookConfig", post(save_book_config))
        .route(
            "/reader3/bookSourceDebugSSE",
            get(book_source_debug_sse).post(book_source_debug_sse),
        )
        .route("/reader3/cacheBookOnServer", post(cache_book_on_server))
        .route(
            "/reader3/getAllContents",
            get(crate::api::pro_export::get_all_contents)
                .post(crate::api::pro_export::get_all_contents),
        )
        .route(
            "/reader3/searchChapter",
            get(crate::api::pro_export::search_chapter)
                .post(crate::api::pro_export::search_chapter),
        )
        .route(
            "/reader3/exportToTxt",
            get(crate::api::pro_export::export_to_txt).post(crate::api::pro_export::export_to_txt),
        )
        .route(
            "/reader3/exportToEpub",
            get(crate::api::pro_export::export_to_epub)
                .post(crate::api::pro_export::export_to_epub),
        )
        .route(
            "/reader3/cacheBookRangeOnServer",
            post(cache_book_range_on_server),
        )
        .route(
            "/reader3/getBookCacheChapters",
            get(get_book_cache_chapters).post(get_book_cache_chapters),
        )
        .route(
            "/reader3/cacheBookSSE",
            get(cache_book_sse).post(cache_book_sse),
        )
        .route(
            "/reader3/cancelCacheBook",
            get(cancel_cache_book).post(cancel_cache_book),
        )
        .route(
            "/reader3/getUserConfig",
            get(get_user_config).post(get_user_config),
        )
        .route("/reader3/saveUserConfig", post(save_user_config))
        .route("/reader3/refreshLocalBook", post(refresh_local_book))
        .route("/reader3/scanLocalBookDir", post(scan_local_book_dir))
        .route("/reader3/deleteBooks", post(delete_books))
        .route("/reader3/deleteBookmarks", post(delete_bookmarks))
        .route("/reader3/saveRssSources", post(save_rss_sources))
        .route("/reader3/saveBookmarks", post(save_bookmarks))
        .route("/reader3/addBookGroupMulti", post(add_book_group_multi))
        .route(
            "/reader3/removeBookGroupMulti",
            post(remove_book_group_multi),
        )
        .route("/reader3/saveBookGroupOrder", post(save_book_group_order))
        .route(
            "/reader3/getAvailableBookSource",
            get(get_available_book_source).post(get_available_book_source),
        )
        .route(
            "/reader3/getInvalidBookSources",
            get(get_invalid_book_sources).post(get_invalid_book_sources),
        )
        // GAP 140：一键禁用失效书源（复用失效检测 → 批量 enabled=0）
        .route(
            "/reader3/disableInvalidBookSources",
            post(disable_invalid_book_sources),
        )
        // GAP 171：legacy loc_book 文件书 → DB 迁移（local_file 关联，保留原记录）
        .route("/reader3/migrateLocBook", post(migrate_loc_book))
        .route(
            "/reader3/setAsDefaultBookSources",
            post(set_as_default_book_sources),
        )
        .route(
            "/reader3/searchBookSourceSSE",
            get(search_book_source_sse).post(search_book_source_sse),
        )
        .route(
            "/reader3/getReadingStats",
            get(get_reading_stats).post(get_reading_stats),
        )
        // 小项补全批：单书缓存删除 / 书架缓存信息 / 导入预览 / 书源文件读取 / 正文缓存写回 /
        // 用户书源删除 / 分组别名 / 目录规则单页调试
        .route(
            "/reader3/deleteBookCache",
            get(delete_book_cache).post(delete_book_cache),
        )
        .route(
            "/reader3/getShelfBookWithCacheInfo",
            get(get_shelf_book_with_cache_info).post(get_shelf_book_with_cache_info),
        )
        .route(
            "/reader3/importBookPreview",
            post(import_book_preview).layer(axum::extract::DefaultBodyLimit::max(upload_limit)),
        )
        .route("/reader3/readSourceFile", post(read_source_file))
        .route("/reader3/saveBookContent", post(save_book_content))
        .route(
            "/reader3/deleteUserBookSource",
            post(delete_user_book_source),
        )
        .route("/reader3/saveBookGroupId", post(save_book_group_id))
        .route(
            "/reader3/getChapterListByRule",
            get(get_chapter_list_by_rule).post(get_chapter_list_by_rule),
        )
        // 命名兼容批 2（legacy 别名路由——外部客户端兼容）
        .route("/reader3/resetPassword", post(reset_user_password))
        .route("/reader3/httpTTS", get(tts_synthesize).post(tts_synthesize))
        // legacy uploadFile（UserController.uploadFile）：assets/{ns}/{type}/ 上传 →
        // URL 数组（与 file/upload 书仓上传语义同名异义，独立 handler）
        .route(
            "/reader3/uploadFile",
            post(upload_user_file).layer(axum::extract::DefaultBodyLimit::max(upload_limit)),
        )
        .route("/reader3/login", post(login))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok!"
}

/// P1-C6：上游 Content-Type 合法性校验——仅接受 RFC 7230 token 字符集的
/// `type/subtype[; param=value]`（拒绝控制字符/空白/CRLF 头注入）；非法回退默认 image/*。
/// 返回值为纯 ASCII token 字符，可安全用作响应头。
fn sanitize_proxy_content_type(content_type: Option<&str>) -> String {
    const DEFAULT_CT: &str = "image/png";
    let is_token = |x: &str| -> bool {
        !x.is_empty()
            && x.bytes().all(|b| {
                b.is_ascii_alphanumeric()
                    || matches!(
                        b,
                        b'!' | b'#'
                            | b'$'
                            | b'%'
                            | b'&'
                            | b'\''
                            | b'*'
                            | b'+'
                            | b'-'
                            | b'.'
                            | b'^'
                            | b'_'
                            | b'`'
                            | b'|'
                            | b'~'
                    )
            })
    };
    let Some(ct) = content_type else {
        return DEFAULT_CT.to_string();
    };
    let ct = ct.trim();
    // 拒绝头注入/硬控制字符（CR/LF/NUL/DEL）；空白（含参数区 OWS）由结构校验处理
    if ct.is_empty() || ct.bytes().any(|b| matches!(b, 0x0A | 0x0D | 0x00 | 0x7F)) {
        return DEFAULT_CT.to_string();
    }
    let mut segs = ct.split(';');
    let media = segs.next().unwrap_or("").trim();
    let mut parts = media.splitn(2, '/');
    let (t, s) = (parts.next(), parts.next());
    if t.map(is_token).unwrap_or(false) && s.map(is_token).unwrap_or(false) {
        // 参数段：token=token 或 token="quoted-token"
        let params_ok = segs.all(|p| {
            let p = p.trim();
            let mut kv = p.splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some(k), Some(v)) => is_token(k) && is_token(v.trim_matches('"')),
                _ => false,
            }
        });
        if params_ok {
            return ct.to_string();
        }
    }
    DEFAULT_CT.to_string()
}

/// GAP #88/125：GET /assets/proxy?url=&referer=（封面/正文图片防盗链代理）
/// GAP #88/125：GET /assets/proxy?url=&referer=（封面/正文图片防盗链代理）
///
/// 服务端拉取图片：自动附加书源 header（书源登录 cookie/UA 按用户命名空间 + Referer）；
/// 超时 10s；大小上限 5MB（Content-Length 预检 + 流式累计兜底）；Content-Type 透传。
/// GAP 130：?fmt=webp&q=80 → 转码 webp 输出（image 编解码，失败回退原图透传）。
/// GAP：磁盘缓存（storage/cache/images，LRU 容量上限 env READER_IMAGE_CACHE_MB 默认 512MB）——
/// 命中直接读盘（Cache-Control 长缓存 public, max-age=31536000, immutable），未命中回源后写盘；
/// 同 URL 并发请求共享一次回源（内存 in-flight map）。
/// secure 模式下按 accessToken 解析用户命名空间（与 /reader3 一致）。
async fn assets_proxy(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let url = params.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return Json(ReturnData::err("参数错误")).into_response();
    }
    // 仅允许 http/https（控制 SSRF 面）
    let parsed = match url::Url::parse(&url) {
        Ok(p) if matches!(p.scheme(), "http" | "https") => p,
        _ => return Json(ReturnData::err("参数错误")).into_response(),
    };
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return (StatusCode::UNAUTHORIZED, Json(ret)).into_response(),
    };
    let referer = params.get("referer").cloned();
    // GAP 130：webp 转换参数（fmt=webp 且 q=质量 1-100，默认 80）
    let to_webp = params
        .get("fmt")
        .map(|v| v.eq_ignore_ascii_case("webp"))
        .unwrap_or(false);
    let quality: u8 = params.get("q").and_then(|v| v.parse().ok()).unwrap_or(80);
    match state
        .image_cache
        .get_or_fetch(
            &namespace,
            parsed.as_str(),
            referer.as_deref(),
            10,
            5 * 1024 * 1024,
        )
        .await
    {
        Ok((bytes, content_type, status, from_cache)) => {
            // 磁盘命中 → 长缓存（内容按 URL 定址；上游图片变更依赖 LRU 淘汰换新）
            let cache_control = if from_cache {
                "public, max-age=31536000, immutable"
            } else {
                "public, max-age=3600"
            };
            let is_raster = content_type
                .as_deref()
                .map(|ct| ct.starts_with("image/"))
                .unwrap_or(false);
            let converted = if to_webp && is_raster {
                crate::service::imaging::to_webp(&bytes, quality)
            } else {
                None
            };
            match converted {
                Some(webp) => match Response::builder()
                    .status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK))
                    .header("Content-Type", "image/webp")
                    .header("Cache-Control", cache_control)
                    .body(Body::from(webp))
                {
                    Ok(resp) => resp,
                    Err(e) => {
                        tracing::error!("图片代理响应构造失败: {e}");
                        Json(ReturnData::err("系统错误")).into_response()
                    }
                },
                None => {
                    // P1-C6：上游 Content-Type 校验（非法回退默认 image/*）——防头注入/任意类型透传
                    let content_type = sanitize_proxy_content_type(content_type.as_deref());
                    match Response::builder()
                        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK))
                        .header("Content-Type", content_type)
                        .header("Cache-Control", cache_control)
                        .body(Body::from(bytes))
                    {
                        Ok(resp) => resp,
                        Err(e) => {
                            tracing::error!("图片代理响应构造失败: {e}");
                            Json(ReturnData::err("系统错误")).into_response()
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("图片代理失败 [{url}]: {e}");
            Json(ReturnData::err(format!("图片加载失败：{e}"))).into_response()
        }
    }
}

/// POST /reader3/login 请求体（兼容 legacy：username/password/isLogin/code）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    username: Option<String>,
    password: Option<String>,
    is_login: Option<bool>,
    code: Option<String>,
}

/// POST /reader3/login：注册或登录，返回 formatUser（camelCase）
/// GAP 61 登录限流（用户名+IP 失败 5 次锁 5 分钟）+ GAP 59 多设备 token
/// M3：限流键默认用直连 socket 对端 IP（axum ConnectInfo，不可伪造）；
/// P1-2：仅当直连 IP 命中 READER_TRUSTED_PROXIES 白名单时才信任 X-Forwarded-For
async fn login(
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginBody>,
) -> Json<ReturnData> {
    let username = body.username.clone().unwrap_or_default();
    let password = body.password.clone().unwrap_or_default();
    let is_login = body.is_login.unwrap_or(false);

    if username.is_empty() {
        return Json(ReturnData::err("请输入用户名"));
    }
    if password.is_empty() {
        return Json(ReturnData::err("请输入密码"));
    }

    // GAP 61：登录限流（用户名+客户端 IP；锁定中直接拒绝）——
    // P1-2：客户端 IP 默认取直连 IP（XFF 可伪造），仅可信代理白名单内才信 XFF
    let ip = client_ip(&peer, &headers);
    if let Err(msg) = crate::util::login_limit::check_allowed(&username, &ip) {
        return Json(ReturnData::err(msg));
    }

    let user = match state.storage.find_user(&username).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("查询用户 {username} 失败: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };

    let Some(mut user) = user else {
        // 用户不存在
        if is_login {
            crate::util::login_limit::record_failure(&username, &ip);
            return Json(ReturnData::err("用户不存在"));
        }
        return register(&state, &username, &password, body.code.clone()).await;
    };

    // 用户已存在
    if !is_login {
        return Json(ReturnData::err("用户名已被占用"));
    }
    // 统一密码校验：argon2id（PHC）优先，legacy 双 MD5 兼容；MD5 通过时自动升级为 argon2id
    if !crate::util::password::verify_password(&state.storage, &user, &password).await {
        crate::util::login_limit::record_failure(&username, &ip);
        return Json(ReturnData::err("密码错误"));
    }
    crate::util::login_limit::reset(&username, &ip); // GAP 59：生成新 token 并追加到 token_map（多设备会话，上限 5；uuid v4 随机防预测）
    let now = now_millis();
    let token = uuid::Uuid::new_v4().simple().to_string();
    if let Err(e) = state.storage.add_user_token(&username, &token, now).await {
        tracing::error!("更新用户 {username} 会话失败: {e}");
        return Json(ReturnData::err("系统错误"));
    }
    user.token = token;
    user.last_login_at = now;
    tracing::info!("用户登录: {username}");
    Json(ReturnData::ok(format_user(&user)))
}

/// 自动注册（校验顺序与错误消息兼容 legacy）
async fn register(
    state: &AppState,
    username: &str,
    password: &str,
    code: Option<String>,
) -> Json<ReturnData> {
    let config = &state.storage.config;

    if username.len() < 5 {
        return Json(ReturnData::err("用户名不能低于5位"));
    }
    if (password.len() as i64) < config.min_user_password_length {
        return Json(ReturnData::err(format!(
            "密码不能低于{}位",
            config.min_user_password_length
        )));
    }
    if username == "default" {
        return Json(ReturnData::err("用户名不能为非法字符"));
    }
    let username_re = Regex::new("^[a-zA-Z0-9]+$").expect("static regex");
    if !username_re.is_match(username) {
        return Json(ReturnData::err("用户名只能由字母和数字组成"));
    }

    // 邀请码校验（配置了才要求）
    if !config.invite_code.is_empty() {
        let code = code.unwrap_or_default();
        if code.is_empty() {
            return Json(ReturnData::err("请输入邀请码"));
        }
        if code != config.invite_code {
            return Json(ReturnData::err("邀请码错误"));
        }
    }

    // 用户数上限（兼容 legacy：max(userLimit, 1)）
    let count = match state.storage.count_users().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("统计用户数失败: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    let user_limit = config.user_limit.max(1);
    if count >= user_limit {
        return Json(ReturnData::err("超过用户数上限"));
    }

    // 创建用户：salt = 8 位随机，默认权限取自 env（READER_APP_DEFAULTUSER*）
    use rand::Rng;
    let salt: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let now = now_millis();
    // P1-7：注册初始 token 与登录一致——uuid v4 随机（原 md5(username+now) 可预测）
    let token = uuid::Uuid::new_v4().simple().to_string();
    let user = User {
        username: username.to_string(),
        // 新用户密码：argon2id PHC（salt 列保留 legacy 8 位随机盐，兼容旧读取路径）
        password: crate::util::password::hash_password(password),
        salt,
        token: token.clone(),
        token_map: None,
        // 首个注册用户自动成为管理员（secure 模式可操作系统 default 配置）
        is_admin: count == 0,
        enable_webdav: config.default_user_enable_webdav,
        enable_local_store: config.default_user_enable_local_store,
        enable_book_source: config.default_user_enable_book_source,
        enable_rss_source: config.default_user_enable_rss_source,
        book_source_limit: config.default_user_book_source_limit,
        book_limit: config.default_user_book_limit,
        last_login_at: now,
        created_at: now,
        user_namespace: username.to_string(),
        raw_json: None,
    };
    if let Err(e) = state.storage.insert_user(&user).await {
        tracing::error!("创建用户 {username} 失败: {e}");
        return Json(ReturnData::err("系统错误"));
    }
    tracing::info!("新用户注册: {username}");
    Json(ReturnData::ok(format_user(&user)))
}

/// POST /reader3/addUser：管理员创建用户（secure + secureKey 校验）。
/// body/query：username/password + 可选权限字段；缺省时按环境默认（未配置则全开）。
async fn add_user(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if let Err(ret) = check_manager_auth(&state, &params, &headers, body_json.as_ref()).await {
        return Json(ret);
    }
    let username = param_of(&params, body_json.as_ref(), "username");
    let password = param_of(&params, body_json.as_ref(), "password");
    if username.is_empty() || password.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let config = &state.storage.config;
    if username.len() < 5 {
        return Json(ReturnData::err("用户名不能低于5位"));
    }
    if (password.len() as i64) < config.min_user_password_length {
        return Json(ReturnData::err(format!(
            "密码不能低于{}位",
            config.min_user_password_length
        )));
    }
    if username == "default" {
        return Json(ReturnData::err("用户名不能为非法字符"));
    }
    let username_re = Regex::new("^[a-zA-Z0-9]+$").expect("static regex");
    if !username_re.is_match(&username) {
        return Json(ReturnData::err("用户名只能由字母和数字组成"));
    }
    if state
        .storage
        .find_user(&username)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return Json(ReturnData::err("用户名已被占用"));
    }
    let count = match state.storage.count_users().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("统计用户数失败: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    if count >= config.user_limit.max(1) {
        return Json(ReturnData::err("超过用户数上限"));
    }
    let bool_param = |key: &str, default: bool| -> bool {
        if let Some(b) = body_json.as_ref().and_then(|b| b.get(key)) {
            return b.as_bool().unwrap_or(default);
        }
        params
            .get(key)
            .map(|v| v == "true" || v == "1")
            .unwrap_or(default)
    };
    let int_param = |key: &str, default: i64| -> i64 {
        if let Some(v) = body_json.as_ref().and_then(|b| b.get(key)) {
            return v.as_i64().unwrap_or(default);
        }
        params
            .get(key)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default)
    };
    use rand::Rng;
    let salt: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let now = now_millis();
    let token = uuid::Uuid::new_v4().simple().to_string();
    let user = User {
        username: username.clone(),
        password: crate::util::password::hash_password(&password),
        salt,
        token: token.clone(),
        token_map: None,
        is_admin: bool_param("isAdmin", false),
        enable_webdav: bool_param("enableWebdav", config.default_user_enable_webdav),
        enable_local_store: bool_param("enableLocalStore", config.default_user_enable_local_store),
        enable_book_source: bool_param("enableBookSource", config.default_user_enable_book_source),
        enable_rss_source: bool_param("enableRssSource", config.default_user_enable_rss_source),
        book_source_limit: int_param("bookSourceLimit", config.default_user_book_source_limit),
        book_limit: int_param("bookLimit", config.default_user_book_limit),
        last_login_at: now,
        created_at: now,
        user_namespace: username.clone(),
        raw_json: None,
    };
    if let Err(e) = state.storage.insert_user(&user).await {
        tracing::error!("addUser 创建用户 {username} 失败: {e}");
        return Json(ReturnData::err("系统错误"));
    }
    tracing::info!("管理员创建用户: {username}");
    Json(ReturnData::ok(format_user(&user)))
}

/// GET /reader3/getBookSources：按命名空间返回书源（legacy 语义：用户无书源回退 default）
/// simple=1 → 仅返回 bookSourceGroup/bookSourceName/bookSourceUrl 且只含 exploreUrl 的书源
async fn get_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let simple = params
        .get("simple")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0)
        > 0;
    match state.storage.get_book_sources(&namespace).await {
        Ok(sources) => {
            let out: Vec<serde_json::Value> = sources
                .into_iter()
                .filter(|s| !simple || s.explore_url.as_deref().is_some_and(|u| !u.is_empty()))
                .map(|s| {
                    if simple {
                        serde_json::json!({
                            "bookSourceGroup": s.book_source_group,
                            "bookSourceName": s.book_source_name,
                            "bookSourceUrl": s.book_source_url,
                        })
                    } else {
                        serde_json::to_value(s).unwrap_or(serde_json::Value::Null)
                    }
                })
                .collect();
            Json(ReturnData::ok(
                serde_json::to_value(out).unwrap_or(serde_json::Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("getBookSources [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST/GET /reader3/getBookSource：单个书源（url 参数）
async fn get_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "bookSourceUrl");
    let url = if url.is_empty() {
        param_of(&params, body_json.as_ref(), "url")
    } else {
        url
    };
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.find_book_source(&namespace, &url).await {
        Ok(Some(s)) => Json(ReturnData::ok(
            serde_json::to_value(s).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Json(ReturnData::err("书源不存在")),
        Err(_) => Json(ReturnData::err("系统错误")),
    }
}

/// GAP 62：multipart 字段读取（带上传上限）——超过上限返回明确错误文案。
/// 说明：axum 的 DefaultBodyLimit 对 Multipart 提取器只表现为流错误（不产生 413），
/// 因此 handler 层必须显式累计字段字节数并给出可读错误；DefaultBodyLimit 仍保留
/// 作为框架级内存保护（配合 UploadLimitLayer 改写 413 响应）。
pub(crate) async fn read_multipart_field_limited(
    field: &mut axum::extract::multipart::Field<'_>,
    max_bytes: usize,
    max_mb: i64,
) -> Result<Vec<u8>, String> {
    let limit_msg = || {
        format!(
            "文件过大：超过上传大小上限（{} MB，可用环境变量 READER_UPLOAD_MAX_MB 调整）",
            max_mb
        )
    };
    let mut out: Vec<u8> = Vec::new();
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                out.extend_from_slice(&chunk);
                if out.len() > max_bytes {
                    return Err(limit_msg());
                }
            }
            Ok(None) => break,
            // P2-17：流错误（如底层 limited body 超限中断）——字段不完整，拒绝而非静默截断
            Err(e) => {
                return Err(format!(
                    "上传被中断（请求体超限或连接异常），字段接收不完整: {e}（可用环境变量 READER_UPLOAD_MAX_MB 调整）"
                ))
            }
        }
    }
    if out.len() > max_bytes {
        return Err(limit_msg());
    }
    Ok(out)
}

/// GAP 62：上传 Content-Length 预检——超限直接返回明确错误。
/// 说明：DefaultBodyLimit 的 Limited 流在 Content-Length 超限时于首次读取即报错，
/// Multipart 提取器表现为泛化流错误（"failed to read stream"，无超限信息）——
/// 因此 handler 层在解析前先按 Content-Length 预检；无 Content-Length 的分块上传
/// 由 read_multipart_field_limited 累计字节数兜底。
pub(crate) fn check_upload_content_length(
    headers: &HeaderMap,
    max_bytes: usize,
    max_mb: i64,
) -> Option<String> {
    let cl = headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())?;
    if cl > max_bytes {
        Some(format!(
            "文件过大：超过上传大小上限（{} MB，可用环境变量 READER_UPLOAD_MAX_MB 调整）",
            max_mb
        ))
    } else {
        None
    }
}

/// F-7 书源数上限校验（saveBookSource / saveBookSources / saveFromRemoteSource 三处收敛，
/// P3-A 抽共享函数）：
/// - limit <= 0 → 不限制（book_source_limit_for 查询失败按不限制处理，同原语义）
/// - 已存在书源不计名额（覆盖更新不占名额）
/// - 返回 Ok(true) = 超限；Ok(false) = 未超限；Err(()) = 统计失败（调用方记日志/报错）
async fn book_source_limit_exceeded(
    storage: &crate::storage::Storage,
    ns: &str,
    candidate_urls: &[&str],
) -> Result<bool, ()> {
    let Some(limit) = storage.book_source_limit_for(ns).await.ok().flatten() else {
        return Ok(false);
    };
    if limit <= 0 {
        return Ok(false);
    }
    let mut new_count = 0i64;
    for url in candidate_urls {
        let exists = storage
            .find_book_source(ns, url)
            .await
            .ok()
            .flatten()
            .is_some();
        if !exists {
            new_count += 1;
        }
    }
    match storage.count_book_sources(ns).await {
        Ok(count) => Ok(count + new_count > limit),
        Err(_) => Err(()),
    }
}

/// POST /reader3/saveBookSource：保存单个书源（body = 完整书源 JSON）
async fn save_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let source: crate::model::BookSource = match serde_json::from_slice(&body) {
        Ok(s) => s,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if source.book_source_url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // F-7 书源数上限（users.book_source_limit；limit<=0 不限制；已存在覆盖不计名额）
    match book_source_limit_exceeded(&state.storage, &namespace, &[&source.book_source_url]).await {
        Ok(true) => return Json(ReturnData::err("超过书源数上限")),
        Ok(false) => {}
        Err(()) => return Json(ReturnData::err("系统错误")),
    }
    match state.storage.save_book_source(&namespace, &source).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveBookSource 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/saveBookSources：批量保存（body = 书源数组）
async fn save_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let sources: Vec<crate::model::BookSource> = match serde_json::from_slice(&body) {
        Ok(s) => s,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if sources.iter().any(|s| s.book_source_url.is_empty()) {
        return Json(ReturnData::err("参数错误"));
    }
    // F-7 书源数上限：逐条统计新增数（已存在覆盖不计名额），超限整批拒绝
    let urls: Vec<&str> = sources.iter().map(|s| s.book_source_url.as_str()).collect();
    match book_source_limit_exceeded(&state.storage, &namespace, &urls).await {
        Ok(true) => return Json(ReturnData::err("超过书源数上限")),
        Ok(false) => {}
        Err(()) => return Json(ReturnData::err("系统错误")),
    }
    match state.storage.save_book_sources(&namespace, &sources).await {
        Ok(_) => Json(ReturnData::ok(
            serde_json::json!({ "count": sources.len() }),
        )),
        Err(e) => {
            tracing::error!("saveBookSources 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// F-4：POST /reader3/saveFromRemoteSource：远程书源订阅导入
/// body/query {url} → 抓取 JSON → 校验书源数组 → save_book_sources 批量入库（已存在覆盖）
async fn save_from_remote_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入远程书源链接"));
    }
    let headers_map: HashMap<String, String> = HashMap::new();
    let resp = match crate::service::crawler::fetch(&url, &headers_map, 15, "GET", None, None, None)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("saveFromRemoteSource 抓取失败 [{url}]: {e}");
            return Json(ReturnData::err("远程书源链接错误"));
        }
    };
    // 校验：必须是书源数组（每项含 bookSourceUrl）
    let json: serde_json::Value = match serde_json::from_str(&resp.body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("书源数据格式错误")),
    };
    let sources = crate::model::book_source::normalize_book_sources(json);
    if sources.is_empty() || sources.iter().any(|s| s.book_source_url.trim().is_empty()) {
        return Json(ReturnData::err("书源数据格式错误"));
    }
    // preview=1：仅返回解析结果供前端弹窗选择/排序，不写库
    if params.get("preview").map(String::as_str) == Some("1") {
        let mut existing = Vec::new();
        for s in &sources {
            if state
                .storage
                .find_book_source(&namespace, &s.book_source_url)
                .await
                .ok()
                .flatten()
                .is_some()
            {
                existing.push(s.book_source_url.clone());
            }
        }
        return Json(ReturnData::ok(serde_json::json!({
            "sources": sources,
            "existing": existing
        })));
    }
    // F-7 书源数上限（同 saveBookSources——共享 book_source_limit_exceeded）
    let urls: Vec<&str> = sources.iter().map(|s| s.book_source_url.as_str()).collect();
    match book_source_limit_exceeded(&state.storage, &namespace, &urls).await {
        Ok(true) => return Json(ReturnData::err("超过书源数上限")),
        Ok(false) => {}
        Err(()) => return Json(ReturnData::err("系统错误")),
    }
    match state.storage.save_book_sources(&namespace, &sources).await {
        Ok(_) => Json(ReturnData::ok(
            serde_json::json!({ "count": sources.len() }),
        )),
        Err(e) => {
            tracing::error!("saveFromRemoteSource 入库失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteBookSource：删除书源（body/query bookSourceUrl）
async fn delete_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "bookSourceUrl");
    let url = if url.is_empty() {
        param_of(&params, body_json.as_ref(), "url")
    } else {
        url
    };
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_book_source(&namespace, &url).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteBookSource 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/deleteBookSources：批量删除（body = [bookSourceUrl]）
async fn delete_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let urls: Vec<String> = match serde_json::from_slice(&body) {
        Ok(u) => u,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let mut deleted = 0u64;
    for url in &urls {
        if let Ok(n) = state.storage.delete_book_source(&namespace, url).await {
            deleted += n;
        }
    }
    Json(ReturnData::ok(serde_json::json!({ "deleted": deleted })))
}

/// POST /reader3/deleteAllBookSources：清空用户书源
async fn delete_all_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    _body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    match state.storage.delete_all_book_sources(&namespace).await {
        Ok(n) => Json(ReturnData::ok(serde_json::json!({ "deleted": n }))),
        Err(e) => {
            tracing::error!("deleteAllBookSources 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

// ---------------- 书源登录态（cookie 按用户隔离） ----------------

/// 登录参数合并：query + body（JSON 优先，form-urlencoded 兑底）——纯函数，可测
fn merge_login_params(
    query: &HashMap<String, String>,
    body: Option<&[u8]>,
) -> HashMap<String, String> {
    let mut m = query.clone();
    let Some(body) = body else { return m };
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(obj) = v.as_object() {
            for (k, val) in obj {
                if let Some(s) = val.as_str() {
                    m.insert(k.clone(), s.to_string());
                } else {
                    m.insert(k.clone(), val.to_string());
                }
            }
            return m;
        }
    }
    for (k, v) in url::form_urlencoded::parse(body) {
        m.insert(k.into_owned(), v.into_owned());
    }
    m
}

/// 解析 bookSource 参数（书源 URL 或完整 JSON）——复用 resolve_book_source 语义
async fn resolve_login_source(
    state: &AppState,
    ns: &str,
    book_source_param: &str,
) -> Option<crate::model::BookSource> {
    if book_source_param.trim_start().starts_with('{') {
        return serde_json::from_str(book_source_param).ok();
    }
    state
        .storage
        .find_book_source(ns, book_source_param)
        .await
        .ok()
        .flatten()
}

/// POST/GET /reader3/loginBookSource：书源登录（登录态独立于系统用户，cookie 按用户存库）
///
/// 参数（query 或 body JSON/form）：bookSource（书源 URL 或完整 JSON）、username、password、
/// captcha（图片验证码文本）、mode=browser（强制浏览器自动登录）。
///
/// 返回：
/// - 成功：{success: true, cookie}
/// - 图片验证码：{success: false, needCaptcha: true, captchaUrl, captchaId, message}
///   （前端显示验证码 → 输入后重新调用本接口（captcha 参数）或 POST submitCaptcha）
/// - 点击类验证码（滑块/点选）无法自动处理：{success: false, needManualCaptcha: true, message}
///   （引导：浏览器登录书源后，在书源设置粘贴 Cookie）
/// - 登录失败：{success: false, message}
#[axum::debug_handler]
async fn login_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let merged = merge_login_params(&params, body.as_deref());
    let book_source_param = merged.get("bookSource").cloned().unwrap_or_default();
    if book_source_param.is_empty() {
        return Json(ReturnData::err("缺少 bookSource 参数"));
    }
    let Some(source) = resolve_login_source(&state, &namespace, &book_source_param).await else {
        return Json(ReturnData::err("书源不存在（请先导入书源）"));
    };
    if source.login_url.as_deref().unwrap_or("").trim().is_empty() {
        return Json(ReturnData::err("书源未配置 loginUrl"));
    }
    let req = crate::service::login::LoginRequest {
        username: merged.get("username").cloned().unwrap_or_default(),
        password: merged.get("password").cloned().unwrap_or_default(),
        captcha: merged.get("captcha").cloned().unwrap_or_default(),
    };
    let mode = merged.get("mode").cloned().unwrap_or_default();
    let outcome = if mode == "browser" {
        if !crate::service::browser::is_browser_available() {
            return Json(ReturnData::err(
                "浏览器后端不可用（camoufox）——无法使用浏览器自动登录，请配置 READER_CAMOUFOX_URL（或安装 python3 + camoufox 自启动 scripts/camoufox_solver.py），或在书源设置粘贴 Cookie",
            ));
        }
        crate::service::login::login_browser(&state.storage, &namespace, &source, &req).await
    } else {
        crate::service::login::login_http(&state.storage, &namespace, &source, &req).await
    };
    match outcome {
        Ok(crate::service::login::LoginOutcome::Success { cookie }) => Json(ReturnData::ok(
            serde_json::json!({ "success": true, "needCaptcha": false, "cookie": cookie }),
        )),
        Ok(crate::service::login::LoginOutcome::NeedImageCaptcha {
            captcha_url,
            captcha_id,
            message,
        }) => Json(ReturnData::ok(serde_json::json!({
            "success": false,
            "needCaptcha": true,
            "captchaUrl": captcha_url,
            "captchaId": captcha_id,
            "message": message,
        }))),
        Ok(crate::service::login::LoginOutcome::NeedManualCookie { message }) => {
            Json(ReturnData::ok(
                serde_json::json!({ "success": false, "needManualCaptcha": true, "message": message }),
            ))
        }
        Ok(crate::service::login::LoginOutcome::Failed { message }) => Json(ReturnData::ok(
            serde_json::json!({ "success": false, "message": message }),
        )),
        Err(e) => {
            tracing::error!("loginBookSource 失败 [{}]: {e}", source.book_source_name);
            Json(ReturnData::err(e.to_string()))
        }
    }
}

/// POST /reader3/setBookSourceCookie：手动设置书源 cookie（按当前用户存库）
///
/// body/query：bookSource（书源 URL）+ cookie（cookie 串；空值 = 清除）
/// 场景：点击类验证码无法自动处理时，用户在浏览器登录书源后把 cookie 粘贴到书源设置
async fn set_book_source_cookie(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let merged = merge_login_params(&params, body.as_deref());
    let book_source = merged.get("bookSource").cloned().unwrap_or_default();
    if book_source.is_empty() {
        return Json(ReturnData::err("缺少 bookSource 参数"));
    }
    let cookie = merged.get("cookie").cloned().unwrap_or_default();
    if cookie.trim().is_empty() {
        match state.storage.clear_cookie(&namespace, &book_source).await {
            Ok(_) => Json(ReturnData::ok(
                serde_json::json!({ "success": true, "cleared": true }),
            )),
            Err(e) => {
                tracing::error!("setBookSourceCookie 清除失败: {e}");
                Json(ReturnData::err("清除失败"))
            }
        }
    } else {
        match state
            .storage
            .set_cookie(&namespace, &book_source, &cookie)
            .await
        {
            Ok(_) => Json(ReturnData::ok(serde_json::json!({ "success": true }))),
            Err(e) => {
                tracing::error!("setBookSourceCookie 写入失败: {e}");
                Json(ReturnData::err("保存失败"))
            }
        }
    }
}

/// GET/POST /reader3/getBookSourceCookie：读取当前用户全部书源登录态
/// （Cookie 管理：sourceUrl/cookie/userAgent/loginHeader/updatedAt——本人可见原文）
async fn get_book_source_cookie(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let _ = body;
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    match state.storage.list_cookies(&namespace).await {
        Ok(rows) => {
            let arr: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "sourceUrl": r.source_url,
                        "cookie": r.cookie,
                        "userAgent": r.user_agent,
                        "loginHeader": r.login_header,
                        "updatedAt": r.updated_at,
                    })
                })
                .collect();
            Json(ReturnData::ok(serde_json::Value::Array(arr)))
        }
        Err(e) => {
            tracing::error!("getBookSourceCookie [{namespace}] 失败: {e}");
            Json(ReturnData::err("读取失败"))
        }
    }
}

/// POST /reader3/getCaptcha：重新触发登录页 → 检测验证码 → 返回验证码资源
///
/// body：bookSource。返回 {captchaType: image|slider|click|none, captchaUrl(data URI), captchaId, pageUrl}
/// - image：验证码图片（服务端截图，前端可直接显示）→ 前端输入后 POST submitCaptcha
/// - slider/click：浏览器自动处理/降级（见 loginBookSource 契约）
async fn get_captcha(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let merged = merge_login_params(&params, body.as_deref());
    let book_source_param = merged.get("bookSource").cloned().unwrap_or_default();
    if book_source_param.is_empty() {
        return Json(ReturnData::err("缺少 bookSource 参数"));
    }
    let Some(source) = resolve_login_source(&state, &namespace, &book_source_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    match crate::service::login::get_captcha(&state.storage, &namespace, &source).await {
        Ok(data) => Json(ReturnData::ok(data)),
        Err(e) => {
            tracing::error!("getCaptcha 失败 [{}]: {e}", source.book_source_name);
            Json(ReturnData::err(e.to_string()))
        }
    }
}

/// POST /reader3/submitCaptcha：图片验证码文本回填（浏览器流）→ 登录 → {isLogin}
///
/// body：bookSource + captchaId + captchaText（+ 可选 username/password 覆盖会话值）
async fn submit_captcha(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let merged = merge_login_params(&params, body.as_deref());
    let book_source_param = merged.get("bookSource").cloned().unwrap_or_default();
    if book_source_param.is_empty() {
        return Json(ReturnData::err("缺少 bookSource 参数"));
    }
    let Some(source) = resolve_login_source(&state, &namespace, &book_source_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    let captcha_id = merged.get("captchaId").cloned().unwrap_or_default();
    let captcha_text = merged.get("captchaText").cloned().unwrap_or_default();
    if captcha_id.is_empty() {
        return Json(ReturnData::err("缺少 captchaId 参数"));
    }
    match crate::service::login::submit_captcha(
        &state.storage,
        &namespace,
        &source,
        &captcha_id,
        &captcha_text,
        merged.get("username").map(String::as_str),
        merged.get("password").map(String::as_str),
    )
    .await
    {
        Ok(data) => Json(ReturnData::ok(data)),
        Err(e) => {
            tracing::error!("submitCaptcha 失败 [{}]: {e}", source.book_source_name);
            Json(ReturnData::err(e.to_string()))
        }
    }
}

// ---------------- 缓存管理 ----------------

/// GET/POST /reader3/getCacheInfo：缓存统计（toc_cache 行数 / book_chapters 行数 /
/// 章节近似大小 sum length(content) / 目录缓存大小 / 总大小）
/// P0-9：需登录（resolve_namespace）——secure 模式匿名请求拒绝 NEED_LOGIN
async fn get_cache_info(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    // P0-9：缓存统计/清理接口加登录校验（缓存为全局表，不按命名空间隔离，仅要求登录）
    if let Err(ret) = resolve_namespace(&state, &params, &headers).await {
        return Json(ret);
    }
    match state.storage.get_cache_info().await {
        Ok(info) => Json(ReturnData::ok(
            serde_json::to_value(info).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getCacheInfo 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/clearCache：清空缓存（body/query {type: "toc"|"chapters"|"all"}）
/// P0-9：需登录（resolve_namespace）——secure 模式匿名请求拒绝 NEED_LOGIN
async fn clear_cache(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    if let Err(ret) = resolve_namespace(&state, &params, &headers).await {
        return Json(ret);
    }
    let mut cache_type = params.get("type").cloned().unwrap_or_default();
    if let Some(body) = body {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let Some(v) = json.get("type").and_then(|v| v.as_str()) {
                cache_type = v.to_string();
            }
        }
    }
    if cache_type.is_empty() {
        cache_type = "all".to_string();
    }
    if cache_type != "toc" && cache_type != "chapters" && cache_type != "all" {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.clear_cache(&cache_type).await {
        Ok((toc_deleted, chapters_deleted)) => Json(ReturnData::ok(serde_json::json!({
            "deletedToc": toc_deleted,
            "deletedChapters": chapters_deleted,
        }))),
        Err(e) => {
            tracing::error!("clearCache 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

// ---------------- 全书搜索（仅本地书） ----------------

/// GET/POST /reader3/searchBookContent：全书搜索（params key + bookUrl）
/// 本地书：book_chapters 表 LIKE 匹配正文 → data: [{chapterIndex, title, snippet}]
/// 书源书：返回提示“仅支持本地书内容搜索”
async fn search_book_content(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut key = param_of(&params, body_json.as_ref(), "key");
    if key.is_empty() {
        // legacy 参数名 keyword（BookController searchBookContent）
        key = param_of(&params, body_json.as_ref(), "keyword");
    }
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if key.is_empty() {
        return Json(ReturnData::err("请输入搜索关键字"));
    }
    if book_url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // 本地书判定：书架书（origin/url 形态）或 book_chapters 已有章节
    let shelf = state
        .storage
        .find_book(&namespace, &book_url)
        .await
        .ok()
        .flatten();
    let has_chapters = state
        .storage
        .count_chapters(&namespace, &book_url)
        .await
        .unwrap_or(0)
        > 0;
    match &shelf {
        Some(book) => {
            if !crate::service::local_book::is_local_book(&book.book_url, &book.origin) {
                return Json(ReturnData::err("仅支持本地书内容搜索"));
            }
        }
        None if !has_chapters => return Json(ReturnData::err("书籍不存在")),
        None => {}
    }
    // 文件型本地书（legacy loc_book：正文不入章节表）——解析文件逐章匹配
    if let Some(book) = &shelf {
        if book.origin == "loc_book" && book.book_url.starts_with("storage/") {
            return match search_file_book_content(&state, &namespace, book, &key).await {
                Ok(hits) => Json(ReturnData::ok(
                    serde_json::to_value(hits).unwrap_or(serde_json::Value::Null),
                )),
                Err(e) => {
                    tracing::warn!("searchBookContent 文件书失败 [{book_url}]: {e}");
                    Json(ReturnData::err("搜索失败"))
                }
            };
        }
    }
    match state
        .storage
        .search_book_content(&book_url, &key, 100)
        .await
    {
        Ok(hits) => {
            // A4b 契约对齐（Pro SearchResult 全字段并存——保留 title/snippet 别名
            // 供 web 前端，补 chapterTitle/resultText/query/resultCount 等供 App 端）
            let total = hits.len() as i64;
            let enriched: Vec<serde_json::Value> = hits
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    serde_json::json!({
                        "chapterIndex": h.chapter_index,
                        "title": h.title,
                        "snippet": h.snippet,
                        // Pro SearchResult 契约字段
                        "chapterTitle": h.title,
                        "resultText": h.snippet,
                        "query": key,
                        "resultCount": total,
                        "resultCountWithinChapter": 1,
                        "pageIndex": i as i64,
                        "queryIndexInResult": i as i64,
                        "queryIndexInChapter": 0,
                    })
                })
                .collect();
            Json(ReturnData::ok(
                serde_json::to_value(enriched).unwrap_or(serde_json::Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("searchBookContent 失败 [{book_url}]: {e}");
            Json(ReturnData::err("搜索失败"))
        }
    }
}

/// 文件型本地书全书搜索：解析文件 → 逐章匹配（key 大小写不敏感，snippet 取命中上下文）
async fn search_file_book_content(
    state: &AppState,
    namespace: &str,
    book: &crate::model::book::Book,
    key: &str,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let path = resolve_loc_book_file(&state.storage.config.storage_dir(), &book.book_url)
        .ok_or_else(|| anyhow::anyhow!("文件不存在"))?;
    let path_lower = path.to_string_lossy().to_lowercase();
    let imported = if path_lower.ends_with(".epub") {
        let bytes = std::fs::read(&path)?;
        crate::service::local_book::parse_epub(&bytes, &book.toc_url)?
    } else {
        let user_rules = txt_toc_rule_regexes(state, namespace).await;
        crate::service::local_book::parse_txt_file_with_rules(&path, &user_rules)?
    };
    let key_lower = key.to_lowercase();
    let mut hits: Vec<serde_json::Value> = Vec::new();
    for (i, ch) in imported.chapters.iter().enumerate() {
        if hits.len() >= 100 {
            break;
        }
        let title = if ch.title.is_empty() {
            format!("第{}章", i + 1)
        } else {
            ch.title.clone()
        };
        let matched_in_title = ch.title.to_lowercase().contains(&key_lower);
        let content_lower = ch.content.to_lowercase();
        let pos = if matched_in_title {
            Some(0usize)
        } else {
            content_lower.find(&key_lower)
        };
        if let Some(p) = pos {
            let content = &ch.content;
            let start = content.floor_char_boundary(p.saturating_sub(30));
            let end = content.floor_char_boundary((p + key.len() + 50).min(content.len()));
            let snippet = if start > 0 { "…" } else { "" }.to_string()
                + &ch.content[start..end]
                + if end < ch.content.len() { "…" } else { "" };
            hits.push(serde_json::json!({
                "chapterIndex": i,
                "title": title,
                "snippet": snippet,
            }));
        }
    }
    Ok(hits)
}

// ---------------- 书源订阅 ----------------

/// GET/POST /reader3/getSourceSubs：订阅列表（url/name/enabled）
async fn get_source_subs(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = body;
    match state.storage.get_source_subs(&namespace).await {
        Ok(list) => Json(ReturnData::ok(
            serde_json::to_value(list).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getSourceSubs [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// 抓取订阅 URL → 校验书源数组 → 订阅入库（raw_json 存原文）+ 批量导入书源（已存在覆盖）
/// （saveSourceSub / refreshSourceSub 共用；核心逻辑在 service::schedule，定时刷新复用）；
/// 返回导入书源数
async fn fetch_and_store_source_sub(
    state: &AppState,
    ns: &str,
    url: &str,
    name: &str,
    selected_urls: &[String],
) -> Result<(usize, String), ReturnData> {
    crate::service::schedule::refresh_source_sub_core(&state.storage, ns, url, name, selected_urls)
        .await
        .map_err(|e| ReturnData::err(e.to_string()))
}

/// POST /reader3/saveSourceSub：订阅书源集合（body {url, name}）——抓取校验后入库 + 批量导入书源
async fn save_source_sub(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入订阅链接"));
    }
    let mut name = param_of(&params, body_json.as_ref(), "name");
    if name.is_empty() {
        name = url.clone();
    }
    let selected_urls: Vec<String> = body_json
        .as_ref()
        .and_then(|v| v.get("selectedUrls").and_then(|a| a.as_array()))
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    match fetch_and_store_source_sub(&state, &namespace, &url, &name, &selected_urls).await {
        Ok((count, display_name)) => Json(ReturnData::ok(serde_json::json!({
            "count": count,
            "name": display_name
        }))),
        Err(ret) => Json(ret),
    }
}

/// POST /reader3/refreshSourceSub：重新拉取订阅并覆盖书源（url 参数；订阅需已存在）
async fn refresh_source_sub(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入订阅链接"));
    }
    let sub = match state.storage.find_source_sub(&namespace, &url).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(ReturnData::err("订阅不存在")),
        Err(_) => return Json(ReturnData::err("系统错误")),
    };
    match fetch_and_store_source_sub(&state, &namespace, &url, &sub.name, &sub.selected_urls.0)
        .await
    {
        Ok((count, display_name)) => Json(ReturnData::ok(serde_json::json!({
            "count": count,
            "name": display_name
        }))),
        Err(ret) => Json(ret),
    }
}

/// POST /reader3/previewSourceSub：拉取订阅 URL 并返回书源列表 + 库内已存在 URL，
/// 供前端弹窗选择/排序后确认导入（不写订阅、不导入书源）。
async fn preview_source_sub(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入订阅链接"));
    }
    let sources = match crate::service::schedule::fetch_source_sub_sources(&url).await {
        Ok((sources, _)) => sources,
        Err(e) => return Json(ReturnData::err(e.to_string())),
    };
    let mut existing = Vec::new();
    for s in &sources {
        if state
            .storage
            .find_book_source(&namespace, &s.book_source_url)
            .await
            .ok()
            .flatten()
            .is_some()
        {
            existing.push(s.book_source_url.clone());
        }
    }
    Json(ReturnData::ok(serde_json::json!({
        "sources": sources,
        "existing": existing
    })))
}

/// POST /reader3/setSourceSubEnabled：启停订阅（body {url, enabled}）。
/// 禁用后定时任务不再自动刷新该订阅，订阅记录与已导入书源保留；重新启用恢复自动刷新。
async fn set_source_sub_enabled(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let enabled = body_json
        .as_ref()
        .and_then(|v| v.get("enabled").and_then(|e| e.as_bool()))
        .unwrap_or(true);
    match state
        .storage
        .set_source_sub_enabled(&namespace, &url, enabled)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::json!({ "enabled": enabled }))),
        Err(e) => {
            tracing::error!("setSourceSubEnabled 失败 [{url}]: {e}");
            Json(ReturnData::err("操作失败"))
        }
    }
}

/// POST /reader3/deleteSourceSub：删除订阅（url 参数；仅删订阅行，不影响已导入书源）
async fn delete_source_sub(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_source_sub(&namespace, &url).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteSourceSub 失败 [{url}]: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/deleteSourceSubs：批量删除订阅（body：URL 数组或 { urls: [] }）；
/// 逐条复用 delete_source_sub 语义（default 系统订阅对普通用户仅隐藏覆盖）。
async fn delete_source_subs(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let urls: Vec<String> = body_json
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .or_else(|| {
            body_json
                .as_ref()
                .and_then(|v| v.get("urls").and_then(|u| u.as_array()))
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
        })
        .unwrap_or_default();
    if urls.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let mut deleted = 0usize;
    for url in urls {
        match state.storage.delete_source_sub(&namespace, &url).await {
            Ok(n) if n > 0 => deleted += 1,
            Ok(_) => {}
            Err(e) => {
                tracing::error!("deleteSourceSubs 删除失败 [{url}]: {e}");
                return Json(ReturnData::err("删除失败"));
            }
        }
    }
    Json(ReturnData::ok(serde_json::json!({ "deleted": deleted })))
}

// ---------------- RSS ----------------

/// GET/POST /reader3/getRssSources：RSS 源列表（用户命名空间，无则回退 default）
async fn get_rss_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let _ = body;
    match state.storage.get_rss_sources(&namespace).await {
        Ok(list) => {
            let arr: Vec<Value> = list.iter().map(rss_source_json).collect();
            Json(ReturnData::ok(Value::Array(arr)))
        }
        Err(e) => {
            tracing::error!("getRssSources [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveRssSource：保存 RSS 源（body = 完整 RSS 源 JSON，sourceUrl/sourceName 必填）
async fn save_rss_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let body_str = String::from_utf8_lossy(&body).to_string();
    let mut source: crate::model::RssSource = match serde_json::from_slice(&body) {
        Ok(s) => s,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if source.source_url.trim().is_empty() {
        return Json(ReturnData::err("RSS链接不能为空"));
    }
    if source.source_name.trim().is_empty() {
        return Json(ReturnData::err("RSS名称不能为空"));
    }
    // raw_json：完整 JSON 原文保底（未知字段不丢，列表接口原样回吐）
    source.raw_json = Some(body_str);
    match state.storage.save_rss_source(&namespace, &source).await {
        Ok(()) => Json(ReturnData::ok(Value::String(String::new()))),
        Err(e) => {
            tracing::error!("saveRssSource 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteRssSource：删除 RSS 源（rssSourceUrl 参数）
async fn delete_rss_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json: Option<Value> = body.as_ref().and_then(|b| serde_json::from_slice(b).ok());
    let mut url = param_of(&params, body_json.as_ref(), "rssSourceUrl");
    if url.is_empty() {
        // legacy 客户端直接 POST 完整 RssSource JSON——按 sourceUrl 字段匹配
        url = param_of(&params, body_json.as_ref(), "sourceUrl");
    }
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_rss_source(&namespace, &url).await {
        Ok(_) => Json(ReturnData::ok(Value::String(String::new()))),
        Err(e) => {
            tracing::error!("deleteRssSource 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// GET/POST /reader3/getRssArticles：抓取 feed → 解析文章列表 → 入库 → 返回
async fn get_rss_articles(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json: Option<Value> = body.as_ref().and_then(|b| serde_json::from_slice(b).ok());
    // rssSourceUrl 为主参数（兼容 legacy sourceUrl）
    let mut source_url = param_of(&params, body_json.as_ref(), "rssSourceUrl");
    if source_url.is_empty() {
        source_url = param_of(&params, body_json.as_ref(), "sourceUrl");
    }
    let page = body_json
        .as_ref()
        .and_then(|b| b.get("page").and_then(|v| v.as_i64()))
        .or_else(|| params.get("page").and_then(|v| v.parse().ok()))
        .unwrap_or(1);
    // 分类 URL（legacy sortUrl 多段 `名称::地址`，前端按源解析后传其中一段）
    let mut sort_url = param_of(&params, body_json.as_ref(), "sortUrl");
    if sort_url.is_empty() {
        sort_url = param_of(&params, body_json.as_ref(), "sort_url");
    }
    if source_url.is_empty() {
        return Json(ReturnData::err("RSS源链接不能为空"));
    }
    let Some(source) = state
        .storage
        .find_rss_source(&namespace, &source_url)
        .await
        .ok()
        .flatten()
    else {
        return Json(ReturnData::err("RSS源不存在"));
    };
    let sort_param = if sort_url.is_empty() {
        None
    } else {
        Some(sort_url.as_str())
    };
    match crate::service::rss::fetch_articles(&source, page, sort_param).await {
        Ok(articles) => {
            if let Err(e) = state.storage.save_rss_articles(&namespace, &articles).await {
                tracing::warn!("getRssArticles 入库失败: {e}");
            }
            // 已读标记合并：入库后按 url 回读 read 列 → 序列化为 hasRead
            let flags = state
                .storage
                .get_rss_article_read_flags(&namespace, &source_url)
                .await
                .unwrap_or_default();
            let articles: Vec<crate::model::RssArticle> = articles
                .into_iter()
                .map(|mut a| {
                    a.read = flags.get(&a.url).copied().unwrap_or(false);
                    a
                })
                .collect();
            Json(ReturnData::ok(
                serde_json::to_value(&articles).unwrap_or(Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("getRssArticles 抓取失败 [{}]: {e}", source.source_url);
            Json(ReturnData::err("抓取失败"))
        }
    }
}

/// POST /reader3/markRssArticleRead：标记 RSS 文章已读/未读（body { articleUrl, read }）
async fn mark_rss_article_read(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json: Option<Value> = body.as_ref().and_then(|b| serde_json::from_slice(b).ok());
    let url = param_of(&params, body_json.as_ref(), "articleUrl");
    if url.is_empty() {
        return Json(ReturnData::err("RSS文章链接不能为空"));
    }
    let read = body_json
        .as_ref()
        .and_then(|b| b.get("read").and_then(|v| v.as_bool()))
        .unwrap_or(true);
    // P0-4：按 (ns, url) 查改——跨命名空间/不存在的文章影响 0 行 → 显式拒绝
    match state
        .storage
        .set_rss_article_read(&namespace, &url, read)
        .await
    {
        Ok(0) => Json(ReturnData::err("文章不存在或无权操作")),
        Ok(_) => Json(ReturnData::ok(Value::Null)),
        Err(e) => {
            tracing::error!("markRssArticleRead 失败: {e}");
            Json(ReturnData::err("标记失败"))
        }
    }
}

/// GET/POST /reader3/getRssArticle：文章正文（url 参数；feed 已带 content 直接返回，
/// 否则抓取文章网页用 CSS 选择器提取正文）
async fn get_rss_article(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json: Option<Value> = body.as_ref().and_then(|b| serde_json::from_slice(b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("RSS文章链接不能为空"));
    }
    // P0-4：按 (ns, url) 查——跨命名空间文章视为未入库（走网页抓取兜底）
    let mut rss_source: Option<crate::model::RssSource> = None;
    if let Ok(Some(article)) = state.storage.get_rss_article(&namespace, &url).await {
        if article
            .content
            .as_deref()
            .is_some_and(|c| !c.trim().is_empty())
        {
            return Json(ReturnData::ok(
                serde_json::to_value(&article).unwrap_or(Value::Null),
            ));
        }
        rss_source = state
            .storage
            .find_rss_source(&namespace, &article.source_url)
            .await
            .ok()
            .flatten();
    }
    // 未带正文 → 抓取网页提取正文
    let content_result = match rss_source {
        Some(src) => crate::service::rss::fetch_article_content(&src, &url).await,
        None => crate::service::rss::fetch_web_content(&url).await,
    };
    match content_result {
        Ok(content) => {
            let article = crate::model::RssArticle {
                url: url.clone(),
                title: String::new(),
                content: Some(content),
                ..Default::default()
            };
            Json(ReturnData::ok(
                serde_json::to_value(&article).unwrap_or(Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("getRssArticle 正文提取失败 [{url}]: {e}");
            Json(ReturnData::err("正文提取失败"))
        }
    }
}

/// RSS 源 JSON 输出：raw_json（完整 legacy 字段）为基底，表列字段覆盖（名称/分组/启用状态）
fn rss_source_json(source: &crate::model::RssSource) -> Value {
    let mut v = source
        .raw_json
        .as_deref()
        .and_then(|r| serde_json::from_str::<Value>(r).ok())
        .unwrap_or_else(|| serde_json::to_value(source).unwrap_or(Value::Null));
    if let Some(obj) = v.as_object_mut() {
        obj.insert("sourceUrl".into(), Value::String(source.source_url.clone()));
        obj.insert(
            "sourceName".into(),
            Value::String(source.source_name.clone()),
        );
        obj.insert(
            "sourceGroup".into(),
            source
                .source_group
                .as_ref()
                .map(|g| Value::String(g.clone()))
                .unwrap_or(Value::Null),
        );
        obj.insert("enabled".into(), Value::Bool(source.enabled));
    }
    v
}

/// POST/GET /reader3/searchBook：单书源搜索（bookSource 参数：书源 URL 或完整 JSON）
async fn search_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下书源功能未开启 → 拒绝
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return Json(ret);
    }
    // 参数解析（POST body JSON 优先，GET query 兜底）
    let mut key = params.get("key").cloned().unwrap_or_default();
    // legacy "=" 前缀 = 精确搜索（accurate）
    let mut key_exact_prefix = false;
    if key.starts_with('=') {
        key_exact_prefix = true;
        key.remove(0);
    }
    let mut page = params
        .get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1i64);
    let mut book_source_param = params.get("bookSource").cloned().unwrap_or_default();
    if let Some(body) = body {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let Some(v) = json.get("key").and_then(|v| v.as_str()) {
                key = v.to_string();
            }
            if let Some(v) = json.get("page").and_then(|v| v.as_i64()) {
                page = v;
            }
            if let Some(v) = json.get("bookSource").and_then(|v| v.as_str()) {
                book_source_param = v.to_string();
            }
        }
    }
    if key.is_empty() {
        return Json(ReturnData::err("请输入搜索关键字"));
    }
    if book_source_param.is_empty() {
        return Json(ReturnData::err("未配置书源"));
    }

    // 解析书源：完整 JSON 或 URL（从库查）
    let source: Option<crate::model::BookSource> =
        if book_source_param.trim_start().starts_with('{') {
            serde_json::from_str(&book_source_param).ok()
        } else {
            state
                .storage
                .find_book_source(&namespace, &book_source_param)
                .await
                .ok()
                .flatten()
        };
    let Some(source) = source else {
        return Json(ReturnData::err("书源不存在"));
    };

    match crate::service::search::search_one_source(&state.storage, &namespace, &source, &key, page)
        .await
    {
        Ok(books) => {
            // legacy "=" 前缀/exact=1：按书名/作者等值过滤（大小写/全半角忽略）
            let exact = key_exact_prefix || params.get("exact").map(|v| v == "1").unwrap_or(false);
            let books = if exact {
                crate::service::search::filter_exact(books, &key)
            } else {
                books
            };
            Json(ReturnData::ok(
                serde_json::to_value(books).unwrap_or(serde_json::Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("搜索失败 [{}]: {e:?}", source.book_source_name);
            Json(ReturnData::err("搜索失败"))
        }
    }
}

/// POST/GET /reader3/searchBookMulti：多书源并发搜索（可选 bookSourceGroup 过滤）
async fn search_book_multi(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下书源功能未开启 → 拒绝
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return Json(ret);
    }
    let mut key = params.get("key").cloned().unwrap_or_default();
    // legacy "=" 前缀 = 精确搜索（accurate）
    let mut key_exact_prefix = false;
    if key.starts_with('=') {
        key_exact_prefix = true;
        key.remove(0);
    }
    let mut page = params
        .get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1i64);
    let mut group = params.get("bookSourceGroup").cloned().unwrap_or_default();
    // P1-4 单源指定：精确匹配 bookSourceUrl（非空时只搜该源）
    let mut single_source_url = params.get("bookSourceUrl").cloned().unwrap_or_default();
    let mut exact = params.get("exact").map(|v| v == "1").unwrap_or(false);
    let mut max_sources = params
        .get("maxSources")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(usize::MAX);
    if let Some(body) = body {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let Some(v) = json.get("key").and_then(|v| v.as_str()) {
                key = v.to_string();
            }
            if let Some(v) = json.get("page").and_then(|v| v.as_i64()) {
                page = v;
            }
            if let Some(v) = json.get("bookSourceGroup").and_then(|v| v.as_str()) {
                group = v.to_string();
            }
            if let Some(v) = json.get("bookSourceUrl").and_then(|v| v.as_str()) {
                single_source_url = v.to_string();
            }
            if let Some(v) = json.get("bookSourceUrl").and_then(|v| v.as_str()) {
                single_source_url = v.to_string();
            }
            if let Some(v) = json.get("exact") {
                exact = v.as_u64() == Some(1)
                    || v.as_str()
                        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
            }
            if let Some(v) = json.get("maxSources").and_then(|v| v.as_u64()) {
                max_sources = v as usize;
            }
        }
    }
    // legacy "=" 前缀强制精确
    if key_exact_prefix {
        exact = true;
    }
    if key.is_empty() {
        return Json(ReturnData::err("请输入搜索关键字"));
    }
    let sources = match state.storage.get_book_sources(&namespace).await {
        Ok(s) => s,
        Err(_) => return Json(ReturnData::err("系统错误")),
    };
    let mut sources: Vec<crate::model::BookSource> = sources
        .into_iter()
        .filter(|s| s.enabled && s.search_url.is_some())
        .filter(|s| !crate::service::health::is_source_invalid(&namespace, &s.book_source_url))
        .filter(|s| book_source_group_matches(&group, s.book_source_group.as_deref()))
        .filter(|s| single_source_url.is_empty() || s.book_source_url == single_source_url)
        .collect();
    // 防炸：限制搜索源数量（前端按组搜索时通常远小于此）
    if sources.len() > max_sources {
        sources.truncate(max_sources);
    }
    if sources.is_empty() {
        return Json(ReturnData::err("未配置书源"));
    }

    // 并发搜索（限制并发数 24——多书源场景下 8 并发会明显拖慢整批搜索）
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(24));
    let mut handles = Vec::with_capacity(sources.len());
    let ns = namespace.clone();
    let storage = state.storage.clone();
    for source in sources {
        let sem = semaphore.clone();
        let key = key.clone();
        let ns = ns.clone();
        let storage = storage.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            crate::service::search::search_one_source(&storage, &ns, &source, &key, page)
                .await
                .unwrap_or_default()
        }));
    }
    let mut all: Vec<crate::service::search::SearchBook> = Vec::new();
    for h in handles {
        if let Ok(books) = h.await {
            all.extend(books);
        }
    }
    // 精确模式（exact=1）：书源规则解析后按书名/作者等值过滤（大小写/全半角忽略）
    if exact {
        all = crate::service::search::filter_exact(all, &key);
    }
    // 按 书名+作者 去重（legacy searchBookMulti 语义：同书多书源只保留一条）
    all = dedup_search_books(all);
    Json(ReturnData::ok(
        serde_json::to_value(all).unwrap_or(serde_json::Value::Null),
    ))
}

/// 多源搜索结果去重：按 (书名, 作者) 键，保留首个书源命中
/// （对齐 legacy BookController 的 `book.name + "_" + book.author` 去重；
/// legacy 不 trim——含首尾空格的书名视为不同书）
fn dedup_search_books(
    books: Vec<crate::service::search::SearchBook>,
) -> Vec<crate::service::search::SearchBook> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(books.len());
    for b in books {
        let key = format!("{}_{}", b.name, b.author);
        if key.is_empty() {
            continue;
        }
        if seen.insert(key) {
            out.push(b);
        }
    }
    out
}

/// POST/GET /reader3/searchBookSource：换源搜索
///
/// 参数：url（当前书 bookUrl）+ bookSource（当前源 URL/名称）
/// 逻辑：取当前书名 → 全部启用可搜索书源（排除当前源）并发搜索 → 书名匹配过滤 → 按书源去重
/// 返回：SearchBook[]（每项含 origin/originName/tocUrl，前端点击后 saveBook 切源）
async fn search_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：换源同样受书源权限开关约束
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    let book_source_param = param_of(&params, body_json.as_ref(), "bookSource");
    // 精确模式（exact=1）：书名等值匹配（大小写/全半角忽略）；缺省模糊（双向包含）
    let exact = params.get("exact").map(|v| v == "1").unwrap_or(false)
        || body_json
            .as_ref()
            .and_then(|j| j.get("exact"))
            .map(|v| {
                v.as_u64() == Some(1)
                    || v.as_str()
                        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
                        .unwrap_or(false)
            })
            .unwrap_or(false);
    if url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接"));
    }
    if book_source_param.is_empty() {
        return Json(ReturnData::err("未配置书源"));
    }

    // ① 当前书名：书架优先；未入架走详情解析（同 getBookInfo）
    let name = match state.storage.find_book(&namespace, &url).await {
        Ok(Some(b)) => b.name,
        _ => {
            let Some(source) = resolve_book_source(&state, &namespace, &book_source_param).await
            else {
                return Json(ReturnData::err("书源不存在"));
            };
            match crate::service::book::fetch_book_info(&namespace, &url, &source, None).await {
                Ok(info) => {
                    if info.name.is_empty() {
                        return Json(ReturnData::err("获取书籍信息失败"));
                    }
                    info.name
                }
                Err(e) => {
                    tracing::error!("searchBookSource 获取书名失败 [{url}]: {e}");
                    return Json(ReturnData::err("获取书籍信息失败"));
                }
            }
        }
    };
    let key = name.trim();
    if key.is_empty() {
        return Json(ReturnData::err("无法获取书名"));
    }

    // ② 全部启用可搜索书源（排除当前源：URL 或名称匹配）
    let current = book_source_param.trim();
    let mut sources: Vec<crate::model::BookSource> =
        match state.storage.get_book_sources(&namespace).await {
            Ok(s) => s
                .into_iter()
                .filter(|s| {
                    s.enabled
                        && s.search_url.is_some()
                        && s.book_source_url != current
                        && s.book_source_name != current
                })
                .collect(),
            Err(_) => return Json(ReturnData::err("系统错误")),
        };
    if sources.is_empty() {
        return Json(ReturnData::ok(serde_json::Value::Null));
    }

    // F2：legacy 分页契约（YueduApi.kt:1018-1109）——请求带 lastIndex 时启用：
    // 从 lastIndex+1 起取 searchSize 个源并发搜索；bookSourceGroup 过滤；
    // 返回 {lastIndex, list} 形态；越界报「没有更多了」。
    // 无 lastIndex → master 原生全量形态（SearchBook[]）不变。
    let last_index_param = body_json
        .as_ref()
        .and_then(|j| j.get("lastIndex"))
        .and_then(|v| v.as_i64())
        .or_else(|| params.get("lastIndex").and_then(|v| v.parse::<i64>().ok()));
    let search_size = body_json
        .as_ref()
        .and_then(|j| j.get("searchSize"))
        .and_then(|v| v.as_u64())
        .or_else(|| params.get("searchSize").and_then(|v| v.parse::<u64>().ok()))
        .unwrap_or(0) as usize;
    let group_filter = param_of(&params, body_json.as_ref(), "bookSourceGroup");
    if !group_filter.is_empty() {
        sources.retain(|s| {
            s.book_source_group
                .as_deref()
                .map(|g| {
                    g.split([',', '，', ';', '；', ' '])
                        .any(|p| p.trim() == group_filter)
                })
                .unwrap_or(false)
        });
        if sources.is_empty() {
            return Json(ReturnData::ok(serde_json::Value::Null));
        }
    }
    let paginated = last_index_param.is_some();
    if paginated {
        let li = last_index_param.unwrap();
        if li >= sources.len() as i64 - 1 {
            return Json(ReturnData::err("没有更多了"));
        }
        let start = (li + 1).max(0) as usize;
        let size = if search_size > 0 { search_size } else { 5 };
        let end = (start + size).min(sources.len());
        sources = sources[start..end].to_vec();
    }

    // ③ 并发搜索（24，同 searchBookMulti）
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(24));
    let mut handles = Vec::with_capacity(sources.len());
    let ns = namespace.clone();
    let storage = state.storage.clone();
    let src_count = sources.len() as i64;
    for source in &sources {
        let sem = semaphore.clone();
        let key = key.to_string();
        let ns = ns.clone();
        let storage = storage.clone();
        let source = source.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            crate::service::search::search_one_source(&storage, &ns, &source, &key, 1)
                .await
                .unwrap_or_default()
        }));
    }

    // ④ 汇总：书名匹配过滤（忽略大小写，双向包含；精确模式=书名/作者等值）+ 按书源去重（保留首条）
    let mut all: Vec<crate::service::search::SearchBook> = Vec::new();
    for h in handles {
        if let Ok(books) = h.await {
            all.extend(books);
        }
    }
    let ql = key.to_lowercase();
    let mut seen = std::collections::HashSet::new();
    let matched: Vec<_> = all
        .into_iter()
        .filter(|b| {
            if exact {
                crate::service::search::exact_match(b, key)
            } else {
                let bl = b.name.to_lowercase();
                bl.contains(&ql) || ql.contains(&bl)
            }
        })
        .filter(|b| seen.insert(b.origin.clone()))
        .collect();
    tracing::info!(
        "searchBookSource [{namespace}] 《{key}》：命中 {} 条",
        matched.len()
    );
    if paginated {
        let new_last = last_index_param.unwrap() + src_count;
        let new_last = if sources.is_empty() {
            last_index_param.unwrap()
        } else {
            new_last
        };
        return Json(ReturnData::ok(json!({
            "lastIndex": new_last,
            "list": matched,
        })));
    }
    Json(ReturnData::ok(
        serde_json::to_value(matched).unwrap_or(serde_json::Value::Null),
    ))
}

/// 解析书源参数（完整 JSON 或 URL 查库）
async fn resolve_book_source(
    state: &AppState,
    ns: &str,
    param: &str,
) -> Option<crate::model::BookSource> {
    if param.trim_start().starts_with('{') {
        serde_json::from_str(param).ok()
    } else {
        state
            .storage
            .find_book_source(ns, param)
            .await
            .ok()
            .flatten()
    }
}

/// 从 query/body 取参
pub(crate) fn param_of(
    params: &HashMap<String, String>,
    body: Option<&serde_json::Value>,
    key: &str,
) -> String {
    if let Some(b) = body {
        if let Some(v) = b.get(key) {
            if let Some(s) = v.as_str() {
                return s.to_string();
            }
            if !v.is_null() {
                return v.to_string();
            }
        }
    }
    params.get(key).cloned().unwrap_or_default()
}

/// 书源分组匹配：group 为空匹配全部；书源分组支持逗号/顿号/空白分隔
fn book_source_group_matches(group: &str, source_groups: Option<&str>) -> bool {
    if group.is_empty() {
        return true;
    }
    source_groups
        .map(|g| {
            g.split([',', '，', '、', ' ', '\t'])
                .any(|part| part == group)
        })
        .unwrap_or(false)
}

/// POST/GET /reader3/getBookInfo：书籍详情（ruleBookInfo）

/// GET/POST /reader3/setBookSource：换源持久化（legacy setBookSource 对齐）
/// 参数：bookUrl（旧）+ newUrl（新源书籍链接）+ bookSourceUrl（新书源）
/// 行为：书架书原地更新 origin/originName/bookUrl/tocUrl（封面仅原为空时补，
/// 阅读进度保留），旧 URL 章节/目录缓存清理；随后尽力预取新目录缓存（失败不影响换源）

/// GET /reader3/getUserInfo（legacy 对齐）：当前用户 + secure 标志 + 字体清单。
/// 未登录也返回 200（userInfo=null——legacy checkAuth 不拦截此端点）
async fn get_user_info(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    let user = resolve_current_user(&state, &params, &headers).await.ok();
    let user_info = user.as_ref().map(format_user);
    // storage/assets/fonts 下 ttf 清单（legacy listFilesRecursively 过滤 .ttf）
    let fonts_dir = state
        .storage
        .config
        .storage_dir()
        .join("assets")
        .join("fonts");
    let mut fonts: Vec<serde_json::Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&fonts_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_file() {
                continue;
            }
            let is_ttf = p
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("ttf"))
                .unwrap_or(false);
            if !is_ttf {
                continue;
            }
            fonts.push(serde_json::json!({
                "name": p.file_name().and_then(|n| n.to_str()).unwrap_or_default(),
                "size": e.metadata().map(|m| m.len()).unwrap_or(0),
            }));
        }
    }
    Json(ReturnData::ok(serde_json::json!({
        "userInfo": user_info,
        "secure": state.storage.config.secure,
        "secureKey": !state.storage.config.secure_key.is_empty(),
        "fonts": fonts,
    })))
}

/// GET /reader3/cover?path=<图片URL>（legacy getBookCover 兼容）：
/// 复用 image_cache 磁盘缓存代理抓取（UA/Referer/SSRF 防护与 /assets/proxy 一致）；
/// 命中/抓取成功 → Cache-Control: max-age=86400；失败 → 404
async fn book_cover_legacy(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let path = params.get("path").cloned().unwrap_or_default();
    if path.is_empty() {
        return (StatusCode::NOT_FOUND, "").into_response();
    }
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(_) => return (StatusCode::NOT_FOUND, "").into_response(),
    };
    // legacy Referer = url 截到最后一个 '/' 前
    let referer = path.rsplit_once('/').map(|(base, _)| base.to_string());
    match state
        .image_cache
        .get_or_fetch(&namespace, &path, referer.as_deref(), 10, 5 * 1024 * 1024)
        .await
    {
        Ok((bytes, content_type, status, _from_cache))
            if (200..300).contains(&status) && !bytes.is_empty() =>
        {
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    content_type.unwrap_or_else(|| "image/png".to_string()),
                )
                .header(axum::http::header::CACHE_CONTROL, "max-age=86400")
                .body(Body::from(bytes))
                .unwrap()
        }
        _ => (StatusCode::NOT_FOUND, "").into_response(),
    }
}

/// GET/POST /reader3/deleteFile（legacy 对齐）：删除 /assets/{ns}/ 下用户文件/目录。
/// 防穿越：规范化后必须仍位于 assets/{ns} 内
async fn delete_file(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入文件链接"));
    }
    let prefix = format!("/assets/{namespace}/");
    if !url.starts_with(&prefix) {
        return Json(ReturnData::err("文件链接错误"));
    }
    let rel = url[prefix.len()..].replace('\\', "/");
    if rel.is_empty() {
        return Json(ReturnData::err("文件链接错误"));
    }
    let home = state
        .storage
        .config
        .storage_dir()
        .join("assets")
        .join(&namespace);
    let target = home.join(&rel);
    let (Ok(home_canon), Ok(target_canon)) = (home.canonicalize(), target.canonicalize()) else {
        return Json(ReturnData::err("文件链接错误"));
    };
    if !target_canon.starts_with(&home_canon) || target_canon == home_canon {
        return Json(ReturnData::err("文件链接错误"));
    }
    if target_canon.is_dir() {
        let _ = std::fs::remove_dir_all(&target_canon);
    } else {
        let _ = std::fs::remove_file(&target_canon);
    }
    Json(ReturnData::ok(serde_json::json!("")))
}

/// 上传文件名收敛（legacy：`'\\'→'/'` 后取末段 basename；空名/隐藏名拒绝）。
/// 返回 None 表示该文件应跳过（不写入、不计入 URL 列表）
fn sanitize_upload_filename(raw: &str) -> Option<String> {
    let name = raw.replace('\\', "/");
    let base = name.rsplit('/').next().unwrap_or_default();
    if base.is_empty() || base.starts_with('.') {
        return None;
    }
    Some(base.to_string())
}

/// POST /reader3/uploadFile（legacy UserController.uploadFile 对齐）：用户资产上传。
/// 与 /reader3/file/upload 同名异义——本端点写 storage/assets/{ns}/{type}/ 并返回
/// URL 数组 ["/assets/{ns}/{type}/{name}", ...]（可直接经 /assets 静态目录访问，
/// deleteFile 按同形态 URL 删除），而非书仓 entry 列表。
/// - multipart file 字段可多个（字段名不限；无 filename 的表单字段跳过）
/// - type 参数缺省 "images"；"." / ".." / 含路径分隔符 → 文件类型错误
/// - GAP 62：Content-Length 预检 + 字段级大小限额
async fn upload_user_file(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // legacy：type 缺省 images；"."/".."/含分隔符 → 文件类型错误
    let upload_type_raw = param_of(&params, None, "type");
    let upload_type = if upload_type_raw.is_empty() {
        "images"
    } else {
        upload_type_raw.as_str()
    };
    if upload_type == "." || upload_type == ".." || upload_type.contains(['/', '\\']) {
        return Json(ReturnData::err("文件类型错误"));
    }
    // GAP 62：Content-Length 预检 + 字段级限额（DefaultBodyLimit 对 Multipart 只表现为流错误）
    let max_bytes = state.storage.config.upload_max_bytes();
    let max_mb = state.storage.config.upload_max_mb;
    if let Some(msg) = check_upload_content_length(&headers, max_bytes, max_mb) {
        return Json(ReturnData::err(msg));
    }
    let dir = state
        .storage
        .config
        .storage_dir()
        .join("assets")
        .join(&namespace)
        .join(upload_type);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::error!("uploadUserFile 建目录失败 [{}]: {e}", dir.display());
        return Json(ReturnData::err("上传失败"));
    }
    let mut urls: Vec<Value> = Vec::new();
    let mut seen_file = false;
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                let Some(raw_name) = field.file_name().map(str::to_string) else {
                    continue; // 无 filename 的普通表单字段（Vert.x fileUploads 不含）
                };
                seen_file = true;
                let Some(name) = sanitize_upload_filename(&raw_name) else {
                    continue;
                };
                match read_multipart_field_limited(&mut field, max_bytes, max_mb).await {
                    Ok(bytes) => {
                        if std::fs::write(dir.join(&name), &bytes).is_ok() {
                            urls.push(json!(format!("/assets/{namespace}/{upload_type}/{name}")));
                        } else {
                            tracing::error!(
                                "uploadUserFile 写入失败 [{}]",
                                dir.join(&name).display()
                            );
                        }
                    }
                    Err(msg) => return Json(ReturnData::err(msg)),
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::debug!("uploadUserFile multipart 读取失败: {e}");
                break;
            }
        }
    }
    if !seen_file {
        return Json(ReturnData::err("请上传文件"));
    }
    Json(ReturnData::ok(Value::Array(urls)))
}
async fn set_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let new_url = param_of(&params, body_json.as_ref(), "newUrl");
    let source_url = param_of(&params, body_json.as_ref(), "bookSourceUrl");
    if book_url.is_empty() {
        return Json(ReturnData::err("书籍链接不能为空"));
    }
    if new_url.is_empty() {
        return Json(ReturnData::err("新源书籍链接不能为空"));
    }
    if source_url.is_empty() {
        return Json(ReturnData::err("书源链接不能为空"));
    }
    if new_url.is_empty() {
        return Json(ReturnData::err("新源书籍链接不能为空"));
    }
    if source_url.is_empty() {
        return Json(ReturnData::err("书源链接不能为空"));
    }
    let mut book = match state.storage.find_book(&namespace, &book_url).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(ReturnData::err("书籍信息错误")),
        Err(_) => return Json(ReturnData::err("系统错误")),
    };
    // 新书源必须存在（用户源 → default 回退由 resolve_book_source 处理）
    let Some(source) = resolve_book_source(&state, &namespace, &source_url).await else {
        return Json(ReturnData::err("书源信息错误"));
    };
    // 获取新源书籍详情（legacy webBook.getBookInfo(newUrl)；失败 → 书源信息错误）
    let info = match crate::service::book::fetch_book_info(
        &namespace,
        &new_url,
        &source,
        Some(&book.name),
    )
    .await
    {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("setBookSource 获取新书详情失败 [{new_url}]: {e}");
            return Json(ReturnData::err("书源信息错误"));
        }
    };
    let toc_url_new = info
        .toc_url
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| new_url.clone());
    let origin_new = if info.origin.is_empty() {
        source.book_source_url.clone()
    } else {
        info.origin.clone()
    };
    match state
        .storage
        .switch_book_source(
            &namespace,
            &book_url,
            &new_url,
            &origin_new,
            &info.origin_name,
            &toc_url_new,
            info.cover_url.as_deref(),
        )
        .await
    {
        Ok(0) => Json(ReturnData::err("书籍信息错误")),
        Ok(_) => {
            // 内存同步返回值（legacy 返回更新后的 existBook）
            book.origin = origin_new;
            book.origin_name = info.origin_name;
            book.book_url = new_url.clone();
            book.toc_url = toc_url_new.clone();
            if book
                .cover_url
                .as_deref()
                .map(|c| c.is_empty())
                .unwrap_or(true)
            {
                book.cover_url = info.cover_url;
            }
            // 尽力预取新源目录缓存（JAR 语义：刷新失败不影响换源结果）
            match crate::service::book::analyze_toc(
                &namespace,
                &toc_url_new,
                &source,
                20,
                Some(&book.name),
                &new_url,
            )
            .await
            {
                Ok(chapters) => {
                    if let Ok(json) = serde_json::to_string(&chapters) {
                        let _ = state
                            .storage
                            .cache_toc(&namespace, &new_url, &toc_url_new, &json)
                            .await;
                    }
                }
                Err(e) => tracing::warn!("setBookSource 新目录预取失败（忽略）: {e}"),
            }
            Json(ReturnData::ok(
                serde_json::to_value(book).unwrap_or(serde_json::Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("setBookSource 更新失败 [{book_url}]: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}
/// 进程内书籍详情缓存（legacy ACache bookInfoCache 对齐）：
/// 非书架书重复 getBookInfo 时复用上次成功结果（上次成功的书源），避免每次选到不同源。
/// 简单 HashMap + 插入序淘汰（FIFO，容量 200）+ TTL 淘汰。
const BOOK_INFO_CACHE_CAP: usize = 200;
const BOOK_INFO_CACHE_TTL_MS: u64 = 10 * 60 * 1000;

#[derive(Default)]
struct BookInfoCacheStore {
    map: HashMap<String, (crate::model::book_chapter::BookInfo, std::time::Instant)>,
    order: std::collections::VecDeque<String>,
}

impl BookInfoCacheStore {
    /// 命中返回缓存详情；过期条目顺带清除
    fn get(&mut self, ns: &str, url: &str) -> Option<crate::model::book_chapter::BookInfo> {
        let key = format!("{ns}:{url}");
        match self.map.get(&key) {
            Some((info, at)) if at.elapsed().as_millis() <= BOOK_INFO_CACHE_TTL_MS as u128 => {
                Some(info.clone())
            }
            Some(_) => {
                self.map.remove(&key);
                None
            }
            None => None,
        }
    }

    /// 写入缓存；容量满时按插入序淘汰最早条目
    fn put(&mut self, ns: &str, url: &str, info: crate::model::book_chapter::BookInfo) {
        let key = format!("{ns}:{url}");
        if !self.map.contains_key(&key) {
            self.order.push_back(key.clone());
            while self.order.len() > BOOK_INFO_CACHE_CAP {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                }
            }
        }
        self.map.insert(key, (info, std::time::Instant::now()));
    }
}

static BOOK_INFO_CACHE: std::sync::LazyLock<std::sync::Mutex<BookInfoCacheStore>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(BookInfoCacheStore::default()));

/// 命中返回缓存详情；过期条目顺带清除
fn book_info_cache_get(ns: &str, url: &str) -> Option<crate::model::book_chapter::BookInfo> {
    BOOK_INFO_CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(ns, url)
}

/// 写入缓存；容量满时按插入序淘汰最早条目
fn book_info_cache_put(ns: &str, url: &str, info: crate::model::book_chapter::BookInfo) {
    BOOK_INFO_CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .put(ns, url, info)
}

async fn get_book_info(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        // legacy POST 语义：body.searchBook.bookUrl 兜底
        url = body_json
            .as_ref()
            .and_then(|b| b.get("searchBook"))
            .and_then(|s| s.get("bookUrl"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
    }
    if url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接"));
    }
    // 本地书（local:// 或文件路径型）——查书架返回信息，不走书源
    let books = match state.storage.list_books(&namespace).await {
        Ok(b) => b,
        Err(_) => return Json(ReturnData::err("系统错误")),
    };
    let shelf_match = books.iter().find(|b| b.book_url == url);
    if shelf_match.is_some()
        && crate::service::local_book::is_local_book(&url, shelf_match.unwrap().origin.as_str())
    {
        let book = shelf_match.unwrap();
        let info = crate::model::book_chapter::BookInfo {
            name: book.name.clone(),
            author: book.author.clone(),
            kind: book.kind.clone(),
            intro: book.intro.clone(),
            cover_url: book
                .custom_cover_url
                .clone()
                .or_else(|| book.cover_url.clone()),
            toc_url: Some(if book.toc_url.is_empty() {
                book.book_url.clone()
            } else {
                book.toc_url.clone()
            }),
            book_url: book.book_url.clone(),
            origin: book.origin.clone(),
            origin_name: book.origin_name.clone(),
            language: book.language.clone(),
            publisher: book.publisher.clone(),
            published_at: book.published_at.clone(),
            ..Default::default()
        };
        return Json(ReturnData::ok(
            serde_json::to_value(info).unwrap_or(serde_json::Value::Null),
        ));
    }
    if url.starts_with("local://") || url.ends_with(".txt") {
        // 书架无此本地书
        return Json(ReturnData::err("未找到这本书（可能不在书架中）"));
    }
    // 进程内书籍信息缓存（legacy bookInfoCache）：非书架书命中直接返回，
    // 跳过源解析 + 网络请求（同一 URL 复用上次成功的书源）
    if shelf_match.is_none() {
        if let Some(cached) = book_info_cache_get(&namespace, &url) {
            return Json(ReturnData::ok(
                serde_json::to_value(cached).unwrap_or(serde_json::Value::Null),
            ));
        }
    }
    let mut bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    // bookSource 缺失时按 URL 匹配启用书源（bookUrlPattern 正则/域名）——详情页直接访问链接可用
    if bs_param.is_empty() {
        if let Ok(sources) = state.storage.get_book_sources(&namespace).await {
            for s in sources {
                if !s.enabled {
                    continue;
                }
                if let Some(pat) = &s.book_url_pattern {
                    if !pat.is_empty()
                        && crate::util::regex::Regex::new(pat)
                            .map(|r| r.is_match(&url))
                            .unwrap_or(false)
                    {
                        bs_param = s.book_source_url.clone();
                        break;
                    }
                }
            }
        }
    }
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    match crate::service::book::fetch_book_info(
        &namespace,
        &url,
        &source,
        // F12/AR4：书架书解析前已知名 → @get:{bookName} 内建回退（legacy setBook）
        shelf_match.map(|b| b.name.as_str()),
    )
    .await
    {
        Ok(mut info) => {
            // legacy canReName 语义：书源规则未声明 canReName 时保留书架已有
            // 书名/作者（书架书详情刷新/换源不覆盖用户自定义名称）
            if let Some(shelf) = shelf_match {
                if !crate::service::local_book::is_local_book(&url, &shelf.origin) {
                    crate::service::book::merge_existing_identity(
                        &mut info,
                        &source,
                        &shelf.name,
                        &shelf.author,
                    );
                }
            } else {
                // 非书架书：写入进程内详情缓存（下次同 URL 直接复用）
                book_info_cache_put(&namespace, &url, info.clone());
            }
            Json(ReturnData::ok(
                serde_json::to_value(info).unwrap_or(serde_json::Value::Null),
            ))
        }
        Err(e) => {
            tracing::error!("getBookInfo 失败 [{url}]: {e}");
            Json(ReturnData::err("获取详情失败"))
        }
    }
}

/// POST/GET /reader3/getBookToc（= /reader3/getChapterList）：章节目录（ruleToc）
/// 错误文案对齐 legacy getChapterList：请输入书籍链接 / 未配置书源 / 本地书籍源文件不存在
async fn get_book_toc(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url_param = param_of(&params, body_json.as_ref(), "url");
    let mut toc_url = param_of(&params, body_json.as_ref(), "tocUrl");
    if toc_url.is_empty() {
        toc_url = url_param.clone();
    }
    // 换源后 tocUrl 常为空：从书架书 DB 取 toc_url（书源书/本地书均可）兜底
    if toc_url.is_empty() {
        let req_url = param_of(&params, body_json.as_ref(), "url");
        let book_url_param = if !req_url.is_empty() {
            req_url
        } else {
            String::new()
        };
        if !book_url_param.is_empty() {
            if let Ok(Some(b)) = state.storage.find_book(&namespace, &book_url_param).await {
                if !b.toc_url.is_empty() {
                    toc_url = b.toc_url.clone();
                }
            }
        }
    }
    // F8：目录回写目标（书架书优先按 url 参数命中，其次 toc_url 命中）
    let shelf_for_write = {
        let mut found = state
            .storage
            .find_book(&namespace, &url_param)
            .await
            .ok()
            .flatten();
        if found.is_none() {
            found = state
                .storage
                .find_book(&namespace, &toc_url)
                .await
                .ok()
                .flatten();
        }
        found
    };
    if toc_url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接"));
    }
    // 本地书（local://）——不走书源解析
    if toc_url.starts_with("local://") {
        let book_id = toc_url
            .trim_start_matches("local://")
            .split('/')
            .next()
            .unwrap_or("");
        if let Some(ret) =
            get_book_toc_local(&state, &namespace, &format!("local://{book_id}")).await
        {
            return ret;
        }
        return Json(ReturnData::err("本地书籍源文件不存在"));
    }
    // 文件型本地书（legacy：bookUrl = storage/data/.../xx.txt 或任意白名单扩展名）——按扩展名解析分章
    if crate::service::local_book::SUPPORTED_EXTENSIONS
        .iter()
        .any(|e| toc_url.to_lowercase().ends_with(&format!(".{e}")))
    {
        if let Some(ret) = get_book_toc_file(&state, &namespace, &toc_url).await {
            return ret;
        }
        return Json(ReturnData::err("本地书籍源文件不存在"));
    }
    // legacy 本地书（origin=loc_book——toc_url 可能是分章正则或 storage/ 文件路径）——查书架定位文件
    if toc_url.starts_with("storage/")
        || toc_url.starts_with("spin")
        || toc_url.contains("(?")
        || toc_url.contains("序章")
        || toc_url.contains("楔子")
    {
        let mut req_url = param_of(&params, body_json.as_ref(), "url");
        if req_url.is_empty() {
            req_url = toc_url.clone();
        }
        if let Some(ret) = get_book_toc_loc_book(&state, &namespace, &req_url, &toc_url).await {
            return ret;
        }
        return Json(ReturnData::err("本地书籍源文件不存在"));
    }
    // F8：refresh>0 跳过目录缓存（legacy getBookToc refresh 语义）
    let refresh = param_of(&params, body_json.as_ref(), "refresh")
        .parse::<i64>()
        .unwrap_or(0);
    // F-10：目录缓存命中（TTL 5 分钟，同 tocUrl 直读）直接返回，不依赖书源
    if refresh <= 0 {
        if let Ok(Some(cached)) = state
            .storage
            .get_toc_cache(&namespace, &toc_url, TOC_CACHE_TTL_MS)
            .await
        {
            if let Ok(chapters) =
                serde_json::from_str::<Vec<crate::model::book_chapter::BookChapter>>(&cached)
            {
                tracing::debug!("getBookToc 命中目录缓存 [{toc_url}]");
                return Json(ReturnData::ok(
                    serde_json::to_value(chapters).unwrap_or(serde_json::Value::Null),
                ));
            }
        }
    }
    let bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        // legacy getChapterList：非本地书且无可用书源 → 未配置书源
        return Json(ReturnData::err("未配置书源"));
    };
    // F8：目录回写目标书架书——url 参数或 toc_url 对应的书
    let shelf_for_write = state
        .storage
        .find_book(&namespace, &url_param)
        .await
        .ok()
        .flatten();
    match crate::service::book::analyze_toc(
        &namespace,
        &toc_url,
        &source,
        20,
        shelf_for_write.as_ref().map(|b| b.name.as_str()),
        url_param.as_str(),
    )
    .await
    {
        Ok(chapters) => {
            // F-10：抓取成功后缓存目录（book_url 未知时以 toc_url 为键）
            if let Ok(json) = serde_json::to_string(&chapters) {
                let _ = state
                    .storage
                    .cache_toc(&namespace, &toc_url, &toc_url, &json)
                    .await;
            }
            // F8：成功回写 latestChapterTitle/totalChapterNum/lastCheckTime，清 lastCheckError
            if let Some(shelf) = shelf_for_write.as_ref() {
                let latest = chapters.last().map(|c| c.title.clone());
                let mut patch = serde_json::Map::new();
                patch.insert("totalChapterNum".into(), json!(chapters.len() as i64));
                patch.insert("lastCheckTime".into(), json!(now_millis()));
                patch.insert("lastCheckError".into(), json!(Value::Null));
                if let Some(t) = latest {
                    patch.insert("latestChapterTitle".into(), json!(t));
                }
                let _ = state
                    .storage
                    .patch_book(&namespace, &shelf.book_url, &patch)
                    .await;
            }
            Json(ReturnData::ok(
                serde_json::to_value(chapters).unwrap_or(serde_json::Value::Null),
            ))
        }
        Err(e) => {
            // F8：失败记录 lastCheckError（legacy 行为）
            if let Some(shelf) = shelf_for_write.as_ref() {
                let _ = state
                    .storage
                    .patch_book(
                        &namespace,
                        &shelf.book_url,
                        &[("lastCheckError", json!(format!("{e:#}")))]
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.clone()))
                            .collect(),
                    )
                    .await;
            }
            tracing::error!("getBookToc 失败 [{toc_url}]: {e}");
            Json(ReturnData::err("获取目录失败"))
        }
    }
}

/// 书源使用统计：正文抓取成功 → use_count+1 / use_ts 刷新（计数失败仅记 debug，不影响响应）
async fn bump_source_use(state: &AppState, ns: &str, source: &crate::model::BookSource) {
    if let Err(e) = state
        .storage
        .bump_book_source_use(ns, &source.book_source_url)
        .await
    {
        tracing::debug!("书源使用计数失败 [{}]: {e}", source.book_source_name);
    }
}

/// POST/GET /reader3/getBookContent：章节正文（ruleContent）
async fn get_book_content(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let chapter_url = param_of(&params, body_json.as_ref(), "chapterUrl");
    let chapter_url = if chapter_url.is_empty() {
        param_of(&params, body_json.as_ref(), "url")
    } else {
        chapter_url
    };
    // legacy 参数：index（章节索引，正文读取成功时更新书架进度）、cache（1=仅缓存模式不保存进度）
    let index_hint = param_of(&params, body_json.as_ref(), "index")
        .parse::<i64>()
        .ok()
        .filter(|i| *i >= 0);
    let cache_only = param_of(&params, body_json.as_ref(), "cache")
        .parse::<i64>()
        .unwrap_or(0)
        == 1;
    // legacy 参数：epubContent（1=EPUB XHTML 原文模式）——本地 EPUB 返回基本 HTML 结构
    //（纯文本按段落 <p> 包裹；legacy 章节原文 XHTML 的最小对齐，前端 HTML 渲染直接可用）
    let epub_content = param_of(&params, body_json.as_ref(), "epubContent")
        .parse::<i64>()
        .unwrap_or(0)
        == 1;
    if chapter_url.is_empty() {
        return Json(ReturnData::err("请输入章节链接"));
    }
    // 本地书（local://）——不走书源解析
    if chapter_url.starts_with("local://") {
        if let Some(ret) =
            get_book_content_local(&state, &namespace, &chapter_url, epub_content).await
        {
            return ret;
        }
        return Json(ReturnData::err("本地书章节不存在"));
    }
    // legacy 本地书：bookUrl#index（bookUrl 是 storage/ 路径或任意白名单扩展名文件）
    if chapter_url.contains("#") && is_loc_book_file_chapter(&chapter_url) {
        if let Some(ret) =
            get_book_content_file(&state, &namespace, &chapter_url, epub_content).await
        {
            return ret;
        }
        return Json(ReturnData::err("本地书章节不存在"));
    }
    let bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    // ---- 非文本书分派（legacy BookType：0 文本/1 音频/2 漫画/3 文件/4 视频） ----
    // 优先级：客户端显式 type 参数（临时书直读）> 书架书 book_type > 书源 bookSourceType
    let mut book_type = source.book_source_type.clamp(0, 4);
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    // F12/AR4：书架书实体保留——@get:{bookName} 内建回退源（legacy AnalyzeRule.setBook）
    let mut shelf_book: Option<crate::model::book::Book> = None;
    if !book_url.is_empty() && !book_url.starts_with("local://") {
        if let Ok(Some(b)) = state.storage.find_book(&namespace, &book_url).await {
            if b.book_type > 0 {
                book_type = b.book_type.clamp(0, 4);
            }
            shelf_book = Some(b);
        }
    }
    // 章节标题（客户端随请求携带）→ @get:{title} 内建回退源
    let chapter_title_param = param_of(&params, body_json.as_ref(), "title");
    let chapter_title_ctx = if chapter_title_param.is_empty() {
        None
    } else {
        Some(chapter_title_param.as_str())
    };
    let book_name_ctx = shelf_book.as_ref().map(|b| b.name.as_str());
    // E10/AR5：章节/书上下文 seed 到书级变量缓存——analyze_content/analyze_media_url/
    // analyze_comic_images 的 load_book_vars 会带出，push_js_context 透传保留键 →
    // JS 求值绑定 chapter/{title,url,index}/book/{name,author,bookUrl}/nextChapterUrl
    // （legacy AnalyzeRule.setBook/setChapter 对齐；目录缓存反查 index/下一章 URL）
    if !book_url.is_empty() && !book_url.starts_with("local://") {
        let mut ctx =
            crate::parser::rule::load_book_vars(&namespace, &source.book_source_url, &chapter_url);
        ctx.insert(
            crate::parser::rule::RK_CHAPTER_URL.to_string(),
            chapter_url.clone(),
        );
        if let Some(t) = chapter_title_ctx {
            ctx.insert(
                crate::parser::rule::RK_CHAPTER_TITLE.to_string(),
                t.to_string(),
            );
        }
        if let Some(b) = &shelf_book {
            ctx.insert(
                crate::parser::rule::RK_BOOK_NAME.to_string(),
                b.name.clone(),
            );
            if !b.author.is_empty() {
                ctx.insert(
                    crate::parser::rule::RK_BOOK_AUTHOR.to_string(),
                    b.author.clone(),
                );
            }
            ctx.insert(
                crate::parser::rule::RK_BOOK_URL.to_string(),
                b.book_url.clone(),
            );
        }
        if let Some((idx, toc_title, next_url)) =
            lookup_toc_chapter_ctx(&state, &namespace, &book_url, &chapter_url).await
        {
            ctx.insert(
                crate::parser::rule::RK_CHAPTER_INDEX.to_string(),
                idx.to_string(),
            );
            if let Some(t) = toc_title {
                ctx.entry(crate::parser::rule::RK_CHAPTER_TITLE.to_string())
                    .or_insert(t);
            }
            if let Some(n) = next_url {
                ctx.insert(crate::parser::rule::RK_NEXT_CHAPTER_URL.to_string(), n);
            }
        }
        crate::parser::rule::save_book_vars(
            &namespace,
            &source.book_source_url,
            &chapter_url,
            &ctx,
        );
    }
    let type_param = param_of(&params, body_json.as_ref(), "type");
    if let Ok(t) = type_param.parse::<i64>() {
        if (0..=4).contains(&t) {
            book_type = t;
        }
    }
    match book_type {
        // 音频书：ruleContent 提取音频流 URL（或章节 URL 直链）→ {audioUrl, contentType}
        // （m3u8 → application/vnd.apple.mpegurl，前端按此决定 HLS 播放）
        1 => {
            return match crate::service::book::analyze_media_url(
                &namespace,
                &chapter_url,
                &source,
                chapter_title_ctx,
                book_name_ctx,
                &book_url,
            )
            .await
            {
                Ok(url) => {
                    bump_source_use(&state, &namespace, &source).await;
                    Json(ReturnData::ok(serde_json::json!({
                        "audioUrl": url,
                        "contentType": crate::service::book::audio_content_type(&url),
                    })))
                }
                Err(e) => {
                    tracing::error!("getBookContent(音频) 失败 [{chapter_url}]: {e}");
                    Json(ReturnData::err("获取音频失败"))
                }
            };
        }
        // 漫画书：ruleContent 提取图片列表 → {images: [url]}
        2 => {
            return match crate::service::book::analyze_comic_images(
                &namespace,
                &chapter_url,
                &source,
                chapter_title_ctx,
                book_name_ctx,
                &book_url,
            )
            .await
            {
                Ok(images) if !images.is_empty() => {
                    bump_source_use(&state, &namespace, &source).await;
                    Json(ReturnData::ok(serde_json::json!({
                        "images": images,
                    })))
                }
                Ok(_) => Json(ReturnData::err("未提取到图片，请检查书源规则")),
                Err(e) => {
                    tracing::error!("getBookContent(漫画) 失败 [{chapter_url}]: {e}");
                    Json(ReturnData::err("获取图片失败"))
                }
            };
        }
        // 文件书：ruleContent 提取下载链接（或章节 URL）→ {downloadUrl}
        3 => {
            return match crate::service::book::analyze_media_url(
                &namespace,
                &chapter_url,
                &source,
                chapter_title_ctx,
                book_name_ctx,
                &book_url,
            )
            .await
            {
                Ok(url) => {
                    bump_source_use(&state, &namespace, &source).await;
                    Json(ReturnData::ok(serde_json::json!({
                        "downloadUrl": url,
                    })))
                }
                Err(e) => {
                    tracing::error!("getBookContent(文件) 失败 [{chapter_url}]: {e}");
                    Json(ReturnData::err("获取下载链接失败"))
                }
            };
        }
        // 视频书：ruleContent 提取视频 URL（或章节 URL）→ {videoUrl}
        4 => {
            return match crate::service::book::analyze_media_url(
                &namespace,
                &chapter_url,
                &source,
                chapter_title_ctx,
                book_name_ctx,
                &book_url,
            )
            .await
            {
                Ok(url) => {
                    bump_source_use(&state, &namespace, &source).await;
                    Json(ReturnData::ok(serde_json::json!({
                        "videoUrl": url,
                    })))
                }
                Err(e) => {
                    tracing::error!("getBookContent(视频) 失败 [{chapter_url}]: {e}");
                    Json(ReturnData::err("获取视频失败"))
                }
            };
        }
        _ => {}
    }
    // ---- 文本书（type=0）：正文缓存 + ruleContent 文本解析 ----
    // F-10：书源书正文缓存——book_url 为键 + chapter_index = chapterUrl md5 哈希，
    // 同 chapterUrl 直读（永久，清理接口 clearCache 可清）；local:// 键域不参与
    if !book_url.is_empty() && !book_url.starts_with("local://") {
        let idx = crate::util::md5::chapter_url_hash(&chapter_url);
        if let Ok(Some(content)) = state
            .storage
            .get_chapter_content(&namespace, &book_url, idx)
            .await
        {
            if !content.trim().is_empty() {
                tracing::debug!("getBookContent 命中正文缓存 [{book_url} #{idx}]");
                // legacy：缓存命中同样保存书架进度（cache=1 纯缓存模式除外）
                if !cache_only {
                    save_reading_progress_if_shelf(
                        &state,
                        &namespace,
                        &book_url,
                        &chapter_url,
                        index_hint,
                    )
                    .await;
                }
                return Json(ReturnData::ok(serde_json::json!({ "content": content })));
            }
        }
    }
    match crate::service::book::analyze_content(
        &namespace,
        &chapter_url,
        &source,
        5,
        chapter_title_ctx,
        book_name_ctx,
        &book_url,
    )
    .await
    {
        Ok(content) => {
            // 抓取成功 → 写回正文缓存（仅书源书且带 bookUrl；CACHECHAPTERCONTENT=false 时跳过）
            if !book_url.is_empty()
                && !book_url.starts_with("local://")
                && state.storage.config.cache_chapter_content
            {
                let idx = crate::util::md5::chapter_url_hash(&chapter_url);
                let title = param_of(&params, body_json.as_ref(), "title");
                let _ = state
                    .storage
                    .cache_chapter_content(&namespace, &book_url, idx, &title, &content)
                    .await;
                // legacy：正文读取成功 → 自动保存书架进度（cache=1 纯缓存模式除外）
                if !cache_only {
                    save_reading_progress_if_shelf(
                        &state,
                        &namespace,
                        &book_url,
                        &chapter_url,
                        index_hint,
                    )
                    .await;
                }
            }
            // 书源使用统计：正文抓取成功
            bump_source_use(&state, &namespace, &source).await;
            Json(ReturnData::ok(serde_json::json!({ "content": content })))
        }
        Err(e) => {
            tracing::error!("getBookContent 失败 [{chapter_url}]: {e}");
            Json(ReturnData::err("获取正文失败"))
        }
    }
}

/// legacy saveShelfBookProgress：getBookContent 正文读取（或缓存命中）成功后，
/// 若该书在书架上则自动更新 dur_chapter_index/title/time（进度位置保持不动）。
/// 章节索引优先用客户端传入的 index；缺失时从目录缓存按 chapterUrl 反查。
async fn save_reading_progress_if_shelf(
    state: &AppState,
    ns: &str,
    book_url: &str,
    chapter_url: &str,
    index_hint: Option<i64>,
) {
    let Ok(Some(book)) = state.storage.find_book(ns, book_url).await else {
        return;
    };
    let mut index = index_hint.unwrap_or(book.dur_chapter_index);
    let mut title: Option<String> = None;
    // 从目录缓存反查章节 index/title（toc_url == book_url，与 getBookToc 缓存键一致）
    if let Ok(Some(toc_json)) = state
        .storage
        .get_toc_cache(ns, book_url, crate::api::router::TOC_CACHE_TTL_MS)
        .await
    {
        if let Ok(chapters) = serde_json::from_str::<Vec<serde_json::Value>>(&toc_json) {
            for c in chapters {
                let url = c.get("url").and_then(|v| v.as_str()).unwrap_or("");
                if url == chapter_url {
                    if let Some(i) = c.get("index").and_then(|v| v.as_i64()) {
                        index = i;
                    }
                    if let Some(t) = c.get("title").and_then(|v| v.as_str()) {
                        title = Some(t.to_string());
                    }
                    break;
                }
            }
        }
    }
    let _ = state
        .storage
        .update_book_progress(
            ns,
            book_url,
            title.as_deref(),
            index,
            book.dur_chapter_pos,
            now_millis(),
        )
        .await;
}

/// E10/AR5：从目录缓存反查当前章节上下文（index/title/下一章 URL）——
/// getBookContent JS 求值绑定 chapter.{index,url}/nextChapterUrl 用（legacy setChapter）。
/// 目录缓存未命中 → None（绑定为 undefined，与 legacy null 一致）。
async fn lookup_toc_chapter_ctx(
    state: &AppState,
    ns: &str,
    book_url: &str,
    chapter_url: &str,
) -> Option<(i64, Option<String>, Option<String>)> {
    let toc_json = state
        .storage
        .get_toc_cache(ns, book_url, TOC_CACHE_TTL_MS)
        .await
        .ok()
        .flatten()?;
    let chapters = serde_json::from_str::<Vec<serde_json::Value>>(&toc_json).ok()?;
    for (i, c) in chapters.iter().enumerate() {
        let url = c.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if url == chapter_url {
            let index = c.get("index").and_then(|v| v.as_i64()).unwrap_or(i as i64);
            let title = c.get("title").and_then(|v| v.as_str()).map(str::to_string);
            let next = chapters
                .get(i + 1)
                .and_then(|n| n.get("url"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            return Some((index, title, next));
        }
    }
    None
}

// ==================== 差距补全批：导出 / 调试 / 缓存 / 配置 / 刷新 / 批量 / 健康 / 统计 ====================

/// GET /reader3/exportBook：多格式导出（url 单本 + format=txt|epub|html）
/// txt=章节拼接（GAP 104：encoding=utf-8|gbk|gb2312|gb18030 转码）、epub=zip 构造
/// （GAP 176：font=none|lxk-wenkai|source-han-serif 内嵌中文字体）、html=单页
async fn export_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret).into_response(),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接")).into_response();
    }
    let format = param_of(&params, body_json.as_ref(), "format");
    let format = if format.is_empty() {
        "txt"
    } else {
        format.as_str()
    };
    // F10（legacy）：isEpub=1 且未显式指定 format → epub
    let format = if format.is_empty() {
        let is_epub = param_of(&params, body_json.as_ref(), "isEpub");
        if is_epub == "1" {
            "epub"
        } else {
            format
        }
    } else {
        format
    };
    if !matches!(format, "txt" | "epub" | "html") {
        return Json(ReturnData::err("不支持的导出格式（txt|epub|html）")).into_response();
    }
    // GAP 104：TXT 导出编码（utf-8 默认；gbk/gb2312/gb18030；其他格式固定 UTF-8）
    let encoding = param_of(&params, body_json.as_ref(), "encoding");
    // GAP 176：epub 内嵌中文字体（none|lxk-wenkai|source-han-serif；缺省 none；仅 epub 生效）
    let font = match crate::service::export_book::EmbedFont::parse_param(&param_of(
        &params,
        body_json.as_ref(),
        "font",
    )) {
        Ok(f) => f,
        Err(msg) => return Json(ReturnData::err(msg)).into_response(),
    };
    let (title, author, chapters, failed_chapters) = match collect_export_chapters(
        &state,
        &namespace,
        &url,
        &params,
        body_json.as_ref(),
    )
    .await
    {
        Ok(v) => v,
        Err(msg) => return Json(ReturnData::err(msg)).into_response(),
    };
    if chapters.is_empty() {
        return Json(ReturnData::err("没有可导出的章节")).into_response();
    }
    let export_chapters: Vec<crate::service::export_book::ExportChapter> = chapters
        .iter()
        .map(|(t, c)| crate::service::export_book::ExportChapter {
            title: t.clone(),
            content: c.clone(),
        })
        .collect();
    // P2：GBK 不可映射字符转义计数（仅 txt/gbk 路径非 0）——随警告头返回
    let mut unmappable_chars = 0usize;
    let (bytes, mime, ext) = match format {
        // GAP 111：epub 导出携带封面（OPF manifest properties="cover-image" +
        // <meta name="cover"> + OEBPS/cover.{jpg,png} 图片条目）；本地封面直读，
        // 远程封面短超时抓取，失败静默降级为无封面
        // GAP 176：font 参数指定时内嵌中文字体（OPF manifest 字体条目 + style.css
        // @font-face + OEBPS/fonts/*.woff2 + 章节链接样式表）
        "epub" => {
            let cover = book_cover_bytes(&state, &namespace, &url).await;
            let epub = crate::service::export_book::build_epub_full(
                &title,
                &author,
                &crate::service::export_book::EpubMeta {
                    cover,
                    font,
                    ..Default::default()
                },
                &export_chapters,
            );
            (epub, "application/epub+zip".to_string(), "epub")
        }
        "html" => (
            crate::service::export_book::build_html(&title, &export_chapters).into_bytes(),
            "text/html; charset=utf-8".to_string(),
            "html",
        ),
        _ => {
            let txt = crate::service::export_book::build_txt(&title, &export_chapters);
            let (bytes, unmappable) = match crate::service::export_book::encode_txt(&txt, &encoding)
            {
                Ok(v) => v,
                Err(msg) => return Json(ReturnData::err(msg)).into_response(),
            };
            unmappable_chars = unmappable;
            let charset = match encoding.trim().to_ascii_lowercase().as_str() {
                "gbk" | "gb2312" | "gb_2312" => "gbk",
                "gb18030" => "gb18030",
                _ => "utf-8",
            };
            (bytes, format!("text/plain; charset={charset}"), "txt")
        }
    };
    // F10（legacy exportBook）：文件名《书名》作者：xxx；Cache-Control: max-age=300
    let real_author = author.trim();
    let legacy_name = if real_author.is_empty() {
        title.clone()
    } else {
        format!("《{}》作者：{}", title, real_author)
    };
    let filename = sanitize_filename(&legacy_name);
    let filename = if filename.is_empty() {
        "export".to_string()
    } else {
        filename
    };
    // RFC 5987：非 ASCII 文件名百分号编码（HeaderValue 需可见 ASCII）
    let encoded = percent_encode_filename(&filename);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{encoded}.{ext}\""),
        )
        .header("Cache-Control", "max-age=300");
    // P2：导出警告头（X-Export-Warning：percent 编码 JSON，纯 ASCII 可作 HeaderValue）——
    // failedChapters 并发抓章失败列表（不再静默丢弃）；unmappableChars GBK 不可映射
    // 转义计数——前端解析提示
    if !failed_chapters.is_empty() || unmappable_chars > 0 {
        let mut warning = serde_json::Map::new();
        if !failed_chapters.is_empty() {
            warning.insert(
                "failedChapters".to_string(),
                serde_json::to_value(&failed_chapters).unwrap_or(serde_json::Value::Null),
            );
        }
        if unmappable_chars > 0 {
            warning.insert(
                "unmappableChars".to_string(),
                serde_json::json!(unmappable_chars),
            );
        }
        builder = builder.header(
            "X-Export-Warning",
            percent_encode_filename(&serde_json::Value::Object(warning).to_string()),
        );
    }
    builder.body(Body::from(bytes)).unwrap()
}

/// 文件名百分号编码（保留 ASCII 字母数字与 -_. 空格，其余 UTF-8 字节 %XX）
fn percent_encode_filename(name: &str) -> String {
    let mut out = String::new();
    for &b in name.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b' ') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// GAP 111：导出 epub 用封面字节——书架书 cover_url 本地文件（/assets/...）直读；
/// 远程 URL 短超时抓取（5s / 2MB）；任何失败返回 None（导出降级为无封面，不阻塞）
async fn book_cover_bytes(state: &AppState, ns: &str, book_url: &str) -> Option<Vec<u8>> {
    let book = state.storage.find_book(ns, book_url).await.ok().flatten()?;
    let cover_url = book.cover_url.clone()?;
    if cover_url.is_empty() {
        return None;
    }
    if let Some(rel) = cover_url.strip_prefix("/assets/") {
        // 本地封面文件（防穿越：仅接受 assets/{ns}/covers/ 纯文件名）
        let prefix = format!("{ns}/covers/");
        let file = rel.strip_prefix(&prefix)?;
        if file.is_empty() || file.contains('/') || file.contains('\\') || file.contains("..") {
            return None;
        }
        let path = state
            .storage
            .config
            .storage_dir()
            .join("assets")
            .join(ns)
            .join("covers")
            .join(file);
        return std::fs::read(&path).ok();
    }
    if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
        // 远程封面：短超时 + 2MB 上限（失败静默）
        if let Ok((bytes, _, _)) =
            crate::service::crawler::fetch_image(ns, &cover_url, None, 5, 2 * 1024 * 1024).await
        {
            return Some(bytes);
        }
    }
    None
}

/// 收集导出章节：(书名, 作者, [(章节标题, 正文)], 并发抓章失败记录)——
/// 本地书/文件书/书源书统一入口；书源书失败章节逐条记录（P2：不再静默丢弃）
async fn collect_export_chapters(
    state: &AppState,
    ns: &str,
    url: &str,
    params: &HashMap<String, String>,
    body_json: Option<&serde_json::Value>,
) -> Result<
    (
        String,
        String,
        Vec<(String, String)>,
        Vec<crate::service::export_book::FetchChapterFailure>,
    ),
    String,
> {
    let shelf = state.storage.find_book(ns, url).await.ok().flatten();
    let is_local = url.starts_with("local://");
    let is_file = url.starts_with("storage/")
        || crate::service::local_book::SUPPORTED_EXTENSIONS
            .iter()
            .any(|e| url.to_lowercase().ends_with(&format!(".{e}")));

    // ① 本地书（local://）：章节表直读
    if is_local {
        let book = shelf.ok_or_else(|| "书籍不存在（请先加入书架）".to_string())?;
        let rows = state
            .storage
            .list_chapters(url)
            .await
            .map_err(|e| format!("读取章节失败: {e}"))?;
        let mut chapters = Vec::with_capacity(rows.len());
        for (idx, title) in rows {
            let content = state
                .storage
                .get_chapter_content(ns, url, idx)
                .await
                .map_err(|e| format!("读取章节失败: {e}"))?
                .unwrap_or_default();
            chapters.push((title, content));
        }
        return Ok((book.name, book.author, chapters, Vec::new()));
    }
    // ② 文件型本地书：解析原文件（TXT 用用户规则）。P0-7：必须当前 ns 书架归属
    //（防跨用户任意文件读取——url 直传不再放行）
    if is_file {
        let book = shelf.ok_or_else(|| "书籍不存在（请先加入书架）".to_string())?;
        let path = resolve_export_file_path(&state.storage.config.storage_dir(), url)
            .ok_or_else(|| "本地书文件不存在".to_string())?;
        let user_rules = txt_toc_rule_regexes(state, ns).await;
        let imported = crate::service::local_book::parse_loc_book_path(
            &path,
            &user_rules,
            &book.toc_url,
            book.split_long_chapter,
        )
        .map_err(|e| format!("解析失败: {e}"))?;
        let name = if book.name.is_empty() {
            imported.meta.title.clone()
        } else {
            book.name.clone()
        };
        let chapters: Vec<(String, String)> = imported
            .chapters
            .iter()
            .map(|c| (c.title.clone(), c.content.clone()))
            .collect();
        return Ok((name, imported.meta.author, chapters, Vec::new()));
    }
    // ③ 书源书：书架 origin 定位书源（兜底 bookSource 参数）→ 目录 → 逐章正文（优先缓存）
    let book = shelf.ok_or_else(|| "书籍不存在（请先加入书架）".to_string())?;
    let mut source = if !book.origin.is_empty() {
        state
            .storage
            .find_book_source(ns, &book.origin)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    if source.is_none() {
        let bs = param_of(params, body_json, "bookSource");
        if !bs.is_empty() {
            source = resolve_book_source(state, ns, &bs).await;
        }
    }
    let Some(source) = source else {
        return Err("书源不存在".to_string());
    };
    let toc_url = if book.toc_url.is_empty() {
        url.to_string()
    } else {
        book.toc_url.clone()
    };
    let toc = crate::service::book::analyze_toc(ns, &toc_url, &source, 20, Some(&book.name), url)
        .await
        .map_err(|e| format!("获取目录失败: {e}"))?;
    // GAP 104b：书源书导出并发抓章（并发 4——网络抓取是瓶颈；错误章跳过继续；
    // 结果按章节索引重组；章节缓存命中直接复用不抓取）
    let jobs: Vec<(String, String)> = toc
        .iter()
        .filter(|ch| !ch.is_volume)
        .map(|ch| (ch.title.clone(), ch.url.clone()))
        .collect();
    let storage = state.storage.clone();
    let ns_owned = ns.to_string();
    let book_url = url.to_string();
    let src = source.clone();
    let export_book_name = book.name.clone();
    let outcome =
        crate::service::export_book::fetch_chapters_concurrent(jobs, 4, move |_i, chapter_url| {
            let storage = storage.clone();
            let ns = ns_owned.clone();
            let book_url = book_url.clone();
            let src = src.clone();
            let book_name = export_book_name.clone();
            async move {
                let idx = crate::util::md5::chapter_url_hash(&chapter_url);
                match storage
                    .get_chapter_content(&ns, &book_url, idx)
                    .await
                    .ok()
                    .flatten()
                {
                    Some(c) if !c.trim().is_empty() => Ok(c),
                    _ => crate::service::book::analyze_content(
                        &ns,
                        &chapter_url,
                        &src,
                        5,
                        None,
                        Some(&book_name),
                        &book_url,
                    )
                    .await
                    .map_err(|e| format!("获取正文失败: {e}")),
                }
            }
        })
        .await;
    Ok((book.name, book.author, outcome.chapters, outcome.failed))
}

/// 文件名净化（去路径分隔符/非法字符，截断 80 字符）
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .chars()
        .take(80)
        .collect()
}

/// 文件型本地书路径解析（严格防穿越 + legacy 目录式兜底）
pub(crate) fn resolve_export_file_path(
    storage_dir: &std::path::Path,
    book_url: &str,
) -> Option<std::path::PathBuf> {
    resolve_storage_path(storage_dir, book_url)
        .or_else(|| resolve_loc_book_file(storage_dir, book_url))
}

/// GET/POST /reader3/bookSourceDebugSSE：逐规则执行测试（SSE 事件流）
/// 参数：bookSource（必填）+ action=search|explore|toc|content + key + url/chapterUrl
/// 输出：{type:step,message:{ruleName,url,elapsedMs,resultLen,error,detail}} → {type:result,data}
async fn book_source_debug_sse(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return sse_error(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let action = param_of(&params, body_json.as_ref(), "action");
    let key = param_of(&params, body_json.as_ref(), "key");
    let mut target = param_of(&params, body_json.as_ref(), "chapterUrl");
    if target.is_empty() {
        target = param_of(&params, body_json.as_ref(), "url");
    }
    if !matches!(action.as_str(), "search" | "explore" | "toc" | "content") {
        return sse_error(ReturnData::err(
            "请输入调试动作（search|explore|toc|content）",
        ));
    }
    if action == "search" && key.is_empty() {
        return sse_error(ReturnData::err("请输入搜索关键字"));
    }
    let bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        return sse_error(ReturnData::err("书源不存在"));
    };

    let (tx, rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<Bytes, std::convert::Infallible>>();
    let ns = namespace.clone();
    tokio::spawn(async move {
        let send =
            |tx: &tokio::sync::mpsc::UnboundedSender<Result<Bytes, std::convert::Infallible>>,
             payload: &serde_json::Value| {
                let text = format!("data: {payload}\n\n");
                let _ = tx.send(Ok(Bytes::from(text)));
            };
        send(
            &tx,
            &json!({
                "type": "start",
                "message": { "action": action, "bookSource": source.book_source_name },
            }),
        );
        let result =
            crate::service::debug::run_debug(&ns, &source, &action, &key, &target, |step| {
                send(&tx, &json!({ "type": "step", "message": step }));
            })
            .await;
        match result {
            Ok(data) => send(&tx, &json!({ "type": "result", "data": data })),
            Err(e) => send(&tx, &json!({ "type": "error", "message": e.to_string() })),
        }
    });

    let stream = futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// POST /reader3/cacheBookOnServer：后台整书缓存（目录 → 逐章正文 → 缓存表，并发 3）
async fn cache_book_on_server(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    // F3（legacy cacheBookOnServer）：body.bookUrlList 批量契约——逐本启动后台任务
    let batch_urls: Vec<String> = body_json
        .as_ref()
        .and_then(|b| b.get("bookUrlList"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    if url.is_empty() && batch_urls.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    if !batch_urls.is_empty() {
        let mut results: Vec<Value> = Vec::with_capacity(batch_urls.len());
        for u in &batch_urls {
            if state
                .storage
                .find_book(&namespace, u)
                .await
                .ok()
                .flatten()
                .is_none()
            {
                results.push(json!({ "url": u, "error": "书籍不存在" }));
                continue;
            }
            let progress = crate::service::cache_job::start(&namespace, u, state.storage.clone());
            let p = progress.lock().unwrap_or_else(|e| e.into_inner());
            results.push(json!({
                "url": u,
                "started": !p.finished,
                "cached": p.cached,
                "total": p.total,
                "title": p.title,
            }));
        }
        return Json(ReturnData::ok(json!({ "jobs": results })));
    }
    if state
        .storage
        .find_book(&namespace, &url)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return Json(ReturnData::err("书籍不存在（请先加入书架）"));
    }
    let progress = crate::service::cache_job::start(&namespace, &url, state.storage.clone());
    let p = progress.lock().unwrap_or_else(|e| e.into_inner());
    Json(ReturnData::ok(json!({
        "started": !p.finished,
        "url": url,
        "cached": p.cached,
        "total": p.total,
        "title": p.title,
    })))
}

/// POST /reader3/cacheBookRangeOnServer：后台章节范围缓存（目录实章 0 基闭区间）
/// body { url, from, to }；返回 { taskId, started, cached, total, title }
async fn cache_book_range_on_server(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let from = param_of(&params, body_json.as_ref(), "from")
        .parse::<usize>()
        .ok();
    let to = param_of(&params, body_json.as_ref(), "to")
        .parse::<usize>()
        .ok();
    let (Some(from), Some(to)) = (from, to) else {
        return Json(ReturnData::err("缓存范围参数错误"));
    };
    if from > to {
        return Json(ReturnData::err("缓存范围参数错误"));
    }
    if state
        .storage
        .find_book(&namespace, &url)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return Json(ReturnData::err("书籍不存在（请先加入书架）"));
    }
    let (task_id, progress) = crate::service::cache_job::start_range(
        &namespace,
        &url,
        Some((from, to)),
        state.storage.clone(),
    );
    let p = progress.lock().unwrap_or_else(|e| e.into_inner());
    Json(ReturnData::ok(json!({
        "taskId": task_id,
        "started": !p.finished,
        "url": url,
        "cached": p.cached,
        "total": p.total,
        "title": p.title,
    })))
}

/// GET/POST /reader3/getBookCacheChapters：拉取服务器已缓存章节（客户端离线缓存用）
/// body/query { url, from?, to? }；只返回服务器 book_chapters 已缓存且在范围内的章节，
/// 未缓存章节不返回（调用方先跑 cacheBookRangeOnServer 补齐再拉取）。
const MAX_CACHE_CHAPTERS_PER_FETCH: usize = 200;

async fn get_book_cache_chapters(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    if state
        .storage
        .find_book(&namespace, &url)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return Json(ReturnData::err("书籍不存在（请先加入书架）"));
    }
    let from = param_of(&params, body_json.as_ref(), "from")
        .parse::<i64>()
        .ok();
    let to = param_of(&params, body_json.as_ref(), "to")
        .parse::<i64>()
        .ok();
    if let (Some(f), Some(t)) = (from, to) {
        if f > t {
            return Json(ReturnData::err("缓存范围参数错误"));
        }
    }
    let chapters = match state.storage.list_cached_chapters(&namespace, &url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("getBookCacheChapters 失败 [{url}]: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    let mut selected: Vec<serde_json::Value> = Vec::new();
    for (index, title, content) in chapters {
        if let Some(f) = from {
            if index < f {
                continue;
            }
        }
        if let Some(t) = to {
            if index > t {
                continue;
            }
        }
        selected.push(json!({ "index": index, "title": title, "content": content }));
        if selected.len() >= MAX_CACHE_CHAPTERS_PER_FETCH {
            break;
        }
    }
    Json(ReturnData::ok(json!({
        "url": url,
        "chapters": selected,
        "hasMore": selected.len() >= MAX_CACHE_CHAPTERS_PER_FETCH,
    })))
}

/// GET/POST /reader3/cacheBookSSE：缓存进度流 {cached,total,title,finished,error,cancelled}
async fn cache_book_sse(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return sse_error(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    let task_id = param_of(&params, body_json.as_ref(), "taskId");
    let has_task_id = !task_id.is_empty();
    let task_id = if has_task_id { task_id } else { url.clone() };
    if task_id.is_empty() {
        return sse_error(ReturnData::err("参数错误"));
    }
    let progress = match crate::service::cache_job::progress_of_key(&task_id) {
        Some(p) => p,
        None if !has_task_id && !url.is_empty() => {
            // P0（legacy cacheBookSSE 自执行语义）：仅带 url 且无运行中任务时
            // 就地启动整书缓存并流式推送进度——客户端一次调用即完成"启动+监听"
            match state.storage.find_book(&namespace, &url).await {
                Ok(Some(_)) => {}
                Ok(None) => {
                    return sse_error(ReturnData::err("书籍不存在（请先加入书架）"));
                }
                Err(e) => {
                    tracing::error!("cacheBookSSE 查询失败 [{url}]: {e}");
                    return sse_error(ReturnData::err("系统错误"));
                }
            }
            crate::service::cache_job::start(&namespace, &url, state.storage.clone())
        }
        None => return sse_error(ReturnData::err("缓存任务不存在")),
    };

    let (tx, rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<Bytes, std::convert::Infallible>>();
    let progress_for_task = progress.clone();
    tokio::spawn(async move {
        loop {
            let (payload, finished) = {
                let p = progress_for_task.lock().unwrap_or_else(|e| e.into_inner());
                (
                    json!({
                        "cached": p.cached,
                        "total": p.total,
                        "title": p.title,
                        "finished": p.finished,
                        "cancelled": p.cancelled,
                        "error": p.error,
                        // legacy 客户端计数字段别名
                        "cachedCount": p.cached,
                        "successCount": p.cached,
                        "failedCount": 0,
                    }),
                    p.finished,
                )
            };
            let text = format!("data: {payload}\n\n");
            if tx.send(Ok(Bytes::from(text))).is_err() {
                return; // 客户端断开
            }
            if finished {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    });

    let stream = futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// GET/POST /reader3/cancelCacheBook：取消后台缓存任务（内存任务表）
async fn cancel_cache_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    let task_id = param_of(&params, body_json.as_ref(), "taskId");
    let task_id = if !task_id.is_empty() { task_id } else { url };
    if task_id.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let cancelled = crate::service::cache_job::cancel_key(&task_id);
    Json(ReturnData::ok(json!({ "cancelled": cancelled })))
}

/// GET/POST /reader3/getUserConfig：用户配置读取（按用户 + 配置命名空间；key/ns 参数，默认 global）
async fn get_user_config(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut key = param_of(&params, body_json.as_ref(), "key");
    if key.is_empty() {
        key = param_of(&params, body_json.as_ref(), "ns");
    }
    if key.is_empty() {
        key = "global".to_string();
    }
    match state.storage.get_user_config(&namespace, &key).await {
        Ok(Some(raw)) => {
            // 配置为 JSON 文本 → 解析返回（解析失败原样返回字符串）；
            // F12：legacy 裸对象直出（不再包 {ns,config} 一层）
            let data = serde_json::from_str::<serde_json::Value>(&raw)
                .unwrap_or(serde_json::Value::String(raw));
            Json(ReturnData::ok(data))
        }
        // F12：缺配置 → err「没有备份文件」（legacy getUserConfig 行为）
        Ok(None) => Json(ReturnData::err("没有备份文件")),
        Err(e) => {
            tracing::error!("getUserConfig [{namespace}/{key}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveUserConfig：用户配置保存（body：{ns?, config: JSON} 或裸 JSON 整体）
async fn save_user_config(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    // 键：body.ns/key → query → global
    let mut key = json
        .get("ns")
        .or_else(|| json.get("key"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if key.is_empty() {
        key = params
            .get("ns")
            .cloned()
            .unwrap_or_else(|| "global".to_string());
    }
    // 配置：body.config（任意 JSON）→ 序列化；无 config 键则整体为配置。
    // F12（legacy）：注入 @updateTime 时间戳
    let mut config = json.get("config").cloned().unwrap_or(json);
    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "@updateTime".to_string(),
            serde_json::json!(chrono::Utc::now().timestamp_millis()),
        );
    }
    let raw = match &config {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_else(|_| "{}".to_string()),
    };
    match state.storage.save_user_config(&namespace, &key, &raw).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveUserConfig [{namespace}/{key}] 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/refreshLocalBook：重扫本地书（local:// 重解析原文件；文件书重解析）
async fn refresh_local_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP 170：与本地书双轨同步对账互斥（重扫/重链读改写序列串行；对账幂等）
    let sync_lock = crate::service::local_sync::namespace_sync_lock(&namespace);
    let _sync_guard = sync_lock.lock().await;
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let book = match state.storage.find_book(&namespace, &url).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(ReturnData::err("书籍不存在")),
        Err(e) => {
            tracing::error!("refreshLocalBook 查询失败: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    let user_rules = txt_toc_rule_regexes(&state, &namespace).await;
    let source_file: Option<std::path::PathBuf>;
    // GAP 78：loc_book 文件书（storage/ 路径 / 支持扩展名 / origin=loc_book）与 local:// 均支持
    let is_file = book.origin == "loc_book"
        || url.starts_with("storage/")
        || crate::service::local_book::SUPPORTED_EXTENSIONS
            .iter()
            .any(|e| url.to_lowercase().ends_with(&format!(".{e}")));
    let imported = if url.starts_with("local://") {
        // ① 双轨同步关联文件优先（书仓目录 / env READER_LOCAL_BOOK_DIR）
        let linked = book
            .local_file
            .as_ref()
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_file());
        // ② 回退原文件：storage/data/{ns}/opds_files/{id}.{ext}（上传时落盘）
        let id = url
            .trim_start_matches("local://")
            .split('/')
            .next()
            .unwrap_or("")
            .to_string();
        let dir = state
            .storage
            .config
            .storage_dir()
            .join("data")
            .join(&namespace)
            .join("opds_files");
        let mut found = linked;
        if found.is_none() {
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    let stem = p
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let ext_ok =
                        crate::service::local_book::SUPPORTED_EXTENSIONS
                            .iter()
                            .any(|ext| {
                                p.to_string_lossy()
                                    .to_lowercase()
                                    .ends_with(&format!(".{ext}"))
                            });
                    if stem == id && ext_ok {
                        found = Some(p);
                        break;
                    }
                }
            }
        }
        match found {
            Some(path) => {
                source_file = Some(path.clone());
                match crate::service::local_book::parse_loc_book_path(
                    &path,
                    &user_rules,
                    &book.toc_url,
                    book.split_long_chapter,
                ) {
                    Ok(b) => b,
                    Err(e) => return Json(ReturnData::err(format!("解析失败：{e}"))),
                }
            }
            None => return Json(ReturnData::err("本地书原文件不存在")),
        }
    } else if is_file {
        let path = match resolve_export_file_path(&state.storage.config.storage_dir(), &url) {
            Some(p) => p,
            None => return Json(ReturnData::err("本地书文件不存在")),
        };
        source_file = Some(path.clone());
        match crate::service::local_book::parse_loc_book_path(
            &path,
            &user_rules,
            &book.toc_url,
            book.split_long_chapter,
        ) {
            Ok(b) => b,
            Err(e) => return Json(ReturnData::err(format!("解析失败：{e}"))),
        }
    } else {
        return Json(ReturnData::err("仅支持本地书刷新"));
    };
    if imported.chapters.is_empty() {
        return Json(ReturnData::err("未解析到章节内容"));
    }
    let pairs: Vec<(String, String)> = imported
        .chapters
        .iter()
        .map(|c| (c.title.clone(), c.content.clone()))
        .collect();
    if let Err(e) = state.storage.save_chapters(&namespace, &url, &pairs).await {
        tracing::error!("refreshLocalBook 章节入库失败: {e}");
        return Json(ReturnData::err("刷新失败"));
    }
    // 更新 total_chapter_num（书名缺失时用解析出的标题补）
    let mut patch = serde_json::Map::new();
    patch.insert("totalChapterNum".to_string(), json!(pairs.len() as i64));
    if book.name.is_empty() {
        if let Some(path) = &source_file {
            let file_name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let (display_name, _) = local_book_display_meta(
                &file_name,
                &crate::service::local_book::file_ext(&file_name),
                &imported,
            );
            if !display_name.is_empty() {
                patch.insert("name".to_string(), json!(display_name));
            }
        }
    }
    let _ = state.storage.patch_book(&namespace, &url, &patch).await;
    tracing::info!("refreshLocalBook [{namespace}] {url}: {} 章", pairs.len());
    // 返回新 totalChapterNum（GAP 78：前端重扫后展示最新章数）
    Json(ReturnData::ok(json!({
        "bookUrl": url,
        "name": book.name,
        "chapterCount": pairs.len(),
        "totalChapterNum": pairs.len() as i64,
    })))
}

/// POST /reader3/deleteBooks：批量删除（body：{bookUrls:[]}）
async fn delete_books(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let mut urls: Vec<String> = json
        .get("bookUrls")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    // legacy body 形态：Book 对象数组（[Book...]，按 bookUrl 或 name+author 匹配）
    if urls.is_empty() {
        if let Some(arr) = json.as_array() {
            for item in arr {
                if let Some(u) = item.get("bookUrl").and_then(|v| v.as_str()) {
                    if !u.is_empty() {
                        urls.push(u.to_string());
                        continue;
                    }
                }
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let author = item.get("author").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    if let Ok(Some(b)) = state
                        .storage
                        .find_book_by_name_author(&namespace, name, author)
                        .await
                    {
                        urls.push(b.book_url);
                    }
                }
            }
        }
    }
    if urls.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_books(&namespace, &urls).await {
        Ok(count) => Json(ReturnData::ok(json!({ "count": count }))),
        Err(e) => {
            tracing::error!("deleteBooks 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/deleteBookmarks：批量删书签（body：{bookUrl, ids:[]}——ids 为书签标题）
async fn delete_bookmarks(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let book_url = json
        .get("bookUrl")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let ids: Vec<String> = json
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    if book_url.is_empty() || ids.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .delete_bookmarks(&namespace, &book_url, &ids)
        .await
    {
        Ok(count) => Json(ReturnData::ok(json!({ "count": count }))),
        Err(e) => {
            tracing::error!("deleteBookmarks 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/saveRssSources：批量保存 RSS 源（body = 数组）
async fn save_rss_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下 RSS 功能未开启 → 拒绝
    if let Err(ret) = require_rss_permission(&state, &namespace).await {
        return Json(ret);
    }
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let mut sources: Vec<crate::model::RssSource> = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    // legacy：批量导入非法条目（缺 url/name）静默跳过，不整批拒绝
    sources.retain(|s| !s.source_url.trim().is_empty() && !s.source_name.trim().is_empty());
    for s in &mut sources {
        s.user_namespace = namespace.clone();
    }
    match state.storage.save_rss_sources(&namespace, &sources).await {
        Ok(_) => Json(ReturnData::ok(json!({ "count": sources.len() }))),
        Err(e) => {
            tracing::error!("saveRssSources 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/saveBookmarks：批量保存书签（body = 数组）
async fn save_bookmarks(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let mut bookmarks: Vec<crate::model::Bookmark> = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if bookmarks.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    for b in &mut bookmarks {
        if b.book_url.trim().is_empty() || b.title.trim().is_empty() {
            return Json(ReturnData::err("参数错误"));
        }
        b.user_namespace = namespace.clone();
        if b.created_at == 0 {
            b.created_at = now_millis();
        }
    }
    match state.storage.save_bookmarks(&namespace, &bookmarks).await {
        Ok(_) => Json(ReturnData::ok(json!({ "count": bookmarks.len() }))),
        Err(e) => {
            tracing::error!("saveBookmarks 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// 批量分组接口书目解析：兼容 master `bookUrls:[str]` 与 legacy `bookList:[Book]`
/// （Book 对象按 bookUrl 优先、name+author 兜底匹配）
async fn resolve_group_book_urls(
    state: &AppState,
    namespace: &str,
    json: &serde_json::Value,
) -> Vec<String> {
    let mut urls: Vec<String> = json
        .get("bookUrls")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    if urls.is_empty() {
        if let Some(arr) = json.get("bookList").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(u) = item.get("bookUrl").and_then(|v| v.as_str()) {
                    if !u.is_empty() {
                        urls.push(u.to_string());
                        continue;
                    }
                }
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let author = item.get("author").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    if let Ok(Some(b)) = state
                        .storage
                        .find_book_by_name_author(namespace, name, author)
                        .await
                    {
                        urls.push(b.book_url);
                    }
                }
            }
        }
    }
    urls
}

/// POST /reader3/addBookGroupMulti：批量设分组（body：{bookUrls, groupId}）
async fn add_book_group_multi(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let urls = resolve_group_book_urls(&state, &namespace, &json).await;
    let group_id = json.get("groupId").and_then(|v| v.as_i64()).unwrap_or(-1);
    if urls.is_empty() || group_id < 0 {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .add_book_group_multi(&namespace, &urls, group_id)
        .await
    {
        Ok(count) => Json(ReturnData::ok(json!({ "count": count }))),
        Err(e) => {
            tracing::error!("addBookGroupMulti 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/removeBookGroupMulti：批量移出分组（body：{bookUrls, groupId?}；
/// groupId 缺省时清空全部多分组）
async fn remove_book_group_multi(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let urls = resolve_group_book_urls(&state, &namespace, &json).await;
    if urls.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let group_id = json.get("groupId").and_then(|v| v.as_i64());
    match state
        .storage
        .remove_book_group_multi(&namespace, &urls, group_id)
        .await
    {
        Ok(count) => Json(ReturnData::ok(json!({ "count": count }))),
        Err(e) => {
            tracing::error!("removeBookGroupMulti 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/saveBookGroupOrder：分组排序（body：{order:[{id,orderNum}]}）
async fn save_book_group_order(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let order: Vec<(i64, i64)> = json
        .get("order")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    // legacy 契约键为 groupId（group.kt:82-106）；master 自有键 id/orderNum 同收
                    let id = item
                        .get("id")
                        .or_else(|| item.get("groupId"))
                        .and_then(|v| v.as_i64())?;
                    let order_num = item
                        .get("orderNum")
                        .or_else(|| item.get("order"))
                        .and_then(|v| v.as_i64())?;
                    Some((id, order_num))
                })
                .collect()
        })
        .unwrap_or_default();
    if order.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .save_book_group_order(&namespace, &order)
        .await
    {
        Ok(count) => Json(ReturnData::ok(json!({ "count": count }))),
        Err(e) => {
            tracing::error!("saveBookGroupOrder 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// GET/POST /reader3/getAvailableBookSource：换源候选列表（legacy getAvailableBookSource 对齐）
///
/// 参数：url（书籍链接）+ refresh（>0 强制重搜）
/// 行为：
/// - 书架书按 `{name}_{author}` 读取持久化候选；非空且 refresh<=0 → 原样返回
/// - refresh>0 → 按候选 origin 重搜（书名等值过滤）并回写持久化；
///   无候选时回退**全部启用可搜索源**精确搜索（对 legacy 空结果的超集增强——
///   使该端点可独立驱动换源流程，不依赖 searchBookSource 先行）
async fn get_available_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接"));
    }
    let refresh = param_of(&params, body_json.as_ref(), "refresh")
        .parse::<i64>()
        .unwrap_or(0);
    let book = match state.storage.find_book(&namespace, &url).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(ReturnData::err("书籍信息错误")),
        Err(e) => {
            tracing::error!("getAvailableBookSource 查询失败 [{url}]: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    if book.name.is_empty() {
        return Json(ReturnData::err("书籍信息错误"));
    }
    let key = format!("{}_{}", book.name.trim(), book.author.trim());

    // 持久化候选：非刷新模式直接返回
    let cached = state
        .storage
        .get_book_candidates(&namespace, &key)
        .await
        .unwrap_or_default();
    if !cached.is_empty() && refresh <= 0 {
        return Json(ReturnData::ok(
            serde_json::to_value(cached).unwrap_or(serde_json::Value::Null),
        ));
    }

    // 重搜源集合：优先候选 origin 集；无候选 → 全部启用可搜索源（超集增强）
    let origins: std::collections::HashSet<String> =
        cached.iter().map(|c| c.origin.clone()).collect();
    let mut sources: Vec<crate::model::BookSource> =
        match state.storage.get_book_sources(&namespace).await {
            Ok(s) => s
                .into_iter()
                .filter(|s| s.enabled && s.search_url.is_some())
                .collect(),
            Err(_) => return Json(ReturnData::err("系统错误")),
        };
    if !origins.is_empty() {
        sources.retain(|s| origins.contains(&s.book_source_url));
    }
    if sources.is_empty() {
        return Json(ReturnData::ok(serde_json::Value::Array(vec![])));
    }

    // 并发精确搜索（16，同 legacy concurrentCount）
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(16));
    let name = book.name.trim().to_string();
    let ns = namespace.clone();
    let storage = state.storage.clone();
    let mut handles = Vec::with_capacity(sources.len());
    for source in sources {
        let sem = semaphore.clone();
        let key = name.clone();
        let ns = ns.clone();
        let storage = storage.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            crate::service::search::search_one_source(&storage, &ns, &source, &key, 1)
                .await
                .unwrap_or_default()
        }));
    }
    let mut all: Vec<crate::service::search::SearchBook> = Vec::new();
    for h in handles {
        if let Ok(books) = h.await {
            all.extend(books);
        }
    }
    let mut seen = std::collections::HashSet::new();
    let matched: Vec<crate::service::search::SearchBook> = all
        .into_iter()
        .filter(|b| crate::service::search::exact_match(b, &name))
        .filter(|b| seen.insert(b.origin.clone()))
        .collect();

    let _ = state
        .storage
        .save_book_candidates(&namespace, &key, &matched)
        .await;
    Json(ReturnData::ok(
        serde_json::to_value(matched).unwrap_or(serde_json::Value::Null),
    ))
}

/// GET/POST /reader3/getInvalidBookSources：返回运行期失效书源快照（legacy 对齐）
///
/// legacy 语义：搜索/详情/目录/正文实际抓取失败时由对应流程写入失效快照
/// （600 秒 TTL，成功抓取自动清除），本接口直接读快照，不再并发探测全部书源。
/// 响应形状：[{sourceUrl, time, error}]（附 bookSourceUrl/errorMsg 兼容字段）。
async fn get_invalid_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = (body, state);
    let arr: Vec<serde_json::Value> = crate::service::health::invalid_snapshot(&namespace)
        .into_iter()
        .map(|(source_url, time, error)| {
            json!({
                "sourceUrl": source_url,
                "time": time,
                "error": error,
                "errorMsg": error,
                "bookSourceUrl": source_url,
            })
        })
        .collect();
    Json(ReturnData::ok(serde_json::Value::Array(arr)))
}

/// POST /reader3/disableInvalidBookSources：失效书源一键禁用（GAP 140）
///
/// 复用 getInvalidBookSources 的并发检测（HEAD/首页轻量探测，超时 8s），
/// 对判定失效的启用中书源批量 enabled=0；返回 {count, disabled:[bookSourceUrl]}。
async fn disable_invalid_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    _body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let sources = match state.storage.get_book_sources(&namespace).await {
        Ok(s) => s.into_iter().filter(|s| s.enabled).collect::<Vec<_>>(),
        Err(e) => {
            tracing::error!("disableInvalidBookSources [{namespace}] 失败: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    if sources.is_empty() {
        return Json(ReturnData::ok(json!({ "count": 0, "disabled": [] })));
    }
    let invalid = crate::service::health::find_invalid(&namespace, &sources).await;
    // 服务监控：记录最近一次书源检测结果
    crate::service::monitor::record_book_source_check(
        &namespace,
        sources.len() as u64,
        invalid.len() as u64,
    );
    let mut disabled: Vec<String> = Vec::new();
    for (s, reason) in &invalid {
        match state
            .storage
            .update_book_source_enabled(&namespace, &s.book_source_url, false)
            .await
        {
            Ok(_) => {
                disabled.push(s.book_source_url.clone());
                tracing::info!(
                    "GAP 140 禁用失效书源 [{}] {}（{reason}）",
                    namespace,
                    s.book_source_url
                );
            }
            Err(e) => tracing::warn!("禁用书源 {} 失败: {e}", s.book_source_url),
        }
    }
    Json(ReturnData::ok(
        json!({ "count": disabled.len(), "disabled": disabled }),
    ))
}

/// POST /reader3/migrateLocBook：legacy loc_book 文件书 → DB 迁移（GAP 171）
///
/// body：{bookUrl} 单本，或 {all:true} 批量（书架全部 origin=loc_book 文件书）。
/// 行为：文件解析入 book_chapters（键 = 原 book_url）→ books.local_file 关联路径
/// （保留原记录：origin/阅读进度等不动，local_file 非空即迁移标记）→ 目录/正文
/// 改由 DB 直读（getBookToc/getBookContent 命中章节表即不再解析文件）。
/// 封面（文件内嵌）落盘 assets/{ns}/covers 并关联。返回 {migrated, skipped:[{bookUrl,error}]}。
async fn migrate_loc_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json: Option<serde_json::Value> =
        body.as_ref().and_then(|b| serde_json::from_slice(b).ok());
    let all = body_json
        .as_ref()
        .and_then(|b| b.get("all").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if !all && book_url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // 收集目标书
    let books = if all {
        match state.storage.list_loc_book_books(&namespace).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("migrateLocBook 查询失败: {e}");
                return Json(ReturnData::err("系统错误"));
            }
        }
    } else {
        match state.storage.find_book(&namespace, &book_url).await {
            Ok(Some(b)) => vec![b],
            Ok(None) => return Json(ReturnData::err("书籍不存在（请先加入书架）")),
            Err(e) => {
                tracing::error!("migrateLocBook 查询失败: {e}");
                return Json(ReturnData::err("系统错误"));
            }
        }
    };
    let user_rules = txt_toc_rule_regexes(&state, &namespace).await;
    let mut migrated = 0usize;
    let mut skipped: Vec<serde_json::Value> = Vec::new();
    for book in books {
        // 文件定位（storage 路径或白名单扩展名；legacy 目录式 index.epub 兜底）
        let Some(path) = resolve_loc_book_file(&state.storage.config.storage_dir(), &book.book_url)
            .or_else(|| resolve_storage_path(&state.storage.config.storage_dir(), &book.book_url))
        else {
            skipped.push(json!({ "bookUrl": book.book_url, "error": "文件不存在" }));
            continue;
        };
        let imported = match crate::service::local_book::parse_loc_book_path(
            &path,
            &user_rules,
            &book.toc_url,
            book.split_long_chapter,
        ) {
            Ok(i) => i,
            Err(e) => {
                skipped
                    .push(json!({ "bookUrl": book.book_url, "error": format!("解析失败：{e}") }));
                continue;
            }
        };
        if imported.chapters.is_empty() {
            skipped.push(json!({ "bookUrl": book.book_url, "error": "未解析到章节内容" }));
            continue;
        }
        let meta = std::fs::metadata(&path).ok();
        let mtime = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let size = meta.map(|m| m.len() as i64).unwrap_or(0);
        let chapters: Vec<(String, String)> = imported
            .chapters
            .iter()
            .map(|c| (c.title.clone(), c.content.clone()))
            .collect();
        // local_file 存规范化绝对路径（与本地书双轨同步一致）
        let local_file = path.to_string_lossy().replace('\\', "/");
        match state
            .storage
            .migrate_loc_book(
                &namespace,
                &book.book_url,
                Some(&local_file),
                mtime,
                size,
                &chapters,
                imported.cover.as_deref(),
            )
            .await
        {
            Ok(true) => {
                migrated += 1;
                tracing::info!(
                    "GAP 171 迁移 loc_book [{}] {}（{} 章）← {}",
                    namespace,
                    book.book_url,
                    chapters.len(),
                    path.display()
                );
            }
            Ok(false) => skipped.push(json!({ "bookUrl": book.book_url, "error": "书籍不存在" })),
            Err(e) => {
                tracing::error!("migrateLocBook 写库失败 [{}]: {e}", book.book_url);
                skipped
                    .push(json!({ "bookUrl": book.book_url, "error": format!("写库失败：{e}") }));
            }
        }
    }
    Json(ReturnData::ok(
        json!({ "migrated": migrated, "skipped": skipped }),
    ))
}

/// POST /reader3/setAsDefaultBookSources：默认书源标记（body：{bookSources:[url...] 或 [对象...]}）
async fn set_as_default_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = params;
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let urls: Vec<String> = json
        .get("bookSources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    item.as_str().map(str::to_string).or_else(|| {
                        item.get("bookSourceUrl")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    if urls.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .set_default_book_sources(&namespace, &urls)
        .await
    {
        Ok(_) => Json(ReturnData::ok(json!({ "count": urls.len() }))),
        Err(e) => {
            tracing::error!("setAsDefaultBookSources 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// GET/POST /reader3/searchBookSourceSSE：流式换源结果（逐书源事件 + end）
async fn search_book_source_sse(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return sse_error(ret),
    };
    // GAP #58：换源同样受书源权限开关约束
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return sse_error(ret);
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    let book_source_param = param_of(&params, body_json.as_ref(), "bookSource");
    if url.is_empty() {
        return sse_error(ReturnData::err("请输入书籍链接"));
    }
    if book_source_param.is_empty() {
        return sse_error(ReturnData::err("未配置书源"));
    }

    // ① 当前书名（书架优先；未入架走详情解析）
    let name = match state.storage.find_book(&namespace, &url).await {
        Ok(Some(b)) => b.name,
        _ => {
            let Some(source) = resolve_book_source(&state, &namespace, &book_source_param).await
            else {
                return sse_error(ReturnData::err("书源不存在"));
            };
            match crate::service::book::fetch_book_info(&namespace, &url, &source, None).await {
                Ok(info) => {
                    if info.name.is_empty() {
                        return sse_error(ReturnData::err("获取书籍信息失败"));
                    }
                    info.name
                }
                Err(e) => {
                    tracing::error!("searchBookSourceSSE 获取书名失败 [{url}]: {e}");
                    return sse_error(ReturnData::err("获取书籍信息失败"));
                }
            }
        }
    };
    let key = name.trim().to_string();
    if key.is_empty() {
        return sse_error(ReturnData::err("无法获取书名"));
    }

    // ② 全部启用可搜索书源（排除当前源）
    let current = book_source_param.trim();
    let sources: Vec<crate::model::BookSource> =
        match state.storage.get_book_sources(&namespace).await {
            Ok(s) => s
                .into_iter()
                .filter(|s| {
                    s.enabled
                        && s.search_url.is_some()
                        && s.book_source_url != current
                        && s.book_source_name != current
                })
                .collect(),
            Err(_) => return sse_error(ReturnData::err("系统错误")),
        };

    let (tx, rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<Bytes, std::convert::Infallible>>();
    let ns = namespace.clone();
    let storage = state.storage.clone();
    tokio::spawn(async move {
        if sources.is_empty() {
            let payload = json!({ "lastIndex": -1, "isEnd": true });
            let _ = tx.send(Ok(Bytes::from(format!("event: end\ndata: {payload}\n\n"))));
            return;
        }
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(24));
        let mut tasks = futures::stream::FuturesUnordered::new();
        for (i, source) in sources.into_iter().enumerate() {
            let sem = semaphore.clone();
            let key = key.clone();
            let ns = ns.clone();
            let storage = storage.clone();
            tasks.push(Box::pin(async move {
                let _permit = sem.acquire().await;
                let books =
                    crate::service::search::search_one_source(&storage, &ns, &source, &key, 1)
                        .await
                        .unwrap_or_default();
                (i as i64, books)
            }));
        }
        // 汇总：书名匹配过滤 + 按书源去重，逐源推送
        let mut last = -1i64;
        let mut seen = std::collections::HashSet::new();
        let ql = key.to_lowercase();
        while let Some((i, books)) = tasks.next().await {
            last = i;
            let matched: Vec<_> = books
                .into_iter()
                .filter(|b| {
                    let bl = b.name.to_lowercase();
                    bl.contains(&ql) || ql.contains(&bl)
                })
                .filter(|b| seen.insert(b.origin.clone()))
                .collect();
            let payload = json!({ "lastIndex": i, "data": matched });
            if tx
                .send(Ok(Bytes::from(format!("data: {payload}\n\n"))))
                .is_err()
            {
                return; // 客户端断开
            }
        }
        let end_payload = json!({ "lastIndex": last, "isEnd": true });
        let _ = tx.send(Ok(Bytes::from(format!(
            "event: end\ndata: {end_payload}\n\n"
        ))));
    });

    let stream = futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// GET/POST /reader3/getReadingStats：阅读统计（today/week/total 秒数 + 单书 books[]）
async fn get_reading_stats(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = (params, body);
    match state.storage.get_reading_stats(&namespace).await {
        Ok(stats) => Json(ReturnData::ok(
            serde_json::to_value(stats).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getReadingStats [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// 书籍 JSON 输出：附加 `groupIds` 多分组数组（内部 group_ids 存 JSON 文本，不直接序列化）
fn book_json_with_group_ids(book: &crate::model::Book) -> serde_json::Value {
    let mut v = serde_json::to_value(book).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = v.as_object_mut() {
        let ids: Vec<i64> = serde_json::from_str(&book.group_ids).unwrap_or_else(|_| {
            // 旧数据兼容：group_ids 可能存逗号/顿号分隔文本而非 JSON 数组
            book.group_ids
                .split(|c: char| c == ',' || c == '，' || c == '、')
                .filter_map(|s| s.trim().parse::<i64>().ok())
                .collect()
        });
        obj.insert("groupIds".to_string(), serde_json::json!(ids));
    }
    v
}

/// GET /reader3/getBookshelf：按命名空间返回书架（user_namespace 取自 accessToken；非 secure 用 default）
async fn get_bookshelf(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    let refresh = params.get("refresh").map(|v| v == "1").unwrap_or(false);
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // legacy getBookShelfBooks?refresh=1：书架刷新前先回写 can_update=1 的书最新章/总数
    if refresh {
        match crate::storage::run_shelf_update(&state.storage).await {
            Ok(n) => tracing::info!("getBookshelf refresh=1 更新 {n} 本"),
            Err(e) => tracing::warn!("getBookshelf refresh=1 书架更新失败: {e:#}"),
        }
    }
    match state.storage.list_books(&namespace).await {
        Ok(books) => {
            tracing::info!("getBookshelf [{}]: {} 本", namespace, books.len());
            let arr: Vec<serde_json::Value> = books.iter().map(book_json_with_group_ids).collect();
            Json(ReturnData::ok(serde_json::Value::Array(arr)))
        }
        Err(e) => {
            tracing::error!("查询书架 [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// F-13：GET/POST /reader3/getShelfBook：书架单书（url 参数；不存在报“书籍不存在”）
async fn get_shelf_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        return Json(ReturnData::err("书源链接不能为空"));
    }
    match state.storage.find_book(&namespace, &url).await {
        Ok(Some(book)) => Json(ReturnData::ok(book_json_with_group_ids(&book))),
        Ok(None) => Json(ReturnData::err("书籍不存在")),
        Err(e) => {
            tracing::error!("getShelfBook 失败 [{url}]: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// F-25：POST /reader3/logout：退出登录（清 token，token 立即失效）
/// GAP 59：仅移除当前设备 token（token_map 中其余设备 token 保持有效）
async fn logout(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let _ = body;
    // 非 secure 模式无会话概念（legacy：不支持的操作）
    if !state.storage.config.secure {
        return Json(ReturnData::err("不支持的操作"));
    }
    let user = match resolve_current_user(&state, &params, &headers).await {
        Ok(u) => u,
        Err(ret) => return Json(ret),
    };
    let username = user.username;
    // 当前请求的 token（resolve_namespace 校验通过后从同一来源取）
    let token = access_token_of(&params, &headers)
        .and_then(|t| t.split_once(':').map(|(_, tk)| tk.to_string()))
        .unwrap_or_default();
    let result = if token.is_empty() {
        // 兼容：无 token 可定位时回退 legacy 全清
        state.storage.logout_user(&username).await
    } else {
        state.storage.remove_user_token(&username, &token).await
    };
    match result {
        Ok(_) => {
            tracing::info!("用户退出登录: {username}");
            Json(ReturnData::ok(serde_json::Value::Null))
        }
        Err(e) => {
            tracing::error!("logout 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// F-34：POST /reader3/clearInactiveUsers：清理不活跃用户（secure + secureKey 校验）
/// body/query：inactiveDay（默认 0）；简化：仅删 users 行，返回被删用户名列表
async fn clear_inactive_users(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let config = &state.storage.config;
    // 需登录（legacy checkAuth）
    let user = match resolve_current_user(&state, &params, &headers).await {
        Ok(u) => u,
        Err(ret) => return Json(ret),
    };
    let username = user.username;
    // 管理校验（legacy checkManagerAuth）：secure 模式 secureKey，非 secure 模式仅管理员
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if config.secure && !config.secure_key.is_empty() {
        let secure_key = param_of(&params, body_json.as_ref(), "secureKey");
        if !crate::util::constant_time::ct_eq(&secure_key, &config.secure_key) {
            return Json(ReturnData {
                is_success: false,
                error_msg: "请输入管理密码".to_string(),
                data: json!("NEED_SECURE_KEY"),
            });
        }
    } else if !user.is_admin {
        return Json(ReturnData::err("仅管理员可执行该操作"));
    }
    let inactive_day = params
        .get("inactiveDay")
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| {
            body_json
                .as_ref()
                .and_then(|b| b.get("inactiveDay").and_then(|v| v.as_i64()))
        })
        .unwrap_or(0);
    let before = now_millis() - inactive_day * 86400 * 1000;
    match state
        .storage
        .clear_inactive_users(before, Some(&username))
        .await
    {
        Ok(deleted) => {
            tracing::info!(
                "clearInactiveUsers：删除 {} 个不活跃用户: {deleted:?}",
                deleted.len()
            );
            Json(ReturnData::ok(json!({
                "deleted": deleted,
                "count": deleted.len(),
            })))
        }
        Err(e) => {
            tracing::error!("clearInactiveUsers 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// F-32 用户管理：GET/POST /reader3/getUsers：用户列表（含启用状态；secure + secureKey 管理校验）
async fn get_users(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    // 需登录（legacy checkAuth）
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if let Err(ret) = check_manager_auth(&state, &params, &headers, body_json.as_ref()).await {
        return Json(ret);
    }
    match state.storage.list_users().await {
        Ok(users) => {
            let arr: Vec<Value> = users.iter().map(user_admin_json).collect();
            Json(ReturnData::ok(Value::Array(arr)))
        }
        Err(e) => {
            tracing::error!("getUsers 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// 用户管理输出 JSON（不含密码/salt/token；camelCase 兼容 legacy）
fn user_admin_json(user: &User) -> Value {
    json!({
        "username": user.username,
        "enableWebdav": user.enable_webdav,
        "enableLocalStore": user.enable_local_store,
        "enableBookSource": user.enable_book_source,
        "enableRssSource": user.enable_rss_source,
        "bookSourceLimit": user.book_source_limit,
        "bookLimit": user.book_limit,
        "isAdmin": user.is_admin,
        "lastLoginAt": user.last_login_at,
        "createdAt": user.created_at,
    })
}

/// POST /reader3/updateUser：更新用户权限/限额（body/query：username + 可选字段；secureKey）
async fn update_user(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if let Err(ret) = check_manager_auth(&state, &params, &headers, body_json.as_ref()).await {
        return Json(ret);
    }
    let username = param_of(&params, body_json.as_ref(), "username");
    if username.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // 布尔参数：body 布尔值或 query "true"/"1"
    let bool_param = |key: &str| -> Option<bool> {
        if let Some(b) = body_json.as_ref().and_then(|b| b.get(key)) {
            return b.as_bool();
        }
        params.get(key).map(|v| v == "true" || v == "1")
    };
    let int_param = |key: &str| -> Option<i64> {
        if let Some(b) = body_json.as_ref().and_then(|b| b.get(key)) {
            return b.as_i64();
        }
        params.get(key).and_then(|v| v.parse::<i64>().ok())
    };
    // 最后一名管理员禁止撤销管理员身份（保证 default 系统配置始终可管理）
    if bool_param("isAdmin") == Some(false) {
        if let Ok(Some(target)) = state.storage.find_user(&username).await {
            if target.is_admin && state.storage.count_admins().await.unwrap_or(1) <= 1 {
                return Json(ReturnData::err("不能撤销最后一名管理员"));
            }
        }
    }
    match state
        .storage
        .update_user_permissions(
            &username,
            bool_param("enableWebdav"),
            bool_param("enableLocalStore"),
            bool_param("enableBookSource"),
            bool_param("enableRssSource"),
            int_param("bookSourceLimit"),
            int_param("bookLimit"),
            bool_param("isAdmin"),
        )
        .await
    {
        Ok(0) => Json(ReturnData::err("用户不存在")),
        Ok(_) => Json(ReturnData::ok(Value::Null)),
        Err(e) => {
            tracing::error!("updateUser [{username}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/deleteUser：删除用户（secureKey；不能删除自己）
async fn delete_user(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let current_user = match resolve_current_user(&state, &params, &headers).await {
        Ok(u) => u,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if let Err(ret) = check_manager_auth(&state, &params, &headers, body_json.as_ref()).await {
        return Json(ret);
    }
    let username = param_of(&params, body_json.as_ref(), "username");
    if username.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    if username == current_user.username {
        return Json(ReturnData::err("不能删除自己"));
    }
    // 最后一名管理员禁止删除
    if let Ok(Some(target)) = state.storage.find_user(&username).await {
        if target.is_admin && state.storage.count_admins().await.unwrap_or(1) <= 1 {
            return Json(ReturnData::err("不能删除最后一名管理员"));
        }
    }
    match state.storage.delete_user(&username).await {
        Ok(0) => Json(ReturnData::err("用户不存在")),
        Ok(_) => {
            tracing::info!("deleteUser：删除用户 {username}");
            Json(ReturnData::ok(Value::Null))
        }
        Err(e) => {
            tracing::error!("deleteUser [{username}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/deleteUsers：批量删除用户（legacy 对齐）。
/// body：`{"usernames":["a","b"]}`（兼容 legacy 原始字符串数组）；secureKey；不能删除自己；
/// 单事务（全部删除 + 数据清理原子提交）。返回剩余用户列表（legacy deleteUsers 语义）。
async fn delete_users(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let current_user = match resolve_current_user(&state, &params, &headers).await {
        Ok(u) => u,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if let Err(ret) = check_manager_auth(&state, &params, &headers, body_json.as_ref()).await {
        return Json(ret);
    }
    let usernames: Vec<String> = match &body_json {
        // legacy 原始字符串数组
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        Some(obj) => obj
            .get("usernames")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    if usernames.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // 不能删除自己（与 deleteUser 一致）
    let mut targets: Vec<String> = Vec::new();
    let admin_count = state.storage.count_admins().await.unwrap_or(1);
    for u in usernames {
        if u == current_user.username {
            continue;
        }
        // 最后一名管理员不可删（批量中跳过）
        let is_last_admin = state
            .storage
            .find_user(&u)
            .await
            .ok()
            .flatten()
            .map(|x| x.is_admin && admin_count <= 1)
            .unwrap_or(false);
        if !is_last_admin {
            targets.push(u);
        }
    }
    match state.storage.delete_users(&targets).await {
        Ok(n) => {
            tracing::info!("deleteUsers：删除 {n} 个用户");
            match state.storage.list_users().await {
                Ok(users) => {
                    let arr: Vec<Value> = users.iter().map(user_admin_json).collect();
                    Json(ReturnData::ok(Value::Array(arr)))
                }
                Err(e) => {
                    tracing::error!("deleteUsers 后刷新用户列表失败: {e}");
                    Json(ReturnData::err("系统错误"))
                }
            }
        }
        Err(e) => {
            tracing::error!("deleteUsers 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/resetUserPassword：重置用户密码（body/query：username + password/newPassword；secureKey）
async fn reset_user_password(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    if let Err(ret) = check_manager_auth(&state, &params, &headers, body_json.as_ref()).await {
        return Json(ret);
    }
    let username = param_of(&params, body_json.as_ref(), "username");
    let mut password = param_of(&params, body_json.as_ref(), "password");
    if password.is_empty() {
        password = param_of(&params, body_json.as_ref(), "newPassword");
    }
    if username.is_empty() || password.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // 新 salt（与注册一致：8 位随机字母数字；argon2id 哈希自带随机盐，此列仅 legacy 兼容）
    use rand::Rng;
    let salt: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let encrypted = crate::util::password::hash_password(&password);
    match state
        .storage
        .reset_user_password(&username, &salt, &encrypted)
        .await
    {
        Ok(0) => Json(ReturnData::err("用户不存在")),
        Ok(_) => {
            tracing::info!("resetUserPassword：重置用户 {username} 密码");
            Json(ReturnData::ok(Value::Null))
        }
        Err(e) => {
            tracing::error!("resetUserPassword [{username}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// 管理校验（legacy checkManagerAuth）：
/// - secure 模式：secureKey 匹配，失败返回 NEED_SECURE_KEY（errorMsg=请输入管理密码）
/// - 非 secure 模式：仅管理员（is_admin）可执行用户管理，普通用户拒绝
async fn check_manager_auth(
    state: &AppState,
    params: &HashMap<String, String>,
    headers: &HeaderMap,
    body: Option<&serde_json::Value>,
) -> Result<(), ReturnData> {
    let config = &state.storage.config;
    if config.secure && !config.secure_key.is_empty() {
        let secure_key = param_of(params, body, "secureKey");
        if !crate::util::constant_time::ct_eq(&secure_key, &config.secure_key) {
            return Err(ReturnData {
                is_success: false,
                error_msg: "请输入管理密码".to_string(),
                data: json!("NEED_SECURE_KEY"),
            });
        }
        return Ok(());
    }
    // 非 secure（或未配置 secureKey）：区分管理员用户——普通用户不得管理
    match resolve_current_user(state, params, headers).await {
        Ok(u) if u.is_admin => Ok(()),
        Ok(_) => Err(ReturnData::err("仅管理员可执行该操作")),
        Err(ret) => Err(ret),
    }
}

// ---------------- GAP #58 权限开关实际执行 ----------------

/// secure 模式书源功能检查：用户 enable_book_source=0（或用户不存在）→ 拒绝
async fn require_book_source_permission(state: &AppState, ns: &str) -> Result<(), ReturnData> {
    if !state.storage.config.secure {
        return Ok(());
    }
    // 管理员显式进入 default（系统配置层）时放行；本人命名空间按权限开关校验
    if ns == "default" {
        return Ok(());
    }
    match state.storage.find_user(ns).await {
        Ok(Some(u)) if u.enable_book_source => Ok(()),
        Ok(Some(_)) => Err(ReturnData::err("书源功能未开启")),
        _ => Err(login_required()),
    }
}

/// secure 模式 RSS 功能检查：用户 enable_rss_source=0（或用户不存在）→ 拒绝
async fn require_rss_permission(state: &AppState, ns: &str) -> Result<(), ReturnData> {
    if !state.storage.config.secure {
        return Ok(());
    }
    // 管理员显式进入 default（系统配置层）时放行；本人命名空间按权限开关校验
    if ns == "default" {
        return Ok(());
    }
    match state.storage.find_user(ns).await {
        Ok(Some(u)) if u.enable_rss_source => Ok(()),
        Ok(Some(_)) => Err(ReturnData::err("RSS功能未开启")),
        _ => Err(login_required()),
    }
}

// ---------------- F-25 TTS ----------------

/// GET/POST /reader3/getTTSVoices：Edge TTS 可用语音列表（预置 zh-CN/en-US）
/// GAP 113：10 分钟内存缓存（Mutex<Option<(ts, voices)>>——edge_voices_cached）
async fn get_tts_voices(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let _ = (&state, &params, &headers, &body);
    let voices = crate::service::tts::edge_voices_cached();
    let arr: Vec<Value> = voices
        .iter()
        .map(|v| {
            json!({
                "name": v.name,
                "value": v.value,
                "locale": v.locale,
                "gender": v.gender,
            })
        })
        .collect();
    Json(ReturnData::ok(Value::Array(arr)))
}

/// GET/POST /reader3/tts：语音合成
/// 参数：text（必填）、voice（默认 zh-CN-XiaoxiaoNeural）、rate（默认 +0%）、pitch（默认 +0Hz）、
/// volume（默认 +0%）、style（mstts express-as 风格，可选）、
/// engine（edge=Edge 语音 / http=HttpTTS，默认 edge）、url（engine=http 时的 HttpTTS 地址）
/// 成功：audio/mpeg 字节流；失败：ReturnData JSON
async fn tts_synthesize(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret).into_response(),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let text = param_of(&params, body_json.as_ref(), "text");
    if text.trim().is_empty() {
        return Json(ReturnData::err("参数错误")).into_response();
    }
    let engine = param_of(&params, body_json.as_ref(), "engine");
    let voice = param_of(&params, body_json.as_ref(), "voice");
    let voice = if voice.is_empty() {
        crate::service::tts::DEFAULT_VOICE.to_string()
    } else {
        voice
    };
    let rate = param_of(&params, body_json.as_ref(), "rate");
    let rate = if rate.is_empty() {
        "+0%".to_string()
    } else {
        rate
    };
    let pitch = param_of(&params, body_json.as_ref(), "pitch");
    let pitch = if pitch.is_empty() {
        "+0Hz".to_string()
    } else {
        pitch
    };
    let volume = param_of(&params, body_json.as_ref(), "volume");
    let volume = if volume.is_empty() {
        "+0%".to_string()
    } else {
        volume
    };
    let style = param_of(&params, body_json.as_ref(), "style");
    let style = if style.is_empty() { None } else { Some(style) };

    // F4：type 参数（legacy 契约）与 master engine 参数双兼容；textToSpeechCn 暂回退 edge
    let type_param = param_of(&params, body_json.as_ref(), "type");
    let engine_eff = if !type_param.is_empty() {
        type_param
    } else {
        engine.to_string()
    };
    let base64_flag = param_of(&params, body_json.as_ref(), "base64") == "1";
    enum TtsOutcome {
        Bytes(Vec<u8>),
        ApiBytes(Vec<u8>, Option<String>),
    }
    let outcome = match engine_eff.as_str() {
        "edge" => crate::service::tts::edge_synthesize(
            &text,
            &voice,
            &rate,
            &pitch,
            &volume,
            style.as_deref(),
        )
        .await
        .map(TtsOutcome::Bytes),
        "textToSpeechCn" => {
            // A3 真实引擎：POST 表单 → {download} URL → 302 客户端直连（Pro 对齐）。
            // 失败时回退 Edge 合成（保底可用）
            match crate::service::tts::text_to_speech_cn(&text, &voice).await {
                Ok(url) => {
                    // 302 直连源站音频（base64=1 无法转码，Pro 语义一致）
                    return axum::response::Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Location", url)
                        .body(Body::empty())
                        .unwrap();
                }
                Err(e) => {
                    tracing::warn!("textToSpeechCn 合成失败，回退 edge: {e}");
                    crate::service::tts::edge_synthesize(
                        &text,
                        crate::service::tts::DEFAULT_VOICE,
                        "+0%",
                        "+0Hz",
                        "+0%",
                        None,
                    )
                    .await
                    .map(TtsOutcome::Bytes)
                }
            }
        }
        "http" | "httptts" | "api" | _ if false => unreachable!(),
        _ => {
            // legacy type=api：voice = HttpTTS 名称（getHttpTTSByName）
            let Some(tts) = state
                .storage
                .get_http_tts_by_name(&namespace, &voice)
                .await
                .ok()
                .flatten()
            else {
                return Json(ReturnData::err("听书源不存在")).into_response();
            };
            // legacy 语速映射：rate 为 0..1 滑杆 → speechRate=(5+(rate-0.5)*30)
            let rate_f = rate.parse::<f64>().unwrap_or(0.5);
            let speed = (5.0 + (rate_f - 0.5) * 30.0) as i64;
            crate::service::tts::http_tts_api_synthesize(&namespace, &tts, &text, speed)
                .await
                .map(|(bytes, ct)| TtsOutcome::ApiBytes(bytes, ct))
        }
    };

    match outcome {
        Ok(TtsOutcome::Bytes(audio)) => {
            if base64_flag {
                use base64::Engine as _;
                Json(ReturnData::ok(json!(
                    base64::engine::general_purpose::STANDARD.encode(&audio)
                )))
                .into_response()
            } else {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "audio/mpeg")
                    .header("Cache-Control", "no-store")
                    .body(Body::from(audio))
                    .unwrap()
            }
        }
        Ok(TtsOutcome::ApiBytes(audio, ct)) => {
            if base64_flag {
                use base64::Engine as _;
                Json(ReturnData::ok(json!(
                    base64::engine::general_purpose::STANDARD.encode(&audio)
                )))
                .into_response()
            } else {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        "Content-Type",
                        ct.unwrap_or_else(|| "audio/mpeg".to_string()),
                    )
                    .header("Cache-Control", "no-store")
                    .body(Body::from(audio))
                    .unwrap()
            }
        }
        Err(e) => {
            tracing::warn!("tts 合成失败 [{engine_eff}]: {e}");
            Json(ReturnData::err("合成失败")).into_response()
        }
    }
}

/// F-39：POST /reader3/backupToWebdav：书架数据 zip 打包写入
/// storage/data/{ns}/webdav/legado/backup-{ts}.zip（secure 模式需开启 webdav 权限）
async fn backup_to_webdav(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    _body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    if state.storage.config.secure {
        let user = match state.storage.find_user(&namespace).await {
            Ok(Some(u)) => u,
            _ => return Json(ReturnData::err("请登录后使用")),
        };
        if !user.enable_webdav {
            return Json(ReturnData::err("未开启webdav功能"));
        }
    }
    match state.storage.create_backup_zip(&namespace).await {
        Ok(path) => Json(ReturnData::ok(json!({ "path": path }))),
        Err(e) => {
            tracing::error!("backupToWebdav 失败 [{namespace}]: {e}");
            Json(ReturnData::err("备份失败"))
        }
    }
}

/// GET /reader3/user/downloadBackupFile（R5/legacy 对齐）：
/// 创建用户数据备份 zip 并以附件下载返回
async fn download_backup_file(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret).into_response(),
    };
    if state.storage.config.secure {
        let user = match state.storage.find_user(&namespace).await {
            Ok(Some(u)) => u,
            _ => return Json(ReturnData::err("请登录后使用")).into_response(),
        };
        if !user.enable_webdav {
            return Json(ReturnData::err("未开启webdav功能")).into_response();
        }
    }
    match state.storage.create_backup_zip(&namespace).await {
        Ok(path) => match std::fs::read(&path) {
            Ok(bytes) => {
                let fname = std::path::Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "backup.zip".to_string());
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/zip")
                    .header(
                        "Content-Disposition",
                        format!("attachment; filename=\"{fname}\""),
                    )
                    .body(Body::from(bytes))
                    .unwrap()
            }
            Err(e) => {
                tracing::error!("downloadBackupFile 读取失败 [{path}]: {e}");
                Json(ReturnData::err("备份文件读取失败")).into_response()
            }
        },
        Err(e) => {
            tracing::error!("downloadBackupFile 备份创建失败 [{namespace}]: {e}");
            Json(ReturnData::err("备份失败")).into_response()
        }
    }
}

/// 解析 MongoDB 备份参数：body/query 的 uri + db（db 默认 reader3）
fn mongo_backup_params(
    params: &HashMap<String, String>,
    body: Option<&serde_json::Value>,
) -> Result<(String, String), String> {
    let uri = param_of(params, body, "uri");
    if uri.trim().is_empty() {
        if let Some(env) = crate::service::mongodb_backup::env_uri() {
            return Ok((env, param_of(params, body, "db")));
        }
        return Err(
            "未配置MongoDB连接地址（请在请求body传入uri或设置环境变量READER_MONGODB_URI）"
                .to_string(),
        );
    }
    let db = param_of(params, body, "db");
    Ok((uri, db))
}

/// 目标命名空间参数（legacy 语义）：query/body 显式传非空 ns → 仅处理该命名空间；
/// 未传/空白 → 空串，由服务层遍历全部命名空间（default + 全部注册用户）。
fn mongo_backup_ns(params: &HashMap<String, String>, body: Option<&serde_json::Value>) -> String {
    param_of(params, body, "ns").trim().to_string()
}

/// POST /reader3/backupToMongodb：MongoDB 备份（body/query：uri、db、ns）
///
/// legacy 语义：不带 ns → 遍历全部命名空间（default + 全部注册用户）逐个备份；
/// 带 ns → 仅该命名空间。返回单命名空间扁平报告或 {total, failed, namespaces:{...}}。
async fn backup_to_mongodb(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    // 认证解析保持不变（secure 模式校验 accessToken；非 secure 放行）
    if let Err(ret) = resolve_namespace(&state, &params, &headers).await {
        return Json(ret);
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let namespace = mongo_backup_ns(&params, body_json.as_ref());
    let (uri, db) = match mongo_backup_params(&params, body_json.as_ref()) {
        Ok(v) => v,
        Err(msg) => return Json(ReturnData::err(msg)),
    };
    let db = if db.trim().is_empty() {
        crate::service::mongodb_backup::DEFAULT_DB.to_string()
    } else {
        db
    };
    match crate::service::mongodb_backup::backup_to_mongodb(&state.storage, &namespace, &uri, &db)
        .await
    {
        Ok(report) => Json(ReturnData::ok(report)),
        Err(e) => {
            tracing::error!("backupToMongodb 失败 [{namespace}]: {e}");
            Json(ReturnData::err("MongoDB备份失败"))
        }
    }
}

/// POST /reader3/restoreFromMongodb：从 MongoDB 恢复（body/query：uri、db、ns）
///
/// legacy 语义：不带 ns → 遍历全部命名空间逐个恢复；带 ns → 仅该命名空间。
async fn restore_from_mongodb(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    // 认证解析保持不变（secure 模式校验 accessToken；非 secure 放行）
    if let Err(ret) = resolve_namespace(&state, &params, &headers).await {
        return Json(ret);
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let namespace = mongo_backup_ns(&params, body_json.as_ref());
    let (uri, db) = match mongo_backup_params(&params, body_json.as_ref()) {
        Ok(v) => v,
        Err(msg) => return Json(ReturnData::err(msg)),
    };
    let db = if db.trim().is_empty() {
        crate::service::mongodb_backup::DEFAULT_DB.to_string()
    } else {
        db
    };
    match crate::service::mongodb_backup::restore_from_mongodb(
        &state.storage,
        &namespace,
        &uri,
        &db,
    )
    .await
    {
        Ok(report) => Json(ReturnData::ok(report)),
        Err(e) => {
            tracing::error!("restoreFromMongodb 失败 [{namespace}]: {e}");
            Json(ReturnData::err("MongoDB恢复失败"))
        }
    }
}

/// F-55：POST /reader3/restoreFromZip：从备份 zip 恢复（multipart：file=zip + overwrite 字段）
///
/// 返回 {restored: {sources, books, groups, rules, txtRules, rss, config, tts, bookmarks},
///        skipped: {...}}——逐项幂等：已存在且 overwrite=false 时跳过。
async fn restore_from_zip(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let max_bytes = state.storage.config.upload_max_bytes();
    let max_mb = state.storage.config.upload_max_mb;
    // GAP 62：Content-Length 预检（超限 → 明确错误）
    if let Some(msg) = check_upload_content_length(&headers, max_bytes, max_mb) {
        return Json(ReturnData::err(msg));
    }
    let mut bytes: Vec<u8> = Vec::new();
    let mut overwrite = params
        .get("overwrite")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => match field.name() {
                Some("file") => {
                    // GAP 62：显式字段大小上限（超限 → 明确错误）
                    match read_multipart_field_limited(&mut field, max_bytes, max_mb).await {
                        Ok(b) => bytes = b,
                        Err(msg) => return Json(ReturnData::err(msg)),
                    }
                }
                Some("overwrite") => {
                    if let Ok(v) = field.text().await {
                        overwrite = v == "true" || v == "1";
                    }
                }
                _ => {}
            },
            Ok(None) => break,
            Err(e) => {
                tracing::debug!("restoreFromZip multipart 读取失败: {e}");
                break;
            }
        }
    }
    if bytes.is_empty() {
        return Json(ReturnData::err("请上传备份文件"));
    }
    match state
        .storage
        .restore_backup_zip(&namespace, &bytes, overwrite)
        .await
    {
        Ok(report) => Json(ReturnData::ok(
            serde_json::to_value(report).unwrap_or(json!(null)),
        )),
        Err(e) => {
            tracing::error!("restoreFromZip 失败 [{namespace}]: {e}");
            Json(ReturnData::err(format!("恢复失败：{e}")))
        }
    }
}

/// F-55：POST /reader3/restoreFromWebdav：从 webdav 目录读备份 zip 恢复（body {path, overwrite}）
/// path 相对当前用户 webdav 根（如 legado/backup-2024-01-01-120000.zip）；复用 restore_backup_zip 核心
async fn restore_from_webdav(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    if state.storage.config.secure {
        let user = match state.storage.find_user(&namespace).await {
            Ok(Some(u)) => u,
            _ => return Json(ReturnData::err("请登录后使用")),
        };
        if !user.enable_webdav {
            return Json(ReturnData::err("未开启webdav功能"));
        }
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let path = param_of(&params, body_json.as_ref(), "path");
    if path.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let mut overwrite = params
        .get("overwrite")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    if let Some(v) = body_json
        .as_ref()
        .and_then(|b| b.get("overwrite"))
        .and_then(|v| v.as_bool())
    {
        overwrite = v;
    }
    let webdav_root = state
        .storage
        .config
        .storage_dir()
        .join("data")
        .join(&namespace)
        .join("webdav");
    let Some(file) = crate::api::files::resolve_secure_path(&webdav_root, &path) else {
        return Json(ReturnData::err("参数错误"));
    };
    if !file.is_file() {
        return Json(ReturnData::err("路径不存在"));
    }
    let bytes = match tokio::fs::read(&file).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("restoreFromWebdav 读取失败 [{}]: {e}", file.display());
            return Json(ReturnData::err("读取失败"));
        }
    };
    match state
        .storage
        .restore_backup_zip(&namespace, &bytes, overwrite)
        .await
    {
        Ok(report) => Json(ReturnData::ok(
            serde_json::to_value(report).unwrap_or(json!(null)),
        )),
        Err(e) => {
            tracing::error!("restoreFromWebdav 失败 [{namespace}] {path}: {e}");
            Json(ReturnData::err(format!("恢复失败：{e}")))
        }
    }
}

/// 未登录返回（兼容 legacy checkAuth 失败：errorMsg=请登录后使用，data=NEED_LOGIN）
fn login_required() -> ReturnData {
    ReturnData {
        is_success: false,
        error_msg: "请登录后使用".to_string(),
        data: json!("NEED_LOGIN"),
    }
}

/// 解析命名空间：
/// - 非 secure → "default"
/// - secure → 从 query/header 解析 accessToken（username:token）并校验 token，合法则返回用户名
/// GAP 59：主 token 或 users.token_map（多设备）中任一 token 均可通过
pub(crate) async fn resolve_namespace(
    state: &AppState,
    params: &HashMap<String, String>,
    headers: &HeaderMap,
) -> Result<String, ReturnData> {
    if !state.storage.config.secure {
        return Ok("default".to_string());
    }
    let user = resolve_current_user(state, params, headers).await?;
    // 管理员默认使用本人账号命名空间（个人书架/书源等）；
    // 显式传 ns=default 时进入系统配置层（default 公用数据，如公用书源）。
    // 普通用户始终使用本人命名空间，default 只通过回退/覆盖语义生效。
    if user.is_admin && params.get("ns").map(|s| s == "default").unwrap_or(false) {
        return Ok("default".to_string());
    }
    Ok(user.username)
}

/// 解析当前登录用户（secure 模式）：从 query/header 解析 accessToken（username:token）
/// 并校验 token，返回完整用户行（logout/用户管理自检等需要真实用户名的场景用）。
/// GAP 59：主 token 或 users.token_map（多设备）中任一 token 均可通过
pub(crate) async fn resolve_current_user(
    state: &AppState,
    params: &HashMap<String, String>,
    headers: &HeaderMap,
) -> Result<User, ReturnData> {
    if !state.storage.config.secure {
        return Err(login_required());
    }
    let Some(access_token) = access_token_of(params, headers) else {
        return Err(login_required());
    };
    let Some((username, token)) = access_token.split_once(':') else {
        return Err(login_required());
    };
    if username.is_empty() || token.is_empty() {
        return Err(login_required());
    }
    match state.storage.find_user(username).await {
        Ok(Some(user)) => {
            let token_ok = (!user.token.is_empty() && user.token == token)
                || crate::model::user::token_map_valid(&user.token_map, token, now_millis());
            if !token_ok {
                return Err(login_required());
            }
            // GAP 118：token 过期——基于 users.last_login_at + READER_TOKEN_TTL_DAYS（默认 30 天）；
            // 过期（或 legacy 用户 last_login_at=0 从未登录）→ NEED_LOGIN 重新登录；ttl<=0 永不过期
            let ttl_days = state.storage.config.token_ttl_days;
            if ttl_days > 0 && now_millis() - user.last_login_at > ttl_days * 86_400_000 {
                return Err(login_required());
            }
            Ok(user)
        }
        _ => Err(login_required()),
    }
}

/// 从 query/header 提取 accessToken（query → accessToken 头 → Authorization: Bearer）
fn access_token_of(params: &HashMap<String, String>, headers: &HeaderMap) -> Option<String> {
    params
        .get("accessToken")
        .cloned()
        .or_else(|| {
            headers
                .get("accessToken")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.strip_prefix("Bearer ").unwrap_or(v).to_string())
        })
}

/// formatUser：登录/注册返回结构（camelCase，兼容 legacy BaseController.formatUser）
fn format_user(user: &User) -> Value {
    json!({
        "username": user.username,
        "lastLoginAt": user.last_login_at,
        "accessToken": format!("{}:{}", user.username, user.token),
        "enableWebdav": user.enable_webdav,
        "enableLocalStore": user.enable_local_store,
        "enableBookSource": user.enable_book_source,
        "enableRssSource": user.enable_rss_source,
        "bookSourceLimit": user.book_source_limit,
        "bookLimit": user.book_limit,
        "isAdmin": user.is_admin,
        "createdAt": user.created_at,
    })
}

fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

// ---------------- 客户端 IP 解析（P1-2：可信代理白名单） ----------------

/// 可信代理网络（IP 或 CIDR，来自 READER_TRUSTED_PROXIES 环境变量：逗号分隔）
#[derive(Debug, Clone, Copy, PartialEq)]
enum TrustedNet {
    Ip(std::net::IpAddr),
    Cidr { net: std::net::IpAddr, prefix: u8 },
}

impl TrustedNet {
    fn matches(&self, ip: std::net::IpAddr) -> bool {
        match *self {
            TrustedNet::Ip(net) => net == ip,
            TrustedNet::Cidr { net, prefix } => match (ip, net) {
                (std::net::IpAddr::V4(a), std::net::IpAddr::V4(b)) => {
                    let p = prefix.min(32);
                    p == 0
                        || (u32::from(a) & (u32::MAX << (32 - p)))
                            == (u32::from(b) & (u32::MAX << (32 - p)))
                }
                (std::net::IpAddr::V6(a), std::net::IpAddr::V6(b)) => {
                    let p = prefix.min(128);
                    p == 0
                        || (u128::from(a) & (u128::MAX << (128 - p)))
                            == (u128::from(b) & (u128::MAX << (128 - p)))
                }
                _ => false,
            },
        }
    }
}

/// 解析 READER_TRUSTED_PROXIES（逗号分隔 IP/CIDR；非法项忽略）——纯函数可测
fn parse_trusted_proxies(raw: &str) -> Vec<TrustedNet> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| match s.split_once('/') {
            Some((ip, prefix)) => {
                let net = ip.parse::<std::net::IpAddr>().ok()?;
                let prefix = prefix.parse::<u8>().ok()?;
                Some(TrustedNet::Cidr { net, prefix })
            }
            None => s.parse::<std::net::IpAddr>().ok().map(TrustedNet::Ip),
        })
        .collect()
}

/// 可信代理白名单（首次使用时读 env；未配置 = 空表 = 任何 XFF 都不信）
static TRUSTED_PROXIES: std::sync::LazyLock<Vec<TrustedNet>> = std::sync::LazyLock::new(|| {
    parse_trusted_proxies(&std::env::var("READER_TRUSTED_PROXIES").unwrap_or_default())
});

/// 客户端 IP 解析（P1-2）：默认只用直连 IP（ConnectInfo，不可伪造）；
/// X-Forwarded-For 仅当直连 IP 命中 READER_TRUSTED_PROXIES 白名单时取最左项
/// （可信代理链追加的原始客户端地址）。
fn client_ip(peer: &std::net::SocketAddr, headers: &HeaderMap) -> String {
    client_ip_with(peer, headers, &TRUSTED_PROXIES)
}

/// 纯函数版本（测试用）：proxies 显式传入
fn client_ip_with(
    peer: &std::net::SocketAddr,
    headers: &HeaderMap,
    proxies: &[TrustedNet],
) -> String {
    let direct = peer.ip().to_string();
    if !proxies.iter().any(|p| p.matches(peer.ip())) {
        return direct;
    }
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.parse::<std::net::IpAddr>().is_ok())
        .unwrap_or(&direct)
        .to_string()
}

/// 通用错误 JSON（axum 兜底）
pub fn internal_error(err: anyhow::Error) -> axum::response::Response {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "isSuccess": false, "errorMsg": err.to_string(), "data": null })),
    )
        .into_response()
}

/// 命名空间解析（OPDS 认证）：非 secure 模式一律 default；secure 模式支持：
/// ① Basic——独立 OPDS 账号优先（system_settings opds_username/opds_password，sha256+salt），
///    未配置或校验失败回退系统用户账号（users 表，legacy 双 md5 校验）；
/// ② accessToken（query/header，username:token，与 /reader3 一致）。
/// P1-1：Basic 密码校验接入登录限流（用户名+客户端 IP——失败 5 次锁 5 分钟）。
async fn opds_ns(
    state: &AppState,
    headers: &HeaderMap,
    params: &HashMap<String, String>,
    client_ip: &str,
) -> Result<String, Response> {
    if !state.storage.config.secure {
        return Ok("default".to_string());
    }
    // ① Basic 认证
    if let Some(creds) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Basic "))
    {
        let decoded = String::from_utf8(opds_base64_decode(creds)).unwrap_or_default();
        if let Some((username, password)) = decoded.split_once(':') {
            // P1-1：锁定中直接拒绝 Basic 校验（不泄露锁定状态，统一 401）
            if crate::util::login_limit::check_allowed(username, client_ip).is_err() {
                return Err(opds_unauthorized());
            }
            // 独立 OPDS 账号（配置后优先）
            if let Ok(Some((opds_user, stored))) = state.storage.get_opds_account().await {
                if username == opds_user && crate::util::sha256::verify_password(password, &stored)
                {
                    crate::util::login_limit::reset(&opds_user, client_ip);
                    return Ok(opds_user);
                }
            }
            // 系统用户账号（users 表；argon2id 或 legacy 双 md5——统一校验，MD5 通过自动升级）
            if let Ok(Some(user)) = state.storage.find_user(username).await {
                if crate::util::password::verify_password(&state.storage, &user, password).await {
                    crate::util::login_limit::reset(&user.username, client_ip);
                    return Ok(user.username);
                }
            }
            // 密码错误 / 账号不存在 → 计入失败次数（与 /reader3/login 一致）
            crate::util::login_limit::record_failure(username, client_ip);
        }
    }
    // ② accessToken（query/header，username:token，与 /reader3 一致）
    match resolve_namespace(state, params, headers).await {
        Ok(ns) => Ok(ns),
        Err(_) => Err(opds_unauthorized()),
    }
}

/// OPDS 401 响应（WWW-Authenticate: Basic）
fn opds_unauthorized() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic realm=\"reader\"")
        .body(Body::empty())
        .unwrap()
}

/// Basic 凭证解码（标准 base64）
fn opds_base64_decode(s: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s.trim())
        .unwrap_or_default()
}

/// OPDS 统一分发（/opds/*rest）：OPDS 1.2 目录/搜索/获取/下载/保存 + OPDS 2.0 JSON
async fn opds_dispatch(
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    State(state): State<AppState>,
    path: Option<axum::extract::Path<String>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let rest = path.as_deref().map(String::as_str).unwrap_or("");
    // P1-2：客户端 IP（直连优先，可信代理白名单内才信 XFF）——OPDS Basic 限流键
    let ip = client_ip(&peer, &headers);
    let ns = match opds_ns(&state, &headers, &params, &ip).await {
        Ok(ns) => ns,
        Err(resp) => return resp,
    };
    // GAP 52：自引用 base（Host 头优先，缺失回退 localhost:{port}）——所有链接绝对化
    let base = crate::api::opds::opds_base(&headers, state.storage.config.port);
    let (start, max) = crate::api::opds::parse_page(&params);
    let segs: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();

    // OPDS 1.2（Atom）
    let atom = "application/atom+xml;profile=opds-catalog;charset=utf-8";
    // OPDS 2.0（JSON）
    let opds2 = "application/opds+json";

    let make = |r: Result<String, anyhow::Error>, ct: &str| -> Response {
        match r {
            Ok(body) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", ct)
                .body(Body::from(body))
                .unwrap(),
            Err(e) => {
                tracing::error!("OPDS 请求失败 [/opds/{rest}]: {e}");
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap()
            }
        }
    };

    let resp = match segs.as_slice() {
        // ---------------- OPDS 1.2 ----------------
        [] => make(
            crate::api::opds::root(&state.storage, &ns, &base).await,
            "application/atom+xml;profile=opds-catalog;kind=navigation;charset=utf-8",
        ),
        ["opensearch.xml"] => make(
            Ok(crate::api::opds::open_search_xml(&base)),
            "application/opensearchdescription+xml;charset=utf-8",
        ),
        ["shelf"] => make(
            crate::api::opds::shelf(&state.storage, &ns, start, max, &base).await,
            atom,
        ),
        ["recent"] => make(
            crate::api::opds::recent(&state.storage, &ns, start, max, &base).await,
            atom,
        ),
        ["local"] => make(
            crate::api::opds::local(&state.storage, &ns, start, max, &base).await,
            atom,
        ),
        ["groups"] => make(
            crate::api::opds::groups(&state.storage, &ns, &base).await,
            atom,
        ),
        ["group", id] => match id.parse::<i64>() {
            Ok(gid) => make(
                crate::api::opds::group(&state.storage, &ns, gid, start, max, &base).await,
                atom,
            ),
            Err(_) => opds_404(),
        },
        ["source"] => make(
            crate::api::opds::sources(&state.storage, &ns, &base).await,
            atom,
        ),
        ["source", name] => make(
            crate::api::opds::source(&state.storage, &ns, name, start, max, &base).await,
            atom,
        ),
        ["search"] => {
            let q = params.get("q").cloned().unwrap_or_default();
            make(
                crate::api::opds::search(&state.storage, &ns, &q, start, max, &base).await,
                atom,
            )
        }
        // 获取/下载
        ["acquire", id] => match crate::api::opds::acquire(&state.storage, &ns, id).await {
            Ok((name, bytes)) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header(
                    "Content-Disposition",
                    format!("inline; filename=\"{}\"", name),
                )
                .body(Body::from(bytes))
                .unwrap(),
            Err(e) => {
                tracing::warn!("OPDS 正文获取失败: {e}");
                opds_404()
            }
        },
        ["download", id] => {
            let format = params
                .get("format")
                .cloned()
                .unwrap_or_else(|| "txt".to_string());
            let max_chapters = params
                .get("maxChapters")
                .and_then(|v| v.parse::<usize>().ok());
            match crate::api::opds::download(&state.storage, &ns, id, &format, max_chapters).await {
                Ok((name, bytes, ct)) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", ct)
                    .header(
                        "Content-Disposition",
                        format!("attachment; filename=\"{}\"", name),
                    )
                    .body(Body::from(bytes))
                    .unwrap(),
                Err(e) => {
                    tracing::warn!("OPDS 下载失败: {e}");
                    opds_404()
                }
            }
        }
        // OPDS-PSE：GET 进度 entry
        ["save", id] => {
            let want_json = params.get("format").map(|v| v == "json").unwrap_or(false)
                || headers
                    .get(axum::http::header::ACCEPT)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("application/json") && !v.contains("atom"))
                    .unwrap_or(false);
            if want_json {
                match crate::api::opds::save_entry_json(&state.storage, &ns, id).await {
                    Ok(v) => Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json; charset=utf-8")
                        .body(Body::from(v.to_string()))
                        .unwrap(),
                    Err(_) => opds_404(),
                }
            } else {
                match crate::api::opds::save_entry_xml(&state.storage, &ns, id, &base).await {
                    Ok(xml) => Response::builder()
                        .status(StatusCode::OK)
                        .header(
                            "Content-Type",
                            "application/atom+xml;type=entry;charset=utf-8",
                        )
                        .body(Body::from(xml))
                        .unwrap(),
                    Err(_) => opds_404(),
                }
            }
        }
        // ---------------- OPDS 2.0 ----------------
        ["catalog"] => make(
            crate::api::opds::catalog_json(&state.storage, &ns, &base).await,
            opds2,
        ),
        ["catalog", "shelf"] => make(
            crate::api::opds::shelf_json(&state.storage, &ns, start, max, &base).await,
            opds2,
        ),
        ["catalog", "recent"] => make(
            crate::api::opds::recent_json(&state.storage, &ns, start, max, &base).await,
            opds2,
        ),
        ["catalog", "local"] => make(
            crate::api::opds::local_json(&state.storage, &ns, start, max, &base).await,
            opds2,
        ),
        ["catalog", "groups"] => make(
            crate::api::opds::groups_json(&state.storage, &ns, &base).await,
            opds2,
        ),
        ["catalog", "group", id] => match id.parse::<i64>() {
            Ok(gid) => make(
                crate::api::opds::group_json(&state.storage, &ns, gid, start, max, &base).await,
                opds2,
            ),
            Err(_) => opds_404(),
        },
        ["catalog", "source"] => make(
            crate::api::opds::sources_json(&state.storage, &ns, &base).await,
            opds2,
        ),
        ["catalog", "source", name] => make(
            crate::api::opds::source_json(&state.storage, &ns, name, start, max, &base).await,
            opds2,
        ),
        ["catalog", "search"] => {
            let q = params.get("q").cloned().unwrap_or_default();
            make(
                crate::api::opds::search_json(&state.storage, &ns, &q, start, max, &base).await,
                opds2,
            )
        }
        _ => opds_404(),
    };
    resp
}

/// OPDS 404
fn opds_404() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

/// POST /opds/save/{bookId}：OPDS-PSE 保存进度（body/query：progress/position/total/chapterIndex/chapterTitle/timestamp）
async fn opds_save_post(
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    // P1-2：客户端 IP（直连优先，可信代理白名单内才信 XFF）——OPDS Basic 限流键
    let ip = client_ip(&peer, &headers);
    let ns = match opds_ns(&state, &headers, &params, &ip).await {
        Ok(ns) => ns,
        Err(resp) => return resp,
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let id = param_of(&params, body_json.as_ref(), "bookId");
    let f64_of = |keys: &[&str]| -> Option<f64> {
        for k in keys {
            if let Some(v) = params.get(*k).and_then(|v| v.parse::<f64>().ok()) {
                return Some(v);
            }
            if let Some(b) = body_json.as_ref() {
                if let Some(v) = b.get(*k).and_then(|v| v.as_f64()) {
                    return Some(v);
                }
            }
        }
        None
    };
    let i64_of = |keys: &[&str]| -> Option<i64> {
        for k in keys {
            if let Some(v) = params.get(*k).and_then(|v| v.parse::<i64>().ok()) {
                return Some(v);
            }
            if let Some(b) = body_json.as_ref() {
                if let Some(v) = b.get(*k).and_then(|v| v.as_i64()) {
                    return Some(v);
                }
            }
        }
        None
    };
    let str_of = |keys: &[&str]| -> Option<String> {
        for k in keys {
            if let Some(v) = params.get(*k) {
                return Some(v.clone());
            }
            if let Some(b) = body_json.as_ref() {
                if let Some(v) = b.get(*k).and_then(|v| v.as_str()) {
                    return Some(v.to_string());
                }
            }
        }
        None
    };
    let chapter_title = str_of(&["chapterTitle", "durChapterTitle"]);
    match crate::api::opds::apply_save(
        &state.storage,
        &ns,
        &id,
        f64_of(&["progress"]),
        i64_of(&["position", "durChapterPos"]),
        i64_of(&["total"]),
        i64_of(&["chapterIndex", "durChapterIndex"]),
        chapter_title,
        i64_of(&["timestamp", "durChapterTime"]),
    )
    .await
    {
        Ok(v) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(Body::from(v.to_string()))
            .unwrap(),
        Err(e) => {
            tracing::warn!("OPDS-PSE 保存失败: {e}");
            opds_404()
        }
    }
}

/// GET /reader3/getOpdsSettings：OPDS 独立账号配置（enabled/username/passwordSet；不回传密码）
async fn get_opds_settings(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    match state.storage.get_opds_account().await {
        Ok(Some((username, _))) => Json(ReturnData::ok(json!({
            "enabled": true,
            "username": username,
            "passwordSet": true,
            "namespace": namespace,
        }))),
        Ok(None) => Json(ReturnData::ok(json!({
            "enabled": false,
            "username": "",
            "passwordSet": false,
            "namespace": namespace,
        }))),
        Err(e) => {
            tracing::error!("getOpdsSettings 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveOpdsSettings：配置 OPDS 独立账号（body {username, password}；username 空 = 禁用）
async fn save_opds_settings(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let username = body_json
        .as_ref()
        .and_then(|b| b.get("username").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    let password = body_json
        .as_ref()
        .and_then(|b| b.get("password").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    if username.is_empty() {
        // 禁用独立账号（回退系统账号/token）
        match state.storage.clear_opds_account().await {
            Ok(_) => Json(ReturnData::ok(json!({"enabled": false}))),
            Err(e) => {
                tracing::error!("saveOpdsSettings(禁用) 失败: {e}");
                Json(ReturnData::err("系统错误"))
            }
        }
    } else if password.len() < 4 {
        Json(ReturnData::err("密码至少 4 位"))
    } else {
        let stored = crate::util::sha256::store_password(&password);
        match state.storage.set_opds_account(&username, &stored).await {
            Ok(_) => Json(ReturnData::ok(json!({
                "enabled": true,
                "username": username,
            }))),
            Err(e) => {
                tracing::error!("saveOpdsSettings 失败: {e}");
                Json(ReturnData::err("系统错误"))
            }
        }
    }
}

/// POST /reader3/deleteBook：移出书架（bookUrl）
async fn delete_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if book_url.is_empty() {
        book_url = param_of(&params, body_json.as_ref(), "url");
    }
    // legacy：URL 未命中时按 书名+作者 匹配（客户端可能只传 name/author）
    if book_url.is_empty() {
        let name = param_of(&params, body_json.as_ref(), "name");
        let author = param_of(&params, body_json.as_ref(), "author");
        if !name.is_empty() {
            if let Ok(Some(b)) = state
                .storage
                .find_book_by_name_author(&namespace, &name, &author)
                .await
            {
                book_url = b.book_url;
            }
        }
    }
    if book_url.is_empty() {
        return Json(ReturnData::err("书架书籍不存在"));
    }
    match state.storage.delete_book(&namespace, &book_url).await {
        Ok(0) => Json(ReturnData::err("书架书籍不存在")),
        Ok(_) => Json(ReturnData::ok(serde_json::json!("删除书籍成功"))),
        Err(e) => {
            tracing::error!("deleteBook 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// legacy saveBookToShelf 本地书三分支迁移（BookController.kt:3388）：
/// 1. bookUrl 以 `/assets/`/`assets/` 开头 → 上传临时文件（master 落在 storage/assets/**）
/// 2. bookUrl 含 `localStore` → 本地书仓文件（storage/localStore/**）
/// 3. bookUrl 含 `webdav` → webdav 目录文件（storage/data/{ns}/webdav/**）
///
/// 三分支统一语义：源文件存在且不在目标位置时移入
/// `storage/data/{ns}/{name}_{author}/{filename}`，返回新相对路径
/// （`storage/` 前缀形态——resolve_loc_book_file 与 loc_book toc 白名单兼容）。
/// 源不存在/已在目标位置/移动失败 → None（调用方降级保留原路径继续保存）。
fn migrate_local_book_file(
    storage_dir: &std::path::Path,
    namespace: &str,
    name: &str,
    author: &str,
    book_url: &str,
) -> Option<String> {
    let is_temp = book_url.starts_with("/assets/")
        || book_url.starts_with("assets/")
        || book_url.contains("localStore")
        || book_url.contains("webdav");
    if !is_temp {
        return None;
    }
    // 源定位：storage 相对路径（兼容 storage/ 前缀与 / 前缀；防 .. 穿越）
    let trimmed = book_url.trim_start_matches('/');
    let rel = trimmed.strip_prefix("storage/").unwrap_or(trimmed);
    if rel.is_empty() || rel.split(&['/', '\\'][..]).any(|seg| seg == "..") {
        tracing::debug!("saveBook 本地书迁移：非法路径 [{book_url}]");
        return None;
    }
    let src = storage_dir.join(rel);
    if !src.is_file() {
        tracing::debug!("saveBook 本地书迁移：源文件不存在 [{book_url}]");
        return None;
    }
    // 目标目录 {name}_{author}（路径分隔符清洗防穿越）
    let dir_name = format!(
        "{}_{}",
        name.replace(|c: char| c == '/' || c == '\\', "_"),
        author.replace(|c: char| c == '/' || c == '\\', "_")
    );
    let target_dir = storage_dir.join("data").join(namespace).join(&dir_name);
    let filename = src.file_name()?;
    let target = target_dir.join(filename);
    if src == target {
        return None;
    }
    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        tracing::warn!("saveBook 本地书迁移：目录创建失败 [{target_dir:?}] {e}");
        return None;
    }
    // Windows rename 不覆盖已存在目标 → copy 覆盖 + 删源（legacy copyRecursively+deleteRecursively）
    let moved = if target.exists() {
        std::fs::copy(&src, &target)
            .and_then(|_| std::fs::remove_file(&src))
            .map(|_| ())
    } else {
        std::fs::rename(&src, &target).or_else(|_| {
            std::fs::copy(&src, &target)
                .and_then(|_| std::fs::remove_file(&src))
                .map(|_| ())
        })
    };
    if let Err(e) = moved {
        tracing::warn!("saveBook 本地书迁移失败 [{book_url} → {dir_name}]: {e}");
        return None;
    }
    Some(format!(
        "storage/data/{namespace}/{dir_name}/{}",
        filename.to_string_lossy()
    ))
}

/// POST /reader3/saveBook：入架/编辑（完整 Book JSON）
///
/// 语义（对齐 legacy saveBook）：
/// - body = 完整 Book JSON（camelCase，如搜索结果/书架书），bookUrl 必填
/// - 书不在书架 → 全量 INSERT 入架（book_source 校验按任务规格简化为跳过）
/// - 书已在书架 → 按 body 出现的字段增量 UPDATE（未提供字段保持原值，兼容旧版四字段编辑）
async fn save_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let body_json: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let mut book: crate::model::Book = match serde_json::from_value(body_json.clone()) {
        Ok(b) => b,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let mut book_url = if book.book_url.is_empty() {
        param_of(&params, Some(&body_json), "bookUrl")
    } else {
        book.book_url.clone()
    };
    if book_url.is_empty() {
        // legacy saveBookToShelf 文案
        return Json(ReturnData::err("书籍链接不能为空"));
    }

    // 判重：先按 URL，再按 书名+作者（legacy 判重键——同书不同 URL 视为同一本，
    // 换源式保存不产生重复条目/丢进度）
    let mut existing = state
        .storage
        .find_book(&namespace, &book_url)
        .await
        .ok()
        .flatten();
    if existing.is_none() && !book.name.is_empty() {
        existing = state
            .storage
            .find_book_by_name_author(&namespace, &book.name, &book.author)
            .await
            .ok()
            .flatten();
    }
    let exists = existing.is_some();
    // P1-C4：书籍数上限（users.book_limit；limit<=0 不限制；已存在覆盖不计名额）
    if !exists {
        if let Some(limit) = state
            .storage
            .book_limit_for(&namespace)
            .await
            .ok()
            .flatten()
        {
            if limit > 0 {
                let count = match state.storage.count_books_for_user(&namespace).await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("统计书籍数失败: {e}");
                        return Json(ReturnData::err("系统错误"));
                    }
                };
                if count >= limit {
                    return Json(ReturnData::err("你已达到书籍数上限，请联系管理员"));
                }
            }
        }
    }
    // legacy saveBookToShelf 本地书三分支迁移：临时上传/localStore/webdav 文件
    // 移入 data/{ns}/{name}_{author}/ 并重写 bookUrl/tocUrl 为 storage/data 相对路径。
    // 仅新入架或 bookUrl 变化时执行（编辑书名不改路径）；失败降级保留原路径
    let mut loc_migrated = false;
    if (!exists || existing.as_ref().is_some_and(|ex| ex.book_url != book_url))
        && crate::service::local_book::is_local_book(&book_url, &book.origin)
    {
        if let Some(new_url) = migrate_local_book_file(
            &state.storage.config.storage_dir(),
            &namespace,
            &book.name,
            &book.author,
            &book_url,
        ) {
            tracing::info!("saveBook 本地书迁移: {book_url} → {new_url}");
            book_url = new_url.clone();
            book.book_url = new_url.clone();
            book.toc_url = new_url;
            loc_migrated = true;
        }
    }
    let result = if let Some(ex) = existing {
        // 编辑：按 body 出现的字段增量更新。
        // legacy：saveBook 不允许改进度——dur 三字段以库内为准（客户端走 saveBookProgress）
        let mut patch = body_json.as_object().cloned().unwrap_or_default();
        for k in [
            "durChapterIndex",
            "durChapterPos",
            "durChapterTime",
            "durChapterTitle",
        ] {
            patch.remove(k);
        }
        // 迁移成功后 body 内旧临时 tocUrl 不回写（patch 统一为迁移后路径）
        if loc_migrated {
            if let Some(v) = patch.get_mut("tocUrl") {
                *v = serde_json::Value::String(book.toc_url.clone());
            }
        }
        // 跨 URL 保存（换源式）：主键迁移 + 进度保留 + 旧缓存清理
        if ex.book_url != book_url {
            let origin_new = if book.origin.is_empty() {
                &ex.origin
            } else {
                &book.origin
            };
            let origin_name_new = if book.origin_name.is_empty() {
                &ex.origin_name
            } else {
                &book.origin_name
            };
            let toc_new = if book.toc_url.is_empty() {
                book_url.clone()
            } else {
                book.toc_url.clone()
            };
            let _ = state
                .storage
                .switch_book_source(
                    &namespace,
                    &ex.book_url,
                    &book_url,
                    origin_new,
                    origin_name_new,
                    &toc_new,
                    None,
                )
                .await;
        }
        state
            .storage
            .patch_book(&namespace, &book_url, &patch)
            .await
    } else {
        // 新增入架：全量写入；durChapterTime=now 使其按最近阅读排在最前（legacy 语义）
        let mut b = book;
        b.book_url = book_url.clone();
        b.user_namespace = namespace.clone();
        b.is_in_shelf = true;
        if b.created_at == 0 {
            b.created_at = now_millis();
        }
        if b.dur_chapter_time == 0 {
            b.dur_chapter_time = now_millis();
        }
        // F1/P1-16（legacy saveBookCover）：远程封面下载落盘
        // /assets/{ns}/covers/{md5}.{ext}，coverUrl 重写为本地路径
        if let Some(cov) = b.cover_url.clone() {
            if cov.starts_with("http://") || cov.starts_with("https://") {
                match crate::service::crawler::fetch_image(
                    &namespace,
                    &cov,
                    None,
                    15,
                    5 * 1024 * 1024,
                )
                .await
                {
                    Ok((bytes, _, _)) if !bytes.is_empty() => {
                        // 剥掉 URL 查询串再取扩展名：`x.png?token=1` 否则产出含 `?`
                        // 的非法文件名（Windows fs::write 必败 → 封面静默不落盘）
                        let cov_path = cov.split('?').next().unwrap_or(&cov);
                        let ext = crate::service::local_book::file_ext(cov_path);
                        let ext = if ext.is_empty() { "jpg" } else { &ext };
                        let md5 = crate::util::md5::md5_encode(&cov);
                        let dir = state
                            .storage
                            .config
                            .storage_dir()
                            .join("assets")
                            .join(&namespace)
                            .join("covers");
                        if std::fs::create_dir_all(&dir).is_ok() {
                            let fname = format!("{md5}.{ext}");
                            if std::fs::write(dir.join(&fname), &bytes).is_ok() {
                                b.cover_url = Some(format!("/assets/{namespace}/covers/{fname}"));
                            }
                        }
                    }
                    _ => tracing::debug!("saveBook 封面下载失败（保留原 URL）: {cov}"),
                }
            }
        }
        state
            .storage
            .upsert_book(&namespace, &b)
            .await
            .map(|_| 1u64)
    };
    match result {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveBook 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/saveBookProgress：保存阅读进度（body/query：bookUrl + durChapterIndex/durChapterPos/durChapterTime/durChapterTitle；兼容 legacy url/index 命名）
async fn save_book_progress(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if book_url.is_empty() {
        book_url = param_of(&params, body_json.as_ref(), "url");
    }
    if book_url.is_empty() {
        // legacy POST 语义：body.searchBook.bookUrl 兜底
        book_url = body_json
            .as_ref()
            .and_then(|b| b.get("searchBook"))
            .and_then(|s| s.get("bookUrl"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
    }
    if book_url.is_empty() {
        // 进度保存静默：无 bookUrl 时不弹错（前端组件卸载竞态等场景）
        return Json(ReturnData::ok(serde_json::Value::Null));
    }
    let int_of = |keys: &[&str]| -> Option<i64> {
        for k in keys {
            if let Some(v) = params.get(*k).and_then(|v| v.parse::<i64>().ok()) {
                return Some(v);
            }
            if let Some(b) = body_json.as_ref() {
                if let Some(v) = b.get(*k).and_then(|v| v.as_i64()) {
                    return Some(v);
                }
            }
        }
        None
    };
    let index = int_of(&["durChapterIndex", "index"]).unwrap_or(0);
    let pos = int_of(&["durChapterPos"]).unwrap_or(0);
    let time = int_of(&["durChapterTime"]).unwrap_or_else(now_millis);
    // legacy「章节不存在」校验：目录已知（章节表或目录缓存）且 index 越界时拒绝；
    // 两处都未知时不阻塞——legacy 会即时拉取目录，这里放行交由阅读链路自愈
    let mut toc_len: Option<i64> = state
        .storage
        .count_book_chapters(&namespace, &book_url)
        .await
        .ok()
        .filter(|n| *n > 0);
    if toc_len.is_none() {
        if let Ok(Some(toc_json)) = state
            .storage
            .get_toc_cache(&namespace, &book_url, TOC_CACHE_TTL_MS)
            .await
        {
            if let Ok(chapters) = serde_json::from_str::<Vec<serde_json::Value>>(&toc_json) {
                if !chapters.is_empty() {
                    toc_len = Some(chapters.len() as i64);
                }
            }
        }
    }
    if let Some(len) = toc_len {
        if index >= len {
            return Json(ReturnData::err("章节不存在"));
        }
    }
    let title = if params.contains_key("durChapterTitle") {
        params.get("durChapterTitle").cloned()
    } else {
        body_json
            .as_ref()
            .and_then(|b| b.get("durChapterTitle").and_then(|v| v.as_str()))
            .map(str::to_string)
    };
    // 阅读统计：先取旧进度（增量时长/字数），再更新
    let old = match state.storage.find_book(&namespace, &book_url).await {
        Ok(Some(b)) => Some((b.dur_chapter_time, b.dur_chapter_pos)),
        _ => None,
    };
    match state
        .storage
        .update_book_progress(&namespace, &book_url, title.as_deref(), index, pos, time)
        .await
    {
        Ok(0) => Json(ReturnData::err("书籍未加入书架")),
        Ok(_) => {
            // 增量累计阅读时长/字数到 reading_stats（今日行）
            if let Some((old_time, old_pos)) = old {
                let delta_seconds = if old_time > 0 && time > old_time {
                    (time - old_time) / 1000
                } else {
                    0
                };
                let delta_chars = if pos > old_pos { pos - old_pos } else { 0 };
                if delta_seconds > 0 || delta_chars > 0 {
                    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                    if let Err(e) = state
                        .storage
                        .record_reading_stats(
                            &namespace,
                            &book_url,
                            &date,
                            delta_seconds,
                            delta_chars,
                        )
                        .await
                    {
                        tracing::warn!("记录阅读统计失败 [{book_url}]: {e}");
                    }
                }
            }
            Json(ReturnData::ok(serde_json::Value::Null))
        }
        Err(e) => {
            tracing::error!("saveBookProgress 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// GET/POST /reader3/getExploreSources：探索书源列表（精确分类数——parse_explore_entries 执行后计数）
async fn get_explore_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下书源功能未开启 → 拒绝
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return Json(ret);
    }
    let sources = match state.storage.get_book_sources(&namespace).await {
        Ok(s) => s,
        Err(_) => return Json(ReturnData::err("系统错误")),
    };
    let list: Vec<serde_json::Value> = sources
        .iter()
        .filter(|s| s.enabled_explore && s.explore_url.is_some())
        .map(|s| {
            let count = crate::service::explore::parse_explore_entries(
                s.explore_url.as_deref().unwrap_or(""),
            )
            .len();
            serde_json::json!({
                "bookSourceUrl": s.book_source_url,
                "bookSourceName": s.book_source_name,
                "categoryCount": count,
            })
        })
        .filter(|v| v.get("categoryCount").and_then(|c| c.as_u64()).unwrap_or(0) > 0)
        .collect();
    Json(ReturnData::ok(serde_json::Value::Array(list)))
}

/// GET/POST /reader3/getExploreUrls：返回书源的 exploreUrl 集合（bookSource 参数：书源 URL 或完整 JSON）
async fn get_explore_urls(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下书源功能未开启 → 拒绝
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    // legado 语义：exploreUrl 可能是 @js: 代码（执行后返回 [{title,url}]）或普通 URL 集合
    let raw = source.explore_url.as_deref().unwrap_or("");
    let entries = crate::service::explore::parse_explore_entries(raw);
    Json(ReturnData::ok(
        serde_json::to_value(entries).unwrap_or(serde_json::Value::Null),
    ))
}

/// GET/POST /reader3/exploreBook：探索/书海（url=ruleFindUrl + bookSource + page）
///
/// GAP #51：page 参数由服务端替换书源分页变量（{{page}}/{page}）；
/// 响应形状对齐 legacy BookController.exploreBook：data = SearchBook 纯数组
/// （legacy WebBook.exploreBook 返回 List<SearchBook> 原样 setData）
async fn explore_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    // GAP #58：secure 模式下书源功能未开启 → 拒绝
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return Json(ret);
    }
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    let url = if url.is_empty() {
        param_of(&params, body_json.as_ref(), "ruleFindUrl")
    } else {
        url
    };
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let page: i64 = params
        .get("page")
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            body_json
                .as_ref()
                .and_then(|b| b.get("page").and_then(|v| v.as_i64()))
        })
        .unwrap_or(1);
    let bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    match crate::service::explore::explore_url(&namespace, &url, page, &source).await {
        Ok(books) => Json(ReturnData::ok(
            serde_json::to_value(&books).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("exploreBook 失败 [{url}]: {e}");
            Json(ReturnData::err(format!("探索失败：{e}")))
        }
    }
}

/// SSE 并发数生效值：缺省 24（对齐 legacy searchBookMultiSSE concurrentCount 默认），
/// 显式传值 clamp 到 1..=128（防止客户端传超大值打爆连接数）
fn effective_concurrent_count(v: Option<usize>) -> usize {
    v.unwrap_or(24).clamp(1, 128)
}

/// GET/POST /reader3/searchBookMultiSSE：多书源流式搜索（SSE）
///
/// 参数：key/bookSourceGroup/lastIndex/searchSize/concurrentCount（POST body 或 GET query）
/// 输出：逐源无名 data 事件 {"lastIndex","data":[SearchBook]}（legacy 对齐：旧客户端用
/// onmessage 接收，不带 event 名），结束 `event: end`；校验失败输出 `event: error`
async fn search_book_multi_sse(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Response {
    // 参数解析（POST body JSON 优先，GET query 兜底）
    let mut key = params.get("key").cloned().unwrap_or_default();
    // legacy "=" 前缀 = 精确搜索（accurate）
    let mut key_exact_prefix = false;
    if key.starts_with('=') {
        key_exact_prefix = true;
        key.remove(0);
    }
    let mut group = params.get("bookSourceGroup").cloned().unwrap_or_default();
    // P1-4 单源指定：精确匹配 bookSourceUrl（非空时只搜该源）
    let mut single_source_url = params.get("bookSourceUrl").cloned().unwrap_or_default();
    let mut exact = params.get("exact").map(|v| v == "1").unwrap_or(false);
    let mut last_index = params
        .get("lastIndex")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(-1);
    let mut search_size = params
        .get("searchSize")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50);
    let mut concurrent_count = params
        .get("concurrentCount")
        .and_then(|v| v.parse::<usize>().ok());
    if let Some(body) = body {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
            if let Some(v) = json.get("key").and_then(|v| v.as_str()) {
                key = v.to_string();
            }
            if let Some(v) = json.get("bookSourceGroup").and_then(|v| v.as_str()) {
                group = v.to_string();
            }
            if let Some(v) = json.get("bookSourceUrl").and_then(|v| v.as_str()) {
                single_source_url = v.to_string();
            }
            if let Some(v) = json.get("exact") {
                exact = v.as_u64() == Some(1)
                    || v.as_str()
                        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
            }
            if let Some(v) = json.get("lastIndex").and_then(|v| v.as_i64()) {
                last_index = v;
            }
            if let Some(v) = json.get("searchSize").and_then(|v| v.as_u64()) {
                search_size = v as usize;
            }
            if let Some(v) = json.get("concurrentCount").and_then(|v| v.as_u64()) {
                concurrent_count = Some(v as usize);
            }
        }
    }
    // legacy "=" 前缀强制精确
    if key_exact_prefix {
        exact = true;
    }

    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return sse_error(ret),
    };
    // GAP #58：secure 模式下书源功能未开启 → 拒绝
    if let Err(ret) = require_book_source_permission(&state, &namespace).await {
        return sse_error(ret);
    }
    if key.is_empty() {
        return sse_error(ReturnData::err("请输入搜索关键字"));
    }
    let sources = match state.storage.get_book_sources(&namespace).await {
        Ok(s) => s,
        Err(_) => return sse_error(ReturnData::err("系统错误")),
    };
    // 换源语义：600s 内已标记失效的源直接跳过（搜索必失败，不进结果列表——
    // 用户反馈「失败的源就没必要显示了」）
    let sources: Vec<crate::model::BookSource> = sources
        .into_iter()
        .filter(|s| {
            s.enabled
                && s.search_url.is_some()
                && !crate::service::health::is_source_invalid(&namespace, &s.book_source_url)
        })
        .filter(|s| book_source_group_matches(&group, s.book_source_group.as_deref()))
        .filter(|s| single_source_url.is_empty() || s.book_source_url == single_source_url)
        .collect();
    if sources.is_empty() {
        return sse_error(ReturnData::err("未配置书源"));
    }
    if last_index >= sources.len() as i64 - 1 {
        return sse_error(ReturnData::err("没有更多了"));
    }
    search_size = search_size.max(1);
    let concurrent_count = effective_concurrent_count(concurrent_count);

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::convert::Infallible>>(
        concurrent_count.min(64).max(4),
    );
    let total = sources.len() as i64;
    let start = (last_index + 1).max(0) as usize;
    let end = (start + search_size).min(sources.len());
    let ns = namespace.clone();
    let storage = state.storage.clone();
    tokio::spawn(async move {
        // 并发受控（semaphore），结果到达即推送（FuturesUnordered 完成顺序）
        let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrent_count));
        let mut tasks = futures::stream::FuturesUnordered::new();
        for i in start..end {
            let sem = sem.clone();
            let key = key.clone();
            let ns = ns.clone();
            let storage = storage.clone();
            let source = sources[i].clone();
            tasks.push(Box::pin(async move {
                let _permit = sem.acquire().await;
                let books =
                    crate::service::search::search_one_source(&storage, &ns, &source, &key, 1)
                        .await
                        .unwrap_or_default();
                // 精确模式（exact=1）：书源规则解析后按书名/作者等值过滤（大小写/全半角忽略）
                let books = if exact {
                    crate::service::search::filter_exact(books, &key)
                } else {
                    books
                };
                // 跨书源去重：同 书名+作者 只保留首个命中的书源
                let mut unique = Vec::with_capacity(books.len());
                let mut local_seen = std::collections::HashSet::new();
                for b in books {
                    let k = format!("{}_{}", b.name.trim(), b.author.trim());
                    if k.is_empty() {
                        continue;
                    }
                    if local_seen.insert(k) {
                        unique.push(b);
                    }
                }
                let payload = serde_json::json!({ "lastIndex": i as i64, "data": unique });
                (i as i64, format!("data: {payload}\n\n"))
            }));
        }
        let mut last = last_index;
        while let Some((i, text)) = tasks.next().await {
            last = i;
            if tx.send(Ok(Bytes::from(text))).await.is_err() {
                break; // 客户端断开
            }
        }
        let end_payload = serde_json::json!({ "lastIndex": last, "isEnd": last >= total - 1 });
        let _ = tx
            .send(Ok(Bytes::from(format!(
                "event: end\ndata: {end_payload}\n\n"
            ))))
            .await;
    });

    // mpsc receiver → TryStream → SSE body
    let stream = futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    });
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// SSE 错误事件（兼容 legacy：event: error + data: ReturnData）
fn sse_error(ret: ReturnData) -> Response {
    let payload = serde_json::to_string(&ret).unwrap_or_default();
    let body = format!("event: error\ndata: {payload}\n\n");
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(Body::from(body))
        .unwrap()
}

/// POST /reader3/saveBookmark：保存书签（body：bookUrl/title/paragraphIndex/chapterIndex/createdAt）
async fn save_bookmark(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let mut bookmark: crate::model::Bookmark = match serde_json::from_slice(&body) {
        Ok(b) => b,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if bookmark.book_url.is_empty() || bookmark.title.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    bookmark.user_namespace = namespace.clone();
    if bookmark.created_at == 0 {
        bookmark.created_at = now_millis();
    }
    match state.storage.save_bookmark(&namespace, &bookmark).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveBookmark 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// GET/POST /reader3/getBookmarks：书签列表（bookUrl 参数）
async fn get_bookmarks(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if book_url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.list_bookmarks(&namespace, &book_url).await {
        Ok(bookmarks) => Json(ReturnData::ok(
            serde_json::to_value(bookmarks).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getBookmarks 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/deleteBookmark：删除书签（body：bookUrl + title）
async fn delete_bookmark(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let title = param_of(&params, body_json.as_ref(), "title");
    if book_url.is_empty() || title.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .delete_bookmark(&namespace, &book_url, &title)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteBookmark 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// GET/POST /reader3/getBookGroups：书架分组列表（含组内书数 bookCount；order/orderNum 双字段）
async fn get_book_groups(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = body;
    match state.storage.list_book_groups_with_count(&namespace).await {
        Ok(groups) => Json(ReturnData::ok(
            serde_json::to_value(groups).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getBookGroups 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveBookGroup：保存分组（body：id?/name/order?；id>0 覆盖，否则新建）。
/// 分组重命名契约：body 仅 {id,name}（无 order）→ 只改名称、保留排序
async fn save_book_group(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let v: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let group: crate::model::BookGroup = match serde_json::from_value(v.clone()) {
        Ok(g) => g,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if group.name.is_empty() {
        return Json(ReturnData::err("分组名称不能为空"));
    }
    // 仅 {id,name} → 重命名（保留 order；saveBookGroupName/updateBookGroup 兼容契约）
    if group.id > 0 && v.get("order").is_none() && v.get("orderNum").is_none() {
        return match state
            .storage
            .rename_book_group(&namespace, group.id, &group.name)
            .await
        {
            Ok(0) => Json(ReturnData::err("分组不存在")),
            Ok(_) => {
                let saved = state
                    .storage
                    .list_book_groups(&namespace)
                    .await
                    .ok()
                    .and_then(|list| list.into_iter().find(|g| g.id == group.id));
                Json(ReturnData::ok(
                    serde_json::to_value(saved.unwrap_or(group)).unwrap_or(serde_json::Value::Null),
                ))
            }
            Err(e) => {
                tracing::error!("saveBookGroup 重命名失败: {e}");
                Json(ReturnData::err("保存失败"))
            }
        };
    }
    match state.storage.save_book_group(&namespace, &group).await {
        Ok(saved) => Json(ReturnData::ok(
            serde_json::to_value(saved).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("saveBookGroup 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteBookGroup：删除分组（body/query：id；组内书 group 置 0）
async fn delete_book_group(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let id = params
        .get("id")
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| params.get("groupId").and_then(|v| v.parse::<i64>().ok()))
        .or_else(|| {
            body_json
                .as_ref()
                .and_then(|b| b.get("id").and_then(|v| v.as_i64()))
        })
        .or_else(|| {
            // legacy 契约键 groupId（group.kt:24-26 checker 合并键）
            body_json
                .as_ref()
                .and_then(|b| b.get("groupId").and_then(|v| v.as_i64()))
        })
        .unwrap_or(-1);
    if id <= 0 {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_book_group(&namespace, id).await {
        Ok(0) => Json(ReturnData::err("分组不存在")),
        Ok(_) => Json(ReturnData::ok(serde_json::json!(""))),
        Err(e) => {
            tracing::error!("deleteBookGroup [{id}] 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/updateBookGroupId：书设分组（body：bookUrl + group）
async fn update_book_group_id(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let group = params
        .get("group")
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| {
            body_json
                .as_ref()
                .and_then(|b| b.get("group").and_then(|v| v.as_i64()))
        })
        .unwrap_or(-1);
    if book_url.is_empty() || group < 0 {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .update_book_group_id(&namespace, &book_url, group)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("updateBookGroupId 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/setBookGroups：多分组设置（body {bookUrl, groupIds:[...]}）——
/// legacy 多分组位掩码语义；groupIds 空数组 = 移入未分组
async fn set_book_groups(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let ids: Vec<i64> = body_json
        .as_ref()
        .and_then(|b| b.get("groupIds").and_then(|v| v.as_array()))
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
                .collect()
        })
        .unwrap_or_default();
    if book_url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .set_book_groups(&namespace, &book_url, &ids)
        .await
    {
        Ok(_) => Json(ReturnData::ok(json!({ "groupIds": ids }))),
        Err(e) => {
            tracing::error!("setBookGroups 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/addBookGroup：追加单分组（body {bookUrl, groupId}）
async fn add_book_group(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let group_id = body_json
        .as_ref()
        .and_then(|b| b.get("groupId").and_then(|v| v.as_i64()))
        .unwrap_or(-1);
    if book_url.is_empty() || group_id <= 0 {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .add_book_group(&namespace, &book_url, group_id)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("addBookGroup 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/removeBookGroup：从多分组移除单分组（body {bookUrl, groupId}）
async fn remove_book_group(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let group_id = body_json
        .as_ref()
        .and_then(|b| b.get("groupId").and_then(|v| v.as_i64()))
        .unwrap_or(-1);
    if book_url.is_empty() || group_id <= 0 {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .remove_book_group(&namespace, &book_url, group_id)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("removeBookGroup 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

// ---------------- 小项补全批 ----------------

/// GET/POST /reader3/deleteBookCache：删除单书缓存（book_chapters 该 book_url 行——
/// 本地书章节 + 书源书正文缓存）；不影响书架 books 行。
/// GAP 79：支持 body {bookUrl}（兼容 query url）；按用户（先解析命名空间，且书需在本人书架）。
/// 文案/返回值对齐 legacy deleteBookCache：请输入书籍链接 / 请先加入书架 /
/// 本地书籍无需删除缓存；成功 setData("")（data = ""）
async fn delete_book_cache(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    let url = if url.is_empty() {
        param_of(&params, body_json.as_ref(), "bookUrl")
    } else {
        url
    };
    if url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接"));
    }
    // 按用户：书必须在该用户书架（book_chapters 无命名空间列，借书架行校验归属）
    let book = match state.storage.find_book(&namespace, &url).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(ReturnData::err("请先加入书架")),
        Err(e) => {
            tracing::error!("deleteBookCache 查询失败 [{url}]: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    // legacy：本地书籍无需删除缓存
    if crate::service::local_book::is_local_book(&book.book_url, &book.origin) {
        return Json(ReturnData::err("本地书籍无需删除缓存"));
    }
    match state.storage.delete_book_cache(&namespace, &url).await {
        Ok(_) => Json(ReturnData::ok(json!(""))),
        Err(e) => {
            tracing::error!("deleteBookCache 失败 [{url}]: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// GET/POST /reader3/getShelfBookWithCacheInfo：书架书 + 缓存信息（缓存章数/正文大小）
async fn get_shelf_book_with_cache_info(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let url = param_of(&params, body_json.as_ref(), "url");
    // legacy 语义：无 url → 返回全书架列表（每本附 cachedChapterCount）
    if url.is_empty() {
        let books = match state.storage.list_books(&namespace).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("getShelfBookWithCacheInfo 列表失败: {e}");
                return Json(ReturnData::err("系统错误"));
            }
        };
        let mut out: Vec<Value> = Vec::with_capacity(books.len());
        for b in &books {
            let mut item = serde_json::to_value(b).unwrap_or(serde_json::Value::Null);
            let (cache_chapter_count, cache_size) = state
                .storage
                .book_cache_info(&namespace, &b.book_url)
                .await
                .unwrap_or((0, 0));
            if let Some(obj) = item.as_object_mut() {
                obj.insert("cacheChapterCount".to_string(), json!(cache_chapter_count));
                obj.insert("cacheSize".to_string(), json!(cache_size));
            }
            out.push(item);
        }
        return Json(ReturnData::ok(Value::Array(out)));
    }
    let book = match state.storage.find_book(&namespace, &url).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(ReturnData::err("书籍不存在")),
        Err(e) => {
            tracing::error!("getShelfBookWithCacheInfo 失败 [{url}]: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    let (cache_chapter_count, cache_size) = state
        .storage
        .book_cache_info(&namespace, &url)
        .await
        .unwrap_or((0, 0));
    let mut data = serde_json::to_value(book).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = data.as_object_mut() {
        obj.insert("cacheChapterCount".to_string(), json!(cache_chapter_count));
        obj.insert("cacheSize".to_string(), json!(cache_size));
    }
    Json(ReturnData::ok(data))
}

/// POST /reader3/book/saveBookConfig（legacy）：书籍级阅读配置持久化。
/// body：{bookUrl, pdfImageWidth, ...其余键并入 books.read_config}
async fn save_book_config(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if book_url.is_empty() {
        return Json(ReturnData::err("请输入书籍链接"));
    }
    let book = match state.storage.find_book(&namespace, &book_url).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(ReturnData::err("书籍不存在")),
        Err(e) => {
            tracing::error!("saveBookConfig 查询失败 [{book_url}]: {e}");
            return Json(ReturnData::err("系统错误"));
        }
    };
    let mut cfg = match book.read_config {
        Some(serde_json::Value::Object(m)) => m,
        _ => serde_json::Map::new(),
    };
    // 除 bookUrl 外的顶层键全部并入 read_config（legacy 只写 pdfImageWidth，超集兼容）
    if let Some(obj) = body_json.as_ref().and_then(|b| b.as_object()) {
        for (k, v) in obj {
            if k == "bookUrl" {
                continue;
            }
            cfg.insert(k.clone(), v.clone());
        }
    }
    let mut patch = serde_json::Map::new();
    patch.insert("readConfig".to_string(), serde_json::Value::Object(cfg));
    match state
        .storage
        .patch_book(&namespace, &book_url, &patch)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::json!(""))),
        Err(e) => {
            tracing::error!("saveBookConfig 失败 [{book_url}]: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/importBookPreview：导入预览（multipart file——解析但不入库）
/// 返回 {name, author, format, chapterCount, preview: [前 10 章标题]}
async fn import_book_preview(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let max_bytes = state.storage.config.upload_max_bytes();
    let max_mb = state.storage.config.upload_max_mb;
    // GAP 62：Content-Length 预检（超限 → 明确错误）
    if let Some(msg) = check_upload_content_length(&headers, max_bytes, max_mb) {
        return Json(ReturnData::err(msg));
    }
    // 取 file 字段（首块）
    let mut file_name = String::new();
    let mut bytes: Vec<u8> = Vec::new();
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                if field.name() == Some("file") {
                    file_name = field.file_name().unwrap_or("file").to_string();
                    // GAP 62：显式字段大小上限（超限 → 明确错误）
                    match read_multipart_field_limited(&mut field, max_bytes, max_mb).await {
                        Ok(b) => bytes = b,
                        Err(msg) => return Json(ReturnData::err(msg)),
                    }
                    break;
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::debug!("importBookPreview multipart 读取失败: {e}");
                break;
            }
        }
    }
    if bytes.is_empty() {
        return Json(ReturnData::err("请上传文件"));
    }
    let safe_name = std::path::Path::new(&file_name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = crate::service::local_book::file_ext(&safe_name);
    if ext.is_empty()
        || !crate::service::local_book::SUPPORTED_EXTENSIONS
            .iter()
            .any(|e| *e == ext)
    {
        // legacy 对齐：文案含扩展名插值
        return Json(ReturnData::err(format!("不支持导入{ext}格式的书籍文件")));
    }
    // 解析（parse_loc_book_path 按扩展名分派；核心逻辑在可测的纯函数中）
    let user_rules = txt_toc_rule_regexes(&state, &namespace).await;
    match import_preview_from_bytes(&bytes, &safe_name, &ext, &user_rules) {
        Ok(json) => Json(ReturnData::ok(json)),
        Err(e) => Json(ReturnData::err(format!("解析失败：{e}"))),
    }
}

/// 导入预览核心（纯函数，可测）：字节 → 临时文件 → parse_loc_book_path 解析
/// （复用本地书解析链路）→ {name, author, format, chapterCount, preview: [前 10 章标题]}；不入库
fn import_preview_from_bytes(
    bytes: &[u8],
    file_name: &str,
    ext: &str,
    user_rules: &[String],
) -> anyhow::Result<serde_json::Value> {
    let tmp_path =
        std::env::temp_dir().join(format!("reader-preview-{}.{ext}", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, bytes)?;
    let result = (|| -> anyhow::Result<serde_json::Value> {
        let imported = crate::service::local_book::parse_loc_book_path(
            &tmp_path,
            user_rules,
            crate::service::local_book::DEFAULT_EPUB_TOC_MODE,
            false,
        )?;
        let (name, author) = local_book_display_meta(file_name, ext, &imported);
        let preview: Vec<String> = imported
            .chapters
            .iter()
            .take(10)
            .map(|c| c.title.clone())
            .collect();
        // P0-6 软兼容：legacy 两步导入流期望 {book, chapters} 字段（saveBook 直接
        // 消费 book JSON + 章节清单）——在 master 形状上补充，双端均可解析
        let book_json = json!({
            "name": name,
            "author": author,
            "kind": format!("{}{}", imported.format.to_uppercase(), "书籍"),
            "bookUrl": format!("assets/{file_name}"),
            "origin": "loc_book",
            "tocUrl": "",
        });
        let chapters: Vec<serde_json::Value> = imported
            .chapters
            .iter()
            .enumerate()
            .map(|(i, c)| {
                json!({
                    "title": c.title,
                    "url": format!("{file_name}#{i}"),
                    "index": i,
                    "isVolume": false,
                })
            })
            .collect();
        Ok(json!({
            "name": name,
            "author": author,
            "format": imported.format,
            "chapterCount": imported.chapters.len(),
            "preview": preview,
            "book": book_json,
            "chapters": chapters,
        }))
    })();
    let _ = std::fs::remove_file(&tmp_path);
    result
}

/// POST /reader3/readSourceFile：读取书源文件文本（body {path}）
/// P0-5：secure 模式限 {storage_dir}/data/{ns}/ 用户子目录内（防跨用户读取，
/// resolve_secure_path 组件级防穿越）；非 secure 限工作目录内
async fn read_source_file(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let path = param_of(&params, body_json.as_ref(), "path");
    if path.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    // secure：base = 用户子目录（storage/data/{ns}/）；legacy "storage/" 前缀按相对用户目录处理
    let base = if state.storage.config.secure {
        state
            .storage
            .config
            .storage_dir()
            .join("data")
            .join(&namespace)
    } else {
        std::path::PathBuf::from(&state.storage.config.work_dir)
    };
    let rel = if state.storage.config.secure {
        path.trim_start_matches("storage/")
    } else {
        path.as_str()
    };
    let Some(file) = crate::api::files::resolve_secure_path(&base, rel) else {
        return Json(ReturnData::err("路径不存在"));
    };
    if !file.is_file() {
        return Json(ReturnData::err("路径不存在"));
    }
    match tokio::fs::read_to_string(&file).await {
        Ok(content) => Json(ReturnData::ok(serde_json::Value::String(content))),
        Err(e) => {
            tracing::error!("readSourceFile 读取失败 [{}]: {e}", file.display());
            Json(ReturnData::err("读取失败"))
        }
    }
}

/// POST /reader3/saveBookContent：写章节正文缓存（body {bookUrl, chapterUrl, title, content}）
/// chapter_index = chapterUrl md5 哈希（与 getBookContent 正文缓存同键）
async fn save_book_content(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    if book_url.is_empty() {
        // legacy 契约：url = 书籍链接
        book_url = param_of(&params, body_json.as_ref(), "url");
    }
    let mut chapter_url = param_of(&params, body_json.as_ref(), "chapterUrl");
    let mut title = param_of(&params, body_json.as_ref(), "title");
    // legacy 契约：{url,index,content}——按目录缓存反查章节 URL/标题后走同一存储键
    if chapter_url.is_empty() && !book_url.is_empty() {
        if let Ok(idx_legacy) = param_of(&params, body_json.as_ref(), "index").parse::<i64>() {
            if let Ok(Some(toc_json)) = state
                .storage
                .get_toc_cache(&namespace, &book_url, TOC_CACHE_TTL_MS)
                .await
            {
                if let Ok(chapters) = serde_json::from_str::<Vec<serde_json::Value>>(&toc_json) {
                    if let Some(c) = chapters.get(idx_legacy.max(0) as usize) {
                        if let Some(u) = c.get("url").and_then(|v| v.as_str()) {
                            chapter_url = u.to_string();
                        }
                        if title.is_empty() {
                            if let Some(t) = c.get("title").and_then(|v| v.as_str()) {
                                title = t.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    let content = param_of(&params, body_json.as_ref(), "content");
    if book_url.is_empty() || chapter_url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    if content.is_empty() {
        return Json(ReturnData::err("正文不能为空"));
    }
    let idx = crate::util::md5::chapter_url_hash(&chapter_url);
    match state
        .storage
        .cache_chapter_content(&namespace, &book_url, idx, &title, &content)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveBookContent 失败 [{book_url}]: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteUserBookSource：删除当前用户书源（body {bookSource}；
/// 兼容 bookSourceUrl/url 参数名）
async fn delete_user_book_source(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut url = param_of(&params, body_json.as_ref(), "bookSource");
    if url.is_empty() {
        url = param_of(&params, body_json.as_ref(), "bookSourceUrl");
    }
    if url.is_empty() {
        url = param_of(&params, body_json.as_ref(), "url");
    }
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_book_source(&namespace, &url).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteUserBookSource 失败 [{url}]: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/saveBookGroupId：设置书分组（body {bookUrl, groupId}）——
/// updateBookGroupId 别名（参数名兼容 group/groupId）
async fn save_book_group_id(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let book_url = param_of(&params, body_json.as_ref(), "bookUrl");
    let group = params
        .get("groupId")
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| {
            body_json
                .as_ref()
                .and_then(|b| b.get("groupId").and_then(|v| v.as_i64()))
        })
        .or_else(|| {
            params
                .get("group")
                .and_then(|v| v.parse::<i64>().ok())
                .or_else(|| {
                    body_json
                        .as_ref()
                        .and_then(|b| b.get("group").and_then(|v| v.as_i64()))
                })
        })
        .unwrap_or(-1);
    if book_url.is_empty() || group < 0 {
        return Json(ReturnData::err("参数错误"));
    }
    match state
        .storage
        .update_book_group_id(&namespace, &book_url, group)
        .await
    {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveBookGroupId 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// GET/POST /reader3/getChapterListByRule：书源 ruleToc 单页解析调试
/// 参数：url（目录页，缺省用 chapterUrl）+ bookSource（书源 URL 或完整 JSON）
/// 返回章节数组（同 getBookToc 结构：title/url/isVolume/index）
async fn get_chapter_list_by_rule(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());

    // ---- 双模式 ①：body = 本地书 Book JSON（legacy getChapterListByRule 语义）----
    // 本地书目录规则预览（"修改目录规则"流程）：按用户 TXT 规则重解析文件，
    // 返回 {book, chapters}；不落库——确认后走 saveBook + refreshLocalBook
    let body_book: Option<crate::model::Book> = body_json
        .as_ref()
        .and_then(|b| serde_json::from_value::<crate::model::Book>(b.clone()).ok())
        .filter(|b| {
            !b.name.is_empty() && crate::service::local_book::is_local_book(&b.book_url, &b.origin)
        });
    if let Some(bk) = body_book {
        // legacy 校验顺序与文案
        if bk.origin.is_empty() {
            return Json(ReturnData::err("未找到书源信息"));
        }
        let ext = crate::service::local_book::file_ext(&bk.book_url);
        if !matches!(ext.as_str(), "txt" | "epub" | "pdf") && !bk.book_url.starts_with("local://") {
            return Json(ReturnData::err("非本地txt/epub/pdf书籍"));
        }
        let path = resolve_loc_book_file(&state.storage.config.storage_dir(), &bk.book_url)
            .or_else(|| resolve_storage_path(&state.storage.config.storage_dir(), &bk.book_url));
        let entries: Vec<serde_json::Value> = match &path {
            Some(p) => {
                let user_rules = txt_toc_rule_regexes(&state, &namespace).await;
                match crate::service::local_book::parse_loc_book_path(
                    p,
                    &user_rules,
                    &bk.toc_url,
                    bk.split_long_chapter,
                ) {
                    Ok(imported) => imported
                        .chapters
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let entry_url = if bk.book_url.starts_with("local://") {
                                format!("{}/{}", bk.book_url, i)
                            } else {
                                format!("{}#{}", bk.book_url, i)
                            };
                            serde_json::json!({
                                "title": c.title,
                                "url": entry_url,
                                "isVolume": false,
                                "index": i,
                                "chapterWordCount": c.content.chars().count(),
                            })
                        })
                        .collect(),
                    Err(e) => {
                        return Json(ReturnData::err(format!("解析失败：{e:#}")));
                    }
                }
            }
            None => {
                // 无关联文件（已迁移 local:// DB 书）→ 章节表现状预览
                match state
                    .storage
                    .list_chapters_with_word_count(&bk.book_url)
                    .await
                {
                    Ok(rows) if !rows.is_empty() => rows
                        .iter()
                        .map(|(idx, title, wc)| {
                            serde_json::json!({
                                "title": title,
                                "url": format!("{}/{idx}", bk.book_url),
                                "isVolume": false,
                                "index": idx,
                                "chapterWordCount": wc,
                            })
                        })
                        .collect(),
                    _ => return Json(ReturnData::err("本地书文件不存在")),
                }
            }
        };
        return Json(ReturnData::ok(serde_json::json!({
            "book": serde_json::to_value(&bk).unwrap_or(serde_json::Value::Null),
            "chapters": entries,
        })));
    }

    // ---- 双模式 ②：网源目录页解析（url + bookSource）----
    let mut url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        url = param_of(&params, body_json.as_ref(), "chapterUrl");
    }
    if url.is_empty() {
        return Json(ReturnData::err("请输入目录链接"));
    }
    let bs_param = param_of(&params, body_json.as_ref(), "bookSource");
    if bs_param.is_empty() {
        // find_book_source 对空串走 LIKE '%' 会命中首源——显式拦截
        return Json(ReturnData::err("书源不存在"));
    }
    let Some(source) = resolve_book_source(&state, &namespace, &bs_param).await else {
        return Json(ReturnData::err("书源不存在"));
    };
    match crate::service::book::parse_toc_page(&namespace, &url, &source, None, "").await {
        Ok(chapters) => Json(ReturnData::ok(
            serde_json::to_value(chapters).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getChapterListByRule 失败 [{url}]: {e}");
            Json(ReturnData::err("获取目录失败"))
        }
    }
}

// ---------------- F-28 替换规则 ----------------

/// GET/POST /reader3/getReplaceRules：替换规则列表（用户命名空间，无则回退 default）
async fn get_replace_rules(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = body;
    match state.storage.get_replace_rules(&namespace).await {
        Ok(rules) => Json(ReturnData::ok(
            serde_json::to_value(rules).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            tracing::error!("getReplaceRules [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveReplaceRule：保存单条替换规则（body = 完整规则 JSON；id 缺失自动补 uuid）
async fn save_replace_rule(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let mut rule: crate::model::ReplaceRule = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if rule.name.trim().is_empty() {
        return Json(ReturnData::err("名称不能为空"));
    }
    if rule.find.trim().is_empty() {
        return Json(ReturnData::err("规则不能为空"));
    }
    if rule.id.trim().is_empty() {
        rule.id = format!("rule-{}", uuid::Uuid::new_v4());
    }
    rule.user_namespace = namespace.clone();
    match state.storage.save_replace_rule(&namespace, &rule).await {
        // P1-C2：返回生效 id（归属冲突时后端已改插新 id——前端据此同步本地列表，避免重复保存）
        Ok(id) => Json(ReturnData::ok(serde_json::json!({ "id": id }))),
        Err(e) => {
            tracing::error!("saveReplaceRule 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/saveReplaceRules：批量保存（body = 规则数组；逐条校验，id 缺失自动补）
async fn save_replace_rules(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let mut rules: Vec<crate::model::ReplaceRule> = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if rules.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    for rule in &mut rules {
        if rule.name.trim().is_empty() || rule.find.trim().is_empty() {
            return Json(ReturnData::err("参数错误"));
        }
        if rule.id.trim().is_empty() {
            rule.id = format!("rule-{}", uuid::Uuid::new_v4());
        }
        rule.user_namespace = namespace.clone();
    }
    match state.storage.save_replace_rules(&namespace, &rules).await {
        Ok(_) => Json(ReturnData::ok(serde_json::json!({ "count": rules.len() }))),
        Err(e) => {
            tracing::error!("saveReplaceRules 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/replaceRule/saveMulti：批量保存替换规则（legacy 对齐）。
/// body：`{"items":[{name,find,replace,...},...]}`（兼容 legacy 原始数组）；单事务；
/// 逐条校验 name/find 必填、id 缺失自动补 uuid（与 saveReplaceRules 相同语义）。
async fn save_replace_rule_multi(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let items: Vec<serde_json::Value> = match &json {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(obj) => obj
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => return Json(ReturnData::err("参数错误")),
    };
    if items.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let mut rules: Vec<crate::model::ReplaceRule> = Vec::with_capacity(items.len());
    for item in items {
        let mut rule: crate::model::ReplaceRule = match serde_json::from_value(item) {
            Ok(r) => r,
            Err(_) => return Json(ReturnData::err("参数错误")),
        };
        if rule.name.trim().is_empty() || rule.find.trim().is_empty() {
            return Json(ReturnData::err("参数错误"));
        }
        if rule.id.trim().is_empty() {
            rule.id = format!("rule-{}", uuid::Uuid::new_v4());
        }
        rule.user_namespace = namespace.clone();
        rules.push(rule);
    }
    match state.storage.save_replace_rules(&namespace, &rules).await {
        Ok(_) => Json(ReturnData::ok(serde_json::json!({ "count": rules.len() }))),
        Err(e) => {
            tracing::error!("replaceRule/saveMulti 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteReplaceRule：删除替换规则（body/query：id）
async fn delete_replace_rule(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let id = param_of(&params, body_json.as_ref(), "id");
    if id.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_replace_rule(&namespace, &id).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteReplaceRule 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/deleteReplaceRules：批量删除替换规则（legacy 对齐，单事务）。
/// body：`{"ids":["a","b"]}` 删除指定 id；`{"all":true}` 清空全部；
/// 兼容 legacy 原始规则对象数组（取 id 字段，缺失时用 name 兜底——legacy 以 name 匹配）。
async fn delete_replace_rules(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(json) = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok()) else {
        return Json(ReturnData::err("参数错误"));
    };
    let run = |ids: Vec<String>| {
        let state = &state;
        let namespace = &namespace;
        async move {
            if ids.is_empty() {
                return Json(ReturnData::err("参数错误"));
            }
            match state.storage.delete_replace_rules(namespace, &ids).await {
                Ok(n) => Json(ReturnData::ok(serde_json::json!({ "count": n }))),
                Err(e) => {
                    tracing::error!("deleteReplaceRules 失败: {e}");
                    Json(ReturnData::err("删除失败"))
                }
            }
        }
    };
    match &json {
        serde_json::Value::Array(arr) => {
            // legacy 原始数组：规则对象 → id（缺省 name）
            let ids: Vec<String> = arr
                .iter()
                .filter_map(|v| {
                    v.get("id")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            v.get("name")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string())
                        })
                })
                .collect();
            run(ids).await
        }
        serde_json::Value::Object(obj) => {
            if obj.get("all").and_then(|v| v.as_bool()).unwrap_or(false) {
                return match state.storage.delete_all_replace_rules(&namespace).await {
                    Ok(n) => Json(ReturnData::ok(serde_json::json!({ "count": n }))),
                    Err(e) => {
                        tracing::error!("deleteReplaceRules(all) 失败: {e}");
                        Json(ReturnData::err("删除失败"))
                    }
                };
            }
            let ids: Vec<String> = obj
                .get("ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            run(ids).await
        }
        _ => Json(ReturnData::err("参数错误")),
    }
}

// ---------------- F-26 HttpTTS ----------------

/// HttpTTS 输出 JSON：id 与 url 同值（前端 HttpTts 类型兼容）
fn http_tts_json(tts: &crate::model::HttpTts) -> serde_json::Value {
    serde_json::json!({
        "id": tts.url,
        "url": tts.url,
        "name": tts.name,
        "type": tts.tts_type,
        "contentType": tts.content_type,
        "concurrentRate": tts.concurrent_rate,
        "loginUrl": tts.login_url,
        "loginUi": tts.login_ui,
        "header": tts.header,
        "jsLib": tts.js_lib,
        "enabledCookieJar": tts.enabled_cookie_jar,
        "loginCheckJs": tts.login_check_js,
        "lastUpdateTime": tts.last_update_time,
    })
}

/// GET/POST /reader3/getHttpTTSList：HttpTTS 听书源列表（用户命名空间，无则回退 default）
async fn get_http_tts_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = body;
    match state.storage.get_http_tts_list(&namespace).await {
        Ok(list) => {
            let arr: Vec<serde_json::Value> = list.iter().map(http_tts_json).collect();
            Json(ReturnData::ok(serde_json::Value::Array(arr)))
        }
        Err(e) => {
            tracing::error!("getHttpTTSList [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveHttpTTS：保存听书源（body：url/name/type；url 缺失时用 id 兜底）
async fn save_http_tts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let mut tts: crate::model::HttpTts = match serde_json::from_value(json.clone()) {
        Ok(t) => t,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    // url 主键；前端可能只传 id（旧契约），用 id 兜底
    if tts.url.trim().is_empty() {
        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
            tts.url = id.to_string();
        }
    }
    if tts.url.trim().is_empty() {
        return Json(ReturnData::err("链接不能为空"));
    }
    if tts.name.trim().is_empty() {
        return Json(ReturnData::err("名称不能为空"));
    }
    tts.user_namespace = namespace.clone();
    match state.storage.save_http_tts(&namespace, &tts).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("saveHttpTTS 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/httpTTS/saveMulti：批量保存听书源（legacy 对齐）。
/// body：`{"items":[{url,name,type},...]}`（兼容 legacy 原始数组）；单事务；
/// 逐条校验：url 缺失时用 id 兜底；url/name 必填。
async fn save_http_tts_multi(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let items: Vec<serde_json::Value> = match &json {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(obj) => obj
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => return Json(ReturnData::err("参数错误")),
    };
    if items.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let mut tts_list = Vec::with_capacity(items.len());
    for item in items {
        let mut tts: crate::model::HttpTts = match serde_json::from_value(item.clone()) {
            Ok(t) => t,
            Err(_) => return Json(ReturnData::err("参数错误")),
        };
        // url 主键；前端可能只传 id（旧契约），用 id 兜底（与 saveHttpTTS 一致）
        if tts.url.trim().is_empty() {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                tts.url = id.to_string();
            }
        }
        if tts.url.trim().is_empty() {
            return Json(ReturnData::err("链接不能为空"));
        }
        if tts.name.trim().is_empty() {
            return Json(ReturnData::err("名称不能为空"));
        }
        tts.user_namespace = namespace.clone();
        tts_list.push(tts);
    }
    match state
        .storage
        .save_http_tts_multi(&namespace, &tts_list)
        .await
    {
        Ok(_) => Json(ReturnData::ok(
            serde_json::json!({ "count": tts_list.len() }),
        )),
        Err(e) => {
            tracing::error!("httpTTS/saveMulti 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteHttpTTS：删除听书源（body/query：id 或 url）
async fn delete_http_tts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let mut url = param_of(&params, body_json.as_ref(), "url");
    if url.is_empty() {
        url = param_of(&params, body_json.as_ref(), "id");
    }
    if url.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_http_tts(&namespace, &url).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteHttpTTS 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/deleteHttpTTSs：批量删除听书源（body：{ids: string[]} 或 string[]；id/url 同值）
async fn delete_http_tts_multi(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    let urls: Vec<String> = match &json {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        serde_json::Value::Object(obj) => obj
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        _ => return Json(ReturnData::err("参数错误")),
    };
    if urls.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_http_tts_multi(&namespace, &urls).await {
        Ok(count) => Json(ReturnData::ok(json!({ "count": count }))),
        Err(e) => {
            tracing::error!("deleteHttpTTSs 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

// ---------------- 自定义 TXT 目录规则 ----------------

/// GET/POST /reader3/getTxtTocRules：TXT 目录规则列表（legacy 语义：内置默认规则 + 用户自定义规则）
async fn get_txt_toc_rules(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = body;
    let mut rules: Vec<serde_json::Value> = Vec::new();
    // 内置默认规则（id 固定 default-{serial+1}，可被 importDefaultTxtTocRules 导入为用户规则）
    for def in crate::service::local_book::DEFAULT_TOC_RULE_DEFS {
        rules.push(serde_json::json!({
            "id": format!("default-{}", def.serial_number + 1),
            "name": def.name,
            "rule": def.rule,
            "enable": def.enable,
            "serialNumber": def.serial_number,
        }));
    }
    // 用户自定义规则（含导入的默认规则副本）
    match state.storage.get_txt_toc_rules(&namespace).await {
        Ok(custom) => {
            for rule in custom {
                rules.push(serde_json::to_value(rule).unwrap_or(serde_json::Value::Null));
            }
            Json(ReturnData::ok(serde_json::Value::Array(rules)))
        }
        Err(e) => {
            tracing::error!("getTxtTocRules [{namespace}] 失败: {e}");
            Json(ReturnData::err("系统错误"))
        }
    }
}

/// POST /reader3/saveTxtTocRule：保存自定义 TXT 目录规则（body：id?/name/rule/enable/serialNumber）
async fn save_txt_toc_rule(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let Some(body) = body else {
        return Json(ReturnData::err("参数错误"));
    };
    let mut rule: crate::model::TxtTocRule = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return Json(ReturnData::err("参数错误")),
    };
    if rule.name.trim().is_empty() {
        return Json(ReturnData::err("名称不能为空"));
    }
    if rule.rule.trim().is_empty() {
        return Json(ReturnData::err("规则不能为空"));
    }
    if rule.id.trim().is_empty() {
        rule.id = format!("toc-{}", uuid::Uuid::new_v4());
    }
    rule.user_namespace = namespace.clone();
    match state.storage.save_txt_toc_rule(&namespace, &rule).await {
        // P1-C2：返回生效 id（归属冲突时后端已改插新 id）
        Ok(id) => Json(ReturnData::ok(serde_json::json!({ "id": id }))),
        Err(e) => {
            tracing::error!("saveTxtTocRule 失败: {e}");
            Json(ReturnData::err("保存失败"))
        }
    }
}

/// POST /reader3/deleteTxtTocRule：删除自定义 TXT 目录规则（body/query：id）
async fn delete_txt_toc_rule(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let id = param_of(&params, body_json.as_ref(), "id");
    if id.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    match state.storage.delete_txt_toc_rule(&namespace, &id).await {
        Ok(_) => Json(ReturnData::ok(serde_json::Value::Null)),
        Err(e) => {
            tracing::error!("deleteTxtTocRule 失败: {e}");
            Json(ReturnData::err("删除失败"))
        }
    }
}

/// POST /reader3/importDefaultTxtTocRules：内置默认规则导入为用户规则（幂等，返回导入条数）
async fn import_default_txt_toc_rules(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<axum::body::Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let _ = body;
    match state.storage.import_default_txt_toc_rules(&namespace).await {
        Ok(count) => Json(ReturnData::ok(serde_json::json!({ "count": count }))),
        Err(e) => {
            tracing::error!("importDefaultTxtTocRules [{namespace}] 失败: {e}");
            Json(ReturnData::err("导入失败"))
        }
    }
}

// ---------------- 系统信息 + 服务监控 + 书源导出 ----------------

/// GET /reader3/getSystemInfo：系统信息（版本/端口/用户数/书数/书源数 + 真实监控聚合）
///
/// legacy 兼容内存字段（freeMemory/totalMemory/maxMemory，"NNN M" 字符串）为真实值——
/// Windows 上经 sysinfo 读取物理内存，不再全 0M；另附结构化 memory/cpu/requests/
/// online/bookSource（与 getServerStats 相同聚合）。
async fn get_system_info(
    State(state): State<AppState>,
    Query(_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Json<ReturnData> {
    let _ = headers;
    let user_count = state.storage.count_users().await.unwrap_or(0);
    let book_count = state.storage.count_books().await.unwrap_or(0);
    let source_count = state.storage.count_all_book_sources().await.unwrap_or(0);
    let version = env!("CARGO_PKG_VERSION");
    let agg = crate::service::monitor::collect(&state.storage).await;
    let mut data = agg.to_json();
    data["version"] = json!(version);
    data["port"] = json!(state.storage.config.port);
    data["userCount"] = json!(user_count);
    data["bookCount"] = json!(book_count);
    data["bookSourceCount"] = json!(source_count);
    // legacy 兼容字段（真实值，单位 MB 字符串）
    data["freeMemory"] = json!(format!("{}M", agg.memory.available_mb));
    data["totalMemory"] = json!(format!("{}M", agg.memory.total_mb));
    data["maxMemory"] = json!(format!("{}M", agg.memory.total_mb));
    Json(ReturnData::ok(data))
}

/// GET /reader3/getServerStats：服务监控聚合
///
/// 内存（总量/可用/已用/进程，MB + 百分比）、CPU（短采样 ~200ms）、请求计数
/// （总数/今日/按接口 Top10）、在线会话（有效 token 数）、书源成功率（最近一次检测
/// 结果，未检测则 successRate=null + 说明）、uptime。
async fn get_server_stats(State(state): State<AppState>) -> Json<ReturnData> {
    let agg = crate::service::monitor::collect(&state.storage).await;
    let mut data = agg.to_json();
    data["version"] = json!(env!("CARGO_PKG_VERSION"));
    data["port"] = json!(state.storage.config.port);
    Json(ReturnData::ok(data))
}

/// GET /reader3/exportBookSources：当前命名空间书源 JSON 下载（attachment）
async fn export_book_sources(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret).into_response(),
    };
    let sources = match state.storage.get_book_sources(&namespace).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("exportBookSources [{namespace}] 失败: {e}");
            return Json(ReturnData::err("系统错误")).into_response();
        }
    };
    let bytes = serde_json::to_vec_pretty(&sources).unwrap_or_default();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json; charset=utf-8")
        .header(
            "Content-Disposition",
            "attachment; filename=bookSource.json",
        )
        .body(Body::from(bytes))
        .unwrap()
}

/// POST /reader3/scanLocalBookDir：直接读取书仓/用户目录/WebDAV 下已有的书籍文件
/// 导入书架（无需再上传）。body：`{path, home?, recursive?}`；path 为文件时导入单本，
/// 为目录时递归扫描支持扩展名的文件。重复导入按绝对路径稳定去重（覆盖更新）。
async fn scan_local_book_dir(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let body_json = body.and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let path = param_of(&params, body_json.as_ref(), "path");
    if path.is_empty() {
        return Json(ReturnData::err("参数错误"));
    }
    let home = param_of(&params, body_json.as_ref(), "home");
    let _recursive = body_json
        .as_ref()
        .and_then(|b| b.get("recursive"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let user = state.storage.find_user(&namespace).await.ok().flatten();
    // 只读导入：不要求管理密码；secure 模式书仓/WebDAV 权限仍由 file_home 校验
    let base = match crate::api::files::file_home(
        &state.storage.config,
        &namespace,
        &home,
        false,
        false,
        false,
        user.as_ref(),
    ) {
        Ok(b) => b,
        Err(ret) => return Json(ret),
    };
    let Some(file) = crate::api::files::resolve_secure_path(&base, &path) else {
        return Json(ReturnData::err("参数错误"));
    };
    if !file.exists() {
        return Json(ReturnData::err("路径不存在"));
    }

    // 收集目标文件（目录递归；深度/数量上限防误扫大目录拖垮服务；全程受
    // READER_DIR_SCAN_RPS 节流——网盘挂载目录瞬时大量 readdir/stat 会触发风控）
    let mut targets: Vec<std::path::PathBuf> = Vec::new();
    fn collect_book_files<'a>(
        dir: &'a std::path::Path,
        depth: usize,
        out: &'a mut Vec<std::path::PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if depth > 8 || out.len() >= 500 {
                return;
            }
            crate::service::fs_rate::tick().await;
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            let mut paths: Vec<std::path::PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| !p.file_name().is_none())
                .collect();
            paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
            for p in paths {
                if out.len() >= 500 {
                    break;
                }
                crate::service::fs_rate::tick().await;
                if p.is_dir() {
                    collect_book_files(&p, depth + 1, out).await;
                } else {
                    let ext = crate::service::local_book::file_ext(&p.to_string_lossy());
                    if crate::service::local_book::SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                        out.push(p);
                    }
                }
            }
        })
    }
    if file.is_dir() {
        collect_book_files(&file, 0, &mut targets).await;
    } else {
        let ext = crate::service::local_book::file_ext(&file.to_string_lossy());
        if crate::service::local_book::SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
            targets.push(file);
        }
    }
    if targets.is_empty() {
        return Json(ReturnData::err("未找到可导入的书籍文件"));
    }

    let user_rules = txt_toc_rule_regexes(&state, &namespace).await;
    let mut imported = 0usize;
    let mut failed = 0usize;
    let mut errors: Vec<serde_json::Value> = Vec::new();
    for target in targets {
        crate::service::fs_rate::tick().await;
        let file_name = target
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "book".to_string());
        let abs = target
            .canonicalize()
            .unwrap_or_else(|_| target.clone())
            .to_string_lossy()
            .into_owned();
        let imported_book = match crate::service::local_book::parse_loc_book_path(
            &target,
            &user_rules,
            crate::service::local_book::DEFAULT_EPUB_TOC_MODE,
            false,
        ) {
            Ok(b) => b,
            Err(e) => {
                failed += 1;
                errors.push(json!({ "name": file_name, "error": format!("解析失败：{e}") }));
                continue;
            }
        };
        if imported_book.chapters.is_empty() {
            failed += 1;
            errors.push(json!({ "name": file_name, "error": "未解析到章节内容" }));
            continue;
        }
        use sha2::Digest as _;
        let mut hasher = sha2::Sha256::new();
        hasher.update(abs.as_bytes());
        let id = format!("{:x}", hasher.finalize());
        let book_url = format!("local://store/{id}");
        let ext = crate::service::local_book::file_ext(&file_name);
        let (book_name, book_author) = local_book_display_meta(&file_name, &ext, &imported_book);
        let book = crate::model::book_chapter::BookInfo {
            name: book_name,
            author: book_author,
            kind: imported_book.meta.subjects.first().cloned(),
            intro: imported_book.meta.description.clone(),
            language: imported_book.meta.language.clone(),
            publisher: imported_book.meta.publisher.clone(),
            published_at: imported_book.meta.published_at.clone(),
            toc_url: Some(format!("{book_url}/toc")),
            book_url: book_url.clone(),
            origin: "local".to_string(),
            origin_name: "本地书".to_string(),
            book_type: crate::service::local_book::local_book_type(&ext),
            ..Default::default()
        };
        if let Err(e) = state
            .storage
            .save_local_book(&namespace, &book, &imported_book)
            .await
        {
            failed += 1;
            errors.push(json!({ "name": file_name, "error": format!("入库失败：{e}") }));
            continue;
        }
        let mtime = target
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let size = target.metadata().map(|m| m.len() as i64).unwrap_or(0);
        if let Err(e) = state
            .storage
            .link_local_file(&namespace, &book_url, Some(&abs), mtime, size, false)
            .await
        {
            tracing::warn!("scanLocalBookDir 文件关联失败 [{}]: {e}", file_name);
        }
        if let Some(cover) = &imported_book.cover {
            let cover_dir = state
                .storage
                .config
                .storage_dir()
                .join("assets")
                .join(&namespace)
                .join("covers");
            let _ = std::fs::create_dir_all(&cover_dir);
            let cover_id = format!("{}.jpg", uuid::Uuid::new_v4());
            let cover_path = cover_dir.join(&cover_id);
            if std::fs::write(&cover_path, cover).is_ok() {
                let _ = state
                    .storage
                    .update_book_cover(
                        &namespace,
                        &book_url,
                        &format!("/assets/{namespace}/covers/{cover_id}"),
                    )
                    .await;
            }
        }
        imported += 1;
        tracing::info!(
            "scanLocalBookDir 导入 [{namespace}]：{}（{} 章）",
            book.name,
            imported_book.chapters.len()
        );
    }
    Json(ReturnData::ok(json!({
        "imported": imported,
        "failed": failed,
        "total": imported + failed,
        "errors": errors,
    })))
}

/// POST /reader3/uploadLocalBook：导入本地书（multipart：file）
async fn upload_local_book(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Json<ReturnData> {
    let namespace = match resolve_namespace(&state, &params, &headers).await {
        Ok(ns) => ns,
        Err(ret) => return Json(ret),
    };
    let max_bytes = state.storage.config.upload_max_bytes();
    let max_mb = state.storage.config.upload_max_mb;
    // GAP 62：Content-Length 预检（超限 → 明确错误）
    if let Some(msg) = check_upload_content_length(&headers, max_bytes, max_mb) {
        return Json(ReturnData::err(msg));
    }
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name = String::new();
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                if field.name() == Some("file") {
                    file_name = field.file_name().unwrap_or("book").to_string();
                    // GAP 62：显式字段大小上限（超限 → 明确错误）
                    match read_multipart_field_limited(&mut field, max_bytes, max_mb).await {
                        Ok(b) => file_bytes = Some(b),
                        Err(msg) => return Json(ReturnData::err(msg)),
                    }
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::debug!("uploadLocalBook multipart 读取失败: {e}");
                break;
            }
        }
    }
    let Some(bytes) = file_bytes else {
        return Json(ReturnData::err("未收到文件"));
    };
    if bytes.is_empty() {
        return Json(ReturnData::err("文件为空"));
    }

    let ext = crate::service::local_book::file_ext(&file_name);
    if !crate::service::local_book::SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Json(ReturnData::err(
            "仅支持 EPUB/TXT/MOBI/AZW3/PDF/FB2/DOCX/ZIP(含OPF)/CBZ/UMD",
        ));
    }
    // 用户自定义 TXT 目录规则（启用 + 按 serialNumber 排序）；无则用内置默认规则（仅 TXT 使用）
    let user_rules = txt_toc_rule_regexes(&state, &namespace).await;
    let imported = if ext == "txt" {
        // TXT 解析失败保持静默回退（与旧行为一致：空书 → “未解析到章节内容”）
        crate::service::local_book::parse_txt_with_rules(&bytes, &user_rules).unwrap_or_else(|e| {
            tracing::error!("TXT 解析失败: {e}");
            crate::service::local_book::ImportedBook {
                meta: Default::default(),
                chapters: vec![],
                cover: None,
                format: "txt".into(),
            }
        })
    } else {
        match crate::service::local_book::parse_file_bytes(
            &bytes,
            &ext,
            &user_rules,
            crate::service::local_book::DEFAULT_EPUB_TOC_MODE,
            false,
        ) {
            Ok(b) => b,
            Err(e) => {
                return Json(ReturnData::err(format!(
                    "{} 解析失败：{e}",
                    ext.to_uppercase()
                )))
            }
        }
    };

    if imported.chapters.is_empty() {
        return Json(ReturnData::err("未解析到章节内容"));
    }

    let book_url = format!("local://{}", uuid::Uuid::new_v4());
    let (book_name, book_author) = local_book_display_meta(&file_name, &ext, &imported);
    let book = crate::model::book_chapter::BookInfo {
        name: book_name,
        author: book_author,
        kind: imported.meta.subjects.first().cloned(),
        intro: imported.meta.description.clone(),
        language: imported.meta.language.clone(),
        publisher: imported.meta.publisher.clone(),
        published_at: imported.meta.published_at.clone(),
        toc_url: Some(format!("{book_url}/toc")),
        book_url: book_url.clone(),
        origin: "local".to_string(),
        origin_name: "本地书".to_string(),
        // 本地书默认文本；CBZ 漫画按漫画类型入架
        book_type: crate::service::local_book::local_book_type(&ext),
        ..Default::default()
    };

    if let Err(e) = state
        .storage
        .save_local_book(&namespace, &book, &imported)
        .await
    {
        tracing::error!("本地书入库失败: {e}");
        return Json(ReturnData::err("入库失败"));
    }

    // OPDS 原文件下载：原始文件落盘 data/{ns}/opds_files/{uuid}.{ext}（供 /opds/download 直下）
    let opds_dir = state
        .storage
        .config
        .storage_dir()
        .join("data")
        .join(&namespace)
        .join("opds_files");
    if let Err(e) = std::fs::create_dir_all(&opds_dir) {
        tracing::warn!("OPDS 原文件目录创建失败: {e}");
    } else {
        let file_id = book.book_url.trim_start_matches("local://");
        if let Err(e) = std::fs::write(opds_dir.join(format!("{file_id}.{ext}")), &bytes) {
            tracing::warn!("OPDS 原文件落盘失败: {e}");
        }
    }

    if let Some(cover) = &imported.cover {
        let cover_dir = state
            .storage
            .config
            .storage_dir()
            .join("assets")
            .join(&namespace)
            .join("covers");
        let _ = std::fs::create_dir_all(&cover_dir);
        let file_id = format!("{}.jpg", uuid::Uuid::new_v4());
        if std::fs::write(cover_dir.join(&file_id), cover).is_ok() {
            let _ = state
                .storage
                .update_book_cover(
                    &namespace,
                    &book_url,
                    &format!("/assets/{namespace}/covers/{file_id}"),
                )
                .await;
        }
    }

    tracing::info!(
        "本地书导入 [{namespace}]：{}（{} 章）",
        book.name,
        imported.chapters.len()
    );
    Json(ReturnData::ok(
        serde_json::to_value(book).unwrap_or(serde_json::Value::Null),
    ))
}

/// storage 内安全路径解析（防穿越）
fn resolve_storage_path(
    storage_dir: &std::path::Path,
    book_url: &str,
) -> Option<std::path::PathBuf> {
    let rel = book_url.trim_start_matches("storage/");
    let candidate = storage_dir.join(rel);
    let abs = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.clone());
    let root = storage_dir
        .canonicalize()
        .unwrap_or_else(|_| storage_dir.to_path_buf());
    if abs.starts_with(&root) && abs.is_file() {
        Some(abs)
    } else {
        None
    }
}

/// 用户 TXT 目录规则正则列表（启用 + 按 serialNumber 排序；失败/无规则返回空 → 调用方回退默认）
async fn txt_toc_rule_regexes(state: &AppState, ns: &str) -> Vec<String> {
    match state.storage.get_txt_toc_rules(ns).await {
        Ok(rules) => rules
            .into_iter()
            .filter(|r| r.enable && !r.rule.trim().is_empty())
            .map(|r| r.rule)
            .collect(),
        Err(e) => {
            tracing::warn!("getTxtTocRules 失败（回退默认规则）: {e}");
            Vec::new()
        }
    }
}

/// 本地书展示名/作者（legacy 语义）：
/// - TXT：文件名解析优先（legacy TextFile 无内容元数据，名称来自 analyzeNameAuthor）；
/// - 其他格式：内容元数据优先（EPUB OPF/UMD 头/CBZ ComicInfo），文件名解析回退；
/// - 两者皆空再退文件主名。
fn local_book_display_meta(
    file_name: &str,
    ext: &str,
    imported: &crate::service::local_book::ImportedBook,
) -> (String, String) {
    let (file_name_title, file_name_author) =
        crate::service::local_book::analyze_name_author(file_name);
    let stem = std::path::Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());
    let name = if ext == "txt" {
        if !file_name_title.is_empty() {
            file_name_title
        } else if !imported.meta.title.is_empty() {
            imported.meta.title.clone()
        } else {
            stem
        }
    } else if !imported.meta.title.is_empty() {
        imported.meta.title.clone()
    } else if !file_name_title.is_empty() {
        file_name_title
    } else {
        stem
    };
    let author = if !imported.meta.author.is_empty() {
        imported.meta.author.clone()
    } else {
        file_name_author
    };
    (name, author)
}

/// chapterUrl 是否文件型本地书章节（bookPart 是 storage/ 路径或白名单扩展名文件）
fn is_loc_book_file_chapter(chapter_url: &str) -> bool {
    let Some((book_part, _)) = chapter_url.rsplit_once('#') else {
        return false;
    };
    if book_part.starts_with("storage/") {
        return true;
    }
    let lower = book_part.to_lowercase();
    crate::service::local_book::SUPPORTED_EXTENSIONS
        .iter()
        .any(|e| lower.ends_with(&format!(".{e}")))
}

/// legacy 本地书文件定位：book_url 指向的文件可能缺失（legacy 导入时改名 index.epub）
/// 兜底：父目录 index.epub → 任意 epub/txt。
/// P0-7：返回前 canonicalize + containment 校验——路径必须在 storage_dir 内
/// （防 .. 穿越/绝对路径/符号链接逃逸；与 resolve_storage_path 同模式）
fn resolve_loc_book_file(
    storage_dir: &std::path::Path,
    book_url: &str,
) -> Option<std::path::PathBuf> {
    let path = storage_dir.join(book_url.trim_start_matches("storage/"));
    let mut found: Option<std::path::PathBuf> = None;
    if path.is_file() {
        found = Some(path.clone());
    }
    // legacy 的 epub 是目录（{书名}.epub/ 内含 index.epub）
    if found.is_none() && path.is_dir() {
        let idx = path.join("index.epub");
        if idx.is_file() {
            found = Some(idx);
        } else {
            let rd = std::fs::read_dir(&path).ok()?;
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() && p.to_string_lossy().to_lowercase().ends_with(".epub") {
                    found = Some(p);
                    break;
                }
            }
        }
    }
    if found.is_none() {
        let parent = path.parent()?;
        let idx = parent.join("index.epub");
        if idx.is_file() {
            found = Some(idx);
        } else {
            let rd = std::fs::read_dir(parent).ok()?;
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() {
                    let lower = p.to_string_lossy().to_lowercase();
                    if lower.ends_with(".epub") || lower.ends_with(".txt") {
                        found = Some(p);
                        break;
                    }
                }
            }
        }
    }
    let p = found?;
    let abs = p.canonicalize().unwrap_or_else(|_| p.clone());
    let root = storage_dir
        .canonicalize()
        .unwrap_or_else(|_| storage_dir.to_path_buf());
    if abs.starts_with(&root) && abs.is_file() {
        Some(abs)
    } else {
        None
    }
}

/// legacy 本地书目录：toc_url 是分章正则（或空）——查书架定位 TXT 文件 → 按规则分章
async fn get_book_toc_loc_book(
    state: &AppState,
    namespace: &str,
    req_url: &str,
    toc_rule: &str,
) -> Option<Json<ReturnData>> {
    // 书架找本地书：优先按传入 url 精确匹配，兜底第一本 loc_book
    let books = state.storage.list_books(namespace).await.ok()?;
    let book = books
        .iter()
        .find(|b| b.origin == "loc_book" && !b.book_url.is_empty() && b.book_url == req_url)
        .or_else(|| {
            books
                .iter()
                .find(|b| b.origin == "loc_book" && !b.book_url.is_empty())
        })
        .or_else(|| books.iter().find(|b| b.origin == "loc_book"))?;
    let book_url = &book.book_url;
    tracing::debug!("loc_book toc: req={req_url} matched={book_url}");
    // GAP 171：已迁移（migrateLocBook）的书——章节表命中即 DB 直读（含字数）
    if let Ok(chapters) = state.storage.list_chapters_with_word_count(book_url).await {
        if !chapters.is_empty() {
            return toc_json_from_chapters(book_url, &chapters);
        }
    }
    if !book_url.starts_with("storage/") {
        tracing::debug!("loc_book toc: book_url 非 storage 路径");
        return None;
    }
    let Some(path) = resolve_loc_book_file(&state.storage.config.storage_dir(), book_url) else {
        tracing::debug!("loc_book toc: 文件定位失败 [{book_url}]");
        return None;
    };
    let path_lower = path.to_string_lossy().to_lowercase();
    // 按扩展名分派（复用 resolve_loc_book_file 定位结果；TXT 用默认规则分章）。
    // EPUB 目录模式：请求 tocUrl 为合法六模式优先（前端切换），否则书架书 toc_url，
    // 再否则默认 spin+toc（parse 内部对非法值回退默认）
    let toc_mode = if crate::service::local_book::is_epub_toc_mode(toc_rule) {
        toc_rule
    } else {
        book.toc_url.as_str()
    };
    let imported = match crate::service::local_book::parse_loc_book_path(
        &path,
        &[],
        toc_mode,
        book.split_long_chapter,
    ) {
        Ok(b) => b,
        Err(e) => {
            tracing::debug!("loc_book toc: 解析失败 [{path_lower}] {e}");
            return None;
        }
    };
    let chapters = imported.chapters;
    let list: Vec<serde_json::Value> = chapters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            serde_json::json!({
                "title": c.title,
                "url": format!("{book_url}#{i}"),
                "isVolume": false,
                "index": i,
                "chapterWordCount": c.content.chars().count(),
            })
        })
        .collect();
    Some(Json(ReturnData::ok(serde_json::Value::Array(list))))
}

/// 文件型本地书目录：按扩展名解析（TXT 用用户规则）→ 章节列表（chapterUrl = bookUrl#index）
async fn get_book_toc_file(state: &AppState, ns: &str, book_url: &str) -> Option<Json<ReturnData>> {
    // GAP 171：已迁移（migrateLocBook）的书——章节表命中即 DB 直读（含字数）
    if let Ok(chapters) = state.storage.list_chapters_with_word_count(book_url).await {
        if !chapters.is_empty() {
            return toc_json_from_chapters(book_url, &chapters);
        }
    }
    // 优先严格路径（防穿越），失败回退 legacy 目录式 index.epub 定位
    let path = resolve_storage_path(&state.storage.config.storage_dir(), book_url)
        .or_else(|| resolve_loc_book_file(&state.storage.config.storage_dir(), book_url))?;
    // EPUB 目录模式：书架书 toc_url（六模式）→ 默认 spin+toc；
    // TXT 长章节拆分标志同取自书架书——目录与正文必须同参保证 #index 一致
    let shelf = state.storage.find_book(ns, book_url).await.ok().flatten();
    let toc_mode = shelf
        .as_ref()
        .map(|b| b.toc_url.clone())
        .unwrap_or_default();
    let split_long = shelf
        .as_ref()
        .map(|b| b.split_long_chapter)
        .unwrap_or(false);
    let user_rules = txt_toc_rule_regexes(state, ns).await;
    let imported =
        crate::service::local_book::parse_loc_book_path(&path, &user_rules, &toc_mode, split_long)
            .ok()?;
    let list: Vec<serde_json::Value> = imported
        .chapters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            serde_json::json!({
                "title": c.title,
                "url": format!("{book_url}#{i}"),
                "isVolume": false,
                "index": i,
                "chapterWordCount": c.content.chars().count(),
            })
        })
        .collect();
    Some(Json(ReturnData::ok(serde_json::Value::Array(list))))
}

/// 章节表 → 目录 JSON（文件书 url 格式 {book_url}#{index}；含字数 chapterWordCount）
fn toc_json_from_chapters(
    book_url: &str,
    chapters: &[(i64, String, i64)],
) -> Option<Json<ReturnData>> {
    let list: Vec<serde_json::Value> = chapters
        .iter()
        .map(|(idx, title, wc)| {
            serde_json::json!({
                "title": title,
                "url": format!("{book_url}#{idx}"),
                "isVolume": false,
                "index": idx,
                "chapterWordCount": wc,
            })
        })
        .collect();
    Some(Json(ReturnData::ok(serde_json::Value::Array(list))))
}

/// 文件型本地书正文：bookUrl#index → 定位文件（白名单扩展名）→ 提取章节
async fn get_book_content_file(
    state: &AppState,
    ns: &str,
    chapter_url: &str,
    epub_content: bool,
) -> Option<Json<ReturnData>> {
    let (book_part, idx_part) = chapter_url.rsplit_once('#')?;
    let index: i64 = idx_part.parse().ok()?;
    // epubContent=1 仅对 EPUB 生效（legacy epubContent 语义；其余格式走纯文本）
    let is_epub = epub_content && crate::service::local_book::file_ext(book_part) == "epub";
    let shelf = state.storage.find_book(ns, book_part).await.ok().flatten();
    // 非文本文型分派（legacy getBookContent 漫画/PDF 页图模式）——优先于文本通道
    if let Some(ret) = local_book_image_content(state, ns, book_part, index, shelf.as_ref()).await {
        return Some(ret);
    }
    // GAP 171：已迁移的书——章节表直读（索引命中即返回，不再解析文件）
    if let Ok(Some(content)) = state
        .storage
        .get_chapter_content(ns, book_part, index)
        .await
    {
        if !content.trim().is_empty() {
            let content = if is_epub {
                wrap_epub_html(&content)
            } else {
                content
            };
            return Some(Json(ReturnData::ok(
                serde_json::json!({ "content": content }),
            )));
        }
    }
    let path = resolve_loc_book_file(&state.storage.config.storage_dir(), book_part)?;
    // 按扩展名分派（TXT 用用户规则，其余格式用各自解析器）。
    // EPUB 目录模式 + TXT 长章节拆分标志均取自书架书——目录与正文同参，
    // 保证 #index 索引与 getBookToc 返回的章节顺序一致
    let toc_mode = shelf
        .as_ref()
        .map(|b| b.toc_url.clone())
        .unwrap_or_default();
    let split_long = shelf
        .as_ref()
        .map(|b| b.split_long_chapter)
        .unwrap_or(false);
    let user_rules = txt_toc_rule_regexes(state, ns).await;
    let imported =
        crate::service::local_book::parse_loc_book_path(&path, &user_rules, &toc_mode, split_long)
            .ok()?;
    let content = imported.chapters.get(index as usize)?.content.clone();
    let content = if is_epub {
        wrap_epub_html(&content)
    } else {
        content
    };
    Some(Json(ReturnData::ok(
        serde_json::json!({ "content": content }),
    )))
}

/// 本地书目录（local://book_id/toc）
async fn get_book_toc_local(
    state: &AppState,
    _namespace: &str,
    book_url: &str,
) -> Option<Json<ReturnData>> {
    let chapters = state
        .storage
        .list_chapters_with_word_count(book_url)
        .await
        .ok()?;
    let list: Vec<serde_json::Value> = chapters
        .iter()
        .map(|(idx, title, wc)| {
            serde_json::json!({
                "title": title,
                "url": format!("{book_url}/{idx}"),
                "isVolume": false,
                "index": idx,
                "chapterWordCount": wc,
            })
        })
        .collect();
    Some(Json(ReturnData::ok(serde_json::Value::Array(list))))
}

/// 本地书正文（local://book_id/index）
async fn get_book_content_local(
    state: &AppState,
    ns: &str,
    chapter_url: &str,
    epub_content: bool,
) -> Option<Json<ReturnData>> {
    let rest = chapter_url.trim_start_matches("local://");
    let (book_id, idx_str) = rest.rsplit_once('/')?;
    let index: i64 = idx_str.parse().ok()?;
    let book_url = format!("local://{book_id}");
    // 非文本文型分派（legacy getBookContent 漫画/PDF 页图模式）——优先于 DB 章节直读
    let shelf = state.storage.find_book(ns, &book_url).await.ok().flatten();
    if let Some(ret) = local_book_image_content(state, ns, &book_url, index, shelf.as_ref()).await {
        return Some(ret);
    }
    // epubContent=1 仅对 EPUB 生效（上传书无扩展名 → 源文件定位后按扩展名判断）
    let is_epub = epub_content
        && crate::service::local_book::file_ext(
            &resolve_local_book_source_file(state, ns, &book_url)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
        ) == "epub";
    let content = state
        .storage
        .get_chapter_content(ns, &book_url, index)
        .await
        .ok()??;
    let content = if is_epub {
        wrap_epub_html(&content)
    } else {
        content
    };
    Some(Json(ReturnData::ok(
        serde_json::json!({ "content": content }),
    )))
}

/// epubContent=1：EPUB 本地书正文包裹基本 HTML 结构（legacy 章节原文 XHTML 模式的最小对齐
/// ——纯文本按段落 <p> 包裹，前端 HTML 渲染直接可用；仅 EPUB 调用）
fn wrap_epub_html(text: &str) -> String {
    let html = text
        .lines()
        .map(|l| format!("<p>{l}</p>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("<html><body>{html}</body></html>")
}

/// 本地书源文件定位（漫画页图解压用）：
/// - `local://{uuid}`：uploadLocalBook 落盘的 data/{ns}/opds_files/{uuid}.{ext}
/// - `storage/**`：legacy 文件路径书 → resolve_loc_book_file 白名单解析
fn resolve_local_book_source_file(
    state: &AppState,
    ns: &str,
    book_url: &str,
) -> Option<std::path::PathBuf> {
    if let Some(file_id) = book_url.strip_prefix("local://") {
        // 防穿越：uuid 段不允许路径分隔符/..
        if file_id.is_empty()
            || file_id.contains('/')
            || file_id.contains('\\')
            || file_id.contains("..")
        {
            return None;
        }
        let dir = state
            .storage
            .config
            .storage_dir()
            .join("data")
            .join(ns)
            .join("opds_files");
        crate::service::local_book::SUPPORTED_EXTENSIONS
            .iter()
            .find_map(|ext| {
                let p = dir.join(format!("{file_id}.{ext}"));
                p.is_file().then_some(p)
            })
    } else {
        resolve_loc_book_file(&state.storage.config.storage_dir(), book_url)
    }
}

/// URL 路径段编码（img src 用——中文/空格/# 等字符安全进 URL）
fn encode_url_path_segments(rel: &str) -> String {
    rel.split('/')
        .map(|seg| urlencoding::encode(seg).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// img 标签列表 → {content} 返回（legacy getBookContent 漫画/PDF 页图模式形态）
fn image_list_content(base_href: String, files: &[String]) -> Json<ReturnData> {
    let html = files
        .iter()
        .map(|rel| format!("<img src=\"{base_href}{}\">", encode_url_path_segments(rel)))
        .collect::<Vec<_>>()
        .join("\n");
    Json(ReturnData::ok(serde_json::json!({ "content": html })))
}

/// 本地书非文本文型内容（legacy getBookContent 三模式最小对齐）：
/// - CBZ 漫画（type=2 / cbz 标记 / .cbz）：章节页图解压到 assets/{ns}/cbz/{md5(bookUrl)}/{index}/，
///   返回 `<img src="/assets/{ns}/cbz/{md5}/{index}/{name}">` 标签列表（前端漫画渲染 + /assets 静态路由直读；
///   解压失败降级回文本通道）
/// - PDF（type=4 / pdf 标记 / .pdf）：已转换页图存在时返回页图标签，否则提示先转换
/// - 其余（含 TXT；EPUB 的 epubContent=1 在文本通道包裹 HTML 结构）→ None（走原有文本提取返回）
async fn local_book_image_content(
    state: &AppState,
    ns: &str,
    book_url: &str,
    index: i64,
    shelf: Option<&crate::model::book::Book>,
) -> Option<Json<ReturnData>> {
    let (comic_flag, pdf_flag) = shelf
        .map(|b| {
            (
                b.cbz || b.book_type == 2,
                b.pdf || b.local_pdf || b.book_type == 4,
            )
        })
        .unwrap_or((false, false));
    let ext = crate::service::local_book::file_ext(book_url);
    let is_comic = comic_flag || ext == "cbz";
    let is_pdf = !is_comic && (pdf_flag || ext == "pdf");
    if !is_comic && !is_pdf {
        return None;
    }
    let md5 = crate::util::md5::md5_encode(book_url);
    if is_pdf {
        // PDF：仅展示已转换页图（convertPdfToImage 落盘 assets/{ns}/pdf/{md5}/{page}.jpg 通道）
        let base = format!("/assets/{ns}/pdf/{md5}/");
        let pages_dir = state
            .storage
            .config
            .storage_dir()
            .join("assets")
            .join(ns)
            .join("pdf")
            .join(&md5);
        let files = crate::service::local_book::list_image_files(&pages_dir);
        if files.is_empty() {
            return Some(Json(ReturnData::ok(serde_json::json!(
                { "content": "PDF 阅读需要先转换页面" }
            ))));
        }
        return Some(image_list_content(base, &files));
    }
    // CBZ：源文件缺失 → 回退文本通道；已解压目录直读（幂等），否则从源文件解压
    let src = resolve_local_book_source_file(state, ns, book_url)?;
    let base = format!("/assets/{ns}/cbz/{md5}/{index}/");
    let chapter_dir = state
        .storage
        .config
        .storage_dir()
        .join("assets")
        .join(ns)
        .join("cbz")
        .join(&md5)
        .join(index.to_string());
    let mut files = crate::service::local_book::list_image_files(&chapter_dir);
    if files.is_empty() {
        let bytes = tokio::fs::read(&src).await.ok()?;
        match crate::service::local_book::extract_cbz_chapter_images(&bytes, &chapter_dir) {
            Ok(list) => {
                files = list;
                if files.is_empty() {
                    // 空解压结果清理占位目录，避免残留空目录
                    let _ = std::fs::remove_dir_all(&chapter_dir);
                    return None;
                }
            }
            Err(e) => {
                tracing::warn!("CBZ 页图解压失败 [{book_url}#{index}]: {e:#}");
                let _ = std::fs::remove_dir_all(&chapter_dir);
                return None;
            }
        }
    }
    Some(image_list_content(base, &files))
}

/// fallback：webdav 分流 / API 404 JSON / 前端 SPA（index.html）
async fn fallback_handler(
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    State(state): State<AppState>,
    method: axum::http::Method,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let path = uri.path();
    tracing::debug!("fallback: {} {}", method, path);
    // WebDAV
    if path.starts_with("/reader3/webdav") {
        // P1-2：客户端 IP（直连优先，可信代理白名单内才信 XFF）——WebDAV Basic 限流键
        let ip = client_ip(&peer, &headers);
        return crate::api::webdav::handle(&state.storage, method, path, &headers, body, &ip).await;
    }
    // 其他 /reader3 未匹配 → JSON 404
    if path.starts_with("/reader3") {
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(
                serde_json::json!({"isSuccess": false, "errorMsg": "接口不存在", "data": null}),
            ),
        )
            .into_response();
    }
    // 前端静态资源（/static/** 等构建产物——按扩展名 MIME，防路径穿越）
    // 内嵌资产优先（rust-embed——发布单文件免外部 dist）；磁盘目录回退
    // （READER_APP_WEB_ROOT 自定义主题 / 开发热更）
    let rel = path.trim_start_matches('/');
    if let Some((bytes, mime)) = crate::web_assets::get(rel) {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime)
            .body(Body::from(bytes))
            .unwrap();
    }
    let web_root = std::path::PathBuf::from(&state.storage.config.web_root);
    let file = web_root.join(rel);
    let file_abs = file.canonicalize().unwrap_or_else(|_| file.clone());
    let root_abs = web_root.canonicalize().unwrap_or_else(|_| web_root.clone());
    if file_abs.starts_with(&root_abs) && file.is_file() {
        if let Ok(bytes) = tokio::fs::read(&file).await {
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_for(&file))
                .body(Body::from(bytes))
                .unwrap();
        }
    }
    // 前端 SPA：index.html（内嵌优先）
    if let Some((bytes, mime)) = crate::web_assets::index_html() {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime)
            .body(Body::from(bytes))
            .unwrap();
    }
    let index = web_root.join("index.html");
    match tokio::fs::read(&index).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(bytes))
            .unwrap(),
        Err(_) => webdav_status_404(),
    }
}

/// 按扩展名推断 MIME（前端静态资源）
fn mime_for(file: &std::path::Path) -> &'static str {
    match file.extension().and_then(|e| e.to_str()) {
        Some("js") => "application/javascript; charset=utf-8",
        Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("map") => "application/json",
        _ => "application/octet-stream",
    }
}

fn webdav_status_404() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

/// legacy 静态路由 /book-assets/* 与 /epub/*（YueduApi.kt:136-162）共享实现：
/// 以 storage/data/ 为 Web 根服务 EPUB 解压后的图片/CSS/章节 HTML 等书籍资源；
/// HTML（html/htm/xhtml）响应在 </body> 前注入
/// `<script>window.__API_ROOT__="{base}"</script>`（legacy
/// BookConfig.injectJavascriptToEpubChapter 为磁盘改写注入，此处改为响应级注入，
/// 不落盘、天然幂等）。文件不存在/目录/不安全路径 → 404。
async fn serve_data_file(
    state: &AppState,
    headers: &HeaderMap,
    prefix: &str,
    uri_path: &str,
) -> Response {
    let Some(rel) = uri_path.strip_prefix(prefix).and_then(safe_data_rel_path) else {
        return webdav_status_404();
    };
    let root = state.storage.config.storage_dir().join("data");
    let file = root.join(&rel);
    // 防穿越兜底：规范化后必须仍位于 data 根内（符号链接/盘符等），且必须是普通文件
    let (Ok(root_abs), Ok(file_abs)) = (root.canonicalize(), file.canonicalize()) else {
        return webdav_status_404();
    };
    if !file_abs.starts_with(&root_abs) || !file_abs.is_file() {
        return webdav_status_404();
    }
    let bytes = match tokio::fs::read(&file_abs).await {
        Ok(b) => b,
        Err(_) => return webdav_status_404(),
    };
    let ext = file_abs
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if matches!(ext.as_str(), "html" | "htm" | "xhtml") {
        let html = String::from_utf8_lossy(&bytes);
        let injected = inject_api_root_script(&html, &request_base_url(headers));
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(injected))
            .unwrap();
    }
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_for(&file_abs))
        .body(Body::from(bytes))
        .unwrap()
}

/// GET /book-assets/*rest：storage/data/** 书籍资源（legacy YueduApi.kt:136-146）
async fn book_assets(
    State(state): State<AppState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
) -> Response {
    serve_data_file(&state, &headers, "/book-assets/", uri.path()).await
}

/// GET /epub/*rest：storage/data/** EPUB 章节 HTML（含 JS 注入，legacy YueduApi.kt:147-162）
async fn epub_asset(
    State(state): State<AppState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
) -> Response {
    serve_data_file(&state, &headers, "/epub/", uri.path()).await
}

/// 逐段解码并校验相对路径（防穿越）：拒绝 ..、段内分隔符残留（%2F/%5C 解码后再查）、
/// 盘符/ADS 冒号、NUL 与控制字符；空段与 "." 折叠。None = 不安全或空路径。
fn safe_data_rel_path(tail: &str) -> Option<std::path::PathBuf> {
    let mut rel = std::path::PathBuf::new();
    for seg in tail.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        let decoded = urlencoding::decode(seg).ok()?;
        let d = decoded.as_ref();
        if d == ".."
            || d.contains('/')
            || d.contains('\\')
            || d.contains(':')
            || d.contains('\0')
            || d.bytes().any(|b| b < 0x20)
        {
            return None;
        }
        rel.push(d);
    }
    if rel.as_os_str().is_empty() {
        None
    } else {
        Some(rel)
    }
}

/// ASCII 大小写不敏感子串查找（返回字节偏移——不能用 to_lowercase 的偏移映射回原文：
/// Unicode 小写化对部分非 ASCII 字符变长）
fn find_ascii_ci(hay: &str, needle: &str) -> Option<usize> {
    let (h, n) = (hay.as_bytes(), needle.as_bytes());
    if n.is_empty() || h.len() < n.len() {
        return None;
    }
    (0..=h.len() - n.len()).find(|&i| {
        h[i..i + n.len()]
            .iter()
            .zip(n)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
    })
}

/// HTML 正文注入：在 </body>（大小写不敏感）前插入 __API_ROOT__ 脚本；
/// 无 </body> 时追加到末尾。base_url 已由 request_base_url 白名单过滤 + 此处转义。
fn inject_api_root_script(html: &str, base_url: &str) -> String {
    let escaped = base_url.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("<script>window.__API_ROOT__=\"{escaped}\"</script>");
    match find_ascii_ci(html, "</body>") {
        Some(pos) => format!("{}{}{}", &html[..pos], script, &html[pos..]),
        None => format!("{html}{script}"),
    }
}

/// 请求基址 scheme://host：proto 取 X-Forwarded-Proto（http/https 白名单，缺省 http），
/// host 取 Host 头且仅允许 [A-Za-z0-9.\-:\[\]]（缺失/非法 → 空串，脚本退化为根相对）
fn request_base_url(headers: &HeaderMap) -> String {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let host_ok = !host.is_empty()
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b':' | b'[' | b']'));
    if !host_ok {
        return String::new();
    }
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|p| *p == "http" || *p == "https")
        .unwrap_or("http");
    format!("{proto}://{host}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Bytes;
    use axum::extract::State as AxumState;

    /// 独立临时目录存储（避免污染真实 storage/reader.db）
    async fn test_state(tag: &str) -> (AppState, std::path::PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("reader-router-test-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut config = crate::AppConfig::from_env();
        config.work_dir = dir.to_string_lossy().into_owned();
        // 既有认证测试不关心过期——默认禁用（GAP 118 过期由专用测试单独覆盖）
        config.token_ttl_days = 0;
        let storage = crate::storage::init(&config).await.unwrap();
        // 图片代理磁盘缓存：独立临时目录 + 固定 1MB 容量（不受宿主 env READER_IMAGE_CACHE_MB 影响）
        let image_cache = crate::service::image_cache::ImageCache::with_capacity(
            dir.join("storage").join("cache").join("images"),
            1024 * 1024,
        );
        (
            AppState {
                storage,
                image_cache,
            },
            dir,
        )
    }

    async fn cleanup(state: AppState, dir: std::path::PathBuf) {
        state.storage.pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn source_json(i: i64) -> serde_json::Value {
        serde_json::json!({
            "bookSourceUrl": format!("https://s{i}.com"),
            "bookSourceName": format!("源{i}"),
        })
    }

    /// F-7：saveBookSource / saveBookSources 书源数上限（users.book_source_limit）
    #[tokio::test]
    async fn test_save_book_source_limit() {
        let (state, dir) = test_state("bslimit").await;
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_source_limit: 2,
                ..Default::default()
            })
            .await
            .unwrap();

        // 单个保存：前两个成功
        for i in 1..=2 {
            let body = Bytes::from(source_json(i).to_string());
            let ret = save_book_source(
                AxumState(state.clone()),
                Query(HashMap::new()),
                HeaderMap::new(),
                Some(body),
            )
            .await;
            assert!(
                ret.0.is_success,
                "第 {i} 个书源应保存成功: {}",
                ret.0.error_msg
            );
        }
        // 第三个超限
        let body = Bytes::from(source_json(3).to_string());
        let ret = save_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "超过书源数上限");
        // 覆盖已存在的不计名额
        let body = Bytes::from(source_json(1).to_string());
        let ret = save_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "覆盖已存在书源应成功");

        // 批量：3 个新源超限整批拒绝
        let batch = serde_json::json!([source_json(10), source_json(11), source_json(12)]);
        let ret = save_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(batch.to_string())),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "超过书源数上限");
        // 上限提到 3：批量 1 个新源 + 已存在源 → 成功
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_source_limit: 3,
                ..Default::default()
            })
            .await
            .unwrap();
        let batch = serde_json::json!([source_json(1), source_json(10)]);
        let ret = save_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(batch.to_string())),
        )
        .await;
        assert!(
            ret.0.is_success,
            "新增 1 个不超限应成功: {}",
            ret.0.error_msg
        );

        // limit=0（无用户行/不限制）→ 放行
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_source_limit: 0,
                ..Default::default()
            })
            .await
            .unwrap();
        let body = Bytes::from(source_json(20).to_string());
        let ret = save_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);

        cleanup(state, dir).await;
    }

    /// P3-A：共享书源数上限函数 book_source_limit_exceeded 直接单测
    /// （三个端点 saveBookSource/saveBookSources/saveFromRemoteSource 均委托它）
    #[tokio::test]
    async fn test_book_source_limit_exceeded_shared_fn() {
        let (state, dir) = test_state("bslimit-fn").await;
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_source_limit: 2,
                ..Default::default()
            })
            .await
            .unwrap();
        let storage = &state.storage;
        // 空库：2 个名额内不超限
        assert!(
            !book_source_limit_exceeded(storage, "default", &["https://a.com", "https://b.com"])
                .await
                .unwrap(),
            "2 个新源恰好 2 名额不应超限"
        );
        // 入库 2 个后：再新增 1 个超限
        storage
            .save_book_sources(
                "default",
                &[
                    serde_json::from_value(source_json(1)).unwrap(),
                    serde_json::from_value(source_json(2)).unwrap(),
                ],
            )
            .await
            .unwrap();
        assert!(
            book_source_limit_exceeded(storage, "default", &["https://c.com"])
                .await
                .unwrap(),
            "已满 2 个再新增应超限"
        );
        // 覆盖已存在不计名额 → 不超限（s1.com 已入库，覆盖不占名额）
        assert!(
            !book_source_limit_exceeded(storage, "default", &["https://s1.com"])
                .await
                .unwrap(),
            "覆盖已存在书源不计名额"
        );
        // limit=0（不限制）→ 不超限
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_source_limit: 0,
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(
            !book_source_limit_exceeded(storage, "default", &["https://d.com"])
                .await
                .unwrap(),
            "limit=0 不限制"
        );
        cleanup(state, dir).await;
    }

    /// F-13：getShelfBook 返回书架单书 / 不存在报“书籍不存在”
    #[tokio::test]
    async fn test_get_shelf_book() {
        let (state, dir) = test_state("shelf").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://book.com/a".into(),
                    name: "测试书".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let params: HashMap<String, String> = [("url".into(), "https://book.com/a".into())]
            .into_iter()
            .collect();
        let ret = get_shelf_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["bookUrl"], "https://book.com/a");
        assert_eq!(ret.0.data["name"], "测试书");

        let params: HashMap<String, String> = [("url".into(), "https://nope.com".into())]
            .into_iter()
            .collect();
        let ret = get_shelf_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "书籍不存在");

        cleanup(state, dir).await;
    }

    /// deleteBookCache：删单书缓存（book_chapters 行）——只删目标书、书架不受影响；
    /// GAP 79：支持 body {bookUrl}、按用户（书需在本人书架）；
    /// legacy 对齐文案（请输入书籍链接/请先加入书架/本地书籍无需删除缓存）+ 成功 data=""
    #[tokio::test]
    async fn test_delete_book_cache_api() {
        let (state, dir) = test_state("delbcache").await;
        state
            .storage
            .save_chapters(
                "default",
                "https://book.com/a",
                &[("第一章".to_string(), "正文A".to_string())],
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "https://book.com/b",
                &[("第一章".to_string(), "正文B".to_string())],
            )
            .await
            .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://book.com/a".into(),
                    name: "书A".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // query url（GET）
        let params: HashMap<String, String> = [("url".into(), "https://book.com/a".into())]
            .into_iter()
            .collect();
        let ret = delete_book_cache(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data, json!(""), "legacy setData(\"\")");
        assert_eq!(
            state
                .storage
                .count_chapters("default", "https://book.com/a")
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            state
                .storage
                .count_chapters("default", "https://book.com/b")
                .await
                .unwrap(),
            1,
            "其他书缓存不受影响"
        );
        assert!(
            state
                .storage
                .find_book("default", "https://book.com/a")
                .await
                .unwrap()
                .is_some(),
            "删除缓存不应动书架"
        );

        // body {bookUrl}（POST）
        state
            .storage
            .save_chapters(
                "default",
                "https://book.com/a",
                &[("第一章".to_string(), "正文A2".to_string())],
            )
            .await
            .unwrap();
        let body = Bytes::from(r#"{"bookUrl":"https://book.com/a"}"#);
        let ret = delete_book_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data, json!(""));
        assert_eq!(
            state
                .storage
                .count_chapters("default", "https://book.com/a")
                .await
                .unwrap(),
            0
        );

        // 缺 url/bookUrl → legacy「请输入书籍链接」
        let ret = delete_book_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "请输入书籍链接");

        // 按用户：书不在该用户书架 → 拒绝（缓存保留）
        state
            .storage
            .save_chapters(
                "default",
                "https://book.com/b",
                &[("第一章".to_string(), "正文B".to_string())],
            )
            .await
            .unwrap();
        let body = Bytes::from(r#"{"bookUrl":"https://book.com/b"}"#);
        let ret = delete_book_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "请先加入书架", "书不在书架 → 按用户拒绝");
        assert_eq!(
            state
                .storage
                .count_chapters("default", "https://book.com/b")
                .await
                .unwrap(),
            1,
            "缓存应保留"
        );
        // 该书入自己的书架后即可删
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://book.com/b".into(),
                    name: "书B".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let body = Bytes::from(r#"{"bookUrl":"https://book.com/b"}"#);
        let ret = delete_book_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data, json!(""));

        // 本地书 → legacy「本地书籍无需删除缓存」（缓存保留）
        let local_url = "storage/data/default/本地书.txt";
        state
            .storage
            .save_chapters("default", local_url, &[("第一章".to_string(), "本地正文".to_string())])
            .await
            .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: local_url.into(),
                    name: "本地书".into(),
                    origin: "loc_book".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let params: HashMap<String, String> =
            [("url".into(), local_url.into())].into_iter().collect();
        let ret = delete_book_cache(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "本地书籍无需删除缓存");
        assert_eq!(
            state
                .storage
                .count_chapters("default", local_url)
                .await
                .unwrap(),
            1,
            "本地书缓存应保留"
        );
        cleanup(state, dir).await;
    }

    /// getShelfBookWithCacheInfo：书架书 + cacheChapterCount/cacheSize
    #[tokio::test]
    async fn test_get_shelf_book_with_cache_info_api() {
        let (state, dir) = test_state("shelfcache").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://book.com/a".into(),
                    name: "缓存书".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "https://book.com/a",
                &[
                    ("第一章".to_string(), "正文一二三".to_string()),
                    ("第二章".to_string(), "正文四五六".to_string()),
                ],
            )
            .await
            .unwrap();

        let params: HashMap<String, String> = [("url".into(), "https://book.com/a".into())]
            .into_iter()
            .collect();
        let ret = get_shelf_book_with_cache_info(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["bookUrl"], "https://book.com/a");
        assert_eq!(ret.0.data["name"], "缓存书");
        assert_eq!(ret.0.data["cacheChapterCount"], 2);
        assert_eq!(ret.0.data["cacheSize"], 10, "5+5 字符×2 章");

        // 不存在 → 书籍不存在；缺 url → 返回全书架列表（legacy 语义，含 cacheChapterCount）
        let params: HashMap<String, String> = [("url".into(), "https://book.com/none".into())]
            .into_iter()
            .collect();
        let ret = get_shelf_book_with_cache_info(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "书籍不存在");
        let ret = get_shelf_book_with_cache_info(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "无 url 应返回全书架列表（legacy）");
        assert!(ret.0.data.is_array());
        cleanup(state, dir).await;
    }

    /// saveBookContent：写正文缓存（chapterUrl md5 键）→ 可读回
    #[tokio::test]
    async fn test_save_book_content_api() {
        let (state, dir) = test_state("savecontent").await;
        let chapter_url = "https://book.com/c/1";
        let body = Bytes::from(format!(
            r#"{{"bookUrl":"https://book.com/a","chapterUrl":"{chapter_url}","title":"第一章","content":"手动写入的正文"}}"#
        ));
        let ret = save_book_content(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let idx = crate::util::md5::chapter_url_hash(chapter_url);
        let cached = state
            .storage
            .get_chapter_content("default", "https://book.com/a", idx)
            .await
            .unwrap();
        assert_eq!(cached.as_deref(), Some("手动写入的正文"));
        assert_eq!(
            state
                .storage
                .list_chapters("https://book.com/a")
                .await
                .unwrap()[0]
                .1,
            "第一章",
            "标题一并入库"
        );

        // 缺 bookUrl/chapterUrl → 参数错误；空正文 → 正文不能为空
        let ret = save_book_content(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(r#"{"bookUrl":"x"}"#)),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let ret = save_book_content(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(
                r#"{"bookUrl":"x","chapterUrl":"y","content":""}"#,
            )),
        )
        .await;
        assert_eq!(ret.0.error_msg, "正文不能为空");
        cleanup(state, dir).await;
    }

    /// deleteUserBookSource：删当前用户书源（body {bookSource}）
    #[tokio::test]
    async fn test_delete_user_book_source_api() {
        let (state, dir) = test_state("delusersrc").await;
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s1.com".into(),
                    book_source_name: "源1".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s2.com".into(),
                    book_source_name: "源2".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let body = Bytes::from(r#"{"bookSource":"https://s1.com"}"#);
        let ret = delete_user_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(state
            .storage
            .find_book_source("default", "https://s1.com")
            .await
            .unwrap()
            .is_none());
        assert!(
            state
                .storage
                .find_book_source("default", "https://s2.com")
                .await
                .unwrap()
                .is_some(),
            "其他书源保留"
        );

        // 缺参 → 参数错误；query bookSource 形式生效
        let ret = delete_user_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let params: HashMap<String, String> = [("bookSource".into(), "https://s2.com".into())]
            .into_iter()
            .collect();
        let ret = delete_user_book_source(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert!(state
            .storage
            .find_book_source("default", "https://s2.com")
            .await
            .unwrap()
            .is_none());
        cleanup(state, dir).await;
    }

    /// saveBookGroupId：updateBookGroupId 别名（groupId/group 参数名兼容）
    #[tokio::test]
    async fn test_save_book_group_id_api() {
        let (state, dir) = test_state("savegrpid").await;
        let gid = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "玄幻".into(),
                    order: 1,
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .id;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://b.com/1".into(),
                    name: "书1".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // body groupId
        let body = Bytes::from(format!(
            r#"{{"bookUrl":"https://b.com/1","groupId":{gid}}}"#
        ));
        let ret = save_book_group_id(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/1")
                .await
                .unwrap()
                .unwrap()
                .group,
            gid
        );

        // query group 参数名（兼容旧 updateBookGroupId 命名）
        let params: HashMap<String, String> = [
            ("bookUrl".into(), "https://b.com/1".into()),
            ("group".into(), "0".into()),
        ]
        .into_iter()
        .collect();
        let ret = save_book_group_id(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/1")
                .await
                .unwrap()
                .unwrap()
                .group,
            0
        );

        // 缺 bookUrl / 非法 groupId → 参数错误
        let ret = save_book_group_id(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(r#"{"groupId":1}"#)),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let body = Bytes::from(r#"{"bookUrl":"https://b.com/1","groupId":-5}"#);
        let ret = save_book_group_id(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        cleanup(state, dir).await;
    }

    /// readSourceFile：读书源文件文本；secure 限 storage 内，非 secure 限工作目录内（防穿越）
    #[tokio::test]
    async fn test_read_source_file_api() {
        let (mut state, dir) = test_state("readsrc").await;
        // 非 secure：工作目录内可读
        std::fs::write(
            dir.join("bookSource.json"),
            r#"{"bookSourceUrl":"https://x.com"}"#,
        )
        .unwrap();
        let body = Bytes::from(r#"{"path":"bookSource.json"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data, json!(r#"{"bookSourceUrl":"https://x.com"}"#));

        // 穿越/绝对路径拒绝（解析不出 → 路径不存在）
        let body = Bytes::from(r#"{"path":"../escape.json"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "路径不存在");
        // 不存在 → 路径不存在；缺 path → 参数错误
        let body = Bytes::from(r#"{"path":"ghost.json"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "路径不存在");
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(r#"{}"#)),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        // secure：用户子目录（storage/data/{ns}/）内可读、目录外拒绝（需登录）
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "tok9".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                token: "tokb".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let auth_params: HashMap<String, String> = [("accessToken".into(), "alice:tok9".into())]
            .into_iter()
            .collect();
        let storage_dir = state.storage.config.storage_dir();
        let alice_dir = storage_dir.join("data").join("alice");
        std::fs::create_dir_all(&alice_dir).unwrap();
        std::fs::write(alice_dir.join("bookSource.json"), "[secure]").unwrap();
        let body = Bytes::from(r#"{"path":"storage/bookSource.json"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(auth_params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "secure 用户目录内应可读: {}",
            ret.0.error_msg
        );
        assert_eq!(ret.0.data, json!("[secure]"));
        let body = Bytes::from(r#"{"path":"work-only.txt"}"#);
        std::fs::write(dir.join("work-only.txt"), "outside").unwrap();
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(auth_params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success, "工作目录文件在 secure 下不可达");
        assert_eq!(ret.0.error_msg, "路径不存在");
        // storage 根下的文件（非用户子目录）也不可达
        std::fs::write(storage_dir.join("root-only.json"), "root").unwrap();
        let body = Bytes::from(r#"{"path":"storage/root-only.json"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(auth_params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success, "storage 根文件在 secure 下不可达");
        assert_eq!(ret.0.error_msg, "路径不存在");
        // P0-5 跨用户拒绝：bob 目录内文件，alice 不可读（含 .. 穿越到 bob 目录）
        let bob_dir = storage_dir.join("data").join("bob");
        std::fs::create_dir_all(&bob_dir).unwrap();
        std::fs::write(bob_dir.join("secret.txt"), "bob-secret").unwrap();
        let body = Bytes::from(r#"{"path":"storage/data/bob/secret.txt"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(auth_params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success, "跨用户路径应拒绝");
        assert_eq!(ret.0.error_msg, "路径不存在");
        let body = Bytes::from(r#"{"path":"../bob/secret.txt"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(auth_params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success, ".. 穿越到其他用户目录应拒绝");
        // 本人（alice）目录内文件仍可读（相对路径不带 storage/ 前缀）
        std::fs::write(alice_dir.join("mine.txt"), "mine").unwrap();
        let body = Bytes::from(r#"{"path":"mine.txt"}"#);
        let ret = read_source_file(
            AxumState(state.clone()),
            Query(auth_params),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "用户目录内相对路径应可读: {}",
            ret.0.error_msg
        );
        assert_eq!(ret.0.data, json!("mine"));
        cleanup(state, dir).await;
    }

    /// getChapterListByRule：书源 ruleToc 单页解析（url 抓取 → 章节数组）
    #[tokio::test]
    async fn test_get_chapter_list_by_rule_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("chplist").await;
        let base_url = serve_bodies(vec![r#"{"chapters":[
            {"t":"第一章 开始","h":"/c/1.html"},
            {"t":"第二章 继续","h":"/c/2.html"}
        ]}"#
        .to_string()])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "目录源".into(),
                    rule_toc: Some(serde_json::json!({
                        "chapterList": "$.chapters[*]",
                        "chapterName": "$.t",
                        "chapterUrl": "$.h",
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // url + bookSource（书源 URL）
        let toc_url = format!("{base}/toc");
        let params: HashMap<String, String> = [
            ("url".into(), toc_url.clone()),
            ("bookSource".into(), base.clone()),
        ]
        .into_iter()
        .collect();
        let ret = get_chapter_list_by_rule(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["title"], "第一章 开始");
        assert_eq!(
            arr[0]["url"],
            format!("{base}/c/1.html"),
            "相对 URL 应转绝对"
        );
        assert_eq!(arr[0]["index"], 0);
        assert_eq!(arr[1]["title"], "第二章 继续");

        // chapterUrl 参数兜底
        let params: HashMap<String, String> = [
            ("chapterUrl".into(), toc_url.clone()),
            ("bookSource".into(), base.clone()),
        ]
        .into_iter()
        .collect();
        let ret = get_chapter_list_by_rule(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(
            ret.0.is_success,
            "chapterUrl 兜底应生效: {}",
            ret.0.error_msg
        );

        // 缺 url/书源不存在 → 参数校验
        let ret = get_chapter_list_by_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "请输入目录链接");
        let params: HashMap<String, String> = [("url".into(), toc_url)].into_iter().collect();
        let ret = get_chapter_list_by_rule(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源不存在");
        cleanup(state, dir).await;
    }

    /// importBookPreview：multipart file → 解析预览（不入库）——handler 全链路 + 纯函数
    #[tokio::test]
    async fn test_import_book_preview_api() {
        // 纯函数核心：TXT 三章 → {name/format/chapterCount/preview 前 10 章}
        let txt = "第一章 起点\n内容一。\n第二章 成长\n内容二。\n第三章 终局\n内容三。";
        let json = import_preview_from_bytes(txt.as_bytes(), "测试.txt", "txt", &[]).unwrap();
        assert_eq!(json["format"], "txt");
        assert_eq!(json["chapterCount"], 3);
        let preview = json["preview"].as_array().unwrap();
        assert_eq!(preview.len(), 3);
        assert_eq!(preview[0], "第一章 起点");
        assert_eq!(preview[2], "第三章 终局");
        // 不支持的格式
        assert!(import_preview_from_bytes(b"x", "x.exe", "exe", &[]).is_err());

        // handler 全链路：构造 multipart 请求体 → Multipart 提取器 → 响应
        let (state, dir) = test_state("importprev").await;
        let boundary = "reader-test-boundary";
        let multipart_body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\n{txt}\r\n--{boundary}--\r\n"
        );
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(multipart_body))
            .unwrap();
        use axum::extract::FromRequest;
        let multipart = axum::extract::Multipart::from_request(req, &())
            .await
            .unwrap();
        let ret = import_book_preview(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["format"], "txt");
        assert_eq!(ret.0.data["chapterCount"], 3);
        assert_eq!(ret.0.data["preview"][0], "第一章 起点");
        // 不支持的格式 → legacy 文案（含扩展名插值）
        let bad_body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.exe\"\r\nContent-Type: application/octet-stream\r\n\r\nx\r\n--{boundary}--\r\n"
        );
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(bad_body))
            .unwrap();
        let multipart = axum::extract::Multipart::from_request(req, &())
            .await
            .unwrap();
        let ret = import_book_preview(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "不支持导入exe格式的书籍文件");
        // 不入库：书架/章节表无痕
        assert!(state
            .storage
            .list_books("default")
            .await
            .unwrap()
            .is_empty());
        cleanup(state, dir).await;
    }

    /// F-25：logout——非 secure 拒绝；secure 清 token 且 token 立即失效
    #[tokio::test]
    async fn test_logout_clears_token() {
        let (state, dir) = test_state("logout").await;
        // 非 secure → 不支持的操作
        let ret = logout(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "不支持的操作");

        // secure：登录用户 logout 后 token 清空、旧 token 失效
        let mut state = state;
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "tok123".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [("accessToken".into(), "alice:tok123".into())]
            .into_iter()
            .collect();
        let ret = logout(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "logout 应成功: {}", ret.0.error_msg);
        assert!(state
            .storage
            .find_user("alice")
            .await
            .unwrap()
            .unwrap()
            .token
            .is_empty());

        // 旧 token 再次访问 → NEED_LOGIN
        let ret = logout(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.data, json!("NEED_LOGIN"));

        cleanup(state, dir).await;
    }

    /// GAP 118：token 过期——users.last_login_at + READER_TOKEN_TTL_DAYS（默认 30 天）；
    /// 过期 → NEED_LOGIN 重新登录；ttl<=0 永不过期；登出清 token 后立即失效
    #[tokio::test]
    async fn test_token_expiry_by_last_login() {
        let (state, dir) = test_state("tokenttl").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.token_ttl_days = 30;
        let now = now_millis();
        let day_ms = 86_400_000i64;
        // 新登录用户（last_login_at = now）
        state
            .storage
            .insert_user(&User {
                username: "fresh".into(),
                token: "tok-fresh".into(),
                last_login_at: now,
                ..Default::default()
            })
            .await
            .unwrap();
        // 31 天前登录 → 已过期
        state
            .storage
            .insert_user(&User {
                username: "stale".into(),
                token: "tok-stale".into(),
                last_login_at: now - 31 * day_ms,
                ..Default::default()
            })
            .await
            .unwrap();
        // 29 天前登录 → 未过期（边界内）
        state
            .storage
            .insert_user(&User {
                username: "edge".into(),
                token: "tok-edge".into(),
                last_login_at: now - 29 * day_ms,
                ..Default::default()
            })
            .await
            .unwrap();
        // 从未登录（last_login_at=0，legacy 迁移数据）→ 过期
        state
            .storage
            .insert_user(&User {
                username: "never".into(),
                token: "tok-never".into(),
                last_login_at: 0,
                ..Default::default()
            })
            .await
            .unwrap();

        let auth = |u: &str, t: &str| {
            let params: HashMap<String, String> = [("accessToken".into(), format!("{u}:{t}"))]
                .into_iter()
                .collect();
            params
        };
        // 未过期 → 正常解析命名空间
        assert_eq!(
            resolve_namespace(&state, &auth("fresh", "tok-fresh"), &HeaderMap::new())
                .await
                .unwrap(),
            "fresh"
        );
        assert_eq!(
            resolve_namespace(&state, &auth("edge", "tok-edge"), &HeaderMap::new())
                .await
                .unwrap(),
            "edge"
        );
        // 过期 → NEED_LOGIN
        for (u, t) in [("stale", "tok-stale"), ("never", "tok-never")] {
            let ret = resolve_namespace(&state, &auth(u, t), &HeaderMap::new()).await;
            assert!(ret.is_err());
            assert_eq!(
                ret.unwrap_err().data,
                json!("NEED_LOGIN"),
                "{u} 应需重新登录"
            );
        }
        // token 不匹配仍 NEED_LOGIN（不受过期影响）
        let ret = resolve_namespace(&state, &auth("fresh", "wrong"), &HeaderMap::new()).await;
        assert!(ret.is_err());

        // ttl<=0（永不过期）→ stale 也可用
        state.storage.config.token_ttl_days = 0;
        assert_eq!(
            resolve_namespace(&state, &auth("stale", "tok-stale"), &HeaderMap::new())
                .await
                .unwrap(),
            "stale"
        );

        // 重新登录刷新 last_login_at → 恢复可用（模拟 login 更新会话）
        state.storage.config.token_ttl_days = 30;
        state
            .storage
            .update_user_session("stale", "tok-stale", now)
            .await
            .unwrap();
        assert_eq!(
            resolve_namespace(&state, &auth("stale", "tok-stale"), &HeaderMap::new())
                .await
                .unwrap(),
            "stale"
        );

        // 登出（清 token）→ 立即 NEED_LOGIN
        state.storage.logout_user("fresh").await.unwrap();
        let ret = resolve_namespace(&state, &auth("fresh", "tok-fresh"), &HeaderMap::new()).await;
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().data, json!("NEED_LOGIN"));

        cleanup(state, dir).await;
    }

    /// GAP 59：多设备 token——主 token 与 token_map 任一 token 均可解析命名空间；
    /// 登出仅移除当前设备（其余设备不受影响）。GAP 61：登录限流（用户名+IP 失败 5 次锁 5 分钟）
    #[tokio::test]
    async fn test_multi_device_token_and_login_rate_limit() {
        let (state, dir) = test_state("multidev").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.token_ttl_days = 0;
        let now = now_millis();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "main".into(),
                last_login_at: now,
                ..Default::default()
            })
            .await
            .unwrap();
        // 模拟两台设备登录（追加 token_map）+ 主 token 刷新
        state
            .storage
            .add_user_token("alice", "dev1", now)
            .await
            .unwrap();
        state
            .storage
            .add_user_token("alice", "dev2", now)
            .await
            .unwrap();
        state
            .storage
            .add_user_token("alice", "main", now)
            .await
            .unwrap();
        let auth = |t: &str| {
            let params: HashMap<String, String> = [("accessToken".into(), format!("alice:{t}"))]
                .into_iter()
                .collect();
            params
        };
        // 主 token 与 token_map 任一 token 均可通过
        for t in ["main", "dev1", "dev2"] {
            assert_eq!(
                resolve_namespace(&state, &auth(t), &HeaderMap::new())
                    .await
                    .unwrap(),
                "alice",
                "token {t} 应可通过"
            );
        }
        // 未记录 token → NEED_LOGIN
        let ret = resolve_namespace(&state, &auth("ghost-token"), &HeaderMap::new()).await;
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().data, json!("NEED_LOGIN"));
        // 登出 dev1 → 仅 dev1 失效，dev2/main 仍有效
        let ret = logout(
            AxumState(state.clone()),
            Query(auth("dev1")),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "logout 应成功: {}", ret.0.error_msg);
        assert!(resolve_namespace(&state, &auth("dev1"), &HeaderMap::new())
            .await
            .is_err());
        assert!(resolve_namespace(&state, &auth("dev2"), &HeaderMap::new())
            .await
            .is_ok());
        assert!(resolve_namespace(&state, &auth("main"), &HeaderMap::new())
            .await
            .is_ok());

        // GAP 61：登录限流（用户名+直连 IP 失败 5 次 → 锁定，返回“尝试过多请稍后”）；
        // M3：限流键 = 直连 socket IP（ConnectInfo）——X-Forwarded-For 伪造不再生效
        let login_body = |u: &str, pw: &str| {
            Json(LoginBody {
                username: Some(u.into()),
                password: Some(pw.into()),
                is_login: Some(true),
                code: None,
            })
        };
        let peer_a: std::net::SocketAddr = "203.0.113.9:40001".parse().unwrap();
        for _ in 0..5 {
            let ret = login(
                ConnectInfo(peer_a),
                AxumState(state.clone()),
                HeaderMap::new(),
                login_body("alice", "wrong"),
            )
            .await;
            assert_eq!(ret.0.error_msg, "密码错误");
        }
        // 第 6 次（已锁定）→ 尝试过多请稍后（即使密码正确也不放行）
        let ret = login(
            ConnectInfo(peer_a),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("alice", "whatever"),
        )
        .await;
        assert_eq!(ret.0.error_msg, "尝试过多请稍后");
        // M3：XFF 伪造无法绕过——同一直连 IP 换 X-Forwarded-For 头仍被锁定
        let mut h_spoof = HeaderMap::new();
        h_spoof.insert("x-forwarded-for", "9.9.9.9".parse().unwrap());
        h_spoof.insert("x-real-ip", "9.9.9.9".parse().unwrap());
        let ret = login(
            ConnectInfo(peer_a),
            AxumState(state.clone()),
            h_spoof,
            login_body("alice", "wrong"),
        )
        .await;
        assert_eq!(
            ret.0.error_msg, "尝试过多请稍后",
            "XFF/x-real-ip 伪造应无法绕过锁定（直连 IP 桶）"
        );
        // 同用户名不同直连 IP 不受影响（各自独立计数）
        let peer_b: std::net::SocketAddr = "203.0.113.10:40002".parse().unwrap();
        let ret = login(
            ConnectInfo(peer_b),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("alice", "wrong"),
        )
        .await;
        assert_eq!(ret.0.error_msg, "密码错误");

        // 正确密码登录成功 → 计数清零（新用户名+IP，避免与上文锁定状态耦合）
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                password: crate::util::md5::gen_encrypted_password("pass1234", "saltsalt"),
                salt: "saltsalt".into(),
                token: "bobt".into(),
                last_login_at: now,
                ..Default::default()
            })
            .await
            .unwrap();
        let peer_bob: std::net::SocketAddr = "198.51.100.7:40003".parse().unwrap();
        for _ in 0..4 {
            let ret = login(
                ConnectInfo(peer_bob),
                AxumState(state.clone()),
                HeaderMap::new(),
                login_body("bob", "bad"),
            )
            .await;
            assert_eq!(ret.0.error_msg, "密码错误");
        }
        // 第 5 次失败 → 锁定（错误消息仍为密码错误，但后续被拒）
        let ret = login(
            ConnectInfo(peer_bob),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("bob", "bad"),
        )
        .await;
        assert_eq!(ret.0.error_msg, "密码错误");
        let ret = login(
            ConnectInfo(peer_bob),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("bob", "pass1234"),
        )
        .await;
        assert_eq!(ret.0.error_msg, "尝试过多请稍后");
        // 锁定自动恢复（5 分钟过期）已由 util::login_limit 单测覆盖——此处验证正确密码
        // 在未锁定状态下可登录成功并重置计数
        let peer_c: std::net::SocketAddr = "198.51.100.8:40004".parse().unwrap();
        let ret = login(
            ConnectInfo(peer_c),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("bob", "pass1234"),
        )
        .await;
        assert!(ret.0.is_success, "正确密码应登录成功: {}", ret.0.error_msg);
        // 登录成功后计数清零：再失败 4 次仍不锁（需满 5 次）
        for _ in 0..4 {
            let ret = login(
                ConnectInfo(peer_c),
                AxumState(state.clone()),
                HeaderMap::new(),
                login_body("bob", "bad"),
            )
            .await;
            assert_eq!(ret.0.error_msg, "密码错误");
        }
        assert!(
            crate::util::login_limit::check_allowed("bob", "198.51.100.8").is_ok(),
            "成功登录已重置计数"
        );

        cleanup(state, dir).await;
    }

    /// argon2id 密码哈希：新用户注册 → argon2id 存储/登录；legacy MD5 旧用户登录 →
    /// 自动升级（password 变 $argon2id$）→ 再次登录走 argon2 路径；错误密码拒绝
    #[tokio::test]
    async fn test_argon2id_register_login_and_md5_upgrade() {
        let (state, dir) = test_state("argon2").await;
        let peer: std::net::SocketAddr = "203.0.113.50:41001".parse().unwrap();
        let login_body = |u: &str, pw: &str| {
            Json(LoginBody {
                username: Some(u.into()),
                password: Some(pw.into()),
                is_login: Some(true),
                code: None,
            })
        };
        let auto_register = |u: &str, pw: &str| {
            Json(LoginBody {
                username: Some(u.into()),
                password: Some(pw.into()),
                is_login: Some(false),
                code: None,
            })
        };

        // 1) 新用户注册（is_login=false）→ users.password 存 argon2id PHC（不再 MD5）
        let ret = login(
            ConnectInfo(peer),
            AxumState(state.clone()),
            HeaderMap::new(),
            auto_register("carol", "pass1234"),
        )
        .await;
        assert!(ret.0.is_success, "注册应成功: {}", ret.0.error_msg);
        // P1-7：注册初始 token 为 uuid v4（32 位十六进制）——与登录一致，不再可预测
        let carol = state.storage.find_user("carol").await.unwrap().unwrap();
        assert_eq!(carol.token.len(), 32, "uuid v4 simple 应为 32 字符");
        assert!(
            carol.token.chars().all(|c| c.is_ascii_hexdigit()),
            "token 应为十六进制: {}",
            carol.token
        );
        assert!(
            carol
                .password
                .starts_with("$argon2id$v=19$m=65536,t=3,p=4$"),
            "新用户密码应为 argon2id PHC: {}",
            carol.password
        );
        assert!(crate::util::password::verify_argon2id(
            "pass1234",
            &carol.password
        ));

        // 2) 注册用户登录成功（argon2id 校验路径）
        let ret = login(
            ConnectInfo(peer),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("carol", "pass1234"),
        )
        .await;
        assert!(
            ret.0.is_success,
            "argon2id 用户登录应成功: {}",
            ret.0.error_msg
        );

        // 3) 错误密码拒绝
        let ret = login(
            ConnectInfo(peer),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("carol", "wrongpass"),
        )
        .await;
        assert_eq!(ret.0.error_msg, "密码错误");

        // 4) legacy MD5 旧用户：登录成功 → 自动升级为 argon2id（无需重置密码）
        let salt = "legacysalt".to_string();
        state
            .storage
            .insert_user(&User {
                username: "dave".into(),
                password: crate::util::md5::gen_encrypted_password("oldpass1", &salt),
                salt: salt.clone(),
                token: "davet".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let ret = login(
            ConnectInfo(peer),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("dave", "oldpass1"),
        )
        .await;
        assert!(
            ret.0.is_success,
            "MD5 旧用户登录应成功: {}",
            ret.0.error_msg
        );
        let dave = state.storage.find_user("dave").await.unwrap().unwrap();
        assert!(
            dave.password.starts_with("$argon2id$"),
            "MD5 旧用户登录后 password 应自动升级为 argon2id: {}",
            dave.password
        );

        // 5) 升级后再次登录成功（argon2id 校验路径）
        let ret = login(
            ConnectInfo(peer),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("dave", "oldpass1"),
        )
        .await;
        assert!(
            ret.0.is_success,
            "升级后再次登录应成功: {}",
            ret.0.error_msg
        );

        // 6) 升级后错误密码仍拒绝
        let ret = login(
            ConnectInfo(peer),
            AxumState(state.clone()),
            HeaderMap::new(),
            login_body("dave", "wrongpass"),
        )
        .await;
        assert_eq!(ret.0.error_msg, "密码错误");

        cleanup(state, dir).await;
    }

    /// F-34：clearInactiveUsers——secureKey 校验 + 仅删超期用户（调用者受保护）
    #[tokio::test]
    async fn test_clear_inactive_users() {
        let (state, dir) = test_state("inactive").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        let mk = |name: &str, last: i64| User {
            username: name.into(),
            token: "t".into(),
            last_login_at: last,
            ..Default::default()
        };
        state.storage.insert_user(&mk("old", 1000)).await.unwrap();
        state
            .storage
            .insert_user(&mk("new", now_millis()))
            .await
            .unwrap();

        // 缺 secureKey → NEED_SECURE_KEY（需先登录，legacy checkAuth 优先）
        let body = Bytes::from(r#"{"inactiveDay":1}"#);
        let auth_params: HashMap<String, String> = [("accessToken".into(), "new:t".into())]
            .into_iter()
            .collect();
        let ret = clear_inactive_users(
            AxumState(state.clone()),
            Query(auth_params),
            HeaderMap::new(),
            Some(body.clone()),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        // 带 secureKey（登录 accessToken）→ 删除 old，保留 new
        let params: HashMap<String, String> = [
            ("accessToken".into(), "new:t".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();
        let ret = clear_inactive_users(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "清理应成功: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["deleted"], json!(["old"]));
        assert_eq!(ret.0.data["count"], 1);
        assert!(state.storage.find_user("old").await.unwrap().is_none());
        assert!(state.storage.find_user("new").await.unwrap().is_some());

        cleanup(state, dir).await;
    }

    /// F-39：backupToWebdav——secure 未开启 webdav 拒绝；成功返回 zip 路径
    #[tokio::test]
    async fn test_backup_to_webdav() {
        let (state, dir) = test_state("backup").await;
        let mut state = state;
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                enable_webdav: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [("accessToken".into(), "alice:t1".into())]
            .into_iter()
            .collect();
        // 未开启 webdav → 拒绝
        let ret = backup_to_webdav(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "未开启webdav功能");
        // 开启 webdav → 打包成功，zip 在 webdav/legado 下
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                enable_webdav: true,
                ..Default::default()
            })
            .await
            .unwrap();
        let ret = backup_to_webdav(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "备份应成功: {}", ret.0.error_msg);
        let path = ret.0.data["path"].as_str().expect("应返回 zip 路径");
        assert!(
            path.contains("legado") && path.contains("backup-") && path.ends_with(".zip"),
            "路径: {path}"
        );
        assert!(std::path::Path::new(path).exists());

        cleanup(state, dir).await;
    }

    /// MongoDB 备份/恢复：未配置 uri（body/env 均无）→ 明确错误，不 panic
    #[tokio::test]
    async fn test_mongodb_backup_requires_uri() {
        let (state, dir) = test_state("mongobackup").await;
        std::env::remove_var("READER_MONGODB_URI");

        let ret = backup_to_mongodb(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(r#"{"db":"reader3"}"#)),
        )
        .await;
        assert!(!ret.0.is_success);
        assert!(
            ret.0.error_msg.contains("READER_MONGODB_URI"),
            "错误信息: {}",
            ret.0.error_msg
        );

        let ret = restore_from_mongodb(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(r#"{"db":"reader3"}"#)),
        )
        .await;
        assert!(!ret.0.is_success);
        assert!(
            ret.0.error_msg.contains("READER_MONGODB_URI"),
            "错误信息: {}",
            ret.0.error_msg
        );

        cleanup(state, dir).await;
    }

    /// MongoDB 备份/恢复目标命名空间参数：显式非空 ns → 仅该命名空间；缺省/空白 → 全部
    #[test]
    fn test_mongodb_backup_ns_param() {
        // query 显式 ns
        let params: HashMap<String, String> = [("ns".into(), "alice".into())].into_iter().collect();
        assert_eq!(mongo_backup_ns(&params, None), "alice");
        // body 显式 ns 优先级高于 query（param_of 语义）
        let body = serde_json::from_value::<serde_json::Value>(json!({"ns": "bob"})).unwrap();
        assert_eq!(mongo_backup_ns(&params, Some(&body)), "bob");
        // 空白 ns → 视为未指定（遍历全部）
        let blank: HashMap<String, String> = [("ns".into(), "   ".into())].into_iter().collect();
        assert_eq!(mongo_backup_ns(&blank, None), "");
        // 未传 ns → 遍历全部
        assert_eq!(mongo_backup_ns(&HashMap::new(), None), "");
    }

    /// F-28：替换规则 API——保存（缺 id 自动补）/列表/批量/删除/校验
    #[tokio::test]
    async fn test_replace_rules_api() {
        let (state, dir) = test_state("replapi").await;

        // 空名称/空 find → 校验失败
        let body = Bytes::from(r#"{"name":"","find":"a"}"#);
        let ret = save_replace_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "名称不能为空");
        let body = Bytes::from(r#"{"name":"规则","find":""}"#);
        let ret = save_replace_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "规则不能为空");

        // 保存（无 id → 后端补 uuid；legacy 字段名 pattern/replacement/isEnabled 兼容）
        let body = Bytes::from(
            r#"{"name":"净化","pattern":"口口","replacement":"","scope":"content","scopeTitle":false,"scopeContent":true,"isEnabled":true,"isRegex":true,"timeoutMillisecond":5000,"order":1}"#,
        );
        let ret = save_replace_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "保存应成功: {}", ret.0.error_msg);

        // 列表
        let ret = get_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "净化");
        assert_eq!(arr[0]["find"], "口口");
        assert_eq!(arr[0]["replace"], "");
        assert_eq!(arr[0]["scope"], "content");
        assert_eq!(arr[0]["scopeContent"], true);
        assert_eq!(arr[0]["isRegex"], true);
        assert_eq!(arr[0]["timeoutMillisecond"], 5000);
        assert!(
            arr[0]["id"].as_str().unwrap().starts_with("rule-"),
            "缺 id 应自动补: {arr:?}"
        );

        // 批量
        let batch = serde_json::json!([
            { "id": "b1", "name": "批量1", "find": "x", "replace": "y", "enabled": true, "order": 0 },
            { "name": "批量2", "find": "z", "enabled": false, "order": 1 },
        ]);
        let ret = save_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(batch.to_string())),
        )
        .await;
        assert!(ret.0.is_success, "批量保存应成功: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let ret = get_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data.as_array().unwrap().len(), 3);

        // 批量含空 find → 整批拒绝
        let batch = serde_json::json!([{ "name": "a", "find": "" }]);
        let ret = save_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(batch.to_string())),
        )
        .await;
        assert!(!ret.0.is_success);

        // 删除
        let body = Bytes::from(r#"{"id":"b1"}"#);
        let ret = delete_replace_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let ret = get_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data.as_array().unwrap().len(), 2);
        // query 参数删除
        let params: HashMap<String, String> = [("id".into(), "b1".into())].into_iter().collect();
        let ret = delete_replace_rule(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "重复删除不报错");
        // 缺 id
        let ret = delete_replace_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);

        cleanup(state, dir).await;
    }

    /// F-28 批量：deleteReplaceRules——{ids} / {all:true} / legacy 数组；单事务
    #[tokio::test]
    async fn test_delete_replace_rules_api() {
        let (state, dir) = test_state("delrepl").await;
        let storage = state.storage.clone();
        let save = move |id: String, name: String| {
            let storage = storage.clone();
            let rule = crate::model::ReplaceRule {
                id,
                name,
                find: "f".into(),
                replace: String::new(),
                enabled: true,
                order: 0,
                user_namespace: "default".into(),
                ..Default::default()
            };
            async move { storage.save_replace_rule("default", &rule).await }
        };
        save("r1".into(), "规则一".into()).await.unwrap();
        save("r2".into(), "规则二".into()).await.unwrap();
        save("r3".into(), "规则三".into()).await.unwrap();

        // {ids} 删指定
        let body = Bytes::from(r#"{"ids":["r1","r3"]}"#);
        let ret = delete_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "deleteReplaceRules 应成功: {}",
            ret.0.error_msg
        );
        assert_eq!(ret.0.data["count"], 2);
        let rules = state.storage.get_replace_rules("default").await.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "r2");

        // 空 ids → 参数错误
        let body = Bytes::from(r#"{"ids":[]}"#);
        let ret = delete_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "参数错误");

        // {all:true} 清空
        let body = Bytes::from(r#"{"all":true}"#);
        let ret = delete_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["count"], 1);
        assert!(state
            .storage
            .get_replace_rules("default")
            .await
            .unwrap()
            .is_empty());

        // legacy 原始数组（规则对象；取 id，缺 id 用 name 兜底）
        save("r9".into(), "规则九".into()).await.unwrap();
        save("r8".into(), "规则八".into()).await.unwrap();
        let body = Bytes::from(r#"[{"id":"r9"},{"name":"规则八"}]"#);
        let ret = delete_replace_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "legacy 数组删除应成功: {}",
            ret.0.error_msg
        );
        assert_eq!(ret.0.data["count"], 2);
        assert!(state
            .storage
            .get_replace_rules("default")
            .await
            .unwrap()
            .is_empty());

        cleanup(state, dir).await;
    }

    /// F-28 批量：replaceRule/saveMulti——{items} 与 legacy 数组；逐条校验；单事务
    #[tokio::test]
    async fn test_replace_rule_save_multi_api() {
        let (state, dir) = test_state("savemulti").await;

        // {items} 批量保存
        let body = Bytes::from(
            r#"{"items":[{"id":"m1","name":"批量甲","find":"x","replace":"y","enabled":true,"order":0},{"name":"批量乙","find":"z","enabled":false,"order":1}]}"#,
        );
        let ret = save_replace_rule_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "saveMulti 应成功: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let rules = state.storage.get_replace_rules("default").await.unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "m1");
        assert!(
            rules.iter().any(|r| r.id.starts_with("rule-")),
            "缺 id 应自动补"
        );

        // legacy 原始数组
        let body = Bytes::from(r#"[{"name":"批量丙","find":"w"}]"#);
        let ret = save_replace_rule_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["count"], 1);

        // 含空 find → 整批拒绝（事务回滚：新项不落库）
        let body =
            Bytes::from(r#"{"items":[{"name":"坏项","find":""},{"name":"好项","find":"ok"}]}"#);
        let ret = save_replace_rule_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        let rules = state.storage.get_replace_rules("default").await.unwrap();
        assert_eq!(rules.len(), 3, "校验失败整批拒绝，不落库");
        assert!(!rules.iter().any(|r| r.name == "好项"));

        // 空 items → 参数错误
        let body = Bytes::from(r#"{"items":[]}"#);
        let ret = save_replace_rule_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);

        cleanup(state, dir).await;
    }

    /// F-26：HttpTTS API——保存（id 兜底 url）/列表（id+url 双字段）/删除
    #[tokio::test]
    async fn test_http_tts_api() {
        let (state, dir) = test_state("ttsapi").await;

        // 校验：缺 url/name
        let body = Bytes::from(r#"{"name":"甲"}"#);
        let ret = save_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "链接不能为空");
        let body = Bytes::from(r#"{"url":"https://t.com/a"}"#);
        let ret = save_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "名称不能为空");

        // 保存
        let body = Bytes::from(
            r#"{"name":"引擎甲","url":"https://t.com/a","type":0,"contentType":"audio/mpeg","concurrentRate":"0","loginUrl":"https://t.com/login","loginUi":"[{\"type\":\"input\"}]","header":"{\"X-Token\":\"a\"}","jsLib":"lib.js","enabledCookieJar":true,"loginCheckJs":"java.ajax('x')","lastUpdateTime":1700000000000}"#,
        );
        let ret = save_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "保存应成功: {}", ret.0.error_msg);
        // 只传 id（旧契约）→ url 兜底
        let body = Bytes::from(r#"{"id":"https://t.com/b","name":"引擎乙","type":1}"#);
        let ret = save_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);

        // 列表：id 与 url 同值
        let ret = get_http_tts_list(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let a = arr
            .iter()
            .find(|v| v["name"] == "引擎甲")
            .expect("应含引擎甲");
        assert_eq!(a["id"], a["url"]);
        assert_eq!(a["type"], 0);
        assert_eq!(a["contentType"], "audio/mpeg");
        assert_eq!(a["concurrentRate"], "0");
        assert_eq!(a["loginUrl"], "https://t.com/login");
        assert_eq!(a["loginUi"], r#"[{"type":"input"}]"#);
        assert_eq!(a["header"], r#"{"X-Token":"a"}"#);
        assert_eq!(a["jsLib"], "lib.js");
        assert_eq!(a["enabledCookieJar"], true);
        assert_eq!(a["loginCheckJs"], "java.ajax('x')");
        assert_eq!(a["lastUpdateTime"], 1700000000000_i64);

        // 同 url 覆盖不新增
        let body = Bytes::from(r#"{"name":"引擎甲v2","url":"https://t.com/a","type":0}"#);
        let ret = save_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let ret = get_http_tts_list(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data.as_array().unwrap().len(), 2);

        // 删除（按 id）
        let body = Bytes::from(r#"{"id":"https://t.com/a"}"#);
        let ret = delete_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let ret = get_http_tts_list(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data.as_array().unwrap().len(), 1);

        // 批量删除（{ids:[]} 与原始数组两种 body）
        let body = Bytes::from(r#"{"id":"https://t.com/a"}"#);
        let _ = delete_http_tts(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        let body = Bytes::from(r#"{"items":[{"name":"丁","url":"https://t.com/d","type":0}]}"#);
        let _ = save_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        let body = Bytes::from(r#"{"ids":["https://t.com/d"]}"#);
        let ret = delete_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "deleteHttpTTSs 应成功: {}",
            ret.0.error_msg
        );
        assert_eq!(ret.0.data["count"], 1);
        let list = state.storage.get_http_tts_list("default").await.unwrap();
        assert_eq!(list.len(), 1);

        // 空 ids → 参数错误
        let body = Bytes::from(r#"{"ids":[]}"#);
        let ret = delete_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);

        cleanup(state, dir).await;
    }

    /// F-26 批量：httpTTS/saveMulti——{items} 与 legacy 数组；id 兜底 url；逐条校验；单事务
    #[tokio::test]
    async fn test_http_tts_save_multi_api() {
        let (state, dir) = test_state("ttsmulti").await;

        // {items} 批量保存（含 id 兜底 url 的旧契约项）
        let body = Bytes::from(
            r#"{"items":[{"name":"甲","url":"https://t.com/a","type":0},{"id":"https://t.com/b","name":"乙","type":1}]}"#,
        );
        let ret = save_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "saveMulti 应成功: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let list = state.storage.get_http_tts_list("default").await.unwrap();
        assert_eq!(list.len(), 2);
        assert!(
            list.iter().any(|t| t.url == "https://t.com/b"),
            "id 应兜底 url"
        );

        // legacy 原始数组
        let body = Bytes::from(r#"[{"name":"丙","url":"https://t.com/c","type":0}]"#);
        let ret = save_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["count"], 1);

        // 含缺 url 且无 id 的项 → 整批拒绝（事务回滚）
        let body = Bytes::from(
            r#"{"items":[{"name":"坏项"},{"name":"好项","url":"https://t.com/good"}]}"#,
        );
        let ret = save_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "链接不能为空");
        let list = state.storage.get_http_tts_list("default").await.unwrap();
        assert_eq!(list.len(), 3, "校验失败整批拒绝，不落库");
        assert!(!list.iter().any(|t| t.url == "https://t.com/good"));

        // 空 items / 缺 name → 参数错误
        let body = Bytes::from(r#"{"items":[]}"#);
        let ret = save_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        let body = Bytes::from(r#"{"items":[{"url":"https://t.com/x"}]}"#);
        let ret = save_http_tts_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "名称不能为空");

        cleanup(state, dir).await;
    }

    /// 自定义 TXT 目录规则 API：默认规则 + 用户规则合并列表/保存/删除/导入默认
    #[tokio::test]
    async fn test_txt_toc_rules_api() {
        let (state, dir) = test_state("tocapi").await;
        let default_len = crate::service::local_book::DEFAULT_TOC_RULE_DEFS.len();

        // 初始：仅内置默认规则
        let ret = get_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), default_len);
        assert_eq!(arr[0]["id"], "default-1");
        assert!(arr[0]["enable"].as_bool().unwrap());
        assert_eq!(arr[0]["name"], "目录(去空白)");
        // legacy 默认集含禁用项——列表原样展示
        let disabled = arr
            .iter()
            .filter(|r| !r["enable"].as_bool().unwrap())
            .count();
        assert!(disabled > 0, "默认规则应保留 legacy 禁用项");

        // 保存自定义规则
        let body =
            Bytes::from(r#"{"name":"我的规则","rule":"^第.+章$","enable":true,"serialNumber":0}"#);
        let ret = save_txt_toc_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "保存应成功: {}", ret.0.error_msg);
        // 校验：空 name/rule
        let body = Bytes::from(r#"{"name":"","rule":"x"}"#);
        let ret = save_txt_toc_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "名称不能为空");
        let body = Bytes::from(r#"{"name":"x","rule":""}"#);
        let ret = save_txt_toc_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);

        // 列表：默认 + 自定义（自定义在尾部）
        let ret = get_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), default_len + 1);
        let last = arr.last().unwrap();
        assert_eq!(last["name"], "我的规则");
        assert_eq!(last["serialNumber"], 0);
        assert!(
            last["id"].as_str().unwrap().starts_with("toc-"),
            "缺 id 应自动补"
        );

        // 删除
        let id = last["id"].as_str().unwrap();
        let body = Bytes::from(format!(r#"{{"id":"{id}"}}"#));
        let ret = delete_txt_toc_rule(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let ret = get_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data.as_array().unwrap().len(), default_len);

        // 导入默认规则（用户规则中出现 default-* 副本）
        let ret = import_default_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["count"], default_len as i64);
        let ret = get_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), default_len * 2, "默认规则 + 用户导入副本");
        // 重复导入幂等
        let ret = import_default_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data["count"], default_len as i64);
        let ret = get_txt_toc_rules(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data.as_array().unwrap().len(), default_len * 2);

        cleanup(state, dir).await;
    }

    /// getSystemInfo：版本/端口/用户数/书数/书源数
    #[tokio::test]
    async fn test_get_system_info() {
        let (state, dir) = test_state("sysapi").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://book.com/a".into(),
                    name: "测试书".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s.com".into(),
                    book_source_name: "源A".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let ret = get_system_info(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(ret.0.data["port"], 8080, "默认端口");
        assert_eq!(ret.0.data["userCount"], 0);
        assert_eq!(ret.0.data["bookCount"], 1);
        assert_eq!(ret.0.data["bookSourceCount"], 1);
        // 真实内存（修复 Windows 全 0M bug）：legacy 字符串字段 + 结构化字段
        assert!(ret.0.data["freeMemory"].is_string());
        assert!(ret.0.data["totalMemory"].is_string());
        assert_ne!(
            ret.0.data["totalMemory"], "0M",
            "物理内存应真实读取（非 0M）"
        );
        assert_ne!(
            ret.0.data["freeMemory"], "0M",
            "可用内存应真实读取（非 0M）"
        );
        assert!(
            ret.0.data["memory"]["totalMb"].as_u64().unwrap() > 0,
            "memory.totalMb > 0"
        );
        assert!(
            ret.0.data["memory"]["availableMb"].as_u64().unwrap() > 0,
            "memory.availableMb > 0"
        );
        assert!(
            ret.0.data["memory"]["percent"].as_f64().unwrap() > 0.0,
            "memory.percent > 0"
        );
        assert!(
            ret.0.data["cpu"]["cores"].as_u64().unwrap() >= 1,
            "cpu.cores >= 1"
        );
        assert!(ret.0.data["requests"]["total"].is_number());
        assert!(ret.0.data["online"]["sessions"].is_number());
        assert!(
            ret.0.data["bookSource"]["successRate"].is_null()
                || ret.0.data["bookSource"]["successRate"].is_number()
        );

        cleanup(state, dir).await;
    }

    /// getServerStats：监控聚合结构断言（内存/CPU/请求/在线/书源/uptime）
    #[tokio::test]
    async fn test_get_server_stats_structure() {
        let (state, dir) = test_state("statsapi").await;
        // 模拟中间件已计入的请求（getServerStats 自身不依赖中间件计数）
        crate::service::monitor::record_request("/reader3/getBookshelf");
        crate::service::monitor::record_request("/reader3/getBookshelf");
        crate::service::monitor::record_request("/reader3/getBookSources");
        // 模拟最近一次书源检测
        crate::service::monitor::record_book_source_check("default", 10, 3);

        let ret = get_server_stats(AxumState(state.clone())).await;
        assert!(ret.0.is_success, "{} {}", ret.0.error_msg, ret.0.data);
        let d = &ret.0.data;
        // 版本/端口
        assert_eq!(d["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(d["port"], 8080);
        assert!(d["timestamp"].as_i64().unwrap() > 0);
        assert!(d["uptimeSeconds"].as_i64().unwrap() >= 0);
        // 内存：真实值（总量/可用/已用/进程 + 百分比）
        assert!(d["memory"]["totalMb"].as_u64().unwrap() > 0, "内存真实读取");
        assert!(d["memory"]["usedMb"].as_u64().unwrap() > 0);
        assert!(
            d["memory"]["processMb"].as_u64().unwrap() > 0,
            "进程内存真实读取"
        );
        let pct = d["memory"]["percent"].as_f64().unwrap();
        assert!((0.0..=100.0).contains(&pct), "内存百分比 0..=100: {pct}");
        // CPU：短采样 + 核心数
        let cpu = d["cpu"]["percent"].as_f64().unwrap();
        assert!((0.0..=100.0).contains(&cpu), "CPU 使用率 0..=100: {cpu}");
        assert!(d["cpu"]["cores"].as_u64().unwrap() >= 1);
        // 请求计数：总数/今日/按接口 Top（排序断言）
        assert!(d["requests"]["total"].as_u64().unwrap() >= 3);
        assert!(d["requests"]["today"].as_u64().unwrap() >= 3);
        let top = d["requests"]["topEndpoints"].as_array().unwrap();
        assert!(!top.is_empty());
        assert_eq!(top[0]["path"], "/reader3/getBookshelf");
        assert_eq!(top[0]["count"], 2, "同路径应聚合计数");
        // 在线会话：数字
        assert!(d["online"]["sessions"].as_i64().unwrap() >= 0);
        // 书源成功率：全局最近检测结果（其他检测测试可能并发写入）——只断言结构
        assert!(d["bookSource"]["total"].as_u64().is_some());
        assert!(d["bookSource"]["ok"].as_u64().is_some());
        assert!(d["bookSource"]["failed"].as_u64().is_some());
        match d["bookSource"]["successRate"].as_f64() {
            Some(rate) => assert!((0.0..=1.0).contains(&rate), "成功率 0..=1: {rate}"),
            None => assert!(d["bookSource"]["successRate"].is_null(), "未检测 → null"),
        }
        assert!(d["bookSource"]["note"].is_string());
        assert!(d["bookSource"]["checkedAt"].is_null() || d["bookSource"]["checkedAt"].is_number());

        cleanup(state, dir).await;
    }

    /// 书源导出：attachment + 内容为当前命名空间书源 JSON
    #[tokio::test]
    async fn test_export_book_sources() {
        let (state, dir) = test_state("expapi").await;
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s1.com".into(),
                    book_source_name: "源一".into(),
                    search_url: Some("https://s1.com/search".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s2.com".into(),
                    book_source_name: "源二".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let resp = export_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Disposition")
                .and_then(|v| v.to_str().ok()),
            Some("attachment; filename=bookSource.json")
        );
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("application/json; charset=utf-8")
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr
            .iter()
            .any(|v| v["bookSourceUrl"] == "https://s1.com" && v["bookSourceName"] == "源一"));
        // 空命名空间 → 合法空数组（含 default 回退，此处 default 有数据）
        let params: HashMap<String, String> = [("accessToken".into(), "ghost:tok".into())]
            .into_iter()
            .collect();
        let resp =
            export_book_sources(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            json.as_array().unwrap().len(),
            2,
            "非 secure 模式 accessToken 无效，仍走 default"
        );

        cleanup(state, dir).await;
    }

    /// 文件型本地书 TXT 目录：用户自定义规则生效（无规则回退默认规则）
    #[tokio::test]
    async fn test_txt_toc_rules_in_local_book_toc() {
        let (state, dir) = test_state("localtoc").await;
        // 写一个文件型本地书（storage/data/default/books/示例.txt）
        let file_dir = state
            .storage
            .config
            .storage_dir()
            .join("data/default/books");
        std::fs::create_dir_all(&file_dir).unwrap();
        let txt = "第一章 起点\n内容一。\n第二章 成长\n内容二。";
        std::fs::write(file_dir.join("示例.txt"), txt).unwrap();
        let book_url = "storage/data/default/books/示例.txt";

        // 无用户规则 → 默认规则分章（两章）
        let ret = get_book_toc_file(&state, "default", book_url)
            .await
            .expect("默认规则应可解析");
        let titles: Vec<&str> = ret
            .0
            .data
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["title"].as_str().unwrap())
            .collect();
        assert_eq!(titles, vec!["第一章 起点", "第二章 成长"]);

        // 用户规则只匹配「第二章」→ 第一章内容并入前置「正文」章
        state
            .storage
            .save_txt_toc_rule(
                "default",
                &crate::model::TxtTocRule {
                    id: "t1".into(),
                    name: "仅第二章".into(),
                    rule: r"^第二章.*$".into(),
                    enable: true,
                    serial_number: 0,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let ret = get_book_toc_file(&state, "default", book_url)
            .await
            .expect("自定义规则应可解析");
        let arr = ret.0.data.as_array().unwrap();
        let titles: Vec<&str> = arr.iter().map(|v| v["title"].as_str().unwrap()).collect();
        assert_eq!(
            titles,
            vec!["正文", "第二章 成长"],
            "用户规则应替代默认规则"
        );
        // 正文按同一规则读取（章索引一致）
        let url = arr[1]["url"].as_str().unwrap();
        let ret = get_book_content_file(&state, "default", url, false)
            .await
            .expect("正文应可解析");
        assert_eq!(ret.0.data["content"], "内容二。");

        // 禁用规则 → 回退默认
        state
            .storage
            .save_txt_toc_rule(
                "default",
                &crate::model::TxtTocRule {
                    id: "t1".into(),
                    name: "仅第二章".into(),
                    rule: r"^第二章.*$".into(),
                    enable: false,
                    serial_number: 0,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let ret = get_book_toc_file(&state, "default", book_url)
            .await
            .unwrap();
        let titles: Vec<&str> = ret
            .0
            .data
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["title"].as_str().unwrap())
            .collect();
        assert_eq!(
            titles,
            vec!["第一章 起点", "第二章 成长"],
            "禁用规则后回退默认"
        );

        cleanup(state, dir).await;
    }

    /// F-25：getTTSVoices——预置语音列表（zh-CN 晓晓 + en-US Aria）
    #[tokio::test]
    async fn test_get_tts_voices_api() {
        let (state, dir) = test_state("ttsvoices").await;
        let ret = get_tts_voices(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let arr = ret.0.data.as_array().unwrap();
        assert!(!arr.is_empty());
        assert!(arr
            .iter()
            .any(|v| v["value"] == "zh-CN-XiaoxiaoNeural" && v["name"] == "晓晓"));
        assert!(arr.iter().any(|v| v["value"] == "en-US-AriaNeural"));
        for v in arr {
            assert!(v["locale"].is_string() && v["gender"].is_string());
        }
        cleanup(state, dir).await;
    }

    /// F-25：tts 合成——参数校验（无 text / 未知引擎 / http 缺 url），不发起网络请求
    #[tokio::test]
    async fn test_tts_synthesize_param_validation() {
        let (state, dir) = test_state("ttsval").await;

        // 缺 text → 参数错误
        let ret = tts_synthesize(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = axum::body::to_bytes(ret.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());
        assert_eq!(json["errorMsg"], "参数错误");

        // 未知引擎 → 不支持的TTS引擎
        let params: HashMap<String, String> = [
            ("text".into(), "你好".into()),
            ("engine".into(), "nope".into()),
        ]
        .into_iter()
        .collect();
        let ret = tts_synthesize(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = axum::body::to_bytes(ret.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["errorMsg"], "听书源不存在");

        // http 引擎（F4 契约）：voice=听书源名，缺失/未命中 → 听书源不存在
        let params: HashMap<String, String> = [
            ("text".into(), "你好".into()),
            ("engine".into(), "http".into()),
        ]
        .into_iter()
        .collect();
        let ret = tts_synthesize(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = axum::body::to_bytes(ret.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());
        assert_eq!(json["errorMsg"], "听书源不存在");

        cleanup(state, dir).await;
    }

    /// F-32：getUsers——secureKey 校验（未登录/缺 key/错 key）+ 用户列表含启用状态
    #[tokio::test]
    async fn test_get_users_api() {
        let (state, dir) = test_state("getusers").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "admin".into(),
                token: "t1".into(),
                enable_webdav: true,
                enable_book_source: false,
                book_source_limit: 5,
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t2".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        // 未登录 → 请登录后使用
        let ret = get_users(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.data, json!("NEED_LOGIN"));

        // 已登录但缺 secureKey → NEED_SECURE_KEY
        let params: HashMap<String, String> = [("accessToken".into(), "admin:t1".into())]
            .into_iter()
            .collect();
        let ret = get_users(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        // 错 secureKey → NEED_SECURE_KEY
        let params: HashMap<String, String> = [
            ("accessToken".into(), "admin:t1".into()),
            ("secureKey".into(), "wrong".into()),
        ]
        .into_iter()
        .collect();
        let ret = get_users(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        // 正确 secureKey → 列表（含启用状态；不含密码字段）
        let params: HashMap<String, String> = [
            ("accessToken".into(), "admin:t1".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();
        let ret = get_users(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "getUsers 应成功: {}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let admin = arr.iter().find(|v| v["username"] == "admin").unwrap();
        assert_eq!(admin["enableWebdav"], true);
        assert_eq!(admin["enableBookSource"], false);
        assert_eq!(admin["bookSourceLimit"], 5);
        assert!(admin.get("password").is_none(), "列表不应泄露密码");
        assert!(admin.get("salt").is_none());
        assert!(admin.get("token").is_none());

        cleanup(state, dir).await;
    }

    /// F-32：updateUser——权限/限额更新（body 布尔 + query int），不存在用户报错
    #[tokio::test]
    async fn test_update_user_api() {
        let (state, dir) = test_state("upduser").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                enable_webdav: false,
                enable_local_store: false,
                enable_book_source: true,
                enable_rss_source: true,
                book_source_limit: 10,
                book_limit: 20,
                ..Default::default()
            })
            .await
            .unwrap();
        let auth = |extra: Vec<(&str, &str)>| -> HashMap<String, String> {
            let mut m: HashMap<String, String> = [
                ("accessToken".into(), "alice:t1".into()),
                ("secureKey".into(), "sk".into()),
            ]
            .into_iter()
            .collect();
            for (k, v) in extra {
                m.insert(k.into(), v.into());
            }
            m
        };

        // body：部分字段更新（camelCase）
        let body = Bytes::from(
            r#"{"username":"alice","enableWebdav":true,"enableBookSource":false,"bookLimit":99}"#,
        );
        let ret = update_user(
            AxumState(state.clone()),
            Query(auth(vec![])),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "updateUser 应成功: {}", ret.0.error_msg);
        let alice = state.storage.find_user("alice").await.unwrap().unwrap();
        assert!(alice.enable_webdav);
        assert!(!alice.enable_book_source);
        assert_eq!(alice.book_limit, 99);
        assert_eq!(alice.book_source_limit, 10, "未提供的字段保持原值");
        assert!(alice.enable_rss_source, "未提供的字段保持原值");

        // query 参数：int + bool
        let ret = update_user(
            AxumState(state.clone()),
            Query(auth(vec![
                ("username", "alice"),
                ("enableRssSource", "false"),
                ("bookSourceLimit", "7"),
            ])),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let alice = state.storage.find_user("alice").await.unwrap().unwrap();
        assert!(!alice.enable_rss_source);
        assert_eq!(alice.book_source_limit, 7);

        // 不存在的用户 → 用户不存在
        let body = Bytes::from(r#"{"username":"ghost","enableWebdav":true}"#);
        let ret = update_user(
            AxumState(state.clone()),
            Query(auth(vec![])),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "用户不存在");

        // 缺 username → 参数错误；缺 secureKey → NEED_SECURE_KEY
        let body = Bytes::from(r#"{"enableWebdav":true}"#);
        let ret = update_user(
            AxumState(state.clone()),
            Query(auth(vec![])),
            HeaderMap::new(),
            Some(body.clone()),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let no_key: HashMap<String, String> = [("accessToken".into(), "alice:t1".into())]
            .into_iter()
            .collect();
        let ret = update_user(
            AxumState(state.clone()),
            Query(no_key),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        cleanup(state, dir).await;
    }

    /// F-32：addUser——管理员创建用户；缺省权限全开 + 80000/5000；isAdmin/上限可指定
    #[tokio::test]
    async fn test_add_user_api() {
        let (state, dir) = test_state("adduser").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state.storage.config.default_user_enable_webdav = true;
        state.storage.config.default_user_enable_local_store = true;
        state.storage.config.default_user_enable_book_source = true;
        state.storage.config.default_user_enable_rss_source = true;
        state.storage.config.default_user_book_source_limit = 80000;
        state.storage.config.default_user_book_limit = 5000;
        state
            .storage
            .insert_user(&User {
                username: "admin".into(),
                token: "t1".into(),
                is_admin: true,
                ..Default::default()
            })
            .await
            .unwrap();
        let auth: HashMap<String, String> = [
            ("accessToken".into(), "admin:t1".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();

        // 缺省权限：全开 + 80000/5000，非管理员
        let body = Bytes::from(r#"{"username":"bobuser","password":"pass1234"}"#);
        let ret = add_user(
            AxumState(state.clone()),
            Query(auth.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "addUser 应成功: {}", ret.0.error_msg);
        let bob = state.storage.find_user("bobuser").await.unwrap().unwrap();
        assert!(bob.enable_webdav && bob.enable_local_store);
        assert!(bob.enable_book_source && bob.enable_rss_source);
        assert_eq!(bob.book_source_limit, 80000);
        assert_eq!(bob.book_limit, 5000);
        assert!(!bob.is_admin);

        // 指定管理员 + 自定义上限
        let body = Bytes::from(
            r#"{"username":"rootuser","password":"pass1234","isAdmin":true,"bookSourceLimit":123,"bookLimit":45}"#,
        );
        let ret = add_user(
            AxumState(state.clone()),
            Query(auth.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "addUser 应成功: {}", ret.0.error_msg);
        let root = state.storage.find_user("rootuser").await.unwrap().unwrap();
        assert!(root.is_admin);
        assert_eq!(root.book_source_limit, 123);
        assert_eq!(root.book_limit, 45);

        // 重复用户名 → 用户名已被占用
        let body = Bytes::from(r#"{"username":"bobuser","password":"pass1234"}"#);
        let ret = add_user(
            AxumState(state.clone()),
            Query(auth.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "用户名已被占用");

        // 缺 secureKey → NEED_SECURE_KEY
        let no_key: HashMap<String, String> = [("accessToken".into(), "admin:t1".into())]
            .into_iter()
            .collect();
        let ret = add_user(
            AxumState(state.clone()),
            Query(no_key),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        cleanup(state, dir).await;
    }

    /// 管理员默认本人命名空间；显式 ns=default 进入系统配置层；普通用户忽略
    #[tokio::test]
    async fn test_admin_namespace_resolution() {
        let (state, dir) = test_state("admins").await;
        let mut state = state;
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "admin".into(),
                token: "t1".into(),
                is_admin: true,
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t2".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let auth = |u: &str, t: &str| -> HashMap<String, String> {
            [("accessToken".into(), format!("{u}:{t}"))]
                .into_iter()
                .collect()
        };
        let auth_ns_default = |u: &str, t: &str| -> HashMap<String, String> {
            [
                ("accessToken".into(), format!("{u}:{t}")),
                ("ns".into(), "default".into()),
            ]
            .into_iter()
            .collect()
        };
        assert_eq!(
            resolve_namespace(&state, &auth("admin", "t1"), &HeaderMap::new())
                .await
                .unwrap(),
            "admin",
            "管理员默认使用本人账号命名空间"
        );
        assert_eq!(
            resolve_namespace(&state, &auth_ns_default("admin", "t1"), &HeaderMap::new())
                .await
                .unwrap(),
            "default",
            "管理员显式 ns=default 进入系统配置层"
        );
        assert_eq!(
            resolve_namespace(&state, &auth("alice", "t2"), &HeaderMap::new())
                .await
                .unwrap(),
            "alice",
            "普通用户保持本人命名空间"
        );
        assert_eq!(
            resolve_namespace(&state, &auth_ns_default("alice", "t2"), &HeaderMap::new())
                .await
                .unwrap(),
            "alice",
            "普通用户即使传 ns=default 也保持本人命名空间"
        );

        cleanup(state, dir).await;
    }

    /// 最后一名管理员禁止撤销/删除；存在第二位管理员后允许撤销
    #[tokio::test]
    async fn test_last_admin_protection() {
        let (state, dir) = test_state("lastadmin").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "admin".into(),
                token: "t1".into(),
                is_admin: true,
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "op".into(),
                token: "t3".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let auth: HashMap<String, String> = [
            ("accessToken".into(), "op:t3".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();

        // 撤销最后一名管理员 → 拒绝
        let body = Bytes::from(r#"{"username":"admin","isAdmin":false}"#);
        let ret = update_user(
            AxumState(state.clone()),
            Query(auth.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "不能撤销最后一名管理员");

        // 删除最后一名管理员 → 拒绝
        let body = Bytes::from(r#"{"username":"admin"}"#);
        let ret = delete_user(
            AxumState(state.clone()),
            Query(auth.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "不能删除最后一名管理员");

        // 第二位管理员加入后允许撤销
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                token: "t2".into(),
                is_admin: true,
                ..Default::default()
            })
            .await
            .unwrap();
        let body = Bytes::from(r#"{"username":"admin","isAdmin":false}"#);
        let ret = update_user(
            AxumState(state.clone()),
            Query(auth.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "第二位管理员存在时允许撤销: {}",
            ret.0.error_msg
        );
        assert!(
            !state
                .storage
                .find_user("admin")
                .await
                .unwrap()
                .unwrap()
                .is_admin
        );

        cleanup(state, dir).await;
    }

    /// F-32：deleteUser——不能删自己；删他人成功；secureKey 校验
    #[tokio::test]
    async fn test_delete_user_api() {
        let (state, dir) = test_state("deluser").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "admin".into(),
                token: "t1".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                token: "t2".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [
            ("accessToken".into(), "admin:t1".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();

        // 删自己 → 拒绝
        let body = Bytes::from(r#"{"username":"admin"}"#);
        let ret = delete_user(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "不能删除自己");
        assert!(state.storage.find_user("admin").await.unwrap().is_some());

        // 删他人 → 成功
        let body = Bytes::from(r#"{"username":"bob"}"#);
        let ret = delete_user(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "deleteUser 应成功: {}", ret.0.error_msg);
        assert!(state.storage.find_user("bob").await.unwrap().is_none());

        // 不存在 → 用户不存在；缺 secureKey → NEED_SECURE_KEY
        let body = Bytes::from(r#"{"username":"ghost"}"#);
        let ret = delete_user(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body.clone()),
        )
        .await;
        assert_eq!(ret.0.error_msg, "用户不存在");
        let no_key: HashMap<String, String> = [("accessToken".into(), "admin:t1".into())]
            .into_iter()
            .collect();
        let ret = delete_user(
            AxumState(state.clone()),
            Query(no_key),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        cleanup(state, dir).await;
    }

    /// F-32 批量：deleteUsers——{usernames} 与 legacy 数组；不能删自己；单事务；返回剩余用户列表
    #[tokio::test]
    async fn test_delete_users_api() {
        let (state, dir) = test_state("delusers").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        for (u, t) in [("admin", "t1"), ("alice", "t2"), ("bob", "t3")] {
            state
                .storage
                .insert_user(&User {
                    username: u.into(),
                    token: t.into(),
                    ..Default::default()
                })
                .await
                .unwrap();
        }
        // alice 有一本书（验证用户数据随删除清理）
        let book = crate::model::Book {
            book_url: "local://alice-book".into(),
            name: "Alice 的书".into(),
            user_namespace: "alice".into(),
            ..Default::default()
        };
        state.storage.upsert_book("alice", &book).await.unwrap();
        let params: HashMap<String, String> = [
            ("accessToken".into(), "admin:t1".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();

        // {usernames} 批量删除（含不存在的 ghost——跳过）
        let body = Bytes::from(r#"{"usernames":["alice","bob","ghost"]}"#);
        let ret = delete_users(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "deleteUsers 应成功: {}", ret.0.error_msg);
        // 返回剩余用户列表
        let arr = ret.0.data.as_array().expect("返回用户列表");
        assert!(arr
            .iter()
            .all(|v| v["username"] != "alice" && v["username"] != "bob"));
        assert!(arr.iter().any(|v| v["username"] == "admin"));
        assert!(state.storage.find_user("alice").await.unwrap().is_none());
        assert!(state.storage.find_user("bob").await.unwrap().is_none());
        // 用户数据清理：alice 的书没了
        assert!(state
            .storage
            .find_book("alice", "local://alice-book")
            .await
            .unwrap()
            .is_none());

        // legacy 原始数组
        let body = Bytes::from(r#"["ghost"]"#);
        let ret = delete_users(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "legacy 数组应成功: {}", ret.0.error_msg);

        // 不能删除自己（admin 是当前命名空间）
        let body = Bytes::from(r#"{"usernames":["admin"]}"#);
        let ret = delete_users(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "跳过自己不报错");
        assert!(
            state.storage.find_user("admin").await.unwrap().is_some(),
            "自己不能被删"
        );

        // 空 usernames → 参数错误；缺 secureKey → NEED_SECURE_KEY
        let body = Bytes::from(r#"{"usernames":[]}"#);
        let ret = delete_users(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "参数错误");
        let no_key: HashMap<String, String> = [("accessToken".into(), "admin:t1".into())]
            .into_iter()
            .collect();
        let body = Bytes::from(r#"{"usernames":["x"]}"#);
        let ret = delete_users(
            AxumState(state.clone()),
            Query(no_key),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));

        cleanup(state, dir).await;
    }

    /// F-32：resetUserPassword——新密码生效（genEncryptedPassword 可校验）+ token 失效；secureKey 校验
    #[tokio::test]
    async fn test_reset_user_password_api() {
        let (state, dir) = test_state("resetpw").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                password: "old".into(),
                salt: "oldsalt".into(),
                token: "t1".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [
            ("accessToken".into(), "alice:t1".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();

        // body：username + newPassword
        let body = Bytes::from(r#"{"username":"alice","newPassword":"新密码abc"}"#);
        let ret = reset_user_password(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(
            ret.0.is_success,
            "resetUserPassword 应成功: {}",
            ret.0.error_msg
        );
        let alice = state.storage.find_user("alice").await.unwrap().unwrap();
        assert_ne!(alice.password, "old");
        assert_ne!(alice.salt, "oldsalt", "salt 应重新生成");
        assert!(alice.token.is_empty(), "旧 token 应失效");
        assert!(
            alice.password.starts_with("$argon2id$"),
            "重置后密码应为 argon2id PHC: {}",
            alice.password
        );
        assert!(
            crate::util::password::verify_argon2id("新密码abc", &alice.password),
            "新密码应可通过 argon2id 校验"
        );

        // 重置后旧 token 已失效——重新登录（新 token）以便继续测试管理接口
        state
            .storage
            .update_user_session("alice", "t2", now_millis())
            .await
            .unwrap();
        let params: HashMap<String, String> = [
            ("accessToken".into(), "alice:t2".into()),
            ("secureKey".into(), "sk".into()),
        ]
        .into_iter()
        .collect();

        // query：password 参数；不存在 → 用户不存在；缺 secureKey → NEED_SECURE_KEY
        let mut q = params.clone();
        q.insert("username".into(), "ghost".into());
        q.insert("password".into(), "whatever1".into());
        let ret =
            reset_user_password(AxumState(state.clone()), Query(q), HeaderMap::new(), None).await;
        assert_eq!(ret.0.error_msg, "用户不存在");
        let no_key: HashMap<String, String> = [("accessToken".into(), "alice:t2".into())]
            .into_iter()
            .collect();
        let ret = reset_user_password(
            AxumState(state.clone()),
            Query(no_key),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data, json!("NEED_SECURE_KEY"));
        // 缺密码 → 参数错误
        let body = Bytes::from(r#"{"username":"alice"}"#);
        let ret = reset_user_password(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// 微型 HTTP 服务器：按 bodies 顺序应答每次请求（耗尽后重复最后一个）；返回 URL
    async fn serve_bodies(bodies: Vec<String>) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let bodies = std::sync::Arc::new(std::sync::Mutex::new(bodies));
            for _ in 0..10 {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 2048];
                let _ = sock.read(&mut buf).await;
                let body = {
                    let mut b = bodies.lock().unwrap();
                    if b.len() > 1 {
                        b.remove(0)
                    } else {
                        b[0].clone()
                    }
                };
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}/sources.json")
    }

    /// 按请求路径返回响应体的 mock（并发抓取场景确定性——请求到达序 ≠ 队列序时
    /// 路径寻址保证每个 URL 拿到自己的响应；普通顺序场景仍用 serve_bodies）
    async fn serve_bodies_by_path(entries: Vec<(String, String)>) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let map: std::sync::Arc<std::collections::HashMap<String, String>> =
                std::sync::Arc::new(entries.into_iter().collect());
            for _ in 0..10 {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 2048];
                let _ = sock.read(&mut buf).await;
                let req = String::from_utf8_lossy(&buf);
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                let body = map.get(&path).cloned().unwrap_or_default();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}/sources.json")
    }

    /// 缓存管理 API：getCacheInfo 统计 + clearCache 按 type 清空 + 参数校验
    #[tokio::test]
    async fn test_cache_api() {
        let (state, dir) = test_state("cacheapi").await;
        state
            .storage
            .cache_toc(
                "default",
                "https://book.com/a",
                "https://book.com/toc",
                "[]",
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "local://book1",
                &[
                    ("第一章".to_string(), "正文一甲乙".to_string()),
                    ("第二章".to_string(), "正文二丙丁戊".to_string()),
                ],
            )
            .await
            .unwrap();

        // getCacheInfo：统计字段（camelCase）
        let ret = get_cache_info(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["tocCacheCount"], 1);
        assert_eq!(ret.0.data["tocCacheSize"], 2, "SQLite length() 按字符计");
        assert_eq!(ret.0.data["chapterCount"], 2);
        assert_eq!(ret.0.data["chapterSize"], 11, "5+6 字符");
        assert_eq!(ret.0.data["totalSize"], 13);

        // clearCache：type=toc（body）
        let body = Bytes::from(r#"{"type":"toc"}"#);
        let ret = clear_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["deletedToc"], 1);
        assert_eq!(ret.0.data["deletedChapters"], 0);
        let info = state.storage.get_cache_info().await.unwrap();
        assert_eq!(info.toc_cache_count, 0);
        assert_eq!(info.chapter_count, 2);

        // clearCache：type=chapters（query）
        let params: HashMap<String, String> =
            [("type".into(), "chapters".into())].into_iter().collect();
        let ret = clear_cache(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["deletedChapters"], 2);

        // 非法 type → 参数错误
        let body = Bytes::from(r#"{"type":"books"}"#);
        let ret = clear_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "参数错误");
        // 空 body + 空 query → 默认 all（成功）
        let ret = clear_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);

        cleanup(state, dir).await;
    }

    /// P0-9：getCacheInfo/clearCache 需登录——secure 模式匿名请求拒绝 NEED_LOGIN（且不清数据），
    /// 携带有效 accessToken 后可用
    #[tokio::test]
    async fn test_cache_api_requires_login() {
        let (mut state, dir) = test_state("cacheauth").await;
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "tok".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .cache_toc(
                "default",
                "https://book.com/a",
                "https://book.com/toc",
                "[]",
            )
            .await
            .unwrap();

        // 匿名（无 accessToken）→ 两个接口均 NEED_LOGIN
        let ret = get_cache_info(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.data, json!("NEED_LOGIN"));
        let ret = clear_cache(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.data, json!("NEED_LOGIN"));
        // 数据未被清除
        let info = state.storage.get_cache_info().await.unwrap();
        assert_eq!(info.toc_cache_count, 1);
        assert_eq!(info.chapter_count, 0);

        // 携带有效 accessToken → 统计可用、清理可执行
        let params: HashMap<String, String> = [("accessToken".into(), "alice:tok".into())]
            .into_iter()
            .collect();
        let ret = get_cache_info(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["tocCacheCount"], 1);
        let ret = clear_cache(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["deletedToc"], 1);
        let info = state.storage.get_cache_info().await.unwrap();
        assert_eq!(info.toc_cache_count, 0);

        cleanup(state, dir).await;
    }

    /// 全书搜索 API：本地书命中（chapterIndex/title/snippet）/ 书源书提示 / 参数校验
    #[tokio::test]
    async fn test_search_book_content_api() {
        let (state, dir) = test_state("searchapi").await;
        state
            .storage
            .save_chapters(
                "default",
                "local://book1",
                &[
                    (
                        "第一章".to_string(),
                        "这是第一章的正文，关键词出现了。".to_string(),
                    ),
                    ("第二章".to_string(), "没有匹配。".to_string()),
                ],
            )
            .await
            .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "local://book1".into(),
                    name: "本地书".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://book.com/web".into(),
                    name: "网文书".into(),
                    origin: "https://source.com".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // GET：key + bookUrl → 命中列表
        let params: HashMap<String, String> = [
            ("key".into(), "关键词".into()),
            ("bookUrl".into(), "local://book1".into()),
        ]
        .into_iter()
        .collect();
        let ret = search_book_content(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let hits = ret.0.data.as_array().unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["chapterIndex"], 0);
        assert_eq!(hits[0]["title"], "第一章");
        assert!(hits[0]["snippet"].as_str().unwrap().contains("关键词"));

        // POST body 变体
        let body = Bytes::from(r#"{"key":"关键词","bookUrl":"local://book1"}"#);
        let ret = search_book_content(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data.as_array().unwrap().len(), 1);

        // 书源书 → 仅支持本地书内容搜索
        let params: HashMap<String, String> = [
            ("key".into(), "关键词".into()),
            ("bookUrl".into(), "https://book.com/web".into()),
        ]
        .into_iter()
        .collect();
        let ret = search_book_content(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "仅支持本地书内容搜索");

        // 不存在的书 → 书籍不存在；缺 key / 缺 bookUrl → 参数错误
        let params: HashMap<String, String> = [
            ("key".into(), "关键词".into()),
            ("bookUrl".into(), "local://ghost".into()),
        ]
        .into_iter()
        .collect();
        let ret = search_book_content(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书籍不存在");
        let params: HashMap<String, String> = [("bookUrl".into(), "local://book1".into())]
            .into_iter()
            .collect();
        let ret = search_book_content(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "请输入搜索关键字");
        let params: HashMap<String, String> =
            [("key".into(), "关键词".into())].into_iter().collect();
        let ret = search_book_content(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// 书源订阅 API：saveSourceSub 抓取校验+批量导入 / getSourceSubs / refreshSourceSub 覆盖 /
    /// deleteSourceSub / 格式错误与上限校验
    #[tokio::test]
    async fn test_source_sub_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("subsapi").await;
        let v1 = r#"[{"bookSourceUrl":"https://s1.com","bookSourceName":"源1"},{"bookSourceUrl":"https://s2.com","bookSourceName":"源2"}]"#;
        let v2 = r#"[{"bookSourceUrl":"https://s1.com","bookSourceName":"源1v2"},{"bookSourceUrl":"https://s2.com","bookSourceName":"源2"},{"bookSourceUrl":"https://s3.com","bookSourceName":"源3"}]"#;
        let sub_url = serve_bodies(vec![v1.to_string(), v2.to_string()]).await;

        // saveSourceSub：抓取 → 校验 → 订阅入库 + 批量导入书源
        let body = Bytes::from(format!(r#"{{"url":"{sub_url}","name":"全量书源"}}"#));
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let subs = state.storage.get_source_subs("default").await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].url, sub_url);
        assert_eq!(subs[0].name, "全量书源");
        assert_eq!(subs[0].raw_json.as_deref(), Some(v1), "raw_json 存抓取原文");
        assert_eq!(
            state.storage.count_book_sources("default").await.unwrap(),
            2,
            "书源已批量导入"
        );
        let s1 = state
            .storage
            .find_book_source("default", "https://s1.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s1.book_source_name, "源1");

        // getSourceSubs：列表（url/name/enabled）
        let ret = get_source_subs(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let list = ret.0.data.as_array().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["url"], sub_url.as_str());
        assert_eq!(list[0]["name"], "全量书源");
        assert_eq!(list[0]["enabled"], true, "订阅默认启用");

        // setSourceSubEnabled：禁用保留订阅记录，列表 enabled=false
        let body = Bytes::from(format!(r#"{{"url":"{sub_url}","enabled":false}}"#));
        let ret = set_source_sub_enabled(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let subs = state.storage.get_source_subs("default").await.unwrap();
        assert_eq!(subs.len(), 1, "禁用不删除订阅记录");
        assert!(!subs[0].enabled);
        let ret = get_source_subs(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data[0]["enabled"], false);
        // 重新启用
        let body = Bytes::from(format!(r#"{{"url":"{sub_url}","enabled":true}}"#));
        let ret = set_source_sub_enabled(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let subs = state.storage.get_source_subs("default").await.unwrap();
        assert!(subs[0].enabled);

        // refreshSourceSub：重新拉取 → 覆盖订阅 raw_json + 覆盖/新增书源
        let body = Bytes::from(format!(r#"{{"url":"{sub_url}"}}"#));
        let ret = refresh_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 3);
        let subs = state.storage.get_source_subs("default").await.unwrap();
        assert_eq!(subs[0].name, "全量书源", "刷新保留原订阅名");
        assert_eq!(subs[0].raw_json.as_deref(), Some(v2));
        assert_eq!(
            state.storage.count_book_sources("default").await.unwrap(),
            3
        );
        let s1 = state
            .storage
            .find_book_source("default", "https://s1.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s1.book_source_name, "源1v2", "已存在书源覆盖更新");

        // refresh 不存在的订阅 → 订阅不存在
        let body = Bytes::from(r#"{"url":"http://127.0.0.1:1/nope.json"}"#);
        let ret = refresh_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "订阅不存在");

        // deleteSourceSub：删除订阅，不影响已导入书源
        let body = Bytes::from(format!(r#"{{"url":"{sub_url}"}}"#));
        let ret = delete_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(state
            .storage
            .get_source_subs("default")
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            state.storage.count_book_sources("default").await.unwrap(),
            3,
            "书源保留"
        );

        // 缺 url → 参数校验
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "请输入订阅链接");
        let ret = delete_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        // 抓取失败（连接拒绝）→ 远程书源链接错误
        let body = Bytes::from(r#"{"url":"http://127.0.0.1:1/x.json","name":"坏链接"}"#);
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "远程书源链接错误");

        // 非 JSON / 非书源数组 / 空数组 / 缺 bookSourceUrl → 书源数据格式错误
        let bad_url = serve_bodies(vec!["not json".to_string()]).await;
        let body = Bytes::from(format!(r#"{{"url":"{bad_url}"}}"#));
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源数据格式错误");
        let bad_url = serve_bodies(vec!["[{\"foo\":1}]".to_string()]).await;
        let body = Bytes::from(format!(r#"{{"url":"{bad_url}"}}"#));
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源数据格式错误");
        let bad_url = serve_bodies(vec!["[]".to_string()]).await;
        let body = Bytes::from(format!(r#"{{"url":"{bad_url}"}}"#));
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源数据格式错误");

        // 书源数上限：limit=1 时导入 2 个新源 → 超过书源数上限
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_source_limit: 1,
                ..Default::default()
            })
            .await
            .unwrap();
        let ok_url = serve_bodies(vec![v1.to_string()]).await;
        let body = Bytes::from(format!(r#"{{"url":"{ok_url}"}}"#));
        let ret = save_source_sub(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "超过书源数上限");
        assert!(
            state
                .storage
                .get_source_subs("default")
                .await
                .unwrap()
                .is_empty(),
            "超限时订阅不落库"
        );

        cleanup(state, dir).await;
    }

    /// 分组收尾：getBookGroups 输出 {id,name,order,orderNum,bookCount}（COUNT 子查询）
    #[tokio::test]
    async fn test_book_groups_with_count_api() {
        let (state, dir) = test_state("grpcnt").await;
        let g1 = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "玄幻".into(),
                    order: 1,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let g2 = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "言情".into(),
                    order: 2,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        for url in ["https://b.com/1", "https://b.com/2", "https://b.com/3"] {
            state
                .storage
                .upsert_book(
                    "default",
                    &crate::model::Book {
                        book_url: url.into(),
                        name: url.into(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
        }
        state
            .storage
            .update_book_group_id("default", "https://b.com/1", g1.id)
            .await
            .unwrap();
        state
            .storage
            .update_book_group_id("default", "https://b.com/2", g1.id)
            .await
            .unwrap();
        state
            .storage
            .update_book_group_id("default", "https://b.com/3", g2.id)
            .await
            .unwrap();

        let ret = get_book_groups(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], g1.id);
        assert_eq!(arr[0]["name"], "玄幻");
        assert_eq!(arr[0]["order"], 1, "legacy order 字段保留");
        assert_eq!(arr[0]["orderNum"], 1, "orderNum 别名同值");
        assert_eq!(arr[0]["bookCount"], 2, "组内书数");
        assert_eq!(arr[1]["name"], "言情");
        assert_eq!(arr[1]["bookCount"], 1);
        assert_eq!(arr[1]["orderNum"], 2);

        cleanup(state, dir).await;
    }

    /// 分组收尾：saveBookGroup 仅 {id,name} → 重命名保留 order；deleteBookGroup 组内书置 0
    #[tokio::test]
    async fn test_save_book_group_rename_and_delete_api() {
        let (state, dir) = test_state("grpren").await;

        // 新建 {name, order}
        let body = Bytes::from(r#"{"name":"玄幻","order":3}"#);
        let ret = save_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let gid = ret.0.data["id"].as_i64().expect("新建应返回 id");

        // 仅 {id,name} → 重命名（order 保留）
        let body = Bytes::from(format!(r#"{{"id":{gid},"name":"玄幻v2"}}"#));
        let ret = save_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let list = state
            .storage
            .list_book_groups_with_count("default")
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "玄幻v2");
        assert_eq!(list[0].order, 3, "重命名应保留排序");
        assert_eq!(list[0].id, gid);

        // 重命名不存在的分组 → 分组不存在；空名称 → 分组名称不能为空；非 JSON → 参数错误
        let body = Bytes::from(r#"{"id":9999,"name":"幽灵"}"#);
        let ret = save_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "分组不存在");
        let body = Bytes::from(format!(r#"{{"id":{gid},"name":""}}"#));
        let ret = save_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "分组名称不能为空");
        let ret = save_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from("nope")),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        // 带 order 的 {id,name,order} → 仍走全量覆盖（兼容旧行为）
        let body = Bytes::from(format!(r#"{{"id":{gid},"name":"玄幻v3","order":9}}"#));
        let ret = save_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let list = state
            .storage
            .list_book_groups_with_count("default")
            .await
            .unwrap();
        assert_eq!(list[0].name, "玄幻v3");
        assert_eq!(list[0].order, 9);

        // 删除：组内书 group 置 0，分组移除
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://b.com/1".into(),
                    name: "书1".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .update_book_group_id("default", "https://b.com/1", gid)
            .await
            .unwrap();
        let body = Bytes::from(format!(r#"{{"id":{gid}}}"#));
        let ret = delete_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/1")
                .await
                .unwrap()
                .unwrap()
                .group,
            0,
            "组内书应置 0"
        );
        assert!(state
            .storage
            .list_book_groups("default")
            .await
            .unwrap()
            .is_empty());

        // 再删 → 分组不存在；缺 id → 参数错误；query 形式 id 同样生效
        let body = Bytes::from(format!(r#"{{"id":{gid}}}"#));
        let ret = delete_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "分组不存在");
        let ret = delete_book_group(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from(r#"{"name":"x"}"#)),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let params: HashMap<String, String> =
            [("id".into(), gid.to_string())].into_iter().collect();
        let ret = delete_book_group(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "分组不存在", "query 形式 id 应被识别");

        cleanup(state, dir).await;
    }

    /// F-10 正文缓存：getBookContent 先查 book_chapters（chapterUrl md5 键）命中直读，
    /// 抓取成功后写回；local:// 与缺 bookUrl 不参与缓存
    #[tokio::test]
    async fn test_get_book_content_cache_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("contentcache").await;
        let base_url = serve_bodies(vec![
            r#"<html><body><div class="content">正文一。</div></body></html>"#.to_string(),
            r#"<html><body><div class="content">正文二。</div></body></html>"#.to_string(),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "缓存测试源".into(),
                    rule_content: Some(serde_json::json!({ "content": "div.content@text" })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let book_url = "https://book.com/a";
        let ch1 = format!("{base}/ch1.html");
        let ch2 = format!("{base}/ch2.html");
        let idx1 = crate::util::md5::chapter_url_hash(&ch1);
        let params = |chapter_url: &str| -> HashMap<String, String> {
            [
                ("chapterUrl".into(), chapter_url.to_string()),
                ("bookUrl".into(), book_url.to_string()),
                ("bookSource".into(), base.clone()),
            ]
            .into_iter()
            .collect()
        };

        // 首次：抓取成功 → 返回 + 写回缓存
        let ret = get_book_content(
            AxumState(state.clone()),
            Query(params(&ch1)),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["content"], "正文一。");
        assert_eq!(
            state
                .storage
                .get_chapter_content("default", book_url, idx1)
                .await
                .unwrap()
                .as_deref(),
            Some("正文一。"),
            "抓取成功应写回 book_chapters"
        );

        // 二次同 chapterUrl：命中缓存直读（若再抓取会拿到正文二。→ 断言失败即回归）
        let ret = get_book_content(
            AxumState(state.clone()),
            Query(params(&ch1)),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["content"], "正文一。", "缓存命中应直读不回源");

        // 不同 chapterUrl → 未命中，抓取正文二。并写回
        let ret = get_book_content(
            AxumState(state.clone()),
            Query(params(&ch2)),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["content"], "正文二。");
        let idx2 = crate::util::md5::chapter_url_hash(&ch2);
        assert_eq!(
            state
                .storage
                .get_chapter_content("default", book_url, idx2)
                .await
                .unwrap()
                .as_deref(),
            Some("正文二。")
        );

        // 缺 bookUrl：照常抓取返回，但不落缓存
        let p: HashMap<String, String> = [
            ("chapterUrl".into(), format!("{base}/ch3.html")),
            ("bookSource".into(), base.clone()),
        ]
        .into_iter()
        .collect();
        let ret =
            get_book_content(AxumState(state.clone()), Query(p), HeaderMap::new(), None).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(state
            .storage
            .get_chapter_content(
                "default",
                "",
                crate::util::md5::chapter_url_hash(&format!("{base}/ch3.html"))
            )
            .await
            .unwrap()
            .is_none());

        // local:// 章节不走缓存（不命中、不写回）
        let p: HashMap<String, String> = [
            ("chapterUrl".into(), "local://book1/0".into()),
            ("bookUrl".into(), "local://book1".into()),
        ]
        .into_iter()
        .collect();
        let ret =
            get_book_content(AxumState(state.clone()), Query(p), HeaderMap::new(), None).await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "本地书章节不存在");
        assert_eq!(
            state
                .storage
                .count_chapters("default", "local://book1")
                .await
                .unwrap(),
            0,
            "local:// 不落正文缓存"
        );

        cleanup(state, dir).await;
    }

    /// 命名兼容批：6 条 legacy 别名路由端到端（真实 router + HTTP 请求）
    #[tokio::test]
    async fn test_alias_routes_end_to_end() {
        let (state, dir) = test_state("alias").await;
        let app = router(state.storage.config.clone(), state.storage.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let base = format!("http://{addr}");
        let client = reqwest::Client::new();

        // getChapterList（= getBookToc）：缺参 → 业务错误（路由可达，非 404）
        let resp = client
            .get(format!("{base}/reader3/getChapterList"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let json: Value = resp.json().await.unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());
        assert_eq!(json["errorMsg"], "请输入书籍链接");

        // getRssContent（= getRssArticle）：缺 url → 业务错误
        let resp = client
            .get(format!("{base}/reader3/getRssContent"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let json: Value = resp.json().await.unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());

        // getUserList（= getUsers）：非 secure 模式管理接口拒绝（而非 404）
        let resp = client
            .get(format!("{base}/reader3/getUserList"))
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());

        // getBookGroupList（= getBookGroups）：空列表
        let resp = client
            .get(format!("{base}/reader3/getBookGroupList"))
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        assert!(json["isSuccess"].as_bool().unwrap());
        // F7：空库播种内置 5 组（全部/本地/音频/未分组/更新错误）
        let arr = json["data"].as_array().unwrap();
        assert_eq!(arr.len(), 5, "空库应播种 5 个默认分组");
        assert_eq!(arr[0]["name"], "全部");
        assert_eq!(arr[0]["groupId"], -1);
        assert_eq!(arr[0]["order"], -10);

        // saveBookGroupName（= saveBookGroup）：新建
        let resp = client
            .post(format!("{base}/reader3/saveBookGroupName"))
            .json(&serde_json::json!({ "name": "分组A" }))
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        assert!(json["isSuccess"].as_bool().unwrap(), "{}", json);
        let gid = json["data"]["id"].as_i64().expect("新建应返回 id");

        // updateBookGroup（= saveBookGroup）：重命名 {id,name}
        let resp = client
            .post(format!("{base}/reader3/updateBookGroup"))
            .json(&serde_json::json!({ "id": gid, "name": "分组A2" }))
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        assert!(json["isSuccess"].as_bool().unwrap(), "{}", json);

        // getBookGroupList 复核：改名生效 + 双字段 + 书数
        let resp = client
            .get(format!("{base}/reader3/getBookGroupList"))
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        let arr = json["data"].as_array().unwrap();
        // F7 播种的 5 个默认组 + 新建的分组A2
        assert_eq!(arr.len(), 6);
        let ga2 = arr
            .iter()
            .find(|g| g["name"] == "分组A2")
            .expect("分组A2 应存在");
        assert_eq!(ga2["orderNum"], ga2["order"], "order/orderNum 同值");
        assert_eq!(ga2["bookCount"], 0);
        let _ = ga2;

        // deleteBookGroup：删除成功
        let resp = client
            .post(format!("{base}/reader3/deleteBookGroup"))
            .json(&serde_json::json!({ "id": gid }))
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        assert!(json["isSuccess"].as_bool().unwrap(), "{}", json);

        cleanup(state, dir).await;
    }

    /// 换源：searchBookSource——书架书取书名、排除当前源、其余源搜索失败优雅降级为空数组
    #[tokio::test]
    async fn test_search_book_source_api() {
        let (state, dir) = test_state("srcswitch").await;
        // 两个书源：s1=当前源（有搜索规则），s2=其他源（search_url 指向不可达域名，爬取失败→空）
        for (url, name) in [("https://s1.com", "源1"), ("https://s2.com", "源2")] {
            state
                .storage
                .save_book_source(
                    "default",
                    &crate::model::BookSource {
                        book_source_url: url.into(),
                        book_source_name: name.into(),
                        enabled: true,
                        search_url: Some(format!("{url}/search?q={{key}}")),
                        rule_search: Some(serde_json::json!({
                            "bookList": "@js:JSON.parse(result).data",
                            "name": "$.name",
                            "author": "$.author",
                            "bookUrl": "$.url",
                            "tocUrl": "$.toc"
                        })),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
        }
        // 书架书（当前源 s1）
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://s1.com/book/1".into(),
                    name: "测试书".into(),
                    origin: "https://s1.com".into(),
                    origin_name: "源1".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // 缺参 → 业务错误
        let ret = search_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "请输入书籍链接");

        // 正常调用：排除当前源 s1，仅 s2 搜索（不可达 → 空数组，不报错）
        let params: HashMap<String, String> = [
            ("url".into(), "https://s1.com/book/1".into()),
            ("bookSource".into(), "https://s1.com".into()),
        ]
        .into_iter()
        .collect();
        let ret = search_book_source(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "应成功: {}", ret.0.error_msg);
        let data = ret
            .0
            .data
            .as_array()
            .expect("应返回数组（源搜索失败降级为空）");
        assert!(data.is_empty());

        // 仅当前源 → 无其他源 → data null
        state
            .storage
            .delete_book_source("default", "https://s2.com")
            .await
            .unwrap();
        let params: HashMap<String, String> = [
            ("url".into(), "https://s1.com/book/1".into()),
            ("bookSource".into(), "https://s1.com".into()),
        ]
        .into_iter()
        .collect();
        let ret = search_book_source(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert!(ret.0.data.is_null(), "无其他源应返回 null");

        cleanup(state, dir).await;
    }

    /// OPDS：本地书 acquire 正文 / download 下载（存库章节重建），目录条目含两个 acquisition 链接
    #[tokio::test]
    async fn test_opds_local_book_acquire_and_download() {
        let (state, dir) = test_state("opdslocal").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "local://abc".into(),
                    name: "本地书".into(),
                    author: "作者".into(),
                    origin: "loc_book".into(),
                    origin_name: "本地".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "local://abc",
                &[
                    ("第一章".to_string(), "第一段内容".to_string()),
                    ("第二章".to_string(), "第二段内容".to_string()),
                ],
            )
            .await
            .unwrap();
        let id = crate::api::opds::encode_id("local://abc");

        // acquire：正文（新 API 返回首章正文，不含标题）
        let (name, bytes) = crate::api::opds::acquire(&state.storage, "default", &id)
            .await
            .expect("acquire 应成功");
        assert_eq!(name, "本地书.txt");
        let text = String::from_utf8(bytes).unwrap();
        assert!(
            text.contains("第一段内容"),
            "acquire 应返回首章正文: {text}"
        );

        // download：章节重建（带文件名）
        let (fname, bytes, _ct) =
            crate::api::opds::download(&state.storage, "default", &id, "", None)
                .await
                .unwrap();
        assert_eq!(fname, "本地书.txt");
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("第二段内容"));

        // 书架目录（OPDS 2.0 JSON）：含两个 acquisition 链接（download + acquire，绝对 URL）
        let json = crate::api::opds::shelf_json(
            &state.storage,
            "default",
            0,
            50,
            "http://reader.example.com",
        )
        .await
        .unwrap();
        assert!(
            json.contains(&format!("http://reader.example.com/opds/download/{id}")),
            "目录应含下载链接"
        );
        assert!(
            json.contains(&format!("http://reader.example.com/opds/acquire/{id}")),
            "目录应含正文链接"
        );
        assert!(json.contains("本地书"));

        cleanup(state, dir).await;
    }

    /// P1-2：客户端 IP 解析——默认只用直连 IP（XFF 可伪造，不信任）；
    /// 仅当直连 IP 命中 READER_TRUSTED_PROXIES 白名单（IP/CIDR）时才取 XFF 最左项
    #[test]
    fn test_client_ip_defaults_to_direct() {
        let peer: std::net::SocketAddr = "203.0.113.9:40000".parse().unwrap();
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "9.9.9.9".parse().unwrap());
        h.insert("x-real-ip", "9.9.9.9".parse().unwrap());
        // 无白名单：XFF 完全忽略
        assert_eq!(client_ip_with(&peer, &h, &[]), "203.0.113.9");
        // 白名单不含直连 IP：仍忽略 XFF
        let proxies = parse_trusted_proxies("198.51.100.1, 10.0.0.0/8");
        assert_eq!(client_ip_with(&peer, &h, &proxies), "203.0.113.9");
    }

    /// P1-2：直连 IP 命中白名单 → 信任 XFF（取最左项）；XFF 缺失/非法回退直连 IP
    #[test]
    fn test_client_ip_trusted_proxy_xff() {
        let proxies = parse_trusted_proxies("10.0.0.0/8, 192.168.1.1");
        let peer: std::net::SocketAddr = "10.0.0.5:40000".parse().unwrap();
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "203.0.113.7, 10.0.0.5".parse().unwrap());
        assert_eq!(client_ip_with(&peer, &h, &proxies), "203.0.113.7");
        // XFF 缺失 → 回退直连 IP
        assert_eq!(
            client_ip_with(&peer, &HeaderMap::new(), &proxies),
            "10.0.0.5"
        );
        // XFF 非法 → 回退直连 IP
        let mut h2 = HeaderMap::new();
        h2.insert("x-forwarded-for", "not-an-ip".parse().unwrap());
        assert_eq!(client_ip_with(&peer, &h2, &proxies), "10.0.0.5");
        // 精确 IP 白名单
        let proxies2 = parse_trusted_proxies("192.168.1.1");
        let peer2: std::net::SocketAddr = "192.168.1.1:9".parse().unwrap();
        let mut h3 = HeaderMap::new();
        h3.insert("x-forwarded-for", "198.51.100.2".parse().unwrap());
        assert_eq!(client_ip_with(&peer2, &h3, &proxies2), "198.51.100.2");
        // IPv6 CIDR 白名单
        let proxies3 = parse_trusted_proxies("fd00::/8");
        let peer3: std::net::SocketAddr = "[fd00::1]:4000".parse().unwrap();
        let mut h4 = HeaderMap::new();
        h4.insert("x-forwarded-for", "2001:db8::1".parse().unwrap());
        assert_eq!(client_ip_with(&peer3, &h4, &proxies3), "2001:db8::1");
    }

    /// P1-2：READER_TRUSTED_PROXIES 解析——逗号分隔 IP/CIDR，非法项忽略，v4/v6 均支持
    #[test]
    fn test_parse_trusted_proxies() {
        assert!(parse_trusted_proxies("").is_empty());
        assert!(parse_trusted_proxies("  , , ").is_empty());
        let p = parse_trusted_proxies("10.0.0.1, 192.168.1.0/24, garbage, 2001:db8::/32");
        assert_eq!(p.len(), 3, "非法项忽略");
        assert!(p[0].matches("10.0.0.1".parse().unwrap()));
        assert!(!p[0].matches("10.0.0.2".parse().unwrap()));
        assert!(p[1].matches("192.168.1.55".parse().unwrap()));
        assert!(!p[1].matches("192.168.2.55".parse().unwrap()));
        assert!(p[2].matches("2001:db8::42".parse().unwrap()));
        assert!(!p[2].matches("2001:db9::42".parse().unwrap()));
        // /0 匹配全部
        let any = parse_trusted_proxies("0.0.0.0/0");
        assert!(any[0].matches("8.8.8.8".parse().unwrap()));
    }

    /// OPDS：accessToken 查询参数认证（secure 模式），与 /reader3 一致
    #[tokio::test]
    async fn test_opds_access_token_query_auth() {
        let (state, dir) = test_state("opdsauth").await;
        let mut state = state;
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "tok123".into(),
                last_login_at: now_millis(),
                ..Default::default()
            })
            .await
            .unwrap();

        // 正确 accessToken → alice
        let params: HashMap<String, String> = [("accessToken".into(), "alice:tok123".into())]
            .into_iter()
            .collect();
        let ns = opds_ns(&state, &HeaderMap::new(), &params, "198.51.100.60")
            .await
            .expect("应认证通过");
        assert_eq!(ns, "alice");

        // 错误 token → 401
        let params: HashMap<String, String> = [("accessToken".into(), "alice:bad".into())]
            .into_iter()
            .collect();
        let ret = opds_ns(&state, &HeaderMap::new(), &params, "198.51.100.60").await;
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().status(), StatusCode::UNAUTHORIZED);

        // 缺 accessToken → 401（secure）
        let ret = opds_ns(&state, &HeaderMap::new(), &HashMap::new(), "198.51.100.60").await;
        assert!(ret.is_err());

        // 非 secure → 恒 default（accessToken 不参与）
        let mut state2 = state.clone();
        state2.storage.config.secure = false;
        let ret = opds_ns(&state2, &HeaderMap::new(), &HashMap::new(), "198.51.100.60").await;
        assert_eq!(ret.unwrap(), "default");

        cleanup(state, dir).await;
    }

    /// OPDS：Basic 认证——独立 OPDS 账号优先（system_settings），回退系统用户（users 表）；
    /// 密码存储：独立账号 sha256(salt||pwd)（salt$hash）；系统用户 argon2id 或 legacy 双 md5（MD5 通过自动升级）
    #[tokio::test]
    async fn test_opds_basic_auth_accounts() {
        let (state, dir) = test_state("opdsbasic").await;
        let mut state = state;
        state.storage.config.secure = true;
        // 系统用户（users 表，legacy 双 md5 哈希存储——兼容路径，校验通过后自动升级）
        let salt = "s1".to_string();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                password: crate::util::md5::gen_encrypted_password("pw123456", &salt),
                salt: salt.clone(),
                token: "tok123".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        // 独立 OPDS 账号（sha256+salt 存储，不回传明文）
        let stored = crate::util::sha256::store_password("opds-pass");
        state
            .storage
            .set_opds_account("reader", &stored)
            .await
            .unwrap();

        let basic = |u: &str, p: &str| {
            use base64::Engine;
            let mut h = HeaderMap::new();
            let cred = base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}"));
            h.insert(
                axum::http::header::AUTHORIZATION,
                format!("Basic {cred}").parse().unwrap(),
            );
            h
        };

        // 独立 OPDS 账号通过
        let ns = opds_ns(
            &state,
            &basic("reader", "opds-pass"),
            &HashMap::new(),
            "198.51.100.61",
        )
        .await
        .expect("独立 OPDS 账号应通过");
        assert_eq!(ns, "reader");

        // 独立账号密码错误 → 401（不回退系统账号的同名用户）
        let ret = opds_ns(
            &state,
            &basic("reader", "wrong"),
            &HashMap::new(),
            "198.51.100.61",
        )
        .await;
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().status(), StatusCode::UNAUTHORIZED);

        // 系统用户 Basic 通过（密码为哈希存储，统一校验：MD5 通过自动升级 argon2id）
        let ns = opds_ns(
            &state,
            &basic("alice", "pw123456"),
            &HashMap::new(),
            "198.51.100.61",
        )
        .await
        .expect("系统用户 Basic 应通过");
        assert_eq!(ns, "alice");
        // 升级钩子：legacy MD5 校验通过后 users.password 应变为 argon2id PHC
        let alice = state.storage.find_user("alice").await.unwrap().unwrap();
        assert!(
            alice.password.starts_with("$argon2id$"),
            "OPDS Basic 校验后旧 MD5 密码应自动升级: {}",
            alice.password
        );
        assert!(crate::util::password::verify_argon2id(
            "pw123456",
            &alice.password
        ));

        // 系统用户密码错误 → 401
        let ret = opds_ns(
            &state,
            &basic("alice", "bad"),
            &HashMap::new(),
            "198.51.100.61",
        )
        .await;
        assert!(ret.is_err());

        // 禁用独立账号后：系统账号仍可用
        state.storage.clear_opds_account().await.unwrap();
        let ns = opds_ns(
            &state,
            &basic("alice", "pw123456"),
            &HashMap::new(),
            "198.51.100.61",
        )
        .await
        .expect("禁用独立账号后系统用户应通过");
        assert_eq!(ns, "alice");

        cleanup(state, dir).await;
    }

    /// OPDS 分发路由：根导航 / shelf / opensearch / 404 / acquire / save
    #[tokio::test]
    async fn test_opds_dispatch_routes() {
        let (state, dir) = test_state("opdsdispatch").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://a.com/1".into(),
                    name: "测试书".into(),
                    author: "作者".into(),
                    origin: "https://s.com".into(),
                    origin_name: "源".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let dispatch = |rest: &str, params: HashMap<String, String>| {
            opds_dispatch(
                ConnectInfo("198.51.100.62:8080".parse().unwrap()),
                AxumState(state.clone()),
                Some(axum::extract::Path(rest.to_string())),
                Query(params),
                HeaderMap::new(),
            )
        };

        // 根：导航 feed
        let resp = dispatch("", HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("书架") && body.contains("/opds/shelf"));

        // shelf：acquisition feed 含条目
        let resp = dispatch("shelf", HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("测试书"));
        assert!(body.contains("opds:totalResults"));

        // opensearch.xml
        let resp = dispatch("opensearch.xml", HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("opensearchdescription"));

        // OPDS 2.0 根
        let resp = dispatch("catalog", HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("application/opds+json"));

        // 未知路径 → 404
        let resp = dispatch("nonexistent", HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // 不存在书籍：acquire/save → 404
        let id = crate::api::opds::encode_id("https://nope.com");
        let resp = dispatch(&format!("acquire/{id}"), HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let resp = dispatch(&format!("save/{id}"), HashMap::new()).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // 搜索
        let resp = dispatch(
            "search",
            [("q".to_string(), "测试".to_string())]
                .into_iter()
                .collect(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        cleanup(state, dir).await;
    }

    /// OPDS 独立账号设置端点：saveOpdsSettings / getOpdsSettings（密码不回传）
    #[tokio::test]
    async fn test_opds_settings_endpoints() {
        let (state, dir) = test_state("opdsset").await;
        // 默认关闭
        let ret = get_opds_settings(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["enabled"], false);
        assert_eq!(ret.0.data["passwordSet"], false);

        // 配置账号
        let body = Bytes::from(json!({"username": "reader", "password": "secret123"}).to_string());
        let ret = save_opds_settings(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "保存失败: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["enabled"], true);

        let ret = get_opds_settings(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["username"], "reader");
        assert_eq!(ret.0.data["passwordSet"], true);
        assert!(ret.0.data.get("password").is_none(), "密码不得回传");
        // 落库为 salt$hash，非明文
        let (_, stored) = state.storage.get_opds_account().await.unwrap().unwrap();
        assert!(stored.contains('$'));
        assert_ne!(stored, "secret123");

        // 短密码拒绝
        let body = Bytes::from(json!({"username": "reader", "password": "123"}).to_string());
        let ret = save_opds_settings(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);

        // 空用户名 → 禁用
        let body = Bytes::from(json!({"username": "", "password": ""}).to_string());
        let ret = save_opds_settings(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let ret = get_opds_settings(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(ret.0.data["enabled"], false);

        cleanup(state, dir).await;
    }

    /// 路由构建冒烟：全部 OPDS 路由注册无冲突（matchit 冲突会在构建时 panic）
    #[tokio::test]
    async fn test_router_constructs_with_opds_routes() {
        let (state, dir) = test_state("opdsrouter").await;
        let app = router(state.storage.config.clone(), state.storage.clone());
        // 不 panic 即通过（axum 0.7 Router 无 routes() 自省——构建冲突会在 router() 时 panic）
        let _ = app;
        cleanup(state, dir).await;
    }

    // ---------------- 书源登录态（loginBookSource 参数解析） ----------------

    /// 登录参数合并：query 兜底 + body JSON 优先 / form-urlencoded 兑底
    #[test]
    fn test_merge_login_params_query_only() {
        let mut q = HashMap::new();
        q.insert("bookSource".to_string(), "https://a.com".to_string());
        q.insert("username".to_string(), "u1".to_string());
        let m = merge_login_params(&q, None);
        assert_eq!(
            m.get("bookSource").map(String::as_str),
            Some("https://a.com")
        );
        assert_eq!(m.get("username").map(String::as_str), Some("u1"));
        assert_eq!(m.get("password"), None);
    }

    #[test]
    fn test_merge_login_params_json_body() {
        let mut q = HashMap::new();
        q.insert("bookSource".to_string(), "https://query.com".to_string());
        let body = br#"{"bookSource":"https://body.com","username":"u1","password":"p1","captcha":"c1","mode":"browser"}"#;
        let m = merge_login_params(&q, Some(body));
        // body JSON 优先于 query
        assert_eq!(
            m.get("bookSource").map(String::as_str),
            Some("https://body.com")
        );
        assert_eq!(m.get("username").map(String::as_str), Some("u1"));
        assert_eq!(m.get("password").map(String::as_str), Some("p1"));
        assert_eq!(m.get("captcha").map(String::as_str), Some("c1"));
        assert_eq!(m.get("mode").map(String::as_str), Some("browser"));
    }

    #[test]
    fn test_merge_login_params_form_body() {
        let mut q = HashMap::new();
        q.insert("bookSource".to_string(), "https://a.com".to_string());
        let body = b"username=u1&password=p1&captcha=c1";
        let m = merge_login_params(&q, Some(body));
        assert_eq!(
            m.get("bookSource").map(String::as_str),
            Some("https://a.com")
        );
        assert_eq!(m.get("username").map(String::as_str), Some("u1"));
        assert_eq!(m.get("password").map(String::as_str), Some("p1"));
        assert_eq!(m.get("captcha").map(String::as_str), Some("c1"));
    }

    #[test]
    fn test_merge_login_params_invalid_body_falls_back_to_query() {
        let mut q = HashMap::new();
        q.insert("bookSource".to_string(), "https://a.com".to_string());
        // 非 JSON 非表单（二进制）→ 保留 query
        let m = merge_login_params(&q, Some(b"\x00\x01\x02"));
        assert_eq!(
            m.get("bookSource").map(String::as_str),
            Some("https://a.com")
        );
    }
    // ==================== 差距补全批：导出 / 调试 / 缓存 / 配置 / 刷新 / 批量 / 健康 / 统计 ====================

    /// 微型 HTTP 服务器：支持 HEAD 与 GET（健康检测用）
    async fn serve_head_get() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            for _ in 0..10 {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 2048];
                let _ = sock.read(&mut buf).await;
                let req = String::from_utf8_lossy(&buf);
                let body = if req.starts_with("HEAD ") {
                    ""
                } else {
                    "<html>ok</html>"
                };
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}")
    }

    /// exportBook：本地书 txt/epub/html 三格式导出 + 参数校验
    #[tokio::test]
    async fn test_export_book_api() {
        let (state, dir) = test_state("exportbook").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "local://exp1".into(),
                    name: "导出测试书".into(),
                    author: "作者甲".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "local://exp1",
                &[
                    ("第一章".to_string(), "正文一 <甲> & 乙。".to_string()),
                    ("第二章".to_string(), "正文二。".to_string()),
                ],
            )
            .await
            .unwrap();
        let params = |format: &str| -> HashMap<String, String> {
            [
                ("url".into(), "local://exp1".into()),
                ("format".into(), format.into()),
            ]
            .into_iter()
            .collect()
        };

        // txt
        let resp = export_book(
            AxumState(state.clone()),
            Query(params("txt")),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("text/plain; charset=utf-8")
        );
        let cd = resp
            .headers()
            .get("Content-Disposition")
            .and_then(|v| v.to_str().ok())
            .expect("应含 Content-Disposition");
        assert!(cd.starts_with("attachment; filename="), "{cd}");
        assert!(cd.ends_with(".txt\""), "{cd}");
        assert!(cd.contains("%E5%AF%BC"), "非 ASCII 应百分号编码: {cd}");
        let txt = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(txt.contains("导出测试书"));
        assert!(txt.contains("第一章"));
        assert!(txt.contains("正文一 <甲> & 乙。"));
        assert!(txt.contains("正文二。"));

        // epub（zip 构造验证：mimetype/container/OPF/spine 章节）
        let resp = export_book(
            AxumState(state.clone()),
            Query(params("epub")),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("application/epub+zip")
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("EPUB 应为合法 zip");
        let mut mime = String::new();
        std::io::Read::read_to_string(&mut zip.by_name("mimetype").unwrap(), &mut mime).unwrap();
        assert_eq!(mime, "application/epub+zip");
        let mut container = String::new();
        std::io::Read::read_to_string(
            &mut zip.by_name("META-INF/container.xml").unwrap(),
            &mut container,
        )
        .unwrap();
        assert!(container.contains("OEBPS/content.opf"));
        let mut opf = String::new();
        std::io::Read::read_to_string(&mut zip.by_name("OEBPS/content.opf").unwrap(), &mut opf)
            .unwrap();
        assert!(opf.contains("<dc:title>导出测试书</dc:title>"));
        assert!(opf.contains("<dc:creator>作者甲</dc:creator>"));
        assert_eq!(opf.matches("<itemref").count(), 2, "spine 两章");
        let mut ch0 = String::new();
        std::io::Read::read_to_string(&mut zip.by_name("OEBPS/chap_0000.xhtml").unwrap(), &mut ch0)
            .unwrap();
        assert!(
            ch0.contains("正文一 &lt;甲&gt; &amp; 乙。"),
            "XML 转义: {ch0}"
        );

        // html（单页：标题 + 章节）
        let resp = export_book(
            AxumState(state.clone()),
            Query(params("html")),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("text/html; charset=utf-8")
        );
        let html = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("<h1>导出测试书</h1>"));
        assert!(html.contains("<h2>第一章</h2>"));
        assert!(html.contains("<p>正文二。</p>"));

        // 非法格式 / 缺 url
        let resp = export_book(
            AxumState(state.clone()),
            Query(params("pdf")),
            HeaderMap::new(),
            None,
        )
        .await;
        let json: Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());
        assert_eq!(json["errorMsg"], "不支持的导出格式（txt|epub|html）");
        let resp = export_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        let json: Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());

        cleanup(state, dir).await;
    }

    /// GAP 176：exportBook font 参数——epub 内嵌中文字体（zip 含字体文件 + OPF manifest
    /// 字体条目 + style.css @font-face）；非法 font 参数明确报错；txt 格式忽略 font
    #[tokio::test]
    async fn test_export_book_epub_font_api() {
        use std::io::Read as _;
        let (state, dir) = test_state("exportfont").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "local://expfont".into(),
                    name: "字体书".into(),
                    author: "作者甲".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "local://expfont",
                &[("第一章".to_string(), "正文一。".to_string())],
            )
            .await
            .unwrap();
        let url = "local://expfont".to_string();

        // font=lxk-wenkai → epub 内嵌霞鹜文楷
        let params: HashMap<String, String> = [
            ("url".into(), url.clone()),
            ("format".into(), "epub".into()),
            ("font".into(), "lxk-wenkai".into()),
        ]
        .into_iter()
        .collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("epub 应合法");
        // zip 含字体文件（字节非空且为 woff2 魔数 wOF2）
        let mut font_file = zip
            .by_name("OEBPS/fonts/lxgw-wenkai-regular.woff2")
            .expect("zip 应含字体文件");
        let mut font_bytes = Vec::new();
        font_file.read_to_end(&mut font_bytes).unwrap();
        drop(font_file); // 释放 zip 可变借用（ZipFile 带 Drop——借用延续到作用域尾）
        assert!(font_bytes.starts_with(b"wOF2"), "woff2 魔数");
        assert!(
            font_bytes.len() > 100_000,
            "完整字体子集: {}B",
            font_bytes.len()
        );
        // OPF manifest 字体条目 + style 条目
        let mut opf = String::new();
        zip.by_name("OEBPS/content.opf")
            .unwrap()
            .read_to_string(&mut opf)
            .unwrap();
        assert!(opf.contains("font-embedded"), "OPF 字体条目: {opf}");
        assert!(opf.contains("fonts/lxgw-wenkai-regular.woff2"));
        assert!(opf.contains("<item id=\"style\" href=\"style.css\" media-type=\"text/css\"/>"));
        // CSS @font-face
        let mut css = String::new();
        zip.by_name("OEBPS/style.css")
            .unwrap()
            .read_to_string(&mut css)
            .unwrap();
        assert!(css.contains("@font-face"), "CSS: {css}");
        assert!(css.contains("font-family: 'LXGW WenKai';"));
        assert!(css.contains("url('fonts/lxgw-wenkai-regular.woff2') format('woff2')"));
        assert!(
            css.contains("font-family: 'LXGW WenKai', 'Kaiti SC', '楷体', serif;"),
            "正文应用: {css}"
        );
        // 章节链接样式表
        let mut ch0 = String::new();
        zip.by_name("OEBPS/chap_0000.xhtml")
            .unwrap()
            .read_to_string(&mut ch0)
            .unwrap();
        assert!(ch0.contains("href=\"style.css\""));

        // font=source-han-serif → 思源宋体
        let params: HashMap<String, String> = [
            ("url".into(), url.clone()),
            ("format".into(), "epub".into()),
            ("font".into(), "source-han-serif".into()),
        ]
        .into_iter()
        .collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("epub 应合法");
        assert!(zip
            .by_name("OEBPS/fonts/source-han-serif-cn-regular.woff2")
            .is_ok());

        // 缺省 font → 无字体条目（既有行为）
        let params: HashMap<String, String> = [
            ("url".into(), url.clone()),
            ("format".into(), "epub".into()),
        ]
        .into_iter()
        .collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("epub 应合法");
        assert!(
            zip.by_name("OEBPS/style.css").is_err(),
            "无 font 不应有 style.css"
        );

        // 非法 font 参数 → 明确错误
        let params: HashMap<String, String> = [
            ("url".into(), url.clone()),
            ("format".into(), "epub".into()),
            ("font".into(), "comic-sans".into()),
        ]
        .into_iter()
        .collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let json: Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());
        assert!(
            json["errorMsg"].as_str().unwrap().contains("不支持的字体"),
            "{json}"
        );

        // txt 格式忽略 font（不报错）
        let params: HashMap<String, String> = [
            ("url".into(), url.clone()),
            ("format".into(), "txt".into()),
            ("font".into(), "lxk-wenkai".into()),
        ]
        .into_iter()
        .collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        cleanup(state, dir).await;
    }

    /// exportBook：书源书（目录 → 逐章正文，复用规则引擎）
    #[tokio::test]
    async fn test_export_book_web_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("exportweb").await;
        let base_url = serve_bodies(vec![
            r#"<ul class="chapters"><li><a href="/ch1.html">第一章</a></li><li><a href="/ch2.html">第二章</a></li></ul>"#.to_string(),
            r#"<html><body><div class="content">正文一。</div></body></html>"#.to_string(),
            r#"<html><body><div class="content">正文二。</div></body></html>"#.to_string(),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "导出源".into(),
                    rule_toc: Some(serde_json::json!({
                        "chapterList": "ul.chapters@li", "chapterName": "a@text", "chapterUrl": "a@href"
                    })),
                    rule_content: Some(serde_json::json!({ "content": "div.content@text" })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let book_url = format!("{base}/book/1");
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.clone(),
                    name: "网文书".into(),
                    origin: base.clone(),
                    toc_url: format!("{base}/toc"),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let params: HashMap<String, String> =
            [("url".into(), book_url.clone())].into_iter().collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK, "书源书应可导出");
        let txt = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(txt.contains("网文书"));
        assert!(txt.contains("第一章"));
        assert!(txt.contains("正文一。"));
        assert!(txt.contains("正文二。"));

        cleanup(state, dir).await;
    }

    /// P2：书源书导出——并发抓章失败章节不再静默丢弃：X-Export-Warning 头携带
    /// failedChapters 列表（percent 编码 JSON），成功章节照常导出
    #[tokio::test]
    async fn test_export_book_web_failed_chapters_warning() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1
        let (state, dir) = test_state("exportwarn").await;
        // 第二章指向死端口（无监听）→ 抓章失败；其余正常（路径寻址保证并发序确定性）
        let toc_html = r#"<ul class="chapters"><li><a href="/ch1.html">第一章</a></li><li><a href="http://127.0.0.1:1/ch2.html">第二章（死端口）</a></li><li><a href="/ch3.html">第三章</a></li></ul>"#;
        let base_url = serve_bodies_by_path(vec![
            // book_url 路径也供目录（upsert_book 不持久化 toc_url——collect 回退抓 book_url 当目录页）
            ("/book/warn".to_string(), toc_html.to_string()),
            ("/toc".to_string(), toc_html.to_string()),
            (
                "/ch1.html".to_string(),
                r#"<html><body><div class="content">正文一。</div></body></html>"#.to_string(),
            ),
            (
                "/ch3.html".to_string(),
                r#"<html><body><div class="content">正文三。</div></body></html>"#.to_string(),
            ),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "导出源2".into(),
                    rule_toc: Some(serde_json::json!({
                        "chapterList": "ul.chapters@li", "chapterName": "a@text", "chapterUrl": "a@href"
                    })),
                    rule_content: Some(serde_json::json!({ "content": "div.content@text" })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let book_url = format!("{base}/book/warn");
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.clone(),
                    name: "网文书2".into(),
                    origin: base.clone(),
                    toc_url: format!("{base}/toc"),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let params: HashMap<String, String> =
            [("url".into(), book_url.clone())].into_iter().collect();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK, "失败章不应阻塞导出");
        // 警告头：failedChapters 含死端口章（percent 编码 JSON，解码后可解析）
        let warn = resp
            .headers()
            .get("X-Export-Warning")
            .and_then(|v| v.to_str().ok())
            .expect("应返回 X-Export-Warning 警告头");
        assert!(
            warn.is_ascii(),
            "警告头必须纯 ASCII（HeaderValue 约束）: {warn}"
        );
        let decoded = percent_decode(&warn);
        let json: serde_json::Value =
            serde_json::from_str(&decoded).expect("警告头应为合法 JSON: {decoded}");
        let failed = json["failedChapters"]
            .as_array()
            .expect("failedChapters 数组");
        assert_eq!(failed.len(), 1, "仅失败章入列表: {json}");
        assert_eq!(failed[0]["title"], "第二章（死端口）");
        assert_eq!(failed[0]["index"], 1);
        assert!(
            failed[0]["error"]
                .as_str()
                .unwrap_or("")
                .contains("获取正文失败"),
            "失败原因: {json}"
        );
        // 成功章节照常导出，失败章不出现
        let txt = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(txt.contains("第一章") && txt.contains("正文一。"));
        assert!(txt.contains("第三章") && txt.contains("正文三。"));
        assert!(!txt.contains("死端口"), "失败章不应出现在导出正文");

        cleanup(state, dir).await;
    }

    /// P2：GBK 导出不可映射字符——不再静默替换为 ?：正文转义 NCR（&#x…;），
    /// X-Export-Warning 携带 unmappableChars 计数；utf-8 导出无警告头
    #[tokio::test]
    async fn test_export_book_gbk_unmappable_warning() {
        let (state, dir) = test_state("exportgbk").await;
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "local://gbk1".into(),
                    name: "GBK书".into(),
                    author: "作者".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "local://gbk1",
                &[("第一章".to_string(), "正文😀结束。".to_string())],
            )
            .await
            .unwrap();
        let params = |extra: &[(&str, &str)]| -> HashMap<String, String> {
            let mut m: HashMap<String, String> = [("url".into(), "local://gbk1".into())]
                .into_iter()
                .collect();
            for (k, v) in extra {
                m.insert(k.to_string(), v.to_string());
            }
            m
        };
        // 第二本书用独立闭包（url 指向 gbk2——修复：原闭包硬编码 gbk1 导致导出错误书）
        let params_gbk2 = |extra: &[(&str, &str)]| -> HashMap<String, String> {
            let mut m: HashMap<String, String> = [("url".into(), "local://gbk2".into())]
                .into_iter()
                .collect();
            for (k, v) in extra {
                m.insert(k.to_string(), v.to_string());
            }
            m
        };
        // gbk：😀 不可映射 → 转义 + 计数警告
        let resp = export_book(
            AxumState(state.clone()),
            Query(params(&[("format", "txt"), ("encoding", "gbk")])),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let warn = resp
            .headers()
            .get("X-Export-Warning")
            .and_then(|v| v.to_str().ok())
            .expect("GBK 不可映射应返回警告头");
        assert!(warn.is_ascii());
        let json: serde_json::Value =
            serde_json::from_str(&percent_decode(warn)).expect("合法 JSON");
        assert_eq!(json["unmappableChars"], 1, "一个不可映射字符: {json}");
        let raw = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        let txt = encoding_rs::GBK.decode(&raw).0.into_owned();
        assert!(txt.contains("&#x1F600;"), "😀 应转义为 NCR: {txt}");
        assert!(!txt.contains('?'), "不应替换为 ?: {txt}");
        assert!(txt.contains("正文") && txt.contains("结束。"));

        // 全部可映射 → 无警告头
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "local://gbk2".into(),
                    name: "GBK书2".into(),
                    author: "作者".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                "local://gbk2",
                &[("第一章".to_string(), "纯中文正文。".to_string())],
            )
            .await
            .unwrap();
        let resp = export_book(
            AxumState(state.clone()),
            Query(params_gbk2(&[("format", "txt"), ("encoding", "gbk")])),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let warn = resp
            .headers()
            .get("X-Export-Warning")
            .map(|v| v.to_str().unwrap_or("").to_string());
        assert!(warn.is_none(), "全部可映射时不应有警告头: {warn:?}");
        // utf-8 导出同样无警告头
        let resp = export_book(
            AxumState(state.clone()),
            Query(params_gbk2(&[("format", "txt"), ("encoding", "utf-8")])),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get("X-Export-Warning").is_none());

        cleanup(state, dir).await;
    }

    /// percent 解码（测试用：解析 X-Export-Warning）
    fn percent_decode(s: &str) -> String {
        let b = s.as_bytes();
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'%' && i + 2 < b.len() {
                // 严格：% 后必须是两位十六进制
                let hex = |c: u8| -> Option<u8> {
                    match c {
                        b'0'..=b'9' => Some(c - b'0'),
                        b'a'..=b'f' => Some(c - b'a' + 10),
                        b'A'..=b'F' => Some(c - b'A' + 10),
                        _ => None,
                    }
                };
                if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                    out.push(h * 16 + l);
                    i += 3;
                    continue;
                }
            }
            out.push(b[i]);
            i += 1;
        }
        String::from_utf8(out).unwrap_or_default()
    }

    /// bookSourceDebugSSE：search 动作逐步骤事件（规则解析/URL 构造/请求/规则应用）→ result
    #[tokio::test]
    async fn test_book_source_debug_sse_search() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("dbgsearch").await;
        let base_url = serve_bodies(vec![
            r#"{"data":[{"name":"调试书","author":"甲","url":"/book/1"}]}"#.to_string(),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "调试源".into(),
                    search_url: Some(format!("{base}/search?q={{key}}")),
                    rule_search: Some(serde_json::json!({
                        "bookList": "$.data[*]",
                        "name": "$.name", "author": "$.author", "bookUrl": "$.url"
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let params: HashMap<String, String> = [
            ("bookSource".into(), base.clone()),
            ("action".into(), "search".into()),
            ("key".into(), "调试书".into()),
        ]
        .into_iter()
        .collect();
        let resp = book_source_debug_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("\"type\":\"start\""), "{body}");
        assert!(body.contains("\"type\":\"step\""), "应含 step 事件: {body}");
        assert!(
            body.contains("规则解析（ruleSearch）"),
            "应含规则解析步骤: {body}"
        );
        assert!(body.contains("URL 构造"), "应含 URL 构造步骤: {body}");
        assert!(body.contains("请求 URL"), "应含请求步骤: {body}");
        assert!(
            body.contains("规则应用（bookList 字段）"),
            "应含规则应用步骤: {body}"
        );
        assert!(
            body.contains("\"type\":\"result\""),
            "应含 result 事件: {body}"
        );
        assert!(
            body.contains("\"name\":\"调试书\""),
            "result 应含搜索结果: {body}"
        );
        assert!(
            body.contains("\"ruleName\":\"规则解析（ruleSearch）\""),
            "step 应含 ruleName 字段: {body}"
        );
        assert!(
            body.contains("bookListKind"),
            "step 应含规则解析明细: {body}"
        );

        // 缺 key → error 事件
        let params: HashMap<String, String> = [
            ("bookSource".into(), base.clone()),
            ("action".into(), "search".into()),
        ]
        .into_iter()
        .collect();
        let resp = book_source_debug_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("请输入搜索关键字"));

        // 非法动作 → error 事件
        let params: HashMap<String, String> = [
            ("bookSource".into(), base.clone()),
            ("action".into(), "bad".into()),
        ]
        .into_iter()
        .collect();
        let resp = book_source_debug_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("请输入调试动作"));

        cleanup(state, dir).await;
    }

    /// bookSourceDebugSSE：toc / content 动作
    #[tokio::test]
    async fn test_book_source_debug_sse_toc_content() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("dbgtoc").await;
        let base_url = serve_bodies(vec![
            r#"<ul class="chapters"><li><a href="/ch.html">第一章</a></li></ul>"#.to_string(),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "目录源".into(),
                    rule_toc: Some(serde_json::json!({
                        "chapterList": "ul.chapters@li", "chapterName": "a@text", "chapterUrl": "a@href"
                    })),
                    rule_content: Some(serde_json::json!({ "content": "div.content@text" })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let params: HashMap<String, String> = [
            ("bookSource".into(), base.clone()),
            ("action".into(), "toc".into()),
            ("url".into(), format!("{base}/toc.html")),
        ]
        .into_iter()
        .collect();
        let resp = book_source_debug_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("规则解析（ruleToc）"), "{body}");
        assert!(
            body.contains("chapterList 提取"),
            "应含 chapterList 提取步骤: {body}"
        );
        assert!(
            body.contains("字段规则（chapterName/chapterUrl）"),
            "{body}"
        );
        assert!(
            body.contains("\"title\":\"第一章\""),
            "result 应含章节: {body}"
        );

        // content
        let base_url = serve_bodies(vec![
            r#"<html><body><div class="content">正文一。</div></body></html>"#.to_string(),
        ])
        .await;
        let base2 = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base2.clone(),
                    book_source_name: "正文源".into(),
                    rule_content: Some(serde_json::json!({ "content": "div.content@text" })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let params: HashMap<String, String> = [
            ("bookSource".into(), base2.clone()),
            ("action".into(), "content".into()),
            ("chapterUrl".into(), format!("{base2}/ch.html")),
        ]
        .into_iter()
        .collect();
        let resp = book_source_debug_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("规则解析（ruleContent）"), "{body}");
        assert!(body.contains("content 规则应用"), "{body}");
        assert!(
            body.contains("\"content\":\"正文一。\""),
            "result 应含正文: {body}"
        );

        // 缺 url → error 事件（toc）
        let params: HashMap<String, String> = [
            ("bookSource".into(), base.clone()),
            ("action".into(), "toc".into()),
        ]
        .into_iter()
        .collect();
        let resp = book_source_debug_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("请输入目录链接"));

        cleanup(state, dir).await;
    }

    /// cacheBookOnServer / cacheBookSSE / cancelCacheBook：本地书（无网络）
    #[tokio::test]
    async fn test_cache_book_api() {
        let (state, dir) = test_state("cachebook").await;
        let book_url = "local://cache1";
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.into(),
                    name: "缓存书".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                book_url,
                &[
                    ("第一章".to_string(), "正文一".to_string()),
                    ("第二章".to_string(), "正文二".to_string()),
                ],
            )
            .await
            .unwrap();
        let params: HashMap<String, String> =
            [("url".into(), book_url.into())].into_iter().collect();

        // 启动任务
        let ret = cache_book_on_server(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(ret.0.data["started"].as_bool().unwrap());
        // 等待完成
        assert!(
            crate::service::cache_job::wait_finished(book_url, std::time::Duration::from_secs(5))
                .await,
            "任务应在 5s 内完成"
        );
        let p = crate::service::cache_job::progress_of(book_url).unwrap();
        let p = p.lock().unwrap_or_else(|e| e.into_inner());
        assert!(p.finished);
        assert_eq!(p.total, 2);
        assert_eq!(p.cached, 2);
        assert_eq!(p.title, "缓存书");
        drop(p);

        // SSE 进度流
        let resp = cache_book_sse(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("\"cached\":2"), "{body}");
        assert!(body.contains("\"total\":2"), "{body}");
        assert!(body.contains("\"finished\":true"), "{body}");

        // cancel：任务已完成但仍在表内 → true；再 cancel → false
        let ret = cancel_cache_book(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert!(ret.0.data["cancelled"].as_bool().unwrap());
        let ret = cancel_cache_book(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.data["cancelled"].as_bool().unwrap());

        // 未知任务 SSE → 自启动语义：url 不在书架 → 报「书籍不存在」（不再报任务不存在）
        let ghost: HashMap<String, String> = [("url".into(), "local://ghost".into())]
            .into_iter()
            .collect();
        let resp = cache_book_sse(
            AxumState(state.clone()),
            Query(ghost),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("书籍不存在"));

        // 书不存在 → 不启动
        let ghost: HashMap<String, String> = [("url".into(), "local://ghost2".into())]
            .into_iter()
            .collect();
        let ret = cache_book_on_server(
            AxumState(state.clone()),
            Query(ghost),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "书籍不存在（请先加入书架）");

        cleanup(state, dir).await;
    }

    /// cacheBookOnServer：书源书后台缓存（目录 → 并发 3 逐章 → 缓存表）
    /// 注意：mock 按请求路径返回（并发抓章时请求到达序不定——路径寻址保证确定性）
    #[tokio::test]
    async fn test_cache_book_web_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("cacheweb").await;
        let base_url = serve_bodies_by_path(vec![
            // upsert_book 已持久化 toc_url → 任务直接抓 /toc 目录页
            ("/toc".to_string(), r#"<ul class="chapters"><li><a href="/ch1.html">第一章</a></li><li><a href="/ch2.html">第二章</a></li></ul>"#.to_string()),
            ("/book/cache".to_string(), r#"<ul class="chapters"><li><a href="/ch1.html">第一章</a></li><li><a href="/ch2.html">第二章</a></li></ul>"#.to_string()),
            ("/ch1.html".to_string(), r#"<html><body><div class="content">正文一。</div></body></html>"#.to_string()),
            ("/ch2.html".to_string(), r#"<html><body><div class="content">正文二。</div></body></html>"#.to_string()),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "缓存源".into(),
                    rule_toc: Some(serde_json::json!({
                        "chapterList": "ul.chapters@li", "chapterName": "a@text", "chapterUrl": "a@href"
                    })),
                    rule_content: Some(serde_json::json!({ "content": "div.content@text" })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let book_url = format!("{base}/book/cache");
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.clone(),
                    name: "缓存网文书".into(),
                    origin: base.clone(),
                    toc_url: format!("{base}/toc"),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let params: HashMap<String, String> =
            [("url".into(), book_url.clone())].into_iter().collect();
        let ret = cache_book_on_server(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(
            crate::service::cache_job::wait_finished(&book_url, std::time::Duration::from_secs(10))
                .await,
            "书源书缓存应在 10s 内完成"
        );
        let p = crate::service::cache_job::progress_of(&book_url).unwrap();
        let p = p.lock().unwrap_or_else(|e| e.into_inner());
        assert!(p.finished, "任务应结束");
        assert_eq!(p.total, 2);
        assert_eq!(p.cached, 2, "两章都应缓存成功: {p:?}");
        assert_eq!(p.title, "缓存网文书");
        drop(p);
        // 缓存表已写入（chapterUrl md5 键）
        let idx1 = crate::util::md5::chapter_url_hash(&format!("{base}/ch1.html"));
        let idx2 = crate::util::md5::chapter_url_hash(&format!("{base}/ch2.html"));
        assert_eq!(
            state
                .storage
                .get_chapter_content("default", &book_url, idx1)
                .await
                .unwrap()
                .as_deref(),
            Some("正文一。")
        );
        assert_eq!(
            state
                .storage
                .get_chapter_content("default", &book_url, idx2)
                .await
                .unwrap()
                .as_deref(),
            Some("正文二。")
        );
        // 清理任务表
        crate::service::cache_job::cancel(&book_url);
        cleanup(state, dir).await;
    }

    /// cacheBookRangeOnServer + getBookCacheChapters：范围任务 taskId / 批量拉取已缓存章节
    #[tokio::test]
    async fn test_cache_book_range_and_fetch_api() {
        let (state, dir) = test_state("cacherange").await;
        let book_url = "local://cache-range";
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.into(),
                    name: "范围缓存书".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                book_url,
                &[
                    ("第一章".to_string(), "正文一".to_string()),
                    ("第二章".to_string(), "正文二".to_string()),
                    ("第三章".to_string(), "正文三".to_string()),
                ],
            )
            .await
            .unwrap();

        // 范围任务：from=1&to=2 → taskId 后缀 + 进度 2/2
        let params: HashMap<String, String> = [
            ("url".into(), book_url.into()),
            ("from".into(), "1".into()),
            ("to".into(), "2".into()),
        ]
        .into_iter()
        .collect();
        let ret = cache_book_range_on_server(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let task_id = ret.0.data["taskId"].as_str().unwrap().to_string();
        assert_eq!(task_id, "local://cache-range#1-2");
        // 本地书任务立即结束（章节已在库）；轮询等待 finished
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let p = crate::service::cache_job::progress_of_key(&task_id)
                .map(|p| p.lock().unwrap_or_else(|e| e.into_inner()).clone());
            if let Some(p) = p {
                if p.finished {
                    assert_eq!(p.total, 2);
                    assert_eq!(p.cached, 2);
                    break;
                }
            }
            assert!(std::time::Instant::now() < deadline, "范围任务超时");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        // 批量拉取：全量 3 章；范围 1-2 只回第二章
        let fetch_all: HashMap<String, String> =
            [("url".into(), book_url.into())].into_iter().collect();
        let ret = get_book_cache_chapters(
            AxumState(state.clone()),
            Query(fetch_all),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let chapters = ret.0.data["chapters"].as_array().unwrap();
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[1]["title"], "第二章");
        assert_eq!(chapters[1]["content"], "正文二");

        let ret = get_book_cache_chapters(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let chapters = ret.0.data["chapters"].as_array().unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0]["index"], 1);
        assert_eq!(chapters[1]["index"], 2);

        // 范围参数错误：from>to
        let bad: HashMap<String, String> = [
            ("url".into(), book_url.into()),
            ("from".into(), "2".into()),
            ("to".into(), "1".into()),
        ]
        .into_iter()
        .collect();
        let ret = cache_book_range_on_server(
            AxumState(state.clone()),
            Query(bad),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "缓存范围参数错误");

        // 清理任务表
        crate::service::cache_job::cancel_key(&task_id);
        cleanup(state, dir).await;
    }

    /// cacheBookRangeOnServer JSON body 数值参数：from/to 为 number 时也应解析成功
    #[tokio::test]
    async fn test_cache_book_range_json_body_numeric_params() {
        let (state, dir) = test_state("cacherange-json").await;
        let book_url = "local://cache-range-json";
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.into(),
                    name: "JSON 范围缓存书".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_chapters(
                "default",
                book_url,
                &[
                    ("第一章".to_string(), "正文一".to_string()),
                    ("第二章".to_string(), "正文二".to_string()),
                    ("第三章".to_string(), "正文三".to_string()),
                ],
            )
            .await
            .unwrap();

        let body = Bytes::from(r#"{"url":"local://cache-range-json","from":0,"to":1}"#.to_string());
        let ret = cache_book_range_on_server(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let task_id = ret.0.data["taskId"].as_str().unwrap().to_string();
        assert_eq!(task_id, "local://cache-range-json#0-1");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let p = crate::service::cache_job::progress_of_key(&task_id)
                .map(|p| p.lock().unwrap_or_else(|e| e.into_inner()).clone());
            if let Some(p) = p {
                if p.finished {
                    assert_eq!(p.total, 2);
                    assert_eq!(p.cached, 2);
                    break;
                }
            }
            assert!(std::time::Instant::now() < deadline, "JSON 范围任务超时");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        crate::service::cache_job::cancel_key(&task_id);
        cleanup(state, dir).await;
    }

    /// getUserConfig / saveUserConfig：读写覆盖 + 用户隔离（secure 模式）
    #[tokio::test]
    async fn test_user_config_api() {
        let (state, dir) = test_state("userconf").await;

        // 保存 {ns, config}
        let body = Bytes::from(r#"{"ns":"reader","config":{"fontSize":18,"theme":"dark"}}"#);
        let ret = save_user_config(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        // 读取
        let params: HashMap<String, String> =
            [("key".into(), "reader".into())].into_iter().collect();
        let ret = get_user_config(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["fontSize"], 18);
        assert_eq!(ret.0.data["theme"], "dark");
        // 覆盖
        let body = Bytes::from(r#"{"ns":"reader","config":{"fontSize":20}}"#);
        let ret = save_user_config(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let params: HashMap<String, String> =
            [("key".into(), "reader".into())].into_iter().collect();
        let ret = get_user_config(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data["fontSize"], 20);
        // 未设置的 key → null
        let params: HashMap<String, String> =
            [("key".into(), "ghost".into())].into_iter().collect();
        let ret = get_user_config(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success, "缺配置应报「没有备份文件」");
        assert_eq!(ret.0.error_msg, "没有备份文件");
        // 裸 JSON 整体保存（无 config 键；默认 ns=global）
        let body = Bytes::from(r#"{"fontSize":16}"#);
        let ret = save_user_config(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let ret = get_user_config(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data["fontSize"], 16);
        // 非 JSON → 参数错误
        let ret = save_user_config(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(Bytes::from("nope")),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        // secure 模式用户隔离
        let mut state = state;
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                token: "t2".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let body = Bytes::from(r#"{"ns":"pref","config":{"a":1}}"#);
        let params: HashMap<String, String> = [("accessToken".into(), "alice:t1".into())]
            .into_iter()
            .collect();
        let ret = save_user_config(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let mut q = params.clone();
        q.insert("key".into(), "pref".into());
        let ret = get_user_config(AxumState(state.clone()), Query(q), HeaderMap::new(), None).await;
        assert_eq!(ret.0.data["a"], 1);
        // bob 读不到 alice 的配置
        let qb: HashMap<String, String> = [
            ("accessToken".into(), "bob:t2".into()),
            ("key".into(), "pref".into()),
        ]
        .into_iter()
        .collect();
        let ret = get_user_config(
            AxumState(state.clone()),
            Query(qb.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.data.is_null(), "bob 不应看到 alice 配置: {ret:?}");
        // bob 覆盖自己的配置不影响 alice
        let body = Bytes::from(r#"{"ns":"pref","config":{"a":2}}"#);
        let ret = save_user_config(
            AxumState(state.clone()),
            Query(qb),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        let mut q = params;
        q.insert("key".into(), "pref".into());
        let ret = get_user_config(AxumState(state.clone()), Query(q), HeaderMap::new(), None).await;
        assert_eq!(ret.0.data["a"], 1, "alice 配置不受 bob 影响");

        cleanup(state, dir).await;
    }

    /// refreshLocalBook：local:// 重解析原文件 + 文件书重解析 + 非本地书拒绝
    #[tokio::test]
    async fn test_refresh_local_book_api() {
        let (state, dir) = test_state("refreshlocal").await;
        // local:// 书：原文件在 opds_files/{id}.txt
        let id = "book-abc";
        let opds_dir = state
            .storage
            .config
            .storage_dir()
            .join("data/default/opds_files");
        std::fs::create_dir_all(&opds_dir).unwrap();
        std::fs::write(
            opds_dir.join(format!("{id}.txt")),
            "第一章 起点\n内容一。\n第二章 成长\n内容二。",
        )
        .unwrap();
        let book_url = format!("local://{id}");
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.clone(),
                    name: "刷新书".into(),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let params: HashMap<String, String> =
            [("url".into(), book_url.clone())].into_iter().collect();
        let ret = refresh_local_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["chapterCount"], 2);
        assert_eq!(ret.0.data["totalChapterNum"], 2, "应返回新 totalChapterNum");
        assert_eq!(ret.0.data["name"], "刷新书");
        assert_eq!(
            state.storage.list_chapters(&book_url).await.unwrap().len(),
            2,
            "章节已重扫入库"
        );
        assert_eq!(
            state
                .storage
                .find_book("default", &book_url)
                .await
                .unwrap()
                .unwrap()
                .total_chapter_num,
            2,
            "totalChapterNum 已更新"
        );

        // 文件型本地书（storage/ 路径）
        let file_dir = state
            .storage
            .config
            .storage_dir()
            .join("data/default/books");
        std::fs::create_dir_all(&file_dir).unwrap();
        std::fs::write(
            file_dir.join("示例2.txt"),
            "第一章 起点\n内容一。\n第二章 成长\n内容二。\n第三章 终章\n内容三。",
        )
        .unwrap();
        let fbook = "storage/data/default/books/示例2.txt";
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: fbook.into(),
                    name: "文件书".into(),
                    origin: "loc_book".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let params: HashMap<String, String> = [("url".into(), fbook.into())].into_iter().collect();
        let ret = refresh_local_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["chapterCount"], 3);
        assert_eq!(ret.0.data["totalChapterNum"], 3);

        // 非本地书 → 拒绝；不存在 → 书籍不存在；缺 url → 参数错误
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://web.com/a".into(),
                    name: "网文".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let params: HashMap<String, String> = [("url".into(), "https://web.com/a".into())]
            .into_iter()
            .collect();
        let ret = refresh_local_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "仅支持本地书刷新");
        let params: HashMap<String, String> = [("url".into(), "local://ghost".into())]
            .into_iter()
            .collect();
        let ret = refresh_local_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书籍不存在");
        let ret = refresh_local_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// scanLocalBookDir：书仓目录已有书籍直接导入书架（无需上传）；重复扫描幂等
    #[tokio::test]
    async fn test_scan_local_book_dir_api() {
        let (state, dir) = test_state("scanstore").await;
        let store_dir = state.storage.config.storage_dir().join("localStore");
        let root = store_dir.join("books");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(
            root.join("示例一.txt"),
            "第一章 起点\n内容一。\n第二章 成长\n内容二。",
        )
        .unwrap();
        std::fs::write(
            root.join("sub").join("示例二.txt"),
            "第一章 开始\n甲。\n第二章 结束\n乙。\n第三章 终章\n丙。",
        )
        .unwrap();
        // 不支持的扩展名应被跳过
        std::fs::write(root.join("说明.md"), "# 说明").unwrap();

        let body = Bytes::from(r#"{"path":"/books","home":"__LOCAL_STORE__","recursive":true}"#);
        let ret = scan_local_book_dir(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["imported"], 2, "应导入两本 txt");
        assert_eq!(ret.0.data["failed"], 0, "{:?}", ret.0.data["errors"]);

        let books = state.storage.list_books("default").await.unwrap();
        let scanned: Vec<_> = books
            .iter()
            .filter(|b| b.book_url.starts_with("local://store/"))
            .collect();
        assert_eq!(scanned.len(), 2, "稳定 book_url 去重，不应产生重复副本");
        for b in &scanned {
            assert!(!b.toc_url.is_empty(), "toc_url 必须写入");
            assert!(b.total_chapter_num >= 2, "扫描导入必须写入总章数");
            assert!(
                b.local_file.as_ref().is_some_and(|p| !p.is_empty()),
                "local_file 必须关联原文件"
            );
        }
        // 章节已入库
        let chapters = state
            .storage
            .list_chapters(&scanned[0].book_url)
            .await
            .unwrap();
        assert_eq!(chapters.len(), 2);

        // 再次扫描：同一批文件导入（覆盖更新），数量仍为 2
        let body = Bytes::from(r#"{"path":"/books","home":"__LOCAL_STORE__","recursive":true}"#);
        let ret = scan_local_book_dir(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["imported"], 2);
        let books2 = state.storage.list_books("default").await.unwrap();
        assert_eq!(
            books2
                .iter()
                .filter(|b| b.book_url.starts_with("local://store/"))
                .count(),
            2,
            "重复扫描不应新增书"
        );

        // 穿越路径拒绝
        let body = Bytes::from(r#"{"path":"/../../etc","home":"__LOCAL_STORE__"}"#);
        let ret = scan_local_book_dir(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);

        cleanup(state, dir).await;
    }

    /// deleteBooks：批量删除（含章节清理）与参数校验
    #[tokio::test]
    async fn test_delete_books_api() {
        let (state, dir) = test_state("delbooks").await;
        for (url, name) in [
            ("https://b.com/1", "书1"),
            ("https://b.com/2", "书2"),
            ("https://b.com/3", "书3"),
        ] {
            state
                .storage
                .upsert_book(
                    "default",
                    &crate::model::Book {
                        book_url: url.into(),
                        name: name.into(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
        }
        state
            .storage
            .save_chapters("default", "https://b.com/1", &[("第一章".into(), "正文".into())])
            .await
            .unwrap();

        let body = Bytes::from(r#"{"bookUrls":["https://b.com/1","https://b.com/2"]}"#);
        let ret = delete_books(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        assert!(state
            .storage
            .find_book("default", "https://b.com/1")
            .await
            .unwrap()
            .is_none());
        assert!(state
            .storage
            .find_book("default", "https://b.com/2")
            .await
            .unwrap()
            .is_none());
        assert!(state
            .storage
            .find_book("default", "https://b.com/3")
            .await
            .unwrap()
            .is_some());
        assert_eq!(
            state
                .storage
                .count_chapters("default", "https://b.com/1")
                .await
                .unwrap(),
            0,
            "章节连带删除"
        );

        // 参数校验
        let ret = delete_books(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let body = Bytes::from(r#"{"bookUrls":[]}"#);
        let ret = delete_books(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// deleteBookmarks：批量删书签（{bookUrl, ids}）与参数校验
    #[tokio::test]
    async fn test_delete_bookmarks_api() {
        let (state, dir) = test_state("delbms").await;
        for (i, title) in ["m1", "m2", "m3"].iter().enumerate() {
            state
                .storage
                .save_bookmark(
                    "default",
                    &crate::model::Bookmark {
                        book_url: "https://b.com/1".into(),
                        title: (*title).into(),
                        paragraph_index: i as i64,
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
        }
        // 删两条
        let body = Bytes::from(r#"{"bookUrl":"https://b.com/1","ids":["m1","m3"]}"#);
        let ret = delete_bookmarks(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let rest = state
            .storage
            .list_bookmarks("default", "https://b.com/1")
            .await
            .unwrap();
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].title, "m2");
        // 参数校验
        let body = Bytes::from(r#"{"bookUrl":"https://b.com/1","ids":[]}"#);
        let ret = delete_bookmarks(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let ret = delete_bookmarks(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// saveRssSources：批量保存 RSS 源（覆盖 + 校验）
    #[tokio::test]
    async fn test_save_rss_sources_api() {
        let (state, dir) = test_state("save_rss").await;
        let body = Bytes::from(
            r#"[{"sourceUrl":"https://r1.com/feed","sourceName":"源1","sourceGroup":"科技","enabled":true},
                {"sourceUrl":"https://r2.com/feed","sourceName":"源2","enabled":false}]"#,
        );
        let ret = save_rss_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let list = state.storage.get_rss_sources("default").await.unwrap();
        assert_eq!(list.len(), 2);
        let s1 = list
            .iter()
            .find(|s| s.source_url == "https://r1.com/feed")
            .unwrap();
        assert_eq!(s1.source_name, "源1");
        assert_eq!(s1.source_group.as_deref(), Some("科技"));
        assert!(s1.enabled);
        let s2 = list
            .iter()
            .find(|s| s.source_url == "https://r2.com/feed")
            .unwrap();
        assert!(!s2.enabled);
        // 覆盖同 url 不新增
        let body = Bytes::from(r#"[{"sourceUrl":"https://r1.com/feed","sourceName":"源1v2"}]"#);
        let ret = save_rss_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(
            state
                .storage
                .get_rss_sources("default")
                .await
                .unwrap()
                .len(),
            2
        );
        // 校验：无 body → 参数错误
        let ret = save_rss_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        // 空数组：legacy 语义存空列表返回成功
        let body = Bytes::from(r#"[]"#);
        let ret = save_rss_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "空数组应成功（legacy 存空列表语义）");
        // 非法条目（缺 sourceName）静默跳过——legacy continue 语义，不整批拒绝
        let body = Bytes::from(r#"[{"sourceUrl":"https://r3.com/feed"}]"#);
        let ret = save_rss_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "非法条目应跳过而非拒绝（legacy）");
        assert_eq!(
            state
                .storage
                .get_rss_sources("default")
                .await
                .unwrap()
                .iter()
                .filter(|s| s.source_url == "https://r3.com/feed")
                .count(),
            0,
            "缺 sourceName 的条目不应入库"
        );

        cleanup(state, dir).await;
    }

    /// markRssArticleRead：标记已读/未读（body {articleUrl, read}）
    #[tokio::test]
    async fn test_mark_rss_article_read_api() {
        let (mut state, dir) = test_state("mark_read").await;
        let article = crate::model::RssArticle {
            url: "https://feed.example.com/x".into(),
            source_url: "https://feed.example.com/rss".into(),
            title: "X".into(),
            ..Default::default()
        };
        state
            .storage
            .save_rss_articles("default", &[article])
            .await
            .unwrap();
        // 已读
        let body = Bytes::from(r#"{"articleUrl":"https://feed.example.com/x","read":true}"#);
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let got = state
            .storage
            .get_rss_article("default", "https://feed.example.com/x")
            .await
            .unwrap()
            .unwrap();
        assert!(got.read);
        // 标回未读
        let body = Bytes::from(r#"{"articleUrl":"https://feed.example.com/x","read":false}"#);
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert!(
            !state
                .storage
                .get_rss_article("default", "https://feed.example.com/x")
                .await
                .unwrap()
                .unwrap()
                .read
        );
        // 参数校验：缺 articleUrl
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS文章链接不能为空");
        // 未登录（secure 模式）拒绝
        state.storage.config.secure = true;
        let body = Bytes::from(r#"{"articleUrl":"https://feed.example.com/x","read":true}"#);
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);

        // P0-4 跨用户拒绝：bob 的文章，alice 标记 → 显式拒绝且不影响 bob 状态
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "tok_alice".into(),
                enable_rss_source: true,
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                token: "tok_bob".into(),
                enable_rss_source: true,
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .save_rss_articles(
                "bob",
                &[crate::model::RssArticle {
                    url: "https://feed.example.com/bob-only".into(),
                    source_url: "https://feed.example.com/rss".into(),
                    title: "Bob".into(),
                    ..Default::default()
                }],
            )
            .await
            .unwrap();
        let alice_params: HashMap<String, String> =
            [("accessToken".into(), "alice:tok_alice".into())]
                .into_iter()
                .collect();
        let body = Bytes::from(r#"{"articleUrl":"https://feed.example.com/bob-only","read":true}"#);
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(alice_params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success, "跨用户标记应被拒绝");
        assert_eq!(ret.0.error_msg, "文章不存在或无权操作");
        let bob_art = state
            .storage
            .get_rss_article("bob", "https://feed.example.com/bob-only")
            .await
            .unwrap()
            .unwrap();
        assert!(!bob_art.read, "跨用户标记不得改动他人已读状态");
        // 本人（bob）标记成功
        let bob_params: HashMap<String, String> = [("accessToken".into(), "bob:tok_bob".into())]
            .into_iter()
            .collect();
        let body = Bytes::from(r#"{"articleUrl":"https://feed.example.com/bob-only","read":true}"#);
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(bob_params),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "本人标记应成功: {}", ret.0.error_msg);
        assert!(
            state
                .storage
                .get_rss_article("bob", "https://feed.example.com/bob-only")
                .await
                .unwrap()
                .unwrap()
                .read
        );

        cleanup(state, dir).await;
    }

    /// saveBookmarks：批量保存书签（createdAt 自动补）
    #[tokio::test]
    async fn test_save_bookmarks_api() {
        let (state, dir) = test_state("savebms").await;
        let body = Bytes::from(
            r#"[{"bookUrl":"https://b.com/1","title":"书签甲","bookName":"三体","bookAuthor":"刘慈欣","chapterPos":3,"chapterIndex":1,"chapterName":"第一章","bookText":"这是书签内容","content":"备注A","time":1700000000000},
                {"bookUrl":"https://b.com/1","title":"书签乙"}]"#,
        );
        let ret = save_bookmarks(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let list = state
            .storage
            .list_bookmarks("default", "https://b.com/1")
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
        let jia = list.iter().find(|b| b.title == "书签甲").unwrap();
        assert_eq!(jia.paragraph_index, 3);
        assert_eq!(
            jia.created_at, 1700000000000,
            "legacy time 应映射 createdAt"
        );
        assert_eq!(jia.book_name, "三体");
        assert_eq!(jia.book_author, "刘慈欣");
        assert_eq!(jia.chapter_name, "第一章");
        assert_eq!(jia.book_text, "这是书签内容");
        assert_eq!(jia.content, "备注A");
        let yi = list.iter().find(|b| b.title == "书签乙").unwrap();
        assert!(yi.created_at > 0, "createdAt 应自动补");
        // 校验
        let body = Bytes::from(r#"[{"bookUrl":"https://b.com/1"}]"#);
        let ret = save_bookmarks(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        let ret = save_bookmarks(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// addBookGroupMulti / removeBookGroupMulti：批量设分组/移出分组
    #[tokio::test]
    async fn test_book_group_multi_api() {
        let (state, dir) = test_state("grpmulti").await;
        let g = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "玄幻".into(),
                    order: 1,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        for url in ["https://b.com/1", "https://b.com/2", "https://b.com/3"] {
            state
                .storage
                .upsert_book(
                    "default",
                    &crate::model::Book {
                        book_url: url.into(),
                        name: url.into(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
        }
        // 批量设分组
        let body = Bytes::from(format!(
            r#"{{"bookUrls":["https://b.com/1","https://b.com/2"],"groupId":{}}}"#,
            g.id
        ));
        let ret = add_book_group_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/1")
                .await
                .unwrap()
                .unwrap()
                .group,
            g.id
        );
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/2")
                .await
                .unwrap()
                .unwrap()
                .group,
            g.id
        );
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/3")
                .await
                .unwrap()
                .unwrap()
                .group,
            0
        );
        // 参数校验
        let body = Bytes::from(r#"{"bookUrls":["https://b.com/1"],"groupId":-1}"#);
        let ret = add_book_group_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        // 批量移出
        let body = Bytes::from(r#"{"bookUrls":["https://b.com/1","https://b.com/3"]}"#);
        let ret = remove_book_group_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["count"], 1, "变更行数（仅 b1 实际移出分组）");
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/1")
                .await
                .unwrap()
                .unwrap()
                .group,
            0
        );
        assert_eq!(
            state
                .storage
                .find_book("default", "https://b.com/2")
                .await
                .unwrap()
                .unwrap()
                .group,
            g.id,
            "未涉及的保持"
        );
        // 多分组：追加第二个分组后批量移除该分组，group_ids 应同步保留第一分组
        let g2 = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "都市".into(),
                    order: 2,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .add_book_group_multi("default", &["https://b.com/2".to_string()], g2.id)
            .await
            .unwrap();
        let body = Bytes::from(format!(
            r#"{{"bookUrls":["https://b.com/2"],"groupId":{}}}"#,
            g2.id
        ));
        let ret = remove_book_group_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 1);
        let b2 = state
            .storage
            .find_book("default", "https://b.com/2")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(b2.group, g.id, "移除次要分组后主分组保持第一项");
        assert_eq!(
            serde_json::from_str::<Vec<i64>>(&b2.group_ids).unwrap(),
            vec![g.id],
            "group_ids 应只保留第一分组"
        );
        let ret = remove_book_group_multi(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// saveBookGroupOrder：分组排序批量保存
    #[tokio::test]
    async fn test_save_book_group_order_api() {
        let (state, dir) = test_state("grporder").await;
        let g1 = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "甲".into(),
                    order: 1,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let g2 = state
            .storage
            .save_book_group(
                "default",
                &crate::model::BookGroup {
                    name: "乙".into(),
                    order: 2,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let body = Bytes::from(format!(
            r#"{{"order":[{{"id":{},"orderNum":2}},{{"id":{},"orderNum":1}}]}}"#,
            g1.id, g2.id
        ));
        let ret = save_book_group_order(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let list = state.storage.list_book_groups("default").await.unwrap();
        assert_eq!(list[0].id, g2.id, "乙应排第一");
        assert_eq!(list[0].order, 1);
        assert_eq!(list[1].id, g1.id);
        assert_eq!(list[1].order, 2);
        // 参数校验
        let body = Bytes::from(r#"{"order":[]}"#);
        let ret = save_book_group_order(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// getAvailableBookSource：启用过滤 + key 可搜索过滤 + bookUrlPattern 规则过滤
    #[tokio::test]
    async fn test_get_available_book_source_api() {
        let (state, dir) = test_state("availsrc").await;
        // 书架书（换源语义：按书取候选，而非旧版的书源清单过滤）
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://a.com/book/1".into(),
                    name: "测试书".into(),
                    author: "作者A".into(),
                    origin: "https://s0.com".into(),
                    is_in_shelf: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // 缺 url → 请输入书籍链接
        let ret = get_available_book_source(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "请输入书籍链接");

        // 非书架书 → 书籍信息错误
        let params: HashMap<String, String> = [("url".into(), "https://none.com/b".into())]
            .into_iter()
            .collect();
        let ret = get_available_book_source(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "书籍信息错误");

        // 预置持久化候选 → refresh=0 原样返回（SearchBook 形态）
        let cands = vec![crate::service::search::SearchBook {
            book_url: "https://s1.com/b/1".into(),
            origin: "https://s1.com".into(),
            origin_name: "源1".into(),
            name: "测试书".into(),
            author: "作者A".into(),
            ..Default::default()
        }];
        state
            .storage
            .save_book_candidates("default", "测试书_作者A", &cands)
            .await
            .unwrap();
        let params: HashMap<String, String> = [("url".into(), "https://a.com/book/1".into())]
            .into_iter()
            .collect();
        let ret = get_available_book_source(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["bookUrl"], "https://s1.com/b/1");
        assert_eq!(arr[0]["origin"], "https://s1.com");

        cleanup(state, dir).await;
    }

    /// getInvalidBookSources：返回运行期失败快照（600 秒 TTL），不再并发探测
    #[tokio::test]
    async fn test_get_invalid_book_sources_api() {
        let ns = "default"; // 非 secure 模式 resolve_namespace 恒为 default
        let bad_url = "http://127.0.0.1:1";
        // 清理本键可能的历史残留（全局静态，测试进程共享）
        crate::service::health::clear_source_invalid(ns, bad_url);
        let has_bad = |arr: &Vec<serde_json::Value>| arr.iter().any(|v| v["sourceUrl"] == bad_url);

        let (state, dir) = test_state(ns).await;
        // 无运行期失败记录时不探测（好源/坏源均不出现在响应）
        let ret = get_invalid_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(!has_bad(ret.0.data.as_array().unwrap()));

        // 运行期标记失败 → 快照直接返回（legacy 形状 sourceUrl/time/error）
        crate::service::health::mark_source_invalid(ns, bad_url, "连接失败: refused");
        let ret = get_invalid_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        let item = arr
            .iter()
            .find(|v| v["sourceUrl"] == bad_url)
            .expect("快照应含被标记的坏源");
        assert_eq!(item["bookSourceUrl"], bad_url);
        assert!(item["time"].as_i64().unwrap() > 0, "应携带记录时间戳(ms)");
        assert!(item["error"].as_str().unwrap().contains("连接失败"));
        assert_eq!(item["errorMsg"], item["error"]);

        // 成功抓取清除标记 → 该源从响应消失
        crate::service::health::clear_source_invalid(ns, bad_url);
        let ret = get_invalid_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert!(!has_bad(ret.0.data.as_array().unwrap()));

        cleanup(state, dir).await;
    }

    /// setAsDefaultBookSources：默认书源标记（字符串数组 / 对象数组）
    #[tokio::test]
    async fn test_set_as_default_book_sources_api() {
        let (state, dir) = test_state("defaultsrc").await;
        let body =
            Bytes::from(r#"{"bookSources":["https://s1.com",{"bookSourceUrl":"https://s2.com"}]}"#);
        let ret = set_as_default_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 2);
        let list = state
            .storage
            .get_default_book_sources("default")
            .await
            .unwrap();
        assert_eq!(
            list,
            vec!["https://s1.com".to_string(), "https://s2.com".to_string()]
        );
        // 覆盖
        let body = Bytes::from(r#"{"bookSources":["https://s3.com"]}"#);
        let ret = set_as_default_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(
            state
                .storage
                .get_default_book_sources("default")
                .await
                .unwrap(),
            vec!["https://s3.com".to_string()]
        );
        // 校验
        let body = Bytes::from(r#"{"bookSources":[]}"#);
        let ret = set_as_default_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");

        cleanup(state, dir).await;
    }

    /// searchBookSourceSSE：流式换源（逐书源无名 data 事件 → event: end，legacy 对齐）
    #[tokio::test]
    async fn test_search_book_source_sse_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("srcsse").await;
        // 当前源 s1（排除）
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s1.com".into(),
                    book_source_name: "源1".into(),
                    enabled: true,
                    search_url: Some("https://s1.com/s?q={{key}}".into()),
                    rule_search: Some(serde_json::json!({
                        "bookList": "$.data[*]",
                        "name": "$.name", "author": "$.author", "bookUrl": "$.url", "tocUrl": "$.toc"
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        // s2 → 本地测试服务器（返回命中结果）
        let base_url = serve_bodies(vec![
            r#"{"data":[{"name":"测试书","author":"甲","url":"/book/9","toc":"/toc"}]}"#
                .to_string(),
        ])
        .await;
        let base = base_url.trim_end_matches("/sources.json").to_string();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "源2".into(),
                    enabled: true,
                    search_url: Some(format!("{base}/search?q={{key}}")),
                    rule_search: Some(serde_json::json!({
                        "bookList": "$.data[*]",
                        "name": "$.name", "author": "$.author", "bookUrl": "$.url", "tocUrl": "$.toc"
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://s1.com/book/1".into(),
                    name: "测试书".into(),
                    origin: "https://s1.com".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let params: HashMap<String, String> = [
            ("url".into(), "https://s1.com/book/1".into()),
            ("bookSource".into(), "https://s1.com".into()),
        ]
        .into_iter()
        .collect();
        let resp = search_book_source_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        // legacy 对齐：数据事件为无名 data（旧客户端 onmessage 接收），不得再发 event: book
        assert!(
            !body.contains("event: book"),
            "不应含命名 book 事件: {body}"
        );
        assert!(body.contains("data: {"), "应含无名 data 事件: {body}");
        assert!(body.contains("\"name\":\"测试书\""), "命中书应推送: {body}");
        assert!(body.contains("event: end"), "应含 end 事件: {body}");
        assert!(body.contains("\"isEnd\":true"), "{body}");

        // 缺 url → error 事件
        let params: HashMap<String, String> = [("bookSource".into(), "https://s1.com".into())]
            .into_iter()
            .collect();
        let resp = search_book_source_sse(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(body.contains("请输入书籍链接"));

        cleanup(state, dir).await;
    }

    /// getReadingStats：saveBookProgress 增量累计时长/字数 → today/week/total/books
    #[tokio::test]
    async fn test_reading_stats_api() {
        let (state, dir) = test_state("readstats").await;
        let now = now_millis();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://stats.com/b".into(),
                    name: "统计书".into(),
                    dur_chapter_time: now - 10_000,
                    dur_chapter_pos: 0,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // 第一次进度：+10s / +500 字
        let body = Bytes::from(
            json!({"bookUrl":"https://stats.com/b","durChapterIndex":1,"durChapterPos":500,"durChapterTime":now}).to_string(),
        );
        let ret = save_book_progress(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        // 第二次：+5s / +200 字
        let body = Bytes::from(
            json!({"bookUrl":"https://stats.com/b","durChapterIndex":1,"durChapterPos":700,"durChapterTime":now + 5000}).to_string(),
        );
        let ret = save_book_progress(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);

        // storage 层汇总
        let stats = state.storage.get_reading_stats("default").await.unwrap();
        assert_eq!(stats.today, 15, "10s + 5s");
        assert_eq!(stats.total, 15);
        assert!(stats.week >= 15);
        assert_eq!(stats.books.len(), 1);
        assert_eq!(stats.books[0].book_url, "https://stats.com/b");
        assert_eq!(stats.books[0].name, "统计书");
        assert_eq!(stats.books[0].seconds, 15);
        assert_eq!(stats.books[0].chars, 700, "500 + 200 字");

        // handler 输出（camelCase）
        let ret = get_reading_stats(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data["today"], 15);
        assert_eq!(ret.0.data["total"], 15);
        assert_eq!(ret.0.data["books"][0]["chars"], 700);
        assert_eq!(ret.0.data["books"][0]["bookUrl"], "https://stats.com/b");

        // 未入架书 → 书籍未加入书架（不记统计）
        let body = Bytes::from(
            r#"{"bookUrl":"https://ghost.com/b","durChapterIndex":0,"durChapterPos":10}"#,
        );
        let ret = save_book_progress(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "书籍未加入书架");

        cleanup(state, dir).await;
    }

    /// 命名兼容批 2（端到端）：resetPassword / httpTTS / uploadFile 别名路由
    #[tokio::test]
    async fn test_alias_routes_batch2() {
        let (state, dir) = test_state("alias2").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let app = router(state.storage.config.clone(), state.storage.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let base = format!("http://{addr}");
        let client = reqwest::Client::new();

        // uploadFile（legacy UserController.uploadFile）：assets/{ns}/{type}/ 上传 →
        // URL 数组（缺省 type=images；与 file/upload 书仓语义同名异义）
        let boundary = "----reader-test-boundary";
        let multipart_body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"示例.txt\"\r\nContent-Type: text/plain\r\n\r\n第一章 起点\n内容一。\r\n--{boundary}--\r\n"
        );
        let resp = client
            .post(format!("{base}/reader3/uploadFile"))
            .query(&[("accessToken", "alice:t1")])
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(multipart_body)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let json: Value = resp.json().await.unwrap();
        assert!(
            json["isSuccess"].as_bool().unwrap(),
            "uploadFile 应成功: {json}"
        );
        let arr = json["data"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0], "/assets/alice/images/示例.txt");
        assert!(
            std::path::Path::new(
                &state
                    .storage
                    .config
                    .storage_dir()
                    .join("assets/alice/images/示例.txt")
            )
            .exists(),
            "文件应落盘"
        );

        // httpTTS（= tts）：未知引擎 → 业务错误（无网络请求）
        let resp = client
            .get(format!("{base}/reader3/httpTTS"))
            .query(&[
                ("accessToken", "alice:t1"),
                ("text", "你好"),
                ("engine", "nope"),
            ])
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let json: Value = resp.json().await.unwrap();
        assert_eq!(json["errorMsg"], "听书源不存在");

        // resetPassword（= resetUserPassword）：重置后旧 token 失效
        let resp = client
            .post(format!("{base}/reader3/resetPassword"))
            .query(&[("accessToken", "alice:t1"), ("secureKey", "sk")])
            .json(&json!({"username": "alice", "newPassword": "新密码123"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let json: Value = resp.json().await.unwrap();
        assert!(
            json["isSuccess"].as_bool().unwrap(),
            "resetPassword 应成功: {json}"
        );
        let alice = state.storage.find_user("alice").await.unwrap().unwrap();
        assert!(
            alice.password.starts_with("$argon2id$"),
            "重置后密码应为 argon2id PHC: {}",
            alice.password
        );
        assert!(
            crate::util::password::verify_argon2id("新密码123", &alice.password),
            "新密码应可校验"
        );
        assert!(alice.token.is_empty(), "旧 token 应失效");

        cleanup(state, dir).await;
    }

    /// 文件名收敛：`\\`→`/` 后取末段；空名/隐藏名拒绝
    #[test]
    fn test_sanitize_upload_filename() {
        assert_eq!(
            sanitize_upload_filename("a.png").as_deref(),
            Some("a.png"),
            "普通文件名原样"
        );
        assert_eq!(
            sanitize_upload_filename("sub/dir/pic a.png").as_deref(),
            Some("pic a.png"),
            "POSIX 子目录收敛为 basename"
        );
        assert_eq!(
            sanitize_upload_filename("docs\\note.txt").as_deref(),
            Some("note.txt"),
            "Windows 反斜杠路径收敛为 basename"
        );
        assert_eq!(sanitize_upload_filename(".."), None, ".. 拒绝");
        assert_eq!(sanitize_upload_filename("."), None, ". 拒绝");
        assert_eq!(sanitize_upload_filename(".hidden"), None, "隐藏文件拒绝");
        assert_eq!(sanitize_upload_filename(""), None, "空名拒绝");
        assert_eq!(
            sanitize_upload_filename("a\\..\\..\\evil.txt").as_deref(),
            Some("evil.txt"),
            "穿越片段随 basename 收敛消除"
        );
    }

    /// uploadFile（legacy UserController.uploadFile 对齐）：assets/{ns}/{type}/ 上传 →
    /// URL 数组 + 落盘位置；type 缺省 images / 自定义 type；无 file 字段报请上传文件；
    /// 非法 type 报文件类型错误；/reader3/file/upload 书仓语义不受影响
    #[tokio::test]
    async fn test_upload_user_file_assets_semantics() {
        let (state, dir) = test_state("upuserfile").await;
        let boundary = "----reader-upload-user-file-boundary";

        // 构造 multipart 请求体 → Multipart 提取器
        async fn extract_multipart(boundary: &str, body: String) -> axum::extract::Multipart {
            let req = axum::http::Request::builder()
                .method("POST")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(axum::body::Body::from(body))
                .unwrap();
            use axum::extract::FromRequest;
            axum::extract::Multipart::from_request(req, &())
                .await
                .unwrap()
        }

        // ① 缺省 type=images：两个 file 字段（含子目录名收敛）+ 一个无 filename 表单字段
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"sub/pic one.png\"\r\n\
             Content-Type: image/png\r\n\r\nPNGBYTES1\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"note.txt\"\r\n\
             Content-Type: text/plain\r\n\r\nTXTBYTES2\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"unrelated\"\r\n\r\nignored\r\n\
             --{boundary}--\r\n"
        );
        let multipart = extract_multipart(boundary, body).await;
        let ret = upload_user_file(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(ret.0.is_success, "上传应成功: {}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 2, "仅 file 字段计入: {arr:?}");
        assert_eq!(arr[0], json!("/assets/default/images/pic one.png"));
        assert_eq!(arr[1], json!("/assets/default/images/note.txt"));
        let assets = state.storage.config.storage_dir().join("assets/default");
        let pic = assets.join("images/pic one.png");
        let note = assets.join("images/note.txt");
        assert_eq!(std::fs::read(&pic).unwrap(), b"PNGBYTES1", "落盘内容一致");
        assert_eq!(std::fs::read(&note).unwrap(), b"TXTBYTES2");

        // ② 自定义 type=covers：独立子目录
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"cover.jpg\"\r\n\
             Content-Type: image/jpeg\r\n\r\nJPGDATA\r\n--{boundary}--\r\n"
        );
        let multipart = extract_multipart(boundary, body).await;
        let params: HashMap<String, String> =
            [("type".into(), "covers".into())].into_iter().collect();
        let ret = upload_user_file(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data[0], json!("/assets/default/covers/cover.jpg"));
        assert_eq!(
            std::fs::read(assets.join("covers/cover.jpg")).unwrap(),
            b"JPGDATA"
        );

        // ③ 无任何 file 字段 → 请上传文件
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"foo\"\r\n\r\nbar\r\n--{boundary}--\r\n"
        );
        let multipart = extract_multipart(boundary, body).await;
        let ret = upload_user_file(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "请上传文件");

        // ④ 非法 type（..）→ 文件类型错误
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"x.txt\"\r\n\r\ndata\r\n--{boundary}--\r\n"
        );
        let multipart = extract_multipart(boundary, body).await;
        let params: HashMap<String, String> = [("type".into(), "..".into())].into_iter().collect();
        let ret = upload_user_file(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "文件类型错误");

        // ⑤ 同名异义隔离：/reader3/file/upload 仍为书仓 entry 列表语义（files::upload 未动）
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"book.txt\"\r\n\
             Content-Type: text/plain\r\n\r\nSHELVED\r\n--{boundary}--\r\n"
        );
        let multipart = extract_multipart(boundary, body).await;
        let ret = crate::api::files::upload(
            State(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(ret.0.is_success, "file/upload 应成功: {}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "book.txt", "书仓语义返回 entry 对象");
        assert!(
            state
                .storage
                .config
                .storage_dir()
                .join("data/default/book.txt")
                .exists(),
            "file/upload 落 data/default/"
        );

        cleanup(state, dir).await;
    }

    /// 测试用最小备份 zip 构造（条目名 → 内容）
    fn make_test_zip(entries: &[(&str, &str)]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(&mut buf);
        for (name, content) in entries {
            writer
                .start_file(*name, zip::write::FileOptions::default())
                .unwrap();
            writer.write_all(content.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
        drop(writer);
        buf.into_inner()
    }

    /// F-55：restoreFromZip（multipart file=zip + overwrite 字段）——handler 全链路
    #[tokio::test]
    async fn test_restore_from_zip_api() {
        let (state, dir) = test_state("restorezip").await;
        // 预置一条书源 + 一本书（overwrite=false 恢复时应跳过）
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "https://s1.com".into(),
                    book_source_name: "旧源".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: "https://b1.com".into(),
                    name: "旧书".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let zip = make_test_zip(&[
            (
                "bookSource.json",
                r#"[{"bookSourceUrl":"https://s1.com","bookSourceName":"新源"},{"bookSourceUrl":"https://s2.com","bookSourceName":"源2"}]"#,
            ),
            (
                "bookshelf.json",
                r#"[{"bookUrl":"https://b1.com","name":"书1"}]"#,
            ),
        ]);

        let boundary = "----reader-restore-boundary-9f8e7d6c";
        // ① overwrite 缺省（false）：已存在跳过、新增恢复
        let mut mp: Vec<u8> = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"backup.zip\"\r\nContent-Type: application/zip\r\n\r\n"
        )
        .into_bytes();
        mp.extend_from_slice(&zip);
        mp.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(mp))
            .unwrap();
        use axum::extract::FromRequest;
        let multipart = axum::extract::Multipart::from_request(req, &())
            .await
            .unwrap();
        let ret = restore_from_zip(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["restored"]["sources"], 1, "仅 s2 新增");
        assert_eq!(ret.0.data["skipped"]["sources"], 1, "s1 已存在跳过");
        assert_eq!(ret.0.data["skipped"]["books"], 1, "b1 已存在跳过");
        // 未覆盖：s1 仍是旧源
        let src = state
            .storage
            .find_book_source("default", "https://s1.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(src.book_source_name, "旧源");

        // ② overwrite=true（表单字段）→ 全部恢复/覆盖
        let mut mp2: Vec<u8> = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"backup.zip\"\r\nContent-Type: application/zip\r\n\r\n"
        )
        .into_bytes();
        mp2.extend_from_slice(&zip);
        mp2.extend_from_slice(
            format!(
                "\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"overwrite\"\r\n\r\ntrue\r\n--{boundary}--\r\n"
            )
            .as_bytes(),
        );
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(mp2))
            .unwrap();
        let multipart = axum::extract::Multipart::from_request(req, &())
            .await
            .unwrap();
        let ret = restore_from_zip(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["restored"]["sources"], 2);
        assert_eq!(ret.0.data["restored"]["books"], 1);
        assert_eq!(ret.0.data["skipped"]["sources"], 0);
        // 覆盖生效：s1 被“新源”覆盖；书入架（namespace=default）
        let src = state
            .storage
            .find_book_source("default", "https://s1.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(src.book_source_name, "新源");
        let book = state
            .storage
            .find_book("default", "https://b1.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(book.name, "书1");
        assert_eq!(book.user_namespace, "default");

        // 无 file 字段 → 参数错误
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(format!("--{boundary}--\r\n")))
            .unwrap();
        let multipart = axum::extract::Multipart::from_request(req, &())
            .await
            .unwrap();
        let ret = restore_from_zip(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "请上传备份文件");

        // 非法 zip → 恢复失败
        let mut mp3 = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"bad.zip\"\r\nContent-Type: application/zip\r\n\r\n"
        );
        mp3.push_str("not a zip file");
        mp3.push_str(&format!("\r\n--{boundary}--\r\n"));
        let req = axum::http::Request::builder()
            .method("POST")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(axum::body::Body::from(mp3))
            .unwrap();
        let multipart = axum::extract::Multipart::from_request(req, &())
            .await
            .unwrap();
        let ret = restore_from_zip(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            multipart,
        )
        .await;
        assert!(!ret.0.is_success);
        assert!(ret.0.error_msg.contains("恢复失败"));

        cleanup(state, dir).await;
    }

    /// F-55：restoreFromWebdav（body {path, overwrite}）——zip 放 webdav 目录 → 恢复；
    /// 路径不存在/穿越拒绝；secure 未开启 webdav 拒绝
    #[tokio::test]
    async fn test_restore_from_webdav_api() {
        let (mut state, dir) = test_state("restoredav").await;
        let zip = make_test_zip(&[
            (
                "bookSource.json",
                r#"[{"bookSourceUrl":"https://w1.com","bookSourceName":"网源"}]"#,
            ),
            (
                "bookshelf.json",
                r#"[{"bookUrl":"https://wb1.com","name":"网书"}]"#,
            ),
        ]);
        let webdav = state
            .storage
            .config
            .storage_dir()
            .join("data/default/webdav/legado");
        std::fs::create_dir_all(&webdav).unwrap();
        std::fs::write(webdav.join("backup-test.zip"), &zip).unwrap();

        // 恢复成功 + overwrite 参数（body JSON）
        let body =
            Bytes::from(json!({ "path": "legado/backup-test.zip", "overwrite": true }).to_string());
        let ret = restore_from_webdav(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["restored"]["sources"], 1);
        assert_eq!(ret.0.data["restored"]["books"], 1);
        assert_eq!(ret.0.data["restored"]["groups"], 0);
        assert_eq!(
            state.storage.get_book_sources("default").await.unwrap()[0].book_source_name,
            "网源"
        );

        // 路径不存在
        let body = Bytes::from(json!({ "path": "legado/ghost.zip" }).to_string());
        let ret = restore_from_webdav(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "路径不存在");

        // 路径穿越 → 参数错误
        let body = Bytes::from(json!({ "path": "../secret.zip" }).to_string());
        let ret = restore_from_webdav(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "参数错误");

        // 非 zip 文件 → 恢复失败
        std::fs::write(webdav.join("bad.zip"), b"not a zip").unwrap();
        let body = Bytes::from(json!({ "path": "legado/bad.zip" }).to_string());
        let ret = restore_from_webdav(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert!(ret.0.error_msg.contains("恢复失败"));

        // secure：未开启 webdav → 拒绝
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                enable_webdav: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [("accessToken".into(), "alice:t1".into())]
            .into_iter()
            .collect();
        let body = Bytes::from(json!({ "path": "legado/backup-test.zip" }).to_string());
        let ret = restore_from_webdav(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "未开启webdav功能");

        cleanup(state, dir).await;
    }

    /// GAP #58：secure 模式下 enable_book_source=0 → 搜索/探索/换源拒绝（书源功能未开启）
    #[tokio::test]
    async fn test_permission_book_source_gate() {
        let (state, dir) = test_state("permbook").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                enable_book_source: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [("accessToken".into(), "alice:t1".into())]
            .into_iter()
            .collect();

        // 搜索（单源/多源/SSE）→ 拒绝
        let mut p = params.clone();
        p.insert("key".into(), "测试".into());
        let ret = search_book(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "书源功能未开启");
        let ret = search_book_multi(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源功能未开启");
        let resp = search_book_multi_sse(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let sse = String::from_utf8_lossy(&bytes).to_string();
        assert!(
            sse.contains("书源功能未开启"),
            "SSE 应输出 error 事件: {sse}"
        );

        // 换源 → 拒绝
        let mut p2 = params.clone();
        p2.insert("url".into(), "https://a.com/b".into());
        p2.insert("bookSource".into(), "https://s.com".into());
        let ret = search_book_source(
            AxumState(state.clone()),
            Query(p2.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源功能未开启");
        let resp = search_book_source_sse(
            AxumState(state.clone()),
            Query(p2.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("书源功能未开启"));

        // 探索 → 拒绝
        let mut p3 = params.clone();
        p3.insert("url".into(), "https://a.com/list".into());
        p3.insert("bookSource".into(), "https://s.com".into());
        let ret = explore_book(
            AxumState(state.clone()),
            Query(p3.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源功能未开启");
        let ret = get_explore_sources(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源功能未开启");
        let ret = get_explore_urls(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "书源功能未开启");

        // 开启后放行（无书源 → 走正常业务错误，说明权限已过）
        state
            .storage
            .update_user_permissions("alice", None, None, Some(true), None, None, None, None)
            .await
            .unwrap();
        let ret = search_book(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "未配置书源");

        // 非 secure 模式不拦截
        state.storage.config.secure = false;
        let ret = search_book(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "未配置书源");
        cleanup(state, dir).await;
    }

    /// GAP #58：secure 模式下 enable_rss_source=0 → RSS 接口拒绝（RSS功能未开启）
    #[tokio::test]
    async fn test_permission_rss_gate() {
        let (state, dir) = test_state("permrss").await;
        let mut state = state;
        state.storage.config.secure = true;
        state.storage.config.secure_key = "sk".into();
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "t1".into(),
                enable_rss_source: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let params: HashMap<String, String> = [("accessToken".into(), "alice:t1".into())]
            .into_iter()
            .collect();

        // 列表/保存/删除/文章/已读 → 全部拒绝
        let ret = get_rss_sources(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");
        let body = Bytes::from(r#"{"sourceUrl":"https://r.com/f.xml","sourceName":"R"}"#);
        let ret = save_rss_source(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");
        let mut p = params.clone();
        p.insert("rssSourceUrl".into(), "https://r.com/f.xml".into());
        let ret = delete_rss_source(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");
        let ret = get_rss_articles(
            AxumState(state.clone()),
            Query(p.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");
        let mut p2 = params.clone();
        p2.insert("articleUrl".into(), "https://r.com/a1".into());
        let ret = mark_rss_article_read(
            AxumState(state.clone()),
            Query(p2.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");
        let mut p3 = params.clone();
        p3.insert("url".into(), "https://r.com/a1".into());
        let ret = get_rss_article(
            AxumState(state.clone()),
            Query(p3.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");
        let body = Bytes::from(r#"[{"sourceUrl":"https://r.com/f.xml","sourceName":"R"}]"#);
        let ret = save_rss_sources(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "RSS功能未开启");

        // 开启后放行
        state
            .storage
            .update_user_permissions("alice", None, None, None, Some(true), None, None, None)
            .await
            .unwrap();
        let ret = get_rss_sources(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "开启后应放行: {}", ret.0.error_msg);
        cleanup(state, dir).await;
    }

    /// GAP #51：exploreBook 分页——page 服务端替换 {{page}}；响应 = SearchBook 纯数组
    /// （对齐 legacy BookController.exploreBook：webBook.exploreBook 列表原样 setData）
    #[tokio::test]
    async fn test_explore_book_pagination_response() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("explorepg").await;
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let cap = captured.clone();
        // 25 本书的 JSON 列表（满页 → hasMore=true）
        let mut books: Vec<serde_json::Value> = Vec::new();
        for i in 0..25 {
            books.push(serde_json::json!({
                "name": format!("书{i}"), "author": "作者", "url": format!("https://a.com/b{i}")
            }));
        }
        let body = serde_json::json!({ "data": books }).to_string();
        let body_for_server = body.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            for _ in 0..5 {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                cap.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buf[..n]).to_string());
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body_for_server.len(),
                    body_for_server
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        let base = format!("http://{addr}");
        let explore_url = format!("{base}/list/{{{{page}}}}");
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "探索源".into(),
                    enabled_explore: true,
                    explore_url: Some(explore_url.clone()),
                    rule_explore: Some(serde_json::json!({
                        "bookList": "$.data[*]",
                        "name": "$.name",
                        "author": "$.author",
                        "bookUrl": "$.url",
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // page=3 → 请求路径应含 /list/3（服务端分页变量替换）
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), explore_url.clone());
        params.insert("bookSource".into(), base.clone());
        params.insert("page".into(), "3".into());
        let ret = explore_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let data = ret.0.data;
        // legacy 契约：data 即 SearchBook 纯数组（无 {books, hasMore} 包装）
        assert!(data.is_array(), "exploreBook 应返回纯数组: {data}");
        let arr = data.as_array().unwrap();
        assert_eq!(arr.len(), 25);
        assert_eq!(arr[0]["name"], "书0");
        assert_eq!(arr[0]["origin"], base);
        let req = captured.lock().unwrap()[0].clone();
        assert!(
            req.contains("GET /list/3 "),
            "page 应由服务端替换进 URL: {req}"
        );

        // 空页（返回空数组）→ data = []
        let body2 = "{\"data\":[]}";
        let listener2 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr2 = listener2.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let Ok((mut sock, _)) = listener2.accept().await else {
                return;
            };
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body2.len(),
                body2
            );
            let _ = sock.write_all(resp.as_bytes()).await;
        });
        let base2 = format!("http://{addr2}");
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base2.clone(),
                    book_source_name: "空源".into(),
                    enabled_explore: true,
                    explore_url: Some(format!("{base2}/list/{{{{page}}}}")),
                    rule_explore: Some(serde_json::json!({
                        "bookList": "$.data[*]",
                        "name": "$.name",
                        "bookUrl": "$.url",
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), format!("{base2}/list/{{{{page}}}}"));
        params.insert("bookSource".into(), base2);
        params.insert("page".into(), "2".into());
        let ret = explore_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        // 空页 → 空数组（legacy：列表原样返回）
        assert_eq!(
            ret.0.data.as_array().map(|a| a.len()),
            Some(0),
            "空页应返回空数组: {}",
            ret.0.data
        );
        cleanup(state, dir).await;
    }

    /// GAP 51 边界：exploreBook page 参数——POST body 传 page / 缺省 1 / 0 与负数原样透传
    #[tokio::test]
    async fn test_explore_book_page_param_boundaries() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("explorepgb").await;
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let cap = captured.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            for _ in 0..3 {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                cap.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buf[..n]).to_string());
                let body = "{\"data\":[]}";
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        let base = format!("http://{addr}");
        let explore_url = format!("{base}/list/{{{{page}}}}");
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "边界源".into(),
                    enabled_explore: true,
                    explore_url: Some(explore_url.clone()),
                    rule_explore: Some(serde_json::json!({
                        "bookList": "$.data[*]",
                        "name": "$.name",
                        "bookUrl": "$.url",
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let call = |params: HashMap<String, String>, body: Option<Bytes>| async {
            explore_book(
                AxumState(state.clone()),
                Query(params),
                HeaderMap::new(),
                body,
            )
            .await
        };

        // POST body 传 page=5 → 请求 /list/5（body 优先于缺省）
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), explore_url.clone());
        params.insert("bookSource".into(), base.clone());
        let ret = call(params, Some(Bytes::from(r#"{"page":5}"#))).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let req = captured.lock().unwrap()[0].clone();
        assert!(req.contains("GET /list/5 "), "POST body page 应生效: {req}");

        // 缺 page → 默认 1
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), explore_url.clone());
        params.insert("bookSource".into(), base.clone());
        let ret = call(params, Some(Bytes::from(r#"{}"#))).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let req = captured.lock().unwrap()[1].clone();
        assert!(req.contains("GET /list/1 "), "缺省 page 应为 1: {req}");

        // page=0（0 基分页书源）→ 原样透传
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), explore_url);
        params.insert("bookSource".into(), base);
        params.insert("page".into(), "0".into());
        let ret = call(params, None).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let req = captured.lock().unwrap()[2].clone();
        assert!(req.contains("GET /list/0 "), "page=0 应原样透传: {req}");

        cleanup(state, dir).await;
    }

    /// GAP #88/125：/assets/proxy 图片代理端点
    #[tokio::test]
    async fn test_assets_proxy_endpoint() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1
        let (state, dir) = test_state("proxy").await;
        // mock 图片服务器（记录请求头）
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let cap = captured.clone();
        let png: Vec<u8> = vec![0x89, b'P', b'N', b'G', 9, 9, 9];
        let png_for_server = png.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf).await;
            cap.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buf).to_string());
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                png_for_server.len()
            );
            let mut resp = head.into_bytes();
            resp.extend_from_slice(&png_for_server);
            let _ = sock.write_all(&resp).await;
        });
        let img_url = format!("http://{addr}/cover/1.png");
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), img_url.clone());
        params.insert("referer".into(), "https://src.com/book".into());
        let resp = assets_proxy(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("image/png"),
            "Content-Type 透传"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        assert_eq!(bytes.to_vec(), png, "图片字节透传");
        let req = captured.lock().unwrap()[0].clone();
        assert!(req.contains("GET /cover/1.png"), "{req}");
        assert!(
            req.to_lowercase().contains("referer: https://src.com/book"),
            "Referer 应透传给上游: {req}"
        );

        // 参数错误（缺 url / 非法 scheme）
        let resp = assets_proxy(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("参数错误"));
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), "file:///etc/passwd".into());
        let resp = assets_proxy(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("参数错误"));
        cleanup(state, dir).await;
    }

    /// M1：/assets/proxy 拒绝私网/回环目标（SSRF）——非 secure 模式（test_state 默认
    /// 非 secure）同样拦截；回环 mock 服务器收不到任何请求
    #[tokio::test]
    async fn test_assets_proxy_rejects_private_url() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(false); // 持锁：确保拦截态
        let (state, dir) = test_state("proxy-ssrf").await;
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(
            "url".into(),
            "http://127.0.0.1:8085/reader3/getSystemInfo".into(),
        );
        let resp = assets_proxy(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "错误信息以 200 + JSON 返回（legacy 契约）"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let msg = String::from_utf8_lossy(&bytes);
        assert!(msg.contains("图片加载失败"), "应返回加载失败: {msg}");
        assert!(msg.contains("已拦截"), "应提示私网拦截: {msg}");
        // 私网字面量（云元数据 169.254.169.254）
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(
            "url".into(),
            "http://169.254.169.254/latest/meta-data/".into(),
        );
        let resp = assets_proxy(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("已拦截"));
        cleanup(state, dir).await;
    }

    /// GAP：/assets/proxy 磁盘缓存——首次回源写盘（短缓存），二次磁盘命中
    /// （长 Cache-Control，不再回源）；缓存文件落在 storage/cache/images/{md5(ns|url)}.png
    #[tokio::test]
    async fn test_assets_proxy_disk_cache() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1
        let (state, dir) = test_state("proxy-cache").await;
        // mock 图片服务器（每请求计数）
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cnt = count.clone();
        let png: Vec<u8> = vec![0x89, b'P', b'N', b'G', 9, 9, 9];
        let body = png.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let body = body.clone();
                let cnt = cnt.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let _ = sock.read(&mut buf).await;
                    cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let mut resp = head.into_bytes();
                    resp.extend_from_slice(&body);
                    let _ = sock.write_all(&resp).await;
                });
            }
        });
        let img_url = format!("http://{addr}/c.png");
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), img_url.clone());

        // 首次：回源 + 短缓存
        let resp = assets_proxy(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Cache-Control")
                .and_then(|v| v.to_str().ok()),
            Some("public, max-age=3600"),
            "首次回源应为短缓存"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        assert_eq!(bytes.to_vec(), png, "图片字节透传");

        // 缓存文件落盘：storage/cache/images/{md5("default|url")}.png（M2：键含命名空间）
        let key = crate::util::md5::md5_encode(&format!("default|{img_url}"));
        assert_eq!(key.len(), 32);
        let cache_file = dir
            .join("storage")
            .join("cache")
            .join("images")
            .join(format!("{key}.png"));
        assert!(cache_file.exists(), "首次拉取应写缓存文件: {cache_file:?}");

        // 二次：磁盘命中 + 长缓存 + 不再回源
        let resp = assets_proxy(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Cache-Control")
                .and_then(|v| v.to_str().ok()),
            Some("public, max-age=31536000, immutable"),
            "磁盘命中应下发长 Cache-Control"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        assert_eq!(bytes.to_vec(), png);
        assert_eq!(
            count.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "二次请求不应回源"
        );
        cleanup(state, dir).await;
    }

    // ==================== 收尾批（P3）：上传限制 / EPUB 封面 / 失效禁用 / 迁移 / 内置探索 ====================

    /// GAP 62：READER_UPLOAD_MAX_MB 配置解析——默认 100MB，env 覆盖，上传字节数换算正确
    #[test]
    fn test_upload_max_mb_config() {
        let old = std::env::var("READER_UPLOAD_MAX_MB").ok();
        std::env::remove_var("READER_UPLOAD_MAX_MB");
        let cfg = crate::AppConfig::from_env();
        assert_eq!(cfg.upload_max_mb, 100, "默认 100MB");
        assert_eq!(cfg.upload_max_bytes(), 100 * 1024 * 1024);
        std::env::set_var("READER_UPLOAD_MAX_MB", "50");
        let cfg = crate::AppConfig::from_env();
        assert_eq!(cfg.upload_max_mb, 50);
        assert_eq!(cfg.upload_max_bytes(), 50 * 1024 * 1024);
        // 非法值回退默认
        std::env::set_var("READER_UPLOAD_MAX_MB", "abc");
        let cfg = crate::AppConfig::from_env();
        assert_eq!(cfg.upload_max_mb, 100);
        match old {
            Some(v) => std::env::set_var("READER_UPLOAD_MAX_MB", v),
            None => std::env::remove_var("READER_UPLOAD_MAX_MB"),
        }
    }

    /// GAP 62：端到端——multipart 超限（上限 1MB）→ 明确 JSON 错误（Content-Length 预检）
    #[tokio::test]
    async fn test_upload_limit_413_end_to_end() {
        use tower::ServiceExt as _;
        let (mut state, dir) = test_state("uplimit").await;
        state.storage.config.upload_max_mb = 1; // 1MB 上限
                                                // 小上限路由（1MB）：直接构造路由片段验证 DefaultBodyLimit + 明确错误 + 413 改写层
        let app = axum::Router::new()
            .route(
                "/reader3/uploadLocalBook",
                post(upload_local_book).layer(axum::extract::DefaultBodyLimit::max(
                    state.storage.config.upload_max_bytes(),
                )),
            )
            .with_state(state.clone())
            .layer(crate::middleware::upload_limit::UploadLimitLayer { max_mb: 1 });
        let boundary = "----reader-uplimit-boundary";
        // 构造 2MB multipart 体（超过 1MB 上限）
        let mut mp: Vec<u8> = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"big.txt\"\r\nContent-Type: text/plain\r\n\r\n"
        )
        .into_bytes();
        mp.extend(std::iter::repeat_n(b'x', 2 * 1024 * 1024));
        mp.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/reader3/uploadLocalBook")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .header("content-length", mp.len().to_string())
                    .body(axum::body::Body::from(mp))
                    .unwrap(),
            )
            .await
            .unwrap();
        // axum Multipart 对 DefaultBodyLimit 超限表现为流错误（非 413）——handler 层
        // 显式字段上限给出明确错误（GAP 62 主路径）；413 改写由 UploadLimitLayer 覆盖
        // Bytes/Json 类提取器（middleware::upload_limit 单测已覆盖）
        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap(), "超限应失败: {json}");
        assert!(
            json["errorMsg"]
                .as_str()
                .unwrap()
                .contains("超过上传大小上限"),
            "应为明确错误: {json}"
        );
        cleanup(state, dir).await;
    }

    /// GAP 111：exportBook epub 含封面——OPF manifest properties=cover-image +
    /// <meta name="cover"> + OEBPS/cover.jpg 图片条目（本地封面文件直读）
    #[tokio::test]
    async fn test_export_book_epub_cover_api() {
        use std::io::Read as _;
        let (state, dir) = test_state("epubcover").await;
        // 本地书（local://）入架 + 章节 + 本地封面文件
        let book_url = "local://epub-cover-test";
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.into(),
                    name: "封面书".into(),
                    author: "作者X".into(),
                    origin: "local".into(),
                    cover_url: Some("/assets/default/covers/c1.jpg".into()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .cache_chapter_content("default", book_url, 0, "第一章", "正文内容。")
            .await
            .unwrap();
        let cover_dir = state
            .storage
            .config
            .storage_dir()
            .join("assets/default/covers");
        std::fs::create_dir_all(&cover_dir).unwrap();
        // 最小合法 JPEG 头（封面检测：非 PNG 前缀即按 jpg 处理）
        std::fs::write(
            cover_dir.join("c1.jpg"),
            [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46],
        )
        .unwrap();

        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), book_url.into());
        params.insert("format".into(), "epub".into());
        let resp = export_book(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("epub 应合法");
        let mut opf = String::new();
        zip.by_name("OEBPS/content.opf")
            .unwrap()
            .read_to_string(&mut opf)
            .unwrap();
        assert!(
            opf.contains("properties=\"cover-image\""),
            "manifest 封面声明: {opf}"
        );
        assert!(opf.contains("<meta name=\"cover\" content=\"cover-image\"/>"));
        assert!(
            opf.contains("href=\"cover.jpg\""),
            "manifest 条目应指向 OEBPS/cover.jpg: {opf}"
        );
        let mut cover = Vec::new();
        zip.by_name("OEBPS/cover.jpg")
            .unwrap()
            .read_to_end(&mut cover)
            .unwrap();
        assert_eq!(
            &cover[..4],
            &[0xFF, 0xD8, 0xFF, 0xE0],
            "封面图片条目内容一致"
        );
        // 章节仍在
        let mut ch = String::new();
        zip.by_name("OEBPS/chap_0000.xhtml")
            .unwrap()
            .read_to_string(&mut ch)
            .unwrap();
        assert!(ch.contains("正文内容。"));
        cleanup(state, dir).await;
    }

    /// P0-7：exportBook 文件型本地书越权——is_file 分支必须书架归属：
    /// 未入架文件拒绝（含跨用户文件）；本人书架文件可导出
    #[tokio::test]
    async fn test_export_book_file_authz_api() {
        let (mut state, dir) = test_state("exportauthz").await;
        let storage_dir = state.storage.config.storage_dir();

        // 本人书架文件书：可导出
        let book_dir = storage_dir.join("data/default/books");
        std::fs::create_dir_all(&book_dir).unwrap();
        let book_url = "storage/data/default/books/我的书.txt";
        std::fs::write(
            storage_dir.join("data/default/books/我的书.txt"),
            "第一章 开始\n正文甲。\n第二章 结束\n正文乙。\n",
        )
        .unwrap();
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.into(),
                    name: "我的书".into(),
                    author: "作者甲".into(),
                    origin: "loc_book".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), book_url.into());
        params.insert("format".into(), "txt".into());
        let resp = export_book(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let txt = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(txt.contains("正文甲。"), "书架文件应导出成功: {txt}");

        // 文件存在但未入架 → 拒绝（P0-7：url 直传不再放行）
        std::fs::write(storage_dir.join("data/default/books/未入架.txt"), "秘密\n").unwrap();
        let mut p2 = params.clone();
        p2.insert("url".into(), "storage/data/default/books/未入架.txt".into());
        let resp = export_book(AxumState(state.clone()), Query(p2), HeaderMap::new(), None).await;
        let json: Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap());
        assert_eq!(json["errorMsg"], "书籍不存在（请先加入书架）");

        // secure + 跨用户：bob 目录内文件，alice 导出 → 拒绝
        state.storage.config.secure = true;
        state
            .storage
            .insert_user(&User {
                username: "alice".into(),
                token: "tka".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        state
            .storage
            .insert_user(&User {
                username: "bob".into(),
                token: "tkb".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let bob_dir = storage_dir.join("data/bob");
        std::fs::create_dir_all(&bob_dir).unwrap();
        std::fs::write(bob_dir.join("bob-secret.txt"), "bob机密\n").unwrap();
        // secure 模式：alice 身份 + url 参数
        let alice_req = |url: &str| -> HashMap<String, String> {
            [
                ("accessToken".into(), "alice:tka".into()),
                ("url".into(), url.into()),
                ("format".into(), "txt".into()),
            ]
            .into_iter()
            .collect()
        };
        let resp = export_book(
            AxumState(state.clone()),
            Query(alice_req("storage/data/bob/bob-secret.txt")),
            HeaderMap::new(),
            None,
        )
        .await;
        let json: Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(!json["isSuccess"].as_bool().unwrap(), "跨用户导出应拒绝");
        assert_eq!(json["errorMsg"], "书籍不存在（请先加入书架）");

        // 穿越路径（.. 越出 storage 指向外部文件）→ 拒绝
        let outside = storage_dir.parent().unwrap().join("outside-secret.txt");
        std::fs::write(&outside, "外部机密\n").unwrap();
        let resp = export_book(
            AxumState(state.clone()),
            Query(alice_req("storage/../outside-secret.txt")),
            HeaderMap::new(),
            None,
        )
        .await;
        let json: Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(
            !json["isSuccess"].as_bool().unwrap(),
            "穿越路径应拒绝: {}",
            json["errorMsg"]
        );
        assert_eq!(json["errorMsg"], "书籍不存在（请先加入书架）");

        cleanup(state, dir).await;
    }

    /// P0-7：resolve_loc_book_file canonicalize + containment——正常定位放行、
    /// .. 越出/绝对路径/符号链接逃逸拒绝
    #[test]
    fn test_resolve_loc_book_file_containment() {
        let base = std::env::temp_dir().join("reader-locbook-resolve");
        let _ = std::fs::remove_dir_all(&base);
        let storage_dir = base.join("storage");
        std::fs::create_dir_all(&storage_dir.join("data/alice")).unwrap();
        std::fs::write(storage_dir.join("data/alice/ok.txt"), "ok").unwrap();
        // legacy 目录式 epub（{书}.epub/ 内含 index.epub）
        std::fs::create_dir_all(&storage_dir.join("data/alice/书.epub")).unwrap();
        std::fs::write(storage_dir.join("data/alice/书.epub/index.epub"), "legacy").unwrap();
        let escape = base.join("escape.txt");
        std::fs::write(&escape, "escape").unwrap();
        let storage_dir = storage_dir.canonicalize().unwrap();

        // 正常：直接文件 + legacy 目录式 index.epub 兜底
        let p = resolve_loc_book_file(&storage_dir, "storage/data/alice/ok.txt").unwrap();
        assert_eq!(p, storage_dir.join("data/alice/ok.txt"));
        let p = resolve_loc_book_file(&storage_dir, "storage/data/alice/书.epub").unwrap();
        assert_eq!(p, storage_dir.join("data/alice/书.epub/index.epub"));

        // .. 越出 storage → 拒绝
        assert!(resolve_loc_book_file(&storage_dir, "storage/../escape.txt").is_none());
        assert!(resolve_loc_book_file(&storage_dir, "storage/a/../../escape.txt").is_none());
        // 绝对路径（join 替换 base）→ 拒绝
        assert!(
            resolve_loc_book_file(&storage_dir, escape.to_string_lossy().as_ref()).is_none(),
            "绝对路径应拒绝"
        );
        // 符号链接逃逸（unix）：storage 内链接指向外部文件 → canonicalize 后拒绝
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&escape, storage_dir.join("data/alice/link.txt")).unwrap();
            assert!(
                resolve_loc_book_file(&storage_dir, "storage/data/alice/link.txt").is_none(),
                "符号链接逃逸应拒绝"
            );
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    /// GAP 140：disableInvalidBookSources——坏源批量禁用、好源保留、返回 count/disabled
    #[tokio::test]
    async fn test_disable_invalid_book_sources_api() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("disinv").await;
        let good_url = serve_head_get().await;
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: good_url.clone(),
                    book_source_name: "好源".into(),
                    enabled: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "http://127.0.0.1:1".into(),
                    book_source_name: "坏源".into(),
                    enabled: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        // 已禁用的不参与（也不重复禁用）
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: "http://127.0.0.1:2".into(),
                    book_source_name: "停用源".into(),
                    enabled: false,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let ret = disable_invalid_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["count"], 1, "仅坏源被禁用: {}", ret.0.data);
        assert_eq!(ret.0.data["disabled"][0], "http://127.0.0.1:1");
        let sources = state.storage.get_book_sources("default").await.unwrap();
        let bad = sources
            .iter()
            .find(|s| s.book_source_url == "http://127.0.0.1:1")
            .unwrap();
        assert!(!bad.enabled, "坏源应被禁用");
        let good = sources
            .iter()
            .find(|s| s.book_source_url == good_url)
            .unwrap();
        assert!(good.enabled, "好源应保留启用");

        // 再次执行：已禁用的不再重复计数
        let ret = disable_invalid_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.data["count"], 0);
        cleanup(state, dir).await;
    }

    /// GAP 141：内置探索源导入测试库 → getExploreSources 返回（categoryCount>0，可探索）
    #[tokio::test]
    async fn test_builtin_explore_sources_import_api() {
        let (state, dir) = test_state("builtexplore").await;
        let builtin = crate::service::explore::builtin_explore_sources();
        assert!(builtin.len() >= 2);
        for s in &builtin {
            state.storage.save_book_source("default", s).await.unwrap();
        }
        let ret = get_explore_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(
            arr.len(),
            builtin.len(),
            "内置探索源应全部进入探索列表: {arr:?}"
        );
        for item in arr {
            let count = item["categoryCount"].as_u64().unwrap_or(0);
            assert!(count >= 4, "探索分类数应 >= 4: {item}");
        }
        // 已导入书源可被 saveBookSources 幂等覆盖（raw_json 完整回吐）
        let ret = get_book_sources(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
        )
        .await;
        assert!(ret.0.is_success);
        assert_eq!(ret.0.data.as_array().unwrap().len(), builtin.len());
        cleanup(state, dir).await;
    }

    /// GAP 171：migrateLocBook——单本 + 批量；文件解析入章节表、local_file 关联、
    /// 原记录保留；迁移后 getBookToc/getBookContent 走 DB 直读（不再解析文件）
    #[tokio::test]
    async fn test_migrate_loc_book_api() {
        let (state, dir) = test_state("migloc").await;
        // 构造 legacy loc_book 文件书：storage/data/default/示例.txt（书架记录 origin=loc_book）
        let file_dir = state.storage.config.storage_dir().join("data/default");
        std::fs::create_dir_all(&file_dir).unwrap();
        let txt_path = file_dir.join("示例.txt");
        std::fs::write(&txt_path, "第一章 起点\n内容一。\n第二章 发展\n内容二。\n").unwrap();
        let book_url = format!(
            "storage/data/default/{}",
            txt_path.file_name().unwrap().to_string_lossy()
        );
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url.clone(),
                    name: "示例".into(),
                    origin: "loc_book".into(),
                    origin_name: "本地书".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // 单本迁移
        let body = Bytes::from(json!({ "bookUrl": book_url }).to_string());
        let ret = migrate_loc_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(ret.0.data["migrated"], 1, "{}", ret.0.data);
        assert_eq!(ret.0.data["skipped"].as_array().unwrap().len(), 0);
        // 章节入库 + local_file 关联 + 原记录保留（origin 不变）
        let chapters = state.storage.list_chapters(&book_url).await.unwrap();
        assert_eq!(chapters.len(), 2, "章节应写入 DB");
        assert_eq!(chapters[0].1, "第一章 起点");
        let book = state
            .storage
            .find_book("default", &book_url)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(book.origin, "loc_book", "原记录 origin 保留");
        assert!(
            book.local_file
                .as_deref()
                .unwrap_or("")
                .contains("示例.txt"),
            "local_file 应关联: {:?}",
            book.local_file
        );
        assert!(!book.local_file_deleted);
        // 正文内容可读（DB 直读——通过 get_book_content_file 路径）
        let content = state
            .storage
            .get_chapter_content("default", &book_url, 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(content, "内容二。");
        // getBookToc：storage/ 路径命中 DB 直读（不再解析文件）
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("tocUrl".into(), book_url.clone());
        let ret = get_book_toc(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let arr = ret.0.data.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["title"], "第一章 起点");
        assert_eq!(arr[0]["url"], format!("{book_url}#0"));

        // 重复迁移幂等（migrated=1，章节不翻倍）
        let body = Bytes::from(json!({ "bookUrl": book_url }).to_string());
        let ret = migrate_loc_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.data["migrated"], 1);
        assert_eq!(
            state.storage.list_chapters(&book_url).await.unwrap().len(),
            2
        );

        // 参数错误
        let ret = migrate_loc_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert_eq!(ret.0.error_msg, "参数错误");
        // 不存在的书
        let body = Bytes::from(r#"{"bookUrl":"storage/data/default/ghost.txt"}"#);
        let ret = migrate_loc_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert_eq!(ret.0.error_msg, "书籍不存在（请先加入书架）");

        // 批量（all）：第二本 loc_book 一并迁移；非 loc_book 不受影响
        let txt2 = file_dir.join("示例2.txt");
        std::fs::write(&txt2, "甲章\n甲内容。\n").unwrap();
        let book_url2 = format!(
            "storage/data/default/{}",
            txt2.file_name().unwrap().to_string_lossy()
        );
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: book_url2.clone(),
                    name: "示例2".into(),
                    origin: "loc_book".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let body = Bytes::from(r#"{"all":true}"#);
        let ret = migrate_loc_book(
            AxumState(state.clone()),
            Query(HashMap::new()),
            HeaderMap::new(),
            Some(body),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert_eq!(
            ret.0.data["migrated"], 2,
            "两本 loc_book 均迁移（第一本幂等重迁成功）: {}",
            ret.0.data
        );
        assert_eq!(
            state.storage.list_chapters(&book_url2).await.unwrap().len(),
            1
        );
        cleanup(state, dir).await;
    }

    /// 微型 HTTP 服务器（固定响应体）——getBookContent 分派测试用
    async fn mini_server(body: &'static str) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            for _ in 0..20 {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 4096];
                let _ = sock.read(&mut buf).await;
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let mut resp = head.into_bytes();
                resp.extend_from_slice(body.as_bytes());
                let _ = sock.write_all(&resp).await;
            }
        });
        format!("http://{addr}")
    }

    /// 非文本书 getBookContent 分派（构造 audio/comic/video/file 书源规则 → 断言返回结构）：
    /// 音频 → {audioUrl, contentType}；漫画 → {images:[...]}；视频 → {videoUrl}；
    /// 文件 → {downloadUrl}；文本书 → {content}（正文缓存路径不受影响）
    #[tokio::test]
    async fn test_get_book_content_non_text_dispatch() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1（P1 SSRF 校验放行，仅测试）
        let (state, dir) = test_state("nontext").await;
        let base = mini_server(
            r#"<html><div class="p">测试正文。<audio src="/a/1.mp3"></audio><img src="/i/1.jpg"><img src="/i/2.jpg"></div></html>"#,
        )
        .await;
        let mk = |url: &str, name: &str, ty: i64, rule: Option<serde_json::Value>| {
            crate::model::BookSource {
                book_source_url: url.into(),
                book_source_name: name.into(),
                book_source_type: ty,
                rule_content: rule,
                ..Default::default()
            }
        };
        let q = |chapter_url: &str, src: &str| {
            Query(HashMap::from([
                ("chapterUrl".to_string(), chapter_url.to_string()),
                ("bookSource".to_string(), src.to_string()),
            ]))
        };

        // 音频书（bookSourceType=1）：ruleContent 提取音频 URL → {audioUrl, contentType}
        let audio_src = mk(
            "https://audio.src",
            "音频源",
            1,
            Some(serde_json::json!({ "content": "div.p audio@src" })),
        );
        state
            .storage
            .save_book_source("default", &audio_src)
            .await
            .unwrap();
        let ret = get_book_content(
            AxumState(state.clone()),
            q(&format!("{base}/c/1"), "https://audio.src"),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "音频书应成功: {}", ret.0.error_msg);
        assert_eq!(
            ret.0.data["audioUrl"],
            serde_json::json!(format!("{base}/a/1.mp3"))
        );
        assert_eq!(ret.0.data["contentType"], serde_json::json!("audio/mpeg"));

        // 漫画书（bookSourceType=2）：CSS 规则多值 → {images:[...]}（绝对化）
        let comic_src = mk(
            "https://comic.src",
            "漫画源",
            2,
            Some(serde_json::json!({ "content": "div.p img@src" })),
        );
        state
            .storage
            .save_book_source("default", &comic_src)
            .await
            .unwrap();
        let ret = get_book_content(
            AxumState(state.clone()),
            q(&format!("{base}/c/2"), "https://comic.src"),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "漫画书应成功: {}", ret.0.error_msg);
        assert_eq!(
            ret.0.data["images"],
            serde_json::json!([format!("{base}/i/1.jpg"), format!("{base}/i/2.jpg")])
        );

        // 视频书（bookSourceType=4）：无 ruleContent → 章节 URL 直链 {videoUrl}
        let video_src = mk("https://video.src", "视频源", 4, None);
        state
            .storage
            .save_book_source("default", &video_src)
            .await
            .unwrap();
        let ret = get_book_content(
            AxumState(state.clone()),
            q(&format!("{base}/v/1.mp4"), "https://video.src"),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "视频书应成功: {}", ret.0.error_msg);
        assert_eq!(
            ret.0.data["videoUrl"],
            serde_json::json!(format!("{base}/v/1.mp4"))
        );

        // 文件书（bookSourceType=3）：{downloadUrl}
        let file_src = mk(
            "https://file.src",
            "文件源",
            3,
            Some(serde_json::json!({ "content": "div.p audio@src" })),
        );
        state
            .storage
            .save_book_source("default", &file_src)
            .await
            .unwrap();
        let ret = get_book_content(
            AxumState(state.clone()),
            q(&format!("{base}/c/4"), "https://file.src"),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "文件书应成功: {}", ret.0.error_msg);
        assert_eq!(
            ret.0.data["downloadUrl"],
            serde_json::json!(format!("{base}/a/1.mp3"))
        );

        // 文本书（bookSourceType=0）：原有 {content} 文本返回不受影响
        let text_src = mk(
            "https://text.src",
            "文本源",
            0,
            Some(serde_json::json!({ "content": "div.p@text" })),
        );
        state
            .storage
            .save_book_source("default", &text_src)
            .await
            .unwrap();
        let ret = get_book_content(
            AxumState(state.clone()),
            q(&format!("{base}/c/5"), "https://text.src"),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "文本书应成功: {}", ret.0.error_msg);
        assert!(ret.0.data["content"]
            .as_str()
            .unwrap_or("")
            .contains("测试正文"));

        // 漫画书：规则提取为空 → 失败提示（不返回空 images）
        let empty_src = mk(
            "https://comic-empty.src",
            "空漫画源",
            2,
            Some(serde_json::json!({ "content": "div.nothing img@src" })),
        );
        state
            .storage
            .save_book_source("default", &empty_src)
            .await
            .unwrap();
        let ret = get_book_content(
            AxumState(state.clone()),
            q(&format!("{base}/c/6"), "https://comic-empty.src"),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(!ret.0.is_success, "提取不到图片应报错");

        cleanup(state, dir).await;
    }

    /// 端到端：uploadLocalBook 自动分派 cbz/umd（multipart 上传 → 章节入库）
    #[tokio::test]
    async fn test_upload_local_book_cbz_umd_dispatch() {
        use tower::ServiceExt as _;
        let (state, dir) = test_state("upcbzumd").await;
        let app = axum::Router::new()
            .route("/reader3/uploadLocalBook", post(upload_local_book))
            .with_state(state.clone());

        // 构造 multipart 请求体
        let multipart = |file_name: &str, bytes: &[u8]| {
            let boundary = "----reader-upload-dispatch";
            let mut mp: Vec<u8> = format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
            )
            .into_bytes();
            mp.extend_from_slice(bytes);
            mp.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
            (mp, format!("multipart/form-data; boundary={boundary}"))
        };

        // ① CBZ：构造 3 页 zip（自然序：1.png < 2.jpg < 10.png）
        let png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x62, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        use std::io::Write as _;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::FileOptions::default();
            zip.start_file("10.png", opts).unwrap();
            zip.write_all(png).unwrap();
            zip.start_file("2.jpg", opts).unwrap();
            zip.write_all(b"jpeg").unwrap();
            zip.start_file("1.png", opts).unwrap();
            zip.write_all(png).unwrap();
            zip.finish().unwrap();
        }
        let (mp, ct) = multipart("漫画.cbz", &buf.into_inner());
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/reader3/uploadLocalBook")
                    .header("content-type", ct)
                    .body(axum::body::Body::from(mp))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 8 * 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            json["isSuccess"].as_bool().unwrap(),
            "CBZ 导入应成功: {json}"
        );
        let cbz_url = json["data"]["bookUrl"].as_str().unwrap().to_string();
        assert!(cbz_url.starts_with("local://"));
        assert_eq!(json["data"]["name"], "漫画", "书名用文件主名");
        let rows = state.storage.list_chapters(&cbz_url).await.unwrap();
        let titles: Vec<String> = rows.iter().map(|(_, t)| t.clone()).collect();
        assert_eq!(
            titles,
            vec!["1.png", "2.jpg", "10.png"],
            "图片页按自然序成章"
        );
        let content = state
            .storage
            .get_chapter_content("default", &cbz_url, 0)
            .await
            .unwrap()
            .unwrap();
        assert!(
            content.starts_with("![1.png](data:image/png;base64,"),
            "正文为图片标记"
        );

        // ② UMD：真实样本（样本缺失则跳过）
        let sample =
            "C:/Users/chong/pr-review/reader-dev/target/search-test/samples/明朝那些事儿.umd";
        let Ok(umd_bytes) = std::fs::read(sample) else {
            eprintln!("跳过 UMD 上传用例（样本缺失）");
            cleanup(state, dir).await;
            return;
        };
        let (mp, ct) = multipart("明朝那些事儿.umd", &umd_bytes);
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/reader3/uploadLocalBook")
                    .header("content-type", ct)
                    .body(axum::body::Body::from(mp))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 8 * 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            json["isSuccess"].as_bool().unwrap(),
            "UMD 导入应成功: {json}"
        );
        assert_eq!(
            json["data"]["name"], "明朝那些事儿（1-7全套）终极版",
            "书名取 UMD 元数据"
        );
        assert_eq!(json["data"]["author"], "当年明月");
        let umd_url = json["data"]["bookUrl"].as_str().unwrap().to_string();
        let rows = state.storage.list_chapters(&umd_url).await.unwrap();
        assert_eq!(rows.len(), 7, "7 卷章节");
        assert_eq!(rows[0].1, "明朝那些事儿*壹");
        let content = state
            .storage
            .get_chapter_content("default", &umd_url, 0)
            .await
            .unwrap()
            .unwrap();
        assert!(content.contains("前言"), "正文应含可读文本");

        cleanup(state, dir).await;
    }

    /// 本地书非文本文型 getBookContent 分派（legacy 三模式最小对齐）：
    /// - CBZ（type=2 / .cbz）：章节页图解压到 assets/{ns}/cbz/{md5(bookUrl)}/{index}/，
    ///   返回 `<img src="...">` 标签列表（自然序；zip-slip 恶意条目拒收）；二次请求幂等直读
    /// - local:// 上传书同分派（opds_files 原文件定位）
    /// - PDF（type=4 / .pdf）：无已转换页图 → 提示文本；有 → 页图标签列表（数字序）
    /// - EPUB epubContent 参数接受不报错，TXT 等文本通道不受影响
    #[tokio::test]
    async fn test_get_book_content_cbz_pdf_image_modes() {
        let (state, dir) = test_state("locimg").await;
        let books_dir = state
            .storage
            .config
            .storage_dir()
            .join("data/default/books");
        std::fs::create_dir_all(&books_dir).unwrap();

        // 构造内存 CBZ（乱序写入验证自然序 1.png < 2.jpg < 10.png）+ zip-slip 恶意条目
        let png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x62, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        use std::io::Write as _;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::FileOptions::default();
            zip.start_file("10.png", opts).unwrap();
            zip.write_all(png).unwrap();
            zip.start_file("../evil.png", opts).unwrap();
            zip.write_all(b"evil").unwrap();
            zip.start_file("2.jpg", opts).unwrap();
            zip.write_all(b"jpeg").unwrap();
            zip.start_file(".DS_Store", opts).unwrap();
            zip.write_all(b"junk").unwrap();
            zip.start_file("readme.txt", opts).unwrap();
            zip.write_all(b"not-image").unwrap();
            zip.start_file("1.png", opts).unwrap();
            zip.write_all(png).unwrap();
            zip.finish().unwrap();
        }
        let zip_bytes = buf.into_inner();
        let cbz_path = books_dir.join("漫画.cbz");
        std::fs::write(&cbz_path, &zip_bytes).unwrap();
        let cbz_url = format!(
            "storage/data/default/books/{}",
            cbz_path.file_name().unwrap().to_string_lossy()
        );
        state
            .storage
            .upsert_book(
                "default",
                &crate::model::Book {
                    book_url: cbz_url.clone(),
                    name: "漫画".into(),
                    origin: "loc_book".into(),
                    origin_name: "本地书".into(),
                    book_type: 2,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // ① 文件路径书 #1：页图 img 标签列表（自然序），解压落盘 assets/{ns}/cbz/{md5}/1/
        let md5 = crate::util::md5::md5_encode(&cbz_url);
        let base_href = format!("/assets/default/cbz/{md5}/1/");
        let ret = get_book_content_file(&state, "default", &format!("{cbz_url}#1"), false)
            .await
            .expect("CBZ 正文应返回页图模式");
        let html = ret.0.data["content"].as_str().unwrap().to_string();
        assert_eq!(html.matches("<img").count(), 3, "仅图片条目参与: {html}");
        let t1 = format!("<img src=\"{base_href}1.png\">");
        let t2 = format!("<img src=\"{base_href}2.jpg\">");
        let t10 = format!("<img src=\"{base_href}10.png\">");
        let (p1, p2, p10) = (
            html.find(&t1).expect("缺 1.png"),
            html.find(&t2).expect("缺 2.jpg"),
            html.find(&t10).expect("缺 10.png"),
        );
        assert!(p1 < p2 && p2 < p10, "页图按文件名自然序: {html}");
        let chapter_dir = state
            .storage
            .config
            .storage_dir()
            .join("assets/default/cbz")
            .join(&md5)
            .join("1");
        assert!(chapter_dir.join("1.png").is_file(), "页图应解压落盘");
        assert!(chapter_dir.join("2.jpg").is_file());
        // zip-slip：恶意条目 ../evil.png 不得逃逸出解压目录
        assert!(!chapter_dir.join("evil.png").exists());
        assert!(!state
            .storage
            .config
            .storage_dir()
            .join("assets/default/cbz/evil.png")
            .exists());
        assert!(!state.storage.config.storage_dir().join("evil.png").exists());
        assert!(
            !chapter_dir.join(".DS_Store").is_file() && !chapter_dir.join("readme.txt").is_file(),
            "隐藏/非图片条目不落盘"
        );

        // ② 二次请求幂等：已解压目录直读，内容一致
        let ret2 = get_book_content_file(&state, "default", &format!("{cbz_url}#1"), false)
            .await
            .expect("二次请求应成功");
        assert_eq!(ret2.0.data["content"], serde_json::json!(html));

        // ③ local:// 上传书同分派（源文件经 opds_files 定位）
        use tower::ServiceExt as _;
        let app: axum::Router = axum::Router::new()
            .route(
                "/reader3/uploadLocalBook",
                axum::routing::post(upload_local_book),
            )
            .with_state(state.clone());
        let boundary = "----reader-imgmodes";
        let mut mp: Vec<u8> = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"上传漫画.cbz\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .into_bytes();
        mp.extend_from_slice(&zip_bytes);
        mp.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/reader3/uploadLocalBook")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(axum::body::Body::from(mp))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 8 * 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(
            json["isSuccess"].as_bool().unwrap(),
            "CBZ 上传导入应成功: {json}"
        );
        let upload_url = json["data"]["bookUrl"].as_str().unwrap().to_string();
        assert_eq!(json["data"]["type"], 2, "CBZ 上传书应为漫画类型");
        let ret3 = get_book_content_local(&state, "default", &format!("{upload_url}/0"), false)
            .await
            .expect("local:// CBZ 正文应返回页图模式");
        let html3 = ret3.0.data["content"].as_str().unwrap();
        let umd5 = crate::util::md5::md5_encode(&upload_url);
        assert_eq!(
            html3.matches("<img").count(),
            3,
            "local:// 同样返回页图列表: {html3}"
        );
        assert!(
            html3.contains(&format!("<img src=\"/assets/default/cbz/{umd5}/0/1.png\">")),
            "{html3}"
        );
        assert!(state
            .storage
            .config
            .storage_dir()
            .join("assets/default/cbz")
            .join(&umd5)
            .join("0")
            .join("1.png")
            .is_file());

        // ④ PDF：无已转换页图 → 提示文本；预置页图后 → 标签列表（数字序）
        let pdf_url = "storage/data/default/books/画册.pdf";
        std::fs::write(books_dir.join("画册.pdf"), b"%PDF-1.4\nfake").unwrap();
        let pdf_md5 = crate::util::md5::md5_encode(pdf_url);
        let ret = get_book_content_file(&state, "default", &format!("{pdf_url}#0"), false)
            .await
            .expect("PDF 正文应返回提示");
        assert_eq!(
            ret.0.data["content"],
            serde_json::json!("PDF 阅读需要先转换页面")
        );
        let pages_dir = state
            .storage
            .config
            .storage_dir()
            .join("assets/default/pdf")
            .join(&pdf_md5);
        std::fs::create_dir_all(&pages_dir).unwrap();
        std::fs::write(pages_dir.join("2.jpg"), b"jpeg").unwrap();
        std::fs::write(pages_dir.join("10.jpg"), b"jpeg").unwrap();
        std::fs::write(pages_dir.join("1.jpg"), b"jpeg").unwrap();
        let ret = get_book_content_file(&state, "default", &format!("{pdf_url}#0"), false)
            .await
            .expect("PDF 已转换页应返回标签列表");
        let html4 = ret.0.data["content"].as_str().unwrap().to_string();
        let pdf_base = format!("/assets/default/pdf/{pdf_md5}/");
        let (i1, i2, i10) = (
            html4.find(&format!("{pdf_base}1.jpg")).unwrap(),
            html4.find(&format!("{pdf_base}2.jpg")).unwrap(),
            html4.find(&format!("{pdf_base}10.jpg")).unwrap(),
        );
        assert!(i1 < i2 && i2 < i10, "PDF 页图按页码序: {html4}");

        // ⑤ 文本书不受影响 + epubContent 参数接受不报错（仍文本提取）
        std::fs::write(books_dir.join("小说.txt"), "第一章 起点\n内容一。").unwrap();
        let txt_url = "storage/data/default/books/小说.txt";
        let p: HashMap<String, String> = [
            ("chapterUrl".into(), format!("{txt_url}#0")),
            ("epubContent".into(), "1".into()),
        ]
        .into_iter()
        .collect();
        let ret =
            get_book_content(AxumState(state.clone()), Query(p), HeaderMap::new(), None).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(
            ret.0.data["content"]
                .as_str()
                .unwrap_or("")
                .contains("内容一。"),
            "文本通道不受图片模式影响: {}",
            ret.0.data
        );

        cleanup(state, dir).await;
    }

    /// getBookContent epubContent 模式（legacy 参数对齐）：
    /// - EPUB + epubContent=1 → 正文包裹基本 HTML 结构（<html><body><p>…</p>…</body></html>）
    /// - EPUB 默认（epubContent 缺省）→ 纯文本不变；TXT + epubContent=1 → 不包裹
    /// - local:// 上传书同语义（源文件扩展名判定）；handler 全链路参数解析
    #[tokio::test]
    async fn test_get_book_content_epub_content_mode() {
        let (state, dir) = test_state("epubcontent").await;
        let books_dir = state
            .storage
            .config
            .storage_dir()
            .join("data/default/books");
        std::fs::create_dir_all(&books_dir).unwrap();

        // 构造最小 EPUB：spine 两章（h1 标题 + 段落正文）
        use std::io::Write as _;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::FileOptions::default();
            zip.start_file("mimetype", opts).unwrap();
            zip.write_all(b"application/epub+zip").unwrap();
            zip.start_file("META-INF/container.xml", opts).unwrap();
            zip.write_all(
                br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#,
            )
            .unwrap();
            zip.start_file("OEBPS/content.opf", opts).unwrap();
            zip.write_all(
                br#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="id">
  <metadata><dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">Test</dc:title></metadata>
  <manifest>
    <item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="c2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="c1"/><itemref idref="c2"/></spine>
</package>"#,
            )
            .unwrap();
            zip.start_file("OEBPS/c1.xhtml", opts).unwrap();
            zip.write_all(
                r#"<html><head><title>第一章</title></head><body><h1>第一章</h1><p>内容一。</p></body></html>"#
                    .as_bytes(),
            )
            .unwrap();
            zip.start_file("OEBPS/c2.xhtml", opts).unwrap();
            zip.write_all(
                r#"<html><head><title>第二章</title></head><body><h1>第二章</h1><p>内容二。</p></body></html>"#
                    .as_bytes(),
            )
            .unwrap();
            zip.finish().unwrap();
        }
        let epub_bytes = buf.into_inner();
        let epub_path = books_dir.join("测试书.epub");
        std::fs::write(&epub_path, &epub_bytes).unwrap();
        let epub_url = format!(
            "storage/data/default/books/{}",
            epub_path.file_name().unwrap().to_string_lossy()
        );

        // ① EPUB 默认路径：纯文本不变
        let ret = get_book_content_file(&state, "default", &format!("{epub_url}#0"), false)
            .await
            .expect("EPUB 默认应返回纯文本");
        let plain = ret.0.data["content"].as_str().unwrap();
        assert!(
            plain.contains("内容一。") && !plain.contains("<html"),
            "{plain}"
        );

        // ② epubContent=1：段落 <p> 包裹 + 基本 HTML 结构
        let ret = get_book_content_file(&state, "default", &format!("{epub_url}#0"), true)
            .await
            .expect("EPUB epubContent=1 应返回 HTML");
        let html = ret.0.data["content"].as_str().unwrap();
        assert!(
            html.starts_with("<html><body>") && html.ends_with("</body></html>"),
            "{html}"
        );
        assert!(html.contains("<p>第一章</p>"), "{html}");
        assert!(html.contains("<p>内容一。</p>"), "{html}");
        assert_eq!(
            html.matches("<p>").count(),
            html.matches("</p>").count(),
            "段落标签配平: {html}"
        );

        // ③ handler 全链路：epubContent 参数解析（GET query）
        let p: HashMap<String, String> = [
            ("chapterUrl".into(), format!("{epub_url}#1")),
            ("epubContent".into(), "1".into()),
        ]
        .into_iter()
        .collect();
        let ret =
            get_book_content(AxumState(state.clone()), Query(p), HeaderMap::new(), None).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(
            ret.0.data["content"]
                .as_str()
                .unwrap_or("")
                .contains("<p>内容二。</p>"),
            "{}",
            ret.0.data
        );
        // TXT + epubContent=1 → 不包裹（仅 EPUB 生效）
        std::fs::write(books_dir.join("小说.txt"), "第一章 起点\n内容一。").unwrap();
        let p: HashMap<String, String> = [
            (
                "chapterUrl".into(),
                "storage/data/default/books/小说.txt#0".into(),
            ),
            ("epubContent".into(), "1".into()),
        ]
        .into_iter()
        .collect();
        let ret =
            get_book_content(AxumState(state.clone()), Query(p), HeaderMap::new(), None).await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let txt = ret.0.data["content"].as_str().unwrap();
        assert!(txt.contains("内容一。") && !txt.contains("<html"), "{txt}");

        // ④ local:// 上传书：源文件扩展名判定，同语义返回 HTML 结构
        use tower::ServiceExt as _;
        let app: axum::Router = axum::Router::new()
            .route(
                "/reader3/uploadLocalBook",
                axum::routing::post(upload_local_book),
            )
            .with_state(state.clone());
        let boundary = "----reader-epubcontent";
        let mut mp: Vec<u8> = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"电子书.epub\"\r\nContent-Type: application/epub+zip\r\n\r\n"
        )
        .into_bytes();
        mp.extend_from_slice(&epub_bytes);
        mp.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/reader3/uploadLocalBook")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(axum::body::Body::from(mp))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 8 * 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(
            json["isSuccess"].as_bool().unwrap(),
            "EPUB 上传导入应成功: {json}"
        );
        let upload_url = json["data"]["bookUrl"].as_str().unwrap().to_string();
        // 默认纯文本
        let ret = get_book_content_local(&state, "default", &format!("{upload_url}/0"), false)
            .await
            .expect("local:// EPUB 默认应返回纯文本");
        let plain = ret.0.data["content"].as_str().unwrap();
        assert!(
            plain.contains("内容一。") && !plain.contains("<html"),
            "{plain}"
        );
        // epubContent=1 → HTML 结构
        let ret = get_book_content_local(&state, "default", &format!("{upload_url}/0"), true)
            .await
            .expect("local:// EPUB epubContent=1 应返回 HTML");
        let html = ret.0.data["content"].as_str().unwrap();
        assert!(
            html.starts_with("<html><body>") && html.contains("<p>内容一。</p>"),
            "{html}"
        );

        cleanup(state, dir).await;
    }

    // ---------------- P1-C4：saveBook 书籍数上限 ----------------

    /// saveBook 新增入架超 users.book_limit → 拒绝；已存在覆盖不计名额；limit<=0 不限制
    #[tokio::test]
    async fn test_save_book_limit() {
        let (state, dir) = test_state("booklimit").await;
        state
            .storage
            .insert_user(&User {
                username: "default".into(),
                book_limit: 2,
                ..Default::default()
            })
            .await
            .unwrap();
        let save = |url: &str| {
            let state = state.clone();
            let body = Bytes::from(
                serde_json::json!({
                    "bookUrl": url,
                    "name": url,
                })
                .to_string(),
            );
            async move {
                save_book(
                    AxumState(state),
                    Query(HashMap::new()),
                    HeaderMap::new(),
                    Some(body),
                )
                .await
            }
        };
        // 前 2 本成功
        for i in 1..=2 {
            let ret = save(&format!("https://b{i}.com")).await;
            assert!(ret.0.is_success, "第 {i} 本应成功: {}", ret.0.error_msg);
        }
        // 第 3 本超限拒绝
        let ret = save("https://b3.com").await;
        assert!(!ret.0.is_success);
        assert_eq!(ret.0.error_msg, "你已达到书籍数上限，请联系管理员");
        assert!(state
            .storage
            .find_book("default", "https://b3.com")
            .await
            .unwrap()
            .is_none());
        // 已存在书覆盖（编辑）不计名额
        let ret = save("https://b1.com").await;
        assert!(ret.0.is_success, "编辑已有书应成功: {}", ret.0.error_msg);
        // 无用户行（非 secure 模式）→ 不限制
        let (state2, dir2) = test_state("booklimit2").await;
        for i in 1..=3 {
            let body = Bytes::from(
                serde_json::json!({
                    "bookUrl": format!("https://n{i}.com"),
                    "name": format!("n{i}"),
                })
                .to_string(),
            );
            let ret = save_book(
                AxumState(state2.clone()),
                Query(HashMap::new()),
                HeaderMap::new(),
                Some(body),
            )
            .await;
            assert!(ret.0.is_success, "无用户行不限制: {}", ret.0.error_msg);
        }
        cleanup(state2, dir2).await;
        cleanup(state, dir).await;
    }

    // ---------------- legacy 三分支：saveBook 本地书文件迁移 ----------------

    /// saveBook 本地书三分支迁移（legacy saveBookToShelf）：
    /// ① /assets/ 上传临时路径 → 文件移入 data/{ns}/{name}_{author}/，bookUrl/tocUrl 重写；
    /// ② 编辑保存（已是最终路径）不再迁移；③ localStore 分支；④ webdav 分支；
    /// ⑤ 源文件不存在 → 降级保留原路径照常入架
    #[tokio::test]
    async fn test_save_book_local_file_migration() {
        let (state, dir) = test_state("locmigrate").await;
        let storage = state.storage.config.storage_dir();
        let save = |state: AppState, body: serde_json::Value| {
            Box::pin(async move {
                save_book(
                    AxumState(state),
                    Query(HashMap::new()),
                    HeaderMap::new(),
                    Some(Bytes::from(body.to_string())),
                )
                .await
            })
        };

        // ① assets 分支：上传临时文件 → 入架迁移
        let tmp_dir = storage.join("assets/default/book");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        std::fs::write(tmp_dir.join("测试书.epub"), b"PK\x03\x04epub").unwrap();
        let ret = save(
            state.clone(),
            serde_json::json!({
                "bookUrl": "/assets/default/book/测试书.epub",
                "tocUrl": "/assets/default/book/测试书.epub",
                "name": "测试书",
                "author": "作者A",
                "origin": "loc_book",
            }),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        let new_url = "storage/data/default/测试书_作者A/测试书.epub";
        assert!(
            storage
                .join("data/default/测试书_作者A/测试书.epub")
                .is_file(),
            "文件应迁入 name_author 目录"
        );
        assert!(!tmp_dir.join("测试书.epub").exists(), "临时源文件应删除");
        assert!(
            super::resolve_loc_book_file(&storage, new_url).is_some(),
            "新路径应可被白名单定位"
        );
        let saved = state
            .storage
            .find_book("default", new_url)
            .await
            .unwrap()
            .expect("迁移后书籍入库");
        assert_eq!(saved.book_url, new_url);
        assert_eq!(saved.toc_url, new_url, "tocUrl 应同步为相对路径");
        assert!(saved.is_in_shelf);
        assert!(state
            .storage
            .find_book("default", "/assets/default/book/测试书.epub")
            .await
            .unwrap()
            .is_none());

        // ② 编辑保存（URL 已是最终形态）→ 不迁移不改路径
        let ret = save(
            state.clone(),
            serde_json::json!({
                "bookUrl": new_url,
                "tocUrl": new_url,
                "name": "测试书改",
                "author": "作者A",
                "origin": "loc_book",
            }),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(
            storage
                .join("data/default/测试书_作者A/测试书.epub")
                .is_file(),
            "编辑不应移动文件"
        );
        assert!(
            !storage.join("data/default/测试书改_作者A").exists(),
            "编辑不应产生新目录"
        );
        assert!(state
            .storage
            .find_book("default", new_url)
            .await
            .unwrap()
            .is_some());

        // ③ localStore 分支：storage/localStore/** → data/{ns}/{name}_{author}/
        let store_dir = storage.join("localStore");
        std::fs::create_dir_all(&store_dir).unwrap();
        std::fs::write(store_dir.join("仓书.txt"), "第一章 起点\n内容。").unwrap();
        let ret = save(
            state.clone(),
            serde_json::json!({
                "bookUrl": "storage/localStore/仓书.txt",
                "name": "仓书",
                "author": "作者B",
                "origin": "loc_book",
            }),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(storage.join("data/default/仓书_作者B/仓书.txt").is_file());
        assert!(!store_dir.join("仓书.txt").exists());
        let saved3 = state
            .storage
            .find_book("default", "storage/data/default/仓书_作者B/仓书.txt")
            .await
            .unwrap()
            .expect("localStore 迁移入库");
        assert_eq!(saved3.toc_url, saved3.book_url);

        // ④ webdav 分支：storage/data/{ns}/webdav/** → data/{ns}/{name}_{author}/
        let dav_dir = storage.join("data/default/webdav");
        std::fs::create_dir_all(&dav_dir).unwrap();
        std::fs::write(dav_dir.join("dav书.txt"), "正文内容").unwrap();
        let ret = save(
            state.clone(),
            serde_json::json!({
                "bookUrl": "storage/data/default/webdav/dav书.txt",
                "name": "dav书",
                "author": "作者C",
                "origin": "loc_book",
            }),
        )
        .await;
        assert!(ret.0.is_success, "{}", ret.0.error_msg);
        assert!(storage.join("data/default/dav书_作者C/dav书.txt").is_file());
        assert!(!dav_dir.join("dav书.txt").exists());

        // ⑤ 源文件不存在 → 迁移降级跳过，保留原 URL 照常入架
        let ghost_url = "/assets/default/book/ghost.epub";
        let ret = save(
            state.clone(),
            serde_json::json!({
                "bookUrl": ghost_url,
                "name": "幽灵书",
                "author": "作者D",
                "origin": "loc_book",
            }),
        )
        .await;
        assert!(
            ret.0.is_success,
            "迁移失败不应阻断保存: {}",
            ret.0.error_msg
        );
        let saved5 = state
            .storage
            .find_book("default", ghost_url)
            .await
            .unwrap()
            .expect("降级后按原 URL 入库");
        assert_eq!(saved5.book_url, ghost_url);

        cleanup(state, dir).await;
    }

    // ---------------- P1-C6：/assets/proxy 上游 Content-Type 安全 ----------------

    /// sanitize_proxy_content_type：合法 token 透传；非法/注入/缺失回退默认 image/png
    #[test]
    fn test_sanitize_proxy_content_type() {
        // 合法：无参 / 带参数 / 带引号参数
        assert_eq!(sanitize_proxy_content_type(Some("image/png")), "image/png");
        assert_eq!(
            sanitize_proxy_content_type(Some("text/html; charset=utf-8")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            sanitize_proxy_content_type(Some("application/json; charset=\"utf-8\"")),
            "application/json; charset=\"utf-8\""
        );
        // 缺失 → 默认
        assert_eq!(sanitize_proxy_content_type(None), "image/png");
        assert_eq!(sanitize_proxy_content_type(Some("")), "image/png");
        // 非法：控制字符/CRLF 注入/空白/非 token 字符/坏参数
        assert_eq!(
            sanitize_proxy_content_type(Some("text/html\r\nX-Evil: 1")),
            "image/png",
            "CRLF 注入应回退默认"
        );
        assert_eq!(
            sanitize_proxy_content_type(Some("text /html")),
            "image/png",
            "空白应回退默认"
        );
        assert_eq!(
            sanitize_proxy_content_type(Some("text")),
            "image/png",
            "缺子类型应回退默认"
        );
        assert_eq!(
            sanitize_proxy_content_type(Some("image/png; charset=\"utf 8\"")),
            "image/png",
            "引号参数含空白应回退默认"
        );
        assert_eq!(
            sanitize_proxy_content_type(Some("image/png; badparam")),
            "image/png",
            "坏参数应回退默认"
        );
    }

    /// /assets/proxy 端到端：上游返回非法 Content-Type（含空白）→ 响应回退默认 image/png；
    /// 合法 image/png 照常透传（既有 test_assets_proxy_endpoint 已覆盖）
    #[tokio::test]
    async fn test_assets_proxy_invalid_content_type_falls_back() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1
        let (state, dir) = test_state("proxy-ct").await;
        let png: Vec<u8> = vec![0x89, b'P', b'N', b'G', 9, 9, 9];
        let body = png.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf).await;
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text /html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let mut resp = head.into_bytes();
            resp.extend_from_slice(&body);
            let _ = sock.write_all(&resp).await;
        });
        let img_url = format!("http://{addr}/evil/1");
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("url".into(), img_url);
        let resp = assets_proxy(AxumState(state.clone()), Query(params), HeaderMap::new()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok()),
            Some("image/png"),
            "非法上游 Content-Type 应回退默认 image/png"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        assert_eq!(bytes.to_vec(), png, "图片字节仍应透传");
        cleanup(state, dir).await;
    }

    // ---------------- 搜索去重不 trim / SSE 并发默认 24 / 进程内书籍信息缓存 ----------------

    /// 多源搜索去重：legacy 不 trim——含首尾空格的书名视为不同书（不少合并），
    /// 完全同名同作者才去重
    #[test]
    fn test_dedup_search_books_no_trim() {
        let mk = |name: &str, origin: &str| crate::service::search::SearchBook {
            name: name.into(),
            author: "作者".into(),
            origin: origin.into(),
            ..Default::default()
        };
        let books = vec![
            mk("三体", "https://a.com"),
            mk("三体 ", "https://b.com"), // 尾随空格：legacy 语义不去重
            mk("三体", "https://c.com"),  // 与首条同键 → 被去重
            mk(" 三体", "https://d.com"), // 前导空格：同样不去重
        ];
        let out = dedup_search_books(books);
        assert_eq!(out.len(), 3, "含空格书名不应与去空格版本合并");
        assert_eq!(out[0].origin, "https://a.com");
        assert_eq!(out[1].origin, "https://b.com");
        assert_eq!(out[2].origin, "https://d.com");

        // 同名同作者仍正常去重，保留首个书源命中
        let out = dedup_search_books(vec![mk("书A", "https://a.com"), mk("书A", "https://b.com")]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].origin, "https://a.com");
    }

    /// SSE 并发数：缺省 24 对齐 legacy searchBookMultiSSE；显式值 clamp 到 1..=128
    #[test]
    fn test_effective_concurrent_count_default_24() {
        assert_eq!(effective_concurrent_count(None), 24, "缺省应对齐 legacy 24");
        assert_eq!(effective_concurrent_count(Some(24)), 24);
        assert_eq!(effective_concurrent_count(Some(10)), 10);
        assert_eq!(effective_concurrent_count(Some(0)), 1, "下限 1");
        assert_eq!(
            effective_concurrent_count(Some(9999)),
            128,
            "上限 128 防打爆连接"
        );
    }

    /// 进程内书籍详情缓存：第二次 getBookInfo 命中缓存直接返回（无网络请求——
    /// mock 只接受 1 个连接），非书架书复用上次成功的书源
    #[tokio::test]
    async fn test_get_book_info_in_process_cache_hit() {
        let _ssrf = crate::service::crawler::ssrf_allow_private_guard(true); // mock 绑定 127.0.0.1
        let (state, dir) = test_state("infocache").await;
        let body = r#"{"name":"缓存书","author":"作者","url":"/book/1","intro":"简介"}"#;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            // 仅接受 1 个连接：第二次请求若走网络必然失败
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = sock.write_all(resp.as_bytes()).await;
        });
        let base = format!("http://{addr}");
        state
            .storage
            .save_book_source(
                "default",
                &crate::model::BookSource {
                    book_source_url: base.clone(),
                    book_source_name: "缓存源".into(),
                    rule_book_info: Some(serde_json::json!({
                        "name": "$.name",
                        "author": "$.author",
                        "bookUrl": "$.url",
                        "intro": "$.intro",
                    })),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let book_url = format!("{base}/book/1");
        let params: HashMap<String, String> = [
            ("url".into(), book_url.clone()),
            ("bookSource".into(), base.clone()),
        ]
        .into_iter()
        .collect();

        // 首次：网络抓取成功并写入进程内缓存
        let ret = get_book_info(
            AxumState(state.clone()),
            Query(params.clone()),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "首次应成功: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["name"], "缓存书");

        // 第二次：mock 已不再接受连接 → 成功即证明命中缓存（跳过源解析+网络）
        let ret = get_book_info(
            AxumState(state.clone()),
            Query(params),
            HeaderMap::new(),
            None,
        )
        .await;
        assert!(ret.0.is_success, "第二次应命中缓存: {}", ret.0.error_msg);
        assert_eq!(ret.0.data["name"], "缓存书");
        assert_eq!(ret.0.data["origin"], base);

        cleanup(state, dir).await;
    }

    /// 缓存淘汰与隔离：ns 隔离、FIFO 插入序淘汰（容量 200）、过期条目清除
    /// （用独立 store 实例验证逻辑——不污染全局缓存，避免并行测试互相干扰）
    #[test]
    fn test_book_info_cache_eviction_ns_isolation_and_ttl() {
        let ns = "infocache-unit";
        let mk = |name: &str| crate::model::book_chapter::BookInfo {
            name: name.into(),
            ..Default::default()
        };
        let mut store = BookInfoCacheStore::default();

        // ns 隔离：key = ns:url
        store.put(ns, "u1", mk("A"));
        assert_eq!(store.get(ns, "u1").unwrap().name, "A");
        assert!(store.get("other-ns", "u1").is_none());

        // 过期条目 → get 返回 None 并顺带清除
        let old = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(
                BOOK_INFO_CACHE_TTL_MS + 1000,
            ))
            .unwrap();
        store.map.insert(format!("{ns}:expired"), (mk("OLD"), old));
        assert!(store.get(ns, "expired").is_none(), "过期应失效");

        // FIFO 淘汰：先放 u1，再压入 容量+10 条新键 → 最早的 u1 被逐出
        for i in 0..BOOK_INFO_CACHE_CAP + 10 {
            store.put(ns, &format!("fill-{i}"), mk("B"));
        }
        assert!(store.get(ns, "u1").is_none(), "最早条目应被淘汰");
        assert!(store.get(ns, "fill-209").is_some(), "最新条目应保留");
    }

    /// legacy 静态路由 /book-assets/* + /epub/*（YueduApi.kt:136-162）：
    /// storage/data/** 直读 + HTML </body> 前注入 __API_ROOT__ 脚本 + 404 + 防穿越
    #[tokio::test]
    async fn test_book_assets_and_epub_static_routes() {
        use tower::ServiceExt as _;
        let (state, dir) = test_state("bookassets").await;
        let data = state.storage.config.storage_dir().join("data");
        // EPUB 解压资源形态：{ns}/book-assets/**（图片/CSS）+ {ns}/{书}_{作者}/index/*.x?html
        let img_dir = data.join("alice/book-assets/img");
        std::fs::create_dir_all(&img_dir).unwrap();
        std::fs::write(img_dir.join("cover.png"), [0x89u8, b'P', b'N', b'G']).unwrap();
        std::fs::write(
            data.join("alice/book-assets/main.css"),
            "body { margin: 0 }",
        )
        .unwrap();
        let chap_dir = data.join("alice/测试书_作者A/index");
        std::fs::create_dir_all(&chap_dir).unwrap();
        std::fs::write(
            chap_dir.join("chap1.html"),
            "<html><head><title>t</title></head><BODY><p>正文</p></BODY></html>",
        )
        .unwrap();
        std::fs::write(chap_dir.join("chap2.xhtml"), "<html><body><p>二</p>").unwrap();

        let app = axum::Router::new()
            .route("/book-assets/*rest", get(book_assets))
            .route("/epub/*rest", get(epub_asset))
            .with_state(state.clone());
        let req = |uri: String| {
            axum::http::Request::builder()
                .uri(uri)
                .header("host", "srv.example:8080")
                .body(Body::empty())
                .unwrap()
        };

        // ① book-assets 图片：字节 + MIME
        let resp = app
            .clone()
            .oneshot(req("/book-assets/alice/book-assets/img/cover.png".into()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap(), "image/png");
        let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        assert_eq!(&bytes[..], &[0x89u8, b'P', b'N', b'G']);

        // ② epub 章节 HTML：中文段 percent-encoded；</BODY> 大写也注入其前；base 取 Host 头
        let seg = urlencoding::encode("测试书_作者A");
        let resp = app
            .clone()
            .oneshot(req(format!("/epub/alice/{seg}/index/chap1.html")))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            html.contains(
                "<script>window.__API_ROOT__=\"http://srv.example:8080\"</script></BODY></html>"
            ),
            "应在 </BODY> 前注入: {html}"
        );
        assert_eq!(html.matches("__API_ROOT__").count(), 1, "不重复注入");

        // ③ xhtml 同样注入；无 </body> 时追加末尾
        let resp = app
            .clone()
            .oneshot(req(format!("/epub/alice/{seg}/index/chap2.xhtml")))
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024)
            .await
            .unwrap();
        let xhtml = String::from_utf8(body.to_vec()).unwrap();
        assert!(xhtml.ends_with("<script>window.__API_ROOT__=\"http://srv.example:8080\"</script>"));

        // ④ 非 HTML 不注入（CSS 原样返回）
        let resp = app
            .clone()
            .oneshot(req("/book-assets/alice/book-assets/main.css".into()))
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"body { margin: 0 }");

        // ⑤ 文件不存在 → 404；目录 → 404
        for uri in [
            "/book-assets/alice/book-assets/ghost.png",
            "/epub/alice/ghost/index/chap1.html",
            "/book-assets",
            "/epub/",
        ] {
            let resp = app.clone().oneshot(req(uri.into())).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{uri} 应 404");
        }

        // ⑥ 防穿越：明文 ..、编码 %2e%2e、段内 %2f 解码残留均拒绝（404）
        for uri in [
            "/book-assets/alice/../reader.db",
            "/epub/%2e%2e/secret.txt",
            "/book-assets/a/%2e%2e%2fb.txt",
        ] {
            let resp = app.clone().oneshot(req(uri.into())).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{uri} 应拒绝");
        }
        cleanup(state, dir).await;
    }

    /// P2：/simple-web/* 为 legacy 静态路由别名（YueduApi 静态目录）——
    /// 与 /simple/* 挂同一 web-simple 目录（同 handler）
    #[tokio::test]
    async fn test_simple_web_alias_route() {
        use tower::ServiceExt as _;
        let (state, dir) = test_state("simpleweb").await;
        let config = state.storage.config.clone();
        let app = crate::api::router::router(config, state.storage.clone());
        let req = |uri: String| {
            axum::http::Request::builder()
                .uri(uri)
                .header("host", "srv.example:8080")
                .body(Body::empty())
                .unwrap()
        };
        let index = std::fs::read_to_string("web-simple/index.html").unwrap();

        // ① 别名路由目录请求 → index.html
        let resp = app
            .clone()
            .oneshot(req("/simple-web/".into()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&body),
            index,
            "/simple-web/ 应与 /simple/ 同源返回 index.html"
        );

        // ② 具体文件同样可达
        let js = std::fs::read_to_string("web-simple/zh.js").unwrap();
        let resp = app
            .clone()
            .oneshot(req("/simple-web/zh.js".into()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&body), js);

        // ③ /simple 原路径不受别名影响
        let resp = app.oneshot(req("/simple/index.html".into())).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        crate::service::crawler::clear_cookie_storage();
        cleanup(state, dir).await;
    }

    /// 注入与基址推导的纯函数单元用例：大小写、无 </body>、Host 白名单、XFP proto
    #[test]
    fn test_inject_and_base_url_helpers() {
        // 大小写混合标签 + 引号转义（Host 白名单已挡，此处兜底验证转义路径）
        let out = inject_api_root_script("x</BoDy>y", "a\"b");
        assert_eq!(
            out,
            "x<script>window.__API_ROOT__=\"a\\\"b\"</script></BoDy>y"
        );
        // 无 </body> → 追加末尾
        let out = inject_api_root_script("<p>x</p>", "");
        assert_eq!(out, "<p>x</p><script>window.__API_ROOT__=\"\"</script>");

        // Host 缺失/非法字符 → 空基址
        assert_eq!(request_base_url(&HeaderMap::new()), "");
        let mut h = HeaderMap::new();
        h.insert(axum::http::header::HOST, "h\"ost".parse().unwrap());
        assert_eq!(request_base_url(&h), "");
        // 合法 Host 默认 http；X-Forwarded-Proto=https 生效；非法 proto 忽略
        let mut h = HeaderMap::new();
        h.insert(axum::http::header::HOST, "[::1]:9527".parse().unwrap());
        assert_eq!(request_base_url(&h), "http://[::1]:9527");
        h.insert("x-forwarded-proto", "https".parse().unwrap());
        assert_eq!(request_base_url(&h), "https://[::1]:9527");
        h.insert("x-forwarded-proto", "javascript:alert(1)".parse().unwrap());
        assert_eq!(request_base_url(&h), "http://[::1]:9527");

        // 相对路径校验：正常多级 + 空段折叠；.. 与解码残留拒绝
        assert_eq!(
            safe_data_rel_path("a//b/./c.png")
                .unwrap()
                .to_string_lossy(),
            std::path::Path::new("a")
                .join("b")
                .join("c.png")
                .to_string_lossy()
        );
        assert!(safe_data_rel_path("").is_none());
        assert!(safe_data_rel_path("..").is_none());
        assert!(safe_data_rel_path("a%2Fb").is_none());
        assert!(safe_data_rel_path("%C3%28").is_none(), "非法编码应拒绝");

        // find_ascii_ci：大小写不敏感且偏移正确
        assert_eq!(find_ascii_ci("ab</Body>cd", "</body>"), Some(2));
        assert_eq!(find_ascii_ci("abc", "xyz"), None);
    }
}
