package com.htmake.reader.utils

import com.google.gson.*
import java.lang.reflect.Type

/**
 * Gson TypeAdapter for Int that safely handles null, empty string, and floating point values.
 */
class IntTypeAdapter : JsonSerializer<Int>, JsonDeserializer<Int> {

    override fun serialize(src: Int?, typeOfSrc: Type?, context: JsonSerializationContext?): JsonElement {
        return JsonPrimitive(src.toString())
    }

    override fun deserialize(json: JsonElement, typeOfT: Type?, context: JsonDeserializationContext?): Int? {
        if (!json.isJsonPrimitive) {
            return null
        }
        val primitive = json.asJsonPrimitive
        return if (primitive.isNumber) primitive.asNumber.toInt() else null
    }
}
