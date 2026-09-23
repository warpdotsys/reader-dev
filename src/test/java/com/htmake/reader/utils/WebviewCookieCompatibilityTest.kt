package com.htmake.reader.utils

import com.htmake.reader.init.ReaderAdapter
import com.sun.net.httpserver.HttpServer
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.data.entities.BookSource
import io.legado.app.help.http.CookieStore
import io.legado.app.model.analyzeRule.AnalyzeUrl
import io.legado.app.model.analyzeRule.RuleDataInterface
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

class WebviewCookieCompatibilityTest {

    @Test
    fun ordinarySourceRequestForwardsOnlyTheCookiePair() = runBlocking {
        val testDir = Files.createTempDirectory("reader-source-cookie-").toFile()
        val originalWorkDir = workDirPath
        val originalWorkDirInit = workDirInit
        val originalAdapter = ReaderAdapterHelper.readerAdapter
        val requestCount = AtomicInteger()
        val secondCookie = AtomicReference("")
        val thirdCookie = AtomicReference("")
        val server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/chapter") { exchange ->
            when (requestCount.incrementAndGet()) {
                1 -> exchange.responseHeaders.add("Set-Cookie", "session=alpha==; Path=/; SameSite=Lax")
                2 -> {
                    secondCookie.set(exchange.requestHeaders.getFirst("Cookie") ?: "")
                    exchange.responseHeaders.add("Set-Cookie", "session=; Max-Age=0; Path=/")
                }
                else -> thirdCookie.set(exchange.requestHeaders.getFirst("Cookie") ?: "")
            }
            val response = "chapter".toByteArray(StandardCharsets.UTF_8)
            exchange.sendResponseHeaders(200, response.size.toLong())
            exchange.responseBody.use { it.write(response) }
        }
        try {
            workDirPath = testDir.absolutePath
            workDirInit = true
            ReaderAdapterHelper.setAdapter(ReaderAdapter)
            server.start()
            val baseUrl = "http://127.0.0.1:${server.address.port}"
            val source = BookSource(bookSourceUrl = baseUrl, enabledCookieJar = true)
            val ruleData = object : RuleDataInterface {
                override val variableMap = HashMap<String, String>()
                override fun getUserNameSpace() = "reader-a"
                override fun putVariable(key: String, value: String?) {
                    if (value == null) variableMap.remove(key) else variableMap[key] = value
                }
            }

            repeat(3) {
                AnalyzeUrl("$baseUrl/chapter", source = source, ruleData = ruleData)
                    .getStrResponseAwait()
            }

            assertEquals(3, requestCount.get())
            assertEquals("session=alpha==", secondCookie.get())
            assertEquals("", thirdCookie.get())
        } finally {
            server.stop(0)
            ReaderAdapterHelper.setAdapter(originalAdapter)
            workDirPath = originalWorkDir
            workDirInit = originalWorkDirInit
            testDir.deleteRecursively()
        }
    }

    @Test
    fun remoteCookiesReachOnlyTheRequestingUsersLegacyJar() = runBlocking {
        val testDir = Files.createTempDirectory("reader-webview-cookie-").toFile()
        val originalWorkDir = workDirPath
        val originalWorkDirInit = workDirInit
        val originalAdapter = ReaderAdapterHelper.readerAdapter
        val originalApi = RemoteWebview.remoteWebviewApi
        val requestCount = AtomicInteger()
        val server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/render.html") { exchange ->
            exchange.requestBody.use { it.readBytes() }
            val responseCookie = if (requestCount.incrementAndGet() == 1) {
                "session=alpha==; Path=/; HttpOnly; SameSite=Lax"
            } else {
                "session=; Max-Age=0; Path=/"
            }
            exchange.responseHeaders.add("Set-Cookie", responseCookie)
            val response = "rendered".toByteArray(StandardCharsets.UTF_8)
            exchange.sendResponseHeaders(200, response.size.toLong())
            exchange.responseBody.use { it.write(response) }
        }
        try {
            workDirPath = testDir.absolutePath
            workDirInit = true
            ReaderAdapterHelper.setAdapter(ReaderAdapter)
            server.start()
            RemoteWebview.setRemoteApi("http://127.0.0.1:${server.address.port}")

            RemoteWebview.getStrResponse(
                url = "https://chapter.example.org/1",
                userNameSpace = "reader-a"
            )

            val firstUser = CookieStore("reader-a")
            val secondUser = CookieStore("reader-b")
            assertEquals("alpha==", firstUser.getKey("example.org_cookieJar", "session"))
            assertEquals("session=alpha==", firstUser.getCookie("example.org_cookieJar"))
            assertEquals("", secondUser.getCookie("example.org_cookieJar"))

            firstUser.replaceResponseCookie("example.org_cookieJar", "refresh=beta==; Secure; SameSite=Strict")
            assertEquals("beta==", firstUser.getKey("example.org_cookieJar", "refresh"))
            assertEquals("", firstUser.getKey("example.org_cookieJar", "SameSite"))

            // AnalyzeUrl copies this jar into the ordinary domain cookie before its next request.
            firstUser.replaceCookie("example.org", firstUser.getCookie("example.org_cookieJar"))
            assertEquals("alpha==", firstUser.getKey("https://chapter.example.org/2", "session"))
            assertEquals("beta==", firstUser.getKey("https://chapter.example.org/2", "refresh"))
            assertEquals("", secondUser.getCookie("https://chapter.example.org/2"))

            RemoteWebview.getStrResponse(url = "https://chapter.example.org/2", userNameSpace = "reader-a")
            assertEquals("", firstUser.getKey("example.org_cookieJar", "session"))
            assertEquals("", firstUser.getKey("https://chapter.example.org/2", "session"))
            assertEquals("beta==", firstUser.getKey("https://chapter.example.org/2", "refresh"))

            firstUser.replaceResponseCookie("example.org_cookieJar", "refresh=beta==; Expires=Thu, 01 Jan 1970 00:00:00 GMT")
            assertEquals("", firstUser.getKey("example.org_cookieJar", "refresh"))
            assertEquals("", firstUser.getKey("https://chapter.example.org/2", "refresh"))
            firstUser.setCookie("not a domain/path", "ignored=yes")
            assertEquals("", firstUser.getCookie("not a domain/path"))
        } finally {
            server.stop(0)
            RemoteWebview.setRemoteApi(originalApi)
            ReaderAdapterHelper.setAdapter(originalAdapter)
            workDirPath = originalWorkDir
            workDirInit = originalWorkDirInit
            testDir.deleteRecursively()
        }
    }
}
