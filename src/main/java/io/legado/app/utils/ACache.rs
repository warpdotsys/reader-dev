use crate::prelude::*;
// fix: 显式导入以覆盖 prelude 中多个 glob 重导出导致的同名歧义
use crate::stubs::{
    ByteArrayInputStream, ByteArrayOutputStream, File, ObjectInputStream, ObjectOutputStream,
    Path, Serializable,
};
// fix: `Any` 需为 trait（stubs 枚举 Any 与其它 glob 冲突，显式 std::any::Any 消歧义）
use std::any::Any;
use std::cmp::min;
//Copyright (c) 2017. 章钦豪. All rights reserved.
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

pub fn current_time_millis() -> i64 {
    System::currentTimeMillis()
}

/**
 * 本地缓存
 */
#[allow(unused, dead_code)]
pub struct ACache {
    cacheDir: File,
    max_size: i64,
    max_count: i32,
    mCache: Option<ACacheManager>,
}

impl ACache {
    pub const TIME_HOUR: i32 = 60 * 60;
    pub const TIME_DAY: i32 = Self::TIME_HOUR * 24;
    const MAX_SIZE: i64 = 1000 * 1000 * 50; // 50 mb
    const MAX_COUNT: i32 = i32::MAX; // 不限制存放数据的数量
    fn mInstanceMap() -> &'static Mutex<HashMap<String, Arc<ACache>>> {
        static INSTANCE: OnceLock<Mutex<HashMap<String, Arc<ACache>>>> = OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn get() -> Arc<ACache> {
        let cache_name = "ACache";
        let f = File::new(&Path::join(AppCtx::cache_dir(), cache_name));
        Self::get_file(f, Self::MAX_SIZE as i64, Self::MAX_COUNT)
    }

    pub fn get_file(cache_dir: File, max_size: i64, max_count: i32) -> Arc<ACache> {
        let mut map = Self::mInstanceMap().lock().unwrap();
        let key = cache_dir.absoluteFile().to_string();
        if let Some(manager) = map.get(&key) {
            return manager.clone();
        }
        let manager = Arc::new(ACache::new(cache_dir.clone(), max_size, max_count));
        map.insert(cache_dir.absolutePath(), manager.clone());
        manager
    }

    fn new(cache_dir: File, max_size: i64, max_count: i32) -> ACache {
        let mut m_cache: Option<ACacheManager> = None;
        if !cache_dir.exists() && !cache_dir.mkdirs() {
            logger().info(format!("ACache can't make dirs in %s{}", cache_dir.absolutePath()));
        }
        m_cache = Some(ACacheManager::new(cache_dir.clone(), max_size, max_count));
        ACache { cacheDir: cache_dir, max_size, max_count, mCache: m_cache }
    }

    // =======================================
    // ============ String数据 读写 ==============
    // =======================================

    /**
     * 保存 String数据 到 缓存中
     *
     * @param key   保存的key
     * @param value 保存的String数据
     */
    pub fn put(&mut self, key: &str, value: &str) {
        if let Some(m_cache) = &mut self.mCache {
            let file = m_cache.newFile(key);
            file.writeText(value);
            m_cache.put(&file);
        }
    }

    /**
     * 保存 String数据 到 缓存中
     *
     * @param key      保存的key
     * @param value    保存的String数据
     * @param saveTime 保存的时间，单位：秒
     */
    pub fn put_with_time(&mut self, key: &str, value: &str, saveTime: i32) {
        self.put(key, &Utils::newStringWithDateInfo(saveTime, value));
    }

    /**
     * 读取 String数据
     *
     * @return String 数据
     */
    pub fn getAsString(&mut self, key: &str) -> Option<String> {
        if let Some(m_cache) = &mut self.mCache {
            let file = m_cache.get(key);
            if !file.exists() {
                return None;
            }
            let mut removeFile = false;
            let result: Result<Option<String>, std::io::Error> = (|| {
                let text = file.readText();
                if !Utils::isDue_str(&text) {
                    Ok(Utils::clearDateInfo(Some(&text)))
                } else {
                    removeFile = true;
                    Ok(None)
                }
            })();
            match result {
                Ok(Some(v)) => return Some(v),
                Ok(None) => {
                    if removeFile {
                        self.remove(key);
                    }
                    return None;
                }
                Err(e) => e.print_stack_trace(),
            }
        }
        None
    }

