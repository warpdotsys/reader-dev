package com.htmake.reader.utils

import com.sun.net.httpserver.HttpServer
import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Paths
import java.net.InetAddress
import java.util.concurrent.atomic.AtomicInteger

class LocalWebviewRendererTest {
    @get:Rule val temp = TemporaryFolder()
    private lateinit var server: HttpServer
    private lateinit var renderer: LocalWebviewRenderer
    private lateinit var baseUrl: String
    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface
    private val privateRedirectHits = AtomicInteger()
    private val matchedResourceHits = AtomicInteger()

    @Before
    fun setUp() {
        val executable = System.getenv("READER_BROWSER_EXECUTABLE") ?: ""
        assumeTrue(executable.isNotBlank() && Files.isRegularFile(Paths.get(executable)))
        assumeTrue(System.getenv("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD") == "1")
        originalUserDir = System.getProperty("user.dir")
        originalAdapter = ReaderAdapterHelper.getAdapter()
        System.setProperty("user.dir", temp.root.absolutePath)
        ReaderAdapterHelper.setAdapter(DefaultAdpater())
        renderer = LocalWebviewRenderer(executable, 5000, allowPrivateNetworks = true)
        server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/") { exchange ->
            if (exchange.requestURI.path == "/redirect-to-private") {
                exchange.responseHeaders.add("Location", "http://localhost:${server.address.port}/target")
                exchange.sendResponseHeaders(302, -1)
                exchange.close()
                return@createContext
            }
            if (exchange.requestURI.path == "/target") privateRedirectHits.incrementAndGet()
            if (exchange.requestURI.path == "/asset.js") {
                val script = "document.querySelector('#resource-result').textContent='asset-loaded'"
                    .toByteArray(StandardCharsets.UTF_8)
                exchange.responseHeaders.add("Content-Type", "application/javascript; charset=UTF-8")
                exchange.sendResponseHeaders(200, script.size.toLong())
                exchange.responseBody.use { it.write(script) }
                return@createContext
            }
            if (exchange.requestURI.path == "/resource-page") {
                val html = "<html><body><div id='resource-result'></div><script src='/asset.js'></script></body></html>"
                    .toByteArray(StandardCharsets.UTF_8)
                exchange.responseHeaders.add("Content-Type", "text/html; charset=UTF-8")
                exchange.sendResponseHeaders(200, html.size.toLong())
                exchange.responseBody.use { it.write(html) }
                return@createContext
            }
            if (exchange.requestURI.path == "/events") {
                exchange.responseHeaders.add("Content-Type", "text/event-stream; charset=UTF-8")
                exchange.responseHeaders.add("Cache-Control", "no-cache")
                exchange.sendResponseHeaders(200, 0)
                exchange.responseBody.use { stream ->
                    stream.write("data: first\n\n".toByteArray(StandardCharsets.UTF_8))
                    stream.flush()
                    Thread.sleep(150)
                    stream.write("data: second\n\n".toByteArray(StandardCharsets.UTF_8))
                    stream.flush()
                }
                return@createContext
            }
            if (exchange.requestURI.path == "/media") matchedResourceHits.incrementAndGet()
            val body = exchange.requestBody.readBytes().toString(StandardCharsets.UTF_8)
            val cookie = exchange.requestHeaders.getFirst("Cookie") ?: ""
            val marker = "${exchange.requestMethod}|$body|$cookie"
            if (exchange.requestURI.path == "/seed") {
                exchange.responseHeaders.add("Set-Cookie", "sid=alpha; Path=/; HttpOnly")
            }
            exchange.responseHeaders.add("Content-Type", "text/html; charset=UTF-8")
            val html = "<html><body><div id='result'>$marker</div></body></html>"
                .toByteArray(StandardCharsets.UTF_8)
            exchange.sendResponseHeaders(200, html.size.toLong())
            exchange.responseBody.use { it.write(html) }
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

    private fun request(path: String, user: String, post: Boolean = false, body: String? = null,
                        script: String? = null, regex: String? = null): WebviewRequest = WebviewRequest(
        url = baseUrl + path, html = null, encode = null, tag = null,
        headerMap = null, sourceRegex = regex, javaScript = script, proxy = null,
        post = post, body = body, userNameSpace = user, debugLog = null
    )

    @Test
    fun getAndPostExecuteInBrowser() = runBlocking {
        val get = renderer.render(request("/echo", "reader-a", script = "document.querySelector('#result').textContent"))
        val post = renderer.render(request("/echo", "reader-a", post = true, body = "page=2",
            script = "document.querySelector('#result').textContent"))
        assertEquals("GET||", get.body)
        assertTrue(post.body!!.startsWith("POST|page=2|"))
    }

    @Test
    fun cookiesRemainInTheUserNamespace() = runBlocking {
        renderer.render(request("/seed", "alice"))
        val alice = renderer.render(request("/echo", "alice", script = "document.querySelector('#result').textContent"))
        val bob = renderer.render(request("/echo", "bob", script = "document.querySelector('#result').textContent"))
        assertTrue(alice.body!!.contains("sid=alpha"))
        assertFalse(bob.body!!.contains("sid=alpha"))
    }

    @Test
    fun javascriptSubresourcesUseTheGuardedProxy() = runBlocking {
        val response = renderer.render(request("/resource-page", "reader-a",
            script = "document.querySelector('#resource-result').textContent"))
        assertEquals("asset-loaded", response.body)
    }

    @Test
    fun chunkedEventStreamsReachThePageBeforeTheOriginCloses() = runBlocking {
        val response = renderer.render(request("/resource-page", "reader-a", script =
            "new Promise(resolve => { const events = []; const source = new EventSource('/events'); " +
                "source.onmessage = event => { events.push(event.data); if (events.length === 2) { " +
                "source.close(); resolve(events.join(',')); } }; })"))
        assertEquals("first,second", response.body)
    }

    @Test
    fun sourceRegexReturnsTheFirstMatchingResourceUrlAndAbortsItsFetch() = runBlocking {
        val resourceUrl = "$baseUrl/media?token=alpha"
        val response = renderer.render(request("/echo", "reader-a",
            script = "fetch('$resourceUrl').catch(() => {})",
            regex = Regex.escape(resourceUrl)))

        assertEquals(resourceUrl, response.body)
        assertEquals("A sniffed source should not be downloaded into the origin fixture", 0, matchedResourceHits.get())
    }

    @Test
    fun htmlInputSupportsDeclaredNonUtf8Charset() = runBlocking {
        val html = "<html><body><div id='result'>中文字符集</div></body></html>"
        val response = renderer.render(request("/echo", "reader-a", script = "document.body.innerText")
            .copy(html = html, encode = "GBK"))

        assertTrue(response.body!!.contains("中文字符集"))
    }

    @Test
    fun deniesPrivateTargetsByDefaultBeforeLaunchingChromium() = runBlocking {
        val strictRenderer = LocalWebviewRenderer(
            System.getenv("READER_BROWSER_EXECUTABLE") ?: "", 5000, allowPrivateNetworks = false)
        try {
            strictRenderer.render(request("/echo", "reader-a"))
            fail("Loopback targets must be blocked by default")
        } catch (_: BrowserNetworkPolicyViolation) {
            // Expected: the request is rejected before browser startup.
        } finally {
            strictRenderer.close()
        }
    }

    @Test
    fun blocksPrivateRedirectsAtTheEgressProxy() = runBlocking {
        val policy = object : BrowserNetworkPolicy() {
            override fun resolveRequestTarget(value: String): BrowserNetworkTarget? {
                val uri = java.net.URI(value)
                if (uri.host == "127.0.0.1") {
                    return BrowserNetworkTarget(uri, "127.0.0.1", uri.port,
                        listOf(InetAddress.getByName("127.0.0.1")))
                }
                return super.resolveRequestTarget(value)
            }
        }
        val strictRenderer = LocalWebviewRenderer(
            System.getenv("READER_BROWSER_EXECUTABLE") ?: "", 5000, policy)
        try {
            strictRenderer.render(request("/redirect-to-private", "reader-a"))
            fail("The private redirect target must be rejected by the egress proxy")
        } catch (_: BrowserNetworkPolicyViolation) {
            // Expected: the first loopback fixture is allowed only by this test policy.
        } finally {
            strictRenderer.close()
        }
        assertEquals("The rejected redirect must never reach the fixture target", 0, privateRedirectHits.get())
    }

    @Test(expected = IllegalArgumentException::class)
    fun unknownHtmlCharsetFailsWithClearValidationError() {
        runBlocking {
            renderer.render(request("/echo", "reader-a")
                .copy(html = "<html></html>", encode = "not-a-real-charset"))
        }
    }
}
