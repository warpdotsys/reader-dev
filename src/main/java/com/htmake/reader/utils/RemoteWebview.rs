use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（json_encode 与 stubs 同名；Any 与 AnalyzeByJSoup 同名）
use crate::com_htmake_reader_utils_vertext::json_encode;
use crate::io_legado_app_help_http_cookiestore::CookieStore;
use crate::io_legado_app_help_http_httphelper::get_proxy_client;
use crate::io_legado_app_help_http_okhttputils::{new_call_str_response, post_json};
use crate::io_legado_app_help_http_strresponse::StrResponse;
use crate::io_legado_app_utils_networkutils::NetworkUtils;
use crate::stubs::{Any, RequestBuilder};
// package com.htmake.reader.utils

// import io.legado.app.help.http.StrResponse
// import io.legado.app.help.http.getProxyClient
// import io.legado.app.help.http.newCallStrResponse
// import io.legado.app.help.http.postJson
// import io.legado.app.model.DebugLog
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.help.http.CookieStore

// object RemoteWebview {
pub struct RemoteWebview;

impl RemoteWebview {
    // var remoteWebviewApi: String = ""
    pub fn remote_webview_api() -> &'static std::sync::Mutex<String> {
        // fix: OnceLock 只读（get_or_init 返回 &String，无法 &mut）→ Mutex 内可变性保持 Kotlin `var` 语义
        static REMOTE_WEBVIEW_API: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
        return &REMOTE_WEBVIEW_API;
    }

    // fun setRemoteApi(remoteApi: String) {
    pub fn set_remote_api(remote_api: &str) {
        *Self::remote_webview_api().lock().unwrap() = remote_api.to_string();
    }

    // suspend fun getStrResponse(
    //     url: String? = None,
    //     html: String? = None,
    //     encode: String? = None,
    //     tag: String? = None,
    //     headerMap: Map<String, String>? = None,
    //     sourceRegex: String? = None,
    //     js_source: String? = None,
    //     proxy: String? = None,
    //     isPost: Boolean = false,
    //     body: String? = None,
    //     userNameSpace: String = "",
    //     debugLog: DebugLog? = None
    // ): StrResponse {
    pub async fn get_str_response(
        url: Option<String>,
        html: Option<String>,
        encode: Option<String>,
        tag: Option<String>,
        header_map: Option<std::collections::HashMap<String, String>>,
        source_regex: Option<String>,
        js_source: Option<String>,
        proxy: Option<String>,
        is_post: bool,
        body: Option<String>,
        user_name_space: String,
        debug_log: Option<&dyn DebugLog>,
    ) -> StrResponse {
        if Self::remote_webview_api().lock().unwrap().is_empty() {
            panic!("不支持webview");
        }

        let request_body = json_encode(
            Any::Map(std::collections::HashMap::from([
                ("url".to_string(), url.clone().map(Any::from_string).unwrap_or_default()),
                ("html".to_string(), html.map(Any::from_string).unwrap_or_default()),
                ("headers".to_string(), header_map.map(|h| Any::Map(h.into_iter().map(|(k, v)| (k, Any::from_string(v))).collect())).unwrap_or_default()),
                ("js_source".to_string(), js_source.map(Any::from_string).unwrap_or_default()),
                ("proxy".to_string(), proxy.map(Any::from_string).unwrap_or_default()),
                ("http_method".to_string(), Any::from_string(if is_post { "POST".to_string() } else { "GET".to_string() })),
                ("body".to_string(), body.map(Any::from_string).unwrap_or_default()),
                ("encode".to_string(), encode.map(Any::from_string).unwrap_or_default()),
                ("tag".to_string(), tag.map(Any::from_string).unwrap_or_default()),
                ("sourceRegex".to_string(), source_regex.map(Any::from_string).unwrap_or_default()),
            ])),
            false,
        );

        let api_url = Self::remote_webview_api().lock().unwrap().clone() + "/render.html";

        let str_response = new_call_str_response(
            &get_proxy_client(None, debug_log),
            0,
            |builder: &mut RequestBuilder| {
                builder.url(&api_url);
                post_json(builder, Some(request_body.as_str()));
            },
        )
        .await;

        // Handle cookies from remote webview response
        if url.is_some() {
            let sub_domain = NetworkUtils::getSubDomain(url.as_deref());
            if !sub_domain.is_empty() {
                let cookies = str_response.raw().headers("Set-Cookie");
                if cookies.len() > 0 {
                    // fix: E0382 sub_domain 循环内被 move → 先拼好 cookieJar 再循环借用
                    let cookie_jar = sub_domain + "_cookieJar";
                    for cookie in cookies {
                        CookieStore::new(user_name_space.clone()).replace_cookie(&cookie_jar, &cookie);
                    }
                }
            }
        }

        return StrResponse::new_url(&url.unwrap_or_default(), str_response.body().cloned());
    }
}
