package com.htmake.reader.api.controller

import org.junit.Assert.assertEquals
import org.junit.Test

class RemoteBookSourcePayloadTest {

    @Test
    fun acceptsAnArrayOfSources() {
        val sources = parseRemoteBookSources(
            """[{"bookSourceUrl":"https://example.org/a"},{"bookSourceUrl":"https://example.org/b"}]"""
        )
        assertEquals(2, sources.size())
    }

    @Test
    fun acceptsOneSourceObject() {
        val sources = parseRemoteBookSources("""{"bookSourceUrl":"https://example.org/a"}""")
        assertEquals(1, sources.size())
    }

    @Test(expected = IllegalArgumentException::class)
    fun rejectsEmptySourceList() {
        parseRemoteBookSources("[]")
    }

    @Test(expected = IllegalArgumentException::class)
    fun rejectsSourcesWithoutUrls() {
        parseRemoteBookSources("""[{"bookSourceName":"missing URL"}]""")
    }
}
