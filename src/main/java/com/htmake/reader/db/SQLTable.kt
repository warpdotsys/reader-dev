package com.htmake.reader.db

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import mu.KotlinLogging

private val logger = KotlinLogging.logger {}

/**
 * SQL-based table implementation.
 * Since no SQL database is configured, delegates all operations to JSONTable.
 */
class SQLTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {

    private val delegate = JSONTable<T>(userNameSpace, name)

    override fun readAll(): JsonArray? {
        return delegate.readAll()
    }

    override fun <P> findBy(key: String, value: P, clazz: Class<T>): T? {
        return delegate.findBy(key, value, clazz)
    }

    override fun save(
        entity: T,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: ((JsonObject, T) -> Boolean)?
    ) {
        delegate.save(entity, onCheckEnd, checker)
    }

    override fun saveMulti(
        entities: Array<T>,
        onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
        checker: ((JsonObject, T) -> Boolean)?
    ) {
        delegate.saveMulti(entities, onCheckEnd, checker)
    }

    override fun delete(predicate: (JsonObject) -> Boolean) {
        delegate.delete(predicate)
    }

    override fun save() {
        delegate.save()
    }
}
