package com.htmake.reader.api.controller

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import com.htmake.reader.utils.BrowserNetworkPolicy
import java.net.InetAddress
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SourceSubscriptionControllerTest {
    @Test
    fun normalizesLegacyStorageWithoutChangingItsDefaultEnabledMeaning() {
        val normalized = normalizeSourceSubscriptions(JsonArray().add(JsonObject()
            .put("name", "旧订阅")
            .put("link", "https://example.org/sources.json")
            .put("lastSyncTime", 42L)))

        val item = normalized.getJsonObject(0)
        assertEquals("https://example.org/sources.json", item.getString("url"))
        assertEquals("旧订阅", item.getString("name"))
        assertTrue(item.getBoolean("enabled"))
        assertEquals(42L, item.getLong("lastSyncTime").toLong())
    }

    @Test
    fun persistsBothVue3AndLegacyUrlFields() {
        val record = sourceSubscriptionRecord("https://example.org/sub.json", "示例", false, 99L)
        assertEquals("https://example.org/sub.json", record.getString("url"))
        assertEquals("https://example.org/sub.json", record.getString("link"))
        assertFalse(record.getBoolean("enabled"))
        assertEquals(99L, record.getLong("lastSyncTime").toLong())
    }

    @Test
    fun selectedSourcesAreDeduplicatedAndCannotInventEntries() {
        val remote = JsonArray()
            .add(JsonObject().put("bookSourceUrl", "https://example.org/a").put("bookSourceName", "A"))
            .add(JsonObject().put("bookSourceUrl", "https://example.org/a").put("bookSourceName", "A2"))
            .add(JsonObject().put("bookSourceUrl", "https://example.org/b").put("bookSourceName", "B"))

        val selected = selectedRemoteBookSources(remote, setOf("https://example.org/b", "https://example.org/missing"))
        assertEquals(1, selected.size())
        assertEquals("https://example.org/b", selected.getJsonObject(0).getString("bookSourceUrl"))
    }

    @Test
    fun disabledSubscriptionIsSkippedButHistoricalRecordKeepsRefreshing() {
        assertTrue(isSourceSubscriptionAutoRefreshEnabled(JsonObject().put("link", "https://example.org/old")))
        assertFalse(isSourceSubscriptionAutoRefreshEnabled(JsonObject().put("enabled", false)))
        assertFalse(isManagedSourceSubscription(JsonObject().put("link", "https://example.org/old")))
        assertTrue(isManagedSourceSubscription(JsonObject().put("url", "https://example.org/new").put("link", "https://example.org/new")))
    }

    @Test
    fun newSubscriptionRequiresAPublicResolvedTarget() {
        val publicPolicy = BrowserNetworkPolicy(resolve = { arrayOf(InetAddress.getByName("8.8.8.8")) })
        assertEquals("example.org", validateSourceSubscriptionTarget("https://example.org/list.json", publicPolicy).host)

        val privatePolicy = BrowserNetworkPolicy(resolve = { arrayOf(InetAddress.getByName("127.0.0.1")) })
        try {
            validateSourceSubscriptionTarget("https://example.org/list.json", privatePolicy)
            throw AssertionError("private target should be rejected")
        } catch (_: RemoteSourceSubscriptionException) {
            // expected
        }
    }
}
