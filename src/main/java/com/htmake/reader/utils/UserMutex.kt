package com.htmake.reader.utils

import kotlinx.coroutines.sync.Mutex

/**
 * Singleton providing per-user mutex instances, backed by an LRU cache.
 */
object UserMutex {

    val mutex = Mutex()
    val lockerMap = LRUCache<String, Mutex>(10)

    suspend fun getLocker(username: String): Mutex {
        mutex.lock()
        try {
            var locker = lockerMap.get(username)
            if (locker == null) {
                locker = Mutex()
                lockerMap.put(username, locker)
            }
            return locker
        } finally {
            mutex.unlock()
        }
    }
}
