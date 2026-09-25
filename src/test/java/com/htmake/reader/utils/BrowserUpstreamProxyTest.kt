package com.htmake.reader.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class BrowserUpstreamProxyTest {
    @Test
    fun parsesSupportedProxyProtocols() {
        assertEquals(BrowserUpstreamProxy.Protocol.HTTP,
            BrowserUpstreamProxy.parse("http://proxy.example:8080").protocol)
        assertEquals(BrowserUpstreamProxy.Protocol.SOCKS4,
            BrowserUpstreamProxy.parse("socks4://proxy.example:1080").protocol)
        assertEquals(BrowserUpstreamProxy.Protocol.SOCKS5,
            BrowserUpstreamProxy.parse("socks5://proxy.example:1080").protocol)
    }

    @Test
    fun parsesLegacyAndUriCredentialsWithoutLoggingThem() {
        val legacy = BrowserUpstreamProxy.parse("http://proxy.example:3128@reader@secret")
        assertEquals("proxy.example", legacy.host)
        assertEquals(3128, legacy.port)
        assertEquals("Basic cmVhZGVyOnNlY3JldA==", legacy.basicAuthorization)
        assertTrue(!legacy.toString().contains("reader") && !legacy.toString().contains("secret"))

        val uri = BrowserUpstreamProxy.parse("socks5://reader:secret@proxy.example:1080")
        assertEquals("reader", uri.username)
        assertEquals("secret", uri.password)
        assertNull(uri.basicAuthorization)
    }

    @Test
    fun rejectsUnsupportedSchemesAndInvalidAddresses() {
        listOf("https://proxy.example:443", "http://proxy.example:0", "http://proxy.example:8080/path")
            .forEach { value ->
                try {
                    BrowserUpstreamProxy.parse(value)
                    fail("Expected invalid proxy URL to be rejected")
                } catch (_: IllegalArgumentException) {
                    // Expected.
                }
            }
    }
}
