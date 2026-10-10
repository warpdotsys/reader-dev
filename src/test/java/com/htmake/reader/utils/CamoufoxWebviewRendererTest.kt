package com.htmake.reader.utils

import com.sun.net.httpserver.HttpServer
import com.google.gson.Gson
import com.google.gson.JsonObject
import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.help.http.CookieStore
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.net.InetSocketAddress
import java.net.InetAddress
import java.net.URI
import java.io.IOException
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.ExecutorService
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference
import java.util.Collections

/**
 * End-to-end contract checks for the packaged fingerprint engine. The hosted browser-image
 * workflow supplies the pinned Camoufox runtime; ordinary unit-test runs skip this class.
 */
class CamoufoxWebviewRendererTest {
    @get:Rule val temp = TemporaryFolder()

    @Test(timeout = 30000)
    fun numericalLibraryImportsDoNotAllocateTheHostCpuThreadPool() {
        val python = System.getenv("READER_CAMOUFOX_PYTHON")
        assumeTrue(Files.isRegularFile(Paths.get("/proc/self/status")))
        val workerFile = temp.newFile("generated-packaged-worker.py")
        val bytes = javaClass.getResourceAsStream("/camoufox/worker.py")!!.use { it.readBytes() }
        assertTrue("Packaged worker must be bounded", bytes.size <= 128 * 1024)
        Files.write(workerFile.toPath(), bytes)
        val script = """
            import importlib.util,importlib.metadata,json,os,pathlib,sys
            original_stdout=sys.stdout
            spec=importlib.util.spec_from_file_location('generated_packaged_worker',sys.argv[1])
            module=importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            sys.stdout=original_stdout
            import numpy
            threads=int(next(line.split(':')[1] for line in pathlib.Path('/proc/self/status').read_text().splitlines() if line.startswith('Threads:')))
            print(json.dumps({'threadCount':threads,'numpyVersion':numpy.__version__,
                'camoufoxVersion':importlib.metadata.version('camoufox'),
                'numericalFlagsAllOne':all(os.environ.get(name)=='1' for name in module.NUMERICAL_THREAD_ENV)}))
        """.trimIndent()
        val builder = ProcessBuilder(python, "-c", script, workerFile.absolutePath)
            .redirectError(ProcessBuilder.Redirect.DISCARD)
        builder.environment()["PYTHONDONTWRITEBYTECODE"] = "1"
        // Force the problematic inherited values even on a small hosted runner.
        for (name in listOf("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS")) {
            builder.environment()[name] = "32"
        }
        val process = builder.start()
        try {
            assertTrue("Native import must finish under its own deadline", process.waitFor(20, TimeUnit.SECONDS))
            assertEquals("Native import failed; no raw diagnostics are echoed", 0, process.exitValue())
            val output = process.inputStream.use { it.readNBytes(4097) }
            assertTrue("Native import report must be bounded", output.size <= 4096)
            val value = Gson().fromJson(String(output, StandardCharsets.UTF_8), JsonObject::class.java)
            assertEquals(setOf("threadCount", "numpyVersion", "camoufoxVersion", "numericalFlagsAllOne"), value.keySet())
            assertEquals(1, value.get("threadCount").asInt)
            assertEquals("2.2.6", value.get("numpyVersion").asString)
            assertEquals("0.5.6", value.get("camoufoxVersion").asString)
            assertTrue(value.get("numericalFlagsAllOne").asBoolean)
        } finally {
            if (process.isAlive) {
                process.destroyForcibly()
                assertTrue("Owned native import process stopped", process.waitFor(5, TimeUnit.SECONDS))
            }
        }
    }

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
    private val complexProbeHits = AtomicInteger()
    private val complexProbeCookie = AtomicReference("")
    private val echoCookie = AtomicReference("")
    private val echoContentType = AtomicReference("")
    private val utf8PostHits = AtomicInteger()
    private val utf8PostBytes = AtomicReference(ByteArray(0))
    private val utf8PostMethod = AtomicReference("")
    private val utf8PostContentType = AtomicReference("")
    private val utf8PostContentLength = AtomicReference("")
    private val navigationStarts = AtomicInteger()
    private val navigationPostStarts = AtomicInteger()
    private val navigationMaximumStep = AtomicInteger(-1)
    private val navigationFinalVisits = AtomicInteger()
    private val earlyNavigationStarts = AtomicInteger()
    private val earlyNavigationPostStarts = AtomicInteger()
    private val earlyNavigationFinalVisits = AtomicInteger()
    private val earlyNavigationBlockerHits = AtomicInteger()
    private val earlyNavigationDomReadyHits = AtomicInteger()
    private val earlyNavigationDomReadyAtFinal = AtomicInteger(-1)
    private val earlyNavigationDomReadyAtNavigation = AtomicInteger(-1)
    private val earlyNavigationLoadingAtFinal = AtomicInteger(-1)
    private val earlyNavigationFixtureTimeouts = AtomicInteger()
    private val earlyNavigationBlockerStarted = CountDownLatch(1)
    private val earlyNavigationFinalRequested = CountDownLatch(1)
    private val infiniteNavigationHits = AtomicInteger()
    private val infiniteNavigationPostStarts = AtomicInteger()
    private val sourceScriptPageHits = AtomicInteger()
    private val sourceScriptPostStarts = AtomicInteger()
    private val sourceScriptMarkPosts = AtomicInteger()
    private val sourceNavigationStarts = AtomicInteger()
    private val sourceNavigationOriginalPosts = AtomicInteger()
    private val sourceNavigationFinalVisits = AtomicInteger()
    private val sourceNavigationBlockerHits = AtomicInteger()
    private val sourceNavigationFixtureTimeouts = AtomicInteger()
    private val sourceNavigationInteractiveAtFinal = AtomicInteger()
    private val sourceNavigationRulePosts = AtomicInteger()
    private val sourceNavigationRuleBody = AtomicReference("")
    private val sourceNavigationBlockerStarted = CountDownLatch(1)
    private val sourceNavigationDomReady = CountDownLatch(1)
    private val sourceNavigationFinalRequested = CountDownLatch(1)

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
            val rawRequestBody = exchange.requestBody.use { it.readBytes() }
            val requestBody = rawRequestBody.toString(StandardCharsets.UTF_8)
            val cookie = exchange.requestHeaders.getFirst("Cookie") ?: ""
            if (exchange.requestURI.path.startsWith("/navigation-infinite/")) {
                val step = exchange.requestURI.path.substringAfterLast('/').toInt()
                infiniteNavigationHits.incrementAndGet()
                if (step == 0 && exchange.requestMethod == "POST" && requestBody == "seed=generated") {
                    infiniteNavigationPostStarts.incrementAndGet()
                }
                respond(exchange,
                    "<html><body><div id='result'>generated-infinite-intermediate</div>" +
                        "<script>document.addEventListener('DOMContentLoaded', () => " +
                        "location.replace('/navigation-infinite/${step + 1}'));</script></body></html>",
                    "text/html; charset=utf-8")
                return@createContext
            }
            if (exchange.requestURI.path.startsWith("/navigation-chain/")) {
                val step = exchange.requestURI.path.substringAfterLast('/').toInt()
                navigationMaximumStep.updateAndGet { maxOf(it, step) }
                if (step == 0) {
                    navigationStarts.incrementAndGet()
                    if (exchange.requestMethod == "POST" && requestBody == "seed=generated") {
                        navigationPostStarts.incrementAndGet()
                    }
                    exchange.responseHeaders.add("Set-Cookie", "navigation=generated; Path=/; HttpOnly")
                }
                val target = if (step < 12) "/navigation-chain/${step + 1}" else "/navigation-final"
                respond(exchange,
                    "<html><body><div id='result'>generated-navigation-intermediate</div>" +
                        "<script>document.addEventListener('DOMContentLoaded', () => location.replace('$target'));" +
                        "</script></body></html>", "text/html; charset=utf-8")
                return@createContext
            }
            when (exchange.requestURI.path) {
                "/early-navigation-start" -> {
                    earlyNavigationStarts.incrementAndGet()
                    if (exchange.requestMethod == "POST" && requestBody == "seed=generated") {
                        earlyNavigationPostStarts.incrementAndGet()
                    }
                    exchange.responseHeaders.add("Set-Cookie", "earlyNavigation=generated; Path=/; HttpOnly")
                    // The arm response waits until the parser-blocking resource
                    // is actually requested. Its response in turn stays blocked
                    // until the final navigation arrives: no timer-only inference.
                    respond(exchange,
                        "<html><body><div id='result'>generated-early-intermediate</div><script>" +
                            "let initialDomReadyCount = 0; " +
                            "document.addEventListener('DOMContentLoaded', () => { " +
                            "initialDomReadyCount++; " +
                            "navigator.sendBeacon('/early-navigation-dom-ready', 'generated'); });" +
                            "fetch('/early-navigation-arm').then(() => " +
                            "location.replace('/early-navigation-final?initialReady=' + document.readyState + " +
                            "'&initialDomReady=' + initialDomReadyCount));</script>" +
                            "<script src='/early-navigation-blocker.js'></script></body></html>",
                        "text/html; charset=utf-8")
                }
                "/early-navigation-arm" -> {
                    if (!earlyNavigationBlockerStarted.await(5, TimeUnit.SECONDS)) {
                        earlyNavigationFixtureTimeouts.incrementAndGet()
                    }
                    respond(exchange, "generated-armed", "text/plain; charset=utf-8")
                }
                "/early-navigation-blocker.js" -> {
                    earlyNavigationBlockerHits.incrementAndGet()
                    earlyNavigationBlockerStarted.countDown()
                    try {
                        if (!earlyNavigationFinalRequested.await(5, TimeUnit.SECONDS)) {
                            earlyNavigationFixtureTimeouts.incrementAndGet()
                        }
                        respond(exchange, "/* generated blocking resource released */", "application/javascript")
                    } catch (_: IOException) {
                        // A genuine navigation cancels the obsolete subresource.
                        exchange.close()
                    } catch (_: InterruptedException) {
                        Thread.currentThread().interrupt()
                        exchange.close()
                    }
                }
                "/early-navigation-dom-ready" -> {
                    earlyNavigationDomReadyHits.incrementAndGet()
                    respond(exchange, "generated-ready", "text/plain; charset=utf-8")
                }
                "/early-navigation-final" -> {
                    // Two different HTTP requests can reach the server out of order.
                    // Measure the event count in the same JS turn as location.replace,
                    // not from arrival ordering of the cancelled document's beacon.
                    val query = exchange.requestURI.rawQuery.orEmpty().split('&')
                        .associate { it.substringBefore('=') to it.substringAfter('=', "") }
                    earlyNavigationDomReadyAtFinal.set(earlyNavigationDomReadyHits.get())
                    earlyNavigationDomReadyAtNavigation.set(query["initialDomReady"]?.toIntOrNull() ?: -1)
                    earlyNavigationLoadingAtFinal.set(if (query["initialReady"] == "loading") 1 else 0)
                    earlyNavigationFinalVisits.incrementAndGet()
                    earlyNavigationFinalRequested.countDown()
                    respond(exchange,
                        "<html><body><div id='result'>generated-early-complete</div></body></html>",
                        "text/html; charset=utf-8")
                }
                "/source-script-page" -> {
                    sourceScriptPageHits.incrementAndGet()
                    if (exchange.requestMethod == "POST" && requestBody == "seed=generated") {
                        sourceScriptPostStarts.incrementAndGet()
                    }
                    respond(exchange,
                        "<html><body><div id='result'>generated-before-script</div></body></html>",
                        "text/html; charset=utf-8")
                }
                "/source-script-mark" -> {
                    if (exchange.requestMethod == "POST" && requestBody == "mark=generated") {
                        sourceScriptMarkPosts.incrementAndGet()
                    }
                    respond(exchange, "generated-mark", "text/plain; charset=utf-8")
                }
                "/source-navigation-start" -> {
                    sourceNavigationStarts.incrementAndGet()
                    if (exchange.requestMethod == "POST" && requestBody == "seed=generated") {
                        sourceNavigationOriginalPosts.incrementAndGet()
                    }
                    respond(exchange,
                        "<html><body><div id='result'>generated-source-navigation-intermediate</div><script>" +
                            "document.addEventListener('DOMContentLoaded', () => { " +
                            "navigator.sendBeacon('/source-navigation-ready','generated'); " +
                            "fetch('/source-navigation-arm').then(() => " +
                            "location.replace('/source-navigation-final?initialReady=' + document.readyState)); " +
                            "});</script><img src='/source-navigation-blocker'></body></html>",
                        "text/html; charset=utf-8")
                }
                "/source-navigation-ready" -> {
                    sourceNavigationDomReady.countDown()
                    respond(exchange, "generated-ready", "text/plain; charset=utf-8")
                }
                "/source-navigation-arm" -> {
                    if (!sourceNavigationBlockerStarted.await(5, TimeUnit.SECONDS) ||
                        !sourceNavigationDomReady.await(5, TimeUnit.SECONDS)) {
                        sourceNavigationFixtureTimeouts.incrementAndGet()
                    }
                    // Leave time for the old DOMContentLoaded-only path to run
                    // its rule. Actual readiness and blocked load are asserted
                    // via the handshake and the document's readyState below.
                    Thread.sleep(300)
                    respond(exchange, "generated-armed", "text/plain; charset=utf-8")
                }
                "/source-navigation-blocker" -> {
                    sourceNavigationBlockerHits.incrementAndGet()
                    sourceNavigationBlockerStarted.countDown()
                    try {
                        if (!sourceNavigationFinalRequested.await(5, TimeUnit.SECONDS)) {
                            sourceNavigationFixtureTimeouts.incrementAndGet()
                        }
                        respond(exchange, "generated-image-finished", "text/plain; charset=utf-8")
                    } catch (_: IOException) {
                        exchange.close()
                    } catch (_: InterruptedException) {
                        Thread.currentThread().interrupt()
                        exchange.close()
                    }
                }
                "/source-navigation-final" -> {
                    sourceNavigationInteractiveAtFinal.set(
                        if (exchange.requestURI.rawQuery == "initialReady=interactive") 1 else 0)
                    sourceNavigationFinalVisits.incrementAndGet()
                    sourceNavigationFinalRequested.countDown()
                    respond(exchange,
                        "<html><body><div id='result'>generated-source-navigation-final</div></body></html>",
                        "text/html; charset=utf-8")
                }
                "/source-navigation-rule-mark" -> {
                    if (exchange.requestMethod == "POST") {
                        sourceNavigationRulePosts.incrementAndGet()
                        sourceNavigationRuleBody.set(requestBody)
                    }
                    respond(exchange, "generated-rule-marked", "text/plain; charset=utf-8")
                }
                "/navigation-final" -> {
                    navigationFinalVisits.incrementAndGet()
                    respond(exchange,
                        "<html><body><div id='result'>generated-navigation-complete</div></body></html>",
                        "text/html; charset=utf-8")
                }
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
                "/seed-renewed" -> {
                    exchange.responseHeaders.add("Set-Cookie", "session=beta==; Path=/; HttpOnly")
                    respond(exchange, "renewed", "text/plain; charset=utf-8")
                }
                "/seed-scoped" -> {
                    exchange.responseHeaders.add("Set-Cookie", "scoped=only; Path=/scoped; HttpOnly")
                    respond(exchange, "seeded", "text/plain; charset=utf-8")
                }
                "/scripted-seed" -> {
                    exchange.responseHeaders.add("Set-Cookie", "scripted=original; Path=/")
                    respond(exchange, "<html><body>seeded</body></html>", "text/html; charset=utf-8")
                }
                "/scripted-renew-delete" -> {
                    exchange.responseHeaders.add("Set-Cookie", "scripted=renewed; Path=/")
                    exchange.responseHeaders.add("Set-Cookie", "fresh=received; Path=/")
                    exchange.responseHeaders.add("Set-Cookie", "hidden=keep; Path=/; HttpOnly")
                    respond(exchange, "<html><body>renewed</body></html>", "text/html; charset=utf-8")
                }
                "/page-script-delete" -> {
                    exchange.responseHeaders.add("Set-Cookie", "scripted=renewed; Path=/")
                    exchange.responseHeaders.add("Set-Cookie", "fresh=received; Path=/")
                    exchange.responseHeaders.add("Set-Cookie", "hidden=keep; Path=/; HttpOnly")
                    respond(exchange,
                        "<html><body><script>" +
                            "document.cookie='scripted=; Max-Age=0; Path=/';" +
                            "document.cookie='fresh=; Max-Age=0; Path=/';" +
                            "document.cookie='hidden=; Max-Age=0; Path=/';" +
                            "document.body.dataset.cookieScript='ran';" +
                            "</script></body></html>",
                        "text/html; charset=utf-8")
                }
                "/complex-cookie-page" -> {
                    exchange.responseHeaders.add("Set-Cookie",
                        "quoted=\"alpha;beta\"; Path=/; HttpOnly; Expires=Wed, 21 Oct 2037 07:28:00 GMT")
                    respond(exchange,
                        "<html><body><script src='/complex-cookie-probe'></script></body></html>",
                        "text/html; charset=utf-8")
                }
                "/complex-cookie-probe" -> {
                    complexProbeCookie.set(cookie)
                    complexProbeHits.incrementAndGet()
                    respond(exchange, ";", "application/javascript; charset=utf-8")
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
                "/echo" -> {
                    echoCookie.set(cookie)
                    echoContentType.set(exchange.requestHeaders.getFirst("Content-Type") ?: "")
                    respond(exchange,
                        "${exchange.requestMethod}|$requestBody|$cookie|${exchange.requestHeaders.getFirst("X-Reader-Probe") ?: ""}",
                        "text/plain; charset=utf-8")
                }
                "/utf8-post" -> {
                    // Preserve bytes before decoding; re-encoding a summary would hide truncation.
                    utf8PostBytes.set(rawRequestBody)
                    utf8PostMethod.set(exchange.requestMethod)
                    utf8PostContentType.set(exchange.requestHeaders.getFirst("Content-Type") ?: "")
                    utf8PostContentLength.set(exchange.requestHeaders.getFirst("Content-Length") ?: "")
                    utf8PostHits.incrementAndGet()
                    respond(exchange,
                        "<html><body><div id='result'>WebView编码原始书</div></body></html>",
                        "text/html; charset=utf-8")
                }
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
        assertEquals("Ordinary GET must not acquire a form content type", "", echoContentType.get())

        val post = renderer.render(request("/echo", "user-a", post = true, body = "page=2"))
        assertTrue(post.body?.contains("POST|page=2|") == true)
        assertEquals("The actual target must receive the original JAR's default form type",
            "application/x-www-form-urlencoded; charset=UTF-8", echoContentType.get())

        val script = renderer.render(request(
            "/resource-page",
            "user-a",
            javaScript = "document.querySelector('#resource-result').textContent"
        ))
        assertEquals("asset-loaded", script.body)
    }

    @Test(timeout = 30000)
    fun generatedUtf8PostPreservesRawBytesAndSourceScriptResult() = runBlocking {
        // Generated-only input shared with the historical-engine characterization, never a real book.
        val body = """{"query":"黎明之剑𠮷😀 + & %","note":"中文 UTF-8"}"""
        val expectedBytes = body.toByteArray(StandardCharsets.UTF_8)
        assertEquals("The sample distinguishes UTF-16 character count from wire bytes", 44, body.length)
        assertEquals(60, expectedBytes.size)
        val expectedResult = "WebView编码书𠮷😀 + & %"
        val result = renderer.render(request("/utf8-post", "generated-utf8-post",
            headers = mapOf("Content-Type" to "application/json; charset=utf-8"),
            post = true, body = body,
            javaScript = "document.querySelector('#result').textContent = '$expectedResult'; " +
                "document.querySelector('#result').textContent"))

        assertEquals("A generated POST must not be replayed", 1, utf8PostHits.get())
        assertEquals("POST", utf8PostMethod.get())
        assertEquals("application/json; charset=utf-8", utf8PostContentType.get())
        assertEquals("60", utf8PostContentLength.get())
        assertArrayEquals("Compare the actual target bytes, not a reconstructed request", expectedBytes, utf8PostBytes.get())
        val digest = java.security.MessageDigest.getInstance("SHA-256").digest(utf8PostBytes.get())
            .joinToString("") { "%02x".format(it) }
        assertEquals("8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1", digest)
        assertEquals("The source rule must return the full Unicode marker, not the original HTML", expectedResult, result.body)
    }

    @Test
    fun generatedClientNavigationReturnsTheFinalDocumentWithoutReplayingPost() = runBlocking {
        // Independent generated namespaces, never retries of a failed request.
        val rounds = 12
        repeat(rounds) { round ->
            val user = "navigation-probe-$round"
            val result = try {
                renderer.render(request("/navigation-chain/0", user,
                    post = true, body = "seed=generated"))
            } catch (error: Exception) {
                // Generated counters only; no URL, Cookie, headers or page body.
                throw AssertionError("Finite generated navigation failed: generatedOnly=true round=$round " +
                    "maxStep=${navigationMaximumStep.get()} finalVisits=${navigationFinalVisits.get()} " +
                    "starts=${navigationStarts.get()} posts=${navigationPostStarts.get()}", error)
            }
            val diagnostic = "generatedOnly=true round=$round snapshotChars=${result.body?.length ?: 0} " +
                "intermediate=${result.body?.contains("generated-navigation-intermediate") == true} " +
                "maxStep=${navigationMaximumStep.get()} finalVisits=${navigationFinalVisits.get()} " +
                "starts=${navigationStarts.get()} posts=${navigationPostStarts.get()}"
            assertTrue("A finite generated navigation must return the final document: $diagnostic",
                result.body?.contains("generated-navigation-complete") == true)
            assertFalse(result.body?.contains("generated-navigation-intermediate") == true)
            assertEquals("A redirect must not discard the generated HttpOnly response Cookie",
                "generated", BrowserCookieJar.storedCookies(CookieStore(user))
                    .single { it.name == "navigation" }.value)
        }
        assertEquals("HTML snapshot handling must not replay the original navigation", rounds, navigationStarts.get())
        assertEquals("Each generated form must be submitted only once", rounds, navigationPostStarts.get())
    }

    @Test
    fun generatedNavigationBeforeDomReadyReturnsFinalDocumentWithoutReplayingPost() = runBlocking {
        val user = "generated-before-dom-navigation"
        val result = try {
            renderer.render(request("/early-navigation-start", user, post = true, body = "seed=generated"))
        } catch (error: Exception) {
            throw AssertionError("Early generated navigation failed: generatedOnly=true " +
                "starts=${earlyNavigationStarts.get()} posts=${earlyNavigationPostStarts.get()} " +
                "blockerHits=${earlyNavigationBlockerHits.get()} finalVisits=${earlyNavigationFinalVisits.get()} " +
                "domReadyAtFinal=${earlyNavigationDomReadyAtFinal.get()} " +
                "domReadyAtNavigation=${earlyNavigationDomReadyAtNavigation.get()} " +
                "loadingAtFinal=${earlyNavigationLoadingAtFinal.get()} " +
                "fixtureTimeouts=${earlyNavigationFixtureTimeouts.get()}", error)
        }
        assertEquals("The fixture handshake must not silently expire", 0, earlyNavigationFixtureTimeouts.get())
        assertEquals("The original parser must actually be blocked", 1, earlyNavigationBlockerHits.get())
        assertEquals("The generated destination must actually be visited", 1, earlyNavigationFinalVisits.get())
        assertEquals("Navigation must start before the original DOMContentLoaded", 0,
            earlyNavigationDomReadyAtNavigation.get())
        assertEquals("The source document must actually still be loading when it navigates", 1,
            earlyNavigationLoadingAtFinal.get())
        assertTrue("An early navigation must return its actual final document",
            result.body?.contains("generated-early-complete") == true)
        assertFalse(result.body?.contains("generated-early-intermediate") == true)
        assertEquals("The generated initial request must not be replayed", 1, earlyNavigationStarts.get())
        assertEquals("The generated original form must be submitted exactly once", 1, earlyNavigationPostStarts.get())
        assertEquals("An early navigation must retain its generated HttpOnly response Cookie", "generated",
            BrowserCookieJar.storedCookies(CookieStore(user)).single { it.name == "earlyNavigation" }.value)
    }

    @Test
    fun endlessGeneratedNavigationTimesOutAndTheNextRenderRecovers() = runBlocking {
        val python = System.getenv("READER_CAMOUFOX_PYTHON") ?: error("Camoufox Python is required")
        val version = System.getenv("READER_CAMOUFOX_BROWSER_VERSION") ?: "152.0.4-beta.30"
        val limited = CamoufoxWebviewRenderer(python, version, 3_000, allowPrivateNetworks = true)
        try {
            val started = System.nanoTime()
            val outcome = runCatching {
                limited.render(request("/navigation-infinite/0", "endless-navigation-user",
                    post = true, body = "seed=generated"))
            }
            val elapsedMs = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - started)
            val failure = outcome.exceptionOrNull()
            assertTrue("An endless generated chain must actually navigate through multiple documents",
                infiniteNavigationHits.get() >= 5)
            assertEquals("Timeout handling must not replay the initial generated form",
                1, infiniteNavigationPostStarts.get())
            assertTrue("An endless chain must fail rather than return an intermediate document",
                failure is IllegalStateException)
            // Require the real worker timeout, not the 21-second emergency parent
            // watchdog: endlessly renewing a per-document deadline must fail here.
            assertTrue("An endless chain must fail with the worker timeout, not an unrelated error",
                failure?.message?.contains("(TimeoutError)") == true)
            assertTrue("Each redirect must not renew the total budget: ${elapsedMs}ms", elapsedMs < 20_000)
            val stoppedHits = infiniteNavigationHits.get()
            val healthy = limited.render(request("/echo", "endless-navigation-user"))
            assertTrue("The next actual browser render must recover after endless navigation",
                healthy.body?.contains("GET|||") == true)
            assertEquals("The timed-out browser must stop hitting the generated navigation fixture",
                stoppedHits, infiniteNavigationHits.get())
        } finally {
            limited.close()
        }
    }

