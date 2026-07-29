package com.htmake.reader.entity

import java.util.UUID

data class ActiveLicense(
    var host: String = "*",
    var userMaxLimit: Int = 15,
    var expiredAt: Long = 0,
    var openApi: Boolean = false,
    var simpleWebExpiredAt: Long = 1682870399000L,
    var id: String = UUID.randomUUID().toString(),
    var code: String = UUID.randomUUID().toString(),
    var verified: Boolean = false,
    var verifyTime: Long? = null,
    var instances: Int = 1,
    var type: String = "default",
    var activeOrder: Int = 1,
    var activeTime: Long = System.currentTimeMillis(),
    var activeIp: String = "",
    var activeEmail: String = "",
    var lastOnlineIp: String = "",
    var lastOnlineTime: Long? = null,
    var errorMsg: String = ""
)
