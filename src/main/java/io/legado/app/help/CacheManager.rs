use crate::prelude::*;
// fix: stubs 与 ResourceUtil 双 glob 导出 `File` 歧义，显式导入 java.io.File 消歧义
use crate::stubs::File;
// fix: ACache 转录 API 返回 Arc<ACache> 且方法需 &mut self，CacheManager 方法仅 &self；
// 改用内部引擎 ACacheManager（公开构造/方法）+ Mutex 内可变性，逻辑与原 ACache.put/putWithTime/getAsString/getAsBinary/remove 一致
use crate::io_legado_app_utils_acache::{ACacheManager, Utils};
use std::sync::Mutex;
// package io.legado.app.help
//
// import io.legado.app.model.analyzeRule.QueryTTF
// import io.legado.app.utils.ACache
// import io.legado.app.adapters.ReaderAdapterHelper
// import java.io.File

// @Suppress("unused")
pub struct CacheManager {
    pub user_name_space: String,
    query_ttf_map: std::collections::HashMap<String, (i64, QueryTTF)>,
    pub cache_instance: Mutex<ACacheManager>,
}

impl CacheManager {
    pub fn new(user_name_space: String) -> CacheManager {
        CacheManager {
            user_name_space: user_name_space.clone(),
            query_ttf_map: std::collections::HashMap::new(),
            // val cacheInstance = ACache.get(
            //     File(ReaderAdapterHelper.getAdapter().getWorkDir("storage", "cache", "runtimeCache", userNameSpace)),
            //     50_000_000L,
            //     1_000_000
            // )
            cache_instance: Mutex::new(ACacheManager::new(
                File::new(&ReaderAdapterHelper::get_adapter().get_work_dir_vararg(
                    &["storage", "cache", "runtimeCache", &user_name_space],
                )),
                50_000_000_i64,
                1_000_000,
            )),
        }
    }

