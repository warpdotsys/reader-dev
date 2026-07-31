package com.htmake.reader.api.controller

import io.legado.app.data.entities.HttpTTS
import io.legado.app.utils.GSON
import io.legado.app.utils.fromJsonObject
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.asJsonArray
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
        return entity.name == json.getString("name")
    }

    override fun beforeSave(entity: HttpTTS, db: DB<HttpTTS>): ReturnData? {
        val returnData = ReturnData()
        if (entity.name.isEmpty()) return returnData.setErrorMsg("名称不能为空")
        if (entity.url.isEmpty()) return returnData.setErrorMsg("链接不能为空")
        return null
    }

    override fun convertToEntity(json: JsonObject): HttpTTS {
        return GSON.fromJsonObject<HttpTTS>(json.toString()).getOrNull()!!
    }

    override fun convertToEntityList(json: String): Array<HttpTTS> {
        return asJsonArray(json)!!.map { GSON.fromJsonObject<HttpTTS>(it.toString()).getOrNull()!! }.toTypedArray()
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
