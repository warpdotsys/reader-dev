package com.htmake.reader.utils

import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import java.net.IDN
import java.net.URI
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.util.Locale

/** Application-layer guard for browser navigations and HTTP(S) subresources. */
internal open class BrowserNetworkPolicy(
    private val allowPrivateNetworks: Boolean = false,
    private val resolve: (String) -> Array<InetAddress> = { InetAddress.getAllByName(it) }
) {
    fun requireDocumentUrl(value: String) {
        val uri = parse(value)
        val scheme = uri.scheme?.lowercase(Locale.ROOT)
        if (scheme != "http" && scheme != "https") {
            throw BrowserNetworkPolicyViolation("本地 WebView 仅允许 HTTP/HTTPS 页面", BrowserNetworkBlockReason.UNSUPPORTED_SCHEME)
        }
        resolveRequestTarget(value)
    }

    fun requireRequestUrl(value: String) {
        resolveRequestTarget(value)
    }

    /** Resolve once, validate the whole DNS answer set, and hand the pinned addresses to the egress socket. */
    internal open fun resolveRequestTarget(value: String): BrowserNetworkTarget? {
        val uri = parse(value)
        when (uri.scheme?.lowercase(Locale.ROOT)) {
            "http", "https", "ws", "wss" -> return requireNetworkTarget(uri)
            // These schemes do not open a network connection. Do not allow file:, ftp:, or custom schemes.
            "about", "blob", "data" -> return null
            else -> throw BrowserNetworkPolicyViolation("本地 WebView 已阻止不支持的资源协议", BrowserNetworkBlockReason.UNSUPPORTED_SCHEME)
        }
    }

    private fun requireNetworkTarget(uri: URI): BrowserNetworkTarget {
        if (uri.rawUserInfo != null) {
            throw BrowserNetworkPolicyViolation("本地 WebView 不接受包含账号信息的 URL", BrowserNetworkBlockReason.URL_CREDENTIALS)
        }
        if (uri.port == 0 || uri.port > 65535) {
            throw BrowserNetworkPolicyViolation("本地 WebView URL 端口无效", BrowserNetworkBlockReason.INVALID_PORT)
        }
        val authority = uri.rawAuthority
            ?: throw BrowserNetworkPolicyViolation("本地 WebView URL 缺少有效主机名", BrowserNetworkBlockReason.INVALID_HOST)
        val rawHost = uri.host?.removePrefix("[")?.removeSuffix("]") ?: run {
            val withoutUserInfo = authority.substringAfterLast('@')
            if (withoutUserInfo.startsWith("[")) {
                withoutUserInfo.substringAfter("[").substringBefore("]")
            } else {
                val portSeparator = withoutUserInfo.lastIndexOf(':')
                if (portSeparator > 0 && withoutUserInfo.substring(portSeparator + 1).all { it in '0'..'9' }) {
                    withoutUserInfo.substring(0, portSeparator)
                } else withoutUserInfo
            }
        }
        val normalizedHost = try {
            if (rawHost.contains(':')) rawHost.trimEnd('.').lowercase(Locale.ROOT)
            else IDN.toASCII(rawHost.trimEnd('.'), IDN.USE_STD3_ASCII_RULES).lowercase(Locale.ROOT)
        } catch (_: IllegalArgumentException) {
            throw BrowserNetworkPolicyViolation("本地 WebView URL 主机名无效", BrowserNetworkBlockReason.INVALID_HOST)
        }
        val host = normalizedHost.takeIf { it.isNotBlank() }
            ?: throw BrowserNetworkPolicyViolation("本地 WebView URL 缺少有效主机名", BrowserNetworkBlockReason.INVALID_HOST)
        if (host.contains('%')) {
            throw BrowserNetworkPolicyViolation.forHost("本地 WebView 不接受带区域标识的 IP 地址", BrowserNetworkBlockReason.SCOPED_IP, host)
        }
        if (!allowPrivateNetworks && (host == "localhost" || host.endsWith(".localhost") ||
            host == "local" || host.endsWith(".local") ||
            host.endsWith(".internal") || host.endsWith(".home.arpa"))) {
            throw BrowserNetworkPolicyViolation.forHost("本地 WebView 已阻止本机或内网主机名", BrowserNetworkBlockReason.LOCAL_HOST, host)
        }

        val addresses = try {
            resolve(host)
        } catch (_: Exception) {
            throw BrowserNetworkPolicyViolation.forHost("本地 WebView 无法验证目标主机地址", BrowserNetworkBlockReason.DNS_FAILURE, host)
        }
        if (addresses.isEmpty()) {
            throw BrowserNetworkPolicyViolation.forHost("本地 WebView 无法验证空 DNS 结果", BrowserNetworkBlockReason.EMPTY_DNS, host)
        }
        if (!allowPrivateNetworks) {
            val denied = addresses.filter { !isPublicInternetAddress(it) }
            if (denied.isNotEmpty()) {
                // 198.18.0.0/15 can be a proxy's Fake-IP answer, but is still a
                // reserved benchmark range. Report it; never turn it into an allow rule.
                val reason = if (denied.all { address ->
                    val bytes = address.address
                    address is Inet4Address && (bytes[0].toInt() and 0xff) == 198 &&
                        (bytes[1].toInt() and 0xff) in 18..19
                }) BrowserNetworkBlockReason.BENCHMARK_RANGE else BrowserNetworkBlockReason.NON_PUBLIC_ADDRESS
                throw BrowserNetworkPolicyViolation.forHost("本地 WebView 已阻止本机、内网或保留地址", reason, host)
            }
        }
        val port = uri.port.takeIf { it > 0 } ?: when (uri.scheme.lowercase(Locale.ROOT)) {
            "https", "wss" -> 443
            else -> 80
        }
        return BrowserNetworkTarget(uri, host, port, addresses.toList())
    }

    private fun parse(value: String): URI = try {
        URI(value)
    } catch (_: Exception) {
        throw BrowserNetworkPolicyViolation("本地 WebView 收到无效 URL", BrowserNetworkBlockReason.MALFORMED_URL)
    }

    private fun isPublicInternetAddress(address: InetAddress): Boolean {
        if (address.isAnyLocalAddress || address.isLoopbackAddress || address.isLinkLocalAddress ||
            address.isSiteLocalAddress || address.isMulticastAddress) return false
        val bytes = address.address.map { it.toInt() and 0xff }
        return when (address) {
            is Inet4Address -> isPublicIpv4(bytes)
            is Inet6Address -> isPublicIpv6(bytes)
            else -> false
        }
    }

    private fun isPublicIpv4(b: List<Int>): Boolean {
        if (b.size != 4) return false
        val first = b[0]
        val second = b[1]
        val third = b[2]
        return when {
            first == 0 || first == 10 || first == 127 || first >= 224 -> false
            first == 100 && second in 64..127 -> false // Shared address space (RFC 6598).
            first == 169 && second == 254 -> false
            first == 172 && second in 16..31 -> false
            first == 192 && (second == 0 || second == 168) -> false
            first == 192 && second == 88 && third == 99 -> false
            first == 198 && (second in 18..19 || second == 51 && third == 100) -> false
            first == 203 && second == 0 && third == 113 -> false
            else -> true
        }
    }

    private fun isPublicIpv6(b: List<Int>): Boolean {
        if (b.size != 16) return false
        // Permit global unicast only. This excludes ULA, link-local, multicast,
        // IPv4-mapped/NAT64 and other special-purpose ranges by default.
        if ((b[0] and 0xe0) != 0x20) return false // 2000::/3
        if (b[0] == 0x20 && b[1] == 0x01 && b[2] == 0x0d && b[3] == 0xb8) return false // documentation
        if (b[0] == 0x20 && b[1] == 0x01 && b[2] == 0x00 && b[3] == 0x00) return false // Teredo
        if (b[0] == 0x20 && b[1] == 0x02) return false // 6to4
        return true
    }
}

