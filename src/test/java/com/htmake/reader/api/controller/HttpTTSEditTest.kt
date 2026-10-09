package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.getStorageFile
import com.htmake.reader.utils.gson
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.storageFinalPath
import com.htmake.reader.utils.withStorageWriteLock
import io.legado.app.data.entities.HttpTTS
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException

/** Actual generated JSON persistence; no original JAR, real user, cookie or speech request. */
class HttpTTSEditTest {
    @get:Rule val temp = TemporaryFolder()
    private lateinit var previousStorage: String

    @Before fun setUp() { previousStorage = storageFinalPath; storageFinalPath = temp.root.absolutePath }
    @After fun tearDown() { storageFinalPath = previousStorage }

    private fun list(namespace: String = "generated-a") = JsonArray(getStorage("data", namespace, "httpTTS") ?: "[]")
    private fun seed(): JsonObject {
        saveStorage("data", "generated-a", "httpTTS", value = JsonArray()
            .add(JsonObject().put("id", 1900000000001L).put("name", "生成旧名称").put("url", "http://127.0.0.1/old")
                .put("lastUpdateTime", 7L).put("jsLib", "generated-not-executed").put("enabledCookieJar", true)
                .put("generatedUnknown", JsonObject().put("keep", true))))
        return list().getJsonObject(0).copy()
    }
    private fun edit(original: JsonObject, name: String = "生成新名称") = JsonObject()
        .put("id", original.getValue("id")).put("name", name).put("url", "http://127.0.0.1/new")

    // CURD.list returns JsonArray.list; RoutingContext.success serializes that
    // ReturnData with the application's Gson, not JsonArray.toString().
    private fun listedSnapshot(): JsonObject = JsonObject(gson.toJson(ReturnData()
        .setData(DB.table<HttpTTS>("generated-a", "httpTTS").readAll().list)))
        .getJsonArray("data").getJsonObject(0).copy()

    private fun saveLegacy(entity: HttpTTS) = DB.table<HttpTTS>("generated-a", "httpTTS")
        .save(entity, checker = { row, value -> row.getString("name") == value.name })

    @Test fun listedWireSnapshotCanRenameDefaultLegacyEntityWithOmittedNullFields() {
        saveLegacy(HttpTTS(
            id = 1900000000001L, name = "生成旧名称", url = "http://127.0.0.1/old"))
        val stored = list().getJsonObject(0).copy()
        val original = listedSnapshot()
        assertTrue(stored.containsKey("contentType"))
        assertNull(stored.getValue("contentType"))
        assertFalse(original.containsKey("contentType"))
        val result = updateStoredHttpTts("generated-a", original, edit(original))
        assertTrue(result.errorMsg, result.isSuccess)
        val saved = list().getJsonObject(0)
        assertEquals(1, list().size())
        assertEquals("生成新名称", saved.getString("name"))
        assertEquals(stored.getValue("id"), saved.getValue("id"))
        assertTrue(saved.containsKey("contentType"))
        assertNull(saved.getValue("contentType"))
        assertEquals(stored.getValue("tag"), saved.getValue("tag"))
    }

    @Test fun listedWireSnapshotReportsNameCollisionWithoutChangingStoredBytes() {
        saveLegacy(HttpTTS(id = 1900000000001L, name = "生成旧名称", url = "http://127.0.0.1/old"))
        val original = listedSnapshot()
        saveLegacy(HttpTTS(id = 1900000000002L, name = "生成已占用", url = "http://127.0.0.1/other"))
        val before = getStorage("data", "generated-a", "httpTTS")
        val result = updateStoredHttpTts("generated-a", original, edit(original, "生成已占用"))
        assertFalse(result.isSuccess)
        assertEquals("听书源名称已存在，请使用不同名称", result.errorMsg)
        assertEquals(before, getStorage("data", "generated-a", "httpTTS"))
    }

