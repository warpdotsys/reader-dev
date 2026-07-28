package com.htmake.reader.api.controller

import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.gson

private val logger = KotlinLogging.logger {}

/**
 * Generic CURD interface providing default implementations for list/save/delete operations.
 * Uses DB<T> abstraction for persistence.
 */
interface CURD<T> {

    fun getTableName(): String

    fun convertToEntity(json: JsonObject): T {
        return gson.fromJson(json.toString(), getEntityClass())
    }

    fun convertToEntityList(json: String): Array<T> {
        @Suppress("UNCHECKED_CAST")
        return gson.fromJson(json, java.lang.reflect.Array.newInstance(getEntityClass(), 0).javaClass) as Array<T>
    }

    fun onList(list: JsonArray, userNameSpace: String): JsonArray {
        return list
    }

    fun checker(json: JsonObject, entity: T): Boolean

    fun onCheckEnd(entity: T, exists: Boolean, allData: JsonArray) {
    }

    fun beforeSave(entity: T, db: DB<T>): ReturnData? {
        return null
    }

    fun beforeAdd(entity: T, db: DB<T>): ReturnData? {
        return null
    }

    fun beforeDelete(entity: T, db: DB<T>): ReturnData? {
        return null
    }

    suspend fun checkUserAuth(context: RoutingContext): Boolean

    fun getUserNS(context: RoutingContext): String

    fun getEntityClass(): Class<T>

    suspend fun list(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setErrorMsg("NEED_LOGIN")
        }
        val userNS = getUserNS(context)
        val db = DB.table<T>(userNS, getTableName())
        val allData = db.readAll() ?: JsonArray()
        val result = onList(allData, userNS)
        return returnData.setData(result.list)
    }

    suspend fun save(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setErrorMsg("NEED_LOGIN")
        }
        val userNS = getUserNS(context)
        val body = context.bodyAsString
        if (body.isNullOrEmpty()) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val jsonObj = try {
            JsonObject(body)
        } catch (e: Exception) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val entity = convertToEntity(jsonObj)
        val db = DB.table<T>(userNS, getTableName())

        val beforeResult = beforeSave(entity, db)
        if (beforeResult != null) {
            return beforeResult
        }

        // Check if entity already exists
        val allData = db.readAll() ?: JsonArray()
        var found = false
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i)
            if (obj != null && checker(obj, entity)) {
                found = true
                break
            }
        }

        if (!found) {
            val addResult = beforeAdd(entity, db)
            if (addResult != null) {
                return addResult
            }
        }

        db.save(entity, { e, exists, data -> onCheckEnd(e, exists, data) }, { obj, e -> checker(obj, e) })
        return returnData.setData("")
    }

    suspend fun saveMulti(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setErrorMsg("NEED_LOGIN")
        }
        val userNS = getUserNS(context)
        val body = context.bodyAsString
        if (body.isNullOrEmpty()) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val entities = try {
            convertToEntityList(body)
        } catch (e: Exception) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val db = DB.table<T>(userNS, getTableName())

        for (entity in entities) {
            val beforeResult = beforeSave(entity, db)
            if (beforeResult != null) {
                return beforeResult
            }
        }

        db.saveMulti(entities, { e, exists, data -> onCheckEnd(e, exists, data) }, { obj, e -> checker(obj, e) })
        return returnData.setData("")
    }

    suspend fun delete(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setErrorMsg("NEED_LOGIN")
        }
        val userNS = getUserNS(context)
        val body = context.bodyAsString
        if (body.isNullOrEmpty()) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val jsonObj = try {
            JsonObject(body)
        } catch (e: Exception) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val entity = convertToEntity(jsonObj)
        val db = DB.table<T>(userNS, getTableName())

        val beforeResult = beforeDelete(entity, db)
        if (beforeResult != null) {
            return beforeResult
        }

        db.delete { obj -> checker(obj, entity) }
        return returnData.setData("")
    }

    suspend fun deleteMulti(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setErrorMsg("NEED_LOGIN")
        }
        val userNS = getUserNS(context)
        val body = context.bodyAsString
        if (body.isNullOrEmpty()) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val entities = try {
            convertToEntityList(body)
        } catch (e: Exception) {
            return returnData.setErrorMsg("PARAM_ERROR")
        }
        val db = DB.table<T>(userNS, getTableName())

        for (entity in entities) {
            val beforeResult = beforeDelete(entity, db)
            if (beforeResult != null) {
                return beforeResult
            }
            db.delete { obj -> checker(obj, entity) }
        }
        return returnData.setData("")
    }
}
