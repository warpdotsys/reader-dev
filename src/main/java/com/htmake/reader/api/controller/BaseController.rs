// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// open class BaseController(override val coroutineContext: CoroutineContext): CoroutineScope {
pub struct BaseController {
    // var loginExpireDays = 7
    login_expire_days: i32,

    // val appConfig: AppConfig
    app_config: AppConfig,

    // val env: Environment
    env: Environment,

    // val userMutex = Mutex()
    user_mutex: Mutex,
}

impl BaseController {
    // init {
    //     appConfig = SpringContextUtils.getBean("appConfig", AppConfig::class.java)
    //     env = SpringContextUtils.getBean(Environment::class.java)
    // }
    pub fn new() -> BaseController {
        BaseController {
            login_expire_days: 7,
            app_config: SpringContextUtils::get_bean_app_config(),
            env: SpringContextUtils::get_bean_environment(),
            user_mutex: Mutex::new(),
        }
    }

    // suspend fun saveUserSession(context: RoutingContext, user: User, regenerateToken: Boolean = true): Map<String, Any> {
    //     return userMutex.withLock {
    //         var userMap = mutableMapOf<String, Map<String, Any>>()
    //         var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //         if (userMapJson != null) {
    //             userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //         }
    //         user.last_login_at = System.currentTimeMillis()
    //         if (regenerateToken) {
    //             user.token = genEncryptedPassword(user.username, System.currentTimeMillis().toString())
    //             var tokenMap: MutableMap<String, Long>? = null
    //             var expire = System.currentTimeMillis() + loginExpireDays * 86400 * 1000
    //             if (user.token_map != null) {
    //                 tokenMap = user.token_map as? MutableMap<String, Long>
    //             }
    //             if (tokenMap == null) {
    //                 tokenMap = mutableMapOf(user.token to expire)
    //             } else {
    //                 tokenMap.put(user.token, expire)
    //             }
    //             // 删除已过期token
    //             tokenMap.values.removeAll { it < user.last_login_at }
    //             user.token_map = tokenMap
    //         }
    //         userMap.put(user.username, user.toMap())
    //         saveStorage("data", "users", value = Json.encode(userMap))
    //
    //         val loginData = formatUser(user)
    //
    //         context.session().put("username", user.username)
    //         context.put("username", user.username)
    //
    //         loginData
    //     }
    // }
    pub fn save_user_session(&self, context: &RoutingContext, user: &mut User, regenerate_token: bool) -> std::collections::HashMap<String, Box<dyn std::any::Any>> {
        let _guard = self.user_mutex.with_lock();
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", "users"));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        user.last_login_at = System::current_time_millis();
        if regenerate_token {
            user.token = gen_encrypted_password(&user.username, &System::current_time_millis().to_string());
            let mut token_map: Option<std::collections::HashMap<String, i64>> = None;
            let expire = System::current_time_millis() + self.login_expire_days as i64 * 86400 * 1000;
            if user.token_map.is_some() {
                token_map = user.token_map.clone();
            }
            match token_map {
                None => {
                    let mut m = std::collections::HashMap::new();
                    m.insert(user.token.clone(), expire);
                    token_map = Some(m);
                }
                Some(mut m) => {
                    m.insert(user.token.clone(), expire);
                    token_map = Some(m);
                }
            }
            // 删除已过期token
            if let Some(m) = token_map.as_mut() {
                m.retain(|_, v| *v >= user.last_login_at);
            }
            user.token_map = token_map;
        }
        user_map.insert(user.username.clone(), user.to_map());
        save_storage("data", "users", serde_json::to_string(&user_map).unwrap());

        let login_data = self.format_user(&user);

        context.session().put("username", user.username.clone());
        context.put("username", user.username.clone());

        login_data
    }

