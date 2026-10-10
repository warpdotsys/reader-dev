package com.htmake.reader.utils

import com.google.gson.Gson
import com.google.common.net.InternetDomainName
import io.legado.app.help.http.CookieStore
import io.legado.app.utils.NetworkUtils
import java.net.URI
import java.time.Instant
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale

/**
 * Cookie storage used by the short-lived managed browsers.
 *
 * CookieStore predates browser-backed rendering and deliberately stores a plain
 * `name=value` header per domain. That representation cannot retain cookie
 * scope, so it must not be used as a browser profile. This jar is kept in the
 * same user-scoped cache, but records the attributes needed to decide whether a
 * cookie may accompany a particular request.
 */
object BrowserCookieJar {
    const val STORAGE_KEY = "__reader_browser_cookie_jar_v1"

    data class Cookie(
        val name: String = "",
        val value: String = "",
        val domain: String = "",
        val path: String = "/",
        val hostOnly: Boolean = true,
        val secure: Boolean = false,
        val httpOnly: Boolean = false,
        val sameSite: String? = null,
        /** Unix epoch seconds; a negative value denotes a session cookie. */
        val expires: Double = -1.0,
        /** Set-Cookie deletion is an operation, never a persisted cookie. */
        val deleted: Boolean = false
    )

    private data class StoredJar(
        val version: Int = 2,
        val managedHosts: MutableSet<String> = linkedSetOf(),
        /** Reader's historical CookieStore is keyed by this broader legacy scope. */
        val managedLegacyScopes: MutableSet<String> = linkedSetOf(),
        /** User-entered legacy cookie headers, keyed by their Reader cookie scope. */
        val manualCookies: MutableMap<String, MutableMap<String, String>> = linkedMapOf(),
        val cookies: MutableList<Cookie> = mutableListOf()
    )

    private val gson = Gson()

    /**
     * Cookies valid for [url]. Historical flat CookieStore values are migrated only
     * until a browser response has managed this host. User-entered Cookie headers
     * are registered through [setManualCookies] and remain available afterwards.
     */
    fun cookiesForRequest(
        store: CookieStore,
        url: String,
        legacyCookieHeader: String = "",
        explicitCookieHeader: String = ""
    ): List<Cookie> {
        val request = request(url) ?: return emptyList()
        val jar = load(store)
        val browserCookies = jar.cookies.asSequence()
            .filterNot { it.deleted || isExpired(it) }
            .filter { matches(it, request) }
            .sortedByDescending { it.path.length }
            .toMutableList()
        val overrides = linkedMapOf<String, Cookie>()
        val legacyScopeManaged = request.host in jar.managedHosts ||
            (legacyScope(url)?.let(jar.managedLegacyScopes::contains) ?: false)
        if (!legacyScopeManaged) {
            // Existing book-source cookies remain usable during migration. They are
            // treated as host-only session cookies and never written back as a raw
            // Camoufox response jar, which prevents later Set-Cookie deletion from
            // being silently undone.
            legacyFlatCookies(store, url, request.host).forEach { (name, value) ->
                overrides[name] = Cookie(name = name, value = value, domain = request.host)
            }
            store.cookieToMap(legacyCookieHeader).forEach { (name, value) ->
                overrides[name] = Cookie(name = name, value = value, domain = request.host)
            }
        }
        // A manual source-login value remains a deliberate user override after a
        // legacy scope becomes browser-managed. Apply it after the one-time
        // migration so an old flat cache cannot silently replace a newer value.
        jar.manualCookies.forEach { (scope, cookies) ->
            if (domainMatches(request.host, scope)) {
                cookies.forEach { (name, value) ->
                    overrides[name] = Cookie(name = name, value = value, domain = scope, hostOnly = false)
                }
            }
        }
        // A request Cookie header is never persisted and wins over every durable
        // source, including a same-name user-entered manual cookie.
        store.cookieToMap(explicitCookieHeader).forEach { (name, value) ->
            overrides[name] = Cookie(name = name, value = value, domain = request.host)
        }
        return browserCookies.filter { it.name !in overrides } + overrides.values
    }

