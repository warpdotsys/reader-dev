package com.htmake.reader.api.controller

import io.vertx.ext.web.RoutingContext
import io.vertx.core.json.JsonObject
import io.vertx.core.json.JsonArray
import io.vertx.ext.web.client.WebClient
import mu.KotlinLogging
import com.htmake.reader.api.ReturnData
import com.htmake.reader.entity.License
import com.htmake.reader.entity.ActiveLicense
import com.htmake.reader.utils.getStorage
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.getWorkDir
import com.htmake.reader.utils.asJsonObject
import com.htmake.reader.utils.toDataClass
import com.htmake.reader.utils.toMap
import com.htmake.reader.utils.SpringContextUtils
import com.htmake.reader.utils.jsonEncode
import io.legado.app.utils.ACache
import java.io.File
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class LicenseController(coroutineContext: CoroutineContext): BaseController(coroutineContext) {

    private val webClient: WebClient by lazy {
        SpringContextUtils.getBean("webClient", WebClient::class.java)
    }

    private var privateKeyContent: String? = null

    private var tryCodeCache: ACache = ACache.get("tryCodeCache", 1000 * 1000L, 100)

    val backupFileNames: Array<String> by lazy {
        arrayOf("bookSource", "bookshelf", "bookmark", "replaceRule", "rssSource", "bookGroup", "httpTTS")
    }

    private fun loadActiveLicense(): ActiveLicense? {
        val json = getStorage("data", "license")
        if (json.isNullOrEmpty()) return null
        return try {
            val obj = asJsonObject(json)
            obj?.map?.toDataClass<ActiveLicense>()
        } catch (e: Exception) {
            null
        }
    }

    private fun saveActiveLicense(license: ActiveLicense) {
        saveStorage("data", "license", value = license.toMap())
    }

    /**
     * Check if the license is valid. Returns ReturnData with license info.
     * In community mode, always returns valid.
     */
    suspend fun isLicenseValid(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        return returnData.setData(mapOf(
            "valid" to true,
            "message" to "社区版,全部功能可用"
        ))
    }

    /**
     * Check license validity (non-route overload for scheduled jobs).
     * In community mode, this is a no-op.
     */
    suspend fun checkLicense(license: License) {
        // No-op in community mode - all features are always valid
        logger.debug("checkLicense called for license: {}", license.id)
    }

    fun checkLicenseFeature(feature: String): Boolean {
        // All features are enabled by default in dev version
        return true
    }

    suspend fun getLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val license = loadActiveLicense()
        if (license == null) {
            return returnData.setData(mapOf(
                "active" to false,
                "valid" to true,
                "message" to "社区版,全部功能可用"
            ))
        }
        return returnData.setData(mapOf(
            "active" to true,
            "valid" to true,
            "license" to license,
            "message" to "已激活"
        ))
    }

    suspend fun importLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val licenseKey = context.bodyAsJson?.getString("licenseKey", "") ?: ""
        if (licenseKey.isEmpty()) {
            return returnData.setErrorMsg("请输入许可证密钥")
        }

        // In dev version, accept any license key and activate
        val license = ActiveLicense(
            id = licenseKey,
            code = licenseKey,
            verified = true,
            verifyTime = System.currentTimeMillis(),
            activeTime = System.currentTimeMillis(),
            expiredAt = System.currentTimeMillis() + 365L * 86400 * 1000,
            userMaxLimit = 9999,
            openApi = true
        )
        saveActiveLicense(license)
        return returnData.setData(mapOf(
            "active" to true,
            "valid" to true,
            "license" to license,
            "message" to "导入成功"
        ))
    }

    suspend fun activateLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        // In dev version, activation always succeeds
        val license = loadActiveLicense()
        if (license == null) {
            return returnData.setErrorMsg("请先导入许可证")
        }
        license.verified = true
        license.verifyTime = System.currentTimeMillis()
        license.activeTime = System.currentTimeMillis()
        saveActiveLicense(license)
        return returnData.setData(mapOf(
            "active" to true,
            "valid" to true,
            "license" to license,
            "message" to "激活成功"
        ))
    }

    suspend fun checkLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        // License check is always valid in dev version
        return returnData.setData(mapOf(
            "valid" to true,
            "message" to "社区版,全部功能可用"
        ))
    }

    suspend fun supplyLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        // No-op in dev version
        return returnData.setData(mapOf(
            "valid" to true,
            "message" to "社区版,无需补充许可证"
        ))
    }

    suspend fun generateKeys(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        // Stub: key generation not applicable in community version
        return returnData.setErrorMsg("社区版不支持此操作")
    }

    suspend fun generateLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        // Stub: license generation not applicable in community version
        return returnData.setErrorMsg("社区版不支持此操作")
    }

    suspend fun isHostValid(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        // Always valid in dev version
        return returnData.setData(mapOf("valid" to true))
    }

    suspend fun decryptLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val license = loadActiveLicense()
        if (license == null) {
            return returnData.setErrorMsg("未找到许可证信息")
        }
        return returnData.setData(license)
    }

    suspend fun sendCodeToEmail(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        // Stub: email functionality not implemented in community version
        return returnData.setErrorMsg("社区版不支持此操作")
    }
}
