package com.htmake.reader.utils

import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog

/** The request contract shared by remote and future in-process-managed renderers. */
data class WebviewRequest(
    val url: String?,
    val html: String?,
    val encode: String?,
    val tag: String?,
    val headerMap: Map<String, String>?,
    val sourceRegex: String?,
    val javaScript: String?,
    val proxy: String?,
    val post: Boolean,
    val body: String?,
    val userNameSpace: String,
    val debugLog: DebugLog?
)

interface WebviewRenderer {
    /** True only when this renderer imports and persists scoped browser cookies itself. */
    val managesBrowserCookies: Boolean get() = false

    suspend fun render(request: WebviewRequest): StrResponse

    /** Release renderer-owned workers and browser processes during application shutdown. */
    suspend fun close() = Unit
}
