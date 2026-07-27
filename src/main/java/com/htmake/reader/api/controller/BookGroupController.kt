package com.htmake.reader.api.controller

import io.legado.app.data.entities.BookGroup
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.gson
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class BookGroupController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<BookGroup> {

    override fun getTableName(): String {
        return "bookGroup"
    }

    override fun getEntityClass(): Class<BookGroup> {
        return BookGroup::class.java
    }

    override fun checker(json: JsonObject, entity: BookGroup): Boolean {
        return json.getInteger("groupId", 0) == entity.groupId
    }

    override suspend fun checkUserAuth(context: RoutingContext): Boolean {
        return checkAuth(context)
    }

    override fun getUserNS(context: RoutingContext): String {
        return getUserNameSpace(context)
    }

    suspend fun getBookGroups(context: RoutingContext): ReturnData {
        return list(context)
    }

    suspend fun saveBookGroup(context: RoutingContext): ReturnData {
        return save(context)
    }

    suspend fun deleteBookGroup(context: RoutingContext): ReturnData {
        return delete(context)
    }

    suspend fun saveBookGroupOrder(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val userNameSpace = getUserNameSpace(context)
        val body = context.bodyAsString
        if (body.isNullOrEmpty()) {
            return returnData.setErrorMsg("参数错误")
        }
        val groupIds: List<Int> = try {
            val arr = JsonArray(body)
            (0 until arr.size()).map { arr.getInteger(it) }
        } catch (e: Exception) {
            return returnData.setErrorMsg("参数错误")
        }

        val db = DB.table<BookGroup>(userNameSpace, getTableName())
        val allData = db.readAll() ?: JsonArray()

        // Reorder groups based on the provided order
        val reordered = JsonArray()
        for ((order, groupId) in groupIds.withIndex()) {
            for (i in 0 until allData.size()) {
                val obj = allData.getJsonObject(i) ?: continue
                if (obj.getInteger("groupId", 0) == groupId) {
                    obj.put("order", order)
                    reordered.add(obj)
                    break
                }
            }
        }
        // Add any groups not in the order list
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i) ?: continue
            val gid = obj.getInteger("groupId", 0)
            if (!groupIds.contains(gid)) {
                reordered.add(obj)
            }
        }

        db.cachedValue = reordered
        db.save()
        return returnData.setData("")
    }
}
