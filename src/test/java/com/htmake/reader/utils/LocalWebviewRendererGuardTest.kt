package com.htmake.reader.utils

import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class LocalWebviewRendererGuardTest {
    @Test
    fun rejectsLoopbackBeforeStartingPlaywright() = runBlocking {
        val renderer = LocalWebviewRenderer(allowPrivateNetworks = false)
        try {
            renderer.render(WebviewRequest(
                url = "http://127.0.0.1:9/private",
                html = null,
                encode = null,
                tag = null,
                headerMap = null,
                sourceRegex = null,
                javaScript = null,
                proxy = null,
                post = false,
                body = null,
                userNameSpace = "test-user",
                debugLog = null
            ))
            fail("Loopback URL should be rejected before requiring a browser executable")
        } catch (error: BrowserNetworkPolicyViolation) {
            assertTrue(error.message.orEmpty().contains("阻止"))
        } finally {
            renderer.close()
        }
    }
}
