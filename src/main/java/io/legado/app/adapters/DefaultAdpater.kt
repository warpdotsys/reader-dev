package io.legado.app.adapters

import io.legado.app.help.http.StrResponse
import io.legado.app.model.DebugLog
import com.htmake.reader.utils.getRelativePath
import java.nio.file.Paths

/**
 * Default implementation of ReaderAdapterInterface using existing getWorkDir functions.
 */
class DefaultAdpater : ReaderAdapterInterface {

    override fun getWorkDir(subPath: String): String {
        var workDirPath = ""
        val osName = System.getProperty("os.name")
        val currentDir = System.getProperty("user.dir")
        if (osName.startsWith("Mac OS", true) && !currentDir.startsWith("/Users/")) {
            workDirPath = Paths.get(System.getProperty("user.home"), ".reader").toString()
        } else {
            workDirPath = currentDir
        }
        return Paths.get(workDirPath, subPath).toString()
    }

    override fun getWorkDir(vararg subDirFiles: String): String {
        return getWorkDir(getRelativePath(*subDirFiles))
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
