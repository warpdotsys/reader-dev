// package com.htmake.reader.utils

// import kotlinx.coroutines.sync.Mutex

/**
 * Singleton providing per-user mutex instances, backed by an LRU cache.
 */
// object UserMutex {
pub struct UserMutex;

impl UserMutex {
    // val mutex = Mutex()
    // val lockerMap = LRUCache<String, Mutex>(10)
    static MUTEX: std::sync::OnceLock<Mutex> = std::sync::OnceLock::new();
    static LOCKER_MAP: std::sync::OnceLock<LRUCache<String, Mutex>> = std::sync::OnceLock::new();

    // suspend fun getLocker(username: String): Mutex {
    pub async fn get_locker(username: &str) -> Mutex {
        let mutex = MUTEX.get_or_init(|| Mutex::new());
        mutex.lock().await;
        try {
            let mut locker_map = LOCKER_MAP.get_or_init(|| LRUCache::new(10));
            let mut locker = locker_map.get(username);
            if locker.is_none() {
                locker = Some(Mutex::new());
                locker_map.put(username.to_string(), locker.clone().unwrap());
            }
            return locker.unwrap();
        } finally {
            mutex.unlock();
        }
    }
}
