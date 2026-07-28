package com.htmake.reader.init

import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.getRelativePath
import com.htmake.reader.utils.RemoteWebview
import com.htmake.reader.utils.SpringContextUtils
import com.htmake.reader.config.AppConfig

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
        val config = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
        if (config.remoteWebviewApi.isNotEmpty()) {
            RemoteWebview.setRemoteApi(config.remoteWebviewApi)
            return RemoteWebview.getStrResponse(
                url = url,
                html = html,
                encode = encode ?: headerMap?.get("charset"),
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
        throw Exception("不支持webview")
    }
}
