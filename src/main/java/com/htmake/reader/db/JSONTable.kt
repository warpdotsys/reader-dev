package com.htmake.reader.db

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.asJsonArray

/**
 * JSON file-based table implementation using getStorage/saveStorage pattern.
 */
class JSONTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {

    override fun readAll(): JsonArray {
        val dataList = asJsonArray(getStorage("data", userNameSpace, name)) ?: JsonArray()
        cachedValue = dataList
        return dataList
    }

    override fun <P> findBy(key: String, value: P, clazz: Class<T>): T? {
        val allData = readAll()
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i)
            if (value == obj.getValue(key)) {
                return obj.mapTo(clazz)
            }
        }
        return null
    }

    override fun save(
        entity: T,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: (JsonObject, T) -> Boolean
    ) {
        var allData = readAll()
        var existingIndex = -1

        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i)
            if (checker(obj, entity)) {
                existingIndex = i
                break
            }
        }

        onCheckEnd?.invoke(entity, existingIndex >= 0, allData)

        if (existingIndex >= 0) {
            allData.list[existingIndex] = JsonObject.mapFrom(entity)
            allData = JsonArray(allData.list)
        } else {
            allData.add(JsonObject.mapFrom(entity))
        }

        cachedValue = allData
        save()
    }

    override fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: (JsonObject, T) -> Boolean
    ) {
        var allData = readAll()

        for (entity in entities) {
            var existingIndex = -1

            for (i in 0 until allData.size()) {
                val obj = allData.getJsonObject(i)
                if (checker(obj, entity)) {
                    existingIndex = i
                    break
                }
            }

            onCheckEnd?.invoke(entity, existingIndex >= 0, allData)

            if (existingIndex >= 0) {
                allData.list[existingIndex] = JsonObject.mapFrom(entity)
            } else {
                allData.add(JsonObject.mapFrom(entity))
            }
        }

        cachedValue = allData
        save()
    }

    override fun delete(predicate: (JsonObject) -> Boolean) {
        var allData = readAll()
        val removeIndexes = ArrayList<Int>()
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i)
            if (predicate(obj)) {
                removeIndexes.add(i)
            }
        }
        if (removeIndexes.isNotEmpty()) {
            val newData = JsonArray()
            for (i in 0 until allData.size()) {
                if (!removeIndexes.contains(i)) {
                    newData.add(allData.getJsonObject(i))
                }
            }
            allData = newData
        }
        cachedValue = allData
        save()
    }

    override fun save() {
        saveStorage("data", userNameSpace, name, value = cachedValue)
    }
}
