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
        json.map_to(self.get_entity_class())
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
    //     return null
    // }
    fn before_save(&self, entity: &T, db: &DB<T>) -> Option<ReturnData> {
        None
    }

    // fun beforeAdd(entity: T, db: DB<T>): ReturnData? {
    //     return null
    // }
    fn before_add(&self, entity: &T, db: &DB<T>) -> Option<ReturnData> {
        None
    }

    // fun beforeDelete(entity: T, db: DB<T>): ReturnData? {
    //     return null
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
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let user_ns = self.get_user_ns(context);
        let db = DB::table::<T>(user_ns.clone(), self.get_table_name());
        let all_data = db.read_all();
        let result = self.on_list(all_data, user_ns);
        return_data.set_data(Box::new(result.list()), String::from(""));
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
    //     if (beforeResult != null) {
    //         return beforeResult
    //     }
    //
    //     db.save(entity, { e, exists, data -> onCheckEnd(e, exists, data) }, { obj, e -> checker(obj, e) })
    //     return returnData.setData("")
    // }
    fn save(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let entity = self.convert_to_entity(&context.body_as_json());
        let user_ns = self.get_user_ns(context);
        let db = DB::table::<T>(user_ns, self.get_table_name());

        let before_result = self.before_save(&entity, &db);
        if let Some(result) = before_result {
            return result;
        }

        db.save(&entity, |e, exists, data| self.on_check_end(e, exists, data), |obj, e| self.checker(obj, e));
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
    //         if (beforeResult != null) {
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
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let entities = self.convert_to_entity_list(&context.body_as_string());
        if entities.is_empty() {
            return return_data.set_error_msg(String::from("参数错误"));
        }
        let user_ns = self.get_user_ns(context);
        let db = DB::table::<T>(user_ns, self.get_table_name());

        for entity in &entities {
            let before_result = self.before_save(entity, &db);
            if let Some(result) = before_result {
                return result;
            }
        }

        db.save_multi(&entities, |e, exists, data| self.on_check_end(e, exists, data), |obj, e| self.checker(obj, e));
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
    //     if (beforeResult != null) {
    //         return beforeResult
    //     }
    //
    //     db.delete { obj -> checker(obj, entity) }
    //     return returnData.setData("")
    // }
    fn delete(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let entity = self.convert_to_entity(&context.body_as_json());
        let user_ns = self.get_user_ns(context);
        let db = DB::table::<T>(user_ns, self.get_table_name());

        let before_result = self.before_delete(&entity, &db);
        if let Some(result) = before_result {
            return result;
        }

        db.delete(|obj| self.checker(obj, &entity));
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
    //         if (beforeResult != null) {
    //             return beforeResult
    //         }
    //     }
    //     db.delete { obj -> entities.any { entity -> checker(obj, entity) } }
    //     return returnData.setData("")
    // }
    fn delete_multi(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let entities = self.convert_to_entity_list(&context.body_as_string());
        if entities.is_empty() {
            return return_data.set_error_msg(String::from("参数错误"));
        }
        let user_ns = self.get_user_ns(context);
        let db = DB::table::<T>(user_ns, self.get_table_name());

        for entity in &entities {
            let before_result = self.before_delete(entity, &db);
            if let Some(result) = before_result {
                return result;
            }
        }
        db.delete(|obj| entities.iter().any(|entity| self.checker(obj, entity)));
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return_data
    }
}

// 外部依赖类型占位 (external dependency types: io.vertx / DB / gson / ReturnData)
pub struct JsonObject;
pub struct JsonArray;
pub struct RoutingContext;
pub struct DB<T> {
    _marker: std::marker::PhantomData<T>,
}
pub mod gson {
    pub fn from_json<T>(_json: &String, _class: std::any::TypeId) -> Vec<T> {
        unimplemented!()
    }
}
