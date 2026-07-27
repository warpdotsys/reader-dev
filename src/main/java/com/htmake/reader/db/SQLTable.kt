package com.htmake.reader.db

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import mu.KotlinLogging

private val logger = KotlinLogging.logger {}

/**
 * SQL-based table implementation (stub - not yet implemented).
 */
class SQLTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {

    override fun readAll(): JsonArray? {
        throw NotImplementedError("SQLTable.readAll() is not implemented yet")
    }

    override fun <P> findBy(key: String, value: P, clazz: Class<T>): T? {
        throw NotImplementedError("SQLTable.findBy() is not implemented yet")
    }

    override fun save(
        entity: T,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: ((JsonObject, T) -> Boolean)?
    ) {
        throw NotImplementedError("SQLTable.save() is not implemented yet")
    }

    override fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: ((JsonObject, T) -> Boolean)?
    ) {
        throw NotImplementedError("SQLTable.saveMulti() is not implemented yet")
    }

    override fun delete(predicate: (JsonObject) -> Boolean) {
        throw NotImplementedError("SQLTable.delete() is not implemented yet")
    }

    override fun save() {
        throw NotImplementedError("SQLTable.save() is not implemented yet")
    }
}
