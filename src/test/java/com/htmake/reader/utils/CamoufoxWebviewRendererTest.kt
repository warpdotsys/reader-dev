package com.htmake.reader.utils

import com.sun.net.httpserver.HttpServer
import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.help.http.CookieStore
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

/**
 * End-to-end contract checks for the packaged fingerprint engine. The hosted browser-image
 * workflow supplies the pinned Camoufox runtime; ordinary unit-test runs skip this class.
 */
class CamoufoxWebviewRendererTest {
    @get:Rule val temp = TemporaryFolder()

    private lateinit var renderer: CamoufoxWebviewRenderer
    private lateinit var server: HttpServer
    private lateinit var serverWorkers: ExecutorService
    private lateinit var baseUrl: String
    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface
    private val mediaHits = AtomicInteger()
    private val slowPageHits = AtomicInteger()
    private val unscopedProbeHits = AtomicInteger()
    private val unscopedProbeCookie = AtomicReference("")

    @Before
    fun setUp() {
        val python = System.getenv("READER_CAMOUFOX_PYTHON") ?: ""
        assumeTrue(python.isNotBlank() && Files.isRegularFile(Paths.get(python)))
        assumeTrue(System.getenv("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD") == "1")

        originalUserDir = System.getProperty("user.dir")
        originalAdapter = ReaderAdapterHelper.getAdapter()
        System.setProperty("user.dir", temp.root.absolutePath)
        ReaderAdapterHelper.setAdapter(DefaultAdpater())
        val version = System.getenv("READER_CAMOUFOX_BROWSER_VERSION") ?: "152.0.4-beta.30"
        renderer = CamoufoxWebviewRenderer(python, version, 20_000, allowPrivateNetworks = true)

        server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        serverWorkers = Executors.newCachedThreadPool { task ->
            Thread(task, "reader-camoufox-fixture").apply { isDaemon = true }
        }
        server.executor = serverWorkers
        server.createContext("/") { exchange ->
            val requestBody = exchange.requestBody.use { it.readBytes().toString(StandardCharsets.UTF_8) }
            val cookie = exchange.requestHeaders.getFirst("Cookie") ?: ""
            when (exchange.requestURI.path) {
                "/resource-page" -> {
                    val html = "<html><body><div id='resource-result'></div>" +
                        "<script src='/asset.js'></script><img src='/media'></body></html>"
                    respond(exchange, html, "text/html; charset=utf-8")
                }
                "/asset.js" -> respond(
                    exchange,
                    "document.querySelector('#resource-result').textContent='asset-loaded'",
                    "application/javascript; charset=utf-8"
                )
                "/media" -> {
                    mediaHits.incrementAndGet()
                    respond(exchange, "media", "text/plain; charset=utf-8")
                }
                "/slow-page" -> {
                    slowPageHits.incrementAndGet()
                    try {
                        Thread.sleep(10_000)
                        respond(exchange, "late-page", "text/html; charset=utf-8")
                    } catch (_: InterruptedException) {
                        Thread.currentThread().interrupt()
                        exchange.close()
                    }
                }
                "/seed" -> {
                    exchange.responseHeaders.add("Set-Cookie", "session=alpha==; Path=/; HttpOnly")
                    respond(exchange, "seeded", "text/plain; charset=utf-8")
                }
                "/seed-scoped" -> {
                    exchange.responseHeaders.add("Set-Cookie", "scoped=only; Path=/scoped; HttpOnly")
                    respond(exchange, "seeded", "text/plain; charset=utf-8")
                }
                "/scoped/resource-page" -> respond(
                    exchange,
                    "<html><body><div id='main-cookie'>$cookie</div><script src='/unscoped-probe'></script></body></html>",
                    "text/html; charset=utf-8"
                )
                "/unscoped-probe" -> {
                    unscopedProbeCookie.set(cookie)
                    unscopedProbeHits.incrementAndGet()
                    respond(exchange, ";", "application/javascript; charset=utf-8")
                }
                "/echo" -> respond(
                    exchange,
                    "${exchange.requestMethod}|$requestBody|$cookie|${exchange.requestHeaders.getFirst("X-Reader-Probe") ?: ""}",
                    "text/plain; charset=utf-8"
                )
                else -> respond(exchange, "<html><body>empty</body></html>", "text/html; charset=utf-8")
            }
        }
        server.start()
        baseUrl = "http://127.0.0.1:${server.address.port}"
    }

