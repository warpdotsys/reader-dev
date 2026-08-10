// package io.legado.app.help.http
//
// import okhttp3.*
// import okhttp3.Response.Builder

/**
 * An HTTP response.
 */
// @Suppress("unused", "MemberVisibilityCanBePrivate")
pub struct StrResponse {
    raw: Response,
    body: Option<String>,
    error_body: Option<ResponseBody>,
}

impl StrResponse {
    // constructor(rawResponse: Response, body: String?) {
    //     this.raw = rawResponse
    //     this.body = body
    // }
    pub fn new(raw_response: Response, body: Option<String>) -> StrResponse {
        StrResponse {
            raw: raw_response,
            body,
            error_body: None,
        }
    }

    // constructor(url: String, body: String?) {
    //     raw = Builder()
    //         .code(200)
    //         .message("OK")
    //         .protocol(Protocol.HTTP_1_1)
    //         .request(Request.Builder().url(url).build())
    //         .build()
    //     this.body = body
    // }
    pub fn new_url(url: &str, body: Option<String>) -> StrResponse {
        let raw = ResponseBuilder::new()
            .code(200)
            .message("OK")
            .protocol(Protocol::HTTP_1_1)
            .request(Request::builder().url(url).build())
            .build();
        StrResponse {
            raw,
            body,
            error_body: None,
        }
    }

    // constructor(rawResponse: Response, errorBody: ResponseBody?) {
    //     this.raw = rawResponse
    //     this.errorBody = errorBody
    // }
    pub fn new_error(raw_response: Response, error_body: Option<ResponseBody>) -> StrResponse {
        StrResponse {
            raw: raw_response,
            body: None,
            error_body,
        }
    }

    // fun raw() = raw
    pub fn raw(&self) -> &Response {
        &self.raw
    }

    // fun url(): String {
    //     raw.networkResponse?.let {
    //         return it.request.url.toString()
    //     }
    //     return raw.request.url.toString()
    // }
    pub fn url(&self) -> String {
        // raw.networkResponse?.let {
        //     return it.request.url.toString()
        // }
        if let Some(it) = &self.raw.network_response {
            return it.request.url.to_string();
        }
        // return raw.request.url.toString()
        self.raw.request.url.to_string()
    }

    // val url: String get() = url()
    pub fn url_property(&self) -> String {
        self.url()
    }

    // fun body() = body
    pub fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    // fun code(): Int {
    //     return raw.code
    // }
    pub fn code(&self) -> i32 {
        self.raw.code
    }

    // fun message(): String {
    //     return raw.message
    // }
    pub fn message(&self) -> String {
        self.raw.message.clone()
    }

    // fun headers(): Headers {
    //     return raw.headers
    // }
    pub fn headers(&self) -> &Headers {
        &self.raw.headers
    }

    // fun isSuccessful(): Boolean = raw.isSuccessful
    pub fn is_successful(&self) -> bool {
        self.raw.is_successful
    }

    // fun errorBody(): ResponseBody? {
    //     return errorBody
    // }
    pub fn error_body(&self) -> Option<&ResponseBody> {
        self.error_body.as_ref()
    }

    // override fun toString(): String {
    //     return raw.toString()
    // }
    pub fn to_string(&self) -> String {
        self.raw.to_string()
    }
}
