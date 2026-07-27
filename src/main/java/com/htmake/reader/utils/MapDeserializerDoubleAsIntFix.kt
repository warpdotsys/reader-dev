package com.htmake.reader.utils

import com.google.gson.*
import com.google.gson.internal.LinkedTreeMap
import java.lang.reflect.Type

/**
 * Gson deserializer that converts Double values to Int/Long when they are whole numbers.
 * This fixes the default Gson behavior of deserializing all numbers as Double in untyped Maps.
 */
class MapDeserializerDoubleAsIntFix : JsonDeserializer<Map<String, Any>> {

    @Suppress("UNCHECKED_CAST")
    @Throws(JsonParseException::class)
    override fun deserialize(
        json: JsonElement,
        typeOfT: Type,
        context: JsonDeserializationContext
    ): Map<String, Any> {
        return read(json) as Map<String, Any>
    }

    private fun read(element: JsonElement): Any? {
        return when {
            element.isJsonArray -> {
                val list = ArrayList<Any?>()
                val arr = element.asJsonArray
                for (item in arr) {
                    list.add(read(item))
                }
                list
            }
            element.isJsonObject -> {
                val map = LinkedTreeMap<String, Any?>()
                val obj = element.asJsonObject
                for (entry in obj.entrySet()) {
                    map[entry.key] = read(entry.value)
                }
                map
            }
            element.isJsonPrimitive -> {
                val prim = element.asJsonPrimitive
                when {
                    prim.isBoolean -> prim.asBoolean
                    prim.isString -> prim.asString
                    prim.isNumber -> {
                        val num = prim.asNumber
                        if (num.toString().contains(".")) {
                            val doubleVal = num.toDouble()
                            if (doubleVal == Math.floor(doubleVal) && !doubleVal.isInfinite()) {
                                val longVal = doubleVal.toLong()
                                if (longVal in Int.MIN_VALUE..Int.MAX_VALUE) {
                                    longVal.toInt()
                                } else {
                                    longVal
                                }
                            } else {
                                doubleVal
                            }
                        } else {
                            val longVal = num.toLong()
                            if (longVal in Int.MIN_VALUE..Int.MAX_VALUE) {
                                longVal.toInt()
                            } else {
                                longVal
                            }
                        }
                    }
                    else -> null
                }
            }
            else -> null
        }
    }
}
