package com.htmake.reader.utils

import com.htmake.reader.init.ReaderAdapter
import com.sun.net.httpserver.HttpServer
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.help.http.CookieStore
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.nio.file.Files

class WebviewCookieCompatibilityTest {

    @Test
    fun remoteCookiesReachOnlyTheRequestingUsersLegacyJar() = runBlocking {
        val testDir = Files.createTempDirectory("reader-webview-cookie-").toFile()
        val originalWorkDir = workDirPath
        val originalWorkDirInit = workDirInit
        val originalAdapter = ReaderAdapterHelper.readerAdapter
        val originalApi = RemoteWebview.remoteWebviewApi
        val server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
        server.createContext("/render.html") { exchange ->
            exchange.requestBody.use { it.readBytes() }
            exchange.responseHeaders.add("Set-Cookie", "session=alpha; Path=/")
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
            assertEquals("alpha", firstUser.getKey("example.org_cookieJar", "session"))
            assertEquals("", secondUser.getCookie("example.org_cookieJar"))

            // AnalyzeUrl copies this jar into the ordinary domain cookie before its next request.
            firstUser.replaceCookie("example.org", firstUser.getCookie("example.org_cookieJar"))
            assertEquals("alpha", firstUser.getKey("https://chapter.example.org/2", "session"))
            assertEquals("", secondUser.getCookie("https://chapter.example.org/2"))
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
