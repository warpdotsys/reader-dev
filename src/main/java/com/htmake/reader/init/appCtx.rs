use crate::prelude::*;
// fix: 显式导入，消除 stubs get_work_dir() 与 VertExt get_work_dir(sub_path) 的 glob 歧义
use crate::com_htmake_reader_utils_vertext::get_work_dir;
// package com.htmake.reader.init

// import com.htmake.reader.utils.getWorkDir

// 处理 appCtx
// object appCtx {
pub struct AppCtx;

impl AppCtx {
    // val cacheDir: String by lazy {
    //     getWorkDir("storage", "cache")
    // }
    pub fn cache_dir() -> String {
        static CACHE_DIR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        CACHE_DIR
            .get_or_init(|| get_work_dir("storage/cache"))
            .clone()
    }
}