    /**
     * Browser entrypoint. It imports an old flat Reader cookie exactly once into
     * the structured jar before rendering. This is intentionally separate from
     * [cookiesForRequest]: an ordinary HTTP request must not turn its generated
     * Cookie header into an attribute-less browser cookie.
     */
    fun cookiesForBrowserRequest(
        store: CookieStore,
        url: String,
        legacyCookieHeader: String = "",
        explicitCookieHeader: String = ""
    ): List<Cookie> {
        migrateLegacyCookies(store, url, legacyCookieHeader)
        return cookiesForRequest(store, url, explicitCookieHeader = explicitCookieHeader)
    }

    /** Atomically promotes the legacy flat scope before it becomes browser-managed. */
    fun migrateLegacyCookies(store: CookieStore, url: String, legacyCookieHeader: String = "") {
        val request = request(url) ?: return
        val jar = load(store)
        if (request.host in jar.managedHosts) return
        val legacyScope = legacyScope(url)
        if (legacyScope != null && legacyScope !in jar.managedLegacyScopes) {
            val merged = LinkedHashMap<String, Cookie>()
            jar.cookies.asSequence().filterNot(::isExpired).forEach { put(merged, it) }
            legacyFlatCookies(store, url, request.host).forEach { (name, value) ->
                val cookie = Cookie(name = name, value = value, domain = request.host)
                merged.putIfAbsent(identity(cookie), cookie)
            }
            store.cookieToMap(legacyCookieHeader).forEach { (name, value) ->
                val cookie = Cookie(name = name, value = value, domain = request.host)
                merged.putIfAbsent(identity(cookie), cookie)
            }
            jar.cookies.clear()
            jar.cookies.addAll(merged.values)
            // A structured migration is now authoritative for this legacy scope.
            // Subsequent flat values can only be introduced through setManualCookies.
            jar.managedLegacyScopes.add(legacyScope)
            store.setCookie(STORAGE_KEY, gson.toJson(jar))
        }
    }

    /** Records a manual source-login header without weakening browser cookie scope. */
    fun setManualCookies(store: CookieStore, scope: String, cookieHeader: String) {
        val normalizedScope = normalizeDomain(scope) ?: return
        val jar = load(store)
        val values = store.cookieToMap(cookieHeader)
        if (values.isEmpty()) jar.manualCookies.remove(normalizedScope)
        else jar.manualCookies[normalizedScope] = LinkedHashMap(values)
        store.setCookie(STORAGE_KEY, gson.toJson(jar))
    }

    /** Merge browser response state and mark the response host as browser-managed. */
    fun merge(store: CookieStore, responseUrl: String, updates: Collection<Cookie>?) {
        val response = request(responseUrl) ?: return
        val jar = load(store)
        jar.managedHosts.add(response.host)
        legacyScope(responseUrl)?.let(jar.managedLegacyScopes::add)
        val merged = LinkedHashMap<String, Cookie>()
        jar.cookies.asSequence().filterNot(::isExpired).forEach { put(merged, it) }
        updates.orEmpty().forEach { raw ->
            val cookie = normalize(raw) ?: return@forEach
            if (!responseCanSet(cookie, response)) return@forEach
            val identity = identity(cookie)
            // A plain HTTP response must never replace or delete a same-scope
            // Secure cookie. This remains true even when the incoming header
            // itself omits the Secure attribute.
            if (response.scheme != "https" && merged[identity]?.secure == true) return@forEach
            if (cookie.deleted || isExpired(cookie)) merged.remove(identity) else merged[identity] = cookie.copy(deleted = false)
        }
        jar.cookies.clear()
        jar.cookies.addAll(merged.values)
        store.setCookie(STORAGE_KEY, gson.toJson(jar))
    }

    /**
     * Bridge ordinary HTTP responses into a browser-managed scope without
     * recreating CookieStore's unsafe flat response jar. Domain cookies are
     * accepted only after a public-suffix check and response-host validation;
     * all records retain Path/Secure/expiry semantics.
     */
    fun mergeResponseHeaders(store: CookieStore, responseUrl: String, headers: Collection<String>) {
        val updates = headers.mapNotNull { parseSetCookie(it, responseUrl) }
        if (updates.isNotEmpty()) merge(store, responseUrl, updates)
    }

    /** Compatibility name retained for callers compiled during the first recovery pass. */
    fun mergeHostOnlyResponseHeaders(store: CookieStore, responseUrl: String, headers: Collection<String>) =
        mergeResponseHeaders(store, responseUrl, headers)

    /** Test-only view. Callers receive only non-expired persisted browser cookies. */
    fun storedCookies(store: CookieStore): List<Cookie> = load(store).cookies.filterNot(::isExpired)

