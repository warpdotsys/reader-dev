use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（优先于 prelude 的 glob 导入）
use crate::com_htmake_reader_db_db::DB;
use crate::com_htmake_reader_utils_vertext::{as_json_array, get_storage, save_storage};
use crate::stubs::{Any, JsonArray, JsonObject};
// package com.htmake.reader.db

// import io.vertx.core.json.JsonArray
// import io.vertx.core.json.JsonObject
// import com.htmake.reader.utils.getStorage
// import com.htmake.reader.utils.saveStorage
// import com.htmake.reader.utils.asJsonArray

/**
 * JSON file-based table implementation using getStorage/saveStorage pattern.
 */
// class JSONTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {
pub struct JSONTable<T> {
    pub db: DB<T>,
    // fix: DB<T> 的 T 未在 DB 字段中真实使用（DB.rs 自身 E0392 级联）→ PhantomData 使 T 成为真实使用
    pub _marker: std::marker::PhantomData<T>,
}

impl<T: Clone> JSONTable<T> {
    pub fn new(user_name_space: String, name: String) -> JSONTable<T> {
        JSONTable {
            db: DB::<T>::new(user_name_space, name),
            _marker: std::marker::PhantomData,
        }
    }

    // override fun readAll(): JsonArray {
    pub fn read_all(&mut self) -> JsonArray {
        let data_list = as_json_array(get_storage(&vec!["data".to_string(), self.db.user_name_space.clone(), self.db.name.clone()], ".json").map(Any::from_string)).unwrap_or_else(|| JsonArray::new());
        self.db.cached_value = data_list.clone();
        return data_list;
    }

    // override fun <P> findBy(field: String, value: P, clazz: Class<T>): T? {
    pub fn find_by<P: PartialEq<String>>(&mut self, field: &str, value: P, clazz: Class<T>) -> Option<T> {
        let all_data = self.read_all();
        for i in 0..all_data.size() {
            let obj = all_data.get_json_object(i).unwrap_or_default();
            if value == obj.get_string(field) {
                return obj.map_to_with_class(clazz);
            }
        }
        return None;
    }

    // override fun save(
    //     entity: T,
    //     onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
    //     checker: (JsonObject, T) -> Boolean
    // ) {
    pub fn save(
        &mut self,
        entity: T,
        on_check_end: Option<&dyn Fn(T, bool, JsonArray) -> ()>,
        checker: &dyn Fn(JsonObject, T) -> bool,
    ) {
        let mut all_data = self.read_all();
        let mut existing_index = -1;

        for i in 0..all_data.size() {
            let obj = all_data.get_json_object(i).unwrap_or_default();
            if checker(obj.clone(), entity.clone()) {
                existing_index = i as i32;
                break;
            }
        }

        if let Some(cb) = on_check_end {
            cb(entity.clone(), existing_index >= 0, all_data.clone());
        }

        if existing_index >= 0 {
            all_data.0[existing_index as usize] = JsonObject::map_from(entity.clone()).to_string();
            all_data = JsonArray(all_data.0.clone());
        } else {
            all_data.add(JsonObject::map_from(entity.clone()));
        }

        self.db.cached_value = all_data.clone();
        self.save_only();
    }

    // override fun saveMulti(
    //     entities: Array<T>,
    //     onCheckEnd: ((T, Boolean, JsonArray) -> Unit)?,
    //     checker: (JsonObject, T) -> Boolean
    // ) {
    pub fn save_multi(
        &mut self,
        entities: Vec<T>,
        on_check_end: Option<&dyn Fn(T, bool, JsonArray) -> ()>,
        checker: &dyn Fn(JsonObject, T) -> bool,
    ) {
        let mut all_data = self.read_all();

        for entity in entities {
            let mut existing_index = -1;
            for i in 0..all_data.size() {
                let obj = all_data.get_json_object(i).unwrap_or_default();
                if checker(obj.clone(), entity.clone()) {
                    existing_index = i as i32;
                    break;
                }
            }

            if let Some(cb) = on_check_end {
                cb(entity.clone(), existing_index >= 0, all_data.clone());
            }

            if existing_index >= 0 {
                all_data.0[existing_index as usize] = JsonObject::map_from(entity.clone()).to_string();
            } else {
                all_data.add(JsonObject::map_from(entity.clone()));
            }
        }

        self.db.cached_value = all_data.clone();
        self.save_only();
    }

    // override fun delete(predicate: (JsonObject) -> Boolean) {
    pub fn delete(&mut self, predicate: &dyn Fn(JsonObject) -> bool) {
        let mut all_data = self.read_all();
        let mut remove_indexes: Vec<i32> = Vec::new();
        for i in 0..all_data.size() {
            let obj = all_data.get_json_object(i).unwrap_or_default();
            if predicate(obj.clone()) {
                remove_indexes.push(i as i32);
            }
        }
        if !remove_indexes.is_empty() {
            let mut new_data = JsonArray::new();
            for i in 0..all_data.size() {
                if !remove_indexes.contains(&(i as i32)) {
                    new_data.add(all_data.get_json_object(i).unwrap_or_default());
                }
            }
            all_data = new_data;
        }
        self.db.cached_value = all_data.clone();
        self.save_only();
    }

    // override fun save() {
    pub fn save_only(&mut self) {
        save_storage(&vec!["data".to_string(), self.db.user_name_space.clone(), self.db.name.clone()], Any::JsonArray(self.db.cached_value.clone()), false, ".json");
    }
}