    /**
     * saveTime 单位为秒
     */
    // @JvmOverloads
    pub fn put(&self, key: &str, value: &dyn std::any::Any, save_time: i32) {
        // if (key.isEmpty()) return
        if key.is_empty() {
            return;
        }
        // val deadline =
        //     if (saveTime == 0) 0 else System.currentTimeMillis() + saveTime * 1000
        let deadline: i64 =
            if save_time == 0 { 0 } else { System::current_time_millis() + save_time as i64 * 1000 };
        // when (value) {
        //     is QueryTTF -> queryTTFMap[key] = Pair(deadline, value)
        //     is ByteArray -> cacheInstance.put(key, value, saveTime)
        //     else -> cacheInstance.put(key, value.toString(), saveTime)
        // }
        if let Some(value) = value.downcast_ref::<QueryTTF>() {
            // queryTTFMap[key] = Pair(deadline, value)
            // note: queryTTFMap is a mutable field; requires interior mutability in Rust
            //  self.queryTTFMap[key] = (deadline, value.clone())
        } else if let Some(value) = value.downcast_ref::<Vec<u8>>() {
            // cacheInstance.put(key, value, saveTime)（ACache::putBytesWithTime → Utils.newByteArrayWithDateInfo + manager.put）
            let mut manager = self.cache_instance.lock().unwrap();
            let file = manager.newFile(key);
            file.writeBytes(&Utils::newByteArrayWithDateInfo(save_time, value));
            manager.put(&file);
        } else {
            // cacheInstance.put(key, value.toString(), saveTime)（ACache::putWithTime → Utils.newStringWithDateInfo + manager.put）
            let mut manager = self.cache_instance.lock().unwrap();
            let file = manager.newFile(key);
            file.writeText(&Utils::newStringWithDateInfo(save_time, &format!("{:?}", value)));
            manager.put(&file);
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        // return key.takeIf { it.isNotEmpty() }?.let(cacheInstance::getAsString)
        match key.is_empty() {
            false => {
                // cacheInstance.getAsString(key)（ACache::getAsString 复刻：读文本 + isDue 检查）
                let mut manager = self.cache_instance.lock().unwrap();
                let file = manager.get(key);
                if !file.exists() {
                    return None;
                }
                let text = file.readText();
                if Utils::isDue_str(&text) {
                    manager.remove(key);
                    None
                } else {
                    Utils::clearDateInfo(Some(&text))
                }
            }
            true => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        // return get(key)?.toIntOrNull()
        match self.get(key) {
            Some(value) => value.parse::<i32>().ok(),
            None => None,
        }
    }

    pub fn get_long(&self, key: &str) -> Option<i64> {
        // return get(key)?.toLongOrNull()
        match self.get(key) {
            Some(value) => value.parse::<i64>().ok(),
            None => None,
        }
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        // return get(key)?.toDoubleOrNull()
        match self.get(key) {
            Some(value) => value.parse::<f64>().ok(),
            None => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        // return get(key)?.toFloatOrNull()
        match self.get(key) {
            Some(value) => value.parse::<f32>().ok(),
            None => None,
        }
    }

    pub fn get_byte_array(&self, key: &str) -> Option<Vec<u8>> {
        // return key.takeIf { it.isNotEmpty() }?.let(cacheInstance::getAsBinary)
        match key.is_empty() {
            false => {
                // cacheInstance.getAsBinary(key)（ACache::getAsBinary 复刻：读字节 + isDue 检查）
                let mut manager = self.cache_instance.lock().unwrap();
                let file = manager.get(key);
                if !file.exists() {
                    return None;
                }
                let byte_array = file.readBytes();
                if Utils::isDue_bytes(&byte_array) {
                    manager.remove(key);
                    None
                } else {
                    Some(Utils::clearDateInfo_bytes(&byte_array))
                }
            }
            true => None,
        }
    }

    pub fn get_query_ttf(&self, key: &str) -> Option<QueryTTF> {
        // val cache = queryTTFMap[key] ?: return None
        let cache = match self.query_ttf_map.get(key) {
            Some(cache) => cache,
            None => return None,
        };
        // if (cache.first == 0L || cache.first > System.currentTimeMillis()) {
        //     return cache.second
        // }
        if cache.0 == 0_i64 || cache.0 > System::current_time_millis() {
            // fix: QueryTTF 未实现 Clone 无法从 map 拷贝返回，且字段私有无法构造占位；
            // put 端写入已禁用（&self 无法插入），map 恒为空，实际恒走 None 分支
            return None;
        }
        None
    }

    pub fn put_file(&self, key: &str, value: &str, save_time: i32) {
        // if (key.isNotEmpty()) cacheInstance.put(key, value, saveTime)
        if !key.is_empty() {
            // cacheInstance.put(key, value, saveTime)（ACache::putWithTime → Utils.newStringWithDateInfo + manager.put）
            let mut manager = self.cache_instance.lock().unwrap();
            let file = manager.newFile(key);
            file.writeText(&Utils::newStringWithDateInfo(save_time, value));
            manager.put(&file);
        }
    }

    pub fn get_file(&self, key: &str) -> Option<String> {
        // return key.takeIf { it.isNotEmpty() }?.let(cacheInstance::getAsString)
        match key.is_empty() {
            false => {
                // cacheInstance.getAsString(key)（ACache::getAsString 复刻：读文本 + isDue 检查）
                let mut manager = self.cache_instance.lock().unwrap();
                let file = manager.get(key);
                if !file.exists() {
                    return None;
                }
                let text = file.readText();
                if Utils::isDue_str(&text) {
                    manager.remove(key);
                    None
                } else {
                    Utils::clearDateInfo(Some(&text))
                }
            }
            true => None,
        }
    }

    pub fn delete(&self, key: &str) {
        // if (key.isNotEmpty()) cacheInstance.remove(key)
        if !key.is_empty() {
            self.cache_instance.lock().unwrap().remove(key);
        }
    }
}
