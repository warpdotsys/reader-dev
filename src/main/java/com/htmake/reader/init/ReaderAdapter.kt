package com.htmake.reader.init

import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.getRelativePath

/**
 * Singleton ReaderAdapter implementation using getWorkDir from VertExt.kt.
 */
object ReaderAdapter : ReaderAdapterInterface {

    override fun getWorkDir(subPath: String): String {
        return com.htmake.reader.utils.getWorkDir(subPath)
    }

    override fun getWorkDir(vararg subDirFiles: String): String {
        return com.htmake.reader.utils.getWorkDir(getRelativePath(*subDirFiles))
    }

    fun getRelativePath(vararg subDirFiles: String): String {
        return com.htmake.reader.utils.getRelativePath(*subDirFiles)
    }

    override fun getCacheDir(): String {
        return appCtx.cacheDir
    }

    override suspend fun getStrResponseByRemoteWebview(
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
    ): StrResponse? {
        // Remote webview API can be used if configured in AppConfig
        // For now return null (no remote webview available by default)
        return null
    }
}
