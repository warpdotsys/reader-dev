package com.htmake.reader.api.controller

import com.google.gson.JsonParseException
import com.htmake.reader.utils.toDataClass
import com.htmake.reader.utils.toMap
import io.legado.app.data.entities.Book
import io.legado.app.utils.ACache
import mu.KotlinLogging

private val bookInfoCacheLogger = KotlinLogging.logger {}

/** Only rebuildable metadata is a miss on malformed JSON; no deletion, lenient parse or fake book. */
internal fun readCachedBookInfo(cache: ACache, bookUrl: String): Book? {
    if (bookUrl.isEmpty()) return null
    val text = cache.getAsString(bookUrl) ?: return null
    return try {
        val fields: Map<String, Any>? = text.toMap()
        // ACache uses legacy hash-code file names; a colliding key or an empty
        // JSON object/array must not become another book or a fabricated default.
        if (fields == null || fields["bookUrl"] != bookUrl) {
            bookInfoCacheLogger.warn("书籍元数据缓存无效，将按书源重新获取")
            null
        } else fields.toDataClass<Book>()
    } catch (_: JsonParseException) {
        // Do not log URL, payload or exception text: source URLs/metadata can be private.
        bookInfoCacheLogger.warn("书籍元数据缓存无效，将按书源重新获取")
        null
    }
}
