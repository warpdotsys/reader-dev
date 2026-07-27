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
        val config = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
        if (config.remoteWebviewApi.isNotEmpty()) {
            RemoteWebview.setRemoteApi(config.remoteWebviewApi)
            return RemoteWebview.getStrResponse(
                url = url,
                tag = tag,
                headerMap = headerMap,
                js_source = js,
                isPost = postBody.isNotEmpty(),
                body = postBody
            )
        }
        return null
    }
}
