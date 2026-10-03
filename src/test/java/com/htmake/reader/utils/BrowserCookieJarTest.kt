package com.htmake.reader.utils

import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.help.http.CookieStore
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class BrowserCookieJarTest {
    @get:Rule val temp = TemporaryFolder()

    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface

    @Before
    fun setUp() {
        originalUserDir = System.getProperty("user.dir")
        originalAdapter = ReaderAdapterHelper.getAdapter()
        System.setProperty("user.dir", temp.root.absolutePath)
        ReaderAdapterHelper.setAdapter(DefaultAdpater())
    }

    @After
    fun tearDown() {
        if (::originalAdapter.isInitialized) ReaderAdapterHelper.setAdapter(originalAdapter)
        if (::originalUserDir.isInitialized) System.setProperty("user.dir", originalUserDir)
    }

    @Test
    fun namespacesDoNotShareManagedBrowserCookies() {
        BrowserCookieJar.merge(CookieStore("alice"), "https://books.example.test/seed", listOf(
            cookie("session", "alice", "books.example.test")
        ))

        assertEquals(listOf("session"), names("alice", "https://books.example.test/echo"))
        assertTrue(names("bob", "https://books.example.test/echo").isEmpty())
    }

    @Test
    fun pathAndSecureCookiesAreSentOnlyWhereTheyMatch() {
        val store = CookieStore("reader")
        BrowserCookieJar.merge(store, "https://books.example.test/account/login", listOf(
            cookie("root", "one", "books.example.test", path = "/"),
            cookie("account", "two", "books.example.test", path = "/account"),
            cookie("secure", "three", "books.example.test", secure = true)
        ))

        assertEquals(setOf("root", "account", "secure"), names("reader", "https://books.example.test/account/page").toSet())
        assertEquals(setOf("root", "secure"), names("reader", "https://books.example.test/accounting").toSet())
        assertEquals(setOf("root", "account"), names("reader", "http://books.example.test/account/page").toSet())
    }

    @Test
    fun expiryAndDeletionRemoveExactCookieWithoutLegacyRevival() {
        val store = CookieStore("reader")
        store.setCookie("books.example.test", "sid=old")
        BrowserCookieJar.merge(store, "https://books.example.test/seed", listOf(
            cookie("sid", "fresh", "books.example.test")
        ))
        BrowserCookieJar.merge(store, "https://books.example.test/logout", listOf(
            cookie("sid", "", "books.example.test", deleted = true)
        ))
        BrowserCookieJar.merge(store, "https://books.example.test/expired", listOf(
            cookie("old", "gone", "books.example.test", expires = 1.0)
        ))

        assertTrue(names("reader", "https://books.example.test/next").isEmpty())
        assertTrue(BrowserCookieJar.storedCookies(store).isEmpty())
    }

    @Test
    fun domainAndHostOnlyCookiesFollowTheirDifferentScopes() {
        val store = CookieStore("reader")
        BrowserCookieJar.merge(store, "https://books.example.test/seed", listOf(
            cookie("host", "only", "books.example.test", hostOnly = true),
            cookie("domain", "wide", "example.test", hostOnly = false)
        ))

        assertEquals(setOf("host", "domain"), names("reader", "https://books.example.test/a").toSet())
        assertEquals(setOf("domain"), names("reader", "https://cdn.books.example.test/a").toSet())
        assertFalse(names("reader", "https://other.test/a").contains("domain"))
    }

    @Test
    fun browserWorkerCannotInjectAPublicSuffixDomain() {
        val store = CookieStore("reader")
        BrowserCookieJar.merge(store, "https://books.example.com/seed", listOf(
            cookie("leak", "blocked", "com", hostOnly = false)
        ))
        assertTrue(BrowserCookieJar.storedCookies(store).isEmpty())
    }

    @Test
    fun sameOriginResponseUpdatesAreRestoredForNextRequest() {
        val store = CookieStore("reader")
        BrowserCookieJar.merge(store, "https://books.example.test/login", listOf(
            cookie("token", "received", "books.example.test", path = "/reader", httpOnly = true)
        ))

        val restored = BrowserCookieJar.cookiesForRequest(store, "https://books.example.test/reader/chapter")
        assertEquals(1, restored.size)
        assertEquals("token", restored.single().name)
        assertEquals("received", restored.single().value)
        assertTrue(restored.single().httpOnly)
    }

    @Test
    fun clearingLegacyLoginScopeAlsoRevokesBrowserCookies() {
        val store = CookieStore("reader")
        BrowserCookieJar.merge(store, "https://chapter.example.test/login", listOf(
            cookie("chapter", "one", "chapter.example.test"),
            cookie("parent", "two", "example.test", hostOnly = false)
        ))
        // A response from chapter.example.test is not allowed to set other.test.
        // Establish the unrelated origin through its own legitimate response.
        BrowserCookieJar.merge(store, "https://other.test/login", listOf(
            cookie("outside", "three", "other.test")
        ))

        BrowserCookieJar.clearCookieScope(store, "example.test")

        assertTrue(names("reader", "https://chapter.example.test/next").isEmpty())
        assertEquals(listOf("outside"), names("reader", "https://other.test/next"))
    }

    @Test
    fun explicitAndNewManualCookiesOverrideBrowserStateAfterMigration() {
        val store = CookieStore("reader")
        // A pre-existing flat entry is a one-time migration input. Once a browser
        // response owns this Reader scope, it must not revive after a delete.
        store.setCookie("books.example.test", "stale=legacy")
        BrowserCookieJar.merge(store, "https://books.example.test/login", listOf(
            cookie("shared", "browser", "books.example.test")
        ))
        assertFalse(names("reader", "https://books.example.test/next").contains("stale"))

        // SourceLoginController registers a later user-entered header here.
        BrowserCookieJar.setManualCookies(store, "example.test", "shared=manual; user=value")
        val manualRequest = BrowserCookieJar.cookiesForRequest(store, "https://books.example.test/next")
            .associate { it.name to it.value }
        assertEquals("manual", manualRequest["shared"])
        val request = BrowserCookieJar.cookiesForRequest(
            store,
            "https://books.example.test/next",
            explicitCookieHeader = "shared=explicit; once=header"
        ).associate { it.name to it.value }

        assertEquals("explicit", request["shared"])
        assertEquals("value", request["user"])
        assertEquals("header", request["once"])
    }

    @Test
    fun browserMigrationKeepsLegacySubdomainCookieAndManualStillWins() {
        val store = CookieStore("reader")
        // Historical callers used full URLs, which CookieStore persisted under
        // the registrable Reader scope (example.test). A few callers used the
        // bare host; migration must read both without dropping either entry.
        store.setCookie("https://books.example.test", "legacy=url; shared=old")
        store.setCookie("books.example.test", "host=bare")
        val migrated = BrowserCookieJar.cookiesForBrowserRequest(
            store,
            "https://books.example.test/login"
        ).associate { it.name to it.value }
        assertEquals("url", migrated["legacy"])
        assertEquals("bare", migrated["host"])

        BrowserCookieJar.setManualCookies(store, "example.test", "shared=new")
        val manual = BrowserCookieJar.cookiesForRequest(store, "https://books.example.test/next")
            .associate { it.name to it.value }
        assertEquals("new", manual["shared"])
        assertEquals("url", manual["legacy"])
    }

    @Test
    fun ordinaryHostOnlySetCookieRetainsScopeAndRejectsDomainAndInvalidPrefixes() {
        val store = CookieStore("reader")
        BrowserCookieJar.mergeResponseHeaders(store, "https://books.example.com/reader/login", listOf(
            "pathOnly=ok; Path=/reader; Secure; HttpOnly",
            "domainWide=ok; Domain=example.com; Path=/reader",
            "wide=must-not-parse; Domain=com",
            "__Secure-bad=1",
            "__Host-bad=1; Secure; Path=/reader",
            "empty=; Path=/reader"
        ))

        val httpsReader = BrowserCookieJar.cookiesForRequest(store, "https://books.example.com/reader/page")
            .associate { it.name to it.value }
        assertEquals("ok", httpsReader["pathOnly"])
        assertEquals("", httpsReader["empty"])
        assertEquals("ok", httpsReader["domainWide"])
        assertFalse(httpsReader.containsKey("wide"))
        assertFalse(httpsReader.containsKey("__Secure-bad"))
        assertFalse(httpsReader.containsKey("__Host-bad"))
        assertFalse(BrowserCookieJar.cookiesForRequest(store, "http://books.example.com/reader/page")
            .any { it.name == "pathOnly" })
        assertFalse(BrowserCookieJar.cookiesForRequest(store, "https://books.example.com/outside")
            .any { it.name == "pathOnly" })
        assertEquals("ok", BrowserCookieJar.cookiesForRequest(store, "https://cdn.example.com/reader/page")
            .single { it.name == "domainWide" }.value)
    }

    @Test
    fun httpCannotSetOrOverwriteSecureCookie() {
        val store = CookieStore("reader")
        BrowserCookieJar.mergeResponseHeaders(store, "http://books.example.test/login", listOf(
            "newSecure=no; Secure; Path=/"
        ))
        assertFalse(BrowserCookieJar.cookiesForRequest(store, "https://books.example.test/next")
            .any { it.name == "newSecure" })

        BrowserCookieJar.mergeResponseHeaders(store, "https://books.example.test/login", listOf(
            "sid=trusted; Secure; Path=/"
        ))
        BrowserCookieJar.mergeResponseHeaders(store, "http://books.example.test/login", listOf(
            "sid=attacker; Path=/",
            "sid=; Max-Age=0; Path=/"
        ))
        val sid = BrowserCookieJar.cookiesForRequest(store, "https://books.example.test/next")
            .single { it.name == "sid" }
        assertEquals("trusted", sid.value)
        assertTrue(sid.secure)
    }

    @Test
    fun ordinaryExpiryDeletionDoesNotLeaveStructuredCookieAlive() {
        val store = CookieStore("reader")
        BrowserCookieJar.merge(store, "https://books.example.test/login", listOf(
            cookie("sid", "old", "books.example.test")
        ))
        BrowserCookieJar.mergeHostOnlyResponseHeaders(store, "https://books.example.test/logout", listOf(
            "sid=gone; Max-Age=invalid; Expires=Thu, 01 Jan 1970 00:00:00 GMT"
        ))

        assertFalse(BrowserCookieJar.cookiesForRequest(store, "https://books.example.test/next")
            .any { it.name == "sid" })
    }

    private fun names(namespace: String, url: String): List<String> =
        BrowserCookieJar.cookiesForRequest(CookieStore(namespace), url).map { it.name }

    private fun cookie(
        name: String,
        value: String,
        domain: String,
        path: String = "/",
        hostOnly: Boolean = true,
        secure: Boolean = false,
        httpOnly: Boolean = false,
        expires: Double = -1.0,
        deleted: Boolean = false
    ) = BrowserCookieJar.Cookie(
        name = name,
        value = value,
        domain = domain,
        path = path,
        hostOnly = hostOnly,
        secure = secure,
        httpOnly = httpOnly,
        expires = expires,
        deleted = deleted
    )
}
