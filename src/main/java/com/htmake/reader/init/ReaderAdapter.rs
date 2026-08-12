use crate::prelude::*;
// fix: 显式导入消解 prelude 多 glob 歧义（get_work_dir ← stubs(0参)/VertExt；get_relative_path ← VertExt/DefaultAdpater）
use crate::com_htmake_reader_utils_vertext::{get_relative_path, get_work_dir};
// package com.htmake.reader.init

// import io.legado.app.adapters.ReaderAdapterInterface
// import io.legado.app.help.http.StrResponse
// import io.legado.app.model.DebugLog
// import com.htmake.reader.utils.getWorkDir
// import com.htmake.reader.utils.getRelativePath
// import com.htmake.reader.utils.RemoteWebview

/**
 * Singleton ReaderAdapter implementation using getWorkDir from VertExt.kt.
 */
// object ReaderAdapter : ReaderAdapterInterface
pub struct ReaderAdapter;

// impl ReaderAdapterInterface for ReaderAdapter {
impl ReaderAdapter {
    // override fun getWorkDir(subPath: String): String {
    pub fn get_work_dir(sub_path: &str) -> String {
        return get_work_dir(sub_path);
    }

    // override fun getWorkDir(vararg subDirFiles: String): String {
    pub fn get_work_dir_multi(sub_dir_files: &[&str]) -> String {
        return get_work_dir(&Self::get_relative_path(sub_dir_files));
    }

    // fun getRelativePath(vararg subDirFiles: String): String {
    pub fn get_relative_path(sub_dir_files: &[&str]) -> String {
        return get_relative_path(sub_dir_files);
    }

    // override fun getCacheDir(): String {
    pub fn get_cache_dir() -> String {
        return Self::get_work_dir_multi(&["storage", "cache"]);
    }

    // override suspend fun getStrResponseByRemoteWebview(
    //     url: String?,
    //     html: String?,
    //     encode: String?,
    //     tag: String?,
    //     headerMap: Map<String, String>?,
    //     sourceRegex: String?,
    //     javaScript: String?,
    //     proxy: String?,
    //     post: Boolean,
    //     body: String?,
    //     userNameSpace: String,
    //     debugLog: DebugLog?
    // ): StrResponse? {
    pub async fn get_str_response_by_remote_webview(
        url: Option<String>,
        html: Option<String>,
        mut encode: Option<String>,
        tag: Option<String>,
        header_map: Option<std::collections::HashMap<String, String>>,
        source_regex: Option<String>,
        java_script: Option<String>,
        proxy: Option<String>,
        post: bool,
        body: Option<String>,
        user_name_space: String,
        // fix: E0782 DebugLog 为 trait → &dyn trait object（与 RemoteWebview::get_str_response 参数一致）
        debug_log: Option<&dyn DebugLog>,
    ) -> Option<StrResponse> {
        let encode_value = encode
            .take_if(|e| !e.is_empty())
            // fix: E0599/E0308 header_map 为 Option<HashMap> → as_ref 借用 + and_then 取 charset 再 cloned 成 Option<String>；Kotlin `?:` 对应 or_else
            .or_else(|| header_map.as_ref().and_then(|m| m.get("charset")).cloned());
        return Some(
            RemoteWebview::get_str_response(
                url,
                html,
                encode_value,
                tag,
                header_map,
                source_regex,
                java_script,
                proxy,
                post,
                body,
                user_name_space,
                debug_log,
            )
            .await,
        );
    }
}
