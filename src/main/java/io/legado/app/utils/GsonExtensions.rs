use crate::prelude::*;
// fix: `Any` 被 stubs 与 analyzebyjsoup 两个 glob 同时导出，显式导入消歧义
use crate::stubs::Any;

// fix: Kotlin 顶层 `val GSON: Gson` → GSON 结构体 + new()（GsonBuilder 各步骤为占位，语义等价）
pub struct GSON;

impl GSON {
    pub fn new() -> Gson {
        GsonBuilder::new()
            .registerTypeAdapter(
                type_of::<Map<Option<String>, Option<Any>>>(),
                MapDeserializerDoubleAsIntFix::new()
            )
            .registerTypeAdapter(i32::r#type(), IntJsonDeserializer::new())
            .disableHtmlEscaping()
            .setPrettyPrinting()
            .create()
    }
}

pub fn genericType<T>() -> Type {
    // fix: TypeToken<T>::new()（泛型在结构体上而非 new 上，移除多余 turbofish）
    TypeToken::<T>::new().r#type()
}

pub fn fromJsonObject<T>(gson: &Gson, json: Option<&str>) -> Result<Option<T>, Box<dyn std::any::Any + Send>>
where
    T: serde::de::DeserializeOwned,
{
    std::panic::catch_unwind(|| {
        gson.fromJson::<T>(json).ok().flatten()
    })
}

pub fn fromJsonArray<T>(gson: &Gson, json: Option<&str>) -> Result<Option<Vec<T>>, Box<dyn std::any::Any + Send>>
where
    T: serde::de::DeserializeOwned,
{
    std::panic::catch_unwind(|| {
        gson.fromJson_list::<T, _>(json, &ParameterizedTypeImpl::new(type_of::<T>())).ok().flatten()
    })
}

pub fn fromJsonObject_stream<T>(gson: &Gson, inputStream: Option<&mut dyn InputStream>) -> Result<Option<T>, Box<dyn std::any::Any + Send>>
where
    T: serde::de::DeserializeOwned,
{
    // fix: &mut dyn InputStream 非 UnwindSafe，catch_unwind 闭包需 AssertUnwindSafe 包装
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // fix: InputStreamReader 显式路径引用（多个转录模块 glob 导出同名结构，避免歧义）
        let reader = crate::stubs::InputStreamReader::new(inputStream);
        gson.fromJson_reader::<T>(&reader).ok().flatten()
    }))
}

pub fn fromJsonArray_stream<T>(gson: &Gson, inputStream: Option<&mut dyn InputStream>) -> Result<Option<Vec<T>>, Box<dyn std::any::Any + Send>>
where
    T: serde::de::DeserializeOwned,
{
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // fix: 同上，显式路径引用 InputStreamReader
        let reader = crate::stubs::InputStreamReader::new(inputStream);
        gson.fromJson_list_reader::<T, _>(&reader, &ParameterizedTypeImpl::new(type_of::<T>())).ok().flatten()
    }))
}