    // suspend fun checkAuth(context: RoutingContext): Boolean {
    //     if (!appConfig.secure) {
    //         return true
    //     }
    //     var username = context.session().get("username") as String? ?: ""
    //     var userInfo = getUserInfoClass(username)
    //     if (userInfo != null) {
    //         context.put("username", userInfo.username)
    //         context.put("userInfo", userInfo)
    //         return true
    //     }
    //     // 自动登录
    //     var accessToken = context.queryParam("accessToken").firstOrNull() ?: ""
    //     if (accessToken.isNotEmpty()) {
    //         var userMap = mutableMapOf<String, Map<String, Any>>()
    //         var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //         if (userMapJson != null) {
    //             userMap = userMapJson.map as? MutableMap<String, Map<String, Any>> ?: mutableMapOf<String, Map<String, Any>>()
    //         }
    //         var tmp = accessToken.split(":", limit=2)
    //         if (tmp.size >= 2) {
    //             var _username = tmp[0]
    //             var token = tmp[1]
    //             var existedUser: User? = userMap.getOrDefault(_username, null)?.toDataClass()
    //             if (existedUser != null && token.isNotEmpty()) {
    //                 var isLogin = false
    //                 if (existedUser.token.isNotEmpty() && existedUser.token.equals(token)) {
    //                     isLogin = true
    //                 }
    //                 // 查找历史有效会话
    //                 if (!isLogin && existedUser.token_map != null) {
    //                     var tokenMap = existedUser.token_map as? MutableMap<String, Long>
    //                     if (tokenMap != null &&
    //                         tokenMap.containsKey(token)) {
    //                         if (tokenMap.getOrDefault(token, 0L) > System.currentTimeMillis()) {
    //                             isLogin = true
    //                             // 延长有效期
    //                             tokenMap.put(token, System.currentTimeMillis() + loginExpireDays * 86400 * 1000)
    //                         } else {
    //                             // 删除过期token
    //                             tokenMap.remove(token)
    //                         }
    //                         existedUser.token_map = tokenMap
    //                     }
    //                 }
    //                 if (isLogin) {
    //                     // 保存用户session
    //                     saveUserSession(context, existedUser, false)
    //                     context.put("username", existedUser.username)
    //                     context.put("userInfo", existedUser)
    //                 }
    //                 return isLogin
    //             }
    //         }
    //     }
    //
    //     return false
    // }
    pub fn check_auth(&self, context: &RoutingContext) -> bool {
        if !self.app_config.secure {
            return true;
        }
        let mut username = context.session().get("username").unwrap_or(String::from(""));
        let user_info = self.get_user_info_class(&username);
        if let Some(user_info) = user_info {
            context.put("username", user_info.username.clone());
            context.put("userInfo", &user_info);
            return true;
        }
        // 自动登录
        let access_token = context.query_param("accessToken").first().unwrap_or(&String::from("")).clone();
        if !access_token.is_empty() {
            let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
            let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", "users"));
            if let Some(json) = user_map_json {
                user_map = json.map().clone();
            }
            let tmp: Vec<&str> = access_token.splitn(2, ':').collect();
            if tmp.len() >= 2 {
                let _username = tmp[0].to_string();
                let token = tmp[1].to_string();
                let existed_user: Option<User> = user_map.get(&_username).and_then(|v| v.to_data_class());
                if let Some(mut existed_user) = existed_user {
                    if !token.is_empty() {
                        let mut is_login = false;
                        if !existed_user.token.is_empty() && existed_user.token == token {
                            is_login = true;
                        }
                        // 查找历史有效会话
                        if !is_login && existed_user.token_map.is_some() {
                            let token_map = existed_user.token_map.clone();
                            if let Some(mut token_map) = token_map {
                                if token_map.contains_key(&token) {
                                    if *token_map.get(&token).unwrap_or(&0L) > System::current_time_millis() {
                                        is_login = true;
                                        // 延长有效期
                                        token_map.insert(token.clone(), System::current_time_millis() + self.login_expire_days as i64 * 86400 * 1000);
                                    } else {
                                        // 删除过期token
                                        token_map.remove(&token);
                                    }
                                    existed_user.token_map = Some(token_map);
                                }
                            }
                        }
                        if is_login {
                            // 保存用户session
                            self.save_user_session(context, &mut existed_user, false);
                            context.put("username", existed_user.username.clone());
                            context.put("userInfo", &existed_user);
                        }
                        return is_login;
                    }
                }
            }
        }

        return false;
    }

