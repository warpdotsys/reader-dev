package com.htmake.reader.utils

import com.microsoft.playwright.Page
import com.microsoft.playwright.PlaywrightException
import com.microsoft.playwright.TimeoutError
import com.microsoft.playwright.options.LoadState

/** Retry only Playwright's transient HTML snapshot race, never navigation or source scripts. */
internal object BrowserDocumentSnapshot {
    private const val NAVIGATION_RACE =
        "Unable to retrieve content because the page is navigating and changing the content."

    fun read(page: Page, timeoutMs: Int, checkNetwork: () -> Unit,
             nanoTime: () -> Long = System::nanoTime): String {
        require(timeoutMs > 0) { "browserTimeoutMs must be positive" }
        // One budget for this snapshot phase, not a new full timeout for each redirect.
        val deadline = nanoTime() + timeoutMs * 1_000_000L
        fun remainingMs(): Double {
            val remaining = (deadline - nanoTime()) / 1_000_000.0
            if (remaining <= 0) throw TimeoutError("本地 WebView 在 ${timeoutMs}ms 内未能读取稳定的页面内容")
            return remaining
        }

        while (true) {
            checkNetwork()
            remainingMs()
            val content = try {
                page.content()
            } catch (error: PlaywrightException) {
                // A rejected subresource/redirect is fatal even if Chromium also lost
                // its execution context. Do not hide policy failures behind retries.
                checkNetwork()
                if (error is TimeoutError || !error.message.orEmpty().contains(NAVIGATION_RACE)) throw error
                // Pump Playwright events on its own worker. A previously-loaded
                // document can otherwise make waitForLoadState resolve immediately.
                page.waitForTimeout(minOf(50.0, remainingMs()))
                checkNetwork()
                page.waitForLoadState(LoadState.DOMCONTENTLOADED,
                    Page.WaitForLoadStateOptions().setTimeout(remainingMs()))
                continue
            }
            checkNetwork()
            remainingMs()
            return content
        }
    }
}
