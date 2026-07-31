package com.htmake.reader.db

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage

/**
 * SQL-based table implementation.
 * The packaged application persists this table through the storage backend.
 */
class SQLTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {

    override fun readAll(): JsonArray {
        val dataList = asJsonArray(getStorage("data", userNameSpace, name)) ?: JsonArray()
        cachedValue = dataList
        return dataList
    }

    override fun <P> findBy(field: String, value: P, clazz: Class<T>): T? {
        val dataList = readAll()
        for (i in 0 until dataList.size()) {
            val obj = dataList.getJsonObject(i)
            if (value == obj.getValue(field)) {
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
        var dataList = readAll()
        var existingIndex = -1
        for (i in 0 until dataList.size()) {
            if (checker(dataList.getJsonObject(i), entity)) {
                existingIndex = i
                break
            }
        }
        onCheckEnd?.invoke(entity, existingIndex >= 0, dataList)
        if (existingIndex >= 0) {
            dataList.list[existingIndex] = JsonObject.mapFrom(entity)
            dataList = JsonArray(dataList.list)
        } else {
            dataList.add(JsonObject.mapFrom(entity))
        }
        cachedValue = dataList
        save()
    }

    override fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: (JsonObject, T) -> Boolean
    ) {
        var dataList = readAll()
        var existingIndex = -1
        for (entity in entities) {
            for (i in 0 until dataList.size()) {
                if (checker(dataList.getJsonObject(i), entity)) {
                    existingIndex = i
                    break
                }
            }
            onCheckEnd?.invoke(entity, existingIndex >= 0, dataList)
            if (existingIndex >= 0) {
                dataList.list[existingIndex] = JsonObject.mapFrom(entity)
                dataList = JsonArray(dataList.list)
            } else {
                dataList.add(JsonObject.mapFrom(entity))
            }
        }
        cachedValue = dataList
        save()
    }

    override fun delete(predicate: (JsonObject) -> Boolean) {
        val dataList = readAll()
        val removeIndexes = ArrayList<Int>()
        for (i in 0 until dataList.size()) {
            if (predicate(dataList.getJsonObject(i))) {
                removeIndexes.add(i)
            }
        }
        for (index in removeIndexes) {
            dataList.remove(index)
        }
        cachedValue = dataList
        save()
    }

    override fun save() {
        saveStorage("data", userNameSpace, name, value = cachedValue)
    }
}
