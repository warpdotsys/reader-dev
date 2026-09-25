package com.htmake.reader.utils

import java.net.URI
import java.net.URLDecoder
import java.nio.charset.StandardCharsets
import java.util.Base64
import java.util.Locale

internal data class BrowserUpstreamProxy(
    val protocol: Protocol,
    val host: String,
    val port: Int,
    val username: String? = null,
    val password: String? = null
) {
    enum class Protocol { HTTP, SOCKS4, SOCKS5 }

    val endpointUrl: String
        get() = "http://${formatHost(host)}:$port/"

    val basicAuthorization: String?
        get() = if (protocol != Protocol.HTTP || username == null || password == null) null else {
            val bytes = "$username:$password".toByteArray(StandardCharsets.UTF_8)
            "Basic ${Base64.getEncoder().encodeToString(bytes)}"
        }

    override fun toString(): String =
        "BrowserUpstreamProxy(protocol=$protocol, host=$host, port=$port, credentials=${username != null})"

    companion object {
        private val legacyAuthenticated = Regex(
            "^(http|socks4|socks5)://(\\[[^]]+]|[^:/@]+):(\\d{2,5})@([^@]*)@(.+)$",
            RegexOption.IGNORE_CASE
        )

        fun parse(value: String): BrowserUpstreamProxy {
            val legacy = legacyAuthenticated.matchEntire(value.trim())
            if (legacy != null) {
                return create(
                    legacy.groupValues[1], legacy.groupValues[2], legacy.groupValues[3].toInt(),
                    legacy.groupValues[4], legacy.groupValues[5]
                )
            }

            val uri = try {
                URI(value.trim())
            } catch (_: Exception) {
                throw IllegalArgumentException("书源上游代理地址无效")
            }
            val scheme = uri.scheme?.lowercase(Locale.ROOT)
                ?: throw IllegalArgumentException("书源上游代理协议缺失")
            val host = uri.host?.removePrefix("[")?.removeSuffix("]")
                ?: throw IllegalArgumentException("书源上游代理主机名无效")
            val port = uri.port
            if (port !in 1..65535 || uri.rawQuery != null || uri.rawFragment != null ||
                !uri.rawPath.isNullOrEmpty() && uri.rawPath != "/") {
                throw IllegalArgumentException("书源上游代理端口或路径无效")
            }

            val (username, password) = uri.rawUserInfo?.let { raw ->
                val separator = raw.indexOf(':')
                if (separator <= 0) throw IllegalArgumentException("书源上游代理认证信息无效")
                decode(raw.substring(0, separator)) to decode(raw.substring(separator + 1))
            } ?: (null to null)
            return create(scheme, host, port, username, password)
        }

        private fun create(scheme: String, host: String, port: Int, username: String?, password: String?) =
            BrowserUpstreamProxy(
                protocol = when (scheme.lowercase(Locale.ROOT)) {
                    "http" -> Protocol.HTTP
                    "socks4" -> Protocol.SOCKS4
                    "socks5" -> Protocol.SOCKS5
                    else -> throw IllegalArgumentException("书源上游代理协议不受支持")
                },
                host = host.removePrefix("[").removeSuffix("]"),
                port = port.also {
                    if (it !in 1..65535) throw IllegalArgumentException("书源上游代理端口无效")
                },
                username = username?.takeIf { it.isNotEmpty() },
                password = password?.takeIf { it.isNotEmpty() }
            ).also {
                if ((it.username == null) != (it.password == null)) {
                    throw IllegalArgumentException("书源上游代理认证信息不完整")
                }
            }

        private fun decode(value: String): String = URLDecoder.decode(
            value.replace("+", "%2B"), StandardCharsets.UTF_8.name()
        )

        private fun formatHost(value: String): String = if (value.contains(':')) "[$value]" else value
    }
}
