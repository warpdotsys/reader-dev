use crate::prelude::*;
// package com.htmake.reader.entity

// data class User(
//         var username: String="",
//         var password: String="",
//         var salt: String="",
//         var token: String="",
//         var last_login_at: Long = System.currentTimeMillis(),
//         var created_at: Long = System.currentTimeMillis(),
//         var enable_webdav: Boolean = false, // 是否开启 WebDAV 功能
//         var token_map: Map<String, Long>? = None,
//         var enable_local_store: Boolean = false, // 是否开启本地书仓功能
//         var enable_book_source: Boolean = true, // 是否开启书源功能
//         var enable_rss_source: Boolean = true, // 是否开启RSS源功能
//         var book_source_limit: Int = 100, // 书源数量上限
//         var book_limit: Int = 200 // 书籍数量上限
// )
pub struct User {
    pub username: String,
    pub password: String,
    pub salt: String,
    pub token: String,
    pub last_login_at: i64,
    pub created_at: i64,
    pub enable_webdav: bool,        // 是否开启 WebDAV 功能
    pub token_map: Option<std::collections::HashMap<String, i64>>,
    pub enable_local_store: bool,   // 是否开启本地书仓功能
    pub enable_book_source: bool,   // 是否开启书源功能
    pub enable_rss_source: bool,    // 是否开启RSS源功能
    pub book_source_limit: i32,     // 书源数量上限
    pub book_limit: i32,            // 书籍数量上限
}

impl Default for User {
    fn default() -> Self {
        User {
            username: String::new(),
            password: String::new(),
            salt: String::new(),
            token: String::new(),
            last_login_at: System::current_time_millis(),
            created_at: System::current_time_millis(),
            enable_webdav: false, // 是否开启 WebDAV 功能
            token_map: None,
            enable_local_store: false, // 是否开启本地书仓功能
            enable_book_source: true, // 是否开启书源功能
            enable_rss_source: true, // 是否开启RSS源功能
            book_source_limit: 100, // 书源数量上限
            book_limit: 200,        // 书籍数量上限
        }
    }
}