    pub fn getByHashCode(&mut self, hashCode: &str) -> Option<String> {
        if let Some(m_cache) = &mut self.mCache {
            let file = m_cache.newFileFromHashCode(hashCode);
            if !file.exists() {
                return None;
            }
            let mut removeFile = false;
            let result: Result<Option<String>, std::io::Error> = (|| {
                let text = file.readText();
                if !Utils::isDue_str(&text) {
                    Ok(Utils::clearDateInfo(Some(&text)))
                } else {
                    removeFile = true;
                    Ok(None)
                }
            })();
            match result {
                Ok(Some(v)) => return Some(v),
                Ok(None) => {
                    if removeFile {
                        file.delete();
                    }
                    return None;
                }
                Err(e) => e.print_stack_trace(),
            }
        }
        None
    }

    // =======================================
    // ========== JSONObject 数据 读写 =========
    // =======================================

    // /**
    //  * 保存 JSONObject数据 到 缓存中
    //  *
    //  * @param key   保存的key
    //  * @param value 保存的JSON数据
    //  */
    // fun put(key: String, value: JSONObject) {
    //     put(key, value.toString())
    // }

    // /**
    //  * 保存 JSONObject数据 到 缓存中
    //  *
    //  * @param key      保存的key
    //  * @param value    保存的JSONObject数据
    //  * @param saveTime 保存的时间，单位：秒
    //  */
    // fun put(key: String, value: JSONObject, saveTime: Int) {
    //     put(key, value.toString(), saveTime)
    // }

    // /**
    //  * 读取JSONObject数据
    //  *
    //  * @return JSONObject数据
    //  */
    // fun getAsJSONObject(key: String): JSONObject? {
    //     val json = getAsString(key) ?: return None
    //     return try {
    //         JSONObject(json)
    //     } catch (e: Exception) {
    //         None
    //     }
    // }

    // // =======================================
    // // ============ JSONArray 数据 读写 =============
    // // =======================================

    // /**
    //  * 保存 JSONArray数据 到 缓存中
    //  *
    //  * @param key   保存的key
    //  * @param value 保存的JSONArray数据
    //  */
    // fun put(key: String, value: JSONArray) {
    //     put(key, value.toString())
    // }

    // /**
    //  * 保存 JSONArray数据 到 缓存中
    //  *
    //  * @param key      保存的key
    //  * @param value    保存的JSONArray数据
    //  * @param saveTime 保存的时间，单位：秒
    //  */
    // fun put(key: String, value: JSONArray, saveTime: Int) {
    //     put(key, value.toString(), saveTime)
    // }

    // /**
    //  * 读取JSONArray数据
    //  *
    //  * @return JSONArray数据
    //  */
    // fun getAsJSONArray(key: String): JSONArray? {
    //     val json = getAsString(key)
    //     return try {
    //         JSONArray(json)
    //     } catch (e: Exception) {
    //         None
    //     }

    // }

    // =======================================
    // ============== byte 数据 读写 =============
    // =======================================

    /**
     * 保存 byte数据 到 缓存中
     *
     * @param key   保存的key
     * @param value 保存的数据
     */
    pub fn put_bytes(&mut self, key: &str, value: &[u8]) {
        if let Some(m_cache) = &mut self.mCache {
            let file = m_cache.newFile(key);
            file.writeBytes(value);
            m_cache.put(&file);
        }
    }

    /**
     * 保存 byte数据 到 缓存中
     *
     * @param key      保存的key
     * @param value    保存的数据
     * @param saveTime 保存的时间，单位：秒
     */
    pub fn put_bytes_with_time(&mut self, key: &str, value: &[u8], saveTime: i32) {
        self.put_bytes(key, &Utils::newByteArrayWithDateInfo(saveTime, value));
    }

    /**
     * 获取 byte 数据
     *
     * @return byte 数据
     */
    pub fn getAsBinary(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(m_cache) = &mut self.mCache {
            let mut removeFile = false;
            let result: Result<Option<Vec<u8>>, std::io::Error> = (|| {
                let file = m_cache.get(key);
                if !file.exists() {
                    return Ok(None);
                }
                let byteArray = file.readBytes();
                if !Utils::isDue_bytes(&byteArray) {
                    Ok(Some(Utils::clearDateInfo_bytes(&byteArray)))
                } else {
                    removeFile = true;
                    Ok(None)
                }
            })();
            match result {
                Ok(Some(v)) => return Some(v),
                Ok(None) => {
                    if removeFile {
                        self.remove(key);
                    }
                    return None;
                }
                Err(e) => e.print_stack_trace(),
            }
        }
        None
    }

