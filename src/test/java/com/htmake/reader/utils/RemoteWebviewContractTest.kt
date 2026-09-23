package com.htmake.reader.utils

import com.google.gson.JsonParser
import com.sun.net.httpserver.HttpServer
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets

class RemoteWebviewContractTest {

    @Test
    fun sendsTheLegacyRenderPayloadAndReturnsItsBody() = runBlocking {
        var requestMethod = ""
        var requestBody = ""
        val server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/render.html") { exchange ->
            requestMethod = exchange.requestMethod
            requestBody = exchange.requestBody.use { input ->
                String(input.readBytes(), StandardCharsets.UTF_8)
            }
            val response = "<html>rendered</html>".toByteArray(StandardCharsets.UTF_8)
            exchange.sendResponseHeaders(200, response.size.toLong())
            exchange.responseBody.use { it.write(response) }
        }
        val originalApi = RemoteWebview.remoteWebviewApi
        server.start()
        try {
            RemoteWebview.setRemoteApi("http://127.0.0.1:${server.address.port}")
            val rendered = RemoteWebview.render(
                WebviewRequest(
                    url = "https://example.org/chapter",
                    html = null,
                    encode = "GBK",
                    tag = "chapter",
                    headerMap = mapOf("X-Test" to "value"),
                    sourceRegex = "source-rule",
                    javaScript = "document.title",
                    proxy = "http://127.0.0.1:8888",
                    post = true,
                    body = "page=2",
                    userNameSpace = "reader-a",
                    debugLog = null
                )
            )

            assertEquals("POST", requestMethod)
            val payload = JsonParser().parse(requestBody).asJsonObject
            assertEquals("https://example.org/chapter", payload.get("url").asString)
            assertEquals("value", payload.getAsJsonObject("headers").get("X-Test").asString)
            assertEquals("document.title", payload.get("js_source").asString)
            assertEquals("http://127.0.0.1:8888", payload.get("proxy").asString)
            assertEquals("POST", payload.get("http_method").asString)
            assertEquals("page=2", payload.get("body").asString)
            assertEquals("GBK", payload.get("encode").asString)
            assertEquals("chapter", payload.get("tag").asString)
            assertEquals("source-rule", payload.get("sourceRegex").asString)
            assertEquals("<html>rendered</html>", rendered.body)
            assertEquals("https://example.org/chapter", rendered.url)
        } finally {
            RemoteWebview.setRemoteApi(originalApi)
            server.stop(0)
        }
    }
}
