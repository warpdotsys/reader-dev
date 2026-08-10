pub struct GSON;

impl GSON {
    pub fn new() -> Gson {
        GsonBuilder::new()
            .registerTypeAdapter(
                type_of::<Map<Option<String>, Option<Any>>>(),
                MapDeserializerDoubleAsIntFix::new()
            )
            .registerTypeAdapter(i32::type(), IntJsonDeserializer::new())
            .disableHtmlEscaping()
            .setPrettyPrinting()
            .create()
    }
}

pub fn genericType<T>() -> Type {
    TypeToken::new::<T>().type()
}

pub fn fromJsonObject<T>(gson: &Gson, json: Option<&str>) -> Result<Option<T>> {
    std::panic::catch_unwind(|| {
        gson.fromJson::<T>(json).ok().flatten()
    })
}

pub fn fromJsonArray<T>(gson: &Gson, json: Option<&str>) -> Result<Option<Vec<T>>> {
    std::panic::catch_unwind(|| {
        gson.fromJson_list::<T>(json, &ParameterizedTypeImpl::new(T::type())).ok().flatten()
    })
}

pub fn fromJsonObject_stream<T>(gson: &Gson, inputStream: Option<&mut dyn InputStream>) -> Result<Option<T>> {
    std::panic::catch_unwind(|| {
        let reader = InputStreamReader::new(inputStream);
        gson.fromJson_reader::<T>(&reader).ok().flatten()
    })
}

pub fn fromJsonArray_stream<T>(gson: &Gson, inputStream: Option<&mut dyn InputStream>) -> Result<Option<Vec<T>>> {
    std::panic::catch_unwind(|| {
        let reader = InputStreamReader::new(inputStream);
        gson.fromJson_list_reader::<T>(&reader, &ParameterizedTypeImpl::new(T::type())).ok().flatten()
    })
}

pub fn writeToOutputStream(gson: &Gson, out: &mut dyn OutputStream, any: &dyn Any) {
    let mut writer = JsonWriter::new(OutputStreamWriter::new(out, "UTF-8"));
    writer.setIndent("  ");
    if any.is_list() {
        writer.beginArray();
        for it in any.list_iter() {
            if let Some(it) = it {
                gson.toJson_dyn(it, it.type(), &mut writer);
            }
        }
        writer.endArray();
    } else {
        gson.toJson_dyn(any, any.type(), &mut writer);
    }
    writer.close();
}

pub struct ParameterizedTypeImpl {
    clazz: Class,
}

impl ParameterizedTypeImpl {
    pub fn new(clazz: Class) -> ParameterizedTypeImpl {
        ParameterizedTypeImpl { clazz }
    }

    pub fn getRawType(&self) -> Type {
        List::type()
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
                Some(prim.asNumber().toInt())
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

    #[throws(JsonParseException)]
    pub fn deserialize(
        &self,
        jsonElement: &JsonElement,
        type: &Type,
        jsonDeserializationContext: &mut JsonDeserializationContext
    ) -> Option<Map<String, Option<Any>>> {
        self.read(jsonElement).as_map()
    }

    pub fn read(&self, json: &JsonElement) -> Option<Any> {
        if json.isJsonArray() {
            let mut list: Vec<Option<Any>> = Vec::new();
            let arr = json.asJsonArray();
            for anArr in arr {
                list.push(self.read(anArr));
            }
            return Some(Any::from_list(list));
        }
        if json.isJsonObject() {
            let mut map: Map<String, Option<Any>> = LinkedTreeMap::new();
            let obj = json.asJsonObject();
            let entitySet = obj.entrySet();
            for (key, value) in entitySet {
                map.put(key, self.read(value));
            }
            return Some(Any::from_map(map));
        }
        if json.isJsonPrimitive() {
            let prim = json.asJsonPrimitive();
            if prim.isBoolean() {
                return Some(Any::from_bool(prim.asBoolean()));
            }
            if prim.isString() {
                return Some(Any::from_string(prim.asString()));
            }
            if prim.isNumber() {
                let num: Number = prim.asNumber();
                // here you can handle double int/long values
                // and return any type you want
                // this solution will transform 3.0 float to long values
                return if ceil(num.toDouble()) == num.toLong() as f64 {
                    Any::from_long(num.toLong())
                } else {
                    Any::from_double(num.toDouble())
                };
            }
        }
        None
    }
}
