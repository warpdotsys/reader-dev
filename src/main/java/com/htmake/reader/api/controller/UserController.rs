// package com.htmake.reader.api.controller

// private val logger = KotlinLogging.logger {}

// class UserController(coroutineContext: CoroutineContext): BaseController(coroutineContext) {
pub struct UserController {
    base: BaseController,
    // val userMaxCount = 15
    user_max_count: i32,
}

impl UserController {
    // private fun assetUserHome(userNameSpace: String): File? {
    //     val assetsRoot = File(getWorkDir("storage", "assets")).toPath().toAbsolutePath().normalize()
    //     val userHome = assetsRoot.resolve(userNameSpace).normalize()
    //     return userHome.takeIf { it.startsWith(assetsRoot) }?.toFile()
    // }
    fn asset_user_home(&self, user_name_space: &String) -> Option<File> {
        let assets_root = File::new(&get_work_dir("storage", vec![String::from("assets")])).to_path().to_absolute_path().normalize();
        let user_home = assets_root.resolve(user_name_space).normalize();
        if user_home.starts_with(&assets_root) {
            return Some(user_home.to_file());
        }
        return None;
    }

    // private fun getUserLimit(context: RoutingContext): Int {
    //     return Math.max(appConfig.userLimit, 1)
    // }
    fn get_user_limit(&self, context: &RoutingContext) -> i32 {
        return self.base.app_config.user_limit.max(1);
    }

