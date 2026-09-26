package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.getStorage
import io.legado.app.data.entities.BookSource
import io.legado.app.help.http.CookieStore
import io.legado.app.utils.NetworkUtils
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.RoutingContext
import kotlin.coroutines.CoroutineContext

/**
 * Vue 3 书源登录接口的服务端边界。
 *
 * Cookie 继续使用阅读原有的 [CookieStore]，因此普通请求、WebView 与手动登录态共用
 * 同一份按用户命名空间隔离的存储。验证码/浏览器自动登录不能在没有可用浏览器会话的
 * 情况下伪造成功；相应接口会返回明确的可操作错误，而不是空的成功响应。
 */
class SourceLoginController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {

    private fun params(context: RoutingContext): JsonObject = when {
        context.bodyAsString.isNullOrBlank() -> JsonObject()
        else -> runCatching { context.bodyAsJson }.getOrElse { JsonObject() }
    }

    private suspend fun requireAuthenticated(context: RoutingContext, result: ReturnData): Boolean {
        if (checkAuth(context)) return true
        result.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        return false
    }

    private fun findSource(userNameSpace: String, bookSource: String): BookSource? {
        val requested = bookSource.trim()
        // Do not let a caller supply an arbitrary JSON source: login rules are executable
        // code and cookies are security credentials.  A source must first be imported.
        if (requested.isEmpty() || requested.startsWith("{")) return null
        val sources: JsonArray = (asJsonArray(getUserStorage(userNameSpace, "bookSource"))
            ?: if (userNameSpace != "default") asJsonArray(getUserStorage("default", "bookSource")) else null)
            ?: return null
        return SourceLoginSupport.findImportedSource(sources, requested)
    }

    private fun cookieIndex(userNameSpace: String): JsonObject =
        runCatching { JsonObject(getUserStorage(userNameSpace, COOKIE_INDEX_KEY) ?: "{}") }
            .getOrElse { JsonObject() }

    private fun saveCookieIndex(userNameSpace: String, index: JsonObject) {
        saveUserStorage(userNameSpace, COOKIE_INDEX_KEY, index)
    }

    private fun validateCookie(cookie: String): String? = when {
        cookie.length > MAX_COOKIE_LENGTH -> "Cookie 过长"
        cookie.any { it == '\r' || it == '\n' || it == '\u0000' } -> "Cookie 包含非法控制字符"
        cookie.isNotBlank() && cookie.split(';').none { it.trim().contains('=') } -> "Cookie 格式无效"
        else -> null
    }

    suspend fun setBookSourceCookie(context: RoutingContext): ReturnData {
        val result = ReturnData()
        if (!requireAuthenticated(context, result)) return result
        val body = params(context)
        val requested = body.getString("bookSource", "").trim()
        if (requested.isEmpty()) return result.setErrorMsg("缺少 bookSource 参数")
        val namespace = getUserNameSpace(context)
        val source = findSource(namespace, requested)
            ?: return result.setErrorMsg("书源不存在（请先导入书源）")
        val cookie = body.getString("cookie", "")
        validateCookie(cookie)?.let { return result.setErrorMsg(it) }

        val store = CookieStore(namespace)
        val index = cookieIndex(namespace)
        val scope = SourceLoginSupport.cookieScope(source.bookSourceUrl)
            ?: return result.setErrorMsg("书源地址无效")
        val cookieJarKey = "${scope}_cookieJar"
        if (cookie.isBlank()) {
            // CookieStore is keyed by domain, so clearing one source must revoke every
            // indexed/imported source that shares this authentication scope.
            store.removeCookie(scope)
            store.removeCookie(cookieJarKey)
            val candidates = knownSources(namespace, index)
            val affectedUrls = SourceLoginSupport.affectedSourceUrls(
                source.bookSourceUrl,
                candidates.map { it.bookSourceUrl }
            )
            val affectedSources = candidates.filter { it.bookSourceUrl in affectedUrls }
            affectedSources.forEach { shared ->
                    index.remove(shared.bookSourceUrl)
                    shared.setUserNameSpace(namespace)
                    shared.removeLoginHeader()
                    shared.removeLoginInfo()
                }
            saveCookieIndex(namespace, index)
            return result.setData(mapOf(
                "success" to true,
                "cleared" to true,
                "clearedSourceUrls" to affectedSources.map { it.bookSourceUrl }
            ))
        }
        store.setCookie(scope, cookie.trim())
        if (source.enabledCookieJar == true) store.setCookie(cookieJarKey, cookie.trim())
        index.put(source.bookSourceUrl, System.currentTimeMillis())
        saveCookieIndex(namespace, index)
        return result.setData(mapOf("success" to true))
    }

