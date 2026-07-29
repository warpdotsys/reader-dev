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
        return json.mapTo(getEntityClass())
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
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val userNS = getUserNS(context)
        val db = DB.table<T>(userNS, getTableName())
        val allData = db.readAll()
        val result = onList(allData, userNS)
        return returnData.setData(result.list)
    }

    suspend fun save(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val entity = convertToEntity(context.bodyAsJson)
        val userNS = getUserNS(context)
        val db = DB.table<T>(userNS, getTableName())

        val beforeResult = beforeSave(entity, db)
        if (beforeResult != null) {
            return beforeResult
        }

        db.save(entity, { e, exists, data -> onCheckEnd(e, exists, data) }, { obj, e -> checker(obj, e) })
        return returnData.setData("")
    }

    suspend fun saveMulti(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkUserAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val entities = convertToEntityList(context.bodyAsString)
        if (entities.isEmpty()) return returnData.setErrorMsg("参数错误")
        val userNS = getUserNS(context)
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
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val entity = convertToEntity(context.bodyAsJson)
        val userNS = getUserNS(context)
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
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val entities = convertToEntityList(context.bodyAsString)
        if (entities.isEmpty()) return returnData.setErrorMsg("参数错误")
        val userNS = getUserNS(context)
        val db = DB.table<T>(userNS, getTableName())

        for (entity in entities) {
            val beforeResult = beforeDelete(entity, db)
            if (beforeResult != null) {
                return beforeResult
            }
        }
        db.delete { obj -> entities.any { entity -> checker(obj, entity) } }
        return returnData.setData("")
    }
}