    // fun checkManagerAuth(context: RoutingContext): Boolean {
    //     if (!appConfig.secure) {
    //         return true
    //     }
    //     if (appConfig.secureKey.isEmpty()) {
    //         return true
    //     }
    //     var secureKey = context.queryParam("secureKey").firstOrNull() ?: ""
    //     if (secureKey.equals(appConfig.secureKey)) {
    //         // 判断是否需要修改 userNameSpace
    //         var userNS = context.queryParam("userNS").firstOrNull()
    //         if (userNS != null && userNS.isNotEmpty()) {
    //             context.put("userNameSpace", userNS)
    //         } else {
    //             context.remove("userNameSpace")
    //         }
    //         return true
    //     }
    //     return false
    // }
    pub fn check_manager_auth(&self, context: &RoutingContext) -> bool {
        if !self.app_config.secure {
            return true;
        }
        if self.app_config.secure_key.is_empty() {
            return true;
        }
        let secure_key = context.query_param("secureKey").first().unwrap_or(&String::from("")).clone();
        if secure_key == self.app_config.secure_key {
            // 判断是否需要修改 userNameSpace
            let user_ns = context.query_param("userNS").first().cloned();
            if let Some(user_ns) = user_ns {
                if !user_ns.is_empty() {
                    context.put("userNameSpace", user_ns);
                } else {
                    context.remove("userNameSpace");
                }
            } else {
                context.remove("userNameSpace");
            }
            return true;
        }
        return false;
    }

    // fun getUserNameSpace(context: RoutingContext): String {
    //     if (!appConfig.secure) {
    //         return "default"
    //     }
    //     // 管理权限，可以修改 userNameSpace 来获取任意用户信息
    //     checkManagerAuth(context)
    //     var userNS = context.get("userNameSpace") as String?
    //     if (userNS != null && userNS.isNotEmpty()) {
    //         return userNS
    //     }
    //     var username = context.get("username") as String?
    //     if (username != null) {
    //         return username;
    //     }
    //     return "default"
    // }
    pub fn get_user_name_space(&self, context: &RoutingContext) -> String {
        if !self.app_config.secure {
            return String::from("default");
        }
        // 管理权限，可以修改 userNameSpace 来获取任意用户信息
        self.check_manager_auth(context);
        let user_ns = context.get("userNameSpace");
        if let Some(user_ns) = user_ns {
            if !user_ns.is_empty() {
                return user_ns;
            }
        }
        let username = context.get("username");
        if let Some(username) = username {
            return username;
        }
        return String::from("default");
    }

    // fun getUserStorage(context: Any, vararg path: String): String? {
    //     var userNameSpace = ""
    //     when(context) {
    //         is RoutingContext -> userNameSpace = getUserNameSpace(context)
    //         is String -> userNameSpace = context
    //     }
    //     if (userNameSpace.isEmpty()) {
    //         return getStorage("data", *path)
    //     }
    //     return getStorage("data", userNameSpace, *path)
    // }
    pub fn get_user_storage(&self, context: &std::any::Any, path: Vec<String>) -> Option<String> {
        let mut user_name_space = String::from("");
        if let Some(ctx) = context.downcast_ref::<RoutingContext>() {
            user_name_space = self.get_user_name_space(ctx);
        } else if let Some(s) = context.downcast_ref::<String>() {
            user_name_space = s.clone();
        }
        if user_name_space.is_empty() {
            return get_storage("data", &path);
        }
        let mut full_path = vec![user_name_space];
        full_path.extend(path);
        return get_storage("data", &full_path);
    }

