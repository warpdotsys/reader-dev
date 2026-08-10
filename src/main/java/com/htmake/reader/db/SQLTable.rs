// package com.htmake.reader.db

// import io.vertx.core.json.JsonArray
// import io.vertx.core.json.JsonObject
// import com.htmake.reader.utils.asJsonArray
// import com.htmake.reader.utils.getStorage
// import com.htmake.reader.utils.saveStorage

/**
 * SQL-based table implementation.
 * The packaged application persists this table through the storage backend.
 */
// class SQLTable<T>(userNameSpace: String, name: String) : DB<T>(userNameSpace, name) {
pub struct SQLTable<T> {
    pub db: DB<T>,
}

impl<T> SQLTable<T> {
    pub fn new(user_name_space: String, name: String) -> SQLTable<T> {
        SQLTable {
            db: DB::new(user_name_space, name),
        }
    }

    // override fun readAll(): JsonArray {
    pub fn read_all(&mut self) -> JsonArray {
        let data_list = as_json_array(get_storage(vec!["data".to_string(), self.db.user_name_space.clone(), self.db.name.clone()])).unwrap_or_else(|| JsonArray::new());
        self.db.cached_value = data_list.clone();
        return data_list;
    }

    // override fun <P> findBy(field: String, value: P, clazz: Class<T>): T? {
    pub fn find_by<P>(&mut self, field: &str, value: P, clazz: Class<T>) -> Option<T> {
        let data_list = self.read_all();
        for i in 0..data_list.size() {
            let obj = data_list.get_json_object(i);
            if value == obj.get_value(field) {
                return obj.map_to(clazz);
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
        let mut data_list = self.read_all();
        let mut existing_index = -1;
        for i in 0..data_list.size() {
            if checker(data_list.get_json_object(i).clone(), entity.clone()) {
                existing_index = i as i32;
                break;
            }
        }
        if let Some(cb) = on_check_end {
            cb(entity.clone(), existing_index >= 0, data_list.clone());
        }
        if existing_index >= 0 {
            data_list.list[existing_index as usize] = JsonObject::map_from(entity.clone());
            data_list = JsonArray::new(data_list.list.clone());
        } else {
            data_list.add(JsonObject::map_from(entity.clone()));
        }
        self.db.cached_value = data_list.clone();
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
        let mut data_list = self.read_all();
        let mut existing_index = -1;
        for entity in entities {
            for i in 0..data_list.size() {
                if checker(data_list.get_json_object(i).clone(), entity.clone()) {
                    existing_index = i as i32;
                    break;
                }
            }
            if let Some(cb) = on_check_end {
                cb(entity.clone(), existing_index >= 0, data_list.clone());
            }
            if existing_index >= 0 {
                data_list.list[existing_index as usize] = JsonObject::map_from(entity.clone());
                data_list = JsonArray::new(data_list.list.clone());
            } else {
                data_list.add(JsonObject::map_from(entity.clone()));
            }
        }
        self.db.cached_value = data_list.clone();
        self.save_only();
    }

    // override fun delete(predicate: (JsonObject) -> Boolean) {
    pub fn delete(&mut self, predicate: &dyn Fn(JsonObject) -> bool) {
        let mut data_list = self.read_all();
        let mut remove_indexes: Vec<i32> = Vec::new();
        for i in 0..data_list.size() {
            if predicate(data_list.get_json_object(i).clone()) {
                remove_indexes.push(i as i32);
            }
        }
        for index in remove_indexes {
            data_list.remove(index);
        }
        self.db.cached_value = data_list.clone();
        self.save_only();
    }

    // override fun save() {
    pub fn save_only(&mut self) {
        save_storage(vec!["data".to_string(), self.db.user_name_space.clone(), self.db.name.clone()], self.db.cached_value.clone());
    }
}
