package com.htmake.reader.db

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import mu.KotlinLogging
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.gson

private val logger = KotlinLogging.logger {}

/**
 * JSON file-based table implementation using getStorage/saveStorage pattern.
 */
class JSONTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {

    override fun readAll(): JsonArray? {
        if (cachedValue != null) {
            return cachedValue
        }
        val jsonStr = getStorage(userNameSpace, name)
        if (jsonStr != null && jsonStr.isNotEmpty()) {
            try {
                cachedValue = JsonArray(jsonStr)
            } catch (e: Exception) {
                logger.error("Failed to parse JSON for {}/{}: {}", userNameSpace, name, e.message)
                cachedValue = JsonArray()
            }
        } else {
            cachedValue = JsonArray()
        }
        return cachedValue
    }

    override fun <P> findBy(key: String, value: P, clazz: Class<T>): T? {
        val allData = readAll() ?: return null
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i)
            if (obj != null && obj.getValue(key) == value) {
                return gson.fromJson(obj.toString(), clazz)
            }
        }
        return null
    }

    override fun save(
        entity: T,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: ((JsonObject, T) -> Boolean)?
    ) {
        val allData = readAll() ?: JsonArray()
        var found = false

        if (checker != null) {
            for (i in 0 until allData.size()) {
                val obj = allData.getJsonObject(i)
                if (obj != null && checker(obj, entity)) {
                    // Update existing entry
                    val entityJson = JsonObject(gson.toJson(entity))
                    allData.list[i] = entityJson.map
                    found = true
                    break
                }
            }
        }

        if (!found) {
            val entityJson = JsonObject(gson.toJson(entity))
            allData.add(entityJson)
        }

        onCheckEnd?.invoke(entity, !found, allData)

        cachedValue = allData
        save()
    }

    override fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: ((JsonObject, T) -> Boolean)?
    ) {
        val allData = readAll() ?: JsonArray()

        for (entity in entities) {
            var found = false

            if (checker != null) {
                for (i in 0 until allData.size()) {
                    val obj = allData.getJsonObject(i)
                    if (obj != null && checker(obj, entity)) {
                        val entityJson = JsonObject(gson.toJson(entity))
                        allData.list[i] = entityJson.map
                        found = true
                        break
                    }
                }
            }

            if (!found) {
                val entityJson = JsonObject(gson.toJson(entity))
                allData.add(entityJson)
            }

            onCheckEnd?.invoke(entity, !found, allData)
        }

        cachedValue = allData
        save()
    }

    override fun delete(predicate: (JsonObject) -> Boolean) {
        val allData = readAll() ?: return
        val newData = JsonArray()
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i)
            if (obj != null && !predicate(obj)) {
                newData.add(obj)
            }
        }
        cachedValue = newData
        save()
    }

    override fun save() {
        val data = cachedValue ?: return
        try {
            saveStorage(userNameSpace, name, value = data)
        } catch (e: Exception) {
            logger.error("Failed to save JSON for {}/{}: {}", userNameSpace, name, e.message)
        }
    }
}
