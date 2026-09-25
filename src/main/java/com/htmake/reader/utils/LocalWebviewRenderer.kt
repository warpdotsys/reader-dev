package com.htmake.reader.utils

import com.microsoft.playwright.Browser
import com.microsoft.playwright.BrowserType
import com.microsoft.playwright.Page
import com.microsoft.playwright.Playwright
import com.microsoft.playwright.Route
import com.microsoft.playwright.options.Cookie
import com.microsoft.playwright.options.Proxy
import com.microsoft.playwright.options.ServiceWorkerPolicy
import com.microsoft.playwright.options.WaitUntilState
import io.legado.app.help.http.CookieStore
import io.legado.app.help.http.StrResponse
import io.legado.app.utils.NetworkUtils
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.withContext
import java.nio.charset.Charset
import java.nio.file.Paths
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * Opt-in browser capability baseline. No production default or fingerprint claim.
 * All Playwright calls run on one worker because its Java objects are not thread-safe.
 */
class LocalWebviewRenderer private constructor(
    private val executablePath: String,
    private val timeoutMs: Int,
    allowPrivateNetworks: Boolean,
    networkPolicyOverride: BrowserNetworkPolicy?
) : WebviewRenderer {
    constructor(
        executablePath: String = "",
        timeoutMs: Int = 20_000,
        allowPrivateNetworks: Boolean = System.getenv("READER_BROWSER_ALLOW_PRIVATE_NETWORKS")
            ?.equals("true", ignoreCase = true) == true
    ) : this(executablePath, timeoutMs, allowPrivateNetworks, null)

    internal constructor(
        executablePath: String,
        timeoutMs: Int,
        networkPolicy: BrowserNetworkPolicy
    ) : this(executablePath, timeoutMs, false, networkPolicy)

    private val pending = AtomicInteger(0)
    private val worker = Executors.newSingleThreadExecutor { task ->
        Thread(task, "reader-local-webview").apply { isDaemon = true }
    }.asCoroutineDispatcher()
    private val networkPolicy = networkPolicyOverride ?: BrowserNetworkPolicy(allowPrivateNetworks)
    private var playwright: Playwright? = null
    private var browser: Browser? = null

    override suspend fun render(request: WebviewRequest): StrResponse {
        if (pending.incrementAndGet() > 8) {
            pending.decrementAndGet()
            throw IllegalStateException("本地 WebView 请求队列已满")
        }
        try {
            return withContext(worker) { renderOnWorker(request) }
        } finally {
            pending.decrementAndGet()
        }
    }

    private fun renderOnWorker(request: WebviewRequest): StrResponse {
        val url = request.url ?: throw IllegalArgumentException("本地 WebView 需要 HTTP URL")
        networkPolicy.requireDocumentUrl(url)
        val sourcePattern = request.sourceRegex?.takeIf { it.isNotBlank() }?.let(::Regex)
        val html = request.html?.let { encodeHtmlForWebView(it, request.encode) }
        if (timeoutMs <= 0) throw IllegalArgumentException("browserTimeoutMs must be positive")
        if (System.getenv("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD") != "1") {
            throw IllegalStateException("本地 WebView 要求 PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1，禁止运行时下载浏览器")
        }
        val upstreamProxy = request.proxy?.takeIf { it.isNotBlank() }?.let(BrowserUpstreamProxy::parse)

        val activeBrowser = getBrowser()
        val blockedNetworkRequest = AtomicBoolean(false)
        val matchedSourceUrl = java.util.concurrent.atomic.AtomicReference<String?>(null)
        val egressProxy = BrowserEgressProxy(networkPolicy, timeoutMs,
            { blockedNetworkRequest.set(true) }, upstreamProxy)
        try {
            val proxyEndpoint = egressProxy.start()
            val contextOptions = Browser.NewContextOptions()
                .setAcceptDownloads(false)
                .setServiceWorkers(ServiceWorkerPolicy.BLOCK)
                .setProxy(Proxy(proxyEndpoint).setBypass("<-loopback>"))
            val headers = request.headerMap ?: emptyMap()
            headers.entries.firstOrNull { it.key.equals("User-Agent", ignoreCase = true) }
                ?.value?.let(contextOptions::setUserAgent)
            val context = activeBrowser.newContext(contextOptions)
            try {
                context.setDefaultTimeout(timeoutMs.toDouble())
                context.setDefaultNavigationTimeout(timeoutMs.toDouble())
                val domain = NetworkUtils.getSubDomain(url)
                val cookieStore = CookieStore(request.userNameSpace)
                val cookies = cookieStore.cookieToMap(cookieStore.getCookie(domain))
                headers.entries.firstOrNull { it.key.equals("Cookie", ignoreCase = true) }
                    ?.value?.let { cookies.putAll(cookieStore.cookieToMap(it)) }
                if (cookies.isNotEmpty()) {
                    context.addCookies(cookies.map { (name, value) -> Cookie(name, value).setUrl(url) })
                }
                val extraHeaders = headers.filterKeys { key ->
                    !key.equals("Cookie", ignoreCase = true) &&
                        !key.equals("User-Agent", ignoreCase = true) &&
                        !key.equals("Host", ignoreCase = true) &&
                        !key.equals("Content-Length", ignoreCase = true)
                }
                if (extraHeaders.isNotEmpty()) context.setExtraHTTPHeaders(extraHeaders)

                val postSent = AtomicBoolean(false)
                context.route("**/*") { route ->
                    try {
                        networkPolicy.requireRequestUrl(route.request().url())
                    } catch (_: BrowserNetworkPolicyViolation) {
                        blockedNetworkRequest.set(true)
                        route.abort()
                        return@route
                    }
                    val resourceUrl = route.request().url()
                    if (sourcePattern?.matches(resourceUrl) == true) {
                        matchedSourceUrl.compareAndSet(null, resourceUrl)
                        route.abort()
                        return@route
                    }
                    if (request.post && route.request().isNavigationRequest && route.request().url() == url &&
                        postSent.compareAndSet(false, true)) {
                        route.resume(Route.ResumeOptions().setMethod("POST").setPostData(request.body ?: ""))
                    } else route.resume()
                }
                val page = context.newPage()
                val body = try {
                    try {
                        page.navigate(url, Page.NavigateOptions()
                            .setWaitUntil(WaitUntilState.DOMCONTENTLOADED)
                            .setTimeout(timeoutMs.toDouble()))
                    } catch (e: RuntimeException) {
                        if (matchedSourceUrl.get() == null) throw e
                    }
                    failIfNetworkRequestBlocked(blockedNetworkRequest)
                    if (html != null && matchedSourceUrl.get() == null) {
                        page.setContent(html, Page.SetContentOptions()
                            .setWaitUntil(WaitUntilState.DOMCONTENTLOADED)
                            .setTimeout(timeoutMs.toDouble()))
                        failIfNetworkRequestBlocked(blockedNetworkRequest)
                    }
                    val result = if (sourcePattern != null) {
                        if (matchedSourceUrl.get() == null && !request.javaScript.isNullOrBlank()) {
                            try {
                                // Legacy WebView executes this as a javascript: URL; indirect eval
                                // runs in the page's global scope and intentionally ignores its result.
                                page.evaluate("(source) => { (0, eval)(source); }", request.javaScript)
                            } catch (e: RuntimeException) {
                                if (matchedSourceUrl.get() == null) throw e
                            }
                        }
                        val deadline = System.nanoTime() + timeoutMs * 1_000_000L
                        while (matchedSourceUrl.get() == null && System.nanoTime() < deadline) {
                            failIfNetworkRequestBlocked(blockedNetworkRequest)
                            val remainingMs = (deadline - System.nanoTime()) / 1_000_000L
                            if (remainingMs > 0) page.waitForTimeout(minOf(50.0, remainingMs.toDouble()))
                        }
                        matchedSourceUrl.get()
                            ?: throw IllegalStateException("本地 WebView 在 ${timeoutMs}ms 内未找到匹配 sourceRegex 的资源")
                    } else if (request.javaScript.isNullOrBlank()) page.content()
                        else page.evaluate(request.javaScript)?.toString() ?: ""
                    failIfNetworkRequestBlocked(blockedNetworkRequest)
                    result
                } catch (e: RuntimeException) {
                    if (blockedNetworkRequest.get()) {
                        throw BrowserNetworkPolicyViolation("本地 WebView 渲染触及了本机或非公网网络资源，已中止")
                    }
                    matchedSourceUrl.get()?.let { return@renderOnWorker StrResponse(url, it) }
                    throw e
                }

                if (domain.isNotEmpty()) {
                    val stored = context.cookies(url).joinToString(";") { "${it.name}=${it.value}" }
                    cookieStore.setCookie("${domain}_cookieJar", stored)
                    cookieStore.setCookie(domain, stored)
                }
                return StrResponse(url, body)
            } finally {
                context.close()
            }
        } finally {
            egressProxy.close()
        }
    }

    private fun failIfNetworkRequestBlocked(blocked: AtomicBoolean) {
        if (blocked.get()) {
            throw BrowserNetworkPolicyViolation("本地 WebView 渲染触及了本机或非公网网络资源，已中止")
        }
    }

    private fun encodeHtmlForWebView(html: String, encode: String?): String {
        if (encode.isNullOrBlank()) return html
        val charset = try {
            Charset.forName(encode)
        } catch (e: Exception) {
            throw IllegalArgumentException("不支持的 WebView HTML 字符集: $encode", e)
        }
        // Android loadDataWithBaseURL encodes the supplied String using the requested charset.
        // Playwright accepts an already-decoded String, so round-trip it to preserve the same
        // representable characters and charset replacement behavior.
        return String(html.toByteArray(charset), charset)
    }

    private fun getBrowser(): Browser {
        val running = browser
        if (running != null && running.isConnected) return running
        runCatching { running?.close() }
        runCatching { playwright?.close() }
        val driver = Playwright.create(Playwright.CreateOptions()
            .setEnv(mapOf("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD" to "1")))
        try {
            val options = BrowserType.LaunchOptions()
                .setHeadless(true)
                .setArgs(listOf(
                    "--disable-quic",
                    "--force-webrtc-ip-handling-policy=disable_non_proxied_udp",
                    "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1"
                ))
            if (executablePath.isNotBlank()) options.setExecutablePath(Paths.get(executablePath))
            val launched = driver.chromium().launch(options)
            playwright = driver
            browser = launched
            return launched
        } catch (e: Throwable) {
            driver.close()
            throw e
        }
    }

    suspend fun close() {
        try {
            withContext(worker) {
                runCatching { browser?.close() }
                runCatching { playwright?.close() }
                browser = null
                playwright = null
            }
        } finally {
            worker.close()
        }
    }
}
