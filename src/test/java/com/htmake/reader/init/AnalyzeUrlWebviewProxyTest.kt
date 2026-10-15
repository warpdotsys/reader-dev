package com.htmake.reader.init

import com.htmake.reader.utils.WebviewRenderer
import com.htmake.reader.utils.WebviewRequest
import com.htmake.reader.utils.workDirInit
import com.htmake.reader.utils.workDirPath
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.data.entities.BookSource
import io.legado.app.help.http.StrResponse
import io.legado.app.model.analyzeRule.AnalyzeUrl
import io.legado.app.model.analyzeRule.RuleDataInterface
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Test
import java.nio.file.Files

/** Parameter-flow tests with an explicit renderer double, not browser acceptance. */
class AnalyzeUrlWebviewProxyTest {
    @Test
    fun sourceProxyReachesWebviewGetWithoutBecomingAnHttpHeader() = runBlocking {
        verify("""{"proxy":"http://127.0.0.1:18081","X-Generated":"source"}""",
            null, "http://127.0.0.1:18081", false)
    }

    @Test
    fun sourceProxyReachesWebviewPostWithTheOriginalBody() = runBlocking {
        verify("""{"proxy":"socks5://127.0.0.1:18082","X-Generated":"source"}""",
            null, "socks5://127.0.0.1:18082", true)
    }

    @Test
    fun explicitHeaderMapRetainsItsPrecedenceOverSourceProxy() = runBlocking {
        verify("""{"proxy":"http://127.0.0.1:18081","X-Generated":"source"}""",
            mapOf("proxy" to "http://127.0.0.1:18083", "X-Generated" to "override"),
            "http://127.0.0.1:18083", true)
    }

    @Test
    fun aSourceWithoutProxyKeepsTheDirectDefault() = runBlocking {
        verify("""{"X-Generated":"source"}""", null, null, false)
    }

    private suspend fun verify(sourceHeader: String, headerOverride: Map<String, String>?,
                               expectedProxy: String?, post: Boolean) {
        val testDir = Files.createTempDirectory("reader-generated-proxy-contract-").toFile()
        val originalWorkDir = workDirPath
        val originalWorkDirInit = workDirInit
        val originalAdapter = ReaderAdapterHelper.readerAdapter
        val originalRenderer = ReaderAdapter.webviewRenderer
        var received: WebviewRequest? = null
        try {
            workDirPath = testDir.absolutePath
            workDirInit = true
            ReaderAdapterHelper.setAdapter(ReaderAdapter)
            ReaderAdapter.webviewRenderer = object : WebviewRenderer {
                override val managesBrowserCookies = true
                override suspend fun render(request: WebviewRequest): StrResponse {
                    received = request
                    return StrResponse(request.url ?: "", "GENERATED_RENDERER_DOUBLE_ONLY")
                }
            }
            val source = BookSource(bookSourceUrl = "https://generated-source.example",
                header = sourceHeader, enabledCookieJar = false)
            val ruleData = object : RuleDataInterface {
                override val variableMap = HashMap<String, String>()
                override fun getUserNameSpace() = "generated-proxy-reader"
                override fun putVariable(key: String, value: String?) {
                    if (value == null) variableMap.remove(key) else variableMap[key] = value
                }
            }
            val body = "q=GENERATED_ONLY&value=中文"
            val options = if (post)
                """{"webView":true,"method":"POST","body":"$body","webJs":"document.body.innerText"}"""
            else """{"webView":true,"webJs":"document.body.innerText"}"""
            val response = AnalyzeUrl("https://generated-source.example/search, $options",
                source = source, ruleData = ruleData, headerMapF = headerOverride)
                .getStrResponseAwait(jsStr = "document.title", sourceRegex = "generated-rule")

            assertNotNull("The real URL parser must call the selected renderer", received)
            assertEquals("GENERATED_RENDERER_DOUBLE_ONLY", response.body)
            assertEquals(expectedProxy, received?.proxy)
            assertNotNull(received?.headerMap)
            assertFalse(received!!.headerMap!!.containsKey("proxy"))
            assertEquals(if (headerOverride == null) "source" else "override", received?.headerMap?.get("X-Generated"))
            assertEquals(post, received?.post)
            assertEquals(if (post) body else null, received?.body)
            assertEquals("https://generated-source.example/search", received?.url)
            assertEquals(source.bookSourceUrl, received?.tag)
            assertEquals("generated-proxy-reader", received?.userNameSpace)
            assertEquals("generated-rule", received?.sourceRegex)
            assertEquals("document.body.innerText", received?.javaScript)
        } finally {
            ReaderAdapter.webviewRenderer = originalRenderer
            ReaderAdapterHelper.setAdapter(originalAdapter)
            workDirPath = originalWorkDir
            workDirInit = originalWorkDirInit
            // Only the directory created by this test, never an existing Reader store.
            testDir.deleteRecursively()
        }
    }
}
