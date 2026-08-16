use crate::prelude::*;
// fix: E0659 歧义——prelude glob 同时导出 stubs 与 ResourceUtil 模块的 File、textutils 与 stubs 双份 TextUtils，显式导入覆盖
use crate::io_legado_app_utils_textutils::TextUtils;
use crate::stubs::File;
// fix: ACache 转录 API 返回 Arc<ACache> 且方法需 &mut self，CookieStore 方法仅 &self；
// 改用内部引擎 ACacheManager（公开构造/方法）+ Mutex 内可变性，逻辑与原 ACache.put/putWithTime/getAsString/remove/clear 一致
use crate::io_legado_app_utils_acache::{ACacheManager, Utils};
use std::sync::Mutex;
// @file:Suppress("unused")
//
// package io.legado.app.help.http
//
// import io.legado.app.utils.TextUtils
// import io.legado.app.data.entities.Cookie
// import io.legado.app.help.http.api.CookieManager
// import io.legado.app.utils.NetworkUtils
// import io.legado.app.adapters.ReaderAdapterHelper
// import io.legado.app.utils.ACache
// import java.io.File

// class CookieStore(val userNameSpace: String) : CookieManager {
pub struct CookieStore {
    pub user_name_space: String,
    pub cache_instance: Mutex<ACacheManager>,
}

// fix: SimpleBindings 绑定（AnyDebug 要求 Debug）
impl std::fmt::Debug for CookieStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CookieStore({})", self.user_name_space)
    }
}

impl CookieStore {
    pub fn new(user_name_space: String) -> CookieStore {
        CookieStore {
            user_name_space: user_name_space.clone(),
            // val cacheInstance = ACache.get(
            //     File(ReaderAdapterHelper.getAdapter().getWorkDir("storage", "cache", "cookie", userNameSpace)),
            //     50_000_000L,
            //     1_000_000
            // )
            cache_instance: Mutex::new(ACacheManager::new(
                File::new(&ReaderAdapterHelper::get_adapter().get_work_dir_vararg(
                    &["storage", "cache", "cookie", &user_name_space],
                )),
                50_000_000_i64,
                1_000_000,
            )),
        }
    }
}

impl CookieStore {
    // fun getKey(url: String, key: String): String = cookieToMap(getCookie(url))[key] ?: ""
    pub fn get_key(&self, url: &str, key: &str) -> String {
        self.cookie_to_map(&self.get_cookie(url)).get(key).cloned().unwrap_or("".to_string())
    }

    // fun clear() {
    //     cacheInstance.clear()
    // }
    fn clear(&self) {
        self.cache_instance.lock().unwrap().clear();
    }
}

impl CookieManager for CookieStore {
    // override fun setCookie(url: String, cookie: String?) {
    //     val domain = NetworkUtils.getSubDomain(url)
    //     if (domain.isNotEmpty()) cacheInstance.put(domain, cookie ?: "")
    // }
    fn set_cookie(&self, url: &str, cookie: Option<&str>) {
        let domain = NetworkUtils::getSubDomain(Some(url));
        if !domain.is_empty() {
            // cacheInstance.put(domain, cookie ?: "", 0)（putWithTime → Utils.newStringWithDateInfo + manager.put）
            let mut manager = self.cache_instance.lock().unwrap();
            let file = manager.newFile(&domain);
            file.writeText(&Utils::newStringWithDateInfo(0, cookie.unwrap_or("")));
            manager.put(&file);
        }
    }

    // override fun replaceCookie(url: String, cookie: String) {
    //     if (TextUtils.isEmpty(url) || TextUtils.isEmpty(cookie)) {
    //         return
    //     }
    //     val oldCookie = getCookie(url)
    //     if (TextUtils.isEmpty(oldCookie)) {
    //         setCookie(url, cookie)
    //     } else {
    //         val cookieMap = cookieToMap(oldCookie)
    //         cookieMap.putAll(cookieToMap(cookie))
    //         val newCookie = mapToCookie(cookieMap)
    //         setCookie(url, newCookie)
    //     }
    // }
    fn replace_cookie(&self, url: &str, cookie: &str) {
        if TextUtils::is_empty(Some(url)) || TextUtils::is_empty(Some(cookie)) {
            return;
        }
        let old_cookie = self.get_cookie(url);
        if TextUtils::is_empty(Some(old_cookie.as_str())) {
            self.set_cookie(url, Some(cookie));
        } else {
            let mut cookie_map = self.cookie_to_map(&old_cookie);
            cookie_map.extend(self.cookie_to_map(cookie));
            let new_cookie = self.map_to_cookie(Some(&cookie_map));
            match new_cookie {
                Some(new_cookie) => self.set_cookie(url, Some(&new_cookie)),
                None => self.set_cookie(url, None),
            }
        }
    }

