package com.htmake.reader.api.controller

import io.vertx.ext.web.RoutingContext
import io.vertx.core.http.HttpMethod
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
import com.htmake.reader.utils.asJsonArray
import com.htmake.reader.utils.toDataClass
import com.htmake.reader.utils.toMap
import com.htmake.reader.utils.SpringContextUtils
import com.htmake.reader.utils.jsonEncode
import com.htmake.reader.utils.decryptToLicense
import com.htmake.reader.utils.getInstalledLicense
import com.htmake.reader.utils.setLicenseValid
import com.htmake.reader.utils.validateEmail
import com.htmake.reader.utils.sendEmail
import com.htmake.reader.utils.success
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
        return signPayload(license)
    }

    private fun signPayload(payload: Any): String? {
        val key = privateKey() ?: return null
        return EncoderUtils.encryptSegmentByPrivateKey(jsonEncode(payload), key)
    }

    suspend fun isLicenseValid(context: RoutingContext): ReturnData {
        val id = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getString("id") ?: ""
        } else {
            context.queryParam("id").firstOrNull() ?: ""
        }
        val activeLicenseList = asJsonArray(getStorage("data", "activeLicense")) ?: JsonArray()
        var activeLicense: ActiveLicense? = null
        var activeLicenseIndex = -1
        for (index in 0 until activeLicenseList.size()) {
            val candidate = activeLicenseList.getJsonObject(index).mapTo(ActiveLicense::class.java)
            if (candidate.id == id) {
                activeLicense = candidate
                activeLicenseIndex = index
                break
            }
        }

        val result = linkedMapOf<String, Any>()
        val ip = context.request().getHeader("X-Real-IP").takeUnless { it.isNullOrEmpty() }
            ?: context.request().remoteAddress()?.host().orEmpty()
        if (activeLicense == null) {
            result["isValid"] = false
            result["errorMsg"] = "密钥未激活"
        } else {
            result["isValid"] = activeLicense.verified
            result["errorMsg"] = activeLicense.errorMsg
            val lastOnlineTime = activeLicense.lastOnlineTime
            if (lastOnlineTime != null && System.currentTimeMillis() < lastOnlineTime + 600_000 && ip != activeLicense.lastOnlineIp) {
                result["repeat"] = mapOf(
                    "lastOnlineTime" to lastOnlineTime,
                    "lastOnlineIp" to activeLicense.lastOnlineIp
                )
            }
            activeLicense.lastOnlineTime = System.currentTimeMillis()
            activeLicense.lastOnlineIp = ip
            activeLicenseList.set(activeLicenseIndex, JsonObject.mapFrom(activeLicense))
            saveStorage("data", "activeLicense", value = activeLicenseList)
        }
        val signed = signPayload(result) ?: return ReturnData().setErrorMsg("未配置许可证私钥")
        return ReturnData().setData(mapOf("result" to signed))
    }

    suspend fun checkLicense(license: License) {
        webClient.getAbs("https://r.htmake.com/reader3/isLicenseValid?id=${license.id}").timeout(5000).send { response ->
            runCatching {
                val encrypted = response.result()?.bodyAsJsonObject()?.getJsonObject("data")?.getString("result")
                val result = encrypted?.let { JsonObject(com.htmake.reader.utils.decryptData(it)) }
                val isValid = result?.getBoolean("isValid") ?: true
                setLicenseValid(isValid)
                if (!isValid) {
                    logger.info("密钥错误：{}", result?.getString("errorMsg") ?: "")
                }
            }.onFailure {
                logger.info("check license error: {}", it.message)
            }
        }
    }

    fun checkLicenseFeature(feature: String): Boolean {
        return getInstalledLicense()?.isValid() == true
    }

    suspend fun getLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        return returnData.setData(mapOf("license" to getInstalledLicense()))
    }

    suspend fun importLicense(context: RoutingContext) {
        val returnData = ReturnData()
        if (!checkAuth(context)) {
            context.success(returnData.setData("NEED_LOGIN").setErrorMsg("请登录后使用"))
            return
        }
        if (!checkManagerAuth(context)) {
            context.success(returnData.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码"))
            return
        }
        val content = context.bodyAsJson?.getString("content") ?: ""
        if (content.isEmpty()) {
            context.success(returnData.setErrorMsg("请输入密钥"))
            return
        }

        webClient.postAbs("https://r.htmake.com/reader3/activateLicense")
            .timeout(5000)
            .sendJsonObject(JsonObject().put("content", content)) { response ->
                runCatching {
                    val payload = response.result()?.bodyAsJsonObject()
                        ?: throw response.cause() ?: Exception("密钥激活失败")
                    if (payload.getBoolean("isSuccess", false) != true) {
                        throw Exception(payload.getString("errorMsg") ?: "密钥激活失败")
                    }
                    val licenseKey = payload.getJsonObject("data")?.getString("result")
                        ?: throw Exception("密钥错误")
                    val license = decryptToLicense(licenseKey) ?: throw Exception("密钥错误")
                    if (!license.validHost(context.request().host())) {
                        throw Exception("密钥授权域名错误")
                    }
                    licenseKey to license
                }.onSuccess { (licenseKey, license) ->
                    val licenseFile = com.htmake.reader.utils.getStorageFile("data", "license", ext = ".key")
                    licenseFile.parentFile.mkdirs()
                    licenseFile.writeText(licenseKey)
                    context.success(returnData.setData(mapOf("license" to license)))
                }.onFailure { error ->
                    logger.info("import license error: {}", error.message)
                    context.success(returnData.setErrorMsg(error.message ?: "密钥激活错误"))
                }
            }
    }

    suspend fun activateLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val content = context.bodyAsJson?.getString("content", "") ?: ""
        if (content.isEmpty()) return returnData.setErrorMsg("请输入密钥")
        val license = decryptToLicense(content) ?: return returnData.setErrorMsg("密钥错误")
        if (license.verified) return returnData.setErrorMsg("许可证已被使用")
        val activeLicenseList = asJsonArray(getStorage("data", "activeLicense")) ?: JsonArray()
        var activeTimes = 0
        for (index in 0 until activeLicenseList.size()) {
            val activeLicense = activeLicenseList.getJsonObject(index).mapTo(ActiveLicense::class.java)
            if (activeLicense.type == license.type && activeLicense.code == license.code) {
                activeTimes++
            }
        }
        if (activeTimes >= license.instances) return returnData.setErrorMsg("密钥已超过最大使用次数")

        val ip = context.request().getHeader("X-Real-IP").takeUnless { it.isNullOrEmpty() }
            ?: context.request().remoteAddress()?.host().orEmpty()
        license.verified = true
        license.verifyTime = System.currentTimeMillis()
        license.id = UUID.randomUUID().toString()
        val activeLicense = license.toActiveLicense().apply {
            activeOrder = activeTimes + 1
            activeTime = System.currentTimeMillis()
            activeIp = ip
            activeEmail = ""
        }
        activeLicenseList.add(JsonObject.mapFrom(activeLicense))
        logger.info("activeLicenseList: {}", activeLicenseList)
        saveStorage("data", "activeLicense", value = activeLicenseList)
        val signed = signLicense(license) ?: return returnData.setErrorMsg("未配置许可证私钥")
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
        val email = context.bodyAsJson?.getString("email", "") ?: ""
        val code = context.bodyAsJson?.getString("code", "") ?: ""
        if (email.isEmpty() || code.isEmpty()) return returnData.setErrorMsg("参数错误")
        val cached = tryCodeCache.getAsString(email)
        tryCodeCache.remove(email)
        if (code != cached) return returnData.setErrorMsg("验证码错误")
        val license = License(host = "*", userMaxLimit = 15, simpleWebExpiredAt = System.currentTimeMillis() + 7 * 86400_000L, instances = 1, type = "trial", code = email)
        val signed = signLicense(license) ?: return returnData.setErrorMsg("未配置许可证私钥")
        return returnData.setData(mapOf("key" to signed))
    }

    suspend fun generateKeys(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val keyPair = EncoderUtils.generateKeys()
        return returnData.setData(mapOf(
            "publicKey" to Base64.getEncoder().encodeToString(keyPair.public.encoded),
            "privateKey" to Base64.getEncoder().encodeToString(keyPair.private.encoded)
        ))
    }

    suspend fun generateLicense(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val body = context.bodyAsJson
        fun value(name: String): String? = if (context.request().method() == HttpMethod.POST) {
            body?.getString(name)
        } else {
            context.queryParam(name).firstOrNull()
        }
        val host = value("host") ?: ""
        if (host.isEmpty()) return returnData.setErrorMsg("请输入域名")
        if (value("key") != "Pvkp7tMQJpi4kWBE") return returnData.setErrorMsg("参数错误")
        val expiredAt = if (context.request().method() == HttpMethod.POST) body?.getLong("expiredAt") ?: 0L else value("expiredAt")?.toLong() ?: 0L
        val userMaxLimit = if (context.request().method() == HttpMethod.POST) body?.getInteger("userMaxLimit") ?: 15 else value("userMaxLimit")?.toInt() ?: 15
        val openApi = if (context.request().method() == HttpMethod.POST) body?.getBoolean("openApi") ?: false else value("openApi")?.toBoolean() ?: false
        val simpleWebExpiredAt = if (context.request().method() == HttpMethod.POST) body?.getLong("simpleWebExpiredAt") ?: 0L else value("simpleWebExpiredAt")?.toLong() ?: 0L
        val instances = if (context.request().method() == HttpMethod.POST) body?.getInteger("instances") ?: 1 else value("instances")?.toInt() ?: 1
        val license = License(
            host = host,
            userMaxLimit = userMaxLimit,
            expiredAt = expiredAt,
            openApi = openApi,
            simpleWebExpiredAt = simpleWebExpiredAt,
            instances = instances,
            type = value("type") ?: "",
            code = value("code") ?: ""
        )
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
        val content = context.bodyAsJson?.getString("content", "") ?: ""
        if (content.isEmpty()) return returnData.setErrorMsg("请输入密钥")
        val license = decryptToLicense(content) ?: return returnData.setErrorMsg("密钥错误")
        return returnData.setData(license)
    }

    suspend fun sendCodeToEmail(context: RoutingContext): ReturnData {
        val returnData = ReturnData()
        val email = if (context.request().method() == HttpMethod.POST) {
            context.bodyAsJson?.getString("email") ?: ""
        } else {
            context.queryParam("email").firstOrNull() ?: ""
        }
        if (email.isEmpty()) return returnData.setErrorMsg("邮箱错误")
        if (!validateEmail(email)) {
            return returnData.setErrorMsg("仅支持 163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud 等邮箱")
        }
        val activeLicenseList = asJsonArray(getStorage("data", "activeLicense")) ?: JsonArray()
        if (activeLicenseList.any { item ->
                val activeLicense = item as? JsonObject
                activeLicense?.getString("type") == "trial" && activeLicense.getString("code") == email
            }) {
            return returnData.setErrorMsg("该邮箱已被使用")
        }
        if (!tryCodeCache.getAsString(email).isNullOrEmpty()) {
            return returnData.setData("", "您的验证码仍在有效期内，请勿重复获取")
        }
        val code = UUID.randomUUID().toString().substring(0, 6)
        tryCodeCache.put(email, code, 900)
        sendEmail(email, "Reader Kindle端的试用申请验证", "您正在申请Reader Kindle端的试用，验证码是: $code，15分钟内有效，请勿回复")
        return returnData.setData("", "请查收邮件")
    }
}