    /** Saved credentials, not proof that a particular URL or website is authenticated. */
    fun savedCookiesForSource(store: CookieStore, sourceUrl: String): List<Cookie> {
        val target = request(sourceUrl) ?: return emptyList()
        return storedCookies(store).filter {
            if (it.hostOnly) it.domain == target.host else domainMatches(target.host, it.domain)
        }
    }

    fun isNetscapeInput(text: String): Boolean = '\t' in text ||
        text.trimStart('\uFEFF', ' ', '\r', '\n').startsWith("# Netscape HTTP Cookie File")

    /** Seven TAB fields, including HttpOnly comments and empty values; never flatten metadata. */
    fun parseNetscapeCookies(sourceUrl: String, text: String): List<Cookie> {
        require(text.length <= 64 * 1024) { "Cookie 过长" }
        val target = request(sourceUrl) ?: throw IllegalArgumentException("书源地址无效")
        val records = mutableListOf<Cookie>()
        text.removePrefix("\uFEFF").lineSequence().forEachIndexed { index, raw ->
            if (raw.isBlank() || raw.startsWith('#') && !raw.startsWith("#HttpOnly_")) return@forEachIndexed
            fun invalid(detail: String): Nothing =
                throw IllegalArgumentException("Netscape Cookie 第 ${index + 1} 行：$detail")
            if (records.size >= 512) invalid("记录过多")
            val httpOnly = raw.startsWith("#HttpOnly_")
            val fields = raw.removePrefix("#HttpOnly_").split('\t')
            if (fields.size != 7) invalid("需要 7 个制表符分隔字段")
            fun flag(value: String): Boolean = when (value) {
                "TRUE" -> true
                "FALSE" -> false
                else -> invalid("布尔字段必须为 TRUE 或 FALSE")
            }
            val domain = normalizeDomain(fields[0]) ?: invalid("域名无效")
            val hostOnly = !flag(fields[1])
            val secure = flag(fields[3])
            val expires = fields[4].toLongOrNull()?.takeIf { it >= 0 }
                ?: invalid("有效期必须为非负整数")
            val path = fields[2]
            val name = fields[5]
            val value = fields[6]
            if (!path.startsWith('/') || path.any { it <= ' ' || it == ';' || it == '\u007f' }) invalid("路径无效")
            if (name.isEmpty() || name.any { it !in '!'..'~' || it in "()<>@,;:\\\"/[]?={} " }) invalid("名称无效")
            if (value.any { it < ' ' || it == ';' || it == '\u007f' }) invalid("值包含非法字符")
            val cookie = Cookie(name, value, domain, path, hostOnly, secure, httpOnly,
                expires = if (expires == 0L) -1.0 else expires.toDouble())
            // A user-imported Secure cookie may originate from an HTTP source URL,
            // but it must only be sent over HTTPS later. All other browser scope
            // and prefix rules remain the same; no network request occurs here.
            if (!responseCanSet(cookie, target.copy(scheme = "https"))) invalid("域名或安全属性与书源不匹配")
            records.add(cookie)
        }
        require(records.isNotEmpty()) { "Netscape Cookie 没有可导入记录" }
        return records
    }

    /** Replace credentials applicable to this source host, without a flat-cookie copy. */
    fun replaceImportedCookies(store: CookieStore, sourceUrl: String, records: List<Cookie>): Int {
        val target = request(sourceUrl) ?: throw IllegalArgumentException("书源地址无效")
        require(records.isNotEmpty() && records.size <= 512) { "Cookie 记录数量无效" }
        val validated = records.map { raw ->
            val cookie = normalize(raw) ?: throw IllegalArgumentException("Cookie 记录无效")
            require(cookie == raw && responseCanSet(cookie, target.copy(scheme = "https")) &&
                cookie.value.none { it < ' ' || it == ';' || it == '\u007f' }) { "Cookie 安全属性无效" }
            cookie
        }
        val jar = load(store)
        jar.cookies.removeAll {
            if (it.hostOnly) it.domain == target.host else domainMatches(target.host, it.domain)
        }
        jar.manualCookies.keys.removeAll { domainMatches(target.host, it) }
        jar.managedHosts.add(target.host)
        // Legacy URL parsing can collapse example.co.uk to the public suffix
        // co.uk. Make only this host authoritative in that case, rather than
        // suppressing unrelated sites' legacy cookies in the same namespace.
        legacyScope(sourceUrl)?.takeIf(::isRegistrableDomain)?.let(jar.managedLegacyScopes::add)
        val merged = linkedMapOf<String, Cookie>()
        jar.cookies.filterNot(::isExpired).forEach { put(merged, it) }
        validated.forEach { if (isExpired(it)) merged.remove(identity(it)) else put(merged, it) }
        jar.cookies.clear()
        jar.cookies.addAll(merged.values)
        store.setCookie(STORAGE_KEY, gson.toJson(jar))
        // Old flat credentials must not override or revive the imported records.
        // Never remove a public-suffix legacy key shared by unrelated domains.
        legacyScope(sourceUrl)?.takeIf(::isRegistrableDomain)?.let { scope ->
            store.removeCookie(scope)
            store.removeCookie("${scope}_cookieJar")
        }
        return validated.map(::identity).toSet().count { key -> merged.containsKey(key) }
    }

