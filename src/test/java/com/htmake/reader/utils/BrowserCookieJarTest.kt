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
    fun netscapeKeepsDomainPathSecureHttpOnlySessionAndEmptyValue() {
        val records = BrowserCookieJar.parseNetscapeCookies("http://books.example.test",
            "\uFEFF# Netscape HTTP Cookie File\r\n" +
                "#HttpOnly_.example.test\tTRUE\t/account\tTRUE\t0\tsession\talpha==\r\n" +
                "books.example.test\tFALSE\t/\tFALSE\t0\tempty\t")
        assertEquals(2, records.size)
        val session = records[0]
        assertEquals("example.test", session.domain)
        assertEquals("/account", session.path)
        assertEquals("alpha==", session.value)
        assertTrue(session.secure)
        assertTrue(session.httpOnly)
        assertFalse(session.hostOnly)
        assertEquals(-1.0, session.expires, 0.0)
        assertEquals("", records[1].value)
        assertTrue(records[1].hostOnly)
    }

    @Test
    fun importedCookiesReplaceApplicableCredentialsWithoutFlatteningOrCrossUserLeak() {
        val store = CookieStore("importer")
        store.setCookie("https://books.example.test", "session=old-flat")
        BrowserCookieJar.setManualCookies(store, "example.test", "session=old-manual")
        BrowserCookieJar.merge(store, "https://other.test", listOf(cookie("unrelated", "keep", "other.test")))
        val records = BrowserCookieJar.parseNetscapeCookies("http://books.example.test",
            "#HttpOnly_.example.test\tTRUE\t/account\tTRUE\t0\tsession\tfresh==\n" +
                "books.example.test\tFALSE\t/\tFALSE\t0\tempty\t")
        assertEquals(2, BrowserCookieJar.replaceImportedCookies(store, "http://books.example.test", records))
        assertTrue(store.getCookie("https://books.example.test").isEmpty())
        assertEquals(listOf("empty"), names("importer", "http://books.example.test/account"))
        assertEquals(listOf("empty"), names("importer", "https://books.example.test/accounting"))
        assertEquals(listOf("session", "empty"), names("importer", "https://books.example.test/account/chapter"))
        assertEquals(listOf("session"), names("importer", "https://cdn.books.example.test/account"))
        assertEquals(listOf("unrelated"), names("importer", "https://other.test"))
        assertTrue(names("stranger", "https://books.example.test/account").isEmpty())
        val browser = BrowserCookieJar.cookiesForBrowserRequest(store, "https://books.example.test/account")
        assertEquals("fresh==", browser.first { it.name == "session" }.value)
        assertTrue(browser.first { it.name == "session" }.httpOnly)
        assertFalse(browser.any { it.value.contains("old-") })
        // Base URL does not match /account or Secure, but saved state is still visible.
        assertEquals(2, BrowserCookieJar.savedCookiesForSource(store, "http://books.example.test").size)
    }

    @Test
    fun netscapeImportDoesNotManageOrDeleteAnUnrelatedPublicSuffixLegacyKey() {
        val store = CookieStore("public-suffix-import")
        val source = "https://example.co.uk"
        val unrelated = "https://unrelated.co.uk"
        store.setCookie(unrelated, "legacyOther=keep")
        val before = store.getCookie(unrelated)
        assertEquals("legacyOther=keep", before)
        val records = BrowserCookieJar.parseNetscapeCookies(source,
            "example.co.uk\tFALSE\t/\tTRUE\t0\timported\tnew")
        assertEquals(1, BrowserCookieJar.replaceImportedCookies(store, source, records))
        assertEquals(before, store.getCookie(unrelated))
        assertEquals(listOf("imported"), names("public-suffix-import", source))
        assertEquals(listOf("imported"), BrowserCookieJar.cookiesForBrowserRequest(store, source).map { it.name })
        assertEquals(listOf("legacyOther"), names("public-suffix-import", unrelated))
    }

    @Test
    fun expiredNetscapeRecordsCannotReviveOlderValuesAndLastDuplicateWins() {
        val store = CookieStore("expired-import")
        val records = BrowserCookieJar.parseNetscapeCookies("https://books.example.test",
            "books.example.test\tFALSE\t/\tFALSE\t0\tsid\tfirst\n" +
                "books.example.test\tFALSE\t/\tFALSE\t1\tsid\told\n" +
                "books.example.test\tFALSE\t/\tFALSE\t0\tother\tfirst\n" +
                "books.example.test\tFALSE\t/\tFALSE\t0\tother\tlast")
        assertEquals(1, BrowserCookieJar.replaceImportedCookies(store, "https://books.example.test", records))
        assertEquals(listOf("other"), names("expired-import", "https://books.example.test"))
        assertEquals("last", BrowserCookieJar.storedCookies(store).single().value)
    }

    @Test
    fun invalidNetscapeInputIsRejectedWithoutCredentialValuesInErrors() {
        val secret = "generated-private-value"
        val inputs = listOf(
            ".other.test\tTRUE\t/\tFALSE\t0\tsid\t$secret",
            ".co.uk\tTRUE\t/\tFALSE\t0\tsid\t$secret",
            ".com\tTRUE\t/\tFALSE\t0\tsid\t$secret",
            "books.example.test\tMAYBE\t/\tFALSE\t0\tsid\t$secret",
            "books.example.test\tFALSE\t/\tFALSE\t-1\tsid\t$secret",
            "books.example.test\tFALSE\tbadpath\tFALSE\t0\tsid\t$secret",
            "books.example.test\tFALSE\t/\tFALSE\t0\tbad=name\t$secret",
            "books.example.test\tFALSE\t/\tFALSE\t0\tsid\t$secret;injected=x",
            "books.example.test\tFALSE\t/\tFALSE\t0\t__Secure-sid\t$secret",
            ".example.test\tTRUE\t/\tTRUE\t0\t__Host-sid\t$secret",
            "books.example.test\tFALSE\t/auth\tTRUE\t0\t__Host-sid\t$secret",
            "books.example.test\tFALSE\t/\tFALSE\t0\tsid\t$secret\textra",
            "# Netscape HTTP Cookie File\n# comments only"
        )
        inputs.forEach { input ->
            try {
                BrowserCookieJar.parseNetscapeCookies("https://books.example.test", input)
                throw AssertionError("Malformed generated export was accepted")
            } catch (error: IllegalArgumentException) {
                assertFalse(error.message.orEmpty().contains(secret))
            }
        }
    }

    @Test
    fun netscapeCannotSetHostOnlyCookiesForASiblingOrSpoofedDomain() {
        listOf("api.example.test", "example.test", "books.example.test.attacker.test").forEach { domain ->
            try {
                BrowserCookieJar.parseNetscapeCookies("https://books.example.test",
                    "$domain\tFALSE\t/\tFALSE\t0\tsid\tx")
                throw AssertionError("Wrong host was accepted")
            } catch (_: IllegalArgumentException) { }
        }
        // Public suffix rejection must also hold when that suffix is a parent of the source.
        try {
            BrowserCookieJar.parseNetscapeCookies("https://books.example.co.uk",
                ".co.uk\tTRUE\t/\tTRUE\t0\tsid\tx")
            throw AssertionError("Public suffix was accepted")
        } catch (_: IllegalArgumentException) { }
    }

    @Test
    fun invalidImportListCannotPartiallyReplaceAnExistingJar() {
        val store = CookieStore("atomic-import")
        BrowserCookieJar.merge(store, "https://books.example.test", listOf(cookie("existing", "keep", "books.example.test")))
        val before = store.getCookie(BrowserCookieJar.STORAGE_KEY)
        try {
            BrowserCookieJar.replaceImportedCookies(store, "https://books.example.test", listOf(
                cookie("valid", "new", "books.example.test"), cookie("invalid", "bad", "other.test")))
            throw AssertionError("Invalid list was accepted")
        } catch (_: IllegalArgumentException) { }
        assertEquals(before, store.getCookie(BrowserCookieJar.STORAGE_KEY))
    }

    @Test
    fun netscapeInputHasLengthAndRecordBudgetsAndDoesNotMisclassifyPlainHeaders() {
        assertTrue(BrowserCookieJar.isNetscapeInput("# Netscape HTTP Cookie File\n"))
        assertTrue(BrowserCookieJar.isNetscapeInput("books.example.test\tFALSE\t/\tFALSE\t0\tsid\tx"))
        assertFalse(BrowserCookieJar.isNetscapeInput("sid=value; other=two"))
        listOf("x".repeat(65537), (1..513).joinToString("\n") {
            "books.example.test\tFALSE\t/\tFALSE\t0\tsid$it\tx"
        }).forEach { input ->
            try {
                BrowserCookieJar.parseNetscapeCookies("https://books.example.test", input)
                throw AssertionError("Unbounded input was accepted")
            } catch (_: IllegalArgumentException) { }
        }
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