    suspend fun getBookSourceCookie(context: RoutingContext): ReturnData {
        val result = ReturnData()
        if (!requireAuthenticated(context, result)) return result
        val namespace = getUserNameSpace(context)
        val index = cookieIndex(namespace)
        val rows = mutableListOf<Map<String, Any?>>()
        val known = linkedSetOf<String>()
        index.fieldNames().forEach { known.add(it) }
        // Include legacy cookies that predate this controller by checking the user's current sources.
        val sources = asJsonArray(getUserStorage(namespace, "bookSource"))
            ?: if (namespace != "default") asJsonArray(getUserStorage("default", "bookSource")) else null
        sources?.forEach { item ->
            (item as? JsonObject)?.getString("bookSourceUrl")?.let { known.add(it) }
        }
        val store = CookieStore(namespace)
        val cleanedIndex = JsonObject(index.encode())
        known.forEach { sourceUrl ->
            val cookie = store.getCookie(sourceUrl)
            if (cookie.isBlank()) {
                cleanedIndex.remove(sourceUrl)
                return@forEach
            }
            // Do not evaluate a source's dynamic header JS in a listing endpoint, and do
            // not return credentials. The UI only needs a non-empty safe summary/status.
            val preview = SourceLoginSupport.redactCookie(cookie)
            rows.add(mapOf(
                "sourceUrl" to sourceUrl,
                // Compatibility field: intentionally a redacted preview, never the raw value.
                "cookie" to preview,
                "hasCookie" to true,
                "cookiePreview" to preview,
                "userAgent" to "",
                "loginHeader" to "",
                "updatedAt" to index.getLong(sourceUrl, 0L)
            ))
        }
        if (cleanedIndex.encode() != index.encode()) saveCookieIndex(namespace, cleanedIndex)
        return result.setData(rows)
    }

    /**
     * Executes only a source-provided legacy @js/<js> login rule.  Generic HTML form
     * automation is intentionally not claimed here: it requires the Camoufox session
     * protocol and per-site selectors, neither of which can be inferred safely.
     */
    suspend fun loginBookSource(context: RoutingContext): ReturnData {
        val result = ReturnData()
        if (!requireAuthenticated(context, result)) return result
        val body = params(context)
        val namespace = getUserNameSpace(context)
        val source = findSource(namespace, body.getString("bookSource", ""))
            ?: return result.setErrorMsg("书源不存在（请先导入书源）")
        val loginRule = source.loginUrl?.trim().orEmpty()
        if (loginRule.isEmpty()) return manualLoginResult(result, "书源未配置自动登录规则；请在浏览器登录后粘贴 Cookie")
        if (!loginRule.startsWith("@js:", true) && !loginRule.startsWith("<js>", true)) {
            return manualLoginResult(result, "该书源登录规则需要浏览器会话；请在浏览器登录后粘贴 Cookie")
        }
        val username = body.getString("username", "")
        val password = body.getString("password", "")
        if (username.isBlank() || password.isBlank()) return result.setErrorMsg("请输入用户名和密码")
        return try {
            source.setUserNameSpace(namespace)
            val store = CookieStore(namespace)
            val previousCookie = store.getCookie(source.bookSourceUrl)
            // Credentials are supplied only to this immediate source rule evaluation.
            // Do not retain a prior or failed password in the per-source cache.
            source.removeLoginInfo()
            source.putLoginInfo(JsonObject().put("username", username).put("password", password).encode())
            source.login()
            val cookie = store.getCookie(source.bookSourceUrl)
            if (cookie.isBlank() || cookie == previousCookie) {
                manualLoginResult(result, "登录规则未更新登录态；请在浏览器登录后粘贴 Cookie")
            } else {
                val index = cookieIndex(namespace).put(source.bookSourceUrl, System.currentTimeMillis())
                saveCookieIndex(namespace, index)
                result.setData(mapOf("success" to true, "needCaptcha" to false, "cookie" to SourceLoginSupport.redactCookie(cookie)))
            }
        } catch (error: Exception) {
            result.setErrorMsg("书源登录规则执行失败：${error.message ?: error.javaClass.simpleName}")
        } finally {
            // Cookies are the supported durable login state. Never leave submitted
            // username/password cached after a success, failure, or exception.
            source.removeLoginInfo()
        }
    }

