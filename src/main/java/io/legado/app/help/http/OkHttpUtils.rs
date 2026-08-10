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
    //     var response: Response? = null
    //     for (i in 0..retry) {
    //         response = newCall(requestBuilder.build()).await()
    //         if (response.isSuccessful) {
    //             return@withContext response
    //         }
    //     }
    //     return@withContext response!!
    // }
    with_context(Dispatchers::IO, || async {
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
    })
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
    response.body.clone().ok_or_else(|| IOException::new(response.message)).unwrap()
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
            return response.unwrap().body.unwrap();
        }
    }
    let response = response.unwrap();
    response.body.clone().ok_or_else(|| IOException::new(response.message)).unwrap()
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
            return StrResponse::new(&response, Some(text(&response.body.as_ref().unwrap(), None)));
        }
    }
    let response = response.unwrap();
    // response.body?.text() ?: response.message
    let body_text = match &response.body {
        Some(body) => Some(text(body, None)),
        None => None,
    };
    let body_text = body_text.unwrap_or_else(|| response.message.clone());
    StrResponse::new(&response, Some(body_text))
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
pub async fn await_call(call: &Call) -> Response {
    // suspendCancellableCoroutine { block -> ... }
    let (tx, rx) = tokio_oneshot_channel();
    let _ = tx;
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
    call.enqueue(Box::new(|result: Result<Response, IOException>| {
        match result {
            Err(e) => {
                // block.resumeWithException(e)
                rx.send(Err(e));
            }
            Ok(response) => {
                // block.resume(response)
                rx.send(Ok(response));
            }
        }
    }));
    rx.recv().await.unwrap()
}

// fun ResponseBody.text(encode: String? = null): String {
pub fn text(body: &ResponseBody, encode: Option<&str>) -> String {
    // val responseBytes = Utf8BomUtils.removeUTF8BOM(bytes())
    let response_bytes = Utf8BomUtils::remove_utf8_bom(body.bytes());
    // var charsetName: String? = encode
    let charset_name: Option<String> = encode.map(|it| it.to_string());

    // charsetName?.let {
    //     return String(responseBytes, Charset.forName(charsetName))
    // }
    if let Some(charset_name) = &charset_name {
        return String::from_utf8_lossy(&response_bytes).into_owned();
    }

    //根据http头判断
    // contentType()?.charset()?.let {
    //     return String(responseBytes, it)
    // }
    if let Some(charset) = body.content_type().and_then(|it| it.charset()) {
        return String::from_utf8_lossy(&response_bytes).into_owned();
    }

    //根据内容判断
    // charsetName = EncodingDetect.getHtmlEncode(responseBytes)
    let charset_name = EncodingDetect::get_html_encode(&response_bytes);
    // return String(responseBytes, Charset.forName(charsetName))
    String::from_utf8_lossy(&response_bytes).into_owned()
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
    builder.url(http_builder.build());
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
            let media_type = value.get("contentType")
                .and_then(|it| it.downcast_ref::<String>())
                .map(|it| it.to_media_type());
            // val requestBody = when (file) { ... }
            let request_body: RequestBody = if let Some(file) = file {
                if let Some(file) = file.downcast_ref::<File>() {
                    file.as_request_body(media_type)
                } else if let Some(file) = file.downcast_ref::<Vec<u8>>() {
                    file.to_request_body(media_type)
                } else if let Some(file) = file.downcast_ref::<String>() {
                    file.to_request_body(media_type)
                } else {
                    GSON::to_json(file).to_request_body(media_type)
                }
            } else {
                GSON::to_json(file).to_request_body(media_type)
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