    @Test fun listedWireSnapshotPreservesNestedNullsFalseZeroAndRejectsUnknownFieldChanges() {
        val stored = seed().put("enabledCookieJar", false).put("header", "")
            .put("generatedUnknown", JsonObject().put("keep", false).put("zero", 0)
                .put("omitted", null as String?))
        saveStorage("data", "generated-a", "httpTTS", value = JsonArray().add(stored))
        val original = listedSnapshot()
        assertFalse(original.getBoolean("enabledCookieJar"))
        assertEquals("", original.getString("header"))
        assertEquals(0, original.getJsonObject("generatedUnknown").getInteger("zero").toInt())
        assertFalse(original.getJsonObject("generatedUnknown").containsKey("omitted"))
        val changed = stored.copy()
        changed.getJsonObject("generatedUnknown").put("keep", true)
        saveStorage("data", "generated-a", "httpTTS", value = JsonArray().add(changed))
        val before = getStorage("data", "generated-a", "httpTTS")
        val stale = updateStoredHttpTts("generated-a", original, edit(original))
        assertFalse(stale.isSuccess)
        assertEquals("听书源已被修改，请重新加载后编辑", stale.errorMsg)
        assertEquals(before, getStorage("data", "generated-a", "httpTTS"))
        val fresh = listedSnapshot()
        val success = updateStoredHttpTts("generated-a", fresh, edit(fresh))
        assertTrue(success.errorMsg, success.isSuccess)
        val saved = list().getJsonObject(0)
        assertFalse(saved.getBoolean("enabledCookieJar"))
        assertEquals("", saved.getString("header"))
        assertEquals(changed.getJsonObject("generatedUnknown"), saved.getJsonObject("generatedUnknown"))
        assertTrue(saved.getJsonObject("generatedUnknown").containsKey("omitted"))
    }

    @Test fun legacyNameKeyedSaveReallyCreatesTwoRowsWhenTheNameChanges() {
        val table = DB.table<HttpTTS>("generated-a", "httpTTS")
        val checker: (JsonObject, HttpTTS) -> Boolean = { row, entity -> row.getString("name") == entity.name }
        table.save(HttpTTS(id = 1900000000001L, name = "生成旧名称", url = "http://127.0.0.1/old"), checker = checker)
        table.save(HttpTTS(id = 1900000000001L, name = "生成新名称", url = "http://127.0.0.1/new"), checker = checker)
        assertEquals(2, list().size())
        assertEquals(list().getJsonObject(0).getValue("id"), list().getJsonObject(1).getValue("id"))
    }

    @Test fun renameReplacesOneRecordPreservesIdUnknownFieldsAdvancedFieldsAndNamespace() {
        val original = seed()
        val result = updateStoredHttpTts("generated-a", original.copy().put("type", 0), edit(original))
        assertTrue(result.isSuccess)
        assertEquals("", result.data)
        val saved = list().getJsonObject(0)
        assertEquals(1, list().size())
        assertEquals("生成新名称", saved.getString("name"))
        assertEquals(original.getValue("id"), saved.getValue("id"))
        assertEquals(original.getValue("generatedUnknown"), saved.getValue("generatedUnknown"))
        assertEquals("generated-not-executed", saved.getString("jsLib"))
        assertTrue(saved.getBoolean("enabledCookieJar"))
        assertTrue(saved.getLong("lastUpdateTime") > 7L)
        assertFalse(saved.containsKey("type"))
        assertEquals(0, list("generated-b").size())
        assertEquals(temp.root.canonicalFile, getStorageFile("data", "generated-a", "httpTTS").parentFile.parentFile.parentFile.canonicalFile)
    }

    @Test fun collisionAndStaleEditLeaveStoredBytesUnchanged() {
        val original = seed()
        val records = list().add(JsonObject().put("id", 2).put("name", "生成已占用").put("url", "http://127.0.0.1/other"))
        saveStorage("data", "generated-a", "httpTTS", value = records)
        val before = getStorage("data", "generated-a", "httpTTS")
        assertFalse(updateStoredHttpTts("generated-a", original, edit(original, "生成已占用")).isSuccess)
        assertEquals(before, getStorage("data", "generated-a", "httpTTS"))
        assertTrue(updateStoredHttpTts("generated-a", original, edit(original)).isSuccess)
        val after = getStorage("data", "generated-a", "httpTTS")
        assertFalse(updateStoredHttpTts("generated-a", original, edit(original, "生成过期")).isSuccess)
        assertEquals(after, getStorage("data", "generated-a", "httpTTS"))
        assertEquals("生成已占用", list().getJsonObject(1).getString("name"))
    }

