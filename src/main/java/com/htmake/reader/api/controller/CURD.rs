use crate::prelude::*;
// fix: 显式导入消解 prelude 多 glob 重导出歧义（JsonObject/JsonArray ← stubs；DB ← com_htmake_reader_db_db）
use crate::stubs::{JsonArray, JsonObject};
use crate::com_htmake_reader_db_db::DB;
// package com.htmake.reader.api.controller

// import io.vertx.core.json.JsonArray
// import io.vertx.core.json.JsonObject
// import io.vertx.ext.web.RoutingContext
// import mu.KotlinLogging
// import com.htmake.reader.api.ReturnData
// import com.htmake.reader.db.DB
// import com.htmake.reader.utils.gson

// private val logger = KotlinLogging.logger {}

/// Generic CURD interface providing default implementations for list/save/delete operations.
/// Uses DB<T> abstraction for persistence.
pub trait CURD<T> {
    // fun getTableName(): String
    fn get_table_name(&self) -> String;

    // fun getEntityClass(): Class<T>
    fn get_entity_class(&self) -> std::any::TypeId;

    // fun convertToEntity(json: JsonObject): T {
    //     return json.mapTo(getEntityClass())
    // }
    fn convert_to_entity(&self, json: &JsonObject) -> T {
        // fix: stubs JsonObject::map_to 恒返回 None（GSON 反序列化占位），无法构造 T
        json.map_to::<T>().expect("fix: JsonObject::map_to stub 恒返回 None")
    }

    // fun convertToEntityList(json: String): Array<T> {
    //     @Suppress("UNCHECKED_CAST")
    //     return gson.fromJson(json, java.lang.reflect.Array.newInstance(getEntityClass(), 0).javaClass) as Array<T>
    // }
    fn convert_to_entity_list(&self, json: &String) -> Vec<T> {
        gson::from_json(json, self.get_entity_class())
    }

    // fun onList(list: JsonArray, userNameSpace: String): JsonArray {
    //     return list
    // }
    fn on_list(&self, list: JsonArray, user_name_space: String) -> JsonArray {
        list
    }

    // fun checker(json: JsonObject, entity: T): Boolean
    fn checker(&self, json: &JsonObject, entity: &T) -> bool;

    // fun onCheckEnd(entity: T, exists: Boolean, allData: JsonArray) {
    // }
    fn on_check_end(&self, entity: &T, exists: bool, all_data: &JsonArray) {}

    // fun beforeSave(entity: T, db: DB<T>): ReturnData? {
    //     return None
    // }
    fn before_save(&self, entity: &T, db: &DB<T>) -> Option<ReturnData> {
        None
    }

    // fun beforeAdd(entity: T, db: DB<T>): ReturnData? {
    //     return None
    // }
    fn before_add(&self, entity: &T, db: &DB<T>) -> Option<ReturnData> {
        None
    }

    // fun beforeDelete(entity: T, db: DB<T>): ReturnData? {
    //     return None
    // }
    fn before_delete(&self, entity: &T, db: &DB<T>) -> Option<ReturnData> {
        None
    }

    // suspend fun checkUserAuth(context: RoutingContext): Boolean
    fn check_user_auth(&self, context: &RoutingContext) -> bool;

    // fun getUserNS(context: RoutingContext): String
    fn get_user_ns(&self, context: &RoutingContext) -> String;

    // suspend fun list(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkUserAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val userNS = getUserNS(context)
    //     val db = DB.table<T>(userNS, getTableName())
    //     val allData = db.readAll()
    //     val result = onList(allData, userNS)
    //     return returnData.setData(result.list)
    // }
    fn list(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let user_ns = self.get_user_ns(context);
        let mut db: DB<T> = DB::<T>::table::<T>(user_ns.clone(), self.get_table_name(), String::from("JSON"));
        let all_data = db.read_all();
        let result = self.on_list(all_data, user_ns);
        // fix: Kotlin result.list 属性在占位 JsonArray 中不存在，直接装箱整个 JsonArray
        return_data.set_data(Box::new(result), String::from(""));
        return_data
    }

    // suspend fun save(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkUserAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val entity = convertToEntity(context.bodyAsJson)
    //     val userNS = getUserNS(context)
    //     val db = DB.table<T>(userNS, getTableName())
    //
    //     val beforeResult = beforeSave(entity, db)
    //     if (beforeResult != None) {
    //         return beforeResult
    //     }
    //
    //     db.save(entity, { e, exists, data -> onCheckEnd(e, exists, data) }, { obj, e -> checker(obj, e) })
    //     return returnData.setData("")
    // }
    fn save(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let entity = self.convert_to_entity(&context.body_as_json().unwrap());
        let user_ns = self.get_user_ns(context);
        let mut db: DB<T> = DB::<T>::table::<T>(user_ns, self.get_table_name(), String::from("JSON"));

        let before_result = self.before_save(&entity, &db);
        if let Some(result) = before_result {
            return result;
        }

        let on_check_end = |e: T, exists: bool, data: JsonArray| self.on_check_end(&e, exists, &data);
        let checker = |obj: JsonObject, e: T| self.checker(&obj, &e);
        db.save(entity, Some(&on_check_end), &checker);
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return_data
    }