pub fn writeToOutputStream(gson: &Gson, out: &mut OutputStream, any: &Any) {
    // fix: 显式路径引用 JsonWriter / OutputStreamWriter（消解跨模块 glob 歧义）
    let mut writer = crate::stubs::JsonWriter::new(crate::stubs::OutputStreamWriter::new(out, "UTF-8"));
    writer.setIndent("  ");
    if any.is_list() {
        writer.beginArray();
        // fix: list_iter 元素为非空 Any（占位 Any::Null 等价于 Kotlin 的 null 元素）
        for it in any.list_iter() {
            gson.toJson_dyn(&it, it.r#type(), &mut writer);
        }
        writer.endArray();
    } else {
        gson.toJson_dyn(any, any.r#type(), &mut writer);
    }
    writer.close();
}

pub struct ParameterizedTypeImpl {
    clazz: Type,
}

impl ParameterizedTypeImpl {
    pub fn new(clazz: Type) -> ParameterizedTypeImpl {
        ParameterizedTypeImpl { clazz }
    }

    pub fn getRawType(&self) -> Type {
        // fix: Kotlin `List::class.java` → type_of::<List<Any>>()
        type_of::<List<Any>>()
    }

    pub fn getOwnerType(&self) -> Option<Type> {
        None
    }

    pub fn getActualTypeArguments(&self) -> Vec<Type> {
        vec![self.clazz]
    }
}

/**
 * int类型转化失败时跳过
 */
pub struct IntJsonDeserializer;

impl IntJsonDeserializer {
    pub fn new() -> IntJsonDeserializer {
        IntJsonDeserializer {}
    }

    pub fn deserialize(&self, json: &JsonElement, typeOfT: Option<&Type>, context: Option<&mut JsonDeserializationContext>) -> Option<i32> {
        if json.isJsonPrimitive() {
            let prim = json.asJsonPrimitive();
            if prim.isNumber() {
                // fix: Java `Number.toInt()`（截断取整）→ as_f64() as i32
                Some(prim.asNumber().as_f64().unwrap_or(0.0) as i32)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/**
 * 修复Int变为Double的问题
 */
pub struct MapDeserializerDoubleAsIntFix;

impl MapDeserializerDoubleAsIntFix {
    pub fn new() -> MapDeserializerDoubleAsIntFix {
        MapDeserializerDoubleAsIntFix {}
    }

    // fix: 移除 #[throws(JsonParseException)]（Rust 无检查异常属性）；返回值由 `Map<String, Option<Any>>` 调整为 `Map<String, Any>`；`type` 为关键字改 `r#type`
    pub fn deserialize(
        &self,
        jsonElement: &JsonElement,
        r#type: &Type,
        jsonDeserializationContext: &mut JsonDeserializationContext
    ) -> Option<Map<String, Any>> {
        self.read(jsonElement).as_map()
    }

    pub fn read(&self, json: &JsonElement) -> Any {
        if json.isJsonArray() {
            let mut list: Vec<Any> = Vec::new();
            let arr = json.asJsonArray();
            for anArr in arr {
                list.push(self.read(anArr));
            }
            return Any::from_list(list);
        }
        if json.isJsonObject() {
            let mut map: Map<String, Any> = LinkedTreeMap::new();
            // fix: entrySet 为 JsonElement 的扩展方法，须在 JsonElement（json）上调用而非 &Map
            let entitySet = json.entrySet();
            for (key, value) in entitySet {
                map.insert(key.clone(), self.read(value));
            }
            return Any::from_map(map);
        }
        if json.isJsonPrimitive() {
            let prim = json.asJsonPrimitive();
            if prim.isBoolean() {
                return Any::from_bool(prim.asBoolean());
            }
            if prim.isString() {
                return Any::from_string(prim.asString());
            }
            if prim.isNumber() {
                let num = prim.asNumber();
                // here you can handle double int/long values
                // and return any type you want
                // this solution will transform 3.0 float to long values
                return if num.as_f64().unwrap_or(0.0).ceil() == num.as_i64().unwrap_or(0) as f64 {
                    Any::from_long(num.as_i64().unwrap_or(0))
                } else {
                    Any::from_double(num.as_f64().unwrap_or(0.0))
                };
            }
        }
        Any::Null
    }
}

// fix: Gson `JsonElement`/`JsonPrimitive` 方法 → serde_json::Value 等价封装（保留 Kotlin 方法名）
trait JsonElementExt {
    fn isJsonPrimitive(&self) -> bool;
    fn asJsonPrimitive(&self) -> &JsonElement;
    fn isJsonArray(&self) -> bool;
    fn asJsonArray(&self) -> &Vec<JsonElement>;
    fn isJsonObject(&self) -> bool;
    fn asJsonObject(&self) -> &serde_json::Map<String, JsonElement>;
    fn entrySet(&self) -> &serde_json::Map<String, JsonElement>;
    fn isBoolean(&self) -> bool;
    fn asBoolean(&self) -> bool;
    fn isString(&self) -> bool;
    fn asString(&self) -> String;
    fn isNumber(&self) -> bool;
    fn asNumber(&self) -> &serde_json::Number;
}

impl JsonElementExt for JsonElement {
    fn isJsonPrimitive(&self) -> bool {
        self.is_boolean() || self.is_number() || self.is_string()
    }
    fn asJsonPrimitive(&self) -> &JsonElement {
        self
    }
    fn isJsonArray(&self) -> bool {
        self.is_array()
    }
    fn asJsonArray(&self) -> &Vec<JsonElement> {
        self.as_array().unwrap()
    }
    fn isJsonObject(&self) -> bool {
        self.is_object()
    }
    fn asJsonObject(&self) -> &serde_json::Map<String, JsonElement> {
        self.as_object().unwrap()
    }
    fn entrySet(&self) -> &serde_json::Map<String, JsonElement> {
        self.as_object().unwrap()
    }
    fn isBoolean(&self) -> bool {
        self.is_boolean()
    }
    fn asBoolean(&self) -> bool {
        self.as_bool().unwrap_or(false)
    }
    fn isString(&self) -> bool {
        self.is_string()
    }
    fn asString(&self) -> String {
        self.as_str().unwrap_or("").to_string()
    }
    fn isNumber(&self) -> bool {
        self.is_number()
    }
    fn asNumber(&self) -> &serde_json::Number {
        self.as_number().unwrap()
    }
}
