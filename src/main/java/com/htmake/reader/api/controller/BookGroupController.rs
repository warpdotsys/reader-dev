use crate::prelude::*;
use crate::stubs::{JsonArray, JsonObject};
use crate::com_htmake_reader_db_db::DB;
// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class BookGroupController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<BookGroup> {
pub struct BookGroupController {
    base: BaseController,
}

impl BookGroupController {
    pub fn new() -> BookGroupController {
        BookGroupController {
            base: BaseController::new(),
        }
    }

    // Kotlin 按引用修改实体（trait 以 &T 传递 onCheckEnd，此处保留可变实体版本供其委托）
    pub fn on_check_end_mut(&self, entity: &mut BookGroup, exists: bool, all_data: &JsonArray) {
        if exists {
            return;
        }
        let mut max_order = 0;
        let mut ids_sum = 0_i64;
        // fix: Kotlin `for (item in allData) { item as? JsonObject }`——get_list 已逐项转 JsonObject
        for item in all_data.get_list() {
            max_order = max_order.max(item.get_integer("order", 0));
            ids_sum += item.get_long("groupId", 0_i64).max(0_i64);
        }
        let mut group_id = 1_i64;
        while (group_id & ids_sum) != 0_i64 {
            group_id = group_id << 1;
        }
        entity.group_id = group_id;
        entity.order = max_order + 1;
    }

    // suspend fun getBookGroups(context: RoutingContext): ReturnData {
    //     return list(context)
    // }
    pub fn get_book_groups(&self, context: &RoutingContext) -> ReturnData {
        return self.list(context);
    }

    // suspend fun saveBookGroup(context: RoutingContext): ReturnData {
    //     return save(context)
    // }
    pub fn save_book_group(&self, context: &RoutingContext) -> ReturnData {
        return self.save(context);
    }

    // suspend fun deleteBookGroup(context: RoutingContext): ReturnData {
    //     return delete(context)
    // }
    pub fn delete_book_group(&self, context: &RoutingContext) -> ReturnData {
        return self.delete(context);
    }

    // suspend fun saveBookGroupOrder(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val userNameSpace = getUserNameSpace(context)
    //     val bookGroupOrder = context.bodyAsJson?.getJsonArray("order") ?: return returnData.setErrorMsg("参数错误")
    //     var bookGroupList = com.htmake.reader.utils.asJsonArray(getUserStorage(userNameSpace, "bookGroup")) ?: JsonArray()
    //     val orderMap = mutableMapOf<Long, Int>()
    //     for (i in 0 until bookGroupOrder.size()) {
    //         val item = bookGroupOrder.getJsonObject(i) ?: continue
    //         val groupId = item.getLong("groupId") ?: continue
    //         val order = item.getInteger("order") ?: continue
    //         orderMap[groupId] = order
    //     }
    //     val groupList = bookGroupList.getList()
    //     for (i in 0 until bookGroupList.size()) {
    //         val group = bookGroupList.getJsonObject(i)?.mapTo(BookGroup::class.java) ?: continue
    //         orderMap[group.groupId]?.let { group.order = it }
    //         groupList[i] = JsonObject.mapFrom(group)
    //     }
    //     bookGroupList = JsonArray(groupList)
    //     saveUserStorage(userNameSpace, "bookGroup", bookGroupList)
    //     return returnData.setData("")
    // }
    pub fn save_book_group_order(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book_group_order = context.body_as_json().and_then(|j| j.get_json_array("order"));
        let book_group_order = match book_group_order {
            Some(v) => v,
            None => {
                return_data.set_error_msg(String::from("参数错误"));
                return return_data;
            }
        };
        let mut book_group_list = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("bookGroup")]).map(crate::stubs::Any::from_string)).unwrap_or_else(JsonArray::new);
        let mut order_map: std::collections::HashMap<i64, i32> = std::collections::HashMap::new();
        for i in 0..book_group_order.size() {
            let item = book_group_order.get_json_object(i);
            if item.is_none() {
                continue;
            }
            let item = item.unwrap();
            let group_id = item.get_long_opt("groupId");
            if group_id.is_none() {
                continue;
            }
            let group_id = group_id.unwrap();
            let order = item.get_integer_opt("order");
            if order.is_none() {
                continue;
            }
            let order = order.unwrap();
            order_map.insert(group_id, order);
        }
        let mut group_list = book_group_list.get_list();
        for i in 0..book_group_list.size() {
            let group = book_group_list.get_json_object(i).and_then(|o| o.map_to::<BookGroup>());
            if group.is_none() {
                continue;
            }
            let mut group = group.unwrap();
            if let Some(order) = order_map.get(&group.group_id) {
                group.order = *order;
            }
            group_list[i as usize] = JsonObject::map_from(group);
        }
        book_group_list = JsonArray::from_list(group_list);
        self.base.save_user_storage(&user_name_space, String::from("bookGroup"), Box::new(book_group_list));
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return return_data;
    }
}

impl CURD<BookGroup> for BookGroupController {
    // override fun getTableName(): String {
    //     return "bookGroup"
    // }
    fn get_table_name(&self) -> String {
        return String::from("bookGroup");
    }

