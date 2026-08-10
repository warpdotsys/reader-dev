// package io.legado.app.adapters
//
// import io.legado.app.help.http.StrResponse
// import io.legado.app.model.DebugLog
// import com.htmake.reader.utils.getRelativePath
// import java.nio.file.Paths

fn get_relative_path(sub_dir_files: &[&str]) -> String {
    let mut p = std::path::PathBuf::new();
    for f in sub_dir_files {
        p.push(f);
    }
    return p.to_string_lossy().to_string();
}

/**
 * Default implementation of ReaderAdapterInterface using existing getWorkDir functions.
 */
pub struct DefaultAdpater;

impl ReaderAdapterInterface for DefaultAdpater {

    fn get_work_dir(&self, sub_path: &str) -> String {
        let mut work_dir_path = String::new();
        let os_name = std::env::var("os.name").unwrap_or_default();
        let current_dir = std::env::var("user.dir").unwrap_or_default();
        if os_name.to_lowercase().starts_with("mac os") && !current_dir.starts_with("/Users/") {
            work_dir_path = std::path::Path::new(&std::env::var("user.home").unwrap_or_default())
                .join(".reader")
                .to_string_lossy()
                .to_string();
        } else {
            work_dir_path = current_dir;
        }
        return std::path::Path::new(&work_dir_path).join(sub_path).to_string_lossy().to_string();
    }

    fn get_work_dir_vararg(&self, sub_dir_files: &[&str]) -> String {
        return self.get_work_dir(&get_relative_path(sub_dir_files));
    }

    fn get_relative_path(&self, sub_dir_files: &[&str]) -> String {
        return get_relative_path(sub_dir_files);
    }

    fn get_cache_dir(&self) -> String {
        return self.get_work_dir_vararg(&["storage", "cache"]);
    }

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
    ) -> Option<StrResponse> {
        panic!("不支持webview")
    }
}