    // suspend fun login(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     val username = context.bodyAsJson.getString("username", "") ?: ""
    //     val password = context.bodyAsJson.getString("password", "") ?: ""
    //     val isLogin = context.bodyAsJson.getBoolean("isLogin", false) ?: false
    //     if (username.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入用户名")
    //     }
    //     if (password.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入密码")
    //     }
    //     var userMap = mutableMapOf<String, Map<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     var existedUser = userMap.getOrDefault(username, null)
    //     if (existedUser == null) {
    //         if (isLogin) {
    //             // 登录返回用户不存在
    //             return returnData.setErrorMsg("用户不存在")
    //         }
    //         if (username.length < 5) {
    //             return returnData.setErrorMsg("用户名不能低于5位")
    //         }
    //         if (password.length < appConfig.minUserPasswordLength) {
    //             return returnData.setErrorMsg("密码不能低于${appConfig.minUserPasswordLength}位")
    //         }
    //         if (username.equals("default")) {
    //             return returnData.setErrorMsg("用户名不能为非法字符")
    //         }
    //         val usernameReg = Regex("[a-z0-9]+", RegexOption.IGNORE_CASE)    //忽略大小写
    //         if (!usernameReg.matches(username)) {
    //             return returnData.setErrorMsg("用户名只能由字母和数字组成")
    //         }
    //         if (appConfig.inviteCode.isNotEmpty()) {
    //             // 需要填入邀请码才能注册
    //             val code = context.bodyAsJson.getString("code") ?: ""
    //             if (code.isNullOrEmpty()) {
    //                 return returnData.setErrorMsg("请输入邀请码")
    //             }
    //             if (!appConfig.inviteCode.equals(code)) {
    //                 return returnData.setErrorMsg("邀请码错误")
    //             }
    //         }
    //         val userLimit = getUserLimit(context)
    //         if (userMap.keys.size >= userLimit) {
    //             return returnData.setErrorMsg("超过用户数上限")
    //         }
    //
    //         // 自动注册
    //         var salt = getRandomString(8)
    //         var passwordEncrypted = genEncryptedPassword(password, salt)
    //         var newUser = User(username, passwordEncrypted, salt).apply {
    //             enable_webdav = appConfig.defaultUserEnableWebdav
    //             enable_local_store = appConfig.defaultUserEnableLocalStore
    //             enable_book_source = appConfig.defaultUserEnableBookSource
    //             enable_rss_source = appConfig.defaultUserEnableRssSource
    //             book_source_limit = appConfig.defaultUserBookSourceLimit
    //             book_limit = appConfig.defaultUserBookLimit
    //         }
    //
    //         val loginData = saveUserSession(context, newUser)
    //         return returnData.setData(loginData)
    //     } else {
    //         if (!isLogin) {
    //             // 注册时返回用户名已被占用
    //             return returnData.setErrorMsg("用户名已被占用")
    //         }
    //         // 登录
    //         var userInfo: User? = existedUser.toDataClass()
    //         if (userInfo == null) {
    //             return returnData.setErrorMsg("用户信息错误")
    //         }
    //         var passwordEncrypted = genEncryptedPassword(password, userInfo.salt)
    //         if (passwordEncrypted != userInfo.password) {
    //             return returnData.setErrorMsg("密码错误")
    //         }
    //         val loginData = saveUserSession(context, userInfo)
    //         return returnData.setData(loginData)
    //     }
    // }
    pub fn login(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        let username = context.body_as_json().get_string_default("username", "");
        let password = context.body_as_json().get_string_default("password", "");
        let is_login = context.body_as_json().get_boolean_default("isLogin", false);
        if username.is_empty() {
            return return_data.set_error_msg(String::from("请输入用户名"));
        }
        if password.is_empty() {
            return return_data.set_error_msg(String::from("请输入密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        let existed_user = user_map.get(&username).cloned();
        if existed_user.is_none() {
            if is_login {
                // 登录返回用户不存在
                return return_data.set_error_msg(String::from("用户不存在"));
            }
            if username.len() < 5 {
                return return_data.set_error_msg(String::from("用户名不能低于5位"));
            }
            if password.len() < self.base.app_config.min_user_password_length as usize {
                return return_data.set_error_msg(format!("密码不能低于{}位", self.base.app_config.min_user_password_length));
            }
            if username == "default" {
                return return_data.set_error_msg(String::from("用户名不能为非法字符"));
            }
            let username_reg = Regex::new("[a-z0-9]+", RegexOption::IGNORE_CASE);    //忽略大小写
            if !username_reg.matches(&username) {
                return return_data.set_error_msg(String::from("用户名只能由字母和数字组成"));
            }
            if !self.base.app_config.invite_code.is_empty() {
                // 需要填入邀请码才能注册
                let code = context.body_as_json().get_string("code");
                if code.is_empty() {
                    return return_data.set_error_msg(String::from("请输入邀请码"));
                }
                if self.base.app_config.invite_code != code {
                    return return_data.set_error_msg(String::from("邀请码错误"));
                }
            }
            let user_limit = self.get_user_limit(context);
            if user_map.keys().len() >= user_limit as usize {
                return return_data.set_error_msg(String::from("超过用户数上限"));
            }

            // 自动注册
            let salt = get_random_string(8);
            let password_encrypted = gen_encrypted_password(&password, &salt);
            let mut new_user = User::new(username, password_encrypted, salt);
            new_user.enable_webdav = self.base.app_config.default_user_enable_webdav;
            new_user.enable_local_store = self.base.app_config.default_user_enable_local_store;
            new_user.enable_book_source = self.base.app_config.default_user_enable_book_source;
            new_user.enable_rss_source = self.base.app_config.default_user_enable_rss_source;
            new_user.book_source_limit = self.base.app_config.default_user_book_source_limit;
            new_user.book_limit = self.base.app_config.default_user_book_limit;

            let login_data = self.base.save_user_session(context, &mut new_user, true);
            return return_data.set_data(Box::new(login_data), String::from(""));
        } else {
            let existed_user = existed_user.unwrap();
            if !is_login {
                // 注册时返回用户名已被占用
                return return_data.set_error_msg(String::from("用户名已被占用"));
            }
            // 登录
            let user_info: Option<User> = existed_user.to_data_class();
            if user_info.is_none() {
                return return_data.set_error_msg(String::from("用户信息错误"));
            }
            let mut user_info = user_info.unwrap();
            let password_encrypted = gen_encrypted_password(&password, &user_info.salt);
            if password_encrypted != user_info.password {
                return return_data.set_error_msg(String::from("密码错误"));
            }
            let login_data = self.base.save_user_session(context, &mut user_info, true);
            return return_data.set_data(Box::new(login_data), String::from(""));
        }
    }

    // suspend fun logout(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     var username = context.session().get("username") as String? ?: ""
    //     context.session().destroy()
    //
    //     // 清除自动登录token
    //     var accessToken = context.queryParam("accessToken").firstOrNull() ?: ""
    //     if (accessToken.isNotEmpty()) {
    //         var tmp = accessToken.split(":", limit=2)
    //         if (tmp.size >= 2) {
    //             accessToken = tmp[1]
    //             val updated = userMutex.withLock {
    //                 var userMap = mutableMapOf<String, MutableMap<String, Any>>()
    //                 var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //                 if (userMapJson != null) {
    //                     userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //                 }
    //                 val currentUser = userMap.getOrDefault(username, null) ?: return@withLock false
    //                 val tokenMapVal = currentUser.getOrDefault("token_map", null)
    //                 val tokenMap = tokenMapVal as? MutableMap<String, Long>
    //                 if (tokenMap != null) {
    //                     tokenMap.remove(accessToken)
    //                     currentUser.put("token_map", tokenMap)
    //                 }
    //                 if (currentUser.getOrDefault("token", "") == accessToken) {
    //                     currentUser.put("token", "")
    //                 }
    //                 userMap[username] = currentUser
    //                 saveStorage("data", "users", value = userMap)
    //                 true
    //             }
    //             if (!updated) {
    //                 return returnData.setErrorMsg("系统错误")
    //             }
    //         }
    //     }
    //     return returnData.setErrorMsg("请重新登录").setData("NEED_LOGIN")
    // }
    pub fn logout(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        let username = context.session().get("username").unwrap_or(String::from(""));
        context.session().destroy();

        // 清除自动登录token
        let mut access_token = context.query_param("accessToken").first().cloned().unwrap_or(String::from(""));
        if !access_token.is_empty() {
            let tmp: Vec<&str> = access_token.splitn(2, ':').collect();
            if tmp.len() >= 2 {
                access_token = tmp[1].to_string();
                let _guard = self.base.user_mutex.with_lock();
                let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
                let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
                if let Some(json) = user_map_json {
                    user_map = json.map().clone();
                }
                let current_user = match user_map.get_mut(&username) {
                    Some(v) => v,
                    None => {
                        let _ = _guard;
                        let updated = false;
                        if !updated {
                            return return_data.set_error_msg(String::from("系统错误"));
                        }
                        return return_data.set_error_msg(String::from("请重新登录")).set_data(Box::new(String::from("NEED_LOGIN")));
                    }
                };
                let token_map_val = current_user.get("token_map").cloned();
                let token_map = token_map_val.downcast::<std::collections::HashMap<String, i64>>().ok();
                if let Some(mut token_map) = token_map {
                    token_map.remove(&access_token);
                    current_user.insert(String::from("token_map"), Box::new(token_map));
                }
                if current_user.get("token").and_then(|v| v.downcast_ref::<String>().cloned()).unwrap_or(String::from("")) == access_token {
                    current_user.insert(String::from("token"), Box::new(String::from("")));
                }
                user_map.insert(username, current_user.clone());
                save_storage("data", vec![String::from("users")], user_map);
                let _ = _guard;
            }
        }
        return return_data.set_error_msg(String::from("请重新登录")).set_data(Box::new(String::from("NEED_LOGIN")));
    }

    // suspend fun getUserList(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure || appConfig.secureKey.isEmpty()) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     var userMap = mutableMapOf<String, MutableMap<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //     }
    //     var userList = arrayListOf<Map<String, Any>>()
    //     userMap.forEach{
    //         userList.add(formatUser(it.value))
    //     }
    //     return returnData.setData(userList)
    // }
    pub fn get_user_list(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure || self.base.app_config.secure_key.is_empty() {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data(Box::new(user_list), String::from(""));
    }

    // suspend fun addUser(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure || appConfig.secureKey.isEmpty()) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     val username = context.bodyAsJson.getString("username") ?: ""
    //     val password = context.bodyAsJson.getString("password") ?: ""
    //     if (username.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入用户名")
    //     }
    //     if (password.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入密码")
    //     }
    //     if (username.length < 5) {
    //         return returnData.setErrorMsg("用户名不能低于5位")
    //     }
    //     if (password.length < 8) {
    //         return returnData.setErrorMsg("密码不能低于8位")
    //     }
    //     if (username.equals("default")) {
    //         return returnData.setErrorMsg("用户名不能为非法字符")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     val usernameReg = Regex("[a-z0-9]+", RegexOption.IGNORE_CASE)    //忽略大小写
    //     if (!usernameReg.matches(username)) {
    //         return returnData.setErrorMsg("用户名只能由字母和数字组成")
    //     }
    //     var userMap = mutableMapOf<String, Map<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     var existedUser = userMap.getOrDefault(username, null)
    //     if (existedUser != null) {
    //         return returnData.setErrorMsg("用户已存在")
    //     }
    //
    //     val userLimit = getUserLimit(context)
    //     if (userMap.keys.size >= userLimit) {
    //         return returnData.setErrorMsg("超过用户数上限")
    //     }
    //
    //     // 自动注册
    //     var salt = getRandomString(8)
    //     var passwordEncrypted = genEncryptedPassword(password, salt)
    //     var newUser = User(username, passwordEncrypted, salt).apply {
    //         enable_webdav = context.bodyAsJson.getBoolean("enableWebdav") ?: appConfig.defaultUserEnableWebdav
    //         enable_local_store = context.bodyAsJson.getBoolean("enableLocalStore") ?: appConfig.defaultUserEnableLocalStore
    //         enable_book_source = context.bodyAsJson.getBoolean("enableBookSource") ?: appConfig.defaultUserEnableBookSource
    //         enable_rss_source = context.bodyAsJson.getBoolean("enableRssSource") ?: appConfig.defaultUserEnableRssSource
    //         book_source_limit = context.bodyAsJson.getInteger("bookSourceLimit") ?: appConfig.defaultUserBookSourceLimit
    //         book_limit = context.bodyAsJson.getInteger("bookLimit") ?: appConfig.defaultUserBookLimit
    //     }
    //     userMap.put(newUser.username, newUser.toMap())
    //     saveStorage("data", "users", value = userMap)
    //
    //     var userList = arrayListOf<Map<String, Any>>()
    //     userMap.forEach{
    //         userList.add(formatUser(it.value))
    //     }
    //     return returnData.setData(userList)
    // }
    pub fn add_user(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure || self.base.app_config.secure_key.is_empty() {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        let username = context.body_as_json().get_string("username");
        let password = context.body_as_json().get_string("password");
        if username.is_empty() {
            return return_data.set_error_msg(String::from("请输入用户名"));
        }
        if password.is_empty() {
            return return_data.set_error_msg(String::from("请输入密码"));
        }
        if username.len() < 5 {
            return return_data.set_error_msg(String::from("用户名不能低于5位"));
        }
        if password.len() < 8 {
            return return_data.set_error_msg(String::from("密码不能低于8位"));
        }
        if username == "default" {
            return return_data.set_error_msg(String::from("用户名不能为非法字符"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let username_reg = Regex::new("[a-z0-9]+", RegexOption::IGNORE_CASE);    //忽略大小写
        if !username_reg.matches(&username) {
            return return_data.set_error_msg(String::from("用户名只能由字母和数字组成"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        let existed_user = user_map.get(&username).cloned();
        if existed_user.is_some() {
            return return_data.set_error_msg(String::from("用户已存在"));
        }

        let user_limit = self.get_user_limit(context);
        if user_map.keys().len() >= user_limit as usize {
            return return_data.set_error_msg(String::from("超过用户数上限"));
        }

        // 自动注册
        let salt = get_random_string(8);
        let password_encrypted = gen_encrypted_password(&password, &salt);
        let mut new_user = User::new(username, password_encrypted, salt);
        new_user.enable_webdav = context.body_as_json().get_boolean("enableWebdav").unwrap_or(self.base.app_config.default_user_enable_webdav);
        new_user.enable_local_store = context.body_as_json().get_boolean("enableLocalStore").unwrap_or(self.base.app_config.default_user_enable_local_store);
        new_user.enable_book_source = context.body_as_json().get_boolean("enableBookSource").unwrap_or(self.base.app_config.default_user_enable_book_source);
        new_user.enable_rss_source = context.body_as_json().get_boolean("enableRssSource").unwrap_or(self.base.app_config.default_user_enable_rss_source);
        new_user.book_source_limit = context.body_as_json().get_integer("bookSourceLimit").unwrap_or(self.base.app_config.default_user_book_source_limit);
        new_user.book_limit = context.body_as_json().get_integer("bookLimit").unwrap_or(self.base.app_config.default_user_book_limit);
        user_map.insert(new_user.username.clone(), new_user.to_map());
        save_storage("data", vec![String::from("users")], user_map);

        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data(Box::new(user_list), String::from(""));
    }

    // suspend fun resetPassword(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure || appConfig.secureKey.isEmpty()) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     val username = context.bodyAsJson.getString("username") ?: ""
    //     val password = context.bodyAsJson.getString("password") ?: ""
    //     if (username.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入用户名")
    //     }
    //     if (password.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入密码")
    //     }
    //     if (password.length < appConfig.minUserPasswordLength) {
    //         return returnData.setErrorMsg("密码不能低于${appConfig.minUserPasswordLength}位")
    //     }
    //     if (username.equals("default")) {
    //         return returnData.setErrorMsg("用户不存在")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     var userMap = mutableMapOf<String, MutableMap<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //     }
    //
    //     var existedUser = userMap.getOrDefault(username, null)
    //     if (existedUser == null) {
    //         return returnData.setErrorMsg("用户不存在")
    //     }
    //
    //     var salt = getRandomString(8)
    //     var passwordEncrypted = genEncryptedPassword(password, salt)
    //     existedUser.put("salt", salt)
    //     existedUser.put("password", passwordEncrypted)
    //     userMap.put(username, existedUser)
    //     saveStorage("data", "users", value = userMap as MutableMap<String, Map<String, Any>>)
    //
    //     return returnData.setData("")
    // }
    pub fn reset_password(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure || self.base.app_config.secure_key.is_empty() {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        let username = context.body_as_json().get_string("username");
        let password = context.body_as_json().get_string("password");
        if username.is_empty() {
            return return_data.set_error_msg(String::from("请输入用户名"));
        }
        if password.is_empty() {
            return return_data.set_error_msg(String::from("请输入密码"));
        }
        if password.len() < self.base.app_config.min_user_password_length as usize {
            return return_data.set_error_msg(format!("密码不能低于{}位", self.base.app_config.min_user_password_length));
        }
        if username == "default" {
            return return_data.set_error_msg(String::from("用户不存在"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }

        let existed_user = user_map.get_mut(&username);
        if existed_user.is_none() {
            return return_data.set_error_msg(String::from("用户不存在"));
        }
        let existed_user = existed_user.unwrap();

        let salt = get_random_string(8);
        let password_encrypted = gen_encrypted_password(&password, &salt);
        existed_user.insert(String::from("salt"), Box::new(salt));
        existed_user.insert(String::from("password"), Box::new(password_encrypted));
        user_map.insert(username, existed_user.clone());
        save_storage("data", vec![String::from("users")], user_map);

        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun deleteUsers(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure || appConfig.secureKey.isEmpty()) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     var userMap = mutableMapOf<String, MutableMap<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //
    //     if (userMapJson != null) {
    //         val userJsonArray = context.bodyAsJsonArray
    //         for (i in 0 until userJsonArray.size()) {
    //             var username = userJsonArray.getString(i)
    //             if (username != null && userMapJson.containsKey(username)) {
    //                 // 删除用户信息
    //                 userMapJson.remove(username)
    //                 // 移除用户目录
    //                 var userHome = File(getWorkDir("storage", "data", username))
    //                 logger.info("delete userHome: {}", userHome)
    //                 if (userHome.exists()) {
    //                     userHome.deleteRecursively()
    //                 }
    //             }
    //         }
    //         userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //         saveStorage("data", "users", value = userMap)
    //     }
    //
    //     var userList = arrayListOf<Map<String, Any>>()
    //     userMap.forEach{
    //         userList.add(formatUser(it.value))
    //     }
    //     return returnData.setData(userList)
    // }
    pub fn delete_users(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure || self.base.app_config.secure_key.is_empty() {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));

        if let Some(mut user_map_json) = user_map_json {
            let user_json_array = context.body_as_json_array().unwrap();
            for i in 0..user_json_array.size() {
                let username = user_json_array.get_string(i);
                if !username.is_empty() && user_map_json.map().contains_key(&username) {
                    // 删除用户信息
                    user_map_json.map_mut().remove(&username);
                    // 移除用户目录
                    let user_home = File::new(&get_work_dir("storage", vec![String::from("data"), username]));
                    logger.info(format!("delete userHome: {}", user_home.to_string()));
                    if user_home.exists() {
                        user_home.delete_recursively();
                    }
                }
            }
            user_map = user_map_json.map().clone();
            save_storage("data", vec![String::from("users")], user_map);
        }

        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data(Box::new(user_list), String::from(""));
    }

    // suspend fun updateUser(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure || appConfig.secureKey.isEmpty()) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     val username = context.bodyAsJson.getString("username") ?: ""
    //     val enableWebdav = context.bodyAsJson.getBoolean("enableWebdav")
    //     val enableLocalStore = context.bodyAsJson.getBoolean("enableLocalStore")
    //     val enableBookSource = context.bodyAsJson.getBoolean("enableBookSource")
    //     val enableRssSource = context.bodyAsJson.getBoolean("enableRssSource")
    //     val bookSourceLimit = context.bodyAsJson.getInteger("bookSourceLimit")
    //     val bookLimit = context.bodyAsJson.getInteger("bookLimit")
    //     if (username.isEmpty()) {
    //         return returnData.setErrorMsg("参数错误")
    //     }
    //
    //     var userMap = mutableMapOf<String, MutableMap<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //         var existedUser = userMap.getOrDefault(username, null)
    //         if (existedUser == null) {
    //             return returnData.setErrorMsg("用户不存在")
    //         }
    //         if (enableWebdav != null) {
    //             existedUser.put("enable_webdav", enableWebdav)
    //         }
    //         if (enableLocalStore != null) {
    //             existedUser.put("enable_local_store", enableLocalStore)
    //         }
    //         if (enableBookSource != null) {
    //             existedUser.put("enable_book_source", enableBookSource)
    //         }
    //         if (enableRssSource != null) {
    //             existedUser.put("enable_rss_source", enableRssSource)
    //         }
    //         if (bookSourceLimit != null) {
    //             existedUser.put("book_source_limit", bookSourceLimit)
    //         }
    //         if (bookLimit != null) {
    //             existedUser.put("book_limit", bookLimit)
    //         }
    //         userMap.put(username, existedUser)
    //         saveStorage("data", "users", value = userMap)
    //     }
    //
    //     var userList = arrayListOf<Map<String, Any>>()
    //     userMap.forEach{
    //         userList.add(formatUser(it.value))
    //     }
    //     return returnData.setData(userList)
    // }
    pub fn update_user(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure || self.base.app_config.secure_key.is_empty() {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let username = context.body_as_json().get_string("username");
        let enable_webdav = context.body_as_json().get_boolean("enableWebdav");
        let enable_local_store = context.body_as_json().get_boolean("enableLocalStore");
        let enable_book_source = context.body_as_json().get_boolean("enableBookSource");
        let enable_rss_source = context.body_as_json().get_boolean("enableRssSource");
        let book_source_limit = context.body_as_json().get_integer("bookSourceLimit");
        let book_limit = context.body_as_json().get_integer("bookLimit");
        if username.is_empty() {
            return return_data.set_error_msg(String::from("参数错误"));
        }

        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));

        if let Some(json) = user_map_json {
            user_map = json.map().clone();
            let existed_user = user_map.get_mut(&username);
            if existed_user.is_none() {
                return return_data.set_error_msg(String::from("用户不存在"));
            }
            let existed_user = existed_user.unwrap();
            if let Some(v) = enable_webdav {
                existed_user.insert(String::from("enable_webdav"), Box::new(v));
            }
            if let Some(v) = enable_local_store {
                existed_user.insert(String::from("enable_local_store"), Box::new(v));
            }
            if let Some(v) = enable_book_source {
                existed_user.insert(String::from("enable_book_source"), Box::new(v));
            }
            if let Some(v) = enable_rss_source {
                existed_user.insert(String::from("enable_rss_source"), Box::new(v));
            }
            if let Some(v) = book_source_limit {
                existed_user.insert(String::from("book_source_limit"), Box::new(v));
            }
            if let Some(v) = book_limit {
                existed_user.insert(String::from("book_limit"), Box::new(v));
            }
            user_map.insert(username, existed_user.clone());
            save_storage("data", vec![String::from("users")], user_map);
        }

        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data(Box::new(user_list), String::from(""));
    }

    // suspend fun getUserInfo(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     checkAuth(context)
    //     var username = context.session().get("username") as String?
    //     var secure = env.getProperty("reader.app.secure", Boolean::class.java)
    //     var secureKey = env.getProperty("reader.app.secureKey")
    //
    //     var userInfo: Any? = null
    //     if (username != null) {
    //         var user = getUserInfoClass(username)
    //         if (user != null) {
    //             userInfo = formatUser(user)
    //         }
    //     }
    //     val fonts = listFilesRecursively(File(getWorkDir("storage", "assets", "fonts")))
    //         .filter { !it.name.startsWith(".") && it.isFile && getFileExt(it.name) == "ttf" }
    //         .map { mapOf("name" to it.name, "size" to it.length()) }
    //
    //     return returnData.setData(mapOf(
    //         "userInfo" to userInfo,
    //         "secure" to secure,
    //         "secureKey" to secureKey?.isNotEmpty(),
    //         "fonts" to fonts
    //     ))
    // }
    pub fn get_user_info(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        self.base.check_auth(context);
        let username = context.session().get("username");
        let secure = self.base.env.get_property_boolean("reader.app.secure");
        let secure_key = self.base.env.get_property("reader.app.secureKey");

        let mut user_info: Option<std::collections::HashMap<String, Box<dyn std::any::Any>>> = None;
        if let Some(username) = username {
            let user = self.base.get_user_info_class(username);
            if let Some(user) = user {
                user_info = Some(self.base.format_user(&user));
            }
        }
        let fonts: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = list_files_recursively(&File::new(&get_work_dir("storage", vec![String::from("assets"), String::from("fonts")])))
            .into_iter()
            .filter(|it| !it.name().starts_with(".") && it.is_file() && self.base.get_file_ext(it.name(), String::from("")) == "ttf")
            .map(|it| {
                let mut m = std::collections::HashMap::new();
                m.insert(String::from("name"), Box::new(it.name()));
                m.insert(String::from("size"), Box::new(it.length()));
                m
            })
            .collect();

        let mut result = std::collections::HashMap::new();
        result.insert(String::from("userInfo"), Box::new(user_info));
        result.insert(String::from("secure"), Box::new(secure));
        result.insert(String::from("secureKey"), Box::new(secure_key.map(|s| !s.is_empty())));
        result.insert(String::from("fonts"), Box::new(fonts));
        return return_data.set_data(Box::new(result), String::from(""));
    }

    // suspend fun saveUserConfig(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val content = context.bodyAsJson
    //     if (content == null) {
    //         return returnData.setErrorMsg("参数错误")
    //     }
    //     content.put("@updateTime", System.currentTimeMillis())
    //
    //     val userNameSpace = getUserNameSpace(context)
    //     saveUserStorage(userNameSpace, "userConfig", content)
    //     return returnData.setData("")
    // }
    pub fn save_user_config(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let content = context.body_as_json();
        if content.is_none() {
            return return_data.set_error_msg(String::from("参数错误"));
        }
        let mut content = content.unwrap();
        content.put("@updateTime", System::current_time_millis());

        let user_name_space = self.base.get_user_name_space(context);
        self.base.save_user_storage(&user_name_space, String::from("userConfig"), Box::new(content));
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun getUserConfig(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val userNameSpace = getUserNameSpace(context)
    //     val userConfig = asJsonObject(getUserStorage(userNameSpace, "userConfig"))
    //     if (userConfig == null) {
    //         return returnData.setErrorMsg("没有备份文件")
    //     }
    //     return returnData.setData(userConfig.map)
    // }
    pub fn get_user_config(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        let user_config = as_json_object(self.base.get_user_storage(&user_name_space, vec![String::from("userConfig")]));
        if user_config.is_none() {
            return return_data.set_error_msg(String::from("没有备份文件"));
        }
        return return_data.set_data(Box::new(user_config.unwrap().map()), String::from(""));
    }

    // suspend fun uploadFile(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (context.fileUploads() == null || context.fileUploads().isEmpty()) {
    //         return returnData.setErrorMsg("请上传文件")
    //     }
    //     var userNameSpace = getUserNameSpace(context)
    //     var fileList = JsonArray()
    //     var type = context.request().getParam("type")
    //     if (type.isNullOrEmpty()) {
    //         type = "images"
    //     }
    //     val assetType = type ?: "images"
    //     if (assetType == "." || assetType == ".." || assetType.contains('/') || assetType.contains('\\')) {
    //         return returnData.setErrorMsg("文件类型错误")
    //     }
    //     val assetHome = assetUserHome(userNameSpace) ?: return returnData.setErrorMsg("文件路径错误")
    //     val typeHome = assetHome.toPath().resolve(assetType).normalize()
    //     if (!typeHome.startsWith(assetHome.toPath().toAbsolutePath().normalize())) {
    //         return returnData.setErrorMsg("文件路径错误")
    //     }
    //     // logger.info("type: {}", type)
    //     context.fileUploads().forEach {
    //         var file = File(it.uploadedFileName())
    //         logger.info("uploadFile: {} {} {}", it.uploadedFileName(), it.fileName(), file)
    //         if (file.exists()) {
    //             var fileName = File(it.fileName().replace('\\', '/')).name
    //             if (fileName.isEmpty() || fileName == "." || fileName == "..") {
    //                 file.deleteRecursively()
    //                 return@forEach
    //             }
    //             var newFile = typeHome.resolve(fileName).normalize().toFile()
    //             if (!newFile.toPath().startsWith(typeHome)) {
    //                 file.deleteRecursively()
    //                 return@forEach
    //             }
    //             if (!newFile.parentFile.exists()) {
    //                 newFile.parentFile.mkdirs()
    //             }
    //             if (newFile.exists()) {
    //                 newFile.delete()
    //             }
    //             logger.info("moveTo: {}", newFile)
    //             if (file.copyRecursively(newFile)) {
    //                 fileList.add("/assets/" + userNameSpace + "/" + assetType + "/" + fileName)
    //             }
    //             file.deleteRecursively()
    //         }
    //     }
    //     return returnData.setData(fileList.getList())
    // }
    pub fn upload_file(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if context.file_uploads().is_none() || context.file_uploads().unwrap().is_empty() {
            return return_data.set_error_msg(String::from("请上传文件"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        let mut file_list = JsonArray::new();
        let mut type_ = context.request().get_param("type");
        if type_.is_none() || type_.as_ref().unwrap().is_empty() {
            type_ = Some(String::from("images"));
        }
        let asset_type = type_.unwrap();
        if asset_type == "." || asset_type == ".." || asset_type.contains('/') || asset_type.contains('\\') {
            return return_data.set_error_msg(String::from("文件类型错误"));
        }
        let asset_home = match self.asset_user_home(&user_name_space) {
            Some(v) => v,
            None => return return_data.set_error_msg(String::from("文件路径错误")),
        };
        let type_home = asset_home.to_path().resolve(&asset_type).normalize();
        if !type_home.starts_with(&asset_home.to_path().to_absolute_path().normalize()) {
            return return_data.set_error_msg(String::from("文件路径错误"));
        }
        // logger.info("type: {}", type)
        for upload in context.file_uploads().unwrap() {
            let file = File::new(&upload.uploaded_file_name());
            logger.info(format!("uploadFile: {} {} {}", upload.uploaded_file_name(), upload.file_name(), file.to_string()));
            if file.exists() {
                let mut file_name = File::new(&upload.file_name().replace('\\', "/")).name();
                if file_name.is_empty() || file_name == "." || file_name == ".." {
                    file.delete_recursively();
                    continue;
                }
                let new_file = type_home.resolve(&file_name).normalize().to_file();
                if !new_file.to_path().starts_with(&type_home) {
                    file.delete_recursively();
                    continue;
                }
                if !new_file.parent_file().exists() {
                    new_file.parent_file().mkdirs();
                }
                if new_file.exists() {
                    new_file.delete();
                }
                logger.info(format!("moveTo: {}", new_file.to_string()));
                if file.copy_recursively(&new_file) {
                    file_list.add(format!("/assets/{}/{}/{}", user_name_space, asset_type, file_name));
                }
                file.delete_recursively();
            }
        }
        return return_data.set_data(Box::new(file_list.get_list()), String::from(""));
    }

    // suspend fun deleteFile(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     var url: String
    //     if (context.request().method() == HttpMethod.POST) {
    //         // post 请求
    //         url = context.bodyAsJson.getString("url") ?: ""
    //     } else {
    //         // get 请求
    //         url = context.queryParam("url").firstOrNull() ?: ""
    //     }
    //     if (url.isNullOrEmpty()) {
    //         return returnData.setErrorMsg("请输入文件链接")
    //     }
    //     var userNameSpace = getUserNameSpace(context)
    //     if (!url.startsWith("/assets/" + userNameSpace + "/")) {
    //         return returnData.setErrorMsg("文件链接错误")
    //     }
    //     val assetHome = assetUserHome(userNameSpace) ?: return returnData.setErrorMsg("文件链接错误")
    //     val relativePath = url.removePrefix("/assets/" + userNameSpace + "/")
    //     if (relativePath.isEmpty()) {
    //         return returnData.setErrorMsg("文件链接错误")
    //     }
    //     val filePath = assetHome.toPath().resolve(relativePath.replace('\\', '/')).normalize()
    //     if (!filePath.startsWith(assetHome.toPath().toAbsolutePath().normalize())) {
    //         return returnData.setErrorMsg("文件链接错误")
    //     }
    //     var file = filePath.toFile()
    //     logger.info("delete file: {}", file)
    //     file.deleteRecursively()
    //     return returnData.setData("")
    // }
    pub fn delete_file(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            url = context.body_as_json().get_string("url");
        } else {
            // get 请求
            url = context.query_param("url").first().cloned().unwrap_or(String::from(""));
        }
        if url.is_empty() {
            return return_data.set_error_msg(String::from("请输入文件链接"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        if !url.starts_with(&format!("/assets/{}/", user_name_space)) {
            return return_data.set_error_msg(String::from("文件链接错误"));
        }
        let asset_home = match self.asset_user_home(&user_name_space) {
            Some(v) => v,
            None => return return_data.set_error_msg(String::from("文件链接错误")),
        };
        let relative_path = url.trim_start_matches(&format!("/assets/{}/", user_name_space)).to_string();
        if relative_path.is_empty() {
            return return_data.set_error_msg(String::from("文件链接错误"));
        }
        let file_path = asset_home.to_path().resolve(&relative_path.replace('\\', "/")).normalize();
        if !file_path.starts_with(&asset_home.to_path().to_absolute_path().normalize()) {
            return return_data.set_error_msg(String::from("文件链接错误"));
        }
        let file = file_path.to_file();
        logger.info(format!("delete file: {}", file.to_string()));
        file.delete_recursively();
        return return_data.set_data(Box::new(String::from("")), String::from(""));
    }

    // suspend fun downloadBackupFile(context: RoutingContext) {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         context.success(returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用"))
    //         return
    //     }
    //     val bookController = BookController(coroutineContext)
    //     val userNameSpace = getUserNameSpace(context)
    //     val latestZipFilePath = bookController.getLastBackFileFromWebdav(userNameSpace)
    //     val backupDir = getWorkDir("storage", "data", userNameSpace, "backup")
    //     val backupFile = bookController.createUserBackup(userNameSpace, backupDir, latestZipFilePath)
    //     if (backupFile == null) {
    //         context.success(returnData.setErrorMsg("备份失败"))
    //         return
    //     }
    //     context.response()
    //         .putHeader("Cache-Control", "86400")
    //         .putHeader("Content-Disposition", "attachment; filename=${URLEncoder.encode(backupFile.name, "UTF-8")}")
    //         .sendFile(backupFile.toString())
    // }
    pub fn download_backup_file(&self, context: &RoutingContext) {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            context.success(return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用")));
            return;
        }
        let book_controller = BookController::new();
        let user_name_space = self.base.get_user_name_space(context);
        let latest_zip_file_path = book_controller.get_last_back_file_from_webdav(&user_name_space);
        let backup_dir = get_work_dir("storage", vec![String::from("data"), user_name_space.clone(), String::from("backup")]);
        let backup_file = book_controller.create_user_backup(&user_name_space, backup_dir, latest_zip_file_path);
        if backup_file.is_none() {
            context.success(return_data.set_error_msg(String::from("备份失败")));
            return;
        }
        let backup_file = backup_file.unwrap();
        context.response()
            .put_header("Cache-Control", String::from("86400"))
            .put_header("Content-Disposition", format!("attachment; filename={}", url_encode(&backup_file.name(), "UTF-8")))
            .send_file(backup_file.to_string());
    }

    // suspend fun clearInactiveUsers(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (!appConfig.secure || appConfig.secureKey.isEmpty()) {
    //         return returnData.setErrorMsg("不支持的操作")
    //     }
    //     if (!checkManagerAuth(context)) {
    //         return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
    //     }
    //     val inactiveDay = context.bodyAsJson?.getInteger("inactiveDay", 0) ?: 0
    //     clearInactiveUsers(inactiveDay)
    //     return getUserList(context)
    // }
    pub fn clear_inactive_users_ctx(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !self.base.app_config.secure || self.base.app_config.secure_key.is_empty() {
            return return_data.set_error_msg(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let inactive_day = context.body_as_json().map(|j| j.get_integer("inactiveDay", 0)).unwrap_or(0);
        self.clear_inactive_users(inactive_day);
        return self.get_user_list(context);
    }

    // suspend fun clearInactiveUsers(day: Int) {
    //     val expireTime = System.currentTimeMillis() - day * 86400L * 1000L
    //     forEachUser { user ->
    //         if (user.last_login_at >= expireTime) {
    //             false
    //         } else {
    //             File(getWorkDir("storage", "data", user.username)).deleteRecursively()
    //             true
    //         }
    //     }
    // }
    pub fn clear_inactive_users(&self, day: i32) {
        let expire_time = System::current_time_millis() - day as i64 * 86400L * 1000L;
        self.for_each_user(&mut |user: &mut User| {
            if user.last_login_at >= expire_time {
                false
            } else {
                File::new(&get_work_dir("storage", vec![String::from("data"), user.username.clone()])).delete_recursively();
                true
            }
        });
    }

    // suspend fun forEachUser(handler: suspend CoroutineScope.(User) -> Boolean) {
    //     if (!appConfig.secure) return
    //     var userMap = mutableMapOf<String, Map<String, Any>>()
    //     var userMapJson: JsonObject? = asJsonObject(getStorage("data", "users"))
    //     if (userMapJson != null) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     kotlinx.coroutines.coroutineScope {
    //         var hasChanged = false
    //         val iterator = userMap.entries.iterator()
    //         while (iterator.hasNext()) {
    //             val (_, value) = iterator.next()
    //             val username = value["username"] as? String ?: ""
    //             if (username.isEmpty()) continue
    //             val user: User = userMap[username]?.toDataClass() ?: continue
    //             if (handler(user)) {
    //                 hasChanged = true
    //                 iterator.remove()
    //             }
    //         }
    //         if (hasChanged) saveStorage("data", "users", value = userMap)
    //     }
    // }
    pub fn for_each_user(&self, handler: &mut dyn FnMut(&mut User) -> bool) {
        if !self.base.app_config.secure {
            return;
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]));
        if let Some(json) = user_map_json {
            user_map = json.map().clone();
        }
        let mut has_changed = false;
        let usernames: Vec<String> = user_map.keys().cloned().collect();
        for username in usernames {
            let value = user_map.get(&username).cloned();
            let u_name = value.as_ref().and_then(|v| v.get("username")).and_then(|v| v.downcast_ref::<String>().cloned()).unwrap_or(String::from(""));
            if u_name.is_empty() {
                continue;
            }
            let user: Option<User> = user_map.get(&u_name).and_then(|v| v.to_data_class());
            let mut user = match user {
                Some(u) => u,
                None => continue,
            };
            if handler(&mut user) {
                has_changed = true;
                user_map.remove(&username);
            }
        }
        if has_changed {
            save_storage("data", vec![String::from("users")], user_map);
        }
    }
}