    @Test
    fun javaScriptStructuredResultsUseTheArchivedWebviewResponseFormat() = runBlocking {
        val cases = listOf(
            "({answer: 42, ready: true})" to "{\"answer\":42,\"ready\":true}",
            "['alpha', 7]" to "[\"alpha\",7]",
            "42" to "42",
            "Promise.resolve('generated-string')" to "generated-string",
            "Promise.resolve({answer: 42, ready: true})" to "{\"answer\":42,\"ready\":true}",
            "Promise.resolve(['alpha', 7])" to "[\"alpha\",7]",
            "Promise.resolve(42)" to "42",
            "Promise.resolve(null)" to ""
        )
        for ((expression, expected) in cases) {
            val result = renderer.render(request("/resource-page", "script-types", javaScript = expression))
            assertEquals("JavaScript expression $expression", expected, result.body)
        }
        val user = "async-script-generated"
        val delayed = renderer.render(request("/source-script-page", user,
            post = true, body = "seed=generated",
            javaScript = "(()=>{ globalThis.generatedRuns = (globalThis.generatedRuns || 0) + 1; " +
                "return new Promise(resolve => setTimeout(() => { " +
                "document.querySelector('#result').textContent = 'async-ready'; " +
                "document.cookie = 'asyncOnly=generated; Path=/'; " +
                "resolve({runs:globalThis.generatedRuns, dom:document.querySelector('#result').textContent, " +
                "cookie:document.cookie.includes('asyncOnly=generated')}); }, 150)); })()"))
        assertEquals("{\"runs\":1,\"dom\":\"async-ready\",\"cookie\":true}", delayed.body)
        assertEquals("A delayed result must not replay the original generated navigation", 1, sourceScriptPageHits.get())
        assertEquals("A delayed result must not replay the original generated POST", 1, sourceScriptPostStarts.get())
        assertEquals("Async Cookie mutation must be captured after result completion", "generated",
            BrowserCookieJar.storedCookies(CookieStore(user)).single { it.name == "asyncOnly" }.value)
        assertTrue("An async mutation must stay isolated to its generated namespace",
            BrowserCookieJar.storedCookies(CookieStore("async-script-stranger")).isEmpty())
    }

