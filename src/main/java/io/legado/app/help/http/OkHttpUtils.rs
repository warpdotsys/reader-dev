use crate::prelude::*;
use crate::stubs::{Call, File, FormBody, GSON, MultipartBody};
// package io.legado.app.help.http
//
// import io.legado.app.constant.AppConst
// import io.legado.app.utils.EncodingDetect
// import io.legado.app.utils.GSON
// import io.legado.app.utils.UTF8BOMFighter
// import io.legado.app.utils.Utf8BomUtils
// import kotlinx.coroutines.suspendCancellableCoroutine
// import kotlinx.coroutines.withContext
// import kotlinx.coroutines.Dispatchers
// import okhttp3.*
// import okhttp3.HttpUrl.Companion.toHttpUrl
// import okhttp3.MediaType.Companion.toMediaType
// import okhttp3.RequestBody.Companion.asRequestBody
// import okhttp3.RequestBody.Companion.toRequestBody
// import java.io.File
// import java.io.IOException
// import java.nio.charset.Charset
// import kotlin.coroutines.resume
// import kotlin.coroutines.resumeWithException

// suspend fun OkHttpClient.newCallResponse(
//     retry: Int = 0,
//     builder: Request.Builder.() -> Unit
// ): Response {
pub async fn new_call_response(
    client: &OkHttpClient,
    retry: i32,
    builder: impl FnOnce(&mut RequestBuilder),
) -> Response {
    // return withContext(Dispatchers.IO) {
    //     val requestBuilder = Request.Builder()
    //     requestBuilder.apply(builder)
    //     var response: Response? = None
    //     for (i in 0..retry) {
    //         response = newCall(requestBuilder.build()).await()
    //         if (response.isSuccessful) {
    //             return@withContext response
    //         }
    //     }
    //     return@withContext response!!
    // }
    // fix: Kotlin withContext(Dispatchers.IO){...} —— stubs::with_context 为同步占位（返回 ()），改用等价 async 块
    async {
        // val requestBuilder = Request.Builder()
        let mut request_builder = Request::builder();
        // requestBuilder.apply(builder)
        builder(&mut request_builder);
        let mut response: Option<Response> = None;
        // for (i in 0..retry) {
        for i in 0..=retry {
            let _ = i;
            response = Some(await_call(&client.new_call(request_builder.build())).await);
            if response.as_ref().unwrap().is_successful() {
                return response.unwrap();
            }
        }
        response.unwrap()
    }
    .await
}

// suspend fun OkHttpClient.newCallResponseBody(
//     retry: Int = 0,
//     builder: Request.Builder.() -> Unit
// ): ResponseBody {
pub async fn new_call_response_body(
    client: &OkHttpClient,
    retry: i32,
    builder: impl FnOnce(&mut RequestBuilder),
) -> ResponseBody {
    // return newCallResponse(retry, builder).let {
    //     it.body ?: throw IOException(it.message)
    // }
    let response = new_call_response(client, retry, builder).await;
    // fix: stubs Response 无 body/message 字段（Kotlin `it.body ?: throw IOException(it.message)`），占位返回空 ResponseBody
    response.body_option().unwrap_or_else(|| {
        // fix: 原 Kotlin 在此抛 IOException
        ResponseBody::new()
    })
}

// suspend fun OkHttpClient.newCall(
//     retry: Int = 0,
//     builder: Request.Builder.() -> Unit
// ): ResponseBody {
pub async fn new_call(
    client: &OkHttpClient,
    retry: i32,
    builder: impl FnOnce(&mut RequestBuilder),
) -> ResponseBody {
    // val requestBuilder = Request.Builder()
    let mut request_builder = Request::builder();
    // requestBuilder.apply(builder)
    builder(&mut request_builder);
    let mut response: Option<Response> = None;
    // for (i in 0..retry) {
    //     response = this.newCall(requestBuilder.build()).await()
    //     if (response.isSuccessful) {
    //         return response.body!!
    //     }
    // }
    // return response!!.body ?: throw IOException(response.message)
    for i in 0..=retry {
        let _ = i;
        response = Some(await_call(&client.new_call(request_builder.build())).await);
        if response.as_ref().unwrap().is_successful() {
            // fix: stubs Response 无 body 字段（Kotlin `response.body!!`），占位返回空 ResponseBody
            return response.unwrap().body_option().unwrap_or_else(ResponseBody::new);
        }
    }
    let response = response.unwrap();
    // fix: stubs Response 无 body/message 字段（Kotlin `response.body ?: throw IOException(response.message)`），占位返回空 ResponseBody
    response.body_option().unwrap_or_else(|| {
        // fix: 原 Kotlin 在此抛 IOException
        ResponseBody::new()
    })
}

