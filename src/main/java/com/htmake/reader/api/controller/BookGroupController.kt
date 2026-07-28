package com.htmake.reader.api.controller

import io.legado.app.data.entities.BookGroup
import io.vertx.ext.web.RoutingContext
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import kotlin.coroutines.CoroutineContext

class BookGroupController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<BookGroup> {

    override fun getTableName(): String {
        return "bookGroup"
    }

    override fun getEntityClass(): Class<BookGroup> {
        return BookGroup::class.java
    }

    override fun checker(json: JsonObject, entity: BookGroup): Boolean {
        return json.getLong("groupId", 0L) == entity.groupId
    }

    override fun onList(list: JsonArray, userNameSpace: String): JsonArray {
        if (list.size() > 0) {
            return list
        }
        val defaultGroups = JsonArray()
            .add(JsonObject().put("groupId", -1L).put("groupName", "全部").put("order", -10).put("show", true))
            .add(JsonObject().put("groupId", -2L).put("groupName", "本地").put("order", -9).put("show", true))
            .add(JsonObject().put("groupId", -3L).put("groupName", "音频").put("order", -8).put("show", true))
            .add(JsonObject().put("groupId", -4L).put("groupName", "未分组").put("order", -7).put("show", true))
            .add(JsonObject().put("groupId", -5L).put("groupName", "更新错误").put("order", -6).put("show", true))
        saveUserStorage(userNameSpace, getTableName(), defaultGroups)
        return defaultGroups
    }

    override fun beforeSave(entity: BookGroup, db: DB<BookGroup>): ReturnData? {
        return if (entity.groupName.isEmpty()) ReturnData().setErrorMsg("分组名称不能为空") else null
    }

    override fun onCheckEnd(entity: BookGroup, exists: Boolean, allData: JsonArray) {
        if (exists) {
            return
        }
        var maxOrder = 0
        var idsSum = 0L
        for (item in allData) {
            val group = item as? JsonObject ?: continue
            maxOrder = maxOf(maxOrder, group.getInteger("order", 0))
            idsSum += maxOf(group.getLong("groupId", 0L), 0L)
        }
        var groupId = 1L
        while (groupId and idsSum != 0L) {
            groupId = groupId shl 1
        }
        entity.groupId = groupId
        entity.order = maxOrder + 1
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
        val groupIds: List<Long> = try {
            val arr = JsonArray(body)
            (0 until arr.size()).map { arr.getLong(it) }
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
                if (obj.getLong("groupId", 0L) == groupId) {
                    obj.put("order", order)
                    reordered.add(obj)
                    break
                }
            }
        }
        // Add any groups not in the order list
        for (i in 0 until allData.size()) {
            val obj = allData.getJsonObject(i) ?: continue
            val gid = obj.getLong("groupId", 0L)
            if (!groupIds.contains(gid)) {
                reordered.add(obj)
            }
        }

        db.cachedValue = reordered
        db.save()
        return returnData.setData("")
    }
}
