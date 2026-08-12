use crate::prelude::*;
// package com.htmake.reader.utils

// import kotlinx.coroutines.sync.Mutex

/**
 * Singleton providing per-user mutex instances, backed by an LRU cache.
 */
// object UserMutex {
pub struct UserMutex;

// fix: 原 impl 内 static 移到模块级（Rust 不允许关联 static）
// val mutex = Mutex()
// val lockerMap = LRUCache<String, Mutex>(10)
pub static MUTEX: std::sync::OnceLock<Mutex> = std::sync::OnceLock::new();
// fix: OnceLock<&T> 不可变 → 内层 std::sync::Mutex 提供内部可变性
pub static LOCKER_MAP: std::sync::OnceLock<std::sync::Mutex<LRUCache<String, Mutex>>> = std::sync::OnceLock::new();

impl UserMutex {
    // suspend fun getLocker(username: String): Mutex {
    pub async fn get_locker(username: &str) -> Mutex {
        let mutex = MUTEX.get_or_init(|| Mutex::new());
        mutex.lock().await;
        // fix: try/finally → 闭包 + 手动 unlock（保持 finally 语义）
        let try_result: Result<Mutex, StubError> = (|| {
            let locker_map = LOCKER_MAP.get_or_init(|| std::sync::Mutex::new(LRUCache::new(10)));
            // fix: E0308 LRUCache::get 接收 &K（&String），此处 &str 需转成 &String
            let mut locker = locker_map.lock().unwrap().get(&username.to_string());
            if locker.is_none() {
                locker = Some(Mutex::new());
                locker_map.lock().unwrap().put(username.to_string(), locker.clone().unwrap());
            }
            Ok(locker.unwrap())
        })();
        mutex.unlock();
        if let Err(e) = &try_result {
            panic!("{}", e);
        }
        return try_result.unwrap();
    }
}