    /**
     * Revokes browser cookies for a legacy Reader cookie scope (for example
     * `example.org`). Source-login clearing historically has this scope, not a
     * full URL, so include the registrable scope itself and its subdomains.
     */
    fun clearCookieScope(store: CookieStore, scope: String) {
        val normalizedScope = normalizeDomain(scope) ?: return
        val jar = load(store)
        jar.cookies.removeAll { cookie ->
            cookie.domain == normalizedScope || cookie.domain.endsWith(".$normalizedScope")
        }
        jar.manualCookies.keys.removeAll { scope ->
            scope == normalizedScope || scope.endsWith(".$normalizedScope")
        }
        jar.managedHosts.removeAll { host -> host == normalizedScope || host.endsWith(".$normalizedScope") }
        jar.managedLegacyScopes.remove(normalizedScope)
        store.setCookie(STORAGE_KEY, gson.toJson(jar))
    }

    private fun load(store: CookieStore): StoredJar = runCatching {
        gson.fromJson(store.getCookie(STORAGE_KEY), StoredJar::class.java)
    }.getOrNull() ?: StoredJar()

    private fun request(url: String): RequestTarget? = runCatching {
        val uri = URI(url)
        val scheme = uri.scheme?.lowercase(Locale.ROOT) ?: return null
        val host = normalizeDomain(uri.host ?: return null) ?: return null
        if (scheme != "http" && scheme != "https") return null
        RequestTarget(scheme, host, uri.rawPath?.takeIf { it.isNotEmpty() } ?: "/")
    }.getOrNull()

    private fun normalize(raw: Cookie): Cookie? {
        val name = raw.name.trim()
        val domain = normalizeDomain(raw.domain) ?: return null
        if (name.isEmpty() || name.any { it <= ' ' || it == ';' || it == '=' }) return null
        val path = raw.path.takeIf { it.startsWith('/') } ?: "/"
        val sameSite = when {
            raw.sameSite.equals("Lax", true) -> "Lax"
            raw.sameSite.equals("Strict", true) -> "Strict"
            raw.sameSite.equals("None", true) -> "None"
            else -> null
        }
        return raw.copy(name = name, domain = domain, path = path, sameSite = sameSite)
    }

    private fun normalizeDomain(value: String): String? {
        val normalized = value.trim().trimStart('.').trimEnd('.').lowercase(Locale.ROOT)
        return normalized.takeIf { it.isNotEmpty() && it.none { c -> c <= ' ' || c == '/' || c == ';' } }
    }

    private fun legacyScope(url: String): String? = normalizeDomain(NetworkUtils.getSubDomain(url))

    private fun responseCanSet(cookie: Cookie, response: RequestTarget): Boolean {
        if (cookie.secure && response.scheme != "https") return false
        if (cookie.name.startsWith("__Secure-") && (!cookie.secure || response.scheme != "https")) return false
        if (cookie.name.startsWith("__Host-") &&
            (!cookie.secure || !cookie.hostOnly || cookie.path != "/" || response.scheme != "https")
        ) return false
        return if (cookie.hostOnly) cookie.domain == response.host
        else isRegistrableDomain(cookie.domain) && domainMatches(response.host, cookie.domain)
    }

