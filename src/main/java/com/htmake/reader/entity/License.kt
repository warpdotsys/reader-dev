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
    fun isValid(): Boolean {
        return expiredAt == 0L || expiredAt >= System.currentTimeMillis()
    }

    fun validHost(queryHost: String): Boolean {
        if (!isValid() || queryHost.isEmpty()) return false
        if (host == "*") return true
        val queryParts = queryHost.substringBefore(':').split('.')
        for (hostname in host.split(',')) {
            val parts = hostname.split('.')
            if (parts.size != queryParts.size) continue
            if (parts.indices.all { parts[it] == "*" || parts[it] == queryParts[it] }) {
                return true
            }
        }
        return false
    }

    fun toActiveLicense(): ActiveLicense {
        return ActiveLicense(
            host = host,
            userMaxLimit = userMaxLimit,
            expiredAt = expiredAt,
            openApi = openApi,
            simpleWebExpiredAt = simpleWebExpiredAt,
            id = id,
            code = code,
            verified = verified,
            verifyTime = verifyTime,
            instances = instances,
            type = type
        )
    }
}