    // suspend fun saveMulti(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkUserAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val entities = convertToEntityList(context.bodyAsString)
    //     if (entities.isEmpty()) return returnData.setErrorMsg("参数错误")
    //     val userNS = getUserNS(context)
    //     val db = DB.table<T>(userNS, getTableName())
    //
    //     for (entity in entities) {
    //         val beforeResult = beforeSave(entity, db)
    //         if (beforeResult != None) {
    //             return beforeResult
    //         }
    //     }
    //
    //     db.saveMulti(entities, { e, exists, data -> onCheckEnd(e, exists, data) }, { obj, e -> checker(obj, e) })
    //     return returnData.setData("")
    // }
    fn save_multi(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let entities = self.convert_to_entity_list(&context.body_as_string());
        if entities.is_empty() {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let user_ns = self.get_user_ns(context);
        let mut db: DB<T> = DB::<T>::table::<T>(user_ns, self.get_table_name(), String::from("JSON"));

        for entity in &entities {
            let before_result = self.before_save(entity, &db);
            if let Some(result) = before_result {
                return result;
            }
        }

        let on_check_end = |e: T, exists: bool, data: JsonArray| self.on_check_end(&e, exists, &data);
        let checker = |obj: JsonObject, e: T| self.checker(&obj, &e);
        db.save_multi(entities, Some(&on_check_end), &checker);
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return_data
    }

    // suspend fun delete(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkUserAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val entity = convertToEntity(context.bodyAsJson)
    //     val userNS = getUserNS(context)
    //     val db = DB.table<T>(userNS, getTableName())
    //
    //     val beforeResult = beforeDelete(entity, db)
    //     if (beforeResult != None) {
    //         return beforeResult
    //     }
    //
    //     db.delete { obj -> checker(obj, entity) }
    //     return returnData.setData("")
    // }
    fn delete(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let entity = self.convert_to_entity(&context.body_as_json().unwrap());
        let user_ns = self.get_user_ns(context);
        let mut db: DB<T> = DB::<T>::table::<T>(user_ns, self.get_table_name(), String::from("JSON"));

        let before_result = self.before_delete(&entity, &db);
        if let Some(result) = before_result {
            return result;
        }

        let predicate = |obj: JsonObject| self.checker(&obj, &entity);
        db.delete(&predicate);
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return_data
    }

    // suspend fun deleteMulti(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkUserAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val entities = convertToEntityList(context.bodyAsString)
    //     if (entities.isEmpty()) return returnData.setErrorMsg("参数错误")
    //     val userNS = getUserNS(context)
    //     val db = DB.table<T>(userNS, getTableName())
    //
    //     for (entity in entities) {
    //         val beforeResult = beforeDelete(entity, db)
    //         if (beforeResult != None) {
    //             return beforeResult
    //         }
    //     }
    //     db.delete { obj -> entities.any { entity -> checker(obj, entity) } }
    //     return returnData.setData("")
    // }
    fn delete_multi(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let entities = self.convert_to_entity_list(&context.body_as_string());
        if entities.is_empty() {
            return_data.set_error_msg(String::from("参数错误"));
            return return_data;
        }
        let user_ns = self.get_user_ns(context);
        let mut db: DB<T> = DB::<T>::table::<T>(user_ns, self.get_table_name(), String::from("JSON"));

        for entity in &entities {
            let before_result = self.before_delete(entity, &db);
            if let Some(result) = before_result {
                return result;
            }
        }
        let predicate = |obj: JsonObject| entities.iter().any(|entity| self.checker(&obj, entity));
        db.delete(&predicate);
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return_data
    }
}

// 外部依赖类型占位 (external dependency types: io.vertx / DB / gson / ReturnData)
// fix: RoutingContext 保留为本地占位——prelude glob 唯一可见的 RoutingContext（stubs 内嵌 vertx 模块不可 glob），
// stubs.rs 内已为其补充 request/body_as_json/getUser 等方法
pub struct RoutingContext;
// fix: JsonObject/JsonArray 直接使用 stubs 占位（原本地单元结构体与 stubs::body_as_json 返回类型不一致）
// fix: DB 直接使用 com_htmake_reader_db_db::DB（原本地占位缺少 table/read_all/save/save_multi/delete 方法）

mod gson {
    pub fn from_json<T>(_json: &String, _class: std::any::TypeId) -> Vec<T> {
        // fix: GSON 反序列化占位，恒返回空列表（原 Kotlin gson.fromJson(json, Array<T>)）
        Vec::new()
    }
}




