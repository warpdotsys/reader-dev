package com.htmake.reader.config

import org.springframework.boot.context.properties.ConfigurationProperties
import org.springframework.stereotype.Component

@Component
@ConfigurationProperties(prefix = "reader.app")
class AppConfig {
    var storagePath: String = "storage" // 存储路径
    var showUI = false // 是否显示UI
    var debug = false  // 是否调试web
    var packaged = false  // 是否打包为app
    var secure = false    // 是否启用登录鉴权
    var inviteCode = ""   // 注册邀请码
    var secureKey = ""    // 管理密码
    var cacheChapterContent = false // 是否缓存章节内容
    var userLimit = 15    // 用户上限
    var userBookLimit = 200    // 用户书籍上限
    var debugLog = false  // 调试日志
    var autoClearInactiveUser = 0  // 自动清理不活跃用户

    var exportUseReplace = false // 导出不使用净化
    var exportCharset = "UTF-8" // 导出字符集
    var exportNoChapterName = false // 不添加章节名
    var exportPictureFile = false // 导出图片

    // workDir - working directory (replaces storagePath)
    var workDir: String = ""

    // MongoDB configuration
    var mongoUri: String = ""
    var mongoDbName: String = "reader"

    // Shelf update interval (minutes)
    var shelfUpdateInteval: Int = 10
    // Remote webview API
    var remoteWebviewApi: String = ""

    // Default user permission settings
    var defaultUserEnableWebdav: Boolean = false
    var defaultUserEnableLocalStore: Boolean = false
    var defaultUserEnableBookSource: Boolean = true
    var defaultUserEnableRssSource: Boolean = true
    var defaultUserBookSourceLimit: Int = 200
    var defaultUserBookLimit: Int = 200

    // Auto backup user data
    var autoBackupUserData: Boolean = false

    // Minimum user password length
    var minUserPasswordLength: Int = 8

    // Remote book source update interval (minutes)
    var remoteBookSourceUpdateInterval: Int = 720
}
