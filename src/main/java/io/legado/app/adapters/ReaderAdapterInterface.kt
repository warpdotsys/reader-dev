package io.legado.app.adapters

import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog

/**
 * Interface for the reader adapter, abstracting work directory and remote webview operations.
 */
interface ReaderAdapterInterface {

    fun getWorkDir(subPath: String): String

    fun getWorkDir(vararg subDirFiles: String): String

    fun getCacheDir(): String

    suspend fun getStrResponseByRemoteWebview(
        tag: String,
        url: String,
        origin: String,
        referer: String,
        headerMap: Map<String, String>,
        postBody: String,
        cookieStore: String,
        userAgent: String,
        isAutoContent: Boolean,
        js: String,
        contentType: String,
        debugLog: DebugLog?
    ): StrResponse?
}
