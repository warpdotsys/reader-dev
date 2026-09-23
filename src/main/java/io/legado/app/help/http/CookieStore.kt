@file:Suppress("unused")

package io.legado.app.help.http

import io.legado.app.utils.TextUtils
import io.legado.app.data.entities.Cookie
import io.legado.app.help.http.api.CookieManager
import io.legado.app.utils.NetworkUtils
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.utils.ACache
import java.io.File
import java.time.Instant
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

class CookieStore(val userNameSpace: String) : CookieManager {

    val cacheInstance = ACache.get(
        File(ReaderAdapterHelper.getAdapter().getWorkDir("storage", "cache", "cookie", userNameSpace)),
        50_000_000L,
        1_000_000
    )

    override fun setCookie(url: String, cookie: String?) {
        val domain = cookieKey(url)
        if (domain.isNotEmpty()) cacheInstance.put(domain, cookie ?: "")
    }

    override fun replaceCookie(url: String, cookie: String) {
        if (TextUtils.isEmpty(url) || TextUtils.isEmpty(cookie)) {
            return
        }
        val oldCookie = getCookie(url)
        if (TextUtils.isEmpty(oldCookie)) {
            setCookie(url, cookie)
        } else {
            val cookieMap = cookieToMap(oldCookie)
            cookieMap.putAll(cookieToMap(cookie))
            val newCookie = mapToCookie(cookieMap)
            setCookie(url, newCookie)
        }
    }

    /** Store one Set-Cookie response header without forwarding its attributes as Cookie pairs. */
    fun replaceResponseCookie(url: String, setCookieHeader: String) {
        val parts = setCookieHeader.split(';')
        val cookiePair = parts.firstOrNull()?.trim() ?: return
        val separator = cookiePair.indexOf('=')
        if (separator <= 0) return
        val name = cookiePair.substring(0, separator).trim()
        if (name.isEmpty()) return
        val value = cookiePair.substring(separator + 1).trim()
        val attributes = parts.drop(1).map { it.trim() }
        val maxAge = attributes.firstOrNull { it.startsWith("Max-Age=", ignoreCase = true) }
            ?.substringAfter('=')?.trim()?.toLongOrNull()
        val expiredByDate = if (maxAge == null) {
            attributes.firstOrNull { it.startsWith("Expires=", ignoreCase = true) }
                ?.substringAfter('=')?.trim()?.let { expires ->
                    runCatching {
                        !ZonedDateTime.parse(expires, DateTimeFormatter.RFC_1123_DATE_TIME)
                            .toInstant().isAfter(Instant.now())
                    }.getOrDefault(false)
                } ?: false
        } else false
        if (value.isEmpty() || maxAge != null && maxAge <= 0 || expiredByDate) {
            removeResponseCookie(url, name)
            return
        }
        replaceCookie(url, cookiePair)
    }

    private fun removeResponseCookie(url: String, name: String) {
        val keys = if (url.endsWith("_cookieJar")) {
            listOf(url, url.removeSuffix("_cookieJar"))
        } else listOf(url)
        for (key in keys) {
            val cookies = cookieToMap(getCookie(key))
            if (cookies.remove(name) != null) setCookie(key, mapToCookie(cookies))
        }
    }

    override fun getCookie(url: String): String {
        val domain = cookieKey(url)
        return if (domain.isEmpty()) "" else cacheInstance.getAsString(domain) ?: ""
    }

    fun getKey(url: String, key: String): String = cookieToMap(getCookie(url))[key] ?: ""

    override fun removeCookie(url: String) {
        cookieKey(url).takeIf { it.isNotEmpty() }?.let(cacheInstance::remove)
    }

    /** Legacy callers also pass a bare domain or a domain suffixed with `_cookieJar`. */
    private fun cookieKey(urlOrDomain: String): String {
        val domain = NetworkUtils.getSubDomain(urlOrDomain)
        if (domain.isNotEmpty()) return domain
        if (urlOrDomain.isEmpty() ||
            urlOrDomain.any { !it.isLetterOrDigit() && it !in "._:-" }
        ) return ""
        return urlOrDomain
    }

    override fun cookieToMap(cookie: String): MutableMap<String, String> {
        val cookieMap = mutableMapOf<String, String>()
        if (cookie.isBlank()) {
            return cookieMap
        }
        for (pair in cookie.split(';')) {
            val separator = pair.indexOf('=')
            if (separator <= 0) continue
            val key = pair.substring(0, separator).trim()
            val value = pair.substring(separator + 1).trim()
            if (key.isEmpty()) continue
            if (value.isNotBlank() || value.trim { it <= ' ' } == "null") {
                cookieMap[key] = value
            }
        }
        return cookieMap
    }

    override fun mapToCookie(cookieMap: Map<String, String>?): String? {
        if (cookieMap == null || cookieMap.isEmpty()) {
            return null
        }
        val builder = StringBuilder()
        for (key in cookieMap.keys) {
            val value = cookieMap[key]
            if (value?.isNotBlank() == true) {
                builder.append(key)
                    .append("=")
                    .append(value)
                    .append(";")
            }
        }
        return builder.deleteCharAt(builder.lastIndexOf(";")).toString()
    }

    fun clear() {
        cacheInstance.clear()
    }

}
