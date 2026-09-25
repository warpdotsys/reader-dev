package com.htmake.reader.utils

import com.microsoft.playwright.Browser
import com.microsoft.playwright.BrowserType
import com.microsoft.playwright.Page
import com.microsoft.playwright.Playwright
import com.microsoft.playwright.Route
import com.microsoft.playwright.options.Cookie
import com.microsoft.playwright.options.Proxy
import com.microsoft.playwright.options.WaitUntilState
import io.legado.app.help.http.CookieStore
import io.legado.app.help.http.StrResponse
import io.legado.app.utils.NetworkUtils
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.withContext
import java.nio.file.Paths
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * Opt-in browser capability baseline. No production default or fingerprint claim.
 * All Playwright calls run on one worker because its Java objects are not thread-safe.
 */
class LocalWebviewRenderer(
    private val executablePath: String = "",
    private val timeoutMs: Int = 20_000
) : WebviewRenderer {
    private val pending = AtomicInteger(0)
    private val worker = Executors.newSingleThreadExecutor { task ->
        Thread(task, "reader-local-webview").apply { isDaemon = true }
    }.asCoroutineDispatcher()
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
        if (!url.startsWith("http://") && !url.startsWith("https://")) {
            throw IllegalArgumentException("本地 WebView 仅接受 HTTP/HTTPS URL")
        }
        if (!request.sourceRegex.isNullOrBlank()) {
            throw UnsupportedOperationException("本地 WebView 尚未验证 sourceRegex 语义；请使用远程实现")
        }
        if (!request.encode.isNullOrBlank() && !request.encode.equals("UTF-8", ignoreCase = true)) {
            throw UnsupportedOperationException("本地 WebView 尚未验证指定字符集；请使用远程实现")
        }
        if (timeoutMs <= 0) throw IllegalArgumentException("browserTimeoutMs must be positive")
        if (System.getenv("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD") != "1") {
            throw IllegalStateException("本地 WebView 要求 PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1，禁止运行时下载浏览器")
        }

        val activeBrowser = getBrowser()
        val contextOptions = Browser.NewContextOptions().setAcceptDownloads(false)
        if (!request.proxy.isNullOrBlank()) contextOptions.setProxy(Proxy(request.proxy))
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

            val page = context.newPage()
            if (request.post) {
                val sent = AtomicBoolean(false)
                page.route("**/*") { route ->
                    if (route.request().isNavigationRequest && route.request().url() == url &&
                        sent.compareAndSet(false, true)) {
                        route.resume(Route.ResumeOptions().setMethod("POST").setPostData(request.body ?: ""))
                    } else route.resume()
                }
            }
            page.navigate(url, Page.NavigateOptions()
                .setWaitUntil(WaitUntilState.DOMCONTENTLOADED)
                .setTimeout(timeoutMs.toDouble()))
            if (request.html != null) {
                page.setContent(request.html, Page.SetContentOptions()
                    .setWaitUntil(WaitUntilState.DOMCONTENTLOADED)
                    .setTimeout(timeoutMs.toDouble()))
            }
            val body = if (request.javaScript.isNullOrBlank()) page.content()
                else page.evaluate(request.javaScript)?.toString() ?: ""

            if (domain.isNotEmpty()) {
                val stored = context.cookies(url).joinToString(";") { "${it.name}=${it.value}" }
                cookieStore.setCookie("${domain}_cookieJar", stored)
                cookieStore.setCookie(domain, stored)
            }
            return StrResponse(url, body)
        } finally {
            context.close()
        }
    }

    private fun getBrowser(): Browser {
        val running = browser
        if (running != null && running.isConnected) return running
        runCatching { running?.close() }
        runCatching { playwright?.close() }
        val driver = Playwright.create(Playwright.CreateOptions()
            .setEnv(mapOf("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD" to "1")))
        try {
            val options = BrowserType.LaunchOptions().setHeadless(true)
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