    /**
     * 保存 Serializable数据到 缓存中
     *
     * @param key      保存的key
     * @param value    保存的value
     * @param saveTime 保存的时间，单位：秒
     */
    pub fn put_serializable(&mut self, key: &str, value: &dyn Serializable, saveTime: i32) {
        let byteArrayOutputStream = ByteArrayOutputStream::new();
        let mut oos = ObjectOutputStream::new(&byteArrayOutputStream);
        oos.writeObject(value);
        let data = byteArrayOutputStream.toByteArray();
        if saveTime != -1 {
            self.put_bytes_with_time(key, &data, saveTime);
        } else {
            self.put_bytes(key, &data);
        }
    }

    /**
     * 读取 Serializable数据
     *
     * @return Serializable 数据
     */
    pub fn getAsObject(&mut self, key: &str) -> Option<Box<dyn Any>> {
        if let Some(data) = self.getAsBinary(key) {
            let mut bis = ByteArrayInputStream::new(data);
            let mut ois = ObjectInputStream::new(&mut bis);
            let obj = ois.readObject();
            bis.close();
            ois.close();
            return Some(obj);
        }
        None
    }

    // =======================================
    // ============== bitmap 数据 读写 =============
    // =======================================

    // /**
    //  * 保存 bitmap 到 缓存中
    //  *
    //  * @param key   保存的key
    //  * @param value 保存的bitmap数据
    //  */
    // fun put(key: String, value: Bitmap) {
    //     put(key, Utils.bitmap2Bytes(value))
    // }

    // /**
    //  * 保存 bitmap 到 缓存中
    //  *
    //  * @param key      保存的key
    //  * @param value    保存的 bitmap 数据
    //  * @param saveTime 保存的时间，单位：秒
    //  */
    // fun put(key: String, value: Bitmap, saveTime: Int) {
    //     put(key, Utils.bitmap2Bytes(value), saveTime)
    // }

    /**
     * 读取 bitmap 数据
     *
     * @return bitmap 数据
     */
    // fun getAsBitmap(key: String): Bitmap? {
    //     return if (getAsBinary(key) == None) {
    //         None
    //     } else Utils.bytes2Bitmap(getAsBinary(key)!!)
    // }

    // =======================================
    // ============= drawable 数据 读写 =============
    // =======================================

    // /**
    //  * 保存 drawable 到 缓存中
    //  *
    //  * @param key   保存的key
    //  * @param value 保存的drawable数据
    //  */
    // fun put(key: String, value: Drawable) {
    //     put(key, Utils.drawable2Bitmap(value))
    // }

    // /**
    //  * 保存 drawable 到 缓存中
    //  *
    //  * @param key      保存的key
    //  * @param value    保存的 drawable 数据
    //  * @param saveTime 保存的时间，单位：秒
    //  */
    // fun put(key: String, value: Drawable, saveTime: Int) {
    //     put(key, Utils.drawable2Bitmap(value), saveTime)
    // }

    /**
     * 读取 Drawable 数据
     *
     * @return Drawable 数据
     */
    // fun getAsDrawable(key: String): Drawable? {
    //     return if (getAsBinary(key) == None) {
    //         None
    //     } else Utils.bitmap2Drawable(
    //         Utils.bytes2Bitmap(
    //             getAsBinary(key)!!
    //         )
    //     )
    // }

    /**
     * 获取缓存文件
     *
     * @return value 缓存的文件
     */
    pub fn file(&mut self, key: &str) -> Option<File> {
        if let Some(m_cache) = &mut self.mCache {
            let f = m_cache.newFile(key);
            if f.exists() {
                return Some(f);
            }
        }
        None
    }

    /**
     * 移除某个key
     *
     * @return 是否移除成功
     */
    pub fn remove(&mut self, key: &str) -> bool {
        match &mut self.mCache {
            Some(m_cache) => m_cache.remove(key),
            None => false,
        }
    }

