// package com.htmake.reader.utils

// import com.google.gson.*
// import java.lang.reflect.Type

/**
 * Gson TypeAdapter for Long that safely handles null, empty string, and floating point values.
 */
// class LongTypeAdapter : JsonSerializer<Long>, JsonDeserializer<Long> {
pub struct LongTypeAdapter;

impl JsonSerializer<i64> for LongTypeAdapter {
    // override fun serialize(src: Long?, typeOfSrc: Type?, context: JsonSerializationContext?): JsonElement {
    //     return JsonPrimitive(src.toString())
    // }
    fn serialize(&self, src: Option<i64>, type_of_src: Option<Type>, context: Option<JsonSerializationContext>) -> JsonElement {
        return JsonPrimitive::new(src.unwrap_or_default().to_string());
    }
}

impl JsonDeserializer<i64> for LongTypeAdapter {
    // override fun deserialize(json: JsonElement, typeOfT: Type?, context: JsonDeserializationContext?): Long? {
    //     if (!json.isJsonPrimitive) {
    //         return null
    //     }
    //     val primitive = json.asJsonPrimitive
    //     return if (primitive.isNumber) primitive.asNumber.toLong() else null
    // }
    fn deserialize(&self, json: JsonElement, type_of_t: Option<Type>, context: Option<JsonDeserializationContext>) -> Option<i64> {
        if !json.is_json_primitive {
            return None;
        }
        let primitive = json.as_json_primitive;
        return if primitive.is_number { primitive.as_number.to_long() } else { None };
    }
}
