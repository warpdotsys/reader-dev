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
import com.htmake.reader.utils.decryptToLicense
import com.htmake.reader.utils.getInstalledLicense
import com.htmake.reader.utils.setLicenseValid
import com.htmake.reader.utils.validateEmail
import com.htmake.reader.utils.sendEmail
import io.legado.app.utils.EncoderUtils
import java.security.KeyFactory
import java.security.PrivateKey
import java.security.spec.PKCS8EncodedKeySpec
import java.util.Base64
import java.util.UUID
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

    private fun privateKey(): PrivateKey? {
        if (privateKeyContent.isNullOrEmpty()) {
            val file = com.htmake.reader.utils.getStorageFile("data", "privateKey", ext = ".key")
            privateKeyContent = file.takeIf { it.exists() }?.readText()
        }
        val encoded = privateKeyContent ?: return null
        return runCatching {
            KeyFactory.getInstance("RSA").generatePrivate(PKCS8EncodedKeySpec(Base64.getDecoder().decode(encoded)))
        }.onFailure { logger.error("Unable to load license private key", it) }.getOrNull()
    }

    private fun signLicense(license: License): String? {
        val key = privateKey() ?: return null
        return EncoderUtils.encryptSegmentByPrivateKey(jsonEncode(license), key)
    }

    /**
     * Check if the license is valid. Returns ReturnData with license info.
     * Validate the locally installed and remotely-confirmed license state.
     */
    suspend fun isLicenseValid(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val license = getInstalledLicense()
        return returnData.setData(mapOf("valid" to (license?.isValid() == true), "license" to license))
    }

    /**
     * Check license validity (non-route overload for scheduled jobs).
     * Check a license with the licensing service. Network failures keep the
     * previous state rather than granting access.
     */
    suspend fun checkLicense(license: License) {
        webClient.getAbs("https://r.htmake.com/reader3/isLicenseValid?id=${license.id}").timeout(5000).send { response ->
            runCatching {
                val encrypted = response.result()?.bodyAsJsonObject()?.getJsonObject("data")?.getString("result")
                val result = encrypted?.let { JsonObject(com.htmake.reader.utils.decryptData(it)) }
                setLicenseValid(result?.getBoolean("isValid", false) == true)
            }.onFailure {
                setLicenseValid(false)
                logger.info("check license error: {}", it.message)
            }
        }
    }

    fun checkLicenseFeature(feature: String): Boolean {
        return getInstalledLicense()?.isValid() == true
    }

    suspend fun getLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val license = getInstalledLicense(true)
            ?: return returnData.setData(mapOf("active" to false, "valid" to false))
        return returnData.setData(mapOf(
            "active" to license.verified,
            "valid" to license.isValid(),
            "license" to license,
            "message" to if (license.isValid()) "已激活" else "许可证已过期"
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
        val licenseKey = context.bodyAsJson?.getString("content", context.bodyAsJson?.getString("licenseKey", "")) ?: ""
        if (licenseKey.isEmpty()) {
            return returnData.setErrorMsg("请输入许可证密钥")
        }

        val license = decryptToLicense(licenseKey) ?: return returnData.setErrorMsg("许可证密钥错误")
        if (license.expiredAt > 0 && !license.isValid()) return returnData.setErrorMsg("许可证已过期")
        val licenseFile = com.htmake.reader.utils.getStorageFile("data", "license", ext = ".key")
        licenseFile.parentFile.mkdirs()
        licenseFile.writeText(licenseKey)
        setLicenseValid(license.verified)
        return returnData.setData(mapOf("active" to license.verified, "valid" to license.isValid(), "license" to license))
    }

    suspend fun activateLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val content = context.bodyAsJson?.getString("content", "") ?: ""
        val license = decryptToLicense(content) ?: return returnData.setErrorMsg("许可证密钥错误")
        if (license.verified) return returnData.setErrorMsg("许可证已被使用")
        if (license.expiredAt > 0 && !license.isValid()) return returnData.setErrorMsg("许可证已过期")
        license.verified = true
        license.verifyTime = System.currentTimeMillis()
        license.id = UUID.randomUUID().toString()
        val signed = signLicense(license) ?: return returnData.setErrorMsg("未配置许可证私钥")
        val licenseFile = com.htmake.reader.utils.getStorageFile("data", "license", ext = ".key")
        licenseFile.parentFile.mkdirs()
        licenseFile.writeText(signed)
        setLicenseValid(true)
        return returnData.setData(mapOf("result" to signed))
    }

    suspend fun checkLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val license = getInstalledLicense(true) ?: return returnData.setData(mapOf("valid" to false))
        checkLicense(license)
        return returnData.setData(mapOf("valid" to (getInstalledLicense()?.isValid() == true)))
    }

    suspend fun supplyLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        val email = context.bodyAsJson?.getString("email", "") ?: ""
        val code = context.bodyAsJson?.getString("code", "") ?: ""
        val cached = tryCodeCache.getAsString(email)
        tryCodeCache.remove(email)
        if (email.isEmpty() || code != cached) return returnData.setErrorMsg("验证码错误")
        val license = License(host = "*", userMaxLimit = 15, simpleWebExpiredAt = System.currentTimeMillis() + 7 * 86400_000L, instances = 1, type = "trial", code = email)
        val signed = signLicense(license) ?: return returnData.setErrorMsg("未配置许可证私钥")
        return returnData.setData(mapOf("key" to signed))
    }

    suspend fun generateKeys(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val keyPair = EncoderUtils.generateKeys()
        return returnData.setData(mapOf(
            "publicKey" to Base64.getEncoder().encodeToString(keyPair.public.encoded),
            "privateKey" to Base64.getEncoder().encodeToString(keyPair.private.encoded)
        ))
    }

    suspend fun generateLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val body = context.bodyAsJson ?: return returnData.setErrorMsg("参数错误")
        val host = body.getString("host", "")
        if (host.isEmpty()) return returnData.setErrorMsg("请输入域名")
        val license = License(host, body.getInteger("userMaxLimit", 15), body.getLong("expiredAt", 0L), body.getBoolean("openApi", false), body.getLong("simpleWebExpiredAt", 0L), body.getInteger("instances", 1), body.getString("type", ""), code = body.getString("code", ""))
        val signed = signLicense(license) ?: return returnData.setErrorMsg("未配置许可证私钥")
        return returnData.setData(mapOf("host" to host, "key" to signed))
    }

    suspend fun isHostValid(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val host = context.request().getParam("host") ?: context.bodyAsJson?.getString("host", "") ?: ""
        return returnData.setData(mapOf("isValid" to (getInstalledLicense()?.validHost(host) == true)))
    }

    suspend fun decryptLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            return returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用")
        }
        if (!checkManagerAuth(context)) {
            return returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码")
        }
        val content = context.bodyAsJson?.getString("content", "") ?: ""
        val license = decryptToLicense(content) ?: return returnData.setErrorMsg("许可证密钥错误")
        return returnData.setData(license)
    }

    suspend fun sendCodeToEmail(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val email = context.request().getParam("email") ?: context.bodyAsJson?.getString("email", "") ?: ""
        if (!validateEmail(email)) return returnData.setErrorMsg("邮箱错误")
        if (!tryCodeCache.getAsString(email).isNullOrEmpty()) return returnData.setData("", "验证码仍在有效期内")
        val code = UUID.randomUUID().toString().substring(0, 6)
        tryCodeCache.put(email, code, 900)
        return if (sendEmail(email, "Reader Kindle端的试用申请验证", "您的验证码是: $code，15分钟内有效，请勿回复")) {
            returnData.setData("", "请查收邮件")
        } else {
            tryCodeCache.remove(email)
            returnData.setErrorMsg("邮件发送失败")
        }
    }
}
