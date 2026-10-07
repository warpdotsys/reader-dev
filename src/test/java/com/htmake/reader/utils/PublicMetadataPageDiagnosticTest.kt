package com.htmake.reader.utils

import com.google.gson.Gson
import com.google.gson.JsonObject
import io.legado.app.adapters.DefaultAdpater
import io.legado.app.adapters.ReaderAdapterHelper
import io.legado.app.adapters.ReaderAdapterInterface
import io.legado.app.data.entities.Book
import io.legado.app.data.entities.BookSource
import io.legado.app.model.analyzeRule.AnalyzeRule
import io.legado.app.model.webBook.WebBook
import io.legado.app.utils.htmlFormat
import io.vertx.core.json.JsonObject as VertxJsonObject
import kotlinx.coroutines.runBlocking
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
    private lateinit var snapshotRule: String
    private lateinit var sourceTemplate: JsonObject
    private val keys = setOf("nameSelectorPresent", "authorSelectorPresent", "coverSelectorPresent",
        "bodyHasText", "bodyHasExpectedBookTitle", "bodyHasExpectedAuthor",
        "bodyHasSafetyPhrase", "captchaContainerPresent")
    private val snapshotKeys = setOf("rawEmpty", "rawWhitespaceOnly", "rawHtmlTagHint", "rawHeadTagHint",
        "rawBodyTagHint", "headTitleElementPresent", "headScriptElementPresent", "bodyHasChildElements", "frameElementPresent")

    @Before
    fun setUp() {
        val probe = File("scripts/probe-public-metadata.py").readText(Charsets.UTF_8)
        val marker = "PAGE_DIAGNOSTIC_RULE = \"\"\""
        assertEquals(2, probe.split(marker).size)
        rule = probe.substringAfter(marker).substringBefore("\"\"\"")
        val snapshotMarker = "SNAPSHOT_STRUCTURE_RULE = r\"\"\""
        assertEquals(2, probe.split(snapshotMarker).size)
        snapshotRule = probe.substringAfter(snapshotMarker).substringBefore("\"\"\"")
        val sourceMarker = "PUBLIC_SOURCE_TEMPLATE = \"\"\""
        assertEquals(2, probe.split(sourceMarker).size)
        sourceTemplate = Gson().fromJson(probe.substringAfter(sourceMarker).substringBefore("\"\"\""), JsonObject::class.java)
        sourceTemplate.getAsJsonObject("ruleBookInfo").addProperty("intro", rule)
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

    private fun observeSnapshot(html: String): JsonObject {
        val raw = AnalyzeRule(Book()).setContent(html, "https://example.org/generated/")
            .getString(snapshotRule).htmlFormat()
        assertFalse(raw.contains("PRIVATE"))
        val value = Gson().fromJson(raw, JsonObject::class.java)
        assertEquals(setOf("pageDiagnostics", "snapshotStructure"), value.entrySet().map { it.key }.toSet())
        for ((group, expected) in listOf("pageDiagnostics" to keys, "snapshotStructure" to snapshotKeys)) {
            val fields = value.getAsJsonObject(group)
            assertEquals(expected, fields.entrySet().map { it.key }.toSet())
            fields.entrySet().forEach { assertTrue(it.value.isJsonPrimitive && it.value.asJsonPrimitive.isBoolean) }
        }
        return value
    }

    // Same conversion and Jackson serialization used by saveBookSource, then
    // the same deserialization used by WebBook. Generated source and HTML only.
    private fun savedSource(): String = VertxJsonObject.mapFrom(
        BookSource.fromJson(sourceTemplate.toString()).getOrThrow()).encode()

    @Test
    fun exactProbeSourceRetainsRulesThroughSaveAndReaderDeserialization() {
        val source = BookSource.fromJson(savedSource()).getOrThrow()
        assertEquals("https://www.qidian.com", source.bookSourceUrl)
        assertEquals(false, source.enabledCookieJar)
        assertEquals("#bookName@text", source.getBookInfoRule().name)
        assertEquals(".book-info-top .book-meta .author@text", source.getBookInfoRule().author)
        assertEquals("#bookImg img@src", source.getBookInfoRule().coverUrl)
        assertEquals(rule, source.getBookInfoRule().intro)
    }

    @Test
    fun missingTocMarkerCharacterizesExistingLegacyConversionWithoutChangingIt() {
        sourceTemplate.remove("ruleToc")
        val source = BookSource.fromJson(sourceTemplate.toString()).getOrThrow()
        assertTrue(source.getBookInfoRule().name.isNullOrEmpty())
        assertTrue(source.getBookInfoRule().author.isNullOrEmpty())
        assertTrue(source.getBookInfoRule().intro.isNullOrEmpty())
        assertTrue(source.getBookInfoRule().coverUrl.isNullOrEmpty())
    }

    @Test
    fun savedProbeSourceParsesGeneratedMetadataAndDiagnosticViaWebBook() = runBlocking {
        val book = Book().also {
            it.bookUrl = "https://example.org/generated/"
            it.infoHtml = """<h1 id="bookName">黎明之剑</h1>
                <section class="book-info-top"><span class="book-meta"><a class="author">远瞳</a></span></section>
                <div id="bookImg"><img src="https://example.org/generated-cover"></div>"""
        }
        WebBook(savedSource(), debugLog = false, userNameSpace = "generated-only").getBookInfo(book)
        assertEquals("黎明之剑", book.name)
        assertEquals("远瞳", book.author)
        assertEquals("https://example.org/generated-cover", book.coverUrl)
        val value = Gson().fromJson(book.intro, JsonObject::class.java)
        assertEquals(keys, value.entrySet().map { it.key }.toSet())
        keys.filterNot { it == "bodyHasSafetyPhrase" || it == "captchaContainerPresent" }
            .forEach { assertTrue(value[it].asBoolean) }
        assertFalse(value["bodyHasSafetyPhrase"].asBoolean)
        assertFalse(value["captchaContainerPresent"].asBoolean)
    }

    @Test
    fun savedSourceKeepsDiagnosticForAnEmptyGeneratedDocument() = runBlocking {
        val book = Book().also {
            it.bookUrl = "https://example.org/generated-empty/"
            it.infoHtml = "<html><body></body></html>"
        }
        WebBook(savedSource(), debugLog = false, userNameSpace = "generated-only").getBookInfo(book)
        assertEquals("", book.name)
        assertEquals("", book.author)
        val value = Gson().fromJson(book.intro, JsonObject::class.java)
        assertEquals(keys, value.entrySet().map { it.key }.toSet())
        keys.forEach { assertFalse(value[it].asBoolean) }
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

    @Test
    fun emptyRawStringIsDistinctFromJsoupSynthesizedDocumentNodes() {
        val value = observeSnapshot("")
        val structure = value.getAsJsonObject("snapshotStructure")
        assertTrue(structure["rawEmpty"].asBoolean)
        snapshotKeys.filterNot { it == "rawEmpty" }.forEach { assertFalse(structure[it].asBoolean) }
        keys.forEach { assertFalse(value.getAsJsonObject("pageDiagnostics")[it].asBoolean) }
    }

    @Test
    fun whitespaceOnlyRawStringIsNotReportedAsZeroLength() {
        val structure = observeSnapshot(" \n\t ").getAsJsonObject("snapshotStructure")
        assertTrue(structure["rawWhitespaceOnly"].asBoolean)
        snapshotKeys.filterNot { it == "rawWhitespaceOnly" }.forEach { assertFalse(structure[it].asBoolean) }
    }

    @Test
    fun explicitBlankDocumentIsDistinctFromEmptyRawString() {
        val structure = observeSnapshot("<html><head></head><body></body></html>").getAsJsonObject("snapshotStructure")
        val present = setOf("rawHtmlTagHint", "rawHeadTagHint", "rawBodyTagHint")
        snapshotKeys.forEach { assertEquals(present.contains(it), structure[it].asBoolean) }
    }

    @Test
    fun headOnlyScriptAndTitleAreObservedWithoutLeakingOrExecutingContent() {
        val value = observeSnapshot("<head><title>PRIVATE_TITLE</title><script>throw 'PRIVATE_SCRIPT';</script></head>")
        val structure = value.getAsJsonObject("snapshotStructure")
        val present = setOf("rawHeadTagHint", "headTitleElementPresent", "headScriptElementPresent")
        snapshotKeys.forEach { assertEquals(present.contains(it), structure[it].asBoolean) }
        keys.forEach { assertFalse(value.getAsJsonObject("pageDiagnostics")[it].asBoolean) }
    }

    @Test
    fun frameAndElementStructureDoesNotRevealUrlsOrClaimAuthentication() {
        val value = observeSnapshot("<body><iframe src='https://example.org/?ticket=PRIVATE'></iframe></body>")
        val structure = value.getAsJsonObject("snapshotStructure")
        assertTrue(structure["frameElementPresent"].asBoolean)
        assertTrue(structure["bodyHasChildElements"].asBoolean)
        assertTrue(structure["rawBodyTagHint"].asBoolean)
        assertFalse(value.has("authenticated"))
        assertFalse(value.getAsJsonObject("pageDiagnostics")["bodyHasText"].asBoolean)
    }

    @Test
    fun rawTagHintsCanComeFromCommentsAndAreNotAParsedPageVerdict() {
        val structure = observeSnapshot("<!-- <html><head><body> PRIVATE_COMMENT -->").getAsJsonObject("snapshotStructure")
        for (key in setOf("rawHtmlTagHint", "rawHeadTagHint", "rawBodyTagHint")) assertTrue(structure[key].asBoolean)
        assertFalse(structure["bodyHasChildElements"].asBoolean)
        assertFalse(structure["headScriptElementPresent"].asBoolean)
    }

    @Test
    fun detailsSourceRoundtripsAndParsesGeneratedMetadataViaRealWebBook() = runBlocking {
        sourceTemplate.getAsJsonObject("ruleBookInfo").addProperty("intro", snapshotRule)
        val saved = savedSource()
        assertEquals(snapshotRule, BookSource.fromJson(saved).getOrThrow().getBookInfoRule().intro)
        val book = Book().also {
            it.bookUrl = "https://example.org/generated-details/"
            it.infoHtml = """<html><head><script>throw 'PRIVATE_SCRIPT';</script></head><body>
                <h1 id="bookName">黎明之剑</h1>
                <section class="book-info-top"><span class="book-meta"><a class="author">远瞳</a></span></section>
                <div id="bookImg"><img src="https://example.org/generated-cover"></div></body></html>"""
        }
        WebBook(saved, debugLog = false, userNameSpace = "generated-only").getBookInfo(book)
        assertEquals("黎明之剑", book.name)
        assertEquals("远瞳", book.author)
        assertEquals("https://example.org/generated-cover", book.coverUrl)
        val intro = requireNotNull(book.intro)
        assertFalse(intro.contains("PRIVATE"))
        val value = Gson().fromJson(intro, JsonObject::class.java)
        assertEquals(keys, value.getAsJsonObject("pageDiagnostics").entrySet().map { it.key }.toSet())
        assertEquals(snapshotKeys, value.getAsJsonObject("snapshotStructure").entrySet().map { it.key }.toSet())
        assertTrue(value.getAsJsonObject("snapshotStructure")["headScriptElementPresent"].asBoolean)
        assertTrue(value.getAsJsonObject("pageDiagnostics")["bodyHasExpectedBookTitle"].asBoolean)
        assertTrue(value.getAsJsonObject("pageDiagnostics")["bodyHasExpectedAuthor"].asBoolean)
    }
}