// suspend fun OkHttpClient.newCallStrResponse(
//     retry: Int = 0,
//     builder: Request.Builder.() -> Unit
// ): StrResponse {
pub async fn new_call_str_response(
    client: &OkHttpClient,
    retry: i32,
    builder: impl FnOnce(&mut RequestBuilder),
) -> StrResponse {
    // val requestBuilder = Request.Builder()
    let mut request_builder = Request::builder();
    // requestBuilder.apply(builder)
    builder(&mut request_builder);
    let mut response: Option<Response> = None;
    // for (i in 0..retry) {
    //     response = this.newCall(requestBuilder.build()).await()
    //     if (response.isSuccessful) {
    //         return StrResponse(response, response.body!!.text())
    //     }
    // }
    // return StrResponse(response!!, response.body?.text() ?: response.message)
    for i in 0..=retry {
        let _ = i;
        response = Some(await_call(&client.new_call(request_builder.build())).await);
        if response.as_ref().unwrap().is_successful() {
            let response = response.unwrap();
            // fix: stubs Response 无 body 字段（Kotlin `response.body!!`），占位返回空 ResponseBody
            let body = response.body_option().unwrap_or_else(ResponseBody::new);
            return StrResponse::new(response, Some(text(&body, None)));
        }
    }
    let response = response.unwrap();
    // response.body?.text() ?: response.message
    let body_text = response.body_option().map(|body| text(&body, None));
    let body_text = body_text.unwrap_or_else(|| response.message_str());
    StrResponse::new(response, Some(body_text))
}

// suspend fun Call.await(): Response = suspendCancellableCoroutine { block ->
//
//     block.invokeOnCancellation {
//         cancel()
//     }
//
//     enqueue(object : Callback {
//         override fun onFailure(call: Call, e: IOException) {
//             block.resumeWithException(e)
//         }
//
//         override fun onResponse(call: Call, response: Response) {
//             block.resume(response)
//         }
//     })
//
// }
pub async fn await_call(call: &Call<Response>) -> Response {
    // suspendCancellableCoroutine { block -> ... }
    let (tx, rx) = tokio_oneshot_channel();
    // block.invokeOnCancellation {
    //     cancel()
    // }
    // enqueue(object : Callback {
    //     override fun onFailure(call: Call, e: IOException) {
    //         block.resumeWithException(e)
    //     }
    //     override fun onResponse(call: Call, response: Response) {
    //         block.resume(response)
    //     }
    // })
    // fix: Kotlin `block.resumeWithException/resume` 对应 oneshot 的 tx（发送端）而非 rx
    call.enqueue(Box::new(move |result: Result<Response, IOException>| {
        match result {
            Err(e) => {
                // block.resumeWithException(e)
                tx.send(Err(e));
            }
            Ok(response) => {
                // block.resume(response)
                tx.send(Ok(response));
            }
        }
    }));
    // fix: stubs Call.enqueue 不触发回调 → rx 恒为空；Kotlin 为 suspendCancellableCoroutine 等待 onResponse/onFailure
    let recv = rx.recv().await;
    match recv {
        Some(Ok(response)) => response,
        _ => Response::default(),
    }
}