    // fun saveUserStorage(context: Any, path: String, value: Any) {
    //     var userNameSpace = ""
    //     when(context) {
    //         is RoutingContext -> userNameSpace = getUserNameSpace(context)
    //         is String -> userNameSpace = context
    //     }
    //     if (userNameSpace.isEmpty()) {
    //         return saveStorage("data", path, value = value)
    //     }
    //     return saveStorage("data", userNameSpace, path, value = value)
    // }
    pub fn save_user_storage(&self, context: &std::any::Any, path: String, value: Box<dyn std::any::Any>) {
        let mut user_name_space = String::from("");
        if let Some(ctx) = context.downcast_ref::<RoutingContext>() {
            user_name_space = self.get_user_name_space(ctx);
        } else if let Some(s) = context.downcast_ref::<String>() {
            user_name_space = s.clone();
        }
        if user_name_space.is_empty() {
            return save_storage("data", vec![path], value);
        }
        return save_storage("data", vec![user_name_space, path], value);
    }

    // fun getUserInfoClass(username: String): User? {
    //     var user: User? = getUserInfoMap(username)?.toDataClass()
    //     return user
    // }
    pub fn get_user_info_class(&self, username: String) -> Option<User> {
        let user: Option<User> = self.get_user_info_map(&username).and_then(|m| m.to_data_class());
        return user;
    }