    @After
    fun tearDown() {
        if (::renderer.isInitialized) runBlocking { renderer.close() }
        if (::server.isInitialized) server.stop(0)
        if (::serverWorkers.isInitialized) serverWorkers.shutdownNow()
        if (::originalAdapter.isInitialized) ReaderAdapterHelper.setAdapter(originalAdapter)
        if (::originalUserDir.isInitialized) System.setProperty("user.dir", originalUserDir)
    }

    @Test
    fun getPostScriptsAndSubresourcesUseTheBrowser() = runBlocking {
        val get = renderer.render(request("/echo", "user-a", headers = mapOf("X-Reader-Probe" to "header-ok")))
        assertTrue(get.body?.contains("GET|||header-ok") == true)

        val post = renderer.render(request("/echo", "user-a", post = true, body = "page=2"))
        assertTrue(post.body?.contains("POST|page=2|") == true)

        val script = renderer.render(request(
            "/resource-page",
            "user-a",
            javaScript = "document.querySelector('#resource-result').textContent"
        ))
        assertEquals("asset-loaded", script.body)
    }

    @Test
    fun javaScriptStructuredResultsUseTheArchivedWebviewResponseFormat() = runBlocking {
        val cases = listOf(
            "({answer: 42, ready: true})" to "{\"answer\":42,\"ready\":true}",
            "['alpha', 7]" to "[\"alpha\",7]",
            "42" to "42"
        )
        for ((expression, expected) in cases) {
            val result = renderer.render(request("/resource-page", "script-types", javaScript = expression))
            assertEquals("JavaScript expression $expression", expected, result.body)
        }
    }

    @Test
    fun cookiesArePersistedPerReaderNamespace() = runBlocking {
        renderer.render(request("/seed", "alice"))
        val aliceJar = BrowserCookieJar.storedCookies(CookieStore("alice"))
        assertEquals("The synthetic Set-Cookie must be persisted for Alice", "alpha==", aliceJar.single { it.name == "session" }.value)
        assertTrue("Bob must not inherit Alice's synthetic Cookie", BrowserCookieJar.storedCookies(CookieStore("bob")).isEmpty())
        val alice = renderer.render(request("/echo", "alice"))
        val bob = renderer.render(request("/echo", "bob"))
        assertTrue("Alice's next synthetic echo was: ${alice.body}", alice.body?.contains("session=alpha==") == true)
        // Firefox displays text/plain navigation inside an HTML <pre>; verify
        // the fixture content, not the browser's presentation wrapper.
        assertTrue("Bob's synthetic echo was: ${bob.body}", bob.body?.contains("GET|||") == true)
        assertFalse("Bob inherited Alice's Cookie", bob.body?.contains("session=alpha==") == true)
    }

    @Test
    fun importedHostOnlyCookieRetainsPathForSubresources() = runBlocking {
        renderer.render(request("/seed-scoped", "scoped-user"))
        val stored = BrowserCookieJar.storedCookies(CookieStore("scoped-user"))
        assertEquals("/scoped", stored.single { it.name == "scoped" }.path)

        val page = renderer.render(request("/scoped/resource-page", "scoped-user"))
        assertTrue("Scoped navigation lost its Cookie", page.body?.contains("scoped=only") == true)
        assertEquals("The cross-path script request must complete", 1, unscopedProbeHits.get())
        assertFalse("A /scoped Cookie leaked to /unscoped-probe", unscopedProbeCookie.get().contains("scoped=only"))
    }

