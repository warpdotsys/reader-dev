package com.htmake.reader.api.controller

import io.legado.app.data.entities.Bookmark
import io.vertx.ext.web.RoutingContext
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.db.DB
import com.htmake.reader.utils.gson
import io.vertx.core.json.JsonArray
import io.vertx.core.json.JsonObject
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class BookmarkController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<Bookmark> {

    override fun getTableName(): String {
        return "bookmark"
    }

    override fun getEntityClass(): Class<Bookmark> {
        return Bookmark::class.java
    }

    override fun checker(json: JsonObject, entity: Bookmark): Boolean {
        val jsonBookName = json.getString("bookName", "")
        val jsonBookAuthor = json.getString("bookAuthor", "")
        return jsonBookName == entity.bookName && jsonBookAuthor == entity.bookAuthor
    }

    override suspend fun checkUserAuth(context: RoutingContext): Boolean {
        return checkAuth(context)
    }

    override fun getUserNS(context: RoutingContext): String {
        return getUserNameSpace(context)
    }

    suspend fun getBookmarks(context: RoutingContext): ReturnData {
        return list(context)
    }

    suspend fun saveBookmark(context: RoutingContext): ReturnData {
        return save(context)
    }

    suspend fun saveBookmarks(context: RoutingContext): ReturnData {
        return saveMulti(context)
    }

    suspend fun deleteBookmark(context: RoutingContext): ReturnData {
        return delete(context)
    }

    suspend fun deleteBookmarks(context: RoutingContext): ReturnData {
        return deleteMulti(context)
    }
}
