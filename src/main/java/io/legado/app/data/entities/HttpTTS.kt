package io.legado.app.data.entities

import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import io.legado.app.model.DebugLog

@JsonIgnoreProperties("headerMap", "source", "userNameSpace")
data class HttpTTS(
    val id: Long = System.currentTimeMillis(),
    var name: String = "",
    var url: String = "",
    var contentType: String? = null,
    override var concurrentRate: String? = "0",
    override var loginUrl: String? = null,
    override var loginUi: String? = null,
    override var header: String? = null,
    var jsLib: String? = null,
    override var enabledCookieJar: Boolean? = false,
    var loginCheckJs: String? = null,
    var lastUpdateTime: Long = System.currentTimeMillis()
) : BaseSource {

    @Transient
    private var _userNameSpace: String = ""

    @Transient
    private var debugLog: DebugLog? = null

    override fun getTag(): String {
        return name
    }

    override fun getKey(): String {
        return "httpTts:$id"
    }

    fun setUserNameSpace(value: String) {
        _userNameSpace = value
    }

    override fun getUserNameSpace(): String {
        return _userNameSpace
    }

    fun setLogger(value: DebugLog?) {
        debugLog = value
    }

    override fun getLogger(): DebugLog? {
        return debugLog
    }

    companion object
}
