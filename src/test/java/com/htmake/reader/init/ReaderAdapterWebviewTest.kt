package com.htmake.reader.init

import com.htmake.reader.utils.WebviewRenderer
import com.htmake.reader.utils.WebviewRequest
import io.legado.app.help.http.StrResponse
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Test

class ReaderAdapterWebviewTest {

    @Test
    fun passesTheExistingRequestContractToTheSelectedRenderer() = runBlocking {
        val original = ReaderAdapter.webviewRenderer
        val expected = StrResponse("https://example.org/chapter", "rendered")
        var received: WebviewRequest? = null
        ReaderAdapter.webviewRenderer = object : WebviewRenderer {
            override suspend fun render(request: WebviewRequest): StrResponse {
                received = request
                return expected
            }
        }
        try {
            val actual = ReaderAdapter.getStrResponseByRemoteWebview(
                url = "https://example.org/chapter",
                html = null,
                encode = "",
                tag = "chapter",
                headerMap = mapOf("charset" to "GBK", "X-Test" to "value"),
                sourceRegex = "source-rule",
                javaScript = "document.body.innerText",
                proxy = "http://127.0.0.1:8888",
                post = true,
                body = "page=2",
                userNameSpace = "reader-a",
                debugLog = null
            )

            assertSame(expected, actual)
            assertEquals("https://example.org/chapter", received?.url)
            assertEquals("GBK", received?.encode)
            assertEquals("chapter", received?.tag)
            assertEquals("value", received?.headerMap?.get("X-Test"))
            assertEquals("source-rule", received?.sourceRegex)
            assertEquals("document.body.innerText", received?.javaScript)
            assertEquals("http://127.0.0.1:8888", received?.proxy)
            assertEquals(true, received?.post)
            assertEquals("page=2", received?.body)
            assertEquals("reader-a", received?.userNameSpace)
        } finally {
            ReaderAdapter.webviewRenderer = original
        }
    }
}
