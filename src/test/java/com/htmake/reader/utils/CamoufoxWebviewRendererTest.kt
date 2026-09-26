package com.htmake.reader.utils

import com.sun.net.httpserver.HttpServer
import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
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
import java.util.concurrent.atomic.AtomicInteger

/**
 * End-to-end contract checks for the packaged fingerprint engine. The hosted browser-image
 * workflow supplies the pinned Camoufox runtime; ordinary unit-test runs skip this class.
 */
class CamoufoxWebviewRendererTest {
    @get:Rule val temp = TemporaryFolder()

    private lateinit var renderer: CamoufoxWebviewRenderer
    private lateinit var server: HttpServer
    private lateinit var baseUrl: String
    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface
    private val mediaHits = AtomicInteger()

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
                "/seed" -> {
                    exchange.responseHeaders.add("Set-Cookie", "session=alpha==; Path=/; HttpOnly")
                    respond(exchange, "seeded", "text/plain; charset=utf-8")
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
    fun cookiesArePersistedPerReaderNamespace() = runBlocking {
        renderer.render(request("/seed", "alice"))
        val alice = renderer.render(request("/echo", "alice"))
        val bob = renderer.render(request("/echo", "bob"))
        assertTrue(alice.body?.contains("session=alpha==") == true)
        assertTrue(bob.body?.endsWith("||") == true)
    }

    @Test
    fun sourceRegexReturnsTheResourceUrlWithoutFetchingIt() = runBlocking {
        val result = renderer.render(request("/resource-page", "reader", sourceRegex = ".*/media$"))
        assertEquals("$baseUrl/media", result.body)
        assertEquals(0, mediaHits.get())
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