    suspend fun getCaptcha(context: RoutingContext): ReturnData {
        val result = ReturnData()
        if (!requireAuthenticated(context, result)) return result
        val namespace = getUserNameSpace(context)
        val source = findSource(namespace, params(context).getString("bookSource", ""))
            ?: return result.setErrorMsg("书源不存在（请先导入书源）")
        val detail = if (source.loginUrl.isNullOrBlank()) {
            "书源未配置自动登录规则；请在浏览器登录后粘贴 Cookie"
        } else {
            "当前版本未启用浏览器验证码会话；请在浏览器登录后粘贴 Cookie"
        }
        return result.setData(mapOf(
            "captchaType" to "click",
            "needManualCaptcha" to true,
            "message" to detail
        ))
    }

    suspend fun submitCaptcha(context: RoutingContext): ReturnData {
        val result = ReturnData()
        if (!requireAuthenticated(context, result)) return result
        val body = params(context)
        val namespace = getUserNameSpace(context)
        val source = findSource(namespace, body.getString("bookSource", ""))
            ?: return result.setErrorMsg("书源不存在（请先导入书源）")
        return result.setData(mapOf(
            "isLogin" to false,
            "needManualCaptcha" to true,
            "message" to "当前版本未启用验证码回填；请在浏览器登录后粘贴 Cookie"
        ))
    }

    companion object {
        private const val COOKIE_INDEX_KEY = "bookSourceCookieIndex"
        private const val MAX_COOKIE_LENGTH = 64 * 1024
    }

    private fun knownSources(namespace: String, index: JsonObject): List<BookSource> {
        val urls = linkedSetOf<String>().apply { addAll(index.fieldNames()) }
        val stored = asJsonArray(getUserStorage(namespace, "bookSource"))
            ?: if (namespace != "default") asJsonArray(getUserStorage("default", "bookSource")) else null
        stored?.forEach { (it as? JsonObject)?.getString("bookSourceUrl")?.let(urls::add) }
        return urls.mapNotNull { findSource(namespace, it) }
    }

    private fun manualLoginResult(result: ReturnData, message: String): ReturnData = result.setData(mapOf(
        "success" to false,
        "needManualCaptcha" to true,
        "message" to message
    ))
}

internal object SourceLoginSupport {
    fun findImportedSource(sources: JsonArray, requested: String): BookSource? {
        if (requested.isBlank() || requested.trimStart().startsWith("{")) return null
        for (index in 0 until sources.size()) {
            val source = runCatching { sources.getJsonObject(index).mapTo(BookSource::class.java) }.getOrNull()
            if (source?.bookSourceUrl == requested) return source
        }
        return null
    }

    fun cookieScope(sourceUrl: String): String? = NetworkUtils.getSubDomain(sourceUrl)
        .trim().lowercase().takeIf { it.isNotEmpty() }

    fun sharesCookieScope(left: String, right: String): Boolean =
        cookieScope(left)?.let { it == cookieScope(right) } ?: false

    fun affectedSourceUrls(target: String, candidates: Collection<String>): List<String> =
        candidates.filter { sharesCookieScope(target, it) }

    fun redactCookie(cookie: String): String {
        val pairs = cookie.split(';').mapNotNull { pair ->
            val key = pair.substringBefore('=').trim()
            if (key.isBlank() || !pair.contains('=')) null else "$key=***"
        }
        return pairs.joinToString("; ").ifBlank { "已保存" }.take(256)
    }
}
