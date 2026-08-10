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
            .get_or_init(|| get_work_dir_fn("storage/cache"))
            .clone()
    }
}
