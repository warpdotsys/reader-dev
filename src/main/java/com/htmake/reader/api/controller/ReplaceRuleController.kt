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
        return entity.name == json.getString("name")
    }

    override fun beforeSave(entity: ReplaceRule, db: DB<ReplaceRule>): ReturnData? {
        if (entity.name.isEmpty()) {
            return ReturnData().setErrorMsg("名称不能为空")
        }
        if (entity.pattern.isEmpty()) {
            return ReturnData().setErrorMsg("规则不能为空")
        }
        return null
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
