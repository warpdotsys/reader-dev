// package com.htmake.reader.utils

// import com.google.gson.*
// import java.lang.reflect.Type

/**
 * Gson TypeAdapter for Int that safely handles null, empty string, and floating point values.
 */
// class IntTypeAdapter : JsonSerializer<Int>, JsonDeserializer<Int> {
pub struct IntTypeAdapter;

// JsonSerializer<Int>
pub trait JsonSerializer<T> {
    // JsonElement serialize(T? src, Type? typeOfSrc, JsonSerializationContext? context)
    fn serialize(&self, src: Option<T>, type_of_src: Option<Type>, context: Option<JsonSerializationContext>) -> JsonElement;
}

// JsonDeserializer<Int>
pub trait JsonDeserializer<T> {
    // T? deserialize(JsonElement json, Type? typeOfT, JsonDeserializationContext? context)
    fn deserialize(&self, json: JsonElement, type_of_t: Option<Type>, context: Option<JsonDeserializationContext>) -> Option<T>;
}

impl JsonSerializer<i32> for IntTypeAdapter {
    // override fun serialize(src: Int?, typeOfSrc: Type?, context: JsonSerializationContext?): JsonElement {
    //     return JsonPrimitive(src.toString())
    // }
    fn serialize(&self, src: Option<i32>, type_of_src: Option<Type>, context: Option<JsonSerializationContext>) -> JsonElement {
        return JsonPrimitive::new(src.unwrap_or_default().to_string());
    }
}

impl JsonDeserializer<i32> for IntTypeAdapter {
    // override fun deserialize(json: JsonElement, typeOfT: Type?, context: JsonDeserializationContext?): Int? {
    //     if (!json.isJsonPrimitive) {
    //         return null
    //     }
    //     val primitive = json.asJsonPrimitive
    //     return if (primitive.isNumber) primitive.asNumber.toInt() else null
    // }
    fn deserialize(&self, json: JsonElement, type_of_t: Option<Type>, context: Option<JsonDeserializationContext>) -> Option<i32> {
        if !json.is_json_primitive {
            return None;
        }
        let primitive = json.as_json_primitive;
        return if primitive.is_number { primitive.as_number.to_int() } else { None };
    }
}
