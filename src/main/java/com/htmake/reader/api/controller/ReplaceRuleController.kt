package com.htmake.reader.api.controller

import io.legado.app.data.entities.ReplaceRule
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.gson
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class ReplaceRuleController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<ReplaceRule> {

    override fun getTableName(): String {
        return "replaceRule"
    }

    override fun getEntityClass(): Class<ReplaceRule> {
        return ReplaceRule::class.java
    }

    override fun checker(json: JsonObject, entity: ReplaceRule): Boolean {
        val jsonId = json.getLong("id", 0L)
        return jsonId == entity.id
    }

    override suspend fun checkUserAuth(context: RoutingContext): Boolean {
        return checkAuth(context)
    }

    override fun getUserNS(context: RoutingContext): String {
        return getUserNameSpace(context)
    }

    suspend fun getReplaceRules(context: RoutingContext): ReturnData {
        return list(context)
    }

    suspend fun saveReplaceRule(context: RoutingContext): ReturnData {
        return save(context)
    }

    suspend fun saveReplaceRules(context: RoutingContext): ReturnData {
        return saveMulti(context)
    }

    suspend fun deleteReplaceRule(context: RoutingContext): ReturnData {
        return delete(context)
    }

    suspend fun deleteReplaceRules(context: RoutingContext): ReturnData {
        return deleteMulti(context)
    }
}
