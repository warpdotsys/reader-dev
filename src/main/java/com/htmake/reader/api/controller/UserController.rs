use crate::prelude::*;
// package com.htmake.reader.api.controller

// fix: 显式导入消除 stubs / CURD / ResourceUtil 等 glob 重导出歧义（显式导入优先于 glob）
use crate::stubs::{File, JsonArray, JsonObject};

// fix: stubs / VertExt 同名 get_work_dir 的 glob 歧义 → 本地包装，按 Kotlin vararg 参数转发
fn get_work_dir(base: &str, sub_dir_files: Vec<String>) -> String {
    crate::com_htmake_reader_utils_vertext::get_work_dir_multi(
        &std::iter::once(base)
            .chain(sub_dir_files.iter().map(|s| s.as_str()))
            .collect::<Vec<&str>>(),
    )
}

// fix: stubs / VertExt 同名 get_storage 的 glob 歧义 → 本地包装，按 Kotlin vararg 参数转发
fn get_storage(base: &str, names: Vec<String>) -> Option<String> {
    let mut full = vec![String::from(base)];
    full.extend(names);
    crate::com_htmake_reader_utils_vertext::get_storage(&full, ".json")
}

// fix: stubs / VertExt 同名 save_storage 的 glob 歧义 → 本地包装；value 改为借用，避免调用后仍需使用 map 时发生 move（E0382）
fn save_storage<T: std::fmt::Debug>(base: &str, names: Vec<String>, value: &T) {
    let mut full = vec![String::from(base)];
    full.extend(names);
    // fix: 占位——用户 map 无 Serialize 实现，暂按 Debug 文本写入存储
    crate::com_htmake_reader_utils_vertext::save_storage(&full, crate::stubs::Any::Str(format!("{:?}", value)), false, ".json");
}

// fix: ReturnData.set_data/set_error_msg 返回 &mut Self 而函数按值返回 → 消费式包装（保持原 Kotlin 链式 return 结构）
trait ReturnDataOwnedExt {
    pub fn set_error_msg_owned(self, error_msg: String) -> ReturnData;
    pub fn set_data_owned(self, data: Box<dyn std::any::Any>, msg: String) -> ReturnData;
}

impl ReturnDataOwnedExt for ReturnData {
    fn set_error_msg_owned(mut self, error_msg: String) -> ReturnData {
        self.set_error_msg(error_msg);
        self
    }
    fn set_data_owned(mut self, data: Box<dyn std::any::Any>, msg: String) -> ReturnData {
        self.set_data(data, msg);
        self
    }
}

// fix: 私有字段 BaseController.app_config/env/user_mutex 跨模块不可访问 → 经 Spring Bean 工厂等价获取（与原 Kotlin appConfig/env 属性一致）
fn app_config() -> AppConfig {
    SpringContextUtils::get_bean_app_config()
}

fn env() -> Environment {
    SpringContextUtils::get_bean_environment()
}

// fix: User 无 Serialize 实现，Kotlin `user.toMap()` 无法经 Gson 转录 → 手写字段映射（与原 User 字段集一致）
fn user_to_map(user: &User) -> std::collections::HashMap<String, Box<dyn std::any::Any>> {
    let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
    m.insert(String::from("username"), Box::new(user.username.clone()));
    m.insert(String::from("password"), Box::new(user.password.clone()));
    m.insert(String::from("salt"), Box::new(user.salt.clone()));
    m.insert(String::from("token"), Box::new(user.token.clone()));
    m.insert(String::from("last_login_at"), Box::new(user.last_login_at));
    m.insert(String::from("created_at"), Box::new(user.created_at));
    m.insert(String::from("enable_webdav"), Box::new(user.enable_webdav));
    m.insert(String::from("enable_local_store"), Box::new(user.enable_local_store));
    m.insert(String::from("enable_book_source"), Box::new(user.enable_book_source));
    m.insert(String::from("enable_rss_source"), Box::new(user.enable_rss_source));
    m.insert(String::from("book_source_limit"), Box::new(user.book_source_limit));
    m.insert(String::from("book_limit"), Box::new(user.book_limit));
    m
}

