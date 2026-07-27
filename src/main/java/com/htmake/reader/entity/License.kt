package com.htmake.reader.entity

data class License(
    var host: String = "",
    var userMaxLimit: Int = 0,
    var expiredAt: Long = 0,
    var openApi: Boolean = false,
    var simpleWebExpiredAt: Long = 0,
    var instances: Int = 1,
    var type: String = "",
    var id: String = "",
    var code: String = "",
    var verified: Boolean = false,
    var verifyTime: Long? = null
) {
    fun isValid(): Boolean {
        return expiredAt > System.currentTimeMillis()
    }

    fun validHost(currentHost: String): Boolean {
        if (host.isBlank()) {
            return true
        }
        return host == currentHost
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
