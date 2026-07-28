package io.legado.app.adapters

import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.getRelativePath
import com.htmake.reader.utils.RemoteWebview
import com.htmake.reader.utils.SpringContextUtils
import com.htmake.reader.config.AppConfig
import com.htmake.reader.init.appCtx

/**
 * Default implementation of ReaderAdapterInterface using existing getWorkDir functions.
 */
class DefaultAdpater : ReaderAdapterInterface {

    override fun getWorkDir(subPath: String): String {
        return getWorkDir(subPath)
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
        throw Exception("不支持webview")
    }
}
