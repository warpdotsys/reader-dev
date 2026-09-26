package com.htmake.reader.api.controller

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SourceLoginSecurityTest {
    @Test
    fun onlyImportedSourceUrlCanResolveAndInlineJsonIsRejected() {
        val sources = JsonArray().add(JsonObject()
            .put("bookSourceUrl", "https://a.example.test")
            .put("bookSourceName", "已导入"))

        assertEquals("https://a.example.test", SourceLoginSupport
            .findImportedSource(sources, "https://a.example.test")?.bookSourceUrl)
        assertNull(SourceLoginSupport.findImportedSource(
            sources,
            "{\"bookSourceUrl\":\"https://attacker.example\",\"loginUrl\":\"@js:java.lang.Runtime.getRuntime()\"}"
        ))
    }

    @Test
    fun clearScopeCoversAllSameDomainSourcesButNotOtherDomains() {
        val first = "https://chapter.example.com/login"
        val sameScope = "https://api.example.com/account"
        val other = "https://example.net/login"

        assertTrue(SourceLoginSupport.sharesCookieScope(first, sameScope))
        assertFalse(SourceLoginSupport.sharesCookieScope(first, other))
        // This is the exact key CookieStore uses for both the ordinary and cookieJar stores.
        assertEquals("example.com", SourceLoginSupport.cookieScope(first))
        assertEquals("example.com", SourceLoginSupport.cookieScope(sameScope))
        assertEquals(
            listOf(first, sameScope),
            SourceLoginSupport.affectedSourceUrls(first, listOf(first, sameScope, other))
        )
    }

    @Test
    fun cookieListingsAreRedactedRatherThanCredentialExfiltration() {
        val display = SourceLoginSupport.redactCookie("session=secret-token; refresh=another-secret")
        assertEquals("session=***; refresh=***", display)
        assertFalse(display.contains("secret"))
    }
}