    @Test
    fun generatedSourceStateDeletionFailsWithoutReplayingSideEffectsAndRecovers() = runBlocking {
        val failure = runCatching {
            renderer.render(request("/source-script-page", "generated-state-deletion",
                post = true, body = "seed=generated",
                javaScript = "fetch('/source-script-mark', {method:'POST', body:'mark=generated'}).then(() => { " +
                    "Object.getOwnPropertyNames(globalThis).filter(k => k.startsWith('__reader_source_'))" +
                    ".forEach(k => { delete globalThis[k]; }); return new Promise(()=>{}); })"))
        }.exceptionOrNull()
        assertTrue("Explicit generated state deletion must fail, not invent a result", failure is IllegalStateException)
        assertTrue("Require the original specific worker category, not generic transport or timeout",
            failure?.message?.contains("(SourceScriptStateLost)") == true)
        assertEquals("A lost state must not replay the original navigation", 1, sourceScriptPageHits.get())
        assertEquals("A lost state must not replay the original POST", 1, sourceScriptPostStarts.get())
        assertEquals("A lost state must not replay the source's side effect", 1, sourceScriptMarkPosts.get())
        val healthy = renderer.render(request("/echo", "generated-state-deletion"))
        assertTrue("The next browser context must recover after state deletion", healthy.body?.contains("GET|||") == true)
    }

