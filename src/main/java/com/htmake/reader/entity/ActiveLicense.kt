package com.htmake.reader.entity

data class ActiveLicense(
    var host: String = "",
    var userMaxLimit: Int = 0,
    var expiredAt: Long = 0,
    var openApi: Boolean = false,
    var simpleWebExpiredAt: Long = 0,
    var id: String = "",
    var code: String = "",
    var verified: Boolean = false,
    var verifyTime: Long? = null,
    var instances: Int = 1,
    var type: String = "",
    var activeOrder: Int = 0,
    var activeTime: Long = 0,
    var activeIp: String = "",
    var activeEmail: String = "",
    var lastOnlineIp: String = "",
    var lastOnlineTime: Long? = null,
    var errorMsg: String = ""
)