    /**
     * 清除所有数据
     */
    pub fn clear(&mut self) {
        if let Some(m_cache) = &mut self.mCache {
            m_cache.clear();
        }
    }
}   // fix: 提前关闭 impl ACache —— Rust 不允许在 impl 内嵌套 mod，`mod Utils` 移到模块级

    /**
     * @author 杨福海（michael） www.yangfuhai.com
     * @version 1.0
     * title 时间计算工具类
     */
    pub mod Utils {
        use super::*;

        pub const mSeparator: char = ' ';

        /**
         * 判断缓存的String数据是否到期
         *
         * @return true：到期了 false：还没有到期
         */
        pub fn isDue(str: &str) -> bool {
            isDue_bytes(str.as_bytes())
        }

        // fix: 转录调用使用的别名（Kotlin `Utils.isDue(str)`）
        pub fn isDue_str(str: &str) -> bool {
            isDue_bytes(str.as_bytes())
        }

        /**
         * 判断缓存的byte数据是否到期
         *
         * @return true：到期了 false：还没有到期
         */
        pub fn isDue_bytes(data: &[u8]) -> bool {
            let result: Result<bool, String> = (|| {
                let text = getDateInfoFromDate(data);
                if let Some(text) = text {
                    if text.len() == 2 {
                        let mut saveTimeStr = text[0].clone();
                        while saveTimeStr.starts_with("0") {
                            saveTimeStr = saveTimeStr[1..].to_string();
                        }
                        let saveTime = java_long_valueOf(&saveTimeStr);
                        let deleteAfter = java_long_valueOf(&text[1]);
                        if System::currentTimeMillis() > saveTime + deleteAfter * 1000 {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            })();
            match result {
                Ok(b) => b,
                Err(e) => {
                    // fix: String 无 printStackTrace，eprintln 等价占位
                    eprintln!("{e}");
                    false
                }
            }
        }

        pub fn newStringWithDateInfo(second: i32, strInfo: &str) -> String {
            format!("{}{}", createDateInfo(second), strInfo)
        }

        pub fn newByteArrayWithDateInfo(second: i32, data2: &[u8]) -> Vec<u8> {
            let data1 = createDateInfo(second).as_bytes().to_vec();
            let mut retData = vec![0u8; data1.len() + data2.len()];
            System::arraycopy(&data1, 0, &mut retData, 0, data1.len());
            System::arraycopy(data2, 0, &mut retData, data1.len(), data2.len());
            retData
        }

        pub fn clearDateInfo(strInfo: Option<&str>) -> Option<String> {
            if let Some(s) = strInfo {
                if hasDateInfo(s.as_bytes()) {
                    return Some(s[s.index_of(&mSeparator.to_string(), 0) as usize + 1..].to_string());
                }
                return Some(s.to_string());
            }
            None
        }

        pub fn clearDateInfo_bytes(data: &[u8]) -> Vec<u8> {
            if hasDateInfo(data) {
                copyOfRange(data, indexOf(data, mSeparator) + 1, data.len())
            } else {
                data.to_vec()
            }
        }

        pub fn hasDateInfo(data: &[u8]) -> bool {
            data.len() > 15 && data[13] == '-' as u8 && indexOf(data, mSeparator) > 14
        }

        pub fn getDateInfoFromDate(data: &[u8]) -> Option<Vec<String>> {
            if hasDateInfo(data) {
                let saveDate = String::from_utf8_lossy(&copyOfRange(data, 0, 13)).to_string();
                let deleteAfter = String::from_utf8_lossy(&copyOfRange(data, 14, indexOf(data, mSeparator))).to_string();
                return Some(vec![saveDate, deleteAfter]);
            }
            None
        }

        #[allow(dead_code)]
        fn indexOf(data: &[u8], c: char) -> usize {
            for i in 0..data.len() {
                if data[i] == c as u8 {
                    return i;
                }
            }
            return usize::MAX;
        }

        fn copyOfRange(original: &[u8], from: usize, to: usize) -> Vec<u8> {
            let newLength = to - from;
            assert!(newLength >= 0, "{} > {}", from, to);
            let mut copy = vec![0u8; newLength];
            System::arraycopy(
                original, from, &mut copy, 0,
                min(original.len() - from, newLength)
            );
            copy
        }

        fn createDateInfo(second: i32) -> String {
            let mut currentTime = System::currentTimeMillis().to_string() + "";
            while currentTime.len() < 13 {
                currentTime.insert_str(0, "0");
            }
            format!("{currentTime}-{second}{mSeparator}")
        }
    }

/**
 * @author 杨福海（michael） www.yangfuhai.com
 * @version 1.0
 * title 缓存管理器
 */
pub struct ACacheManager {
    cacheDir: File,
    sizeLimit: i64,
    countLimit: i32,
    cacheSize: AtomicI64,
    cacheCount: AtomicI32,
    lastUsageDates: Arc<Mutex<HashMap<File, i64>>>,
}

impl ACacheManager {
    pub fn new(cacheDir: File, sizeLimit: i64, countLimit: i32) -> ACacheManager {
        let mut manager = ACacheManager {
            cacheDir,
            sizeLimit,
            countLimit,
            cacheSize: AtomicI64::new(0),
            cacheCount: AtomicI32::new(0),
            lastUsageDates: Arc::new(Mutex::new(HashMap::new())),
        };
        manager.calculateCacheSizeAndCacheCount();
        manager
    }

    /**
     * 计算 cacheSize和cacheCount
     */
    fn calculateCacheSizeAndCacheCount(&mut self) {
        let mut size = 0i64;
        let mut count = 0i32;
        let cachedFiles = self.cacheDir.listFiles();
        if cachedFiles != None {
            for cachedFile in cachedFiles.unwrap() {
                let last_modified = cachedFile.lastModified();
                size += self.calculateSize(&cachedFile);
                count += 1;
                self.lastUsageDates.lock().unwrap().insert(cachedFile, last_modified);
            }
            self.cacheSize.store(size, Ordering::Relaxed);
            self.cacheCount.store(count, Ordering::Relaxed);
        }
    }

    pub fn put(&mut self, file: &File) {
        let mut curCacheCount = self.cacheCount.load(Ordering::Relaxed);
        while curCacheCount + 1 > self.countLimit {
            let freedSize = self.removeNext();
            self.cacheSize.fetch_add(-freedSize, Ordering::Relaxed);
            curCacheCount = self.cacheCount.fetch_add(-1, Ordering::Relaxed);
        }
        self.cacheCount.fetch_add(1, Ordering::Relaxed);

        let valueSize = self.calculateSize(file);
        let mut curCacheSize = self.cacheSize.load(Ordering::Relaxed);
        while curCacheSize + valueSize > self.sizeLimit {
            let freedSize = self.removeNext();
            curCacheSize = self.cacheSize.fetch_add(-freedSize, Ordering::Relaxed);
        }
        self.cacheSize.fetch_add(valueSize, Ordering::Relaxed);

        let currentTime = System::currentTimeMillis();
        file.setLastModified(currentTime);
        self.lastUsageDates.lock().unwrap().insert(file.clone(), currentTime);
    }

    pub fn get(&mut self, key: &str) -> File {
        let file = self.newFile(key);
        let currentTime = System::currentTimeMillis();
        file.setLastModified(currentTime);
        self.lastUsageDates.lock().unwrap().insert(file.clone(), currentTime);
        file
    }

    pub fn newFile(&self, key: &str) -> File {
        File::new(&Path::join(self.cacheDir.absolutePath(), &key.hashCode().to_string()))
    }

    pub fn newFileFromHashCode(&self, hashCode: &str) -> File {
        File::new(&Path::join(self.cacheDir.absolutePath(), hashCode))
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let image = self.get(key);
        image.delete()
    }

    pub fn clear(&mut self) {
        self.lastUsageDates.lock().unwrap().clear();
        self.cacheSize.store(0, Ordering::Relaxed);
        let files = self.cacheDir.listFiles();
        if files != None {
            for f in files.unwrap() {
                f.delete();
            }
        }
    }

    /**
     * 移除旧的文件
     */
    fn removeNext(&mut self) -> i64 {
        let mut oldestUsage: Option<i64> = None;
        let mut mostLongUsedFile: Option<File> = None;
        {
            let map = self.lastUsageDates.lock().unwrap();
            if map.is_empty() {
                return 0;
            }
            for (key, lastValueUsage) in map.iter() {
                if mostLongUsedFile == None {
                    mostLongUsedFile = Some(key.clone());
                    oldestUsage = Some(*lastValueUsage);
                } else {
                    if *lastValueUsage < oldestUsage.unwrap() {
                        oldestUsage = Some(*lastValueUsage);
                        mostLongUsedFile = Some(key.clone());
                    }
                }
            }
        }

        let mut fileSize: i64 = 0;
        if mostLongUsedFile != None {
            let mostLongUsedFile = mostLongUsedFile.unwrap();
            fileSize = self.calculateSize(&mostLongUsedFile);
            if mostLongUsedFile.delete() {
                self.lastUsageDates.lock().unwrap().remove(&mostLongUsedFile);
            }
        }
        fileSize
    }

    fn calculateSize(&self, file: &File) -> i64 {
        file.length()
    }
}
