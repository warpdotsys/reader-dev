use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（优先于 prelude 的 glob 导入）
use crate::stubs::{JsonArray, JsonObject};
use std::marker::PhantomData;
// package com.htmake.reader.db

/**
 * Abstract database class providing a base for JSON and SQL table implementations.
 */
// open class DB<T>(

/// 实体 → JSON（供 DB 持久化）
pub trait EntityToJson {
    fn to_json_value(&self) -> serde_json::Value;
}

impl<T: EntityToJson> EntityToJson for &T {
    fn to_json_value(&self) -> serde_json::Value {
        (**self).to_json_value()
    }
}

impl EntityToJson for crate::io_legado_app_data_entities_bookgroup::BookGroup {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::book_group_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_bookmark::Bookmark {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::bookmark_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_replacerule::ReplaceRule {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::replace_rule_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_httptts::HttpTTS {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::http_tts_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_txttocrule::TxtTocRule {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::txt_toc_rule_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_booksource::BookSource {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::book_source_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_rsssource::RssSource {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::rss_source_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_book::Book {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::book_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_bookchapter::BookChapter {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::book_chapter_to_json(self)
    }
}
impl EntityToJson for crate::io_legado_app_data_entities_searchbook::SearchBook {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::search_book_to_json(self)
    }
}
impl EntityToJson for crate::com_htmake_reader_entity_user::User {
    fn to_json_value(&self) -> serde_json::Value {
        crate::stubs::user_to_json(self)
    }
}

pub struct DB<T> {
    pub user_name_space: String,
    pub name: String,
    pub cached_value: JsonArray,
    _phantom: PhantomData<T>,
}

impl<T: EntityToJson + serde::de::DeserializeOwned> DB<T> {
    fn table_file_path(&self) -> String {
        crate::com_htmake_reader_utils_vertext::get_storage_file(&[String::from("data"), self.user_name_space.clone(), self.name.clone()], "json").path()
    }

    fn load_cached(&self) -> JsonArray {
        let file = crate::stubs::File::new(&self.table_file_path());
        if file.exists() {
            let text = file.read_text();
            if let Ok(v) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                return JsonArray(v.into_iter().map(|x| x.to_string()).collect());
            }
        }
        JsonArray::new()
    }

    fn flush(&self, data: &JsonArray) {
        let path = self.table_file_path();
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let dir = crate::stubs::File::new(&parent.to_string_lossy().to_string());
            if !dir.exists() {
                let _ = dir.mkdirs();
            }
        }
        let file = crate::stubs::File::new(&path);
        let _ = file.write_text(&data.to_string());
    }

    pub fn new(user_name_space: String, name: String) -> DB<T> {
        let mut db = DB::<T> {
            user_name_space,
            name,
            cached_value: JsonArray::new(),
            _phantom: PhantomData::default(),
        };
        db.cached_value = db.load_cached();
        db
    }

    // open fun readAll(): JsonArray {
    pub fn read_all(&mut self) -> JsonArray {
        self.cached_value = self.load_cached();
        return self.cached_value.clone();
    }

    // open fun <P> findBy(field: String, value: P, clazz: Class<T>): T? {
    pub fn find_by(&mut self, field: &str, value: &str, clazz: Class<T>) -> Option<T> {
        let data = self.read_all();
        let text = value.to_string();
        for i in 0..data.size() {
            if let Some(obj) = data.get_json_object(i) {
                if obj.get_string(field) == text {
                    return crate::stubs::JsonObject::map_to_deser(&obj);
                }
            }
        }
        return None;
    }

    // open fun save(
    //     entity: T,
    //     onCheckEnd: ((T, Boolean, JsonArray) -> Unit)? = None,
    //     checker: (JsonObject, T) -> Boolean
    // ) {
    pub fn save(
        &mut self,
        entity: T,
        on_check_end: Option<&dyn Fn(T, bool, JsonArray) -> ()>,
        checker: &dyn Fn(JsonObject, &T) -> bool,
    ) {
        let mut data = self.load_cached();
        let json = serde_json::to_string(&entity.to_json_value()).unwrap_or_else(|_| String::from("{}"));
        let exists = data
            .0
            .iter()
            .position(|s| {
                serde_json::from_str::<serde_json::Value>(s)
                    .and_then(|v| serde_json::from_str::<serde_json::Value>(&json))
                    .map(|_| checker(JsonObject(s.clone()), &entity))
                    .unwrap_or(false)
            });
        let mut existed = false;
        match exists {
            Some(idx) => {
                existed = true;
                data.0[idx] = json.clone();
            }
            None => {
                data.0.push(json.clone());
            }
        }
        self.cached_value = data.clone();
        self.flush(&data);
        if let Some(cb) = on_check_end {
            cb(entity, existed, data);
        }
    }

    // open fun saveMulti(
    //     entities: Array<T>,
    //     onCheckEnd: ((T, Boolean, JsonArray) -> Unit)? = None,
    //     checker: (JsonObject, T) -> Boolean
    // ) {
    pub fn save_multi(
        &mut self,
        entities: Vec<T>,
        on_check_end: Option<&dyn Fn(T, bool, JsonArray) -> ()>,
        checker: &dyn Fn(JsonObject, &T) -> bool,
    ) {
        let mut data = self.load_cached();
        for entity in entities {
            let json = serde_json::to_string(&entity.to_json_value()).unwrap_or_else(|_| String::from("{}"));
            let exists = data
                .0
                .iter()
                .position(|s| {
                    serde_json::from_str::<serde_json::Value>(s)
                        .and_then(|v| serde_json::from_str::<serde_json::Value>(&json))
                        .map(|_| checker(JsonObject(s.clone()), &entity))
                        .unwrap_or(false)
                });
            let mut existed = false;
            match exists {
                Some(idx) => {
                    existed = true;
                    data.0[idx] = json.clone();
                }
                None => {
                    data.0.push(json.clone());
                }
            }
            if let Some(cb) = on_check_end {
                cb(entity, existed, data.clone());
            }
        }
        self.cached_value = data.clone();
        self.flush(&data);
    }

    // open fun delete(predicate: (JsonObject) -> Boolean) {
    pub fn delete(&mut self, predicate: &dyn Fn(JsonObject) -> bool) {
        let mut data = self.load_cached();
        data.0.retain(|s| {
            let obj = JsonObject(s.clone());
            !predicate(obj)
        });
        self.cached_value = data.clone();
        self.flush(&data);
    }

    // open fun save() {
    pub fn save_only(&mut self) {
        self.flush(&self.cached_value);
    }

    // companion object {
    //     fun <T> table(userNameSpace: String, name: String, driver: String = "JSON"): DB<T> {
    //         return if (driver == "SQL") {
    //             SQLTable(userNameSpace, name)
    //         } else {
    //             JSONTable(userNameSpace, name)
    //         }
    //     }
    // }
    pub fn table<T2>(user_name_space: String, name: String, driver: String) -> DB<T2>
    where
        T2: EntityToJson + serde::de::DeserializeOwned,
        T2: EntityToJson,
    {
        return if driver == "SQL" {
            DB::<T2>::new(user_name_space, name)
        } else {
            DB::<T2>::new(user_name_space, name)
        };
    }
}