    @Test
    fun generatedSourceRuleRunsOnceInTheDocumentThatCompletesLoad() = runBlocking {
        val result = try {
            renderer.render(request("/source-navigation-start", "generated-source-load-navigation",
                post = true, body = "seed=generated",
                javaScript = "fetch('/source-navigation-rule-mark', {method:'POST', " +
                    "body:document.querySelector('#result').textContent}).then(() => " +
                    "new Promise(resolve => setTimeout(() => " +
                    "resolve(document.querySelector('#result').textContent),800)))"))
        } catch (error: Exception) {
            throw AssertionError("Generated source load navigation failed: generatedOnly=true " +
                "starts=${sourceNavigationStarts.get()} posts=${sourceNavigationOriginalPosts.get()} " +
                "blockerHits=${sourceNavigationBlockerHits.get()} finalVisits=${sourceNavigationFinalVisits.get()} " +
                "interactiveAtFinal=${sourceNavigationInteractiveAtFinal.get()} " +
                "rulePosts=${sourceNavigationRulePosts.get()} fixtureTimeouts=${sourceNavigationFixtureTimeouts.get()}", error)
        }
        assertEquals("The generated fixture handshake must not silently expire", 0, sourceNavigationFixtureTimeouts.get())
        assertEquals("A real subresource must hold the initial load event", 1, sourceNavigationBlockerHits.get())
        assertEquals("Navigation must occur after DOMContentLoaded but before load", 1, sourceNavigationInteractiveAtFinal.get())
        assertEquals("The generated destination must actually be visited once", 1, sourceNavigationFinalVisits.get())
        assertEquals("The rule must return the actual document that completes load",
            "generated-source-navigation-final", result.body)
        assertEquals("The original navigation must not be replayed", 1, sourceNavigationStarts.get())
        assertEquals("The original POST must not be replayed", 1, sourceNavigationOriginalPosts.get())
        assertEquals("The source rule's side effect must execute exactly once", 1, sourceNavigationRulePosts.get())
        assertEquals("The source rule must not execute in the discarded intermediate document",
            "generated-source-navigation-final", sourceNavigationRuleBody.get())
        val healthy = renderer.render(request("/echo", "generated-source-load-navigation"))
        assertTrue("The next independent context must remain healthy", healthy.body?.contains("GET|||") == true)
    }

