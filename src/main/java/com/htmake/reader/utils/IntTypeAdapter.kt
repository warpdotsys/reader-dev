package com.htmake.reader.utils

import com.google.gson.*
import java.lang.reflect.Type

/**
 * Gson TypeAdapter for Int that safely handles null, empty string, and floating point values.
 */
class IntTypeAdapter : JsonSerializer<Int>, JsonDeserializer<Int> {

    override fun serialize(src: Int?, typeOfSrc: Type?, context: JsonSerializationContext?): JsonElement {
        return JsonPrimitive(src ?: 0)
    }

    override fun deserialize(json: JsonElement?, typeOfT: Type?, context: JsonDeserializationContext?): Int {
        if (json == null || json.isJsonNull) {
            return 0
        }
        val str = json.asString
        if (str.isNullOrEmpty()) {
            return 0
        }
        return try {
            str.toDouble().toInt()
        } catch (e: NumberFormatException) {
            0
        }
    }
}
