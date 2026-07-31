package com.htmake.reader.utils

import com.google.gson.*
import java.lang.reflect.Type

/**
 * Gson TypeAdapter for Long that safely handles null, empty string, and floating point values.
 */
class LongTypeAdapter : JsonSerializer<Long>, JsonDeserializer<Long> {

    override fun serialize(src: Long?, typeOfSrc: Type?, context: JsonSerializationContext?): JsonElement {
        return JsonPrimitive(src.toString())
    }

    override fun deserialize(json: JsonElement, typeOfT: Type?, context: JsonDeserializationContext?): Long? {
        if (!json.isJsonPrimitive) {
            return null
        }
        val primitive = json.asJsonPrimitive
        return if (primitive.isNumber) primitive.asNumber.toLong() else null
    }
}
