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
    var cachedValue: JsonArray? = null

    open fun readAll(): JsonArray? {
        return null
    }

    open fun <P> findBy(key: String, value: P, clazz: Class<T>): T? {
        return null
    }

    open fun save(
        entity: T,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)? = null,
        checker: ((JsonObject, T) -> Boolean)? = null
    ) {
    }

    open fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)? = null,
        checker: ((JsonObject, T) -> Boolean)? = null
    ) {
    }

    open fun delete(predicate: (JsonObject) -> Boolean) {
    }

    open fun save() {
    }

    companion object {
        @JvmStatic
        fun <T> table(userNameSpace: String, name: String, type: String = "json"): DB<T> {
            return when (type) {
                "sql" -> SQLTable(userNameSpace, name)
                else -> JSONTable(userNameSpace, name)
            }
        }
    }
}
