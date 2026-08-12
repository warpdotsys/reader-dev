use crate::prelude::*;
// package com.htmake.reader.config

// import org.springframework.boot.context.properties.ConfigurationProperties
// import org.springframework.stereotype.Component

// @Component
// @ConfigurationProperties(prefix = "reader.app")
pub struct AppConfig {
    pub show_ui: bool,          // 是否显示UI
    pub debug: bool,            // 是否调试web
    pub packaged: bool,         // 是否打包为app
    pub secure: bool,           // 是否启用登录鉴权
    pub invite_code: String,    // 注册邀请码
    pub secure_key: String,     // 管理密码
    pub cache_chapter_content: bool, // 是否缓存章节内容
    pub user_limit: i32,        // 用户上限（宽松默认，可通过 READER_APP_USERLIMIT 配置）
    pub user_book_limit: i32,   // 用户书籍上限（宽松默认，可通过 READER_APP_USERBOOKLIMIT 配置）
    pub debug_log: bool,        // 调试日志
    pub auto_clear_inactive_user: i32, // 自动清理不活跃用户

    pub export_use_replace: bool,    // 导出不使用净化
    pub export_charset: String,      // 导出字符集
    pub export_no_chapter_name: bool, // 不添加章节名
    pub export_picture_file: bool,   // 导出图片

    // workDir - working directory (replaces storagePath)
    pub work_dir: String,

    // MongoDB configuration
    pub mongo_uri: String,
    pub mongo_db_name: String,

    // Shelf update interval (minutes)
    pub shelf_update_inteval: i32,
    // Remote webview API
    pub remote_webview_api: String,

    // Default user permission settings
    pub default_user_enable_webdav: bool,
    pub default_user_enable_local_store: bool,
    pub default_user_enable_book_source: bool,
    pub default_user_enable_rss_source: bool,
    pub default_user_book_source_limit: i32,
    pub default_user_book_limit: i32,

    // Auto backup user data
    pub auto_backup_user_data: bool,

    // Minimum user password length
    pub min_user_password_length: i32,

    // Remote book source update interval (minutes)
    pub remote_book_source_update_interval: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            show_ui: false,          // 是否显示UI
            debug: false,            // 是否调试web
            packaged: false,         // 是否打包为app
            secure: false,           // 是否启用登录鉴权
            invite_code: String::new(),   // 注册邀请码
            secure_key: String::new(),    // 管理密码
            cache_chapter_content: false, // 是否缓存章节内容
            user_limit: 500000,      // 用户上限（宽松默认，可通过 READER_APP_USERLIMIT 配置）
            user_book_limit: 500000, // 用户书籍上限（宽松默认，可通过 READER_APP_USERBOOKLIMIT 配置）
            debug_log: false,        // 调试日志
            auto_clear_inactive_user: 0, // 自动清理不活跃用户

            export_use_replace: false, // 导出不使用净化
            export_charset: "UTF-8".to_string(), // 导出字符集
            export_no_chapter_name: false, // 不添加章节名
            export_picture_file: false, // 导出图片

            // workDir - working directory (replaces storagePath)
            work_dir: String::new(),

            // MongoDB configuration
            mongo_uri: String::new(),
            mongo_db_name: "reader".to_string(),

            // Shelf update interval (minutes)
            shelf_update_inteval: 10,
            // Remote webview API
            remote_webview_api: String::new(),

            // Default user permission settings
            default_user_enable_webdav: false,
            default_user_enable_local_store: false,
            default_user_enable_book_source: true,
            default_user_enable_rss_source: true,
            default_user_book_source_limit: 200,
            default_user_book_limit: 200,

            // Auto backup user data
            auto_backup_user_data: false,

            // Minimum user password length
            min_user_password_length: 8,

            // Remote book source update interval (minutes)
            remote_book_source_update_interval: 720,
        }
    }
}