    @Test
    fun sourceRegexReturnsTheResourceUrlWithoutFetchingIt() = runBlocking {
        val result = renderer.render(request("/resource-page", "reader", sourceRegex = ".*/media$"))
        assertEquals("$baseUrl/media", result.body)
        assertEquals(0, mediaHits.get())
    }

    @Test
    fun unmatchedSourceRegexTimesOutAndTheNextRenderRecovers() = runBlocking {
        val python = System.getenv("READER_CAMOUFOX_PYTHON") ?: error("Camoufox Python is required")
        val version = System.getenv("READER_CAMOUFOX_BROWSER_VERSION") ?: "152.0.4-beta.30"
        val limited = CamoufoxWebviewRenderer(python, version, 3_000, allowPrivateNetworks = true)
        try {
            val started = System.nanoTime()
            val failure = runCatching {
                limited.render(request("/resource-page", "timeout-user", sourceRegex = ".*/never$"))
            }.exceptionOrNull()
            val elapsedMs = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - started)
            assertTrue("An absent resource must time out: $failure", failure is IllegalStateException)
            assertTrue(
                "An absent resource returned an unrelated failure: ${failure?.message}",
                failure?.message?.contains("TimeoutError") == true || failure?.message?.contains("超时") == true
            )
            assertTrue("The browser timeout exceeded the bounded parent watchdog: ${elapsedMs}ms", elapsedMs < 30_000)

            // The timeout must not poison the single-render queue before a
            // subsequent request starts. Process-count checks run in CI separately.
            val healthy = limited.render(request("/echo", "timeout-user"))
            assertTrue("A healthy request after timeout failed: ${healthy.body}", healthy.body?.contains("GET|||") == true)
        } finally {
            limited.close()
        }
    }

    @Test
    fun stalledMainNavigationFailsInsteadOfReturningProxyErrorPage() = runBlocking {
        val python = System.getenv("READER_CAMOUFOX_PYTHON") ?: error("Camoufox Python is required")
        val version = System.getenv("READER_CAMOUFOX_BROWSER_VERSION") ?: "152.0.4-beta.30"
        val limited = CamoufoxWebviewRenderer(python, version, 3_000, allowPrivateNetworks = true)
        try {
            val started = System.nanoTime()
            val outcome = runCatching { limited.render(request("/slow-page", "slow-navigation-user")) }
            val elapsedMs = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - started)
            assertTrue("The slow main document was not actually requested", slowPageHits.get() > 0)
            assertTrue("Main navigation exceeded its bounded watchdog: ${elapsedMs}ms", elapsedMs < 30_000)
            assertTrue(
                "A stalled main document returned an apparent success: ${outcome.getOrNull()?.body?.take(120)}",
                outcome.exceptionOrNull() is IllegalStateException
            )
            val healthy = limited.render(request("/echo", "slow-navigation-user"))
            assertTrue("A healthy request after a stalled page failed", healthy.body?.contains("GET|||") == true)
        } finally {
            limited.close()
        }
    }

    private fun request(
        path: String,
        user: String,
        headers: Map<String, String>? = null,
        post: Boolean = false,
        body: String? = null,
        javaScript: String? = null,
        sourceRegex: String? = null
    ) = WebviewRequest(
        url = baseUrl + path,
        html = null,
        encode = null,
        tag = null,
        headerMap = headers,
        sourceRegex = sourceRegex,
        javaScript = javaScript,
        proxy = null,
        post = post,
        body = body,
        userNameSpace = user,
        debugLog = null
    )

    private fun respond(exchange: com.sun.net.httpserver.HttpExchange, body: String, contentType: String) {
        val data = body.toByteArray(StandardCharsets.UTF_8)
        exchange.responseHeaders.add("Content-Type", contentType)
        exchange.sendResponseHeaders(200, data.size.toLong())
        exchange.responseBody.use { it.write(data) }
    }
}
