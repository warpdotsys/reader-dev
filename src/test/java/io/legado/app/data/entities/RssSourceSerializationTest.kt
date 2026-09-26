package io.legado.app.data.entities

import io.vertx.core.json.JsonObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class RssSourceSerializationTest {

    @Test
    fun sourceCanBePersistedWithoutSerializingItsSelfReference() {
        val source = RssSource(sourceUrl = "https://example.invalid/rss", sourceName = "RSS test")
        val json = JsonObject.mapFrom(source)

        assertEquals(source.sourceUrl, json.getString("sourceUrl"))
        assertEquals(source.sourceName, json.getString("sourceName"))
        assertFalse(json.containsKey("source"))
    }
}
