package com.htmake.reader.utils

import com.google.gson.Gson
import com.google.gson.JsonObject
import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.data.entities.Book
import io.legado.app.model.analyzeRule.AnalyzeRule
import io.legado.app.utils.htmlFormat
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

/** Exact probe rule through the real JVM parser, using generated HTML only. */
class PublicMetadataPageDiagnosticTest {
    @get:Rule val temp = TemporaryFolder()
    private lateinit var originalUserDir: String
    private lateinit var originalAdapter: ReaderAdapterInterface
    private lateinit var rule: String
    private val keys = setOf("nameSelectorPresent", "authorSelectorPresent", "coverSelectorPresent",
        "bodyHasText", "bodyHasExpectedBookTitle", "bodyHasExpectedAuthor",
        "bodyHasSafetyPhrase", "captchaContainerPresent")

    @Before
    fun setUp() {
        val probe = File("scripts/probe-public-metadata.py").readText(Charsets.UTF_8)
        val marker = "PAGE_DIAGNOSTIC_RULE = \"\"\""
        assertEquals(2, probe.split(marker).size)
        rule = probe.substringAfter(marker).substringBefore("\"\"\"")
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

    private fun observe(html: String): JsonObject {
        val analyzer = AnalyzeRule(Book()).setContent(html, "https://example.org/generated/")
        val raw = analyzer.getString(rule).htmlFormat()
        assertFalse(raw.contains("PRIVATE"))
        val value = Gson().fromJson(raw, JsonObject::class.java)
        assertEquals(keys, value.entrySet().map { it.key }.toSet())
        value.entrySet().forEach { assertTrue(it.value.isJsonPrimitive && it.value.asJsonPrimitive.isBoolean) }
        return value
    }

    @Test
    fun generatedNormalMetadataIsObservedWithoutChangingItsSelectors() {
        val value = observe("""<h1 id="bookName">黎明之剑</h1>
            <section class="book-info-top"><span class="book-meta"><a class="author">远瞳</a></span></section>
            <div id="bookImg"><img src="https://example.org/generated-cover"></div>""")
        keys.filterNot { it == "bodyHasSafetyPhrase" || it == "captchaContainerPresent" }
            .forEach { assertTrue(value[it].asBoolean) }
        assertFalse(value["bodyHasSafetyPhrase"].asBoolean)
        assertFalse(value["captchaContainerPresent"].asBoolean)
    }

    @Test
    fun generatedSafetyPageHasHintsButDoesNotGainMetadata() {
        val value = observe("<p>拖动滑块完成拼图</p><div class='geetest_panel'>PRIVATE_TOKEN</div>")
        assertTrue(value["bodyHasSafetyPhrase"].asBoolean)
        assertTrue(value["captchaContainerPresent"].asBoolean)
        for (key in setOf("nameSelectorPresent", "authorSelectorPresent", "coverSelectorPresent",
            "bodyHasExpectedBookTitle", "bodyHasExpectedAuthor")) assertFalse(value[key].asBoolean)
    }

    @Test
    fun generatedEmptySnapshotRemainsEmpty() {
        val value = observe("")
        keys.forEach { assertFalse(value[it].asBoolean) }
    }

    @Test
    fun scriptTextIsNotMistakenForBodyOrExecuted() {
        val value = observe("<script>throw 'PRIVATE_SCRIPT'; var x='安全验证 黎明之剑 远瞳';</script>")
        keys.forEach { assertFalse(value[it].asBoolean) }
    }

    @Test
    fun hintsDoNotClaimVisibilityOrSuccessfulAuthentication() {
        val value = observe("<div style='display:none' class='geetest_panel'>安全验证</div><p>Generated</p>")
        assertTrue(value["bodyHasSafetyPhrase"].asBoolean)
        assertTrue(value["captchaContainerPresent"].asBoolean)
        assertFalse(value.has("visibleCaptcha"))
        assertFalse(value.has("authenticated"))
    }
}
