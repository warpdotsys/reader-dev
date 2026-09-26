package io.legado.app.model.rss

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class RssParserDefaultTest {

    @Test
    fun parsesAnRssItemWithTheBundledXmlParser() {
        val xml = """<?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0"><channel><title>Fixture feed</title><item>
            <title>Fixture article</title><link>https://example.invalid/article/1</link>
            <description>Fixed summary</description><pubDate>Tue, 10 Sep 2024 00:00:00 GMT</pubDate>
            </item></channel></rss>""".trimIndent()

        val (articles, next) = RssParserDefault.parseXML(
            "Fixture", xml, "https://example.invalid/feed.rss", null
        )

        assertEquals(1, articles.size)
        assertEquals("Fixture article", articles[0].title)
        assertEquals("https://example.invalid/article/1", articles[0].link)
        assertEquals("Fixed summary", articles[0].description)
        assertNull(next)
    }
}
