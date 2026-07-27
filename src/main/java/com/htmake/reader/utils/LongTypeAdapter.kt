package com.htmake.reader.utils

import com.google.gson.*
import java.lang.reflect.Type

/**
 * Gson TypeAdapter for Long that safely handles null, empty string, and floating point values.
 */
class LongTypeAdapter : JsonSerializer<Long>, JsonDeserializer<Long> {

    override fun serialize(src: Long?, typeOfSrc: Type?, context: JsonSerializationContext?): JsonElement {
        return JsonPrimitive(src ?: 0L)
    }

    override fun deserialize(json: JsonElement?, typeOfT: Type?, context: JsonDeserializationContext?): Long {
        if (json == null || json.isJsonNull) {
            return 0L
        }
        val str = json.asString
        if (str.isNullOrEmpty()) {
            return 0L
        }
        return try {
            str.toDouble().toLong()
        } catch (e: NumberFormatException) {
            0L
        }
    }
}