    private fun parseSetCookie(header: String, responseUrl: String): Cookie? {
        val response = request(responseUrl) ?: return null
        val parts = header.split(';')
        val pair = parts.firstOrNull()?.trim().orEmpty()
        val separator = pair.indexOf('=')
        if (separator <= 0) return null
        val name = pair.substring(0, separator).trim()
        if (name.any { it <= ' ' || it == ';' || it == '=' }) return null
        val value = pair.substring(separator + 1).trim()
        val attributes = linkedMapOf<String, String>()
        val flags = linkedSetOf<String>()
        parts.drop(1).forEach { part ->
            val item = part.trim()
            val index = item.indexOf('=')
            if (index < 0) flags.add(item.lowercase(Locale.ROOT))
            else attributes[item.substring(0, index).trim().lowercase(Locale.ROOT)] = item.substring(index + 1).trim()
        }
        val requestedDomain = attributes["domain"]
        val hostOnly = requestedDomain == null
        val domain = if (hostOnly) response.host else normalizeDomain(requestedDomain ?: return null)
        if (domain == null || (!hostOnly && (!domainMatches(response.host, domain) || !isRegistrableDomain(domain)))) {
            return null
        }
        val path = attributes["path"]?.takeIf { it.startsWith('/') } ?: defaultPath(response.path)
        val maxAge = attributes["max-age"]?.toLongOrNull()
        val expires = when {
            maxAge != null -> if (maxAge <= 0) 0.0 else System.currentTimeMillis() / 1000.0 + maxAge
            attributes.containsKey("expires") -> runCatching {
                ZonedDateTime.parse(attributes.getValue("expires"), DateTimeFormatter.RFC_1123_DATE_TIME)
                    .toInstant().epochSecond.toDouble()
            }.getOrDefault(-1.0)
            else -> -1.0
        }
        val deleted = maxAge != null && maxAge <= 0 || (expires >= 0 && expires <= Instant.now().epochSecond)
        return normalize(Cookie(
            name = name,
            value = value,
            domain = domain,
            path = path,
            hostOnly = hostOnly,
            secure = "secure" in flags,
            httpOnly = "httponly" in flags,
            sameSite = attributes["samesite"],
            expires = expires,
            deleted = deleted
        ))?.takeIf { responseCanSet(it, response) }
    }

    private fun defaultPath(responsePath: String): String {
        if (!responsePath.startsWith('/') || responsePath.count { it == '/' } <= 1) return "/"
        return responsePath.substringBeforeLast('/').ifEmpty { "/" }
    }

    /**
     * Reader historically keyed URL writes by NetworkUtils.getSubDomain(url),
     * while a few callers wrote a bare host. Read both during the one-time
     * migration; the narrower bare-host key wins where names collide.
     */
    private fun legacyFlatCookies(store: CookieStore, url: String, host: String): MutableMap<String, String> {
        val values = LinkedHashMap<String, String>()
        values.putAll(store.cookieToMap(store.getCookie(url)))
        values.putAll(store.cookieToMap(store.getCookie(host)))
        return values
    }

    private fun isRegistrableDomain(domain: String): Boolean = runCatching {
        // An unrecognized test/private TLD is not itself a known public suffix.
        // Reject known suffixes such as com/co.uk without discarding otherwise
        // valid Domain cookies that the browser can accept for example.test.
        !InternetDomainName.from(domain).isPublicSuffix
    }.getOrDefault(false)

    private fun matches(cookie: Cookie, request: RequestTarget): Boolean {
        if (cookie.secure && request.scheme != "https") return false
        val domainMatches = if (cookie.hostOnly) cookie.domain == request.host
        else domainMatches(request.host, cookie.domain)
        return domainMatches && pathMatches(request.path, cookie.path)
    }

    private fun domainMatches(host: String, domain: String): Boolean = host == domain || host.endsWith(".$domain")

    private fun pathMatches(requestPath: String, cookiePath: String): Boolean {
        if (requestPath == cookiePath) return true
        if (!requestPath.startsWith(cookiePath)) return false
        return cookiePath.endsWith('/') || requestPath.getOrNull(cookiePath.length) == '/'
    }

    private fun isExpired(cookie: Cookie): Boolean = cookie.deleted || (cookie.expires >= 0 && cookie.expires <= System.currentTimeMillis() / 1000.0)

    private fun identity(cookie: Cookie): String = "${cookie.name}\u0000${cookie.domain}\u0000${cookie.path}"

    private fun put(target: MutableMap<String, Cookie>, cookie: Cookie) {
        target[identity(cookie)] = cookie
    }

    private data class RequestTarget(val scheme: String, val host: String, val path: String)
}
