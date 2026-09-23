package com.htmake.reader.utils

import io.legado.app.help.http.StrResponse
import io.legado.app.help.http.getProxyClient
import io.legado.app.help.http.newCallStrResponse
import io.legado.app.help.http.postJson
import io.legado.app.model.DebugLog
import io.legado.app.utils.NetworkUtils
import io.legado.app.help.http.CookieStore

object RemoteWebview : WebviewRenderer {
    var remoteWebviewApi: String = ""

    fun setRemoteApi(remoteApi: String) {
        remoteWebviewApi = remoteApi
    }

    override suspend fun render(request: WebviewRequest): StrResponse = getStrResponse(
        url = request.url,
        html = request.html,
        encode = request.encode,
        tag = request.tag,
        headerMap = request.headerMap,
        sourceRegex = request.sourceRegex,
        js_source = request.javaScript,
        proxy = request.proxy,
        isPost = request.post,
        body = request.body,
        userNameSpace = request.userNameSpace,
        debugLog = request.debugLog
    )

    suspend fun getStrResponse(
        url: String? = null,
        html: String? = null,
        encode: String? = null,
        tag: String? = null,
        headerMap: Map<String, String>? = null,
        sourceRegex: String? = null,
        js_source: String? = null,
        proxy: String? = null,
        isPost: Boolean = false,
        body: String? = null,
        userNameSpace: String = "",
        debugLog: DebugLog? = null
    ): StrResponse {
        if (remoteWebviewApi.isNullOrEmpty()) {
            throw Exception("不支持webview")
        }

        val requestBody = jsonEncode(
            mapOf(
                "url" to url,
                "html" to html,
                "headers" to headerMap,
                "js_source" to js_source,
                "proxy" to proxy,
                "http_method" to if (isPost) "POST" else "GET",
                "body" to body,
                "encode" to encode,
                "tag" to tag,
                "sourceRegex" to sourceRegex
            )
        )

        val apiUrl = remoteWebviewApi + "/render.html"

        val strResponse = getProxyClient(null, debugLog).newCallStrResponse(0) {
            url(apiUrl)
            postJson(requestBody)
        }

        // Handle cookies from remote webview response
        if (url != null) {
            val subDomain = NetworkUtils.getSubDomain(url)
            if (subDomain.isNotEmpty()) {
                val cookies = strResponse.raw.headers("Set-Cookie")
                if (cookies.size > 0) {
                    for (cookie in cookies) {
                        CookieStore(userNameSpace).replaceResponseCookie(subDomain + "_cookieJar", cookie)
                    }
                }
            }
        }

        return StrResponse(
            url ?: "",
            strResponse.body
        )
    }
}
