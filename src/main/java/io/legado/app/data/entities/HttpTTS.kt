package io.legado.app.data.entities

import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import com.jayway.jsonpath.DocumentContext
import io.legado.app.model.DebugLog
import io.legado.app.utils.GSON
import io.legado.app.utils.jsonPath
import io.legado.app.utils.readLong
import io.legado.app.utils.readString

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

    companion object {
        fun fromJsonDoc(doc: DocumentContext): Result<HttpTTS> = runCatching {
            val loginUi = doc.read<Any>("$.loginUi")
            HttpTTS(
                id = doc.readLong("$.id") ?: System.currentTimeMillis(),
                name = doc.readString("$.name")!!,
                url = doc.readString("$.url")!!,
                contentType = doc.readString("$.contentType"),
                concurrentRate = doc.readString("$.concurrentRate"),
                loginUrl = doc.readString("$.loginUrl"),
                loginUi = if (loginUi is List<*>) GSON.toJson(loginUi) else loginUi?.toString(),
                header = doc.readString("$.header"),
                loginCheckJs = doc.readString("$.loginCheckJs")
            )
        }

        fun fromJson(json: String): Result<HttpTTS> = runCatching {
            fromJsonDoc(jsonPath.parse(json)).getOrThrow()
        }

        fun fromJsonArray(jsonArray: String): Result<List<HttpTTS>> = runCatching {
            val list = jsonPath.parse(jsonArray).read<Any>("$") as List<*>
            list.map { jsonItem ->
                val doc = jsonPath.parse(jsonItem)
                fromJsonDoc(doc).getOrThrow()
            }
        }
    }
}
