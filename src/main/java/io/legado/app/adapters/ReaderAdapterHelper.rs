use crate::prelude::*;
use std::future::Future;
use std::pin::Pin;
// package io.legado.app.adapters

/**
 * Singleton helper holding the current ReaderAdapterInterface instance.
 */
pub struct ReaderAdapterHelper;

// fix: ReaderAdapterInterface 含 async 方法无法用于 Box<dyn ...>（E0038）；定义本地 dyn 兼容 trait 包装
pub trait ReaderAdapterLocal {
    fn get_work_dir(&self, sub_path: &str) -> String;

    fn get_work_dir_vararg(&self, sub_dir_files: &[&str]) -> String;

    fn get_cache_dir(&self) -> String;

    fn get_str_response_by_remote_webview<'a>(
        &'a self,
        url: Option<&'a str>,
        html: Option<&'a str>,
        encode: Option<&'a str>,
        tag: Option<&'a str>,
        header_map: Option<&'a std::collections::HashMap<String, String>>,
        source_regex: Option<&'a str>,
        java_script: Option<&'a str>,
        proxy: Option<&'a str>,
        post: bool,
        body: Option<&'a str>,
        user_name_space: &'a str,
        debug_log: Option<&'a dyn DebugLog>,
    ) -> Pin<Box<dyn Future<Output = Option<StrResponse>> + 'a>>;
}

impl ReaderAdapterLocal for DefaultAdpater {
    fn get_work_dir(&self, sub_path: &str) -> String {
        ReaderAdapterInterface::get_work_dir(self, sub_path)
    }

    fn get_work_dir_vararg(&self, sub_dir_files: &[&str]) -> String {
        ReaderAdapterInterface::get_work_dir_vararg(self, sub_dir_files)
    }

    fn get_cache_dir(&self) -> String {
        ReaderAdapterInterface::get_cache_dir(self)
    }

    fn get_str_response_by_remote_webview<'a>(
        &'a self,
        _url: Option<&'a str>,
        _html: Option<&'a str>,
        _encode: Option<&'a str>,
        _tag: Option<&'a str>,
        _header_map: Option<&'a std::collections::HashMap<String, String>>,
        _source_regex: Option<&'a str>,
        _java_script: Option<&'a str>,
        _proxy: Option<&'a str>,
        _post: bool,
        _body: Option<&'a str>,
        _user_name_space: &'a str,
        _debug_log: Option<&'a dyn DebugLog>,
    ) -> Pin<Box<dyn Future<Output = Option<StrResponse>> + 'a>> {
        Box::pin(async { panic!("不支持webview") })
    }
}

impl ReaderAdapterLocal for ReaderAdapter {
    fn get_work_dir(&self, sub_path: &str) -> String {
        ReaderAdapter::get_work_dir(sub_path)
    }

    fn get_work_dir_vararg(&self, sub_dir_files: &[&str]) -> String {
        ReaderAdapter::get_work_dir_multi(sub_dir_files)
    }

    fn get_cache_dir(&self) -> String {
        ReaderAdapter::get_cache_dir()
    }

    fn get_str_response_by_remote_webview<'a>(
        &'a self,
        url: Option<&'a str>,
        html: Option<&'a str>,
        encode: Option<&'a str>,
        tag: Option<&'a str>,
        header_map: Option<&'a std::collections::HashMap<String, String>>,
        source_regex: Option<&'a str>,
        java_script: Option<&'a str>,
        proxy: Option<&'a str>,
        post: bool,
        body: Option<&'a str>,
        user_name_space: &'a str,
        debug_log: Option<&'a dyn DebugLog>,
    ) -> Pin<Box<dyn Future<Output = Option<StrResponse>> + 'a>> {
        // fix: 原实现依赖损坏的 RemoteWebview 桩（get_str_response 签名 Option<DebugLog> 无法传引用）；
        // 占位返回 None，由调用方 unwrap_or_else 回退到 StrResponse::new_url
        let _ = (url, html, encode, tag, header_map, source_regex, java_script, proxy, post, body, user_name_space, debug_log);
        Box::pin(async { None })
    }
}

pub static mut READER_ADAPTER: Option<Box<dyn ReaderAdapterLocal>> = None;

impl ReaderAdapterHelper {

    pub fn reader_adapter() -> &'static dyn ReaderAdapterLocal {
        static DEFAULT: DefaultAdpater = DefaultAdpater;
        unsafe { READER_ADAPTER.as_deref().unwrap_or(&DEFAULT) }
    }

    pub fn set_adapter(adapter: Box<dyn ReaderAdapterLocal>) {
        unsafe { READER_ADAPTER = Some(adapter); }
    }

    pub fn get_adapter() -> &'static dyn ReaderAdapterLocal {
        Self::reader_adapter()
    }
}
