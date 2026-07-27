package com.htmake.reader.api.controller

import io.legado.app.data.entities.HttpTTS
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.gson
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class HttpTTSController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<HttpTTS> {

    override fun getTableName(): String {
        return "httpTTS"
    }

    override fun getEntityClass(): Class<HttpTTS> {
        return HttpTTS::class.java
    }

    override fun checker(json: JsonObject, entity: HttpTTS): Boolean {
        val jsonId = json.getLong("id", 0L)
        return jsonId == entity.id
    }

    override suspend fun checkUserAuth(context: RoutingContext): Boolean {
        return checkAuth(context)
    }

    override fun getUserNS(context: RoutingContext): String {
        return getUserNameSpace(context)
    }

    suspend fun getHttpTTSList(context: RoutingContext): ReturnData {
        return list(context)
    }

    suspend fun saveHttpTTS(context: RoutingContext): ReturnData {
        return save(context)
    }

    suspend fun saveHttpTTSList(context: RoutingContext): ReturnData {
        return saveMulti(context)
    }

    suspend fun deleteHttpTTS(context: RoutingContext): ReturnData {
        return delete(context)
    }
}
