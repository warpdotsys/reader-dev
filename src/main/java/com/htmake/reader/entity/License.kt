package com.htmake.reader.entity

import java.util.UUID

data class License(
    var host: String = "*",
    var userMaxLimit: Int = 15,
    var expiredAt: Long = 0,
    var openApi: Boolean = false,
    var simpleWebExpiredAt: Long = 1688140799000L,
    var instances: Int = 1,
    var type: String = "default",
    var id: String = UUID.randomUUID().toString(),
    var code: String = UUID.randomUUID().toString(),
    var verified: Boolean = false,
    var verifyTime: Long? = null
) {
    fun isValid(now: Long = System.currentTimeMillis()): Boolean = expiredAt == 0L || expiredAt >= now

    fun validHost(queryHost: String): Boolean {
        if (!isValid() || queryHost.isEmpty()) return false
        if (host == "*") return true
        val queryParts = queryHost.substringBefore(':').split('.')
        return host.split(',').any { configuredHost ->
            val parts = configuredHost.trim().split('.')
            parts.size == queryParts.size && parts.indices.all { parts[it] == "*" || parts[it] == queryParts[it] }
        }
    }
}
