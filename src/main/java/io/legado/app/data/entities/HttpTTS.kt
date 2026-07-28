package io.legado.app.data.entities

import io.legado.app.model.DebugLog

data class HttpTTS(
    val id: Long = System.currentTimeMillis(),
    var name: String = "",
    var url: String = "",
    var contentType: String = "",
    override var concurrentRate: String? = null,
    override var loginUrl: String? = null,
    override var loginUi: String? = null,
    override var header: String? = null,
    var jsLib: String? = null,
    override var enabledCookieJar: Boolean? = false,
    var loginCheckJs: String? = null,
    var lastUpdateTime: Long = 0
) : BaseSource {

    @Transient
    private var _userNameSpace: String? = null

    @Transient
    private var debugLog: DebugLog? = null

    override fun getTag(): String {
        return name
    }

    override fun getKey(): String {
        return id.toString()
    }

    fun setUserNameSpace(value: String?) {
        _userNameSpace = value
    }

    override fun getUserNameSpace(): String {
        return _userNameSpace ?: ""
    }

    fun setLogger(value: DebugLog?) {
        debugLog = value
    }

    override fun getLogger(): DebugLog? {
        return debugLog
    }

    companion object
}
