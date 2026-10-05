package com.htmake.reader.utils

import com.microsoft.playwright.Page
import com.microsoft.playwright.PlaywrightException
import com.microsoft.playwright.TimeoutError
import com.microsoft.playwright.options.LoadState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class BrowserDocumentSnapshotTest {
    private val race = "Unable to retrieve content because the page is navigating and changing the content."

    private fun page(content: () -> String, wait: (Double) -> Unit = { fail("Unexpected wait") },
                     load: (Double) -> Unit = { fail("Unexpected load-state wait") }): Page =
        java.lang.reflect.Proxy.newProxyInstance(Page::class.java.classLoader, arrayOf(Page::class.java)) {
                _, method, arguments ->
            when (method.name) {
                "content" -> content()
                "waitForTimeout" -> { wait(arguments[0] as Double); null }
                "waitForLoadState" -> {
                    assertEquals(LoadState.DOMCONTENTLOADED, arguments[0])
                    load((arguments[1] as Page.WaitForLoadStateOptions).timeout)
                    null
                }
                else -> throw AssertionError("Snapshot reader must not invoke ${method.name}")
            }
        } as Page

    @Test
    fun successfulReadDoesNotWaitOrReplayNavigationAndScripts() {
        var reads = 0
        val result = BrowserDocumentSnapshot.read(page({ reads++; "generated-html" }), 200, {}, { 0 })
        assertEquals("generated-html", result)
        assertEquals(1, reads)
    }

    @Test
    fun retriesOnlyTheSnapshotRaceWithOneDecreasingBudget() {
        var now = 0L
        var reads = 0
        val loadTimeouts = mutableListOf<Double>()
        val result = BrowserDocumentSnapshot.read(page({
            if (++reads <= 2) throw PlaywrightException("Error: $race")
            "generated-html"
        }, { delay -> now += (delay * 1_000_000).toLong() }, { loadTimeouts.add(it) }), 200, {}, { now })
        assertEquals("generated-html", result)
        assertEquals(3, reads)
        assertEquals(listOf(150.0, 100.0), loadTimeouts)
    }

    @Test
    fun repeatedNavigationStopsAtOneSnapshotDeadline() {
        var now = 0L
        var reads = 0
        val delays = mutableListOf<Double>()
        val loadTimeouts = mutableListOf<Double>()
        try {
            BrowserDocumentSnapshot.read(page({ reads++; throw PlaywrightException(race) }, { delay ->
                delays.add(delay); now += (delay * 1_000_000).toLong()
            }, { loadTimeouts.add(it) }), 125, {}, { now })
            fail("An endless navigation must not extend the retry budget")
        } catch (error: TimeoutError) {
            assertTrue(error.message.orEmpty().contains("125ms"))
        }
        assertEquals(3, reads)
        assertEquals(listOf(50.0, 50.0, 25.0), delays)
        assertEquals(listOf(75.0, 25.0), loadTimeouts)
    }

    @Test
    fun closedTargetIsNotRetried() {
        val closed = PlaywrightException("Target page, context or browser has been closed")
        try {
            BrowserDocumentSnapshot.read(page({ throw closed }), 200, {}, { 0 })
            fail("A closed page must fail immediately")
        } catch (error: PlaywrightException) { assertSame(closed, error) }
    }

    @Test
    fun matchingTextFromOtherRuntimeExceptionIsNotRetried() {
        val failure = IllegalStateException(race)
        try {
            BrowserDocumentSnapshot.read(page({ throw failure }), 200, {}, { 0 })
            fail("Only the pinned Playwright snapshot error is recoverable")
        } catch (error: IllegalStateException) { assertSame(failure, error) }
    }

    @Test
    fun timeoutWithMatchingTextIsNotRetried() {
        val failure = TimeoutError(race)
        try {
            BrowserDocumentSnapshot.read(page({ throw failure }), 200, {}, { 0 })
            fail("A browser operation timeout must not be swallowed")
        } catch (error: TimeoutError) { assertSame(failure, error) }
    }

    @Test
    fun networkDenialDuringContentReadWinsOverNavigationRace() {
        var blocked = false
        val denied = BrowserNetworkPolicyViolation("Generated blocked resource")
        try {
            BrowserDocumentSnapshot.read(page({ blocked = true; throw PlaywrightException(race) }), 200,
                { if (blocked) throw denied }, { 0 })
            fail("Policy failures must not be retried")
        } catch (error: BrowserNetworkPolicyViolation) { assertSame(denied, error) }
    }

    @Test
    fun networkDenialDuringWaitStopsBeforeAnotherSnapshot() {
        var blocked = false
        val denied = BrowserNetworkPolicyViolation("Generated blocked redirect")
        try {
            BrowserDocumentSnapshot.read(page({ throw PlaywrightException(race) }, { blocked = true }), 200,
                { if (blocked) throw denied }, { 0 })
            fail("A blocked redirect must stop before load state or another content read")
        } catch (error: BrowserNetworkPolicyViolation) { assertSame(denied, error) }
    }

    @Test
    fun loadStateFailureIsNotSwallowedOrRetried() {
        val failure = PlaywrightException("Target closed during load-state wait")
        try {
            BrowserDocumentSnapshot.read(page({ throw PlaywrightException(race) }, {}, { throw failure }),
                200, {}, { 0 })
            fail("Load-state errors must not start a new retry loop")
        } catch (error: PlaywrightException) { assertSame(failure, error) }
    }

    @Test
    fun slowReadCannotReturnSuccessAfterSnapshotDeadline() {
        var now = 0L
        try {
            BrowserDocumentSnapshot.read(page({ now = 201_000_000L; "generated-html" }), 200, {}, { now })
            fail("A late result must not be called an in-budget success")
        } catch (_: TimeoutError) { /* Expected. No second content read. */ }
    }

    @Test(expected = IllegalArgumentException::class)
    fun zeroBudgetIsRejectedBeforeAccessingThePage() {
        BrowserDocumentSnapshot.read(page({ fail("Page must not be read"); "" }), 0, {}, { 0 })
    }
}
