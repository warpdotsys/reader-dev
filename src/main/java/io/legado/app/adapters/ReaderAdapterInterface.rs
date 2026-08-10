// package io.legado.app.adapters
//
// import io.legado.app.help.http.StrResponse
// import io.legado.app.model.DebugLog

/**
 * Interface for the reader adapter, abstracting work directory and remote webview operations.
 */
pub trait ReaderAdapterInterface {

    fn get_work_dir(&self, sub_path: &str) -> String;

    fn get_work_dir_vararg(&self, sub_dir_files: &[&str]) -> String;

    fn get_cache_dir(&self) -> String;

    async fn get_str_response_by_remote_webview(
        &self,
        url: Option<&str>,
        html: Option<&str>,
        encode: Option<&str>,
        tag: Option<&str>,
        header_map: Option<&std::collections::HashMap<String, String>>,
        source_regex: Option<&str>,
        java_script: Option<&str>,
        proxy: Option<&str>,
        post: bool,
        body: Option<&str>,
        user_name_space: &str,
        debug_log: Option<&DebugLog>,
    ) -> Option<StrResponse>;
}
