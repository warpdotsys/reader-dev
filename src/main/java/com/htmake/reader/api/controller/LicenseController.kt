package com.htmake.reader.api.controller

import com.htmake.reader.ReaderApplication
import com.htmake.reader.api.ReturnData
import com.htmake.reader.entity.License
import com.htmake.reader.utils.decryptLicenseData
import com.htmake.reader.utils.decryptToLicense
import com.htmake.reader.utils.getInstalledLicense
import com.htmake.reader.utils.saveStorage
import com.htmake.reader.utils.setLicenseValid
import com.htmake.reader.utils.success
import io.vertx.core.json.JsonObject
import io.vertx.ext.web.RoutingContext
import io.vertx.ext.web.client.WebClient
import io.vertx.ext.web.client.WebClientOptions
import mu.KotlinLogging
import java.net.URLEncoder
import java.time.Instant
import java.time.LocalDateTime
import java.time.ZoneId
import kotlin.coroutines.CoroutineContext

private val logger = KotlinLogging.logger {}

class LicenseController(coroutineContext: CoroutineContext) : BaseController(coroutineContext) {

    private val webClient: WebClient by lazy {
        val options = WebClientOptions()
        options.isTryUseCompression = true
        options.isFollowRedirects = false
        options.isTrustAll = false
        options.isVerifyHost = true
        WebClient.create(ReaderApplication.vertx(), options)
    }

    private fun centerUrl(path: String): String = appConfig.licenseServerUrl.trimEnd('/') + path

    suspend fun getLicense(context: RoutingContext): ReturnData =
        ReturnData().setData(mapOf("license" to getInstalledLicense()))

    suspend fun importLicense(context: RoutingContext) {
        val result = ReturnData()
        if (!checkAuth(context)) {
            context.success(result.setData("NEED_LOGIN").setErrorMsg("请登录后使用"))
            return
        }
        if (!checkManagerAuth(context)) {
            context.success(result.setData("NEED_SECURE_KEY").setErrorMsg("请输入管理密码"))
            return
        }
        val content = context.bodyAsJson?.getString("content").orEmpty()
        if (content.isEmpty()) {
            context.success(result.setErrorMsg("请输入密钥"))
            return
        }
        if (appConfig.licenseServerUrl.isBlank()) {
            context.success(result.setErrorMsg("未配置许可证中心"))
            return
        }

        webClient.postAbs(centerUrl("/reader3/activateLicense"))
            .timeout(5000)
            .sendJsonObject(JsonObject().put("content", content)) { response ->
                runCatching {
                    if (response.failed()) throw response.cause() ?: IllegalStateException("密钥激活失败")
                    val payload = response.result().bodyAsJsonObject()
                    if (!payload.getBoolean("isSuccess", false)) {
                        throw IllegalStateException(payload.getString("errorMsg", "密钥激活失败"))
                    }
                    val licenseKey = payload.getJsonObject("data")?.getString("result")
                        ?: throw IllegalStateException("密钥错误")
                    val license = decryptToLicense(licenseKey) ?: throw IllegalStateException("密钥签名无法验证")
                    if (!license.validHost(context.request().host())) throw IllegalStateException("密钥授权域名错误")
                    licenseKey to license
                }.onSuccess { (licenseKey, license) ->
                    saveStorage("data", "license", value = licenseKey, ext = ".key")
                    setLicenseValid(true)
                    context.success(result.setData(mapOf("license" to license)))
                }.onFailure { error ->
                    logger.info("许可证激活失败: {}", error.message)
                    context.success(result.setErrorMsg(error.message ?: "密钥激活错误"))
                }
            }
    }

    suspend fun checkLicense(license: License) {
        if (!appConfig.licenseCheckEnabled || appConfig.licenseServerUrl.isBlank()) return
        val id = URLEncoder.encode(license.id, "UTF-8")
        webClient.getAbs(centerUrl("/reader3/isLicenseValid?id=$id"))
            .timeout(5000)
            .send { response ->
                runCatching {
                    if (response.failed()) throw response.cause() ?: IllegalStateException("许可证中心无响应")
                    val payload = response.result().bodyAsJsonObject()
                    if (!payload.getBoolean("isSuccess", false)) {
                        throw IllegalStateException(payload.getString("errorMsg", "许可证校验失败"))
                    }
                    val signed = payload.getJsonObject("data")?.getString("result")
                        ?: throw IllegalStateException("许可证校验响应缺少签名")
                    JsonObject(decryptLicenseData(signed) ?: throw IllegalStateException("许可证校验签名无效"))
                }.onSuccess { onlineResult ->
                    val isValid = onlineResult.getBoolean("isValid", true)
                    setLicenseValid(isValid)
                    if (!isValid) logger.info("密钥错误：{}", onlineResult.getString("errorMsg", ""))
                    onlineResult.getJsonObject("repeat")?.let { repeat ->
                        logger.info(
                            "请勿重复使用授权，上次检查时间：{}，上次检查ip：{}",
                            LocalDateTime.ofInstant(
                                Instant.ofEpochMilli(repeat.getLong("lastOnlineTime", 0L)),
                                ZoneId.systemDefault()
                            ),
                            repeat.getString("lastOnlineIp", "")
                        )
                    }
                }.onFailure { error ->
                    // Preserve the original fail-open network behaviour. A valid,
                    // signed isValid=false response is the only way to revoke.
                    logger.info("许可证在线校验失败（保留当前状态）: {}", error.message)
                }
            }
    }

    suspend fun isHostValid(context: RoutingContext): ReturnData {
        val host = context.bodyAsJson?.getString("host")
            ?: context.queryParam("host").firstOrNull().orEmpty()
        return ReturnData().setData(mapOf("isValid" to getInstalledLicense().validHost(host)))
    }

    suspend fun decryptLicense(context: RoutingContext): ReturnData {
        val content = context.bodyAsJson?.getString("content").orEmpty()
        if (content.isEmpty()) return ReturnData().setErrorMsg("请输入密钥")
        return decryptToLicense(content)?.let { ReturnData().setData(it) }
            ?: ReturnData().setErrorMsg("密钥错误")
    }

    suspend fun sendCodeToEmail(context: RoutingContext) = proxyTrialRequest(context, "/reader3/sendCodeToEmail")

    suspend fun supplyLicense(context: RoutingContext) = proxyTrialRequest(context, "/reader3/supplyLicense")

    private fun proxyTrialRequest(context: RoutingContext, path: String) {
        val result = ReturnData()
        if (appConfig.licenseServerUrl.isBlank()) {
            context.success(result.setErrorMsg("未配置许可证中心"))
            return
        }
        webClient.postAbs(centerUrl(path)).timeout(5000)
            .sendJsonObject(context.bodyAsJson ?: JsonObject()) { response ->
                runCatching {
                    if (response.failed()) throw response.cause() ?: IllegalStateException("许可证中心无响应")
                    response.result().bodyAsJsonObject()
                }.onSuccess { payload ->
                    if (payload.getBoolean("isSuccess", false)) {
                        context.success(result.setData(payload.getValue("data") ?: "", payload.getString("errorMsg", "")))
                    } else {
                        context.success(result.setErrorMsg(payload.getString("errorMsg", "请求失败")))
                    }
                }.onFailure { error ->
                    context.success(result.setErrorMsg(error.message ?: "许可证中心无响应"))
                }
            }
    }
}