    @Test
    fun scriptPromiseFailuresAreBoundedAndTheNextRenderRecovers() = runBlocking {
        val python = System.getenv("READER_CAMOUFOX_PYTHON") ?: error("Camoufox Python is required")
        val version = System.getenv("READER_CAMOUFOX_BROWSER_VERSION") ?: "152.0.4-beta.30"
        val limited = CamoufoxWebviewRenderer(python, version, 3_000, allowPrivateNetworks = true)
        try {
            val scripts = listOf(
                "fetch('/source-script-mark', {method:'POST', body:'mark=generated'}).then(() => " +
                    "Promise.reject(new Error('generated-secret-do-not-log')))" to "SourceScriptRejected",
                "fetch('/source-script-mark', {method:'POST', body:'mark=generated'}).then(() => " +
                    "new Promise(()=>{}))" to "SourceScriptTimeout"
            )
            for ((script, expectedError) in scripts) {
                val started = System.nanoTime()
                val failure = runCatching {
                    limited.render(request("/source-script-page", "async-failure-generated",
                        post = true, body = "seed=generated", javaScript = script))
                }.exceptionOrNull()
                val elapsedMs = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - started)
                assertTrue("A generated Promise failure must not be returned as apparent success",
                    failure is IllegalStateException)
                assertTrue("Require the worker error, not emergency parent termination",
                    failure?.message?.contains("($expectedError)") == true)
                assertFalse("Source rejection must not leak its message into the parent protocol",
                    failure?.message?.contains("generated-secret-do-not-log") == true)
                assertTrue("A Promise must not renew its single budget: ${elapsedMs}ms", elapsedMs < 20_000)
                val healthy = limited.render(request("/echo", "async-failure-generated"))
                assertTrue("The next actual browser render must recover after $expectedError",
                    healthy.body?.contains("GET|||") == true)
            }
            assertEquals("Each failing generated navigation runs only once", 2, sourceScriptPageHits.get())
            assertEquals("Each failing generated form is posted only once", 2, sourceScriptPostStarts.get())
            assertEquals("Each failing source script issues its generated POST only once", 2, sourceScriptMarkPosts.get())
        } finally {
            limited.close()
        }
    }

    @Test
    fun importedNetscapeCookiesRetainScopeInRealBrowserRequests() = runBlocking {
        val user = "netscape-browser"
        val export = "# Netscape HTTP Cookie File\r\n" +
            "#HttpOnly_127.0.0.1\tFALSE\t/scoped\tFALSE\t0\timportedScope\tgenerated-alpha==\r\n" +
            "127.0.0.1\tFALSE\t/\tTRUE\t0\tsecureImport\tgenerated-secure\r\n" +
            "127.0.0.1\tFALSE\t/\tFALSE\t0\temptyImport\t"
        val store = CookieStore(user)
        val records = BrowserCookieJar.parseNetscapeCookies(baseUrl, export)
        assertEquals(3, BrowserCookieJar.replaceImportedCookies(store, baseUrl, records))

        val page = renderer.render(request("/scoped/resource-page", user))
        assertTrue(page.body?.contains("importedScope=generated-alpha==") == true)
        assertTrue(page.body?.contains("emptyImport=") == true)
        assertFalse("Secure cookie must not accompany an HTTP request",
            page.body?.contains("generated-secure") == true)
        assertTrue("The generated unscoped script must actually be requested", unscopedProbeHits.get() > 0)
        assertFalse("Path-scoped cookie must not escape through a subresource",
            unscopedProbeCookie.get().contains("importedScope="))
        assertFalse(unscopedProbeCookie.get().contains("secureImport="))
        assertTrue(unscopedProbeCookie.get().contains("emptyImport="))

        val visible = renderer.render(request("/scoped/resource-page", user, javaScript = "document.cookie"))
        assertFalse("HttpOnly must remain invisible to page JavaScript",
            visible.body?.contains("importedScope=") == true)
        val outside = renderer.render(request("/echo", user))
        assertFalse(outside.body?.contains("importedScope=") == true)
        assertFalse(outside.body?.contains("secureImport=") == true)
        assertTrue(outside.body?.contains("emptyImport=") == true)
        val stranger = renderer.render(request("/scoped/resource-page", "netscape-stranger"))
        assertFalse(stranger.body?.contains("importedScope=") == true)
        assertFalse(stranger.body?.contains("emptyImport=") == true)
        assertEquals(3, BrowserCookieJar.savedCookiesForSource(store, baseUrl).size)
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
    fun existingHttpOnlyCookieCanBeRenewed() = runBlocking {
        val user = "renewed-http-only"
        renderer.render(request("/seed", user))
        assertEquals("alpha==", BrowserCookieJar.storedCookies(CookieStore(user))
            .single { it.name == "session" }.value)
        renderer.render(request("/seed-renewed", user))
        assertEquals("beta==", BrowserCookieJar.storedCookies(CookieStore(user))
            .single { it.name == "session" }.value)
        renderer.render(request("/echo", user))
        assertTrue("Renewed HttpOnly Cookie was not replayed: ${echoCookie.get()}",
            echoCookie.get().contains("session=beta=="))
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
    fun sourceJavaScriptDeletionOverridesSameResponseSetCookie() = runBlocking {
        val user = "js-delete"
        renderer.render(request("/scripted-seed", user))
        assertEquals("original", BrowserCookieJar.storedCookies(CookieStore(user))
            .single { it.name == "scripted" }.value)

        val deleted = renderer.render(request(
            "/scripted-renew-delete", user,
            javaScript = "document.cookie = 'scripted=; Max-Age=0; Path=/'; " +
                "document.cookie = 'fresh=; Max-Age=0; Path=/'; " +
                "document.cookie = 'hidden=; Max-Age=0; Path=/'; 'deleted'"
        ))
        assertEquals("deleted", deleted.body)
        val stored = BrowserCookieJar.storedCookies(CookieStore(user))
        assertFalse("webJs deletion was undone by Set-Cookie fallback: $stored",
            stored.any { it.name == "scripted" || it.name == "fresh" })
        assertEquals("webJs must not revoke an HttpOnly Cookie", "keep",
            stored.single { it.name == "hidden" }.value)
        val next = renderer.render(request("/echo", user))
        assertFalse("Deleted Cookie was sent on the next request: ${next.body}",
            next.body?.contains("scripted=") == true || next.body?.contains("fresh=") == true)
        assertTrue("HttpOnly Cookie was lost on the next request: ${next.body}",
            next.body?.contains("hidden=keep") == true)
    }

    @Test
    fun pageJavaScriptDeletionBeforeDomReadyDoesNotResurrectCookies() = runBlocking {
        val user = "page-js-delete"
        renderer.render(request("/scripted-seed", user))
        assertEquals("original", BrowserCookieJar.storedCookies(CookieStore(user))
            .single { it.name == "scripted" }.value)

        val page = renderer.render(request("/page-script-delete", user))
        assertTrue("The inline deletion script did not run: ${page.body}",
            page.body?.contains("data-cookie-script=\"ran\"") == true)
        val stored = BrowserCookieJar.storedCookies(CookieStore(user))
        assertFalse("The response fallback resurrected an inline-deleted Cookie: $stored",
            stored.any { it.name == "scripted" || it.name == "fresh" })
        assertEquals("An inline script revoked HttpOnly: $stored", "keep",
            stored.single { it.name == "hidden" }.value)
        val next = renderer.render(request("/echo", user))
        assertFalse("An inline-deleted Cookie was sent later: ${next.body}",
            next.body?.contains("scripted=") == true || next.body?.contains("fresh=") == true)
        assertTrue("HttpOnly Cookie was lost: ${next.body}", next.body?.contains("hidden=keep") == true)
    }

    @Test
    fun quotedCookieReplayMatchesWhatTheBrowserActuallyAccepted() = runBlocking {
        val user = "quoted-cookie"
        renderer.render(request("/complex-cookie-page", user))
        assertEquals("The same-render browser subresource did not run", 1, complexProbeHits.get())
        val observed = complexProbeCookie.get()
        val stored = BrowserCookieJar.storedCookies(CookieStore(user))
        if (observed.contains("quoted=")) {
            assertTrue("Browser accepted a Cookie that Reader did not retain: $stored",
                stored.any { it.name == "quoted" })
        } else {
            assertFalse("Reader persisted a Cookie rejected by the browser: $stored",
                stored.any { it.name == "quoted" })
        }
        renderer.render(request("/echo", user))
        assertEquals("Persisted Cookie replay differs from the browser's own request",
            observed, echoCookie.get())
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

    @Test(timeout = 240000)
    fun generatedRuleHeadersStayOnTheirOriginAcrossRedirectsAndScripts() = runBlocking {
        GeneratedHeaderFixture().use { fixture ->
            for (case in listOf("same-get", "cross-get", "cross-post", "resources")) {
                fixture.reset()
                val response = renderer.render(headerRequest(fixture, case))
                fixture.assertDocument(response.body!!, case)
                assertEquals("Legacy response URL remains the original request URL", fixture.start(case), response.url)
                val rows = fixture.snapshot()
                assertEquals(if (case == "resources") 3 else 2, rows.size)
                fixture.assertOriginHeaders(rows)
                if (case == "cross-post") {
                    assertEquals(listOf("POST", "GET"), rows.map { it.method })
                    assertArrayEquals("q=generated-header-only".toByteArray(StandardCharsets.UTF_8), rows[0].body)
                    assertTrue(rows[1].body.isEmpty())
                    assertEquals("application/x-www-form-urlencoded; charset=UTF-8", rows[0].contentType)
                    assertEquals("", rows[1].contentType)
                }
            }
        }
    }

    @Test(timeout = 240000)
    fun generatedPostRedirectsPreserveBodyCookiesAndOriginHeaders() = runBlocking {
        GeneratedHeaderFixture().use { fixture ->
            val raw = "q=生成中文&value=保持原始字节"
            for (case in listOf("same-post-307", "same-post-308", "cross-post-307", "cross-post-308")) {
                fixture.reset()
                val response = renderer.render(headerRequest(fixture, case).copy(
                    post = true, body = raw,
                    headerMap = fixture.headers + ("Content-Type" to "application/json; charset=UTF-8")))
                fixture.assertDocument(response.body!!, case)
                val rows = fixture.snapshot()
                assertEquals(2, rows.size)
                assertEquals(listOf("POST", "POST"), rows.map { it.method })
                for (row in rows) {
                    assertArrayEquals(raw.toByteArray(StandardCharsets.UTF_8), row.body)
                    assertEquals("application/json; charset=UTF-8", row.contentType)
                }
                fixture.assertOriginHeaders(rows)
                assertEquals("", rows[0].cookie)
                assertEquals(if (case.startsWith("same-")) "generated_session=GENERATED_SESSION_ONLY" else "", rows[1].cookie)
                val persisted = BrowserCookieJar.cookiesForRequest(
                    CookieStore("generated-header-$case"), fixture.start(case))
                assertEquals(1, persisted.size)
                assertTrue(persisted.single().httpOnly)
                assertEquals("generated_session", persisted.single().name)
                assertEquals("/probe/", persisted.single().path)
            }
        }
    }

    @Test(timeout = 120000)
    fun generatedScriptAndResourceRedirectsRespectOriginHeaderPolicy() = runBlocking {
        GeneratedHeaderFixture().use { fixture ->
            for (case in listOf("script-cross-get", "resources-redirect")) {
                fixture.reset()
                val response = renderer.render(headerRequest(fixture, case))
                fixture.assertDocument(response.body!!, case)
                val rows = fixture.snapshot()
                assertEquals(if (case == "resources-redirect") 5 else 2, rows.size)
                fixture.assertOriginHeaders(rows)
                assertEquals(1, rows.count { it.actor == "secondary" })
            }
        }
    }

    @Test(timeout = 120000)
    fun generatedLoopbackRedirectCannotBypassTheEgressProxy() = runBlocking {
        GeneratedHeaderFixture().use { fixture ->
            val policy = object : BrowserNetworkPolicy() {
                override fun resolveRequestTarget(value: String): BrowserNetworkTarget? {
                    val uri = URI(value)
                    // Admit only this test's first host/port; no global private-network switch.
                    if (uri.host == "127.0.0.1" && uri.port == URI(fixture.primary).port) {
                        return BrowserNetworkTarget(uri, "127.0.0.1", uri.port,
                            listOf(InetAddress.getByName("127.0.0.1")))
                    }
                    return super.resolveRequestTarget(value)
                }
            }
            val strict = CamoufoxWebviewRenderer(System.getenv("READER_CAMOUFOX_PYTHON"),
                System.getenv("READER_CAMOUFOX_BROWSER_VERSION") ?: "152.0.4-beta.30", 10_000, policy)
            try {
                val failure = runCatching { strict.render(headerRequest(fixture, "cross-get")) }.exceptionOrNull()
                assertTrue("Denied redirect must propagate the Java network policy", failure is BrowserNetworkPolicyViolation)
                assertEquals(1, fixture.snapshot().count { it.actor == "primary" })
                assertEquals("Loopback redirect may not bypass the proxy", 0,
                    fixture.snapshot().count { it.actor == "secondary" })
                fixture.reset()
                fixture.assertDocument(strict.render(headerRequest(fixture, "same-get")).body!!, "same-get")
                assertEquals(2, fixture.snapshot().size)
            } finally {
                strict.close()
            }
        }
    }

    private fun headerRequest(fixture: GeneratedHeaderFixture, case: String) = request(
        "/unused", "generated-header-$case", fixture.headers,
        post = case == "cross-post", body = if (case == "cross-post") "q=generated-header-only" else null,
        javaScript = """
            (() => {
              if (${case.startsWith("resources")} &&
                  (document.documentElement.dataset.generatedSameResource !== 'executed' ||
                   document.documentElement.dataset.generatedCrossResource !== 'executed')) {
                throw new Error('GENERATED_RESOURCE_NOT_EXECUTED');
              }
              return {href:location.href, title:document.querySelector('.name').textContent};
            })()
        """.trimIndent()
    ).copy(url = fixture.start(case))

    private data class HeaderObservation(val actor: String, val role: String, val method: String,
        val authorization: String, val trace: String, val contentType: String, val cookie: String, val body: ByteArray)

    private class GeneratedHeaderFixture : AutoCloseable {
        val headers = mapOf("Authorization" to "Bearer GENERATED_NOT_A_REAL_CREDENTIAL",
            "X-Generated-Trace" to "GENERATED_HEADER_TRACE_ONLY")
        private val workers = Executors.newFixedThreadPool(4)
        private val rows = Collections.synchronizedList(mutableListOf<HeaderObservation>())
        private val primaryServer = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        private val secondaryServer = HttpServer.create(InetSocketAddress("127.0.0.2", 0), 0)
        val primary = "http://127.0.0.1:${primaryServer.address.port}"
        val secondary = "http://127.0.0.2:${secondaryServer.address.port}"

        init {
            for ((server, actor) in listOf(primaryServer to "primary", secondaryServer to "secondary")) {
                server.executor = workers
                server.createContext("/probe/") { exchange ->
                    try {
                        val parts = exchange.requestURI.path.split('/')
                        require(parts.size == 4)
                        val case = parts[2]
                        val role = parts[3]
                        val body = exchange.requestBody.use { it.readNBytes(65537) }
                        require(body.size <= 65536)
                        require(rows.size < 128)
                        fun field(name: String) = exchange.requestHeaders.getFirst(name) ?: ""
                        rows.add(HeaderObservation(actor, role, exchange.requestMethod,
                            field("Authorization"), field("X-Generated-Trace"), field("Content-Type"), field("Cookie"), body))
                        when {
                            role == "start" && case == "script-cross-get" -> send(exchange, 200,
                                "<script>window.location.href='$secondary/probe/$case/end';</script>")
                            role == "start" && !case.startsWith("resources") -> {
                                val status = if (case.endsWith("307")) 307 else if (case.endsWith("308")) 308 else 303
                                exchange.responseHeaders.add("Location", (if (case.startsWith("same-")) primary else secondary) + "/probe/$case/end")
                                if (status != 303) exchange.responseHeaders.add("Set-Cookie",
                                    "generated_session=GENERATED_SESSION_ONLY; Path=/probe/; HttpOnly")
                                send(exchange, status, "")
                            }
                            role == "start" -> {
                                val cross = if (case == "resources") secondary else primary
                                send(exchange, 200, document(case) +
                                    "<script src='$primary/probe/$case/same.js'></script><script src='$cross/probe/$case/cross.js'></script>")
                            }
                            case == "resources-redirect" && (role == "same.js" || role == "cross.js") -> {
                                exchange.responseHeaders.add("Location", (if (role == "same.js") primary else secondary) +
                                    "/probe/$case/" + (if (role == "same.js") "same-end.js" else "cross-end.js"))
                                send(exchange, if (role == "same.js") 307 else 303, "")
                            }
                            role.endsWith(".js") -> send(exchange, 200,
                                "document.documentElement.dataset." +
                                    (if (role.startsWith("same")) "generatedSameResource" else "generatedCrossResource") + "='executed';",
                                "application/javascript")
                            role == "end" -> send(exchange, 200, document(case))
                            else -> send(exchange, 404, "GENERATED_UNKNOWN_PATH")
                        }
                    } finally { exchange.close() }
                }
                server.start()
            }
        }

        fun start(case: String) = "$primary/probe/$case/start"
        fun reset() = rows.clear()
        fun snapshot(): List<HeaderObservation> = synchronized(rows) { rows.toList() }
        fun assertOriginHeaders(observed: List<HeaderObservation>) {
            assertTrue(observed.any { it.actor == "primary" })
            for (row in observed) {
                assertEquals(if (row.actor == "primary") headers["Authorization"] else "", row.authorization)
                assertEquals(if (row.actor == "primary") headers["X-Generated-Trace"] else "", row.trace)
            }
        }
        fun assertDocument(body: String, case: String) {
            val value = Gson().fromJson(body, JsonObject::class.java)
            assertEquals("认证头生成书-$case", value.get("title").asString)
            val host = if (case.startsWith("cross-") || case == "script-cross-get") secondary else primary
            assertEquals("$host/probe/$case/" + (if (case.startsWith("resources")) "start" else "end"), value.get("href").asString)
        }
        private fun document(case: String) = "<!doctype html><meta charset='UTF-8'><span class='name'>认证头生成书-$case</span>"
        private fun send(exchange: com.sun.net.httpserver.HttpExchange, status: Int, body: String,
            contentType: String = "text/html; charset=UTF-8") {
            val bytes = body.toByteArray(StandardCharsets.UTF_8)
            exchange.responseHeaders.add("Content-Type", contentType)
            exchange.responseHeaders.add("Cache-Control", "no-store")
            exchange.sendResponseHeaders(status, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        override fun close() {
            primaryServer.stop(0)
            secondaryServer.stop(0)
            workers.shutdownNow()
            assertTrue("Generated fixture threads stopped", workers.awaitTermination(5, TimeUnit.SECONDS))
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
