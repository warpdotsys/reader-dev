// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class BookGroupController(coroutineContext: CoroutineContext): BaseController(coroutineContext), CURD<BookGroup> {
pub struct BookGroupController {
    base: BaseController,
}

impl BookGroupController {
    // override fun getTableName(): String {
    //     return "bookGroup"
    // }
    fn get_table_name(&self) -> String {
        return String::from("bookGroup");
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
        return json.get_long("groupId") == entity.group_id;
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
        let default_groups = as_json_array(String::from(
            "[{\"groupId\":-1,\"groupName\":\"全部\",\"order\":-10,\"show\":true},{\"groupId\":-2,\"groupName\":\"本地\",\"order\":-9,\"show\":true},{\"groupId\":-3,\"groupName\":\"音频\",\"order\":-8,\"show\":true},{\"groupId\":-4,\"groupName\":\"未分组\",\"order\":-7,\"show\":true},{\"groupId\":-5,\"groupName\":\"更新错误\",\"order\":-6,\"show\":true}]"
        )).unwrap_or_else(JsonArray::new);
        self.base.save_user_storage(&user_name_space, self.get_table_name(), Box::new(default_groups.clone()));
        return default_groups;
    }

    // override fun beforeSave(entity: BookGroup, db: DB<BookGroup>): ReturnData? {
    //     return if (entity.groupName.isEmpty()) ReturnData().setErrorMsg("分组名称不能为空") else null
    // }
    fn before_save(&self, entity: &BookGroup, db: &DB<BookGroup>) -> Option<ReturnData> {
        return if entity.group_name.is_empty() {
            Some(ReturnData::new().set_error_msg(String::from("分组名称不能为空")).clone())
        } else {
            None
        };
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
    fn on_check_end(&self, entity: &mut BookGroup, exists: bool, all_data: &JsonArray) {
        if exists {
            return;
        }
        let mut max_order = 0;
        let mut ids_sum = 0L;
        for item in all_data {
            let group = item.downcast_ref::<JsonObject>();
            if group.is_none() {
                continue;
            }
            let group = group.unwrap();
            max_order = max_order.max(group.get_integer("order", 0));
            ids_sum += group.get_long("groupId", 0L).max(0L);
        }
        let mut group_id = 1L;
        while (group_id & ids_sum) != 0L {
            group_id = group_id << 1;
        }
        entity.group_id = group_id;
        entity.order = max_order + 1;
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
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        let book_group_order = context.body_as_json().and_then(|j| j.get_json_array("order"));
        let book_group_order = match book_group_order {
            Some(v) => v,
            None => return return_data.set_error_msg(String::from("参数错误")),
        };
        let mut book_group_list = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("bookGroup")])).unwrap_or_else(JsonArray::new);
        let mut order_map: std::collections::HashMap<i64, i32> = std::collections::HashMap::new();
        for i in 0..book_group_order.size() {
            let item = book_group_order.get_json_object(i);
            if item.is_none() {
                continue;
            }
            let item = item.unwrap();
            let group_id = item.get_long("groupId");
            if group_id.is_none() {
                continue;
            }
            let group_id = group_id.unwrap();
            let order = item.get_integer("order");
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
            group_list[i] = JsonObject::map_from(group);
        }
        book_group_list = JsonArray::new(group_list);
        self.base.save_user_storage(&user_name_space, String::from("bookGroup"), Box::new(book_group_list));
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }
}