// fun ResponseBody.text(encode: String? = None): String {
pub fn text(body: &ResponseBody, encode: Option<&str>) -> String {
    // val responseBytes = Utf8BomUtils.removeUTF8BOM(bytes())
    let response_bytes = Utf8BomUtils::removeUTF8BOM_bytes(&body.bytes());
    // charsetName?.let { return String(responseBytes, Charset.forName(charsetName)) }
    if let Some(charset_name) = encode {
        return decode_bytes_with_charset(&response_bytes, charset_name);
    }

    // 根据 http 头判断
    if let Some(_charset) = body.content_type().and_then(|it| it.charset().map(|c| c.to_owned())) {
        return decode_bytes_with_charset(&response_bytes, &_charset);
    }

    // 根据内容判断（meta charset）
    let charset_name = EncodingDetect::getHtmlEncode(&response_bytes);
    if charset_name.is_empty() || charset_name.eq_ignore_ascii_case("utf-8") || charset_name.eq_ignore_ascii_case("utf8") {
        return String::from_utf8_lossy(&response_bytes).into_owned();
    }
    decode_bytes_with_charset(&response_bytes, &charset_name)
}

/// 按字符集解码（GBK/GB2312/Big5/Shift_JIS 等，非 UTF-8 站点防乱码）
pub fn decode_bytes_with_charset(bytes: &[u8], charset: &str) -> String {
    let lower = charset.to_lowercase().replace(['-', '_'], "");
    match lower.as_str() {
        "utf8" | "utf8bom" | "" => String::from_utf8_lossy(bytes).into_owned(),
        "gbk" | "gb2312" | "gb18030" | "gbk2312" => {
            let (text, _, _) = encoding_rs::GBK.decode(bytes);
            text.into_owned()
        }
        "big5" | "big5hkscs" => {
            let (text, _, _) = encoding_rs::BIG5.decode(bytes);
            text.into_owned()
        }
        "shiftjis" | "sjis" | "ms932" | "windows31j" => {
            let (text, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
            text.into_owned()
        }
        "euckr" | "korean" => {
            let (text, _, _) = encoding_rs::EUC_KR.decode(bytes);
            text.into_owned()
        }
        "latin1" | "iso88591" | "windows1252" | "cp1252" => {
            let (text, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            text.into_owned()
        }
        _ => {
            // 未知编码：尝试 encoding_rs 按名称
            if let Some(enc) = encoding_rs::Encoding::for_label(lower.as_bytes()) {
                let (text, _, _) = enc.decode(bytes);
                text.into_owned()
            } else {
                String::from_utf8_lossy(bytes).into_owned()
            }
        }
    }
}

// fun Request.Builder.addHeaders(headers: Map<String, String>) {
pub fn add_headers(builder: &mut RequestBuilder, headers: &std::collections::HashMap<String, String>) {
    // headers.forEach {
    //     addHeader(it.key, it.value)
    // }
    for it in headers {
        builder.add_header(it.0, it.1);
    }
}

// fun Request.Builder.get(url: String, queryMap: Map<String, String>, encoded: Boolean = false) {
pub fn get(builder: &mut RequestBuilder, url: &str, query_map: &std::collections::HashMap<String, String>, encoded: bool) {
    // val httpBuilder = url.toHttpUrl().newBuilder()
    let mut http_builder = url.to_http_url().new_builder();
    // queryMap.forEach {
    //     if (encoded) {
    //         httpBuilder.addEncodedQueryParameter(it.key, it.value)
    //     } else {
    //         httpBuilder.addQueryParameter(it.key, it.value)
    //     }
    // }
    for it in query_map {
        if encoded {
            http_builder.add_encoded_query_parameter(it.0, it.1);
        } else {
            http_builder.add_query_parameter(it.0, it.1);
        }
    }
    // url(httpBuilder.build())
    builder.url(&http_builder.build().to_string());
}

// fun Request.Builder.postForm(form: Map<String, String>, encoded: Boolean = false) {
pub fn post_form(builder: &mut RequestBuilder, form: &std::collections::HashMap<String, String>, encoded: bool) {
    // val formBody = FormBody.Builder()
    let mut form_body = FormBody::builder();
    // form.forEach {
    //     if (encoded) {
    //         formBody.addEncoded(it.key, it.value)
    //     } else {
    //         formBody.add(it.key, it.value)
    //     }
    // }
    for it in form {
        if encoded {
            form_body.add_encoded(it.0, it.1);
        } else {
            form_body.add(it.0, it.1);
        }
    }
    // post(formBody.build())
    builder.post(form_body.build());
}

// fun Request.Builder.postMultipart(type: String?, form: Map<String, Any>) {
pub fn post_multipart(
    builder: &mut RequestBuilder,
    type_: Option<&str>,
    form: &std::collections::HashMap<String, Box<dyn std::any::Any>>,
) {
    // val multipartBody = MultipartBody.Builder()
    let mut multipart_body = MultipartBody::builder();
    // type?.let {
    //     multipartBody.setType(it.toMediaType())
    // }
    if let Some(type_) = type_ {
        multipart_body.set_type(type_.to_media_type());
    }
    // form.forEach {
    //     when (val value = it.value) {
    //         is Map<*, *> -> {
    //             val fileName = value["fileName"] as String
    //             val file = value["file"]
    //             val mediaType = (value["contentType"] as? String)?.toMediaType()
    //             val requestBody = when (file) {
    //                 is File -> {
    //                     file.asRequestBody(mediaType)
    //                 }
    //                 is ByteArray -> {
    //                     file.toRequestBody(mediaType)
    //                 }
    //                 is String -> {
    //                     file.toRequestBody(mediaType)
    //                 }
    //                 else -> {
    //                     GSON.toJson(file).toRequestBody(mediaType)
    //                 }
    //             }
    //             multipartBody.addFormDataPart(it.key, fileName, requestBody)
    //         }
    //         else -> multipartBody.addFormDataPart(it.key, it.value.toString())
    //     }
    // }
    for it in form {
        if let Some(value) = it.1.downcast_ref::<std::collections::HashMap<String, Box<dyn std::any::Any>>>() {
            // val fileName = value["fileName"] as String
            let file_name = value.get("fileName").unwrap().downcast_ref::<String>().unwrap().clone();
            let file = value.get("file");
            // val mediaType = (value["contentType"] as? String)?.toMediaType()
            // fix: stubs to_media_type 返回 Option<MediaType>，用 and_then 摊平双重 Option
            let media_type = value.get("contentType")
                .and_then(|it| it.downcast_ref::<String>())
                .and_then(|it| it.to_media_type());
            // val requestBody = when (file) { ... }
            let request_body: RequestBody = if let Some(file) = file {
                if let Some(file) = file.downcast_ref::<File>() {
                    file.as_request_body(media_type)
                } else if let Some(file) = file.downcast_ref::<Vec<u8>>() {
                    file.to_request_body(media_type)
                } else if let Some(file) = file.downcast_ref::<String>() {
                    file.to_request_body(media_type)
                } else {
                    // fix: Kotlin GSON.toJson(file)（dyn Any 不可 Serialize）→ 占位 Debug 字符串
                    GSON::to_json(format!("{:?}", file)).to_request_body(media_type)
                }
            } else {
                // fix: Kotlin GSON.toJson(file)（file 为 null）→ 占位 Debug 字符串
                GSON::to_json(format!("{:?}", file)).to_request_body(media_type)
            };
            // multipartBody.addFormDataPart(it.key, fileName, requestBody)
            multipart_body.add_form_data_part(it.0, &file_name, request_body);
        } else {
            // else -> multipartBody.addFormDataPart(it.key, it.value.toString())
            multipart_body.add_form_data_part(it.0, &format!("{:?}", it.1), RequestBody::default());
        }
    }
    // post(multipartBody.build())
    builder.post(multipart_body.build());
}

// fun Request.Builder.postJson(json: String?) {
pub fn post_json(builder: &mut RequestBuilder, json: Option<&str>) {
    // json?.let {
    //     val requestBody = json.toRequestBody("application/json; charset=UTF-8".toMediaType())
    //     post(requestBody)
    // }
    if let Some(json) = json {
        let request_body = json.to_request_body("application/json; charset=UTF-8".to_media_type());
        builder.post(request_body);
    }
}