    // override fun getCookie(url: String): String {
    //     val domain = NetworkUtils.getSubDomain(url)
    //     return if (domain.isEmpty()) "" else cacheInstance.getAsString(domain) ?: ""
    // }
    fn get_cookie(&self, url: &str) -> String {
        let domain = NetworkUtils::getSubDomain(Some(url));
        if domain.is_empty() {
            "".to_string()
        } else {
            // cacheInstance.getAsString(domain) ?: ""
            let mut manager = self.cache_instance.lock().unwrap();
            let file = manager.get(&domain);
            if !file.exists() {
                return "".to_string();
            }
            let text = file.readText();
            if Utils::isDue_str(&text) {
                manager.remove(&domain);
                "".to_string()
            } else {
                Utils::clearDateInfo(Some(&text)).unwrap_or_default()
            }
        }
    }

    // override fun removeCookie(url: String) {
    //     NetworkUtils.getSubDomain(url).takeIf { it.isNotEmpty() }?.let(cacheInstance::remove)
    // }
    fn remove_cookie(&self, url: &str) {
        let domain = NetworkUtils::getSubDomain(Some(url));
        if !domain.is_empty() {
            self.cache_instance.lock().unwrap().remove(&domain);
        }
    }

    // override fun cookieToMap(cookie: String): MutableMap<String, String> {
    //     val cookieMap = mutableMapOf<String, String>()
    //     if (cookie.isBlank()) {
    //         return cookieMap
    //     }
    //     val pairArray = cookie.split(";".toRegex()).dropLastWhile { it.isEmpty() }.toTypedArray()
    //     for (pair in pairArray) {
    //         val pairs = pair.split("=".toRegex()).dropLastWhile { it.isEmpty() }.toTypedArray()
    //         if (pairs.size == 1) {
    //             continue
    //         }
    //         val key = pairs[0].trim { it <= ' ' }
    //         val value = pairs[1]
    //         if (value.isNotBlank() || value.trim { it <= ' ' } == "null") {
    //             cookieMap[key] = value.trim { it <= ' ' }
    //         }
    //     }
    //     return cookieMap
    // }
    fn cookie_to_map(&self, cookie: &str) -> std::collections::HashMap<String, String> {
        let mut cookie_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        if cookie.trim().is_empty() {
            return cookie_map;
        }
        // val pairArray = cookie.split(";".toRegex()).dropLastWhile { it.isEmpty() }.toTypedArray()
        let pair_array: Vec<&str> = cookie.split(';').collect();
        let pair_array: Vec<&str> = {
            let mut v = pair_array;
            while let Some(true) = v.last().map(|it| it.is_empty()) {
                v.pop();
            }
            v
        };
        for pair in pair_array {
            // val pairs = pair.split("=".toRegex()).dropLastWhile { it.isEmpty() }.toTypedArray()
            let mut pairs: Vec<&str> = pair.split('=').collect();
            while let Some(true) = pairs.last().map(|it| it.is_empty()) {
                pairs.pop();
            }
            if pairs.len() == 1 {
                continue;
            }
            // val key = pairs[0].trim { it <= ' ' }
            let key = pairs[0].trim_matches(|it: char| it <= ' ');
            let value = pairs[1];
            // if (value.isNotBlank() || value.trim { it <= ' ' } == "null") {
            //     cookieMap[key] = value.trim { it <= ' ' }
            // }
            if !value.trim().is_empty() || value.trim_matches(|it: char| it <= ' ') == "null" {
                cookie_map.insert(key.to_string(), value.trim_matches(|it: char| it <= ' ').to_string());
            }
        }
        cookie_map
    }

    // override fun mapToCookie(cookieMap: Map<String, String>?): String? {
    //     if (cookieMap == None || cookieMap.isEmpty()) {
    //         return None
    //     }
    //     val builder = StringBuilder()
    //     for (key in cookieMap.keys) {
    //         val value = cookieMap[key]
    //         if (value?.isNotBlank() == true) {
    //             builder.append(key)
    //                 .append("=")
    //                 .append(value)
    //                 .append(";")
    //         }
    //     }
    //     return builder.deleteCharAt(builder.lastIndexOf(";")).toString()
    // }
    fn map_to_cookie(&self, cookie_map: Option<&std::collections::HashMap<String, String>>) -> Option<String> {
        let cookie_map = match cookie_map {
            Some(cookie_map) => cookie_map,
            None => return None,
        };
        if cookie_map.is_empty() {
            return None;
        }
        // val builder = StringBuilder()
        let mut builder = String::new();
        for key in cookie_map.keys() {
            let value = cookie_map.get(key);
            if let Some(value) = value {
                if !value.trim().is_empty() {
                    builder.push_str(key);
                    builder.push_str("=");
                    builder.push_str(value);
                    builder.push_str(";");
                }
            }
        }
        // return builder.deleteCharAt(builder.lastIndexOf(";")).toString()
        let last_index = builder.rfind(';').unwrap();
        builder.remove(last_index);
        Some(builder)
    }
}