    @Test fun simultaneousEditsOfOneSnapshotCanOnlyCommitOnce() {
        val original = seed()
        val ready = CountDownLatch(2)
        val start = CountDownLatch(1)
        val pool = Executors.newFixedThreadPool(2)
        try {
            val results = (1..2).map { number -> pool.submit<Boolean> {
                ready.countDown(); check(start.await(5, TimeUnit.SECONDS))
                updateStoredHttpTts("generated-a", original.copy(), edit(original, "生成并发$number")).isSuccess
            } }
            check(ready.await(5, TimeUnit.SECONDS)); start.countDown()
            assertEquals(1, results.count { it.get(5, TimeUnit.SECONDS) })
            assertEquals(1, list().size())
        } finally { start.countDown(); pool.shutdownNow(); check(pool.awaitTermination(5, TimeUnit.SECONDS)) }
    }

    @Test fun identicalEditStillAdvancesTheSnapshotTimestampAndRejectsItsReplay() {
        val original = seed().put("lastUpdateTime", System.currentTimeMillis() + 60000L)
        saveStorage("data", "generated-a", "httpTTS", value = JsonArray().add(original))
        val unchanged = JsonObject().put("id", original.getValue("id"))
            .put("name", original.getString("name")).put("url", original.getString("url"))
        assertTrue(updateStoredHttpTts("generated-a", original, unchanged).isSuccess)
        assertEquals(original.getLong("lastUpdateTime") + 1L, list().getJsonObject(0).getLong("lastUpdateTime").toLong())
        assertFalse(updateStoredHttpTts("generated-a", original, unchanged).isSuccess)
    }

    @Test fun legacySaveUsesTheSameTransactionLockAndCannotOverwriteAnEditWithAnEarlierRead() {
        seed()
        val pool = Executors.newSingleThreadExecutor()
        val started = CountDownLatch(1)
        try {
            withStorageWriteLock("data", "generated-a", "httpTTS") {
                val save = pool.submit {
                    started.countDown()
                    DB.table<HttpTTS>("generated-a", "httpTTS").save(
                        HttpTTS(id = 2L, name = "生成并发新增", url = "http://127.0.0.1/other"),
                        checker = { row, entity -> row.getString("name") == entity.name })
                }
                check(started.await(5, TimeUnit.SECONDS))
                try { save.get(250, TimeUnit.MILLISECONDS); fail("legacy mutation must wait for the transaction lock") }
                catch (_: TimeoutException) { /* Actual writer waits before reading. */ }
                val original = list().getJsonObject(0)
                assertTrue(updateStoredHttpTts("generated-a", original, edit(original)).isSuccess)
            }
            pool.shutdown(); check(pool.awaitTermination(5, TimeUnit.SECONDS))
            assertEquals(2, list().size())
            assertEquals("生成新名称", list().getJsonObject(0).getString("name"))
            assertEquals("生成并发新增", list().getJsonObject(1).getString("name"))
        } finally { pool.shutdownNow(); check(pool.awaitTermination(5, TimeUnit.SECONDS)) }
    }

    @Test fun missingDuplicateOrInvalidRecordIsRejectedWithoutChangingBytes() {
        val original = seed()
        val before = getStorage("data", "generated-a", "httpTTS")
        val invalid = listOf(edit(original).put("name", ""), edit(original).put("url", null as String?),
            edit(original).put("header", 42), edit(original).put("enabledCookieJar", "true"),
            edit(original).put("id", 42), edit(original).put("unrecognized", "must-not-be-written"))
        invalid.forEach { assertFalse(updateStoredHttpTts("generated-a", original, it).isSuccess) }
        assertEquals(before, getStorage("data", "generated-a", "httpTTS"))
        assertFalse(updateStoredHttpTts("generated-b", original, edit(original)).isSuccess)
        assertFalse(getStorageFile("data", "generated-b", "httpTTS").exists())
        saveStorage("data", "generated-a", "httpTTS", value = list().add(original.copy()))
        val duplicateBytes = getStorage("data", "generated-a", "httpTTS")
        assertFalse(updateStoredHttpTts("generated-a", original, edit(original)).isSuccess)
        assertEquals(duplicateBytes, getStorage("data", "generated-a", "httpTTS"))
    }

    @Test fun malformedStorageIsNotReplacedWithAnEmptyOrPartialList() {
        val original = seed()
        saveStorage("data", "generated-a", "httpTTS", value = "generated-invalid-json")
        try { updateStoredHttpTts("generated-a", original, edit(original)); fail("Corrupt storage must surface") }
        catch (_: io.vertx.core.json.DecodeException) { /* Retain existing data for diagnosis. */ }
        assertEquals("generated-invalid-json", getStorage("data", "generated-a", "httpTTS"))
    }
}