// private val logger = KotlinLogging.logger {}

// class UserController(coroutineContext: CoroutineContext): BaseController(coroutineContext) {
pub struct UserController {
    base: BaseController,
    // val userMaxCount = 15
    user_max_count: i32,
    // fix: BaseController.user_mutex 为私有字段跨模块不可访问 → 本地互斥量（保持互斥语义）
    user_mutex: Mutex,
}

impl UserController {
    pub fn new() -> UserController {
        UserController {
            base: BaseController::new(),
            user_max_count: 15,
            user_mutex: Mutex::new(),
        }
    }
    // private fun assetUserHome(userNameSpace: String): File? {
    //     val assetsRoot = File(getWorkDir("storage", "assets")).toPath().toAbsolutePath().normalize()
    //     val userHome = assetsRoot.resolve(userNameSpace).normalize()
    //     return userHome.takeIf { it.startsWith(assetsRoot) }?.toFile()
    // }
    pub fn asset_user_home(&self, user_name_space: &String) -> Option<File> {
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
    pub fn get_user_limit(&self, context: &RoutingContext) -> i32 {
        return app_config().user_limit.max(1);
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
    //     if (userMapJson != None) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     var existedUser = userMap.getOrDefault(username, None)
    //     if (existedUser == None) {
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
    //         if (userInfo == None) {
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
        let username = context.body_as_json().map(|j| j.get_string_default("username", "")).unwrap_or(String::from(""));
        let password = context.body_as_json().map(|j| j.get_string_default("password", "")).unwrap_or(String::from(""));
        let is_login = context.body_as_json().map(|j| j.get_boolean_default("isLogin", false)).unwrap_or(false);
        if username.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入用户名"));
        }
        if password.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));
        if let Some(json) = user_map_json {
            user_map = json.user_map_nested();
        }
        let existed_user = user_map.get(&username);
        if existed_user.is_none() {
            if is_login {
                // 登录返回用户不存在
                return return_data.set_error_msg_owned(String::from("用户不存在"));
            }
            if username.len() < 5 {
                return return_data.set_error_msg_owned(String::from("用户名不能低于5位"));
            }
            if password.len() < app_config().min_user_password_length as usize {
                return return_data.set_error_msg_owned(format!("密码不能低于{}位", app_config().min_user_password_length));
            }
            if username == "default" {
                return return_data.set_error_msg_owned(String::from("用户名不能为非法字符"));
            }
            let username_reg = Regex::new("(?i)^[a-z0-9]+$").unwrap();    //忽略大小写
            if !username_reg.is_match(&username) {
                return return_data.set_error_msg_owned(String::from("用户名只能由字母和数字组成"));
            }
            if !app_config().invite_code.is_empty() {
                // 需要填入邀请码才能注册
                let code = context.body_as_json().map(|j| j.get_string("code")).unwrap_or(String::from(""));
                if code.is_empty() {
                    return return_data.set_error_msg_owned(String::from("请输入邀请码"));
                }
                if app_config().invite_code != code {
                    return return_data.set_error_msg_owned(String::from("邀请码错误"));
                }
            }
            let user_limit = self.get_user_limit(context);
            if user_map.keys().len() >= user_limit as usize {
                return return_data.set_error_msg_owned(String::from("超过用户数上限"));
            }

            // 自动注册
            let salt = get_random_string(8);
            let password_encrypted = gen_encrypted_password(&password, &salt);
            let mut new_user = User::default();
            new_user.username = username;
            new_user.password = password_encrypted;
            new_user.salt = salt;
            new_user.enable_webdav = app_config().default_user_enable_webdav;
            new_user.enable_local_store = app_config().default_user_enable_local_store;
            new_user.enable_book_source = app_config().default_user_enable_book_source;
            new_user.enable_rss_source = app_config().default_user_enable_rss_source;
            new_user.book_source_limit = app_config().default_user_book_source_limit;
            new_user.book_limit = app_config().default_user_book_limit;

            let login_data = self.base.save_user_session(context, &mut new_user, true);
            return return_data.set_data_owned(Box::new(login_data), String::from(""));
        } else {
            let existed_user = existed_user.unwrap();
            if !is_login {
                // 注册时返回用户名已被占用
                return return_data.set_error_msg_owned(String::from("用户名已被占用"));
            }
            // 登录
            let user_info: Option<User> = existed_user.to_data_class();
            if user_info.is_none() {
                return return_data.set_error_msg_owned(String::from("用户信息错误"));
            }
            let mut user_info = user_info.unwrap();
            let password_encrypted = gen_encrypted_password(&password, &user_info.salt);
            if password_encrypted != user_info.password {
                return return_data.set_error_msg_owned(String::from("密码错误"));
            }
            let login_data = self.base.save_user_session(context, &mut user_info, true);
            return return_data.set_data_owned(Box::new(login_data), String::from(""));
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
    //                 if (userMapJson != None) {
    //                     userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //                 }
    //                 val currentUser = userMap.getOrDefault(username, None) ?: return@withLock false
    //                 val tokenMapVal = currentUser.getOrDefault("token_map", None)
    //                 val tokenMap = tokenMapVal as? MutableMap<String, Long>
    //                 if (tokenMap != None) {
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        let username = context.session().get("username").unwrap_or(String::from(""));
        context.session().destroy();

        // 清除自动登录token
        let mut access_token = context.query_param("accessToken").unwrap_or(String::from(""));
        if !access_token.is_empty() {
            let tmp: Vec<&str> = access_token.splitn(2, ':').collect();
            if tmp.len() >= 2 {
                access_token = tmp[1].to_string();
                let _guard = self.user_mutex.with_lock();
                let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
                let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));
                if let Some(json) = user_map_json {
                    user_map = json.user_map_nested();
                }
                let current_user = match user_map.get_mut(&username) {
                    Some(v) => v,
                    None => {
                        let _ = _guard;
                        let updated = false;
                        if !updated {
                            return return_data.set_error_msg_owned(String::from("系统错误"));
                        }
                        return return_data.set_error_msg_owned(String::from("请重新登录")).set_data_owned(Box::new(String::from("NEED_LOGIN")), String::new());
                    }
                };
                let token_map_val = current_user.get("token_map").and_then(|v| v.downcast_ref::<std::collections::HashMap<String, i64>>());
                let token_map = token_map_val.cloned();
                if let Some(mut token_map) = token_map {
                    token_map.remove(&access_token);
                    current_user.insert(String::from("token_map"), Box::new(token_map));
                }
                if current_user.get("token").and_then(|v| v.downcast_ref::<String>().cloned()).unwrap_or(String::from("")) == access_token {
                    current_user.insert(String::from("token"), Box::new(String::from("")));
                }
                // fix: 先结束 get_mut 借用再写回，避免 E0499
                let updated_user = std::mem::take(current_user);
                user_map.insert(username, updated_user);
                save_storage("data", vec![String::from("users")], &user_map);
                let _ = _guard;
            }
        }
        return return_data.set_error_msg_owned(String::from("请重新登录")).set_data_owned(Box::new(String::from("NEED_LOGIN")), String::new());
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
    //     if (userMapJson != None) {
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure || app_config().secure_key.is_empty() {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));
        if let Some(json) = user_map_json {
            user_map = json.user_map_nested();
        }
        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data_owned(Box::new(user_list), String::from(""));
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
    //     if (userMapJson != None) {
    //         userMap = userMapJson.map as MutableMap<String, Map<String, Any>>
    //     }
    //     var existedUser = userMap.getOrDefault(username, None)
    //     if (existedUser != None) {
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure || app_config().secure_key.is_empty() {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        let username = context.body_as_json().map(|j| j.get_string("username")).unwrap_or(String::from(""));
        let password = context.body_as_json().map(|j| j.get_string("password")).unwrap_or(String::from(""));
        if username.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入用户名"));
        }
        if password.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入密码"));
        }
        if username.len() < 5 {
            return return_data.set_error_msg_owned(String::from("用户名不能低于5位"));
        }
        if password.len() < 8 {
            return return_data.set_error_msg_owned(String::from("密码不能低于8位"));
        }
        if username == "default" {
            return return_data.set_error_msg_owned(String::from("用户名不能为非法字符"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let username_reg = Regex::new("(?i)^[a-z0-9]+$").unwrap();    //忽略大小写
        if !username_reg.is_match(&username) {
            return return_data.set_error_msg_owned(String::from("用户名只能由字母和数字组成"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));
        if let Some(json) = user_map_json {
            user_map = json.user_map_nested();
        }
        let existed_user = user_map.get(&username);
        if existed_user.is_some() {
            return return_data.set_error_msg_owned(String::from("用户已存在"));
        }

        let user_limit = self.get_user_limit(context);
        if user_map.keys().len() >= user_limit as usize {
            return return_data.set_error_msg_owned(String::from("超过用户数上限"));
        }

        // 自动注册
        let salt = get_random_string(8);
        let password_encrypted = gen_encrypted_password(&password, &salt);
        let mut new_user = User::default();
        new_user.username = username;
        new_user.password = password_encrypted;
        new_user.salt = salt;
        new_user.enable_webdav = context.body_as_json().and_then(|j| j.get_boolean("enableWebdav")).unwrap_or(app_config().default_user_enable_webdav);
        new_user.enable_local_store = context.body_as_json().and_then(|j| j.get_boolean("enableLocalStore")).unwrap_or(app_config().default_user_enable_local_store);
        new_user.enable_book_source = context.body_as_json().and_then(|j| j.get_boolean("enableBookSource")).unwrap_or(app_config().default_user_enable_book_source);
        new_user.enable_rss_source = context.body_as_json().and_then(|j| j.get_boolean("enableRssSource")).unwrap_or(app_config().default_user_enable_rss_source);
        new_user.book_source_limit = context.body_as_json().and_then(|j| j.get_integer_opt("bookSourceLimit")).unwrap_or(app_config().default_user_book_source_limit);
        new_user.book_limit = context.body_as_json().and_then(|j| j.get_integer_opt("bookLimit")).unwrap_or(app_config().default_user_book_limit);
        user_map.insert(new_user.username.clone(), user_to_map(&new_user));
        save_storage("data", vec![String::from("users")], &user_map);

        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data_owned(Box::new(user_list), String::from(""));
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
    //     if (userMapJson != None) {
    //         userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //     }
    //
    //     var existedUser = userMap.getOrDefault(username, None)
    //     if (existedUser == None) {
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure || app_config().secure_key.is_empty() {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        let username = context.body_as_json().map(|j| j.get_string("username")).unwrap_or(String::from(""));
        let password = context.body_as_json().map(|j| j.get_string("password")).unwrap_or(String::from(""));
        if username.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入用户名"));
        }
        if password.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入密码"));
        }
        if password.len() < app_config().min_user_password_length as usize {
            return return_data.set_error_msg_owned(format!("密码不能低于{}位", app_config().min_user_password_length));
        }
        if username == "default" {
            return return_data.set_error_msg_owned(String::from("用户不存在"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));
        if let Some(json) = user_map_json {
            user_map = json.user_map_nested();
        }

        let existed_user = user_map.get_mut(&username);
        if existed_user.is_none() {
            return return_data.set_error_msg_owned(String::from("用户不存在"));
        }
        let existed_user = existed_user.unwrap();

        let salt = get_random_string(8);
        let password_encrypted = gen_encrypted_password(&password, &salt);
        existed_user.insert(String::from("salt"), Box::new(salt));
        existed_user.insert(String::from("password"), Box::new(password_encrypted));
        // fix: 先结束 get_mut 借用再写回，避免 E0499
        let updated_user = std::mem::take(existed_user);
        user_map.insert(username, updated_user);
        save_storage("data", vec![String::from("users")], &user_map);

        return return_data.set_data_owned(Box::new(String::from("")), String::from(""));
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
    //     if (userMapJson != None) {
    //         val userJsonArray = context.bodyAsJsonArray
    //         for (i in 0 until userJsonArray.size()) {
    //             var username = userJsonArray.getString(i)
    //             if (username != None && userMapJson.containsKey(username)) {
    //                 // 删除用户信息
    //                 userMapJson.remove(username)
    //                 // 移除用户目录
    //                 var userHome = File(getWorkDir("storage", "data", username))
    //                 logger().info("delete userHome: {}", userHome)
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure || app_config().secure_key.is_empty() {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));

        if let Some(mut user_map_json) = user_map_json {
            let user_json_array = context.body_as_json_array().unwrap();
            for i in 0..user_json_array.size() {
                let username = user_json_array.get_string(i);
                if !username.is_empty() && user_map_json.map().contains_key(&username) {
                    // 删除用户信息
                    user_map_json.map_mut().remove(&username);
                    // 移除用户目录
                    let user_home = File::new(&get_work_dir("storage", vec![String::from("data"), username]));
                    logger().info(format!("delete userHome: {}", user_home.to_string()));
                    if user_home.exists() {
                        user_home.delete_recursively();
                    }
                }
            }
            user_map = user_map_json.user_map_nested();
            save_storage("data", vec![String::from("users")], &user_map);
        }

        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data_owned(Box::new(user_list), String::from(""));
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
    //     if (userMapJson != None) {
    //         userMap = userMapJson.map as MutableMap<String, MutableMap<String, Any>>
    //         var existedUser = userMap.getOrDefault(username, None)
    //         if (existedUser == None) {
    //             return returnData.setErrorMsg("用户不存在")
    //         }
    //         if (enableWebdav != None) {
    //             existedUser.put("enable_webdav", enableWebdav)
    //         }
    //         if (enableLocalStore != None) {
    //             existedUser.put("enable_local_store", enableLocalStore)
    //         }
    //         if (enableBookSource != None) {
    //             existedUser.put("enable_book_source", enableBookSource)
    //         }
    //         if (enableRssSource != None) {
    //             existedUser.put("enable_rss_source", enableRssSource)
    //         }
    //         if (bookSourceLimit != None) {
    //             existedUser.put("book_source_limit", bookSourceLimit)
    //         }
    //         if (bookLimit != None) {
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure || app_config().secure_key.is_empty() {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
        }
        let username = context.body_as_json().map(|j| j.get_string("username")).unwrap_or(String::from(""));
        let enable_webdav = context.body_as_json().and_then(|j| j.get_boolean("enableWebdav"));
        let enable_local_store = context.body_as_json().and_then(|j| j.get_boolean("enableLocalStore"));
        let enable_book_source = context.body_as_json().and_then(|j| j.get_boolean("enableBookSource"));
        let enable_rss_source = context.body_as_json().and_then(|j| j.get_boolean("enableRssSource"));
        let book_source_limit = context.body_as_json().and_then(|j| j.get_integer_opt("bookSourceLimit"));
        let book_limit = context.body_as_json().and_then(|j| j.get_integer_opt("bookLimit"));
        if username.is_empty() {
            return return_data.set_error_msg_owned(String::from("参数错误"));
        }

        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));

        if let Some(json) = user_map_json {
            user_map = json.user_map_nested();
            let existed_user = user_map.get_mut(&username);
            if existed_user.is_none() {
                return return_data.set_error_msg_owned(String::from("用户不存在"));
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
            // fix: 先结束 get_mut 借用再写回，避免 E0499
            let updated_user = std::mem::take(existed_user);
            user_map.insert(username, updated_user);
            save_storage("data", vec![String::from("users")], &user_map);
        }

        let mut user_list: Vec<std::collections::HashMap<String, Box<dyn std::any::Any>>> = Vec::new();
        for (_, value) in &user_map {
            user_list.push(self.base.format_user(value));
        }
        return return_data.set_data_owned(Box::new(user_list), String::from(""));
    }

    // suspend fun getUserInfo(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     checkAuth(context)
    //     var username = context.session().get("username") as String?
    //     var secure = env.getProperty("reader.app.secure", Boolean::class.java)
    //     var secureKey = env.getProperty("reader.app.secureKey")
    //
    //     var userInfo: Any? = None
    //     if (username != None) {
    //         var user = getUserInfoClass(username)
    //         if (user != None) {
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
        let secure = env().get_property_boolean("reader.app.secure");
        let secure_key = env().get_property("reader.app.secureKey");

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
                let mut m: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
                m.insert(String::from("name"), Box::new(it.name()));
                m.insert(String::from("size"), Box::new(it.length()));
                m
            })
            .collect();

        // fix: 原 Kotlin mapOf 异构值（Option / bool / Vec）→ Box<dyn Any> 统一装箱
        let mut result: std::collections::HashMap<String, Box<dyn std::any::Any>> = std::collections::HashMap::new();
        result.insert(String::from("userInfo"), Box::new(user_info));
        result.insert(String::from("secure"), Box::new(secure));
        result.insert(String::from("secureKey"), Box::new(secure_key.map(|s| !s.is_empty())));
        result.insert(String::from("fonts"), Box::new(fonts));
        return return_data.set_data_owned(Box::new(result), String::from(""));
    }

    // suspend fun saveUserConfig(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val content = context.bodyAsJson
    //     if (content == None) {
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let content = context.body_as_json();
        if content.is_none() {
            return return_data.set_error_msg_owned(String::from("参数错误"));
        }
        let mut content = content.unwrap();
        content.put("@updateTime", System::current_time_millis());

        let user_name_space = self.base.get_user_name_space(context);
        self.base.save_user_storage(&user_name_space, String::from("userConfig"), Box::new(content));
        return return_data.set_data_owned(Box::new(String::from("")), String::from(""));
    }

    // suspend fun getUserConfig(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     val userNameSpace = getUserNameSpace(context)
    //     val userConfig = asJsonObject(getUserStorage(userNameSpace, "userConfig"))
    //     if (userConfig == None) {
    //         return returnData.setErrorMsg("没有备份文件")
    //     }
    //     return returnData.setData(userConfig.map)
    // }
    pub fn get_user_config(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        let user_config = as_json_object(self.base.get_user_storage(&user_name_space, vec![String::from("userConfig")]).map(crate::stubs::Any::from_string));
        if user_config.is_none() {
            return return_data.set_error_msg_owned(String::from("没有备份文件"));
        }
        return return_data.set_data_owned(Box::new(user_config.unwrap().map()), String::from(""));
    }

    // suspend fun uploadFile(context: RoutingContext): ReturnData {
    //     val returnData = ReturnData()
    //     if (!checkAuth(context)) {
    //         return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
    //     }
    //     if (context.fileUploads() == None || context.fileUploads().isEmpty()) {
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
    //     // logger().info("type: {}", type)
    //     context.fileUploads().forEach {
    //         var file = File(it.uploadedFileName())
    //         logger().info("uploadFile: {} {} {}", it.uploadedFileName(), it.fileName(), file)
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
    //             logger().info("moveTo: {}", newFile)
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if context.file_uploads_opt().unwrap_or_default().is_empty() {
            return return_data.set_error_msg_owned(String::from("请上传文件"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        let mut file_list = JsonArray::new();
        let mut type_ = context.request().get_param("type");
        if type_.is_none() || type_.as_ref().unwrap().is_empty() {
            type_ = Some(String::from("images"));
        }
        let asset_type = type_.unwrap();
        if asset_type == "." || asset_type == ".." || asset_type.contains('/') || asset_type.contains('\\') {
            return return_data.set_error_msg_owned(String::from("文件类型错误"));
        }
        let asset_home = match self.asset_user_home(&user_name_space) {
            Some(v) => v,
            None => return return_data.set_error_msg_owned(String::from("文件路径错误")),
        };
        let type_home = asset_home.to_path().resolve(&asset_type).normalize();
        if !type_home.starts_with(&asset_home.to_path().to_absolute_path().normalize()) {
            return return_data.set_error_msg_owned(String::from("文件路径错误"));
        }
        // logger().info("type: {}", type)
        for upload in context.file_uploads_opt().unwrap_or_default() {
            let file = File::new(&upload.uploaded_file_name());
            logger().info(format!("uploadFile: {} {} {}", upload.uploaded_file_name(), upload.file_name(), file.to_string()));
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
                // fix: parent_file() 返回 Option<File>，unwrap 后保持原 Kotlin 逻辑
                if let Some(parent) = new_file.parent_file() {
                    if !parent.exists() {
                        parent.mkdirs();
                    }
                }
                if new_file.exists() {
                    new_file.delete();
                }
                logger().info(format!("moveTo: {}", new_file.to_string()));
                if file.copy_recursively(&new_file) {
                    file_list.add(format!("/assets/{}/{}/{}", user_name_space, asset_type, file_name));
                }
                file.delete_recursively();
            }
        }
        return return_data.set_data_owned(Box::new(file_list.get_list()), String::from(""));
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
    //     logger().info("delete file: {}", file)
    //     file.deleteRecursively()
    //     return returnData.setData("")
    // }
    pub fn delete_file(&self, context: &RoutingContext) -> ReturnData {
        let mut return_data = ReturnData::new();
        if !self.base.check_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        let url: String;
        if context.request().method() == HttpMethod::POST {
            // post 请求
            url = context.body_as_json().map(|j| j.get_string("url")).unwrap_or(String::from(""));
        } else {
            // get 请求
            url = context.query_param("url").unwrap_or(String::from(""));
        }
        if url.is_empty() {
            return return_data.set_error_msg_owned(String::from("请输入文件链接"));
        }
        let user_name_space = self.base.get_user_name_space(context);
        if !url.starts_with(&format!("/assets/{}/", user_name_space)) {
            return return_data.set_error_msg_owned(String::from("文件链接错误"));
        }
        let asset_home = match self.asset_user_home(&user_name_space) {
            Some(v) => v,
            None => return return_data.set_error_msg_owned(String::from("文件链接错误")),
        };
        let relative_path = url.trim_start_matches(&format!("/assets/{}/", user_name_space)).to_string();
        if relative_path.is_empty() {
            return return_data.set_error_msg_owned(String::from("文件链接错误"));
        }
        let file_path = asset_home.to_path().resolve(&relative_path.replace('\\', "/")).normalize();
        if !file_path.starts_with(&asset_home.to_path().to_absolute_path().normalize()) {
            return return_data.set_error_msg_owned(String::from("文件链接错误"));
        }
        let file = file_path.to_file();
        logger().info(format!("delete file: {}", file.to_string()));
        file.delete_recursively();
        return return_data.set_data_owned(Box::new(String::from("")), String::from(""));
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
    //     if (backupFile == None) {
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
            .put_header("Cache-Control", String::from("86400").as_str())
            .put_header("Content-Disposition", format!("attachment; filename={}", url_encode_charset(backup_file.name(), "UTF-8")).as_str())
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
            return return_data.set_data_owned(Box::new(String::from("NEED_LOGIN")), String::from("请登录后使用"));
        }
        if !app_config().secure || app_config().secure_key.is_empty() {
            return return_data.set_error_msg_owned(String::from("不支持的操作"));
        }
        if !self.base.check_manager_auth(context) {
            return return_data.set_data_owned(Box::new(String::from("NEED_SECURE_KEY")), String::from("请输入管理密码"));
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
        let expire_time = System::current_time_millis() - day as i64 * 86400 * 1000;
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
    //     if (userMapJson != None) {
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
        if !app_config().secure {
            return;
        }
        let mut user_map: std::collections::HashMap<String, std::collections::HashMap<String, Box<dyn std::any::Any>>> = std::collections::HashMap::new();
        let user_map_json: Option<JsonObject> = as_json_object(get_storage("data", vec![String::from("users")]).map(crate::stubs::Any::from_string));
        if let Some(json) = user_map_json {
            user_map = json.user_map_nested();
        }
        let mut has_changed = false;
        let usernames: Vec<String> = user_map.keys().cloned().collect();
        for username in usernames {
            let value = user_map.get(&username);
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
            save_storage("data", vec![String::from("users")], &user_map);
        }
    }
}
