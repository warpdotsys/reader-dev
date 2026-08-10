// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class BookSourceController(coroutineContext: CoroutineContext): BaseController(coroutineContext) {
pub struct BookSourceController {
    base: BaseController,
    // private var webClient: WebClient
    web_client: WebClient,
}

impl BookSourceController {
    // init {
    //     webClient = SpringContextUtils.getBean("webClient", WebClient::class.java)
    // }
    pub fn new() -> BookSourceController {
        BookSourceController {
            base: BaseController::new(),
            web_client: SpringContextUtils::get_bean_web_client(),
        }
    }

    // fun getUserBookSourceJsonOpt(userNameSpace: String, fields: Set<String>? = null, checkNotEmpty: Set<String>? = null): JsonArray? {
    //     var bookSourceFile = getStorageFile("data", userNameSpace, "bookSource")
    //     if (!bookSourceFile.exists()) {
    //         bookSourceFile = getStorageFile("data", "default", "bookSource")
    //     }
    //     return parseJsonStringList(bookSourceFile, fields = fields, checkNotEmpty = checkNotEmpty)
    // }
    pub fn get_user_book_source_json_opt(&self, user_name_space: String, fields: Option<std::collections::HashSet<String>>, check_not_empty: Option<std::collections::HashSet<String>>) -> Option<JsonArray> {
        let mut book_source_file = get_storage_file("data", user_name_space, "bookSource");
        if !book_source_file.exists() {
            book_source_file = get_storage_file("data", String::from("default"), "bookSource");
        }
        return parse_json_string_list(&book_source_file, fields, check_not_empty);
    }

