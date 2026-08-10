// package io.legado.app.data.entities

// import com.script.SimpleBindings
// import io.legado.app.utils.Base64
// import io.legado.app.constant.AppConst
// import io.legado.app.help.CacheManager
// import io.legado.app.help.JsExtensions
// import io.legado.app.help.http.CookieStore
// import io.legado.app.utils.*

/**
 * 可在js里调用,source.xxx()
 */
// @Suppress("unused")
pub trait BaseSource : JsExtensions {

    fn concurrent_rate(&self) -> Option<&str>; // 并发率
    fn set_concurrent_rate(&mut self, value: Option<String>);
    fn enabled_cookie_jar(&self) -> Option<bool>;
    fn set_enabled_cookie_jar(&mut self, value: Option<bool>);
    fn login_url(&self) -> Option<&str>;       // 登录地址
    fn set_login_url(&mut self, value: Option<String>);
    fn login_ui(&self) -> Option<&str>;       // 登录UI
    fn set_login_ui(&mut self, value: Option<String>);
    fn header(&self) -> Option<&str>;         // 请求头
    fn set_header(&mut self, value: Option<String>);

    fn get_tag(&self) -> String;

    fn get_key(&self) -> String;

    fn get_source(&self) -> Option<&dyn BaseSource> {
        Some(self)
    }

    fn get_login_js(&self) -> Option<String> {
        let login_js = self.login_url();
        if login_js.is_none() {
            None
        } else if login_js.unwrap().starts_with("@js:") {
            Some(login_js.unwrap()[4..].to_string())
        } else if login_js.unwrap().starts_with("<js>") {
            Some(login_js.unwrap()[4..login_js.unwrap().rfind("<").unwrap()].to_string())
        } else {
            Some(login_js.unwrap().to_string())
        }
    }

    fn login(&self) {
        if let Some(it) = self.get_login_js() {
            self.eval_js(it, None);
        }
    }

    /**
     * 解析header规则
     */
    // has_login_header: Boolean = false
    fn get_header_map(&self, has_login_header: bool) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(AppConst::UA_NAME.to_string(), AppConst::user_agent.to_string());
        if let Some(it) = self.header() {
            // it.startsWith("@js:", true) 忽略大小写
            let parsed = if starts_with_ignore_case(it, "@js:") {
                self.eval_js(it[4..].to_string(), None).map(|v| v.to_string())
            } else if starts_with_ignore_case(it, "<js>") {
                self.eval_js(it[4..it.rfind("<").unwrap()].to_string(), None).map(|v| v.to_string())
            } else {
                Some(it.to_string())
            };
            if let Some(json) = parsed {
                if let Some(map_from_json) = GSON::from_json_object::<HashMap<String, String>>(json).get_or_null() {
                    map.extend(map_from_json);
                }
            }
        }
        if has_login_header {
            if let Some(login_header_map) = self.get_login_header_map() {
                map.extend(login_header_map);
            }
        }
        map
    }

    /**
     * 获取用于登录的头部信息
     */
    fn get_login_header(&self) -> Option<String> {
        self.cache_manager().get(format!("loginHeader_{}", self.get_key()))
    }

    fn get_login_header_map(&self) -> Option<HashMap<String, String>> {
        let cache = self.get_login_header()?;
        GSON::from_json_object::<HashMap<String, String>>(cache).get_or_null()
    }

    /**
     * 保存登录头部信息,map格式,访问时自动添加
     */
    fn put_login_header(&self, header: String) {
        self.cache_manager().put(format!("loginHeader_{}", self.get_key()), header);
    }

    fn remove_login_header(&self) {
        self.cache_manager().delete(format!("loginHeader_{}", self.get_key()));
    }

    /**
     * 获取用户信息,可以用来登录
     * 用户信息采用aes加密存储
     */
    fn get_login_info(&self) -> Option<String> {
        let result = (|| -> Option<String> {
            let key = AppConst::user_agent.encode_to_byte_array(0, 8);
            let cache = self.cache_manager().get(format!("userInfo_{}", self.get_key()))?;
            let encode_bytes = EncoderUtils::base64_decode(cache, Base64::DEFAULT).to_byte_array();
            let decode_bytes = EncoderUtils::decrypt_aes(encode_bytes, key)?;
            Some(String::from_utf8(decode_bytes).ok()?)
        })();
        match result {
            Some(v) => Some(v),
            None => {
                // catch (e: Exception) { log("获取登陆信息出错 " + e.localizedMessage); return null }
                self.log("获取登陆信息出错");
                None
            }
        }
    }

    fn get_login_info_map(&self) -> Option<HashMap<String, String>> {
        GSON::from_json_object::<HashMap<String, String>>(self.get_login_info()).get_or_null()
    }

    /**
     * 保存用户信息,aes加密
     */
    fn put_login_info(&self, info: String) -> bool {
        let result = (|| -> Option<String> {
            let key = AppConst::user_agent.encode_to_byte_array(0, 8);
            let encode_bytes = EncoderUtils::encrypt_aes(info.to_byte_array(), key);
            Some(Base64::encode_to_string(encode_bytes, Base64::DEFAULT))
        })();
        match result {
            Some(encode_str) => {
                self.cache_manager().put(format!("userInfo_{}", self.get_key()), encode_str);
                true
            }
            None => {
                // catch (e: Exception) { log("保存登陆信息出错 " + e.localizedMessage); return false }
                self.log("保存登陆信息出错");
                false
            }
        }
    }

    fn remove_login_info(&self) {
        self.cache_manager().delete(format!("userInfo_{}", self.get_key()));
    }

    fn set_variable(&self, variable: Option<String>) {
        if let Some(v) = variable {
            self.cache_manager().put(format!("sourceVariable_{}", self.get_key()), v);
        } else {
            self.cache_manager().delete(format!("sourceVariable_{}", self.get_key()));
        }
    }

    fn get_variable(&self) -> Option<String> {
        self.cache_manager().get(format!("sourceVariable_{}", self.get_key()))
    }

    /**
     * 执行JS
     */
    // @Throws(Exception::class)
    // bindings_config: SimpleBindings.() -> Unit = {}
    fn eval_js(&self, js_str: String, bindings_config: Option<&mut dyn FnMut(&mut SimpleBindings)>) -> Option<Box<dyn Any>> {
        let mut bindings = SimpleBindings::new();
        if let Some(config) = bindings_config {
            config(&mut bindings);
        }
        bindings["java"] = self;
        bindings["source"] = self;
        bindings["baseUrl"] = self.get_key();
        bindings["cookie"] = self.cookie_store();
        bindings["cache"] = self.cache_manager();
        AppConst::SCRIPT_ENGINE.eval(js_str, bindings)
    }

    fn cache_manager(&self) -> CacheManager {
        CacheManager::new(self.get_user_name_space())
    }

    fn cookie_store(&self) -> CookieStore {
        CookieStore::new(self.get_user_name_space())
    }
}

// it.startsWith(prefix, ignoreCase = true) 的等价实现
fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.to_lowercase().starts_with(&prefix.to_lowercase())
}
