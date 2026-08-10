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
    pub fn remote_webview_api() -> &'static mut String {
        static REMOTE_WEBVIEW_API: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        return REMOTE_WEBVIEW_API.get_or_init(|| String::new());
    }

    // fun setRemoteApi(remoteApi: String) {
    pub fn set_remote_api(remote_api: &str) {
        *Self::remote_webview_api() = remote_api.to_string();
    }

    // suspend fun getStrResponse(
    //     url: String? = null,
    //     html: String? = null,
    //     encode: String? = null,
    //     tag: String? = null,
    //     headerMap: Map<String, String>? = null,
    //     sourceRegex: String? = null,
    //     js_source: String? = null,
    //     proxy: String? = null,
    //     isPost: Boolean = false,
    //     body: String? = null,
    //     userNameSpace: String = "",
    //     debugLog: DebugLog? = null
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
        debug_log: Option<DebugLog>,
    ) -> StrResponse {
        if Self::remote_webview_api().is_empty() {
            panic!("不支持webview");
        }

        let request_body = json_encode(
            std::collections::HashMap::from([
                ("url", url),
                ("html", html),
                ("headers", header_map),
                ("js_source", js_source),
                ("proxy", proxy),
                ("http_method", if is_post { "POST" } else { "GET" }),
                ("body", body),
                ("encode", encode),
                ("tag", tag),
                ("sourceRegex", source_regex),
            ]),
        );

        let api_url = Self::remote_webview_api().clone() + "/render.html";

        let str_response = get_proxy_client(None, debug_log).new_call_str_response(0, || {
            url(api_url);
            post_json(request_body);
        });

        // Handle cookies from remote webview response
        if url.is_some() {
            let sub_domain = NetworkUtils::get_sub_domain(url.clone().unwrap());
            if !sub_domain.is_empty() {
                let cookies = str_response.raw.headers("Set-Cookie");
                if cookies.len() > 0 {
                    for cookie in cookies {
                        CookieStore::new(user_name_space.clone()).replace_cookie(sub_domain + "_cookieJar", cookie);
                    }
                }
            }
        }

        return StrResponse::new(
            url.unwrap_or_default(),
            str_response.body,
        );
    }
}