    // fun getUserBookSourceJson(userNameSpace: String): JsonArray? {
    //     var bookSourceList: JsonArray? = asJsonArray(getUserStorage(userNameSpace, "bookSource"))
    //     if (bookSourceList == null && !userNameSpace.equals("default")) {
    //         // 用户书源文件不存在时使用默认书源，但不创建用户副本。
    //         var systemBookSourceList: JsonArray? = asJsonArray(getUserStorage("default", "bookSource"))
    //         if (systemBookSourceList != null) {
    //             bookSourceList = systemBookSourceList
    //         }
    //     }
    //     return bookSourceList
    // }
    pub fn get_user_book_source_json(&self, user_name_space: String) -> Option<JsonArray> {
        let mut book_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("bookSource")]));
        if book_source_list.is_none() && !user_name_space == String::from("default") {
            // 用户书源文件不存在时使用默认书源，但不创建用户副本。
            let system_book_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&String::from("default"), vec![String::from("bookSource")]));
            if let Some(system_book_source_list) = system_book_source_list {
                book_source_list = Some(system_book_source_list);
            }
        }
        return book_source_list;
    }

    // suspend fun canEditBookSource(context: RoutingContext): Boolean {
    //     if (!appConfig.secure) {
    //         return true
    //     }
    //     val userInfo = context.get("userInfo") as User? ?: return false
    //     return userInfo.enable_book_source
    // }
    pub fn can_edit_book_source(&self, context: &RoutingContext) -> bool {
        if !self.base.app_config.secure {
            return true;
        }
        let user_info = context.get_user::<User>("userInfo");
        let user_info = match user_info {
            Some(u) => u,
            None => return false,
        };
        return user_info.enable_book_source;
    }

    // suspend fun saveBookSource(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditBookSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val bookSource = BookSource.fromJson(context.bodyAsString).getOrNull()
    //     if (bookSource == null) {
    //         return returnData.setErrorMsg("参数错误")
    //     }
    //     // val bookSource = context.bodyAsJson.mapTo(BookSource::class.java)
    //
    //     var userNameSpace = getUserNameSpace(context)
    //     var bookSourceList = getUserBookSourceJson(userNameSpace)
    //     if (bookSourceList == null) {
    //         bookSourceList = JsonArray()
    //     }
    //     // 遍历判断书本是否存在
    //     var existIndex: Int = -1
    //     for (i in 0 until bookSourceList.size()) {
    //         var _bookSource = bookSourceList.getJsonObject(i).mapTo(BookSource::class.java)
    //         if (_bookSource.bookSourceUrl.equals(bookSource.bookSourceUrl)) {
    //             existIndex = i
    //             break;
    //         }
    //     }
    //     if (existIndex >= 0) {
    //         var sourceList = bookSourceList.getList()
    //         sourceList.set(existIndex, JsonObject.mapFrom(bookSource))
    //         bookSourceList = JsonArray(sourceList)
    //     } else {
    //         val user = context.get("userInfo") as User?
    //         if (user != null && bookSourceList.size() >= user.book_source_limit) {
    //             return returnData.setErrorMsg("你已达到书源数上限，请联系管理员")
    //         }
    //         bookSourceList.add(JsonObject.mapFrom(bookSource))
    //     }
    //
    //     // logger.info("bookSourceList: {}", bookSourceList)
    //     saveUserStorage(userNameSpace, "bookSource", bookSourceList)
    //     generateBookSourceMap(userNameSpace, bookSourceList)
    //     return returnData.setData("")
    // }
    pub fn save_book_source(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.can_edit_book_source(context) {
            return return_data.set_error_msg(String::from("权限不足"));
        }
        let book_source = BookSource::from_json(context.body_as_string()).get_or_none();
        if book_source.is_none() {
            return return_data.set_error_msg(String::from("参数错误"));
        }
        let book_source = book_source.unwrap();
        // val bookSource = context.bodyAsJson.mapTo(BookSource::class.java)

        let user_name_space = self.base.get_user_name_space(context);
        let mut book_source_list = self.get_user_book_source_json(user_name_space.clone());
        if book_source_list.is_none() {
            book_source_list = Some(JsonArray::new());
        }
        let mut book_source_list = book_source_list.unwrap();
        // 遍历判断书本是否存在
        let mut exist_index: i32 = -1;
        for i in 0..book_source_list.size() {
            let _book_source = book_source_list.get_json_object(i).map_to::<BookSource>();
            if _book_source.book_source_url == book_source.book_source_url {
                exist_index = i;
                break;
            }
        }
        if exist_index >= 0 {
            let mut source_list = book_source_list.get_list();
            source_list.set(exist_index as usize, JsonObject::map_from(book_source.clone()));
            book_source_list = JsonArray::new(source_list);
        } else {
            let user = context.get_user::<User>("userInfo");
            if let Some(user) = user {
                if book_source_list.size() >= user.book_source_limit {
                    return return_data.set_error_msg(String::from("你已达到书源数上限，请联系管理员"));
                }
            }
            book_source_list.add(JsonObject::map_from(book_source.clone()));
        }

        // logger.info("bookSourceList: {}", bookSourceList)
        self.base.save_user_storage(&user_name_space, String::from("bookSource"), Box::new(book_source_list.clone()));
        self.generate_book_source_map(user_name_space, Some(book_source_list));
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun saveBookSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditBookSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val bookSourceJsonArray = context.bodyAsJsonArray
    //     if (bookSourceJsonArray == null) {
    //         return returnData.setErrorMsg("参数错误")
    //     }
    //     return saveBookSources(context, bookSourceJsonArray)
    // }
    pub fn save_book_sources_ctx(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.can_edit_book_source(context) {
            return return_data.set_error_msg(String::from("权限不足"));
        }
        let book_source_json_array = context.body_as_json_array();
        if book_source_json_array.is_none() {
            return return_data.set_error_msg(String::from("参数错误"));
        }
        return self.save_book_sources(context, book_source_json_array.unwrap());
    }

    // fun saveBookSources(context: RoutingContext, bookSourceJsonArray: JsonArray): ReturnData {
    //     val userNameSpace = getUserNameSpace(context)
    //     val user = context.get("userInfo") as User?
    //     return saveUserBookSources(userNameSpace, user, bookSourceJsonArray)
    // }
    pub fn save_book_sources(&self, context: &RoutingContext, book_source_json_array: JsonArray) -> ReturnData {
        let user_name_space = self.base.get_user_name_space(context);
        let user = context.get_user::<User>("userInfo");
        return self.save_user_book_sources(user_name_space, user, book_source_json_array);
    }

    // fun saveUserBookSources(userNameSpace: String, user: User?, bookSourceJsonArray: JsonArray): ReturnData {
    //     val returnData = ReturnData()
    //     var bookSourceList = getUserBookSourceJson(userNameSpace)
    //     if (bookSourceList == null) {
    //         bookSourceList = JsonArray()
    //     }
    //     // Build a map of bookSourceUrl -> index for fast lookup
    //     val sourceMap = linkedMapOf<String, Int>()
    //     for (i in 0 until bookSourceList.size()) {
    //         val url = bookSourceList.getJsonObject(i).getString("bookSourceUrl")
    //         sourceMap[url] = i
    //     }
    //     var lastIndex = bookSourceList.size() - 1
    //     val updatedIndices = linkedSetOf<Int>()
    //     var reachedLimit = false
    //     var addedCount = 0
    //
    //     for (k in 0 until bookSourceJsonArray.size()) {
    //         val bookSource = try {
    //             BookSource.fromJson(bookSourceJsonArray.getJsonObject(k).toString()).getOrNull()
    //         } catch (e: Exception) {
    //             null
    //         }
    //         if (bookSource == null) continue
    //
    //         val existIndex = sourceMap.getOrDefault(bookSource.bookSourceUrl, -1)
    //         if (existIndex >= 0) {
    //             bookSourceList.set(existIndex, JsonObject.mapFrom(bookSource))
    //             if (existIndex <= lastIndex) {
    //                 updatedIndices.add(existIndex)
    //             }
    //         } else {
    //             if (user != null && bookSourceList.size() >= user.book_source_limit) {
    //                 reachedLimit = true
    //                 break
    //             }
    //             addedCount++
    //             bookSourceList.add(JsonObject.mapFrom(bookSource))
    //             sourceMap[bookSource.bookSourceUrl] = bookSourceList.size() - 1
    //         }
    //     }
    //
    //     saveUserStorage(userNameSpace, "bookSource", bookSourceList)
    //     generateBookSourceMap(userNameSpace, bookSourceList)
    //     val msg = "新增${addedCount}条书源，更新${updatedIndices.size}条书源"
    //     if (reachedLimit) {
    //         return returnData.setErrorMsg(msg + "。你已达到书源数上限，请联系管理员")
    //     }
    //     return returnData.setData("", msg)
    // }
    pub fn save_user_book_sources(&self, user_name_space: String, user: Option<User>, book_source_json_array: JsonArray) -> ReturnData {
        let mut return_data = ReturnData::new();
        let mut book_source_list = self.get_user_book_source_json(user_name_space.clone());
        if book_source_list.is_none() {
            book_source_list = Some(JsonArray::new());
        }
        let mut book_source_list = book_source_list.unwrap();
        // Build a map of bookSourceUrl -> index for fast lookup
        let mut source_map = std::collections::LinkedHashMap::new();
        for i in 0..book_source_list.size() {
            let url = book_source_list.get_json_object(i).get_string("bookSourceUrl");
            source_map.insert(url, i);
        }
        let last_index = book_source_list.size() - 1;
        let mut updated_indices = std::collections::LinkedHashSet::new();
        let mut reached_limit = false;
        let mut added_count = 0;

        for k in 0..book_source_json_array.size() {
            let book_source = match std::panic::catch_unwind(|| {
                BookSource::from_json(book_source_json_array.get_json_object(k).to_string()).get_or_none()
            }) {
                Ok(v) => v,
                Err(_) => None,
            };
            if book_source.is_none() {
                continue;
            }
            let book_source = book_source.unwrap();

            let exist_index = source_map.get(&book_source.book_source_url).cloned().unwrap_or(-1);
            if exist_index >= 0 {
                book_source_list.set(exist_index as usize, JsonObject::map_from(book_source.clone()));
                if exist_index <= last_index {
                    updated_indices.insert(exist_index);
                }
            } else {
                if let Some(user) = &user {
                    if book_source_list.size() >= user.book_source_limit {
                        reached_limit = true;
                        break;
                    }
                }
                added_count += 1;
                book_source_list.add(JsonObject::map_from(book_source.clone()));
                source_map.insert(book_source.book_source_url.clone(), book_source_list.size() - 1);
            }
        }

        self.base.save_user_storage(&user_name_space, String::from("bookSource"), Box::new(book_source_list.clone()));
        self.generate_book_source_map(user_name_space, Some(book_source_list));
        let msg = format!("新增{}条书源，更新{}条书源", added_count, updated_indices.len());
        if reached_limit {
            return return_data.set_error_msg(msg + "。你已达到书源数上限，请联系管理员");
        }
        return return_data.set_data(Box::new(String::from("")), msg);
    }

    // suspend fun getBookSource(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     checkAuth(context)
    //     var bookSourceUrl: String
    //     if (context.request().method() == HttpMethod.POST) {
    //         // post 请求
    //         bookSourceUrl = context.bodyAsJson.getString("bookSourceUrl")
    //     } else {
    //         // get 请求
    //         bookSourceUrl = context.queryParam("bookSourceUrl").firstOrNull() ?: ""
    //     }
    //     if (bookSourceUrl.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("书源链接不能为空")
    //     }
    //
    //     val userNameSpace = getUserNameSpace(context)
    //     val existIndex = getBookSourceMap(userNameSpace).getOrDefault(bookSourceUrl, -1)
    //     if (existIndex < 0) {
    //         return returnData.setErrorMsg("书源信息不存在")
    //     }
    //     val bookSourceList = getUserBookSourceJson(userNameSpace)
    //         ?: return returnData.setErrorMsg("书源信息不存在")
    //     if (existIndex >= bookSourceList.size()) {
    //         return returnData.setErrorMsg("书源信息不存在")
    //     }
    //     return returnData.setData(bookSourceList.getJsonObject(existIndex).map)
    // }
    pub fn get_book_source(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        self.base.check_auth(context);
        let book_source_url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            book_source_url = context.body_as_json().get_string("bookSourceUrl");
        } else {
            // get 请求
            book_source_url = context.query_param("bookSourceUrl").first().cloned().unwrap_or(String::from(""));
        }
        if book_source_url.is_empty() {
            return return_data.set_error_msg(String::from("书源链接不能为空"));
        }

        let user_name_space = self.base.get_user_name_space(context);
        let exist_index = self.get_book_source_map(user_name_space.clone()).get(&book_source_url).cloned().unwrap_or(-1);
        if exist_index < 0 {
            return return_data.set_error_msg(String::from("书源信息不存在"));
        }
        let book_source_list = match self.get_user_book_source_json(user_name_space) {
            Some(v) => v,
            None => return return_data.set_error_msg(String::from("书源信息不存在")),
        };
        if exist_index >= book_source_list.size() {
            return return_data.set_error_msg(String::from("书源信息不存在"));
        }
        return return_data.set_data(Box::new(book_source_list.get_json_object(exist_index).map()), String::from(""));
    }

    // suspend fun getBookSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     checkAuth(context)
    //     var simple: Int = 0
    //     if (context.request().method() == HttpMethod.POST) {
    //         // post 请求
    //         simple = context.bodyAsJson.getInteger("simple", 0)
    //     } else {
    //         // get 请求
    //         simple = context.queryParam("simple").firstOrNull()?.toInt() ?: 0
    //     }
    //     val userNameSpace = getUserNameSpace(context)
    //     val bookSourceList = getUserBookSourceJsonOpt(
    //         userNameSpace,
    //         fields = if (simple > 0) setOf("bookSourceGroup", "bookSourceName", "bookSourceUrl") else null,
    //         checkNotEmpty = if (simple > 0) setOf("exploreUrl") else null
    //     )
    //     if (bookSourceList != null) {
    //         return returnData.setData(bookSourceList.getList().map { JsonObject(it as String).map })
    //     }
    //     return returnData.setData(arrayListOf<Int>())
    // }
    pub fn get_book_sources(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        self.base.check_auth(context);
        let mut simple: i32 = 0;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            simple = context.body_as_json().get_integer("simple", 0);
        } else {
            // get 请求
            simple = context.query_param("simple").first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        }
        let user_name_space = self.base.get_user_name_space(context);
        let fields = if simple > 0 {
            Some(std::collections::HashSet::from([String::from("bookSourceGroup"), String::from("bookSourceName"), String::from("bookSourceUrl")]))
        } else {
            None
        };
        let check_not_empty = if simple > 0 {
            Some(std::collections::HashSet::from([String::from("exploreUrl")]))
        } else {
            None
        };
        let book_source_list = self.get_user_book_source_json_opt(user_name_space, fields, check_not_empty);
        if let Some(book_source_list) = book_source_list {
            return return_data.set_data(Box::new(book_source_list.get_list().into_iter().map(|item| JsonObject::new(item).map()).collect::<Vec<_>>()), String::from(""));
        }
        return return_data.set_data(Box::new(Vec::<i32>::new()), String::from(""));
    }

    // suspend fun deleteBookSource(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditBookSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val bookSource = BookSource.fromJson(context.bodyAsString).getOrNull()
    //         ?: return returnData.setErrorMsg("参数错误")
    //
    //     val userNameSpace = getUserNameSpace(context)
    //     var bookSourceList = getUserBookSourceJson(userNameSpace)
    //     if (bookSourceList == null) {
    //         bookSourceList = JsonArray()
    //     }
    //     val existIndex = getBookSourceMap(userNameSpace).getOrDefault(bookSource.bookSourceUrl, -1)
    //     if (existIndex >= 0) {
    //         bookSourceList.remove(existIndex)
    //     }
    //
    //     // logger.info("bookSourceList: {}", bookSourceList)
    //     saveUserStorage(userNameSpace, "bookSource", bookSourceList)
    //     generateBookSourceMap(userNameSpace, bookSourceList)
    //     return returnData.setData("")
    // }
    pub fn delete_book_source(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.can_edit_book_source(context) {
            return return_data.set_error_msg(String::from("权限不足"));
        }
        let book_source = match BookSource::from_json(context.body_as_string()).get_or_none() {
            Some(v) => v,
            None => return return_data.set_error_msg(String::from("参数错误")),
        };

        let user_name_space = self.base.get_user_name_space(context);
        let mut book_source_list = self.get_user_book_source_json(user_name_space.clone());
        if book_source_list.is_none() {
            book_source_list = Some(JsonArray::new());
        }
        let mut book_source_list = book_source_list.unwrap();
        let exist_index = self.get_book_source_map(user_name_space.clone()).get(&book_source.book_source_url).cloned().unwrap_or(-1);
        if exist_index >= 0 {
            book_source_list.remove(exist_index as usize);
        }

        // logger.info("bookSourceList: {}", bookSourceList)
        self.base.save_user_storage(&user_name_space, String::from("bookSource"), Box::new(book_source_list.clone()));
        self.generate_book_source_map(user_name_space, Some(book_source_list));
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun deleteBookSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditBookSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     val bookSourceJsonArray = context.bodyAsJsonArray
    //         ?: return returnData.setErrorMsg("参数错误")
    //
    //     var userNameSpace = getUserNameSpace(context)
    //     var bookSourceList = getUserBookSourceJson(userNameSpace)
    //     if (bookSourceList == null) {
    //         bookSourceList = JsonArray()
    //     }
    //     for (k in 0 until bookSourceJsonArray.size()) {
    //         var bookSource = bookSourceJsonArray.getJsonObject(k).mapTo(BookSource::class.java)
    //         // 遍历判断书本是否存在
    //         var existIndex: Int = -1
    //         for (i in 0 until bookSourceList.size()) {
    //             var _bookSource = bookSourceList.getJsonObject(i).mapTo(BookSource::class.java)
    //             if (_bookSource.bookSourceUrl.equals(bookSource.bookSourceUrl)) {
    //                 existIndex = i
    //                 break;
    //             }
    //         }
    //         if (existIndex >= 0) {
    //             bookSourceList.remove(existIndex)
    //         }
    //     }
    //
    //     // logger.info("bookSourceList: {}", bookSourceList)
    //     saveUserStorage(userNameSpace, "bookSource", bookSourceList)
    //     generateBookSourceMap(userNameSpace, bookSourceList)
    //     return returnData.setData("")
    // }
    pub fn delete_book_sources(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.can_edit_book_source(context) {
            return return_data.set_error_msg(String::from("权限不足"));
        }
        let book_source_json_array = match context.body_as_json_array() {
            Some(v) => v,
            None => return return_data.set_error_msg(String::from("参数错误")),
        };

        let user_name_space = self.base.get_user_name_space(context);
        let mut book_source_list = self.get_user_book_source_json(user_name_space.clone());
        if book_source_list.is_none() {
            book_source_list = Some(JsonArray::new());
        }
        let mut book_source_list = book_source_list.unwrap();
        for k in 0..book_source_json_array.size() {
            let book_source = book_source_json_array.get_json_object(k).map_to::<BookSource>();
            // 遍历判断书本是否存在
            let mut exist_index: i32 = -1;
            for i in 0..book_source_list.size() {
                let _book_source = book_source_list.get_json_object(i).map_to::<BookSource>();
                if _book_source.book_source_url == book_source.book_source_url {
                    exist_index = i;
                    break;
                }
            }
            if exist_index >= 0 {
                book_source_list.remove(exist_index as usize);
            }
        }

        // logger.info("bookSourceList: {}", bookSourceList)
        self.base.save_user_storage(&user_name_space, String::from("bookSource"), Box::new(book_source_list.clone()));
        self.generate_book_source_map(user_name_space, Some(book_source_list));
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun deleteAllBookSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!canEditBookSource(context)) {
    //         return returnData.setErrorMsg("权限不足")
    //     }
    //     var userNameSpace = getUserNameSpace(context)
    //     saveUserStorage(userNameSpace, "bookSource", JsonArray())
    //     generateBookSourceMap(userNameSpace, JsonArray())
    //     return returnData.setData("")
    // }
    pub fn delete_all_book_sources(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.can_edit_book_source(context) {
            return return_data.set_error_msg(String::from("权限不足"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        self.base.save_user_storage(&user_name_space, String::from("bookSource"), Box::new(JsonArray::new()));
        self.generate_book_source_map(user_name_space, Some(JsonArray::new()));
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun setAsDefaultBookSources(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     var username = context.bodyAsJson.getString("username")
    //     var bookSourceList: JsonArray? = asJsonArray(getUserStorage(username, "bookSource"))
    //     if (bookSourceList == null) {
    //         return returnData.setErrorMsg("用户书源不存在")
    //     }
    //
    //     // 保存为默认书源
    //     saveUserStorage("default", "bookSource", bookSourceList.getList())
    //     generateBookSourceMap("default", bookSourceList)
    //     return returnData.setData("设置默认书源成功")
    // }
    pub fn set_as_default_book_sources(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let username = context.body_as_json().get_string("username");
        let book_source_list: Option<JsonArray> = as_json_array(self.base.get_user_storage(&username, vec![String::from("bookSource")]));
        if book_source_list.is_none() {
            return return_data.set_error_msg(String::from("用户书源不存在"));
        }
        let book_source_list = book_source_list.unwrap();

        // 保存为默认书源
        self.base.save_user_storage(&String::from("default"), String::from("bookSource"), Box::new(book_source_list.get_list()));
        self.generate_book_source_map(String::from("default"), Some(book_source_list));
        return return_data.set_data(Box::new(String::from("设置默认书源成功")), String::from(""));
    }

    // suspend fun readSourceFile(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (context.fileUploads() == null || context.fileUploads().isEmpty()) {
    //         return returnData.setErrorMsg("请上传文件")
    //     }
    //     var sourceList = JsonArray()
    //     context.fileUploads().forEach {
    //         // logger.info("readSourceFile: {}", it.uploadedFileName())
    //         var file = File(it.uploadedFileName())
    //         if (file.exists()) {
    //             sourceList.add(file.readText())
    //             file.delete()
    //         }
    //     }
    //     return returnData.setData(sourceList.getList())
    // }
    pub fn read_source_file(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if context.file_uploads().is_none() || context.file_uploads().unwrap().is_empty() {
            return return_data.set_error_msg(String::from("请上传文件"));
        }
        let mut source_list = JsonArray::new();
        for upload in context.file_uploads().unwrap() {
            // logger.info("readSourceFile: {}", it.uploadedFileName())
            let file = File::new(&upload.uploaded_file_name());
            if file.exists() {
                source_list.add(file.read_text());
                file.delete();
            }
        }
        return return_data.set_data(Box::new(source_list.get_list()), String::from(""));
    }

    // suspend fun deleteUserBookSource(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     val userJsonArray = context.bodyAsJsonArray
    //     for (i in 0 until userJsonArray.size()) {
    //         var username = userJsonArray.getString(i)
    //         var userBookSourceFile = File(getWorkDir("storage", "data", username, "bookSource.json"))
    //         // 删除用户书源文件，恢复默认书源
    //         if (userBookSourceFile.exists()) {
    //             userBookSourceFile.deleteRecursively()
    //         }
    //     }
    //     return returnData.setData("删除书源成功")
    // }
    pub fn delete_user_book_source(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let user_json_array = context.body_as_json_array().unwrap();
        for i in 0..user_json_array.size() {
            let username = user_json_array.get_string(i);
            let user_book_source_file = File::new(&get_work_dir("storage", vec![String::from("data"), username, String::from("bookSource.json")]));
            // 删除用户书源文件，恢复默认书源
            if user_book_source_file.exists() {
                user_book_source_file.delete_recursively();
            }
        }
        return return_data.set_data(Box::new(String::from("删除书源成功")), String::from(""));
    }

    // suspend fun deleteBookSourcesFile(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     var userNameSpace = getUserNameSpace(context)
    //     var userBookSourceFile = File(getWorkDir("storage", "data", userNameSpace, "bookSource.json"))
    //     // 删除用户书源文件，恢复默认书源
    //     if (userBookSourceFile.exists()) {
    //         userBookSourceFile.deleteRecursively()
    //     }
    //     return returnData.setData("")
    // }
    pub fn delete_book_sources_file(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        let user_book_source_file = File::new(&get_work_dir("storage", vec![String::from("data"), user_name_space, String::from("bookSource.json")]));
        // 删除用户书源文件，恢复默认书源
        if user_book_source_file.exists() {
            user_book_source_file.delete_recursively();
        }
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun updateRemoteSourceSub(userNameSpace: String, user: User? = null) {
    //     val remoteBookSourceList = asJsonArray(getUserStorage(userNameSpace, "remoteBookSourceSub")) ?: return
    //     for (i in 0 until remoteBookSourceList.size()) {
    //         val remoteBookSource = remoteBookSourceList.getJsonObject(i) ?: continue
    //         val url = remoteBookSource.getString("link") ?: continue
    //         if (url.isEmpty()) continue
    //         try {
    //             val response = awaitResult<io.vertx.ext.web.client.HttpResponse<io.vertx.core.buffer.Buffer>> { handler ->
    //                 webClient.getAbs(url).timeout(3000).send(handler)
    //             }
    //             val sourceList = response.bodyAsJsonArray()
    //             if (sourceList != null) {
    //                 logger.info("updateRemoteSourceSub link={}, result={}", url, saveUserBookSources(userNameSpace, user, sourceList).errorMsg)
    //                 remoteBookSourceList.set(i, remoteBookSource.put("lastSyncTime", System.currentTimeMillis()))
    //                 saveUserStorage(userNameSpace, "remoteBookSourceSub", remoteBookSourceList)
    //             }
    //         } catch (e: Exception) {
    //             logger.error("更新远程书源失败", e)
    //             throw Exception("更新远程书源失败")
    //         }
    //     }
    //     generateBookSourceMap(userNameSpace)
    // }
    pub fn update_remote_source_sub(&self, user_name_space: String, user: Option<User>) {
        let remote_book_source_list = match as_json_array(self.base.get_user_storage(&user_name_space, vec![String::from("remoteBookSourceSub")])) {
            Some(v) => v,
            None => return,
        };
        for i in 0..remote_book_source_list.size() {
            let remote_book_source = remote_book_source_list.get_json_object(i);
            if remote_book_source.is_none() {
                continue;
            }
            let remote_book_source = remote_book_source.unwrap();
            let url = remote_book_source.get_string("link");
            if url.is_empty() {
                continue;
            }
            let result = std::panic::catch_unwind(|| {
                let response = await_result(|handler| {
                    self.web_client.get_abs(&url).timeout(3000).send(handler);
                });
                let source_list = response.body_as_json_array();
                if let Some(source_list) = source_list {
                    logger.info("updateRemoteSourceSub link={}, result={}", url, self.save_user_book_sources(user_name_space.clone(), user.clone(), source_list).error_msg());
                    remote_book_source_list.set(i, remote_book_source.put("lastSyncTime", System::current_time_millis()));
                    self.base.save_user_storage(&user_name_space, String::from("remoteBookSourceSub"), Box::new(remote_book_source_list.clone()));
                }
            });
            if result.is_err() {
                logger.error("更新远程书源失败", &result.unwrap_err());
                panic!("更新远程书源失败");
            }
        }
        self.generate_book_source_map(user_name_space, None);
    }

    // suspend fun saveFromRemoteSource(context: RoutingContext) {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         context.success(returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用"))
    //         return
    //     }
    //     var url: String
    //     if (context.request().method() == HttpMethod.POST) {
    //         url = context.bodyAsJson.getString("url") ?: ""
    //     } else {
    //         url = context.queryParam("url").firstOrNull() ?: ""
    //     }
    //     if (url.isNullOrEmpty()) {
    //         context.success(returnData.setErrorMsg("请输入远程书源链接"))
    //         return
    //     }
    //
    //     launch(MDCContext() + Dispatchers.IO) {
    //         webClient.getAbs(url).timeout(3000).send {
    //             var body = it.result()?.bodyAsString()
    //             if (body != null) {
    //                 context.success(returnData.setData(arrayListOf(body)))
    //             } else {
    //                 context.success(returnData.setErrorMsg("远程书源链接错误"))
    //             }
    //         }
    //     }
    // }
    pub fn save_from_remote_source(&self, context: &RoutingContext) {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            context.success(return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用")));
            return;
        }
        let url: String;
        if context.request().method() == HttpMethod::POST {
            url = context.body_as_json().get_string("url");
        } else {
            url = context.query_param("url").first().cloned().unwrap_or(String::from(""));
        }
        if url.is_empty() {
            context.success(return_data.set_error_msg(String::from("请输入远程书源链接")));
            return;
        }

        // launch(MDCContext() + Dispatchers.IO) {
        self.web_client.get_abs(&url).timeout(3000).send(|it| {
            let body = it.result().and_then(|r| r.body_as_string());
            if let Some(body) = body {
                context.success(return_data.set_data(Box::new(vec![body]), String::from("")));
            } else {
                context.success(return_data.set_error_msg(String::from("远程书源链接错误")));
            }
        });
        // }
    }

    // fun generateBookSourceMap(userNameSpace: String, bookSourceJsonArray: JsonArray? = null): Map<String, Int> {
    //     var bookSourceList = bookSourceJsonArray ?: getUserBookSourceJson(userNameSpace)
    //     if (bookSourceList == null) {
    //         bookSourceList = JsonArray()
    //     }
    //     val sourceMap = linkedMapOf<String, Int>()
    //     val exploreList = arrayListOf<Map<String, String?>>()
    //     for (i in 0 until bookSourceList.size()) {
    //         val sourceObj = bookSourceList.getJsonObject(i)
    //         val url = sourceObj.getString("bookSourceUrl")
    //         sourceMap[url] = i
    //         val exploreUrl = sourceObj.getString("exploreUrl")
    //         if (!exploreUrl.isNullOrEmpty()) {
    //             exploreList.add(mutableMapOf(
    //                 "bookSourceUrl" to sourceObj.getString("bookSourceUrl"),
    //                 "bookSourceGroup" to sourceObj.getString("bookSourceGroup"),
    //                 "bookSourceName" to sourceObj.getString("bookSourceName")
    //             ))
    //         }
    //     }
    //     saveUserStorage(userNameSpace, "bookSourceMap", sourceMap)
    //     saveUserStorage(userNameSpace, "bookSourceExploreList", exploreList)
    //     return sourceMap
    // }
    pub fn generate_book_source_map(&self, user_name_space: String, book_source_json_array: Option<JsonArray>) -> std::collections::HashMap<String, i32> {
        let mut book_source_list = book_source_json_array.or_else(|| self.get_user_book_source_json(user_name_space.clone()));
        if book_source_list.is_none() {
            book_source_list = Some(JsonArray::new());
        }
        let book_source_list = book_source_list.unwrap();
        let mut source_map = std::collections::LinkedHashMap::new();
        let mut explore_list: Vec<std::collections::HashMap<String, Option<String>>> = Vec::new();
        for i in 0..book_source_list.size() {
            let source_obj = book_source_list.get_json_object(i);
            let url = source_obj.get_string("bookSourceUrl");
            source_map.insert(url.clone(), i);
            let explore_url = source_obj.get_string("exploreUrl");
            if !explore_url.is_empty() {
                let mut m = std::collections::HashMap::new();
                m.insert(String::from("bookSourceUrl"), Some(source_obj.get_string("bookSourceUrl")));
                m.insert(String::from("bookSourceGroup"), Some(source_obj.get_string("bookSourceGroup")));
                m.insert(String::from("bookSourceName"), Some(source_obj.get_string("bookSourceName")));
                explore_list.push(m);
            }
        }
        self.base.save_user_storage(&user_name_space, String::from("bookSourceMap"), Box::new(source_map.clone()));
        self.base.save_user_storage(&user_name_space, String::from("bookSourceExploreList"), Box::new(explore_list));
        return source_map;
    }

    // fun getBookSourceMap(userNameSpace: String): Map<String, Int> {
    //     val bookSourceFile = getStorageFile("data", userNameSpace, "bookSource")
    //     val storageKey = if (bookSourceFile.exists()) userNameSpace else "default"
    //     val mapStr = getUserStorage(storageKey, "bookSourceMap")
    //     if (!mapStr.isNullOrEmpty()) {
    //         val mapJson = asJsonObject(mapStr)
    //         if (mapJson != null) {
    //             val result = mutableMapOf<String, Int>()
    //             for (entry in mapJson.map) {
    //                 result[entry.key] = (entry.value as? Number)?.toInt() ?: 0
    //             }
    //             return result
    //         }
    //     }
    //     // Map doesn't exist, generate it
    //     val sourceFile = getStorageFile("data", storageKey, "bookSource")
    //     return if (sourceFile.exists()) {
    //         generateBookSourceMap(storageKey)
    //     } else {
    //         generateBookSourceMap("default")
    //     }
    // }
    pub fn get_book_source_map(&self, user_name_space: String) -> std::collections::HashMap<String, i32> {
        let book_source_file = get_storage_file("data", user_name_space.clone(), "bookSource");
        let storage_key = if book_source_file.exists() { user_name_space } else { String::from("default") };
        let map_str = self.base.get_user_storage(&storage_key, vec![String::from("bookSourceMap")]);
        if let Some(map_str) = map_str {
            if !map_str.is_empty() {
                let map_json = as_json_object(map_str);
                if let Some(map_json) = map_json {
                    let mut result = std::collections::HashMap::new();
                    for (key, value) in map_json.map() {
                        result.insert(key, value.downcast_ref::<f64>().map(|n| *n as i32).or_else(|| value.downcast_ref::<i32>().copied()).unwrap_or(0));
                    }
                    return result;
                }
            }
        }
        // Map doesn't exist, generate it
        let source_file = get_storage_file("data", storage_key.clone(), "bookSource");
        return if source_file.exists() {
            self.generate_book_source_map(storage_key, None)
        } else {
            self.generate_book_source_map(String::from("default"), None)
        };
    }
}