internal data class BrowserNetworkTarget(
    val uri: URI,
    val host: String,
    val port: Int,
    val addresses: List<InetAddress>
)

internal enum class BrowserNetworkBlockReason {
    UNKNOWN, MALFORMED_URL, UNSUPPORTED_SCHEME, URL_CREDENTIALS, INVALID_PORT, INVALID_HOST,
    SCOPED_IP, LOCAL_HOST, DNS_FAILURE, EMPTY_DNS, BENCHMARK_RANGE, NON_PUBLIC_ADDRESS
}

/** Diagnostic fields deliberately contain neither a URL nor even its raw hostname. */
internal class BrowserNetworkPolicyViolation(
    message: String,
    val reason: BrowserNetworkBlockReason = BrowserNetworkBlockReason.UNKNOWN,
    val hostFingerprint: String? = null
) : IllegalArgumentException(message + if (reason == BrowserNetworkBlockReason.UNKNOWN) "" else
    " [reason=${reason.name}${hostFingerprint?.let { "; hostSha256=$it" }.orEmpty()}]") {
    fun withRenderContext(message: String) = BrowserNetworkPolicyViolation(message, reason, hostFingerprint)

    companion object {
        fun forHost(message: String, reason: BrowserNetworkBlockReason, normalizedHost: String): BrowserNetworkPolicyViolation {
            val fingerprint = MessageDigest.getInstance("SHA-256")
                .digest(normalizedHost.toByteArray(StandardCharsets.UTF_8)).take(8)
                .joinToString("") { "%02x".format(it.toInt() and 0xff) }
            return BrowserNetworkPolicyViolation(message, reason, fingerprint)
        }
    }
}
