package com.htmake.reader.init

import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.getRelativePath
import com.htmake.reader.utils.RemoteWebview

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
        return getWorkDir("storage", "cache")
    }

    override suspend fun getStrResponseByRemoteWebview(
        url: String?,
        html: String?,
        encode: String?,
        tag: String?,
        headerMap: Map<String, String>?,
        sourceRegex: String?,
        javaScript: String?,
        proxy: String?,
        post: Boolean,
        body: String?,
        userNameSpace: String,
        debugLog: DebugLog?
    ): StrResponse? {
        return RemoteWebview.getStrResponse(
            url = url,
            html = html,
            encode = encode.takeUnless { it.isNullOrEmpty() } ?: headerMap?.get("charset"),
            tag = tag,
            headerMap = headerMap,
            sourceRegex = sourceRegex,
            js_source = javaScript,
            proxy = proxy,
            isPost = post,
            body = body,
            userNameSpace = userNameSpace,
            debugLog = debugLog
        )
    }
}
