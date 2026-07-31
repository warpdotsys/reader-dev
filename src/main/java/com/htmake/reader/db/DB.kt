package com.htmake.reader.db

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject

/**
 * Abstract database class providing a base for JSON and SQL table implementations.
 */
open class DB<T>(
    val userNameSpace: String,
    val name: String
) {
    var cachedValue: JsonArray = JsonArray()

    open fun readAll(): JsonArray {
        return JsonArray()
    }

    open fun <P> findBy(field: String, value: P, clazz: Class<T>): T? {
        return null
    }

    open fun save(
        entity: T,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)? = null,
        checker: (JsonObject, T) -> Boolean
    ) {
    }

    open fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)? = null,
        checker: (JsonObject, T) -> Boolean
    ) {
    }

    open fun delete(predicate: (JsonObject) -> Boolean) {
    }

    open fun save() {
    }

    companion object {
        fun <T> table(userNameSpace: String, name: String, driver: String = "JSON"): DB<T> {
            return if (driver.equals("SQL", ignoreCase = true)) {
                SQLTable(userNameSpace, name)
            } else {
                JSONTable(userNameSpace, name)
            }
        }
    }
}
