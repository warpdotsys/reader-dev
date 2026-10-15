package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.utils.gson
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import org.junit.Assert.*
import org.junit.Test

class LicenseTrialResponseTest {
    @Test fun successfulClaimRetainsDataKeyWithoutGsonMapWrapper() {
        val data = JsonObject().put("key", "generated-key")
        // Reproduce the actual RoutingContext.success serializer mismatch.
        val broken = JsonObject(gson.toJson(ReturnData().setData(data)))
        assertNull(broken.getJsonObject("data").getString("key"))
        assertNotNull(broken.getJsonObject("data").getJsonObject("map"))
        val forwarded = JsonObject(trialSuccessResponse(JsonObject().put("data", data)).encode())
        assertTrue(forwarded.getBoolean("isSuccess"))
        assertEquals("", forwarded.getString("errorMsg"))
        assertEquals("generated-key", forwarded.getJsonObject("data").getString("key"))
        assertFalse(forwarded.getJsonObject("data").containsKey("map"))
    }

    @Test fun successfulSendRetainsEmptyDataAndNestedJsonTypes() {
        assertEquals("", trialSuccessResponse(JsonObject().put("data", "")).getString("data"))
        val nested = JsonObject().put("number", 15).put("flag", true)
            .put("items", JsonArray().add(JsonObject().put("text", "生成中文")))
        val forwarded = JsonObject(trialSuccessResponse(JsonObject().put("data", nested)).encode())
        assertEquals(nested, forwarded.getJsonObject("data"))
    }
}
