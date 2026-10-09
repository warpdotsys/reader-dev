package com.htmake.reader.api.controller

import com.htmake.reader.api.ReturnData
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.withStorageWriteLock
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject

private val editableHttpTtsStrings = setOf("name", "url", "contentType", "concurrentRate",
    "loginUrl", "loginUi", "header", "jsLib", "loginCheckJs")

/** Additive edit contract. Legacy save/delete remain name-keyed, including their old defaults. */
internal fun updateStoredHttpTts(userNameSpace: String, original: JsonObject, updated: JsonObject): ReturnData {
    val error = ReturnData()
    if (original.getValue("name") !is String || original.getString("name").isEmpty() ||
        original.getValue("id") !is Number || updated.getValue("id") !is Number) {
        return error.setErrorMsg("参数错误：需要原始听书源记录")
    }
    if (updated.fieldNames().any { it !in editableHttpTtsStrings && it != "id" && it != "enabledCookieJar" } ||
        editableHttpTtsStrings.any { updated.containsKey(it) && updated.getValue(it) != null && updated.getValue(it) !is String } ||
        (updated.getValue("enabledCookieJar") != null && updated.getValue("enabledCookieJar") !is Boolean)) {
        return error.setErrorMsg("参数错误：听书源字段类型不正确")
    }
    if ((updated.getValue("name") as? String).isNullOrBlank()) return error.setErrorMsg("名称不能为空")
    if ((updated.getValue("url") as? String).isNullOrBlank()) return error.setErrorMsg("链接不能为空")

    return withStorageWriteLock("data", userNameSpace, "httpTTS") {
        val all = getStorage("data", userNameSpace, "httpTTS")?.let { JsonArray(it) } ?: JsonArray()
        val matching = (0 until all.size()).filter { all.getJsonObject(it).getValue("name") == original.getString("name") }
        if (matching.size != 1) {
            return@withStorageWriteLock error.setErrorMsg("听书源已删除或名称不唯一，请重新加载后编辑")
        }
        val index = matching.single()
        val current = all.getJsonObject(index)
        // type belongs to Vue's display adapter, not the legacy entity. Keep all
        // other fields in the snapshot so a concurrent edit cannot be overwritten.
        val expected = original.copy().apply { remove("type") }
        val storedSnapshot = current.copy().apply { remove("type") }
        if (storedSnapshot != expected || current.getValue("id") != updated.getValue("id")) {
            return@withStorageWriteLock error.setErrorMsg("听书源已被修改，请重新加载后编辑")
        }
        val newName = updated.getString("name")
        if ((0 until all.size()).any { it != index && all.getJsonObject(it).getValue("name") == newName }) {
            return@withStorageWriteLock error.setErrorMsg("听书源名称已存在，请使用不同名称")
        }
        // Preserve the stable legacy id, unrelated records and unknown stored
        // fields. In particular, do not pass through the lossy legacy fromJson.
        val replacement = current.copy()
        updated.forEach { field -> if (field.key != "id") replacement.put(field.key, field.value) }
        val previousTime = (current.getValue("lastUpdateTime") as? Number)?.toLong() ?: 0L
        if (previousTime == Long.MAX_VALUE) {
            return@withStorageWriteLock error.setErrorMsg("听书源更新时间无效，请检查原记录")
        }
        replacement.put("lastUpdateTime", maxOf(System.currentTimeMillis(), previousTime + 1L))
        all.list[index] = replacement
        saveStorage("data", userNameSpace, "httpTTS", value = all)
        ReturnData().setData("")
    }
}
