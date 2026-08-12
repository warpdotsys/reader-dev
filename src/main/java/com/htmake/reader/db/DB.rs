use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（优先于 prelude 的 glob 导入）
use crate::stubs::{JsonArray, JsonObject};
use std::marker::PhantomData;
// package com.htmake.reader.db

// import io.vertx.core.json.JsonArray
// import io.vertx.core.json.JsonObject

/**
 * Abstract database class providing a base for JSON and SQL table implementations.
 */
// open class DB<T>(
pub struct DB<T> {
    pub user_name_space: String,
    pub name: String,
    pub cached_value: JsonArray,
    // fix: T 仅出现在方法签名中（原 Kotlin 泛型表类型），Rust 需 PhantomData 占位
    _phantom: PhantomData<T>,
}

impl<T> DB<T> {
    pub fn new(user_name_space: String, name: String) -> DB<T> {
        DB::<T> {
            user_name_space,
            name,
            cached_value: JsonArray::new(),
            _phantom: PhantomData::default(),
        }
    }

    // open fun readAll(): JsonArray {
    pub fn read_all(&mut self) -> JsonArray {
        return JsonArray::new();
    }

    // open fun <P> findBy(field: String, value: P, clazz: Class<T>): T? {
    pub fn find_by<P>(&mut self, field: &str, value: P, clazz: Class<T>) -> Option<T> {
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
        checker: &dyn Fn(JsonObject, T) -> bool,
    ) {
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
        checker: &dyn Fn(JsonObject, T) -> bool,
    ) {
    }

    // open fun delete(predicate: (JsonObject) -> Boolean) {
    pub fn delete(&mut self, predicate: &dyn Fn(JsonObject) -> bool) {
    }

    // open fun save() {
    pub fn save_only(&mut self) {
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
    pub fn table<T2>(user_name_space: String, name: String, driver: String) -> DB<T2> {
        return if driver == "SQL" {
            DB::<T2>::new(user_name_space, name)
        } else {
            DB::<T2>::new(user_name_space, name)
        };
    }
}