    // fix: 覆写 save——DB::save 的 on_check_end 在实体序列化后调用（实体修改无效），
    //      此处先调用 on_check_end_mut 回写 groupId/order 再持久化
    fn save(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.check_user_auth(context) {
            return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
            return return_data;
        }
        let mut entity = self.convert_to_entity(&context.body_as_json().unwrap());
        let user_ns = self.get_user_ns(context);
        let mut db: DB<BookGroup> = DB::<BookGroup>::table::<BookGroup>(user_ns, self.get_table_name(), String::from("JSON"));
        let before_result = self.before_save(&entity, &db);
        if let Some(result) = before_result {
            return result;
        }
        // groupId/order 回写（新分组生成唯一 bitwise groupId）
        let all_data = db.read_all();
        let exists = all_data
            .get_list()
            .iter()
            .any(|obj| self.checker(&obj.clone(), &entity));
        if !exists {
            self.on_check_end_mut(&mut entity, false, &all_data);
        }
        let checker = |obj: JsonObject, e: &BookGroup| self.checker(&obj, e);
        db.save(entity, None, &checker);
        return_data.set_data(Box::new(String::from("")), String::from(""));
        return_data
    }

    // override fun getEntityClass(): Class<BookGroup> {
    //     return BookGroup::class.java
    // }
    fn get_entity_class(&self) -> std::any::TypeId {
        return std::any::TypeId::of::<BookGroup>();
    }

    // override fun checker(json: JsonObject, entity: BookGroup): Boolean {
    //     return json.getLong("groupId") == entity.groupId
    // }
    fn checker(&self, json: &JsonObject, entity: &BookGroup) -> bool {
        // fix: Kotlin `json.getLong("groupId")`（Long? 与 groupId 比较，无默认值版本）
        return json.get_long_opt("groupId") == Some(entity.group_id);
    }

    // override fun onList(list: JsonArray, userNameSpace: String): JsonArray {
    //     if (list.size() > 0) {
    //         return list
    //     }
    //     val defaultGroups = com.htmake.reader.utils.asJsonArray("""
    //         [{"groupId":-1,"groupName":"全部","order":-10,"show":true},{"groupId":-2,"groupName":"本地","order":-9,"show":true},{"groupId":-3,"groupName":"音频","order":-8,"show":true},{"groupId":-4,"groupName":"未分组","order":-7,"show":true},{"groupId":-5,"groupName":"更新错误","order":-6,"show":true}]
    //         """) ?: JsonArray()
    //     saveUserStorage(userNameSpace, getTableName(), defaultGroups)
    //     return defaultGroups
    // }
    fn on_list(&self, list: JsonArray, user_name_space: String) -> JsonArray {
        if list.size() > 0 {
            return list;
        }
        let default_groups = as_json_array(Some(crate::stubs::Any::Str(String::from(
            "[{\"groupId\":-1,\"groupName\":\"全部\",\"order\":-10,\"show\":true},{\"groupId\":-2,\"groupName\":\"本地\",\"order\":-9,\"show\":true},{\"groupId\":-3,\"groupName\":\"音频\",\"order\":-8,\"show\":true},{\"groupId\":-4,\"groupName\":\"未分组\",\"order\":-7,\"show\":true},{\"groupId\":-5,\"groupName\":\"更新错误\",\"order\":-6,\"show\":true}]"
        )))).unwrap_or_else(JsonArray::new);
        self.base.save_user_storage(&user_name_space, self.get_table_name(), Box::new(default_groups.clone()));
        return default_groups;
    }

    // override fun beforeSave(entity: BookGroup, db: DB<BookGroup>): ReturnData? {
    //     return if (entity.groupName.isEmpty()) ReturnData().setErrorMsg("分组名称不能为空") else None
    // }
    fn before_save(&self, entity: &BookGroup, db: &DB<BookGroup>) -> Option<ReturnData> {
        if entity.group_name.is_empty() {
            let mut return_data = ReturnData::new();
            return_data.set_error_msg(String::from("分组名称不能为空"));
            return Some(return_data);
        }
        return None;
    }

    // override fun onCheckEnd(entity: BookGroup, exists: Boolean, allData: JsonArray) {
    //     if (exists) {
    //         return
    //     }
    //     var maxOrder = 0
    //     var idsSum = 0L
    //     for (item in allData) {
    //         val group = item as? JsonObject ?: continue
    //         maxOrder = maxOf(maxOrder, group.getInteger("order", 0))
    //         idsSum += maxOf(group.getLong("groupId", 0L), 0L)
    //     }
    //     var groupId = 1L
    //     while (groupId and idsSum != 0L) {
    //         groupId = groupId shl 1
    //     }
    //     entity.groupId = groupId
    //     entity.order = maxOrder + 1
    // }
    fn on_check_end(&self, entity: &BookGroup, exists: bool, all_data: &JsonArray) {
        // fix: trait 签名 &T 无法可变修改实体（transmute/裸指针均被新版 rustc 拒绝），
        //      Kotlin 的 groupId/order 回写在此降级忽略
        let _ = (entity, exists, all_data);
    }

    // override suspend fun checkUserAuth(context: RoutingContext): Boolean {
    //     return checkAuth(context)
    // }
    fn check_user_auth(&self, context: &RoutingContext) -> bool {
        return self.base.check_auth(context);
    }

    // override fun getUserNS(context: RoutingContext): String {
    //     return getUserNameSpace(context)
    // }
    fn get_user_ns(&self, context: &RoutingContext) -> String {
        return self.base.get_user_name_space(context);
    }
}
