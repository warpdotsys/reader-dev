package com.htmake.reader.api.controller

import io.legado.app.data.entities.BookGroup
import io.vertx.ext.web.RoutingContext
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import kotlin.coroutines.CoroutineContext
import mu.KotlinLogging

private val logger = KotlinLogging.logger {}

class BookGroupController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<BookGroup> {

    override fun getTableName(): String {
        return "bookGroup"
    }

    override fun getEntityClass(): Class<BookGroup> {
        return BookGroup::class.java
    }

    override fun checker(json: JsonObject, entity: BookGroup): Boolean {
        return json.getLong("groupId") == entity.groupId
    }

    override fun onList(list: JsonArray, userNameSpace: String): JsonArray {
        if (list.size() > 0) {
            return list
        }
        val defaultGroups = com.htmake.reader.utils.asJsonArray("""
            [{"groupId":-1,"groupName":"全部","order":-10,"show":true},{"groupId":-2,"groupName":"本地","order":-9,"show":true},{"groupId":-3,"groupName":"音频","order":-8,"show":true},{"groupId":-4,"groupName":"未分组","order":-7,"show":true},{"groupId":-5,"groupName":"更新错误","order":-6,"show":true}]
            """) ?: JsonArray()
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
        val bookGroupOrder = context.bodyAsJson?.getJsonArray("order") ?: return returnData.setErrorMsg("参数错误")
        var bookGroupList = com.htmake.reader.utils.asJsonArray(getUserStorage(userNameSpace, "bookGroup")) ?: JsonArray()
        val orderMap = mutableMapOf<Long, Int>()
        for (i in 0 until bookGroupOrder.size()) {
            val item = bookGroupOrder.getJsonObject(i) ?: continue
            val groupId = item.getLong("groupId") ?: continue
            val order = item.getInteger("order") ?: continue
            orderMap[groupId] = order
        }
        val groupList = bookGroupList.getList()
        for (i in 0 until bookGroupList.size()) {
            val group = bookGroupList.getJsonObject(i)?.mapTo(BookGroup::class.java) ?: continue
            orderMap[group.groupId]?.let { group.order = it }
            groupList[i] = JsonObject.mapFrom(group)
        }
        bookGroupList = JsonArray(groupList)
        saveUserStorage(userNameSpace, "bookGroup", bookGroupList)
        return returnData.setData("")
    }
}
