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
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Paths

class LocalWebviewRendererTest {
    @get:Rule val temp = TemporaryFolder()
    private lateinit var server: HttpServer
    private lateinit var renderer: LocalWebviewRenderer
    private lateinit var baseUrl: String
    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface

    @Before
    fun setUp() {
        val executable = System.getenv("READER_BROWSER_EXECUTABLE") ?: ""
        assumeTrue(executable.isNotBlank() && Files.isRegularFile(Paths.get(executable)))
        assumeTrue(System.getenv("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD") == "1")
        originalUserDir = System.getProperty("user.dir")
        originalAdapter = ReaderAdapterHelper.getAdapter()
        System.setProperty("user.dir", temp.root.absolutePath)
        ReaderAdapterHelper.setAdapter(DefaultAdpater())
        renderer = LocalWebviewRenderer(executable, 5000)
        server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/") { exchange ->
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

    @Test(expected = UnsupportedOperationException::class)
    fun unsupportedRegexIsNotSilentlyIgnored() {
        runBlocking { renderer.render(request("/echo", "reader-a", regex = "delete-me")) }
    }
}