    // fun getUserInfoMap(username: String): Map<String, Any>? {
    //     if (username.isEmpty()) {
    //         return null
    //     }
    //     var userMap = mutableMapOf<String, Map<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     return userMap.getOrDefault(username, null)
    // }
    pub fn get_user_info_map(&self, username: &String) -> Option<std::collections::HashMap<String, Box<dyn std::any::Any>>> {
        if username.is_empty() {
            return None;
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", "users"));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        return user_map.get(username).cloned();
    }

    // fun formatUser(userInfo: Any): MutableMap<String, Any> {
    //     var user: User? = null
    //     if (userInfo !is User) {
    //         var userMap = userInfo as? Map<String, Any>
    //         if (userMap != null) {
    //             user = userMap.toDataClass()
    //         }
    //     } else {
    //         user = userInfo
    //     }
    //     if (user == null) {
    //         return mutableMapOf()
    //     }
    //     return mutableMapOf(
    //         "username" to user.username,
    //         "lastLoginAt" to user.last_login_at,
    //         "accessToken" to user.username + ":" + user.token,
    //         "enableWebdav" to user.enable_webdav,
    //         "enableLocalStore" to user.enable_local_store,
    //         "enableBookSource" to user.enable_book_source,
    //         "enableRssSource" to user.enable_rss_source,
    //         "bookSourceLimit" to user.book_source_limit,
    //         "bookLimit" to user.book_limit,
    //         "createdAt" to user.created_at
    //     )
    // }
    pub fn format_user(&self, user_info: &std::any::Any) -> std::collections::HashMap<String, Box<dyn std::any::Any>> {
        let mut user: Option<User> = None;
        if let Some(u) = user_info.downcast_ref::<User>() {
            user = Some(u.clone());
        } else {
            let user_map = user_info.downcast_ref::<std::collections::HashMap<String, Box<dyn std::any::Any>>>();
            if let Some(user_map) = user_map {
                user = user_map.to_data_class();
            }
        }
        if user.is_none() {
            return std::collections::HashMap::new();
        }
        let user = user.unwrap();
        let mut result = std::collections::HashMap::new();
        result.insert(String::from("username"), Box::new(user.username));
        result.insert(String::from("lastLoginAt"), Box::new(user.last_login_at));
        result.insert(String::from("accessToken"), Box::new(user.username + ":" + &user.token));
        result.insert(String::from("enableWebdav"), Box::new(user.enable_webdav));
        result.insert(String::from("enableLocalStore"), Box::new(user.enable_local_store));
        result.insert(String::from("enableBookSource"), Box::new(user.enable_book_source));
        result.insert(String::from("enableRssSource"), Box::new(user.enable_rss_source));
        result.insert(String::from("bookSourceLimit"), Box::new(user.book_source_limit));
        result.insert(String::from("bookLimit"), Box::new(user.book_limit));
        result.insert(String::from("createdAt"), Box::new(user.created_at));
        result
    }

    // fun getUserWebdavHome(context: Any): String {
    //     var prefix = getWorkDir("storage", "data")
    //     var userNameSpace = ""
    //     when(context) {
    //         is RoutingContext -> userNameSpace = getUserNameSpace(context)
    //         is String -> userNameSpace = context
    //     }
    //     if (userNameSpace.isNotEmpty()) {
    //         prefix = prefix + File.separator + userNameSpace
    //     }
    //     prefix = prefix + File.separator + "webdav"
    //     var file = File(prefix)
    //     if (!file.exists()) {
    //         file.mkdirs()
    //     }
    //     return prefix
    // }
    pub fn get_user_webdav_home(&self, context: &std::any::Any) -> String {
        let mut prefix = get_work_dir("storage", "data");
        let mut user_name_space = String::from("");
        if let Some(ctx) = context.downcast_ref::<RoutingContext>() {
            user_name_space = self.get_user_name_space(ctx);
        } else if let Some(s) = context.downcast_ref::<String>() {
            user_name_space = s.clone();
        }
        if !user_name_space.is_empty() {
            prefix = prefix + &std::path::MAIN_SEPARATOR.to_string() + &user_name_space;
        }
        prefix = prefix + &std::path::MAIN_SEPARATOR.to_string() + "webdav";
        let file = File::new(&prefix);
        if !file.exists() {
            file.mkdirs();
        }
        return prefix;
    }

    // fun getFileExt(url: String, defaultExt: String=""): String {
    //     return getFileExtetion(url, defaultExt)
    // }
    pub fn get_file_ext(&self, url: String, default_ext: String) -> String {
        return get_file_extetion(url, default_ext);
    }

    // suspend fun limitConcurrent(concurrentCount: Int, startIndex: Int, endIndex: Int, handler: suspend CoroutineScope.(Int) -> Any) {
    //     limitConcurrent(concurrentCount, startIndex, endIndex, handler) {_, _ ->
    //         true
    //     }
    // }
    pub fn limit_concurrent(&self, concurrent_count: i32, start_index: i32, end_index: i32, handler: fn(i32) -> Box<dyn std::any::Any>) {
        self.limit_concurrent_need_continue(concurrent_count, start_index, end_index, handler, |_, _| true);
    }

    // suspend fun limitConcurrent(concurrentCount: Int, startIndex: Int, endIndex: Int, handler: suspend CoroutineScope.(Int) -> Any, needContinue: (ArrayList<Any>, Int) -> Boolean) {
    //     var lastIndex = startIndex
    //     var loopCount = 0
    //     var resultCount = 0
    //     var loopStart = System.currentTimeMillis()
    //     var costTime = 0L
    //     var deferredList = arrayListOf<Deferred<Any>>()
    //     while(true) {
    //         var croutineCount = deferredList.size;
    //         if (croutineCount < concurrentCount) {
    //             for(i in lastIndex until endIndex) {
    //                 croutineCount += 1;
    //                 deferredList.add(async {
    //                     handler(i)
    //                 })
    //
    //                 lastIndex = i
    //                 if (croutineCount >= concurrentCount) {
    //                     break;
    //                 }
    //             }
    //         }
    //         var resultList = arrayListOf<Any>()
    //
    //         // 等待任何一个完成
    //         while (resultList.size <= 0) {
    //             delay(10)
    //             var stillDeferredList = arrayListOf<Deferred<Any>>()
    //             for (i in 0 until deferredList.size) {
    //                 try {
    //                     var deferred = deferredList.get(i)
    //                     if (deferred.isCompleted) {
    //                         resultCount++
    //                         resultList.add(deferred.getCompleted())
    //                     } else if (!deferred.isCancelled) {
    //                         stillDeferredList.add(deferred)
    //                     } else {
    //                         resultCount++
    //                     }
    //                 } catch(e: Exception) {
    //
    //                 }
    //             }
    //             deferredList.clear()
    //             deferredList.addAll(stillDeferredList)
    //         }
    //
    //         if (resultCount / concurrentCount > loopCount) {
    //             loopCount = resultCount / concurrentCount
    //             costTime = System.currentTimeMillis() - loopStart
    //             logger.info("Loop: {} concurrentCount: {} lastIndex: {} endIndex: {} costTime: {} ms deferredList size: {}", loopCount, croutineCount, lastIndex, endIndex, costTime, deferredList.size)
    //         }
    //
    //         if (lastIndex >= endIndex - 1) {
    //             // 搞完了，等待所有结束
    //             for (i in 0 until deferredList.size) {
    //                 try {
    //                     resultList.add(deferredList.get(i).await())
    //                 } catch(e: Exception) {
    //
    //                 }
    //             }
    //             deferredList.clear()
    //             needContinue(resultList, loopCount)
    //             break;
    //         }
    //         if (resultList.size > 0) {
    //             if (!needContinue(resultList, loopCount)) {
    //                 break;
    //             }
    //         }
    //         lastIndex = lastIndex + 1
    //     }
    //
    //     // for (i in 0 until concurrentCount) {
    //     //     runBlocking(concurrentCount, startIndex + i , endIndex, handler, needContinue)
    //     // }
    // }
    pub fn limit_concurrent_need_continue(&self, concurrent_count: i32, start_index: i32, end_index: i32, handler: fn(i32) -> Box<dyn std::any::Any>, need_continue: fn(Vec<Box<dyn std::any::Any>>, i32) -> bool) {
        let mut last_index = start_index;
        let mut loop_count = 0;
        let mut result_count = 0;
        let mut loop_start = System::current_time_millis();
        let mut cost_time = 0L;
        let mut deferred_list: Vec<Deferred> = Vec::new();
        loop {
            let mut croutine_count = deferred_list.len() as i32;
            if croutine_count < concurrent_count {
                let mut i = last_index;
                while i < end_index {
                    croutine_count += 1;
                    deferred_list.push(async {
                        handler(i)
                    });

                    last_index = i;
                    if croutine_count >= concurrent_count {
                        break;
                    }
                    i += 1;
                }
            }
            let mut result_list: Vec<Box<dyn std::any::Any>> = Vec::new();

            // 等待任何一个完成
            while result_list.len() <= 0 {
                delay(10);
                let mut still_deferred_list: Vec<Deferred> = Vec::new();
                for i in 0..deferred_list.len() {
                    let result = std::panic::catch_unwind(|| {
                        let deferred = &deferred_list[i];
                        if deferred.is_completed() {
                            result_count += 1;
                            result_list.push(deferred.get_completed());
                        } else if !deferred.is_cancelled() {
                            still_deferred_list.push(deferred.clone());
                        } else {
                            result_count += 1;
                        }
                    });
                    let _ = result;
                }
                deferred_list.clear();
                deferred_list.extend(still_deferred_list);
            }

            if result_count / concurrent_count > loop_count {
                loop_count = result_count / concurrent_count;
                cost_time = System::current_time_millis() - loop_start;
                logger.info("Loop: {} concurrentCount: {} lastIndex: {} endIndex: {} costTime: {} ms deferredList size: {}", loop_count, croutine_count, last_index, end_index, cost_time, deferred_list.len());
            }

            if last_index >= end_index - 1 {
                // 搞完了，等待所有结束
                for i in 0..deferred_list.len() {
                    let result = std::panic::catch_unwind(|| {
                        result_list.push(deferred_list[i].await());
                    });
                    let _ = result;
                }
                deferred_list.clear();
                need_continue(result_list, loop_count);
                break;
            }
            if result_list.len() > 0 {
                if !need_continue(result_list, loop_count) {
                    break;
                }
            }
            last_index = last_index + 1;
        }

        // for (i in 0 until concurrentCount) {
        //     runBlocking(concurrentCount, startIndex + i , endIndex, handler, needContinue)
        // }
    }
}
